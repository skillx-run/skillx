use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::error::{Result, SkillxError};

use super::manifest::Manifest;

/// How a file was injected — determines cleanup strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InjectionType {
    /// Direct file copy — cleanup by deleting the file.
    CopiedFile,
    /// Section appended to an aggregate file — cleanup by removing the section.
    AggregateSection,
}

impl Default for InjectionType {
    fn default() -> Self {
        InjectionType::CopiedFile
    }
}

/// Record of an injected file for manifest tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectedRecord {
    pub path: String,
    pub sha256: String,
    #[serde(default)]
    pub injection_type: InjectionType,
}

impl InjectedRecord {
    pub fn copied_file(path: String, sha256: String) -> Self {
        InjectedRecord {
            path,
            sha256,
            injection_type: InjectionType::CopiedFile,
        }
    }

    pub fn aggregate_section(path: String, sha256: String) -> Self {
        InjectedRecord {
            path,
            sha256,
            injection_type: InjectionType::AggregateSection,
        }
    }
}

/// Core inject: copy files from source to target, return (relative_path, sha256) records.
pub fn inject_and_collect(source_dir: &Path, target_dir: &Path) -> Result<Vec<(String, String)>> {
    std::fs::create_dir_all(target_dir).map_err(|e| {
        SkillxError::Session(format!(
            "failed to create target dir {}: {e}",
            target_dir.display()
        ))
    })?;

    let mut records = Vec::new();
    collect_dir_recursive(source_dir, source_dir, target_dir, &mut records)?;
    Ok(records)
}

/// Inject skill files from source_dir into target_dir, recording in manifest.
pub fn inject_skill(source_dir: &Path, target_dir: &Path, manifest: &mut Manifest) -> Result<()> {
    let records = inject_and_collect(source_dir, target_dir)?;
    for (relative, sha256) in records {
        let full_path = target_dir.join(&relative);
        manifest.add_file(full_path.to_string_lossy().to_string(), sha256);
    }
    Ok(())
}

/// Extract the body of a SKILL.md file (strip YAML frontmatter).
pub fn extract_skill_body(source_dir: &Path) -> Result<String> {
    let skill_path = source_dir.join("SKILL.md");
    let content = std::fs::read_to_string(&skill_path).map_err(|e| {
        SkillxError::Session(format!("failed to read {}: {e}", skill_path.display()))
    })?;

    // Strip YAML frontmatter (--- ... ---)
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let body = &content[3 + end + 3..];
            return Ok(body.trim_start_matches('\n').to_string());
        }
    }
    Ok(content)
}

/// Marker comment for the start of a skillx-injected section.
fn begin_marker(skill_name: &str) -> String {
    format!("<!-- skillx:begin:{skill_name} -->")
}

/// Marker comment for the end of a skillx-injected section.
fn end_marker(skill_name: &str) -> String {
    format!("<!-- skillx:end:{skill_name} -->")
}

/// Append skill content to an aggregate file with skillx markers.
/// Creates the file if it doesn't exist. Returns an InjectedRecord.
pub fn append_to_aggregate_file(
    target: &Path,
    skill_name: &str,
    content: &str,
) -> Result<InjectedRecord> {
    let begin = begin_marker(skill_name);
    let end = end_marker(skill_name);

    // Read existing content (or empty)
    let existing = std::fs::read_to_string(target).unwrap_or_default();

    // If the skill is already in the file, replace it
    let cleaned = remove_section_from_string(&existing, skill_name);

    // Build new section
    let section = format!("\n{begin}\n## {skill_name}\n\n{content}\n{end}\n");
    let new_content = format!("{}{}", cleaned.trim_end(), section);

    // Ensure parent dir exists
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            SkillxError::Session(format!(
                "failed to create dir {}: {e}",
                parent.display()
            ))
        })?;
    }

    std::fs::write(target, &new_content).map_err(|e| {
        SkillxError::Session(format!("failed to write {}: {e}", target.display()))
    })?;

    // Hash the section content for tracking
    let mut hasher = Sha256::new();
    hasher.update(section.as_bytes());
    let sha256 = format!("{:x}", hasher.finalize());

    Ok(InjectedRecord::aggregate_section(
        target.to_string_lossy().to_string(),
        sha256,
    ))
}

/// Remove a skill section from an aggregate file by markers.
/// Returns true if a section was found and removed.
pub fn remove_from_aggregate_file(target: &Path, skill_name: &str) -> Result<bool> {
    let content = match std::fs::read_to_string(target) {
        Ok(c) => c,
        Err(_) => return Ok(false),
    };

    let cleaned = remove_section_from_string(&content, skill_name);
    if cleaned.len() == content.len() {
        return Ok(false); // No change
    }

    let trimmed = cleaned.trim_end().to_string();
    let final_content = if trimmed.is_empty() {
        String::new()
    } else {
        format!("{trimmed}\n")
    };

    std::fs::write(target, &final_content).map_err(|e| {
        SkillxError::Session(format!("failed to write {}: {e}", target.display()))
    })?;

    Ok(true)
}

