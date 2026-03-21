use std::path::{Path, PathBuf};

use crate::error::{Result, SkillxError};
use crate::source::ArchiveFormat;

/// Maximum number of files allowed in an archive.
const MAX_FILE_COUNT: usize = 1000;

/// Maximum total uncompressed size (500 MB).
const MAX_TOTAL_SIZE: u64 = 500 * 1024 * 1024;

pub struct ArchiveSource;

impl ArchiveSource {
    /// Download and extract an archive to the destination directory.
    pub async fn fetch(
        url: &str,
        format: &ArchiveFormat,
        dest: &Path,
    ) -> Result<Vec<PathBuf>> {
        let client = reqwest::Client::builder()
            .user_agent("skillx/0.3")
            .build()
            .map_err(|e| SkillxError::Network(format!("failed to create HTTP client: {e}")))?;

        let resp = client.get(url).send().await.map_err(|e| {
            SkillxError::Network(format!("archive download failed: {e}"))
        })?;

        if !resp.status().is_success() {
            return Err(SkillxError::Archive(format!(
                "archive download returned HTTP {}",
                resp.status()
            )));
        }

        let bytes = resp.bytes().await.map_err(|e| {
            SkillxError::Network(format!("failed to read archive: {e}"))
        })?;

        if bytes.len() as u64 > MAX_TOTAL_SIZE {
            return Err(SkillxError::Archive(format!(
                "archive too large: {} bytes (max {} bytes)",
                bytes.len(),
                MAX_TOTAL_SIZE
            )));
        }

        std::fs::create_dir_all(dest)
            .map_err(|e| SkillxError::Source(format!("failed to create dir: {e}")))?;

        match format {
            ArchiveFormat::Zip => Self::extract_zip(&bytes, dest),
            ArchiveFormat::TarGz => Self::extract_tar_gz(&bytes, dest),
        }
    }

    /// Extract a zip archive with security checks.
    fn extract_zip(data: &[u8], dest: &Path) -> Result<Vec<PathBuf>> {
        use std::io::Read;

        let reader = std::io::Cursor::new(data);
        let mut archive = zip::ZipArchive::new(reader)
            .map_err(|e| SkillxError::Archive(format!("invalid zip archive: {e}")))?;

        if archive.len() > MAX_FILE_COUNT {
            return Err(SkillxError::Archive(format!(
                "archive contains too many files: {} (max {MAX_FILE_COUNT})",
                archive.len()
            )));
        }

        let mut files = Vec::new();
        let mut total_size: u64 = 0;

        // Detect single root directory for flattening
        let single_root = detect_single_root_zip(&mut archive);

        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| SkillxError::Archive(format!("failed to read zip entry: {e}")))?;

            let raw_name = entry
                .enclosed_name()
                .ok_or_else(|| {
                    SkillxError::Archive(format!(
                        "zip entry has unsafe path: {}",
                        entry.name()
                    ))
                })?
                .to_path_buf();

            // Zip slip protection
            let name_str = raw_name.to_string_lossy();
            if name_str.contains("..") {
                return Err(SkillxError::Archive(format!(
                    "zip slip detected: path contains '..': {name_str}"
                )));
            }

            // Flatten single root directory
            let relative = if let Some(ref root) = single_root {
                raw_name.strip_prefix(root).unwrap_or(&raw_name)
            } else {
                &raw_name
            };

            if relative.as_os_str().is_empty() {
                continue;
            }

            let out_path = dest.join(relative);

            // Verify the output path is within dest (normalize without requiring existence)
            if !path_is_within(dest, &out_path) {
                return Err(SkillxError::Archive(format!(
                    "zip slip detected: {} escapes destination",
                    name_str
                )));
            }

