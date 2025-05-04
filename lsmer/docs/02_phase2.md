# Chapter 2: Phase 2 — On-Disk SSTables & Flush

In Phase 2 we persist in-memory data to disk, defining the SSTable file format and implementing the flush path.

## 2.1 SSTable Chunk Format

- **Directory per Chunk**: Each flush creates a new directory named by a timestamp (ns precision) under `sstable_dir`.
- **Arrow2 IPC File**: Inside each directory, a single `data.arrow` file stores a `RecordBatch` (all columns) using Arrow IPC V2:
  - A file header with Arrow magic bytes and schema message.
  - One or more record batches or a single batch (depending on memtable size).
  - A footer with dictionary and record block metadata for random access.
- **Row Groups (Optional)**: For very large memtables, data can be logically partitioned into row groups (chunks) of ~64K rows.
- **Compression**: Arrow2 IPC supports per-column compression (LZ4, Zstd) via `WriteOptions`.

## 2.2 Manifest Discovery

- **Manifest.json** (coming in Phase 3) will list the directories to load.
- For now, any directory under `sstable_dir` containing `data.arrow` is considered a valid SSTable.

## 2.3 Flush Semantics

When `LSMTree::flush()` is called:

1. **Invoke SSTable::new** with the current schema and `sstable_dir` to create a new chunk directory.
2. **Call `SSTable::write(&mut memtable)`** to convert all column builders into Arrow arrays and write the IPC file.
3. **Reset Memtable**:
   - Drop or replace the current `Memtable` with an empty one (same schema and capacity).
4. **Rotate WAL**:
   - Close the current WAL log.
   - Rename or archive it (e.g. `wal-<ts>.log`).
   - Create a fresh WAL file for new writes.

## 2.4 Rustdoc Example

```rust,no_run
use arrow2::datatypes::{Schema, Field, DataType};
use lsm_tree::{
    memtable::{Memtable, ScalarValue},
    wal::WriteAheadLog,
    sstable::SSTable,
    lsm::{LSMConfig, LSMTree},
};
use tempfile::tempdir;

// 1) Set up schema and directories
let fields = vec![
    Field::new("time", DataType::Int64, false),
    Field::new("value", DataType::Float64, false),
];
let schema = Schema { fields, metadata: Default::default() };
let wal_dir = tempdir()?.path().to_string_lossy().into();
let sst_dir = tempdir()?.path().to_string_lossy().into();
let mut tree = LSMTree::init(LSMConfig { schema: schema.clone(), memtable_size: 1000, wal_dir, sstable_dir: sst_dir });

// 2) Write some rows
tree.put(vec![ScalarValue::Int64(1), ScalarValue::Float64(3.14)]);
tree.put(vec![ScalarValue::Int64(2), ScalarValue::Float64(2.718)]);

// 3) Flush to disk
tree.flush();

// Check that an SSTable directory with data.arrow exists
for entry in std::fs::read_dir(tree.sstable_dir()).unwrap() {
    let path = entry.unwrap().path().join("data.arrow");
    assert!(path.exists());
}
```

In the next chapter, we'll implement manifest management and the read path to load these SSTables for queries.
