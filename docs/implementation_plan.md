# Implementation Plan: Memory-Mapped Splayed Tables

## Overview
This project unites:
- **BorF**: an expressive, Q-inspired query language.
- **Lsmer**: a column-oriented storage engine using splayed tables and memory-mapped column files.

Together they provide high-performance analytics with safe, zero-copy, Rust-based implementation.

## Phases
## Cross-Cutting: Performance & Profiling
- Set up a `benches/` directory with Criterion-based microbenchmarks:
  - Parser: parsing & evaluating expressions of varying complexity.
  - Storage: `put`, `get`, `count`, and scan operations at different scales.
- Instrument hot paths for inlining; integrate `pprof` or `flamegraph` profiling.
- Ensure lock-free, thread-safe data structures:
  - Readers use read-only memory-mapped slices.
  - Writers append-only to files with minimal synchronization.
- Design data layouts for cache locality:
  - Columnar, contiguous arrays of primitives.
  - Align structures to cache-line boundaries.
- Support SIMD/vectorizable code:
  - Use `std::simd` or `packed_simd` in filter and aggregation loops.
- Integrate benchmark runs into CI to catch performance regressions early.

### Phase 0: Core Parser & REPL
- Set up Cargo workspace: crates `qparser`, `qstorage`, `repl`.
- Shared types: `ScalarValue` enum, Arrow2 `Schema`/`Field`.
- QParser: Chumsky-based parser for atoms (int, float), basic arithmetic verbs (`+ - * %`), AST & evaluator.
- REPL: `repl` crate uses `rustyline` to read commands, call parser & evaluator, print results.

### Phase 1: Splayed-Table Storage
- **QStoreConfig**: `schema: Schema`, `data_dir: PathBuf`.
- **Column Files**: one file per column under `data_dir/<column>.bin`, binary little-endian values.
- **Memory-Mapped I/O**:
  - On `init(config)`, open (or create) each column file and `mmap` into memory as a typed slice.
  - For new columns in a schema, create empty files.
- **Append Writes**:
  - `put(&mut self, row: Vec<ScalarValue>)` appends values by writing to file end and remapping or tracking appended region.
  - In-memory buffers may be used for small writes; flushed to disk automatically.
- **API**:
  - `init()`: load or create column mmaps.
  - `put(row)`: append to column files.
  - `count()`: number of rows from file lengths.
  - `get(index)`: read ith entry from each column slice.

### Phase 2: Query Engine Basics
- REPL commands:
  - `put <table> (…);`, `count <table>`, `get <table> <index>`.
  - Hook into QParser to parse command keywords and call QStore API.
- Query primitives:
  - `select <expr-list> from <table> [where <pred>]`.
  - Full table scans: filter predicate per-row, projection of columns.

### Phase 3: Tables, Dictionaries & Aggregations
- QParser:
  - Dictionary type (`!`), table creation (`flip` of dict-of-vectors).
  - SQL-like (`select`, `update`, `delete`, `insert`).
  - Aggregations (`sum`, `avg`, `count i by col`).
- Execution:
  - Interpreter over columnar data: group-by uses hash map of key → accumulator.9

### Phase 4: I/O, Namespaces & Background
- Commands: `save`, `load`, file I/O primitives (`read0`), system commands (`\\`).
- Namespaces (`.ns.var`), workspace persistence.

### Phase 5: IPC & Advanced Functions
- IPC server via TCP for remote queries (`hopen`).
- Datatypes: date, timestamp, timespan; functions for time.
- Advanced joins (`aj`, `lj`, `wj`), iterators (`scan`, `over`), error trap (`'`).

### Phase 6: Block Encoding & Performance
- Per-column codecs: delta, RLE, dictionary for tags, bit-packing + compression.
- List/table attributes (`s#`, `u#`, `g#`, `p#`) to optimize scans.
- SIMD vectorized kernels, profiling tools.

## Next Steps
1. Replace existing docs with this unified plan.
2. Begin Phase 0 scaffolding in `qparser` and `repl`.
3. Progress through storage implementation in `qstorage`.

*Note: previous LSM-based docs are deprecated; this plan centers on memory-mapped splayed tables.*  