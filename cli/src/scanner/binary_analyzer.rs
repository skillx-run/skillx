use sha2::{Digest, Sha256};
use std::path::Path;

use crate::error::{Result, SkillxError};

/// Information about a binary file.
#[derive(Debug, Clone)]
pub struct BinaryInfo {
    pub file_type: String,
    pub size: u64,
    pub sha256: String,
}

pub struct BinaryAnalyzer;

impl BinaryAnalyzer {
    /// Analyze a binary file and return its metadata.
    pub fn analyze(path: &Path) -> Result<BinaryInfo> {
        let buf = std::fs::read(path)
            .map_err(|e| SkillxError::Scan(format!("failed to read file: {e}")))?;

        let file_type = if let Some(kind) = infer::get(&buf) {
            kind.mime_type().to_string()
        } else {
            "unknown".to_string()
        };

        let size = buf.len() as u64;

        let mut hasher = Sha256::new();
        hasher.update(&buf);
        let sha256 = format!("{:x}", hasher.finalize());

        Ok(BinaryInfo {
            file_type,
            size,
            sha256,
        })
    }

    /// Compute SHA256 hash of a file.
    pub fn sha256_file(path: &Path) -> Result<String> {
        let buf = std::fs::read(path)
            .map_err(|e| SkillxError::Scan(format!("failed to read file: {e}")))?;
        let mut hasher = Sha256::new();
        hasher.update(&buf);
        Ok(format!("{:x}", hasher.finalize()))
    }
}
