# Columnar LSM-Tree Implementation Plan

This document lays out the core components of an LSM-tree engine tailored for a column-oriented, KDB-style time-series database.  For each component, we indicate what we can KEEP from a traditional LSM design and what we must REPLACE or adapt.

## Components Overview
Component                | Status   | Notes
------------------------ | -------- | --------------------------------------------------------------
Memtable layer          | REPLACE  | Switch from SkipMap to per-column in-memory vector/array buffers (e.g. Arrow2 builders or Bump-allocated arrays).
Write-Ahead Log (WAL)   | KEEP*    | Retain framing/checksum design; adapt record format to batch or column-batch appends.
SSTable format          | REPLACE  | Move from row-KV files to per-column chunk files with columnar compression, zone-maps, and optional bloom filters.
Manifest / metadata     | KEEP     | Track on-disk column files, versions, and compaction history in a small manifest or directory layout.
Bloom filters           | KEEP*    | Still useful to filter SSTable chunks by time or partition key; may supplement with min/max indexes.
Background scheduler    | KEEP     | Flush and compaction workers remain, just operating on column-chunk runs instead of KV runs.
Cache management        | KEEP     | Retain concept of block/index cache for column blocks.
Lock-free SkipList      | REPLACE  | No longer needed once memtable is a packed array per column.
Epoch GC                | REPLACE  | Drop crossbeam-epoch; memory reclamation handled by bump allocator lifetimes.
Ring buffer             | REPLACE  | Client queues can use channels or simpler SPSC ringbuf for task coordination.
Node allocator          | REPLACE  | Use `bumpalo::Bump` for fast arena allocation within column builders.

_* WAL and Bloom filters will need modest format changes but remain conceptually the same._

## Phase 1: Foundation & In-Memory Path
- Project scaffolding (Cargo layout, modules, common types)
- Integrate `bumpalo` and set up per-column builders in a `Memtable` struct:
  - Use Arrow2 `ArrayBuilder` for each column in the MVP to leverage robust, type-safe builders.
  - If custom arena allocation is needed, build columns in bump-allocated vectors and convert or copy them into Arrow2 arrays on flush.
- Implement `WAL` API: open/create, append framed batch records, sync/flush
- Wire `put(batch)` and `remove(batch)` to append WAL then buffer into column builders

## Phase 2: On-Disk Format & Flush
- Define SSTable chunk file format:
  • Per-column sorted data blocks
  • Columnar compression (delta, dictionary, RLE)
  • Block metadata: min/max time, row counts, bloom filter or zone-map
- Implement `SSTable::write(path, &Memtable)` to dump column builders to disk
- Add `Manifest` or directory layout to discover and load SSTables
- Implement `LSMTree::flush()` to snapshot memtable → SSTable, then reset memtable and rotate WAL

## Phase 3: Read Path
- **Point lookup**: check in-memory buffers first, then consult SSTables (newest → oldest), use bloom/zone-map to skip files, binary-search within blocks
- **Range scan**: vectorized merge-scan across matching column chunks, returning aligned columnar batches

## Phase 4: Recovery
- On startup, replay WAL into fresh memtable
- Load existing SSTable metadata from manifest
- Ensure consistent state: rotate or re-apply incomplete flushes based on WAL vs manifest

## Phase 5: Compaction
- Trigger when SSTable count or disk footprint exceeds threshold
- Merge sorted column chunks (multi-way merge per column) into new chunk files
- Atomically swap old files for new ones in the manifest or via directory rename
- Support dropping obsolete time ranges or expired versions

## Phase 6: Advanced Features
- **Atomic multi-column writes**: group columns in single WAL record and flush as an atomic unit in manifest
- **Versioning & time travel**: keep old versions in SSTables; expose snapshot queries over time bounds
- **TTL & retention**: compaction policy to drop data older than retention window
- **Background monitoring**: metrics for write/read latency, compaction throughput, disk usage

## Success Criteria
- End-to-end write/read correctness for multi-column batches
- Crash recovery with zero data loss up to last WAL sync
- Range-scan performance competitive (<100μs per record) on vectorized reads
- Compaction throughput ≥ 50 MB/s per column
- Memory overhead minimal (in-memory builder size = batch size)

This plan preserves all the orchestration and tooling of a classic LSM tree (WAL, flush, compaction, background workers) while replacing the core data layout from row-KV skip lists to packed, columnar chunk files—enabling a true KDB+-style analytic engine.

