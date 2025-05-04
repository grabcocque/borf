//! SSTable: on-disk columnar chunk files using Arrow2 IPC
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Result};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
// use arrow2::array::MutableArray; // not needed
use crate::memtable::Memtable;
use arrow2::chunk::Chunk;
use arrow2::datatypes::Schema;
use arrow2::io::ipc::write::{FileWriter, WriteOptions};

/// SSTable representation: directory containing per-column IPC files.
pub struct SSTable {
    /// Path to SSTable directory with column files.
    pub path: PathBuf,
    /// Schema of the data.
    pub schema: Schema,
}

impl SSTable {
    /// Create a new SSTable directory under `base_dir` with a timestamp-based name.
    pub fn new<P: AsRef<Path>>(base_dir: P, schema: Schema) -> Result<Self> {
        let base = base_dir.as_ref();
        // Ensure base directory exists
        create_dir_all(base)?;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = base.join(ts.to_string());
        create_dir_all(&dir)?;
        Ok(SSTable { path: dir, schema })
    }

    /// Write the entire memtable as one Arrow IPC RecordBatch.
    pub fn write(&self, memtable: &mut Memtable) -> Result<()> {
        // Extract builders into boxed arrays
        let (arrays, _row_count) = memtable.to_arrays();
        // Build a single chunk
        let chunk = Chunk::new(arrays);
        // Write IPC file
        let file_path = self.path.join("data.arrow");
        let file = File::create(file_path)?;
        // Create IPC writer and write header
        let mut writer = FileWriter::try_new(
            BufWriter::new(file),
            self.schema.clone(),
            None,
            WriteOptions::default(),
        )
        .map_err(std::io::Error::other)?;
        writer.write(&chunk, None).map_err(std::io::Error::other)?;
        writer.finish().map_err(std::io::Error::other)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memtable::{Memtable, ScalarValue};
    use arrow2::datatypes::{DataType, Field, Schema};
    use arrow2::io::ipc::read::FileReader;
    use std::fs;
    use std::io::BufReader;
    use tempfile::tempdir;

    #[test]
    fn test_sstable_write_and_read() {
        // Define schema
        let fields = vec![
            Field::new("i", DataType::Int64, false),
            Field::new("s", DataType::Utf8, false),
        ];
        let schema = Schema {
            fields,
            metadata: Default::default(),
        };
        // Create memtable and append rows
        let mut mt = Memtable::new(schema.clone(), 2);
        mt.append_row(vec![
            ScalarValue::Int64(100),
            ScalarValue::Utf8("foo".into()),
        ]);
        mt.append_row(vec![
            ScalarValue::Int64(200),
            ScalarValue::Utf8("bar".into()),
        ]);
        // Create temp directories
        let base = tempdir().unwrap();
        let sst = SSTable::new(base.path(), schema.clone()).unwrap();
        // Write SSTable
        sst.write(&mut mt).unwrap();
        // Locate written file
        let entries: Vec<_> = fs::read_dir(base.path()).unwrap().collect();
        assert_eq!(entries.len(), 1);
        let dir = entries[0].as_ref().unwrap().path();
        let fpath = dir.join("data.arrow");
        assert!(fpath.exists());
        // Read back via Arrow IPC
        let file = fs::File::open(&fpath).unwrap();
        // Read back via Arrow IPC
        let mut buf = BufReader::new(file);
        let metadata = arrow2::io::ipc::read::read_file_metadata(&mut buf).unwrap();
        let reader = FileReader::new(buf, metadata, None, None);
        // Collect all batches, panicking on arrow errors
        let batches: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(batches.len(), 1);
        let batch = &batches[0];
        // Check array lengths
        assert_eq!(batch.columns()[0].len(), 2);
        assert_eq!(batch.columns()[1].len(), 2);
    }
}
