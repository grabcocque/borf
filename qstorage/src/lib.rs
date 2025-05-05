use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use arrow2::datatypes::{Field, Schema};

// Column-family data model
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TsKey {
    pub measurement_id: u64,
    pub ts: u64,
}

impl TsKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}

pub type TagSet = Vec<String>;

pub enum TimeValue {
    F64(f64),
    Json(JsonValue),
}

pub type NodeID = u64;
pub type EntityID = u64;
pub type Props = HashMap<String, String>;
pub type Neighbors = Vec<NodeID>;

pub const CF_TIMESTAMPS: &str = "timestamps";
pub const CF_VALUES: &str = "values";
pub const CF_TAGS: &str = "tags";
pub const CF_NODES: &str = "nodes";
pub const CF_OUT_EDGES: &str = "out_edges";
pub const CF_IN_EDGES: &str = "in_edges";
pub const CF_PROPS: &str = "props";

/// Scalar values supported by the storage engine.
#[derive(Clone, Debug, PartialEq)]
pub enum ScalarValue {
    Int64(i64),
    Float64(f64),
    // Extend with more types as needed
}

/// Configuration for the splayed table storage.
pub struct QStoreConfig {
    pub schema: Schema,
    /// Directory where column files are stored (one file per column).
    pub data_dir: PathBuf,
}

/// Splayed table storage: one file per column, memory-backed arrays on init.
pub struct QStore {
    config: QStoreConfig,
    columns: HashMap<String, Vec<ScalarValue>>,
}

impl QStore {
    /// Initialize the storage: load existing column files and prepare in-memory arrays.
    pub fn init(config: QStoreConfig) -> Self {
        // Ensure data directory exists
        std::fs::create_dir_all(&config.data_dir).unwrap();
        let mut columns = HashMap::new();
        // For each column in schema, load existing data
        for Field {
            name, data_type, ..
        } in &config.schema.fields
        {
            let path = config.data_dir.join(name);
            let mut vec = Vec::new();
            if Path::new(&path).exists() {
                let mut file = File::open(&path).unwrap();
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).unwrap();
                // Interpret bytes as little-endian values based on data_type
                match data_type {
                    arrow2::datatypes::DataType::Int64 => {
                        for chunk in buf.chunks_exact(8) {
                            let v = i64::from_le_bytes(chunk.try_into().unwrap());
                            vec.push(ScalarValue::Int64(v));
                        }
                    }
                    arrow2::datatypes::DataType::Float64 => {
                        for chunk in buf.chunks_exact(8) {
                            let v = f64::from_le_bytes(chunk.try_into().unwrap());
                            vec.push(ScalarValue::Float64(v));
                        }
                    }
                    _ => {
                        // Unsupported type
                        panic!("Unsupported data type in storage: {:?}", data_type);
                    }
                }
            }
            // Ensure the file exists for future writes
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .unwrap();
            columns.insert(name.clone(), vec);
        }
        QStore { config, columns }
    }

    /// Append a new row of values, writing each value to its column file.
    pub fn put(&mut self, row: Vec<ScalarValue>) {
        for (i, value) in row.into_iter().enumerate() {
            let field = &self.config.schema.fields[i];
            let name = &field.name;
            let col = self.columns.get_mut(name).unwrap();
            // Append in-memory
            col.push(value.clone());
            // Append to disk
            let path = self.config.data_dir.join(name);
            let mut file = OpenOptions::new().append(true).open(&path).unwrap();
            match value {
                ScalarValue::Int64(v) => file.write_all(&v.to_le_bytes()).unwrap(),
                ScalarValue::Float64(v) => file.write_all(&v.to_le_bytes()).unwrap(),
            }
        }
    }

    /// Number of rows currently stored (based on first column length).
    pub fn memtable_row_count(&self) -> usize {
        self.columns.values().next().map(|v| v.len()).unwrap_or(0)
    }

    /// Retrieve a row by index (not used by REPL yet).
    pub fn get(&self, index: usize) -> Option<Vec<ScalarValue>> {
        if index >= self.memtable_row_count() {
            return None;
        }
        let mut row = Vec::new();
        for name in self.config.schema.fields.iter().map(|f| &f.name) {
            let col = &self.columns[name];
            row.push(col[index].clone());
        }
        Some(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow2::datatypes::{DataType, Field};
    use tempfile::tempdir;

    #[test]
    fn test_put_and_count() {
        let dir = tempdir().unwrap();
        let fields = vec![Field::new("col", DataType::Int64, false)];
        let schema = Schema {
            fields,
            metadata: Default::default(),
        };
        let config = QStoreConfig {
            schema,
            data_dir: dir.path().to_path_buf(),
        };
        let mut store = QStore::init(config);
        assert_eq!(store.memtable_row_count(), 0);
        store.put(vec![ScalarValue::Int64(42)]);
        assert_eq!(store.memtable_row_count(), 1);
        // verify data persisted in file
        let file_path = dir.path().join("col");
        let data = std::fs::read(&file_path).unwrap();
        assert_eq!(i64::from_le_bytes(data[..8].try_into().unwrap()), 42);
    }
}
