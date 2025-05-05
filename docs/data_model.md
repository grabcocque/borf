 # Data Model: Time Series & Graph Data

 This document defines how the storage engine represents both time series and graph data using the splayed-table layout, enabling efficient storage and Q-style combinatorial queries.

 ## Time Series

 Time series data is stored as a single splayed table: one column file per field.
 - **Columns**:
   - `time`: Int64 UNIX nanoseconds since epoch.
   - Measure columns (e.g., `price`: Float64, `size`: Int64).
   - Tag columns (e.g., `symbol`: Utf8 string), later dictionary-encoded.

 Rows are appended by supplying a vector of `ScalarValue` in the same column order:
 ```text
 q> put (time;symbol;price;size)
 q> put (1672000000000;`AAPL;172.33;100)
 ```

 On disk, each column is a file under `data_dir/<column_name>`, containing little-endian binary values.

 ## Graph Data

 Graphs are represented by two logical splayed tables: **nodes** and **edges**.

 ### Nodes Table
 - `id` (Int64): unique node identifier.
 - Additional property columns per node (e.g., `name`: Utf8, `type`: Utf8).

 ### Edges Table
 - `src`, `dst` (Int64): source and destination node IDs.
 - Additional edge property columns (e.g., `weight`: Float64, `label`: Utf8).

 Rows inserted with separate commands:
 ```text
 q> put_nodes ((id;name);(1;`Alice);(2;`Bob))
 q> put_edges ((src;dst;weight);(1;2;1.0))
 ```

 ## Query Patterns

 - **Fetch nodes**:
   ```text
   q> select name from nodes where id = 1
   ```
 - **Traverse edges**:
   ```text
   q> select dst from edges where src = 1
   q> select name from nodes where id in select dst from edges where src = 1
   ```

 These operations compile to columnar scans over the appropriate splayed tables, taking advantage of in-memory vectors for speed.

 ## Block-Encoding Layers (Future)

 To further optimize storage and I/O, we plan to introduce pluggable per-column codecs:
 - **Delta Encoding** for sorted time stamps, with varint packing.
 - **Run-Length Encoding (RLE)** for repeated values or low-cardinality columns.
 - **Dictionary Encoding** for string/tag columns.
 - **Bit-Packing** and compression (e.g., LZ4, Zstd) for binary column files.

 These codecs will be configurable via `QStoreConfig` and applied per column to achieve KDB+-like efficiency.