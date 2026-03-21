use std::path::{Path, PathBuf};

use crate::error::{Result, SkillxError};
use crate::source::{parse_frontmatter, ResolvedSkill, SkillSource};

pub struct LocalSource;

impl LocalSource {
    /// Fetch a skill from a local directory.
    pub fn fetch(path: &Path) -> Result<ResolvedSkill> {
        // Validate the directory exists
        if !path.exists() {
            return Err(SkillxError::SkillNotFound(format!(
                "path does not exist: {}",
                path.display()
            )));
        }
        if !path.is_dir() {
            return Err(SkillxError::Source(format!(
                "not a directory: {}",
                path.display()
            )));
        }

        // Check for SKILL.md
        let skill_md = path.join("SKILL.md");
        if !skill_md.exists() {
            return Err(SkillxError::SkillNotFound(format!(
                "SKILL.md not found in {}",
                path.display()
            )));
        }

        // Read and parse SKILL.md
        let content = std::fs::read_to_string(&skill_md)
            .map_err(|e| SkillxError::Source(format!("failed to read SKILL.md: {e}")))?;
        let metadata = parse_frontmatter(&content)?;

        // Collect all files in the skill directory
        let files = collect_files(path)?;

        Ok(ResolvedSkill {
            source: SkillSource::Local(path.to_path_buf()),
            metadata,
            root_dir: path.to_path_buf(),
            files,
        })
    }
}

/// Recursively collect all files in a directory.
fn collect_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_recursive(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| SkillxError::Source(format!("failed to read dir {}: {e}", dir.display())))?;

    for entry in entries {
        let entry =
            entry.map_err(|e| SkillxError::Source(format!("failed to read dir entry: {e}")))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, files)?;
        } else {
            files.push(path);
        }
    }

    Ok(())
}
