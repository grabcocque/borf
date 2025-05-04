# Chapter 0: The Big Idea

In the world of high-performance, time-series analytics, two domains have long converged:

- **BorF**: an expressive, Q-inspired query language for on-the-fly analysis of streaming and historical data.
- **Lsmer**: a column-oriented, LSM-tree storage engine (inspired by kdb+) that delivers lightning-fast writes and low-latency reads.

This project unites these two pillars under a single, open-source Rust-ecosystem:

1. **BorF** aims to provide a compact, array-centric syntax:
   ```text
   borf> select avg(price), sum(size) by symbol from trades where time between 2023-01-01 and 2023-02-01
   ```
   - Vectorized evaluation over in-memory and on-disk columnar chunks.
   - Built-in support for time-based window functions, joins, and aggregations.

2. **Lsmer** forms the storage backbone:
   - **Columnar Chunks**: data is batched into 64K-row chunks per column, enabling SIMD-friendly scans and high compression.
   - **LSM Architecture**: a durable Write-Ahead Log for fast, crash-safe appends, an in-memory columnar Memtable, and on-disk SSTables for immutable, sorted runs.
   - **Compaction & Indexes**: background merging of chunks, bloom filters and zone maps for efficient skipping, and pluggable compression codecs.

### Why BorF + Lsmer?
- **Performance**: Like kdb+, BorF speaks directly to packed arrays; Lsmer feeds it contiguous column buffers for maximum throughput.
- **Safety & Reliability**: Rust’s ownership and type system ensure memory and thread safety without locking overhead.
- **Open Extensibility**: modular design makes it easy to add new columnar codecs, external data sources, or distributed coordination.

### How the Pieces Fit Together
```
   [1] Client submits BorF query
     ↓
   [2] BorF parser + planner compiles to a vectorized execution plan
     ↓
   [3] Plan operators pull data from Lsmer:
       • In-memory Memtable (recent writes)
       • On-disk SSTable chunks via Arrow2 arrays
     ↓
   [4] Operators apply filters, merges, aggregations in memory
     ↓
   [5] Results streamed back to the client
```

### Roadmap & Manual Structure
- **Chapter 1: Phase 1 Implementation** — Memtable, WAL, and basic put/flush.
- **Chapter 2: On-Disk SSTables** — Format, compression, indexing, and flush logic.
- **Chapter 3: Read Path & APIs** — Point lookups, range scans, and recovery.
- **Chapter 4: Compaction & Background Tasks** — Scheduling, merging, retention policies.
- **Chapter 5: BorF Query Engine** — Parser, planner, execution over columnar data.
- **Chapter 6: Advanced Features** — Distributed mode, multi-versioning, TTL, observability.

Together, these chapters document not just how Lsmer works under the hood, but why each design choice was made—and how you can leverage, customize, or extend it for your own big-data applications.
