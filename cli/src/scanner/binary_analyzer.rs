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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_magic_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("binary");
        // ELF header: \x7fELF + class(64bit) + endian + version + OS/ABI
        std::fs::write(&path, b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00")
            .unwrap();
        assert!(BinaryAnalyzer::is_executable(&path).unwrap());
    }

    #[test]
    fn test_pe_magic_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("program.exe");
        // PE header starts with MZ
        let mut pe_data = b"MZ".to_vec();
        pe_data.extend_from_slice(&[0u8; 100]); // Pad with zeros
        std::fs::write(&path, &pe_data).unwrap();
        assert!(BinaryAnalyzer::is_executable(&path).unwrap());
    }

    #[test]
    fn test_macho_magic_bytes_le() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("macho");
        // Mach-O 64-bit little-endian
        let mut data = vec![0xcf, 0xfa, 0xed, 0xfe];
        data.extend_from_slice(&[0u8; 100]);
        std::fs::write(&path, &data).unwrap();
        assert!(BinaryAnalyzer::is_executable(&path).unwrap());
    }

    #[test]
    fn test_text_file_not_executable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("script.sh");
        std::fs::write(&path, "#!/bin/bash\necho hello world\n").unwrap();
        assert!(!BinaryAnalyzer::is_executable(&path).unwrap());
    }

    #[test]
    fn test_empty_file_not_executable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty");
        std::fs::write(&path, b"").unwrap();
        assert!(!BinaryAnalyzer::is_executable(&path).unwrap());
    }

    #[test]
    fn test_small_file_under_4_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tiny");
        std::fs::write(&path, b"abc").unwrap();
        assert!(!BinaryAnalyzer::is_executable(&path).unwrap());
    }

    #[test]
    fn test_read_magic_bytes_truncates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large");
        // Write 16KB of data — read_magic_bytes should only return 8KB
        let data = vec![0x42u8; 16384];
        std::fs::write(&path, &data).unwrap();
        let bytes = BinaryAnalyzer::read_magic_bytes(&path).unwrap();
        assert_eq!(bytes.len(), MAGIC_BYTES_READ_LIMIT);
    }
}
