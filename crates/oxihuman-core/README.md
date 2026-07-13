# oxihuman-core

Core infrastructure for the [OxiHuman](../../README.md) workspace — privacy-first, client-side human body generator in pure Rust.

**Status:** Stable | **Tests:** 5,571 passing | **Version:** 0.2.2 | **Updated:** 2026-07-13

---

## Overview

`oxihuman-core` provides the foundational subsystems shared across all OxiHuman crates. It handles asset management, data validation, event dispatch, plugin lifecycle, spatial indexing, task orchestration, configuration schemas, and more. Every higher-level crate in the workspace depends on `oxihuman-core` as its common ground.

The crate is purely declarative in its public API surface — no hidden global mutable state, no `unsafe` beyond tightly scoped internals, and zero `todo!()`/`unimplemented!()` calls in the production code path.

---

## Dependency

```toml
[dependencies]
oxihuman-core = "0.2.2"
```

All subsystems are included by default with no feature flags required, except the optional `net` feature (off by default), which enables a `tokio`-backed TCP networking module for streaming/collaboration use cases.

---

## Module Reference

| Module | Description |
|---|---|
| `category` | Target categorization — enum-based classification of morph targets (mirrors MakeHuman's directory layout), with string parsing and a content-safety check |
| `integrity` | Data validation utilities — structural checks, checksum verification, invariant enforcement |
| `manifest` | Asset manifest management — versioned manifest of allowed targets, policy profile, and pack metadata |
| `pack_verify` | Pack verification — validates pack archives against recorded per-file checksums |
| `pack_sign` | Cryptographic signing — SHA-256-based pack signing and signature management |
| `parser` | Parsing utilities — shared lexer/parser primitives used across config and asset formats |
| `policy` | Policy management — runtime enforcement of data-access and generation policies |
| `report` | Pipeline reporting — structured result and diagnostic reporting for build/export pipelines |
| `target_index` | Target indexing and scanning — builds and queries a searchable index of morphing targets |
| `plugin_registry` | Plugin lifecycle management — registration, enumeration, and teardown of plugins |
| `plugin_api` | Plugin API surface — versioned plugin metadata, lifecycle state, and dependency-ordered activation |
| `event_bus` | Event publishing/subscribing — synchronous in-process event bus with typed subscriptions |
| `asset_hash` | Asset hashing — content-addressed hashing and identity comparison for binary assets |
| `asset_cache` | Asset registry and LRU caching — in-memory cache with configurable eviction policy |
| `workspace` | Workspace configuration — loading, validation, and resolution of workspace-level settings |
| `metrics` | Metrics collection — counters, gauges, and histograms for internal instrumentation |
| `spatial_index` | Octree spatial indexing — 3-D point/region queries used by mesh and physics subsystems |
| `command_bus` | Command execution with undo/redo — dispatches typed commands and maintains a revertible history |
| `task_graph` | Task dependency graph — DAG-based scheduler for ordered, sequential pipeline execution |
| `config_schema` | Configuration schema validation — JSON-Schema-compatible validation for TOML/JSON configs |
| `undo_redo` | Undo/redo stack — standalone reversible-action stack with bounded history depth (independent of `command_bus`'s own undo/redo fields) |

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

All public items in this crate follow semantic versioning. The 0.2.2 release is considered stable for downstream consumption within the OxiHuman workspace. Breaking changes will be accompanied by a minor-version bump until a 1.0 release is declared.

---

## License

Apache-2.0 — Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
