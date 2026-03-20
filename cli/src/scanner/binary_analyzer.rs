use std::io::Read;
use std::path::Path;

use crate::error::{Result, SkillxError};

/// Maximum bytes to read for magic bytes detection.
const MAGIC_BYTES_READ_LIMIT: usize = 8192;

pub struct BinaryAnalyzer;

impl BinaryAnalyzer {
    /// Check if a file is a binary executable using magic bytes.
    /// Only reads the first few KB to avoid loading large files into memory.
    pub fn is_executable(path: &Path) -> Result<bool> {
        let buf = Self::read_magic_bytes(path)?;

        if buf.len() < 4 {
            return Ok(false);
        }

        // Check via infer crate
        if let Some(kind) = infer::get(&buf) {
            let mime = kind.mime_type();
            if mime.starts_with("application/x-executable")
                || mime == "application/x-mach-binary"
                || mime == "application/x-elf"
                || mime == "application/vnd.microsoft.portable-executable"
                || mime == "application/x-sharedlib"
            {
                return Ok(true);
            }
        }

        // Direct magic byte checks
        // ELF
        if buf.starts_with(b"\x7fELF") {
            return Ok(true);
        }
        // Mach-O
        if buf.starts_with(&[0xfe, 0xed, 0xfa, 0xce])
            || buf.starts_with(&[0xfe, 0xed, 0xfa, 0xcf])
            || buf.starts_with(&[0xce, 0xfa, 0xed, 0xfe])
            || buf.starts_with(&[0xcf, 0xfa, 0xed, 0xfe])
        {
            return Ok(true);
        }
        // PE (Windows)
        if buf.starts_with(b"MZ") {
            return Ok(true);
        }

        Ok(false)
    }

    /// Read the first few KB of a file for magic bytes / extension mismatch detection.
    /// Avoids reading entire large files into memory.
    pub fn read_magic_bytes(path: &Path) -> Result<Vec<u8>> {
        let mut file = std::fs::File::open(path)
            .map_err(|e| SkillxError::Scan(format!("failed to open {}: {e}", path.display())))?;

        let mut buf = vec![0u8; MAGIC_BYTES_READ_LIMIT];
        let n = file
            .read(&mut buf)
            .map_err(|e| SkillxError::Scan(format!("failed to read {}: {e}", path.display())))?;
        buf.truncate(n);
        Ok(buf)
    }
}
