# oxihuman-core -- TODO

> Version: 0.1.1 | Updated: 2026-03-13

## Status: Stable

Core foundation crate. ~829 source files, ~175k lines. 0 stubs. 5,243 passing tests.

## Completed

### Part 1: Core Infrastructure
- [x] Policy system (Policy, PolicyProfile: Standard/Strict content filtering)
- [x] Asset manifest and integrity verification
- [x] OBJ / MHCLO / .target file parsers
- [x] Target index and scanner
- [x] Asset pack builder, distributor, signer, verifier
- [x] Plugin registry and plugin API (lifecycle, dependencies, versioning)
- [x] Event bus (param changes, export, errors)
- [x] Event log with severity filtering and JSON export
- [x] Asset hashing (SHA-256 based)
- [x] Asset cache with LRU eviction and hit-rate tracking
- [x] Workspace management and configuration
- [x] Metrics registry
- [x] Command bus with undo/redo
- [x] Undo/redo stack
- [x] Task graph (topological ordering, critical path, sequential execution)
- [x] Config schema with validation and defaults
- [x] Binary and JSON serialization utilities
- [x] Resource manager with garbage collection
- [x] Hot reload watcher
- [x] Debug console
- [x] Data pipeline with stage tracking
- [x] Type registry
- [x] Localization (multi-locale string tables, JSON export)

### Part 2: Data Structures and Algorithms
- [x] Tree index, trie map, quad tree
- [x] Type alias map, type cache, type erased storage
- [x] UID generator with recycling
- [x] Union-find (disjoint sets)
- [x] Update queue, value cache, value map, variable store
- [x] Color palette
- [x] Bitmask operations
- [x] Version vector (causal ordering)
- [x] Text tokenizer
- [x] Rolling statistics
- [x] Bloom filter
- [x] Escape/unescape utilities (HTML, JSON, URL)
- [x] Number formatting (SI, bytes, separators)
- [x] Delta encoder (zigzag encoding)
- [x] Run-length encoding
- [x] CRC/checksum (CRC-8, CRC-16, Fletcher-16)
- [x] Argument parser
- [x] Pipeline stage execution
- [x] FSM builder
- [x] Decision tree
- [x] Graph coloring (greedy)

### Part 3: Utilities
- [x] Locale formatter, timezone offset, calendar utilities
- [x] Duration parser (ISO 8601), cron parser
- [x] Holiday calendars (JP, US), date ranges, fiscal year
- [x] Work/business calendar
- [x] Time series buffer, moving averages (SMA, EMA)
- [x] Trend detection, anomaly scoring, outlier filtering
- [x] Bucket histogram, quantile estimator (P2)
- [x] Feature flags and A/B test configuration
- [x] Experiment tracker
- [x] Metrics: counter, gauge, histogram SDK
- [x] Telemetry spans and distributed tracing
- [x] Log aggregator, audit log, change log
- [x] Notification queue, task scheduler, deadline tracker
- [x] Quota manager
- [x] Spatial index (octree with AABB/sphere/ray queries, KNN)
- [x] AABB tree (2D and 3D)

### Testing and Benchmarks
- [x] 5,243 unit tests passing
- [x] Criterion benchmarks (`core_bench`)
- [x] proptest dev-dependency available
