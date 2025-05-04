use crc32fast::Hasher;
use std::fs::{File, OpenOptions};
use std::io::{Result, Write};
use std::path::Path;

/// Write-Ahead Log implementation with framing and CRC32C checksum.
pub struct WriteAheadLog {
    file: File,
}

impl WriteAheadLog {
    /// Open or create a WAL in the given directory. Creates `wal.log` inside it.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let dir = path.as_ref();
        // Ensure WAL directory exists
        std::fs::create_dir_all(dir)?;
        let file_path = dir.join("wal.log");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;
        Ok(Self { file })
    }

    /// Append a record to the log with a simple header and CRC32C.
    pub fn append(&mut self, payload: &[u8]) -> Result<()> {
        // Header: magic (4), version (1), length (4), checksum (4)
        const MAGIC: &[u8; 4] = b"LSM1";
        const VERSION: u8 = 1;
        let mut header = Vec::with_capacity(13);
        header.extend_from_slice(MAGIC);
        header.push(VERSION);
        let len = payload.len() as u32;
        header.extend_from_slice(&len.to_be_bytes());
        let mut hasher = Hasher::new();
        hasher.update(payload);
        let checksum = hasher.finalize();
        header.extend_from_slice(&checksum.to_be_bytes());

        self.file.write_all(&header)?;
        self.file.write_all(payload)?;
        self.file.flush()?;
        Ok(())
    }
}
// Unit tests for WriteAheadLog
#[cfg(test)]
mod tests {
    use super::*;
    use crc32fast::Hasher;
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn test_wal_append_and_parse() {
        let dir = tempdir().unwrap();
        let mut wal = WriteAheadLog::new(dir.path()).unwrap();

        let payload = b"hello world";
        wal.append(payload).unwrap();

        let mut buf = Vec::new();
        let file_path = dir.path().join("wal.log");
        let mut f = std::fs::File::open(file_path).unwrap();
        f.read_to_end(&mut buf).unwrap();

        // Header: magic, version, length, checksum
        assert!(buf.len() >= 13 + payload.len());
        assert_eq!(&buf[0..4], b"LSM1");
        assert_eq!(buf[4], 1);
        let len = u32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]);
        assert_eq!(len as usize, payload.len());
        let checksum = u32::from_be_bytes([buf[9], buf[10], buf[11], buf[12]]);
        let mut hasher = Hasher::new();
        hasher.update(payload);
        assert_eq!(checksum, hasher.finalize());

        // Payload
        assert_eq!(&buf[13..13 + payload.len()], payload);
    }
}