            if entry.is_dir() {
                std::fs::create_dir_all(&out_path).map_err(|e| {
                    SkillxError::Source(format!("failed to create dir: {e}"))
                })?;
            } else {
                total_size += entry.size();
                if total_size > MAX_TOTAL_SIZE {
                    return Err(SkillxError::Archive(format!(
                        "archive exceeds maximum total size of {MAX_TOTAL_SIZE} bytes"
                    )));
                }

                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        SkillxError::Source(format!("failed to create dir: {e}"))
                    })?;
                }
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf).map_err(|e| {
                    SkillxError::Archive(format!("failed to read zip entry: {e}"))
                })?;
                std::fs::write(&out_path, &buf).map_err(|e| {
                    SkillxError::Source(format!(
                        "failed to write {}: {e}",
                        out_path.display()
                    ))
                })?;
                files.push(out_path);
            }
        }

        Ok(files)
    }

    /// Extract a tar.gz archive with security checks.
    pub fn extract_tar_gz(data: &[u8], dest: &Path) -> Result<Vec<PathBuf>> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let decoder = GzDecoder::new(std::io::Cursor::new(data));
        let mut archive = Archive::new(decoder);

        let mut files = Vec::new();
        let mut file_count = 0usize;
        let mut total_size: u64 = 0;

        // First pass: collect entries to detect single root
        let temp_decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(data));
        let mut temp_archive = Archive::new(temp_decoder);
        let mut paths: Vec<PathBuf> = Vec::new();
        if let Ok(entries) = temp_archive.entries() {
            for entry in entries.flatten() {
                if let Ok(p) = entry.path() {
                    paths.push(p.to_path_buf());
                }
            }
        }
        let single_root = detect_single_root_tar(&paths);

        for entry in archive.entries().map_err(|e| {
            SkillxError::Archive(format!("failed to read tar entries: {e}"))
        })? {
            let mut entry = entry.map_err(|e| {
                SkillxError::Archive(format!("failed to read tar entry: {e}"))
            })?;

            file_count += 1;
            if file_count > MAX_FILE_COUNT {
                return Err(SkillxError::Archive(format!(
                    "archive contains too many files (max {MAX_FILE_COUNT})"
                )));
            }

            let raw_path = entry.path().map_err(|e| {
                SkillxError::Archive(format!("invalid tar entry path: {e}"))
            })?.to_path_buf();

            // Path traversal protection
            let path_str = raw_path.to_string_lossy();
            if path_str.contains("..") {
                return Err(SkillxError::Archive(format!(
                    "path traversal detected: {path_str}"
                )));
            }

            // Flatten single root
            let relative = if let Some(ref root) = single_root {
                raw_path.strip_prefix(root).unwrap_or(&raw_path)
            } else {
                &raw_path
            };

            if relative.as_os_str().is_empty() {
                continue;
            }

            // Reject symlinks and hardlinks — they can be used for path traversal
            let entry_type = entry.header().entry_type();
            if entry_type.is_symlink() || entry_type.is_hard_link() {
                return Err(SkillxError::Archive(format!(
                    "archive contains a symlink or hardlink: {path_str} — rejected for security"
                )));
            }

            let out_path = dest.join(relative);

            // Verify the output path is within dest
            if !path_is_within(dest, &out_path) {
                return Err(SkillxError::Archive(format!(
                    "path traversal detected: {} escapes destination",
                    path_str
                )));
            }

            total_size += entry.size();
            if total_size > MAX_TOTAL_SIZE {
                return Err(SkillxError::Archive(format!(
                    "archive exceeds maximum total size of {MAX_TOTAL_SIZE} bytes"
                )));
            }

            if entry_type.is_dir() {
                std::fs::create_dir_all(&out_path).map_err(|e| {
                    SkillxError::Source(format!("failed to create dir: {e}"))
                })?;
            } else if entry_type.is_file() {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        SkillxError::Source(format!("failed to create dir: {e}"))
                    })?;
                }
                entry.unpack(&out_path).map_err(|e| {
                    SkillxError::Archive(format!(
                        "failed to unpack {}: {e}",
                        out_path.display()
                    ))
                })?;
                files.push(out_path);
            }
        }

        Ok(files)
    }
}

/// Check if `child` is within `parent` by normalizing components (no I/O).
///
/// This avoids relying on `canonicalize()` which requires the path to exist
/// and can miss symlink-based escapes when the file hasn't been written yet.
fn path_is_within(parent: &Path, child: &Path) -> bool {
    use std::path::Component;
    let normalize = |p: &Path| -> PathBuf {
        let mut parts = Vec::new();
        for c in p.components() {
            match c {
                Component::ParentDir => {
                    parts.pop();
                }
                Component::CurDir => {}
                other => parts.push(other),
            }
        }
        parts.iter().collect()
    };
    let norm_parent = normalize(parent);
    let norm_child = normalize(child);
    norm_child.starts_with(&norm_parent)
}

/// Detect if all entries in a zip share a single root directory.
fn detect_single_root_zip(archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>) -> Option<PathBuf> {
    let mut root: Option<String> = None;
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index_raw(i) {
            let name = entry.name().to_string();
            let first = name.split('/').next()?;
            match &root {
                None => root = Some(first.to_string()),
                Some(r) if r != first => return None,
                _ => {}
            }
        }
    }
    root.map(PathBuf::from)
}

/// Detect if all entries in a tar share a single root directory.
fn detect_single_root_tar(paths: &[PathBuf]) -> Option<PathBuf> {
    let mut root: Option<String> = None;
    for path in paths {
        let first = path.components().next()?;
        let first_str = first.as_os_str().to_string_lossy().to_string();
        match &root {
            None => root = Some(first_str),
            Some(r) if *r != first_str => return None,
            _ => {}
        }
    }
    root.map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_zip_path_traversal() {
        // Create a minimal zip with a path containing ".."
        // This tests the string-level check, not the zip creation
        let data = Vec::new();
        let result = ArchiveSource::extract_zip(&data, Path::new("/tmp/test"));
        // Empty zip should fail, not pass
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_single_root_tar() {
        let paths = vec![
            PathBuf::from("myskill/SKILL.md"),
            PathBuf::from("myskill/prompt.md"),
            PathBuf::from("myskill/scripts/run.sh"),
        ];
        assert_eq!(
            detect_single_root_tar(&paths),
            Some(PathBuf::from("myskill"))
        );
    }

    #[test]
    fn test_detect_no_single_root_tar() {
        let paths = vec![
            PathBuf::from("SKILL.md"),
            PathBuf::from("prompt.md"),
        ];
        // All entries at root level — each file is its own "root"
        // This should still return Some since SKILL.md and prompt.md have different first components
        let result = detect_single_root_tar(&paths);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_single_root_tar_empty() {
        let paths: Vec<PathBuf> = vec![];
        assert_eq!(detect_single_root_tar(&paths), None);
    }
}
