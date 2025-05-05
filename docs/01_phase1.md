# Chapter 1: Phase 1 — Splayed Tables Storage Engine

In Phase 1 we implement the foundational splayed-table storage engine, inspired by Q/KDB+:

1. **Project Scaffolding**
   - Cargo workspace with modules: `qstorage`, `qparser`, and `repl`.
   - Data directory holds one file per column (splayed table layout).

2. **Splayed Tables Storage** (`qstorage/src/lib.rs`)
   - Leverages **Arrow2** `Schema` and `Field` definitions for column metadata.
   - **ScalarValue** enum supports `Int64` and `Float64` types.
  - **QStoreConfig** holds the `schema` and a `data_dir` for column files.
  - **QStore** methods:
    ```rust
    pub fn init(config: QStoreConfig) -> Self;      // Open or create memory-mapped column files
    pub fn put(&mut self, row: Vec<ScalarValue>);   // Append values to column files
    pub fn count(&self) -> usize;                   // Number of rows
    pub fn get(&self, index: usize) -> Option<Vec<ScalarValue>>; // Retrieve a row by index
    ```
   - **Testing**: Unit test `test_put_and_count` validates row count, file persistence, and data reloading.

3. **Persistence & Memory Mapping**
   - Column files are memory-mapped on init for zero-copy reads.
   - New rows appended to the end of each file; mappings may be updated or tracked internally.
   - No separate WAL or in-memory memtable—writes go directly to the splayed column files.

4. **Next Steps**
   - Phase 1 provides a durable, memory-mapped splayed-table store.
   - In Chapter 2, we will explore building a query execution engine over these splayed tables,
     including filters, aggregations, and joins directly on memory-mapped columns.

## Rustdoc Examples

Below is a minimal example demonstrating Phase 1 usage with the `qstorage` crate:

```rust
use arrow2::datatypes::{Schema, Field, DataType};
use qstorage::{QStoreConfig, QStore, ScalarValue};
use tempfile::tempdir;

// Define the table schema
let schema = Schema {
    fields: vec![
        Field::new("time", DataType::Int64, false),
        Field::new("symbol", DataType::Utf8, false),
    ],
    metadata: Default::default(),
};

// Create a temporary data directory
let tmp = tempdir().unwrap();
let data_dir = tmp.path().to_path_buf();

// Initialize the store
let config = QStoreConfig { schema: schema.clone(), data_dir };
let mut store = QStore::init(config);

// Append a row
store.put(vec![
    ScalarValue::Int64(1_672_000_000),
    ScalarValue::Utf8("AAPL".into()),
]);
assert_eq!(store.count(), 1);

// Retrieve the row
let row = store.get(0).unwrap();
assert_eq!(row, vec![
    ScalarValue::Int64(1_672_000_000),
    ScalarValue::Utf8("AAPL".into()),
]);
```

Phase 1 confirms our end-to-end path for writes: **BorF rows → memory-mapped splayed tables**. In Chapter 2 we'll build a query engine over these tables (filters, projections, joins).
