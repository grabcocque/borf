# Chapter 1: Phase 1 — Foundation & In-Memory Path

In Phase 1 we lay the groundwork for our columnar LSM engine:

1. **Project Scaffolding**
   - Cargo project with modules: `memtable`, `wal`, `lsm`, plus stubs for future layers.
   - `.pre-commit-config.yaml` for formatting and linting.
   - `.gitignore` to exclude Rust build artifacts.

2. **Memtable** (`src/memtable.rs`)
   - Uses **Arrow2** `ArrayBuilder` under the hood for each column:
     - `MutablePrimitiveArray<i64>` for `Int64`.
     - `MutablePrimitiveArray<f64>` for `Float64`.
     - `MutableUtf8Array<i32>` for `Utf8`.
   - Public types:
     ```rust
     #[derive(Debug, Serialize, Deserialize)]
     pub enum ScalarValue {
         Int64(i64),
         Float64(f64),
         Utf8(String),
     }

     pub struct Memtable {
         pub fn new(schema: Schema, capacity: usize) -> Self;
         pub fn append_row(&mut self, row: Vec<ScalarValue>);
         pub fn row_count(&self) -> usize;
     }
     ```
   - **Testing**: Unit test `test_memtable_basic_append` confirms row buffering and builder lengths.

3. **Write-Ahead Log (WAL)** (`src/wal.rs`)
   - Simple framed format:
     - 4 bytes magic `"LSM1"`
     - 1 byte version (v1)
     - 4 bytes big-endian payload length
     - 4 bytes CRC32C of payload
     - payload bytes (e.g. serialized `ScalarValue`)
   - API:
     ```rust
     pub struct WriteAheadLog { /* ... */ }
     impl WriteAheadLog {
         pub fn new<P: AsRef<Path>>(dir: P) -> io::Result<Self>;
         pub fn append(&mut self, payload: &[u8]) -> io::Result<()>;
     }
     ```
   - **Testing**: Unit test `test_wal_append_and_parse` validates header, checksum, and payload.

4. **LSMTree Core** (`src/lsm.rs`)
   - Configuration:
     ```rust
     pub struct LSMConfig {
         pub schema: Schema,
         pub memtable_size: usize,
         pub wal_dir: String,
         pub sstable_dir: String,
     }
     ```
   - Core type:
     ```rust
     pub struct LSMTree {
         pub fn init(config: LSMConfig) -> Self;
         pub fn put(&mut self, row: Vec<ScalarValue>);
         pub fn memtable_row_count(&self) -> usize;
         // stub methods: get, remove, flush
     }
     ```
   - **Integration**:
     - `init` creates directories, opens WAL, builds blank `Memtable`.
     - `put` serializes `Vec<ScalarValue>` via `bincode`, appends to WAL, and buffers in `Memtable`.
   - **Testing**: `test_init_and_put` verifies `memtable_row_count` and that `wal.log` is created and non-empty.

## Rustdoc Examples

Below is a minimal example demonstrating Phase 1 usage:

```rust
use arrow2::datatypes::{Schema, Field, DataType};
use lsm_tree::memtable::{Memtable, ScalarValue};
use lsm_tree::wal::WriteAheadLog;
use lsm_tree::lsm::{LSMConfig, LSMTree};
use tempfile::tempdir;

// Define schema and create Memtable:
let schema = Schema {
    fields: vec![
        Field::new("time", DataType::Int64, false),
        Field::new("symbol", DataType::Utf8, false),
    ],
    metadata: Default::default(),
};
let mut memtable = Memtable::new(schema.clone(), 1024);
memtable.append_row(vec![
    ScalarValue::Int64(1_672_000_000),
    ScalarValue::Utf8("AAPL".into()),
]);
assert_eq!(memtable.row_count(), 1);

// Create WAL and append payload:
let tmp = tempdir().unwrap();
let mut wal = WriteAheadLog::new(tmp.path()).unwrap();
let row = vec![ScalarValue::Int64(42), ScalarValue::Utf8("GOOG".into())];
let payload = bincode::serialize(&row).unwrap();
wal.append(&payload).unwrap();

// Initialize LSMTree and put rows:
let config = LSMConfig {
    schema,
    memtable_size: 128,
    wal_dir: tmp.path().to_string_lossy().into_owned(),
    sstable_dir: tmp.path().to_string_lossy().into_owned(),
};
let mut tree = LSMTree::init(config);
tree.put(row);
assert_eq!(tree.memtable_row_count(), 1);
```

Phase 1 confirms our end-to-end path for writes: **BorF rows → WAL → Memtable**. In Chapter 2 we'll extend this with on-disk SSTables and flush semantics.
