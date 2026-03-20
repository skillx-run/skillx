use sha2::{Digest, Sha256};
use std::path::Path;

use crate::error::{Result, SkillxError};

use super::manifest::Manifest;

/// Inject skill files from source_dir into target_dir, recording in manifest.
pub fn inject_skill(
    source_dir: &Path,
    target_dir: &Path,
    manifest: &mut Manifest,
) -> Result<()> {
    // Create target directory
    std::fs::create_dir_all(target_dir).map_err(|e| {
        SkillxError::Session(format!(
            "failed to create target dir {}: {e}",
            target_dir.display()
        ))
    })?;

    // Recursively copy files
    copy_dir_recursive(source_dir, target_dir, source_dir, target_dir, manifest)?;

    Ok(())
}

fn copy_dir_recursive(
    src: &Path,
    _dst: &Path,
    src_root: &Path,
    dst_root: &Path,
    manifest: &mut Manifest,
) -> Result<()> {
    let entries = std::fs::read_dir(src).map_err(|e| {
        SkillxError::Session(format!("failed to read dir {}: {e}", src.display()))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            SkillxError::Session(format!("dir entry error: {e}"))
        })?;
        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(src_root).unwrap_or(&src_path);
        let dst_path = dst_root.join(rel_path);

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path).map_err(|e| {
                SkillxError::Session(format!(
                    "failed to create dir {}: {e}",
                    dst_path.display()
                ))
            })?;
            copy_dir_recursive(&src_path, &dst_path, src_root, dst_root, manifest)?;
        } else {
            // Read, hash, and copy file
            let content = std::fs::read(&src_path).map_err(|e| {
                SkillxError::Session(format!(
                    "failed to read {}: {e}",
                    src_path.display()
                ))
            })?;

            let mut hasher = Sha256::new();
            hasher.update(&content);
            let sha256 = format!("{:x}", hasher.finalize());

            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            std::fs::write(&dst_path, &content).map_err(|e| {
                SkillxError::Session(format!(
                    "failed to write {}: {e}",
                    dst_path.display()
                ))
            })?;

            manifest.add_file(
                dst_path.to_string_lossy().to_string(),
                sha256,
            );
        }
    }

    Ok(())
}