## 7. Design Decisions & Best-Practice Recommendations

Below are recommended designs and formats for key components, drawing on Apache Arrow best practices and general LSM conventions.

### 1. WAL Record & Framing Schema
- **File naming & rotation**: Use names like `wal-<timestamp>.log`; rotate when size > 64 MiB or after each flush.
- **Record header**: A small fixed header per batch:
  - 4-byte magic `"LSM1"` (ASCII)
  - 1-byte format `version` (start at 1)
  - 4-byte `payload_length` (big-endian `u32`)
  - 4-byte `CRC32C` of payload
- **Payload**: Encoded column-batch:
  - `u16` number of columns
  - For each column:
    - `u16` column ID
    - `u32` data length
    - raw bytes (e.g. Arrow IPC for that column)
- **Versioning**: Bump the 1-byte version field when format changes; readers ignore unknown future versions if backwards-compatible.

### 2. Manifest / Metadata Layout
- **Single JSON manifest** (`manifest.json`) in the database root.
- **Atomic update**: Write `manifest.json.tmp`, `fsync`, then native `rename` → `manifest.json`, and `fsync` parent dir.
- **Schema**: JSON array of SSTable descriptors:
  ```json
  {
    "id": "20240215T123456Z",
    "columns": ["time","price","volume"],
    "time_min": 1676460896,
    "time_max": 1676464496,
    "path": "sst/20240215T123456Z/",
    "block_size": 65536,
    "compression": "lz4",
    "bloom_fpp": 0.01
  }
  ```
- **Discovery**: On startup, parse `manifest.json`; ignore any directories/files not listed. Clean up stray temp files.

### 3. Column Chunk File Format
- **What are packed, columnar chunk files?**
  Instead of writing rows interleaved in a single file, we slice the table into fixed-size row ranges (e.g. 64 Ki rows) called chunks. For each chunk we emit one file per column (or grouped per-column blocks), packing values of that column contiguously with no per-row overhead. This layout:
    - Enables one large, sequential read + decompression per column for high-throughput scans
    - Maximizes compression efficiency (delta, dictionary, RLE) on homogeneous data
    - Allows metadata-driven skipping (zone maps, min/max values, bloom filters) at chunk or sub-chunk granularity

- **Container**: Use Arrow2 IPC File format for each column-chunk file—no reinventing serialization.
- **Row groups**: Split data into row groups of ~64 Ki rows (configurable).
- **Zone maps**: Store `min`/`max` time (or primary key) per row group in the IPC metadata.
- **Compression**: Use LZ4 or Zstd (configurable); apply per-arrow-chunk (row group).

### 4. Configuration & Tuning Parameters
- **Memtable flush**: default at 64 MiB total across all column builders or 1 M rows, whichever first.
- **Compaction**: trigger when >4 SSTables exist in a level or total SSTable size >256 MiB.
- **Background threads**: default to `max(1, num_cpus() / 4)` for flush and compaction independently.

### 5. Error & Crash Recovery Semantics
- **WAL writes**: open in `O_APPEND` mode, `write_all` + `fsync` per batch (or periodically if under high throughput).
- **Startup**: replay all `wal-*.log` files in lexicographic order, then delete those fully applied after manifest flush.
- **SSTable writes**: write to `.../*.tmp` directory, `fsync` all files + directory, then atomically rename directory.
- **Cleanup**: on startup, remove leftover `*.tmp` files/directories.

### 6. Testing Strategy
- Unit tests for:
  - WAL framing/CRC logic
  - Manifest read/write and atomic rename
  - Arrow2 builder → chunk file serialization → deserialization
  - Bloom filter and zone-map predicates
- Integration tests:
  - Single-threaded: write → flush → get → recover → get
  - Multi-threaded: concurrent writes + reads
  - Crash simulation: kill process during flush/compaction, restart & verify
- Benchmarks:
  - Write throughput (records/sec, MiB/sec)
  - Range-scan latency (μs per record)
  - Compaction throughput (MiB/sec)

### 7. Metrics & Observability
- Expose Prometheus metrics via an optional HTTP endpoint:
  - Counters: `writes_total`, `reads_total`, `compactions_total`
  - Gauges: `memtable_size_bytes`, `sstable_count`
  - Histograms: `wal_write_duration_seconds`, `flush_duration_seconds`, `compaction_duration_seconds`

With these conventions in place, the implementation can proceed with clear schemas and robust crash-consistency guarantees.
