use crate::memtable::Memtable;
/// LSM Tree configuration parameters.
use crate::memtable::ScalarValue;
use crate::wal::WriteAheadLog;
use arrow2::datatypes::Schema;
use bincode;
use std::path::PathBuf;

/// LSM Tree configuration parameters.
pub struct LSMConfig {
    /// Schema describing columns and types.
    pub schema: Schema,
    /// Memtable capacity in number of rows.
    pub memtable_size: usize,
    /// Directory to store WAL.
    pub wal_dir: String,
    /// Directory to store SSTables.
    pub sstable_dir: String,
}

/// LSM Tree core data structure.
/// LSM Tree core data structure.
pub struct LSMTree {
    memtable: Memtable,
    wal: WriteAheadLog,
    #[allow(dead_code)]
    sstable_dir: PathBuf,
}

impl LSMTree {
    /// Initialize a new LSM Tree with the provided configuration.
    /// Initialize a new LSM Tree with the provided configuration.
    pub fn init(config: LSMConfig) -> Self {
        // Ensure directories exist
        std::fs::create_dir_all(&config.wal_dir).unwrap();
        std::fs::create_dir_all(&config.sstable_dir).unwrap();
        // Open WAL
        let wal = WriteAheadLog::new(&config.wal_dir).unwrap();
        // Initialize Memtable
        let memtable = Memtable::new(config.schema.clone(), config.memtable_size);
        Self {
            memtable,
            wal,
            sstable_dir: PathBuf::from(config.sstable_dir),
        }
    }

    /// Recover an existing LSM Tree from disk.
    /// Recover an existing LSM Tree from disk by replaying the WAL.
    pub fn recover(config: LSMConfig) -> Self {
        // TODO: replay WAL here
        Self::init(config)
    }

    /// Put a key-value pair into the tree.
    /// Insert a row into the LSM Tree: log to WAL and buffer in memtable.
    pub fn put(&mut self, row: Vec<ScalarValue>) {
        // Serialize row
        let payload = bincode::serialize(&row).unwrap();
        // Append to WAL
        self.wal.append(&payload).unwrap();
        // Append to Memtable
        self.memtable.append_row(row);
    }

    /// Get the value associated with a key.
    /// Point lookups are not yet implemented (in-memory only).
    pub fn get(&self, _key: &[u8]) -> Option<Vec<u8>> {
        None
    }

    /// Remove a key (tombstone) from the tree.
    /// Remove is not yet implemented.
    pub fn remove(&mut self, _key: &[u8]) {
        // TODO: tombstones
    }

    /// Flush in-memory data to disk.
    /// Flush in-memory data to SSTables (not yet implemented).
    pub fn flush(&self) {
        // TODO: write memtable to SSTable
    }
    /// Return number of rows currently buffered in the memtable.
    pub fn memtable_row_count(&self) -> usize {
        self.memtable.row_count()
    }
}
// Unit tests for LSMTree
#[cfg(test)]
mod tests {
    use super::*;
    use crate::memtable::ScalarValue;
    use arrow2::datatypes::{DataType, Field, Schema};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_init_and_put() {
        // Build schema
        let fields = vec![
            Field::new("a", DataType::Int64, false),
            Field::new("b", DataType::Utf8, false),
        ];
        let schema = Schema {
            fields,
            metadata: Default::default(),
        };
        // Create temp dirs
        let wal_dir = tempdir().unwrap();
        let sst_dir = tempdir().unwrap();
        // Configure
        let config = LSMConfig {
            schema: schema.clone(),
            memtable_size: 4,
            wal_dir: wal_dir.path().to_string_lossy().into_owned(),
            sstable_dir: sst_dir.path().to_string_lossy().into_owned(),
        };
        let mut tree = LSMTree::init(config);
        assert_eq!(tree.memtable_row_count(), 0);
        // Append two rows
        tree.put(vec![ScalarValue::Int64(1), ScalarValue::Utf8("x".into())]);
        tree.put(vec![ScalarValue::Int64(2), ScalarValue::Utf8("y".into())]);
        assert_eq!(tree.memtable_row_count(), 2);
        // WAL file should exist and be non-empty
        let entries = fs::read(wal_dir.path().join("wal.log")).unwrap();
        // WAL file should exist and be non-empty
        assert!(!entries.is_empty());
    }
}
