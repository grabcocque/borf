# Chapter 2: Phase 2 — Query Engine over Splayed Tables

In Phase 2 we develop a simple query execution layer on top of the splayed-table storage.

## REPL Commands

- `put <value>`: Append a new row to the storage (single-column for now).
- `count`: Show the current number of rows in the store.
- `get <index>`: Retrieve the row at the given zero-based index (e.g. `(value)`).

Internally, the REPL uses `QStore::get`, `QStore::put`, and `QStore::count`.

## Query Engine Design

- **Direct Column Access**: Query operators read directly from in-memory arrays populated at init or via memory mapping.
- **Row Reconstruction**: Rows are assembled by fetching each column value at a given index.
- **Filters & Projections**: So far unimplemented; can be added by iterating over vectors and collecting results.

## Example

```text
>> put 100
=> ok
>> put 200
=> ok
>> count
=> 2
>> get 1
=> (200)
```

In Chapter 3 we will explore adding filter, select, and aggregation commands, as well as optimizing query plans.

tree.put(vec![ScalarValue::Int64(1), ScalarValue::Float64(3.14)]);
tree.put(vec![ScalarValue::Int64(2), ScalarValue::Float64(2.718)]);
tree.flush();
## 2.4 Rustdoc Example

```rust,no_run
use arrow2::datatypes::{Schema, Field, DataType};
use qstorage::{QStoreConfig, QStore, ScalarValue};
use tempfile::tempdir;

// 1) Set up schema and data directory
let fields = vec![
    Field::new("time", DataType::Int64, false),
    Field::new("value", DataType::Float64, false),
];
let schema = Schema { fields, metadata: Default::default() };
let tmp = tempdir()?;
let data_dir = tmp.path().to_path_buf();
let mut store = QStore::init(QStoreConfig { schema: schema.clone(), data_dir });

// 2) Write some rows
store.put(vec![ScalarValue::Int64(1), ScalarValue::Float64(3.14)]);
store.put(vec![ScalarValue::Int64(2), ScalarValue::Float64(2.718)]);

// 3) Verify row count and values
assert_eq!(store.count(), 2);
assert_eq!(store.get(0), Some(vec![ScalarValue::Int64(1), ScalarValue::Float64(3.14)]));
```

In the next chapter, we'll implement tables, dictionaries, and SQL-like query primitives over these splayed tables.
