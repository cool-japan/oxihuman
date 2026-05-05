# oxihuman-morph

Morphing engine for the [OxiHuman](../../README.md) workspace — privacy-first, client-side human body generator in pure Rust.

**Status:** Stable | **Tests:** 5,865 passing | **Version:** 0.1.2 | **Updated:** 2026-05-05

---

## Overview

`oxihuman-morph` is the primary morphing subsystem of OxiHuman. It exposes a high-level `HumanEngine` orchestrator backed by over 30 specialized modules covering the full lifecycle of human body morphing: parameter management, blending, constraint solving, facial expressions, age and skin-color modeling, animation retargeting, preset I/O, and more.

Heavy computation paths (mesh delta application, population sampling, parallel blending) are parallelized transparently via **rayon**, allowing multi-core utilization without any changes to the calling code.

The crate contains zero `todo!()`/`unimplemented!()` calls; every declared public item is fully implemented.

---

## Dependency

```toml
[dependencies]
oxihuman-morph = "0.1.2"
```

No feature flags are required; all subsystems are included by default.

---

## Quick Start

```rust
use oxihuman_morph::engine::HumanEngine;

let mut engine = HumanEngine::default();
engine.apply_preset("adult_female_average")?;
let mesh = engine.generate()?;
```

---

## Module Reference

| Module | Description |
|---|---|
| `engine` | `HumanEngine` — the central morphing orchestrator; coordinates all subsystems and owns the parameter state |
| `age_model` | Age and lifecycle progression — models continuous aging curves from infant through elder |
| `anthropometry` | Body measurements and randomization — height, proportions, limb ratios, population-aware sampling |
| `apply` | Target application pipeline — applies weighted morph targets to a base mesh in dependency order |
| `blend_profile` | Blending profiles — named collections of blend weights with interpolation metadata |
| `blend_tree` | Hierarchical parameter blending — tree-structured blend nodes for layered morph composition |
| `cache` | Mesh caching — stores computed mesh states keyed by parameter hashes to avoid recomputation |
| `delta_cache` | Delta caching — caches incremental mesh deltas for fast incremental re-application |
| `compress` | Delta compression — compact encoding of sparse vertex displacement arrays |
| `constraint` | Morphing constraints — enforces anatomical limits and mutual-exclusion rules between targets |
| `curves` | Interpolation curves — Bezier, Hermite, and stepped curves for parameter-to-weight mapping |
| `weight_curves` | Weight function curves — specialized curves mapping scalar inputs to blend weights |
| `diversity` | Population sampling — generates statistically representative parameter distributions |
| `expression` | Facial expressions — FACS action unit system for procedural and keyframed expressions |
| `fitting` | Parameter fitting — numerical optimization to fit parameters to target measurements or shapes |
| `history` | Change history — records parameter change events for audit and undo support |
| `interpolate` | Keyframe interpolation — time-based interpolation between discrete parameter snapshots |
| `params` | Parameter state management — typed parameter map with validation, diffing, and snapshotting |
| `pose_blend` | Pose-driven blending — activates corrective morphs based on skeletal pose state |
| `preset_io` | Preset serialization — import/export of parameter presets to/from JSON and TOML |
| `presets` | Built-in preset library — curated factory presets for common archetypes and demographics |
| `regions` | Body regions and tags — named region definitions and tag-based target filtering |
| `search` | Target searching — full-text and tag-based search across the loaded target library |
| `session` | Morph session management — isolates parameter state per session for concurrent use cases |
| `shape_compare` | Shape comparison — metrics and diff visualization between two mesh states |
| `skin_color` | Skin color modeling — Fitzpatrick scale integration, melanin/hemoglobin parameterization |
| `symmetry` | Mirror and symmetry operations — bilateral symmetry enforcement and asymmetry blending |
| `target_lib` | Target library statistics — aggregated metadata and coverage reports for loaded targets |
| `timeline` | Animation timeline — keyframe track management and playback cursor for morph animations |
| `anim_retarget` | Animation retargeting — maps animation curves from a source rig to OxiHuman parameters; includes `BvhData`, `parse_bvh_text`, and `retarget_bvh_to_param_tracks` for BVH animation bridge (added v0.1.2) |
| `fabrik_ik` | FABRIK inverse kinematics — `IkChain` with `solve_fabrik` and `solve_constrained_fabrik` for real-time IK solving (added v0.1.2) |
| `secondary_motion` | XPBD secondary motion — `SecondaryMotionSystem`, `XpbdParticle`, and `SecondaryConstraint` for physics-driven secondary animation (added v0.1.2) |
| `mutation_engine` | Morphological mutation — stochastic perturbation of parameters for generative diversity |

---

## Parallelism

`oxihuman-morph` uses [rayon](https://docs.rs/rayon) for data-parallel workloads. The following operations run across all available CPU cores automatically:

- Morph target application (`apply` module)
- Population sampling (`diversity` module)
- Delta compression and decompression (`compress`, `delta_cache`)
- Batch blend-tree evaluation (`blend_tree`)

No thread-pool configuration is required; rayon's global pool is used by default.

---

## Key Dependencies

| Crate | Purpose |
|---|---|
| `anyhow` | Ergonomic error propagation |
| `thiserror` | Typed error definitions |
| `serde` / `serde_json` | Serialization / deserialization of parameters and presets |
| `oxihuman-core` | Core infrastructure (spatial index, event bus, asset cache, task graph, etc.) |
| `rayon` | Data-parallel iteration for multi-core morphing workloads |

---

## Stability

All public items follow semantic versioning. The 0.1.2 release is considered stable for downstream consumption within the OxiHuman workspace. Breaking changes will be accompanied by a minor-version bump until a 1.0 release is declared.

---

## License

Apache-2.0 — Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
