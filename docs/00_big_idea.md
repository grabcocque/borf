# Chapter 0: The Big Idea

In the world of high-performance, time-series analytics, two domains have long converged:

- **BorF**: an expressive, Q-inspired query language for on-the-fly analysis of streaming and historical data.
- **Lsmer**: a column-oriented, memory mapped splayed-table engine (inspired by kdb+) that delivers lightning-fast writes and low-latency reads.

This project unites these two pillars under a single, open-source Rust-ecosystem:

1. **BorF** aims to provide a compact, array-centric syntax:
   ```text
   borf> select avg(price), sum(size) by symbol from trades where time between 2023-01-01 and 2023-02-01
   ```
   - Vectorized evaluation over in-memory and on-disk columnar chunks.
   - Built-in support for time-based window functions, joins, and aggregations.

2. **Splayed Tables** storage engine:
   - **Memory-Mapped Column Files**: each column is stored in its own file under the data directory and memory-mapped for fast random and sequential access.
   - **In-Memory Arrays**: on initialization, column files are loaded into native Rust vectors for vectorized scans, aggregations, and filtering.
   - **Append-Only Writes**: new rows are appended directly to column files without a separate WAL, ensuring durability and simple crash recovery.
   - **Zero-Copy Queries**: by mapping column files, queries can operate directly on on-disk buffers without additional copying, achieving high throughput.

### Why BorF + Lsmer?
- **Performance**: Like kdb+, BorF speaks directly to packed arrays; Lsmer feeds it contiguous column buffers for maximum throughput.
- **Safety & Reliability**: Rust's ownership and type system ensure memory and thread safety without locking overhead.
- **Open Extensibility**: modular design makes it easy to add new columnar codecs, external data sources, or distributed coordination.

### How the Pieces Fit Together
```
   [1] Client submits BorF query
     ↓
   [2] BorF parser + planner compiles to a vectorized execution plan
     ↓
   [3] Plan operators pull data from storage:
       • In-memory vectors (recent writes)
       • Memory-mapped splayed tables on disk
     ↓
   [4] Operators apply filters, joins, aggregations in memory
     ↓
   [5] Results streamed back to the client
```

### Data Model

We represent both time series and graph data using a splayed-table layout (one file per column and in-memory arrays). For full details on schemas, node/edge tables, and query patterns, see [Data Model](data_model.md).

### Roadmap & Manual Structure
- **Chapter 1: Phase 1 Implementation** — Memory-Mapped Splayed-Table Storage (init, put, count, get).
- **Chapter 2: Phase 2 Implementation** — Basic Query Engine over splayed tables (filters, projections).
- **Chapter 3: Read Path & APIs** — Point lookups, range scans, and recovery.
- **Chapter 4: Compaction & Background Tasks** — Scheduling, merging, retention policies.
- **Chapter 5: BorF Query Engine** — Parser, planner, execution over columnar data.
- **Chapter 6: Advanced Features** — Distributed mode, multi-versioning, TTL, observability, and pluggable storage codecs.
- **Chapter 7: Block Encoding Layers** — Implement pluggable per-column codecs (delta encoding, run-length encoding, dictionary encoding, bit-packing) and compression for optimized storage and high-speed scans.

Together, these chapters document not just how Lsmer works under the hood, but why each design choice was made—and how you can leverage, customize, or extend it for your own big-data applications.