/// Remove the section between skillx markers from a string.
fn remove_section_from_string(content: &str, skill_name: &str) -> String {
    let begin = begin_marker(skill_name);
    let end = end_marker(skill_name);

    if let Some(start_pos) = content.find(&begin) {
        if let Some(end_pos) = content[start_pos..].find(&end) {
            let before = &content[..start_pos];
            let after = &content[start_pos + end_pos + end.len()..];
            return format!("{}{}", before.trim_end_matches('\n'), after);
        }
    }
    content.to_string()
}

fn collect_dir_recursive(
    src: &Path,
    src_root: &Path,
    dst_root: &Path,
    records: &mut Vec<(String, String)>,
) -> Result<()> {
    let entries = std::fs::read_dir(src)
        .map_err(|e| SkillxError::Session(format!("failed to read dir {}: {e}", src.display())))?;

    for entry in entries {
        let entry = entry.map_err(|e| SkillxError::Session(format!("dir entry error: {e}")))?;
        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(src_root).unwrap_or(&src_path);
        let dst_path = dst_root.join(rel_path);

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path).map_err(|e| {
                SkillxError::Session(format!("failed to create dir {}: {e}", dst_path.display()))
            })?;
            collect_dir_recursive(&src_path, src_root, dst_root, records)?;
        } else {
            // Read, hash, and copy file
            let content = std::fs::read(&src_path).map_err(|e| {
                SkillxError::Session(format!("failed to read {}: {e}", src_path.display()))
            })?;

            let mut hasher = Sha256::new();
            hasher.update(&content);
            let sha256 = format!("{:x}", hasher.finalize());

            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    SkillxError::Session(format!(
                        "failed to create parent dir {}: {e}",
                        parent.display()
                    ))
                })?;
            }

            std::fs::write(&dst_path, &content).map_err(|e| {
                SkillxError::Session(format!("failed to write {}: {e}", dst_path.display()))
            })?;

            records.push((rel_path.to_string_lossy().to_string(), sha256));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extract_skill_body_with_frontmatter() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("SKILL.md"),
            "---\nname: test\n---\n\n# Hello\n\nBody content here.\n",
        )
        .unwrap();
        let body = extract_skill_body(dir.path()).unwrap();
        assert!(body.starts_with("# Hello"));
        assert!(body.contains("Body content here."));
    }

    #[test]
    fn test_extract_skill_body_without_frontmatter() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("SKILL.md"), "# No Frontmatter\n\nJust content.\n")
            .unwrap();
        let body = extract_skill_body(dir.path()).unwrap();
        assert!(body.starts_with("# No Frontmatter"));
    }

    #[test]
    fn test_aggregate_file_append_and_remove() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join(".goosehints");

        // Append first skill
        append_to_aggregate_file(&target, "skill-a", "Content for A").unwrap();
        let content = std::fs::read_to_string(&target).unwrap();
        assert!(content.contains("<!-- skillx:begin:skill-a -->"));
        assert!(content.contains("Content for A"));
        assert!(content.contains("<!-- skillx:end:skill-a -->"));

        // Append second skill
        append_to_aggregate_file(&target, "skill-b", "Content for B").unwrap();
        let content = std::fs::read_to_string(&target).unwrap();
        assert!(content.contains("Content for A"));
        assert!(content.contains("Content for B"));

        // Remove first skill
        let removed = remove_from_aggregate_file(&target, "skill-a").unwrap();
        assert!(removed);
        let content = std::fs::read_to_string(&target).unwrap();
        assert!(!content.contains("Content for A"));
        assert!(content.contains("Content for B"));

        // Remove second skill
        let removed = remove_from_aggregate_file(&target, "skill-b").unwrap();
        assert!(removed);
    }

    #[test]
    fn test_aggregate_file_preserves_user_content() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join(".goosehints");

        // Write user content first
        std::fs::write(&target, "# My custom hints\n\nDo not delete this.\n").unwrap();

        // Append skill
        append_to_aggregate_file(&target, "my-skill", "Skill content").unwrap();
        let content = std::fs::read_to_string(&target).unwrap();
        assert!(content.contains("My custom hints"));
        assert!(content.contains("Skill content"));

        // Remove skill — user content preserved
        remove_from_aggregate_file(&target, "my-skill").unwrap();
        let content = std::fs::read_to_string(&target).unwrap();
        assert!(content.contains("My custom hints"));
        assert!(!content.contains("Skill content"));
    }

    #[test]
    fn test_aggregate_file_replaces_existing_section() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("AGENTS.md");

        append_to_aggregate_file(&target, "skill-x", "Version 1").unwrap();
        append_to_aggregate_file(&target, "skill-x", "Version 2").unwrap();
        let content = std::fs::read_to_string(&target).unwrap();

        // Should only have Version 2, not Version 1
        assert!(!content.contains("Version 1"));
        assert!(content.contains("Version 2"));

        // Should have exactly one begin/end marker pair
        assert_eq!(
            content.matches("skillx:begin:skill-x").count(),
            1,
            "should have exactly one begin marker"
        );
    }

    #[test]
    fn test_remove_from_nonexistent_file() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("nonexistent.md");
        let removed = remove_from_aggregate_file(&target, "anything").unwrap();
        assert!(!removed);
    }
}
