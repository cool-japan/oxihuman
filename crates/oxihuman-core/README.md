# oxihuman-core

Core infrastructure for the [OxiHuman](../../README.md) workspace — privacy-first, client-side human body generator in pure Rust.

**Status:** Stable | **Tests:** 5,243 passing | **Version:** 0.1.1 | **Updated:** 2026-03-13

---

## Overview

`oxihuman-core` provides the foundational subsystems shared across all OxiHuman crates. It handles asset management, data validation, event dispatch, plugin lifecycle, spatial indexing, task orchestration, configuration schemas, and more. Every higher-level crate in the workspace depends on `oxihuman-core` as its common ground.

The crate is purely declarative in its public API surface — no hidden global mutable state, no `unsafe` beyond tightly scoped internals, and zero `todo!()`/`unimplemented!()` calls in the production code path.

---

## Dependency

```toml
[dependencies]
oxihuman-core = "0.1.1"
```

No feature flags are required; all subsystems are included by default.

---

## Module Reference

| Module | Description |
|---|---|
| `category` | Target categorization system — hierarchical tag assignment and lookup for morphing targets |
| `integrity` | Data validation utilities — structural checks, checksum verification, invariant enforcement |
| `manifest` | Asset manifest management — tracks asset lists, versions, and dependency declarations |
| `pack_verify` | Pack verification — validates pack archives against embedded checksums and signatures |
| `pack_sign` | Cryptographic signing — SHA-256-based pack signing and signature management |
| `parser` | Parsing utilities — shared lexer/parser primitives used across config and asset formats |
| `policy` | Policy management — runtime enforcement of data-access and generation policies |
| `report` | Pipeline reporting — structured result and diagnostic reporting for build/export pipelines |
| `target_index` | Target indexing and scanning — builds and queries a searchable index of morphing targets |
| `plugin_registry` | Plugin lifecycle management — registration, enumeration, and teardown of plugins |
| `plugin_api` | Plugin API surface — trait definitions and versioned interfaces consumed by plugins |
| `event_bus` | Event publishing/subscribing — synchronous in-process event bus with typed subscriptions |
| `asset_hash` | Asset hashing — content-addressed hashing and identity comparison for binary assets |
| `asset_cache` | Asset registry and LRU caching — in-memory cache with configurable eviction policy |
| `workspace` | Workspace configuration — loading, validation, and resolution of workspace-level settings |
| `metrics` | Metrics collection — counters, gauges, and histograms for internal instrumentation |
| `spatial_index` | Octree spatial indexing — 3-D point/region queries used by mesh and physics subsystems |
| `command_bus` | Command execution with undo/redo — dispatches typed commands and maintains a revertible history |
| `task_graph` | Task dependency graph — DAG-based scheduler for ordered, concurrent pipeline execution |
| `config_schema` | Configuration schema validation — JSON-Schema-compatible validation for TOML/JSON configs |
| `undo_redo` | Undo/redo stack — standalone reversible-action stack consumed by `command_bus` and editors |

---

## Key Dependencies

| Crate | Purpose |
|---|---|
| `anyhow` | Ergonomic error propagation |
| `thiserror` | Typed error definitions |
| `serde` / `serde_json` | Serialization / deserialization |
| `toml` | TOML configuration parsing |
| `sha2` | SHA-256 hashing for pack signing and asset identity |
| `hex` | Hex encoding/decoding for digests |

---

## Stability

All public items in this crate follow semantic versioning. The 0.1.1 release is considered stable for downstream consumption within the OxiHuman workspace. Breaking changes will be accompanied by a minor-version bump until a 1.0 release is declared.

---

## License

Apache-2.0 — Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
