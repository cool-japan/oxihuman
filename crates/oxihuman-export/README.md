# oxihuman-export

Part of the [OxiHuman](../../README.md) workspace — privacy-first, client-side human body generator in pure Rust.

[![Crates.io](https://img.shields.io/crates/v/oxihuman-export.svg)](https://crates.io/crates/oxihuman-export)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](../../LICENSE)

| Metric | Value |
|--------|-------|
| Status | Stable |
| Tests passing | 5,579 |
| Public API items | 8,457 |
| Source files | ~883 `.rs` files |
| Stub coverage | 0 |

## Overview

`oxihuman-export` provides the full export pipeline for the OxiHuman workspace. It supports a broad range of industry-standard 3D formats (stable), custom binary bundle formats, streaming pipelines, and a growing set of experimental integrations covering game engines, motion capture, and VFX. All implementations are pure Rust with no C or Fortran dependencies.

> **Stability note:** All format implementations are stable and production-ready. No stubs remain.

## Dependency

```toml
[dependencies]
oxihuman-export = "0.2.2"
```

## Format Matrix

### Stable Formats

| Format | Module | Notes |
|--------|--------|-------|
| glTF 2.0 / GLB | `gltf` | Binary and JSON-separate; extensions: clearcoat, IOR, sheen, transmission, volume |
| COLLADA (.dae) | `collada` | Full geometry, materials, skeleton |
| Wavefront OBJ / MTL | `obj` | Multi-material, UV, normals |
| STL | `stl` | ASCII and binary variants |
| USD / USDA | `usd` | Time-sampled animation support; `BlendShapeTimeSamples` + `UsdaWriter::write_blend_shape_animation` for blend shape animation (v0.1.2) |
| VRM | `vrm` | Avatar format (VRM 0.x / 1.0) |
| 3MF | `threemf` | 3D Manufacturing Format |
| PLY | `ply` | Point cloud and mesh |
| X3D | `x3d` | XML-based 3D format |
| SVG | `svg` | 2D projection export |

### Asset Pack Format (OHPK v1)

`core_pack` implements **OHPK v1**, a self-describing binary container that
bundles a base mesh plus a set of sparse morph targets in one file: the
`oxihuman-cli pack-core` command *writes* one, and `oxihuman-wasm` *reads* it
back with zero-copy, no-filesystem, `wasm32`-safe parsing.

| Item | Detail |
|------|--------|
| Magic / version | `b"OHPK"`, version `1` (`OHPK_MAGIC`, `OHPK_VERSION`) |
| Compression | Body DEFLATE-compressed via `oxiarc-deflate` (flag `OHPK_FLAG_DEFLATE`, level `OHPK_DEFLATE_LEVEL = 9`) — no `zip`/`flate2` |
| Write API | `CorePackBuilder`: `set_base_mesh`, `set_helper_metadata`, `add_target`, `set_manifest`, `build() -> Result<Vec<u8>>` |
| Read API | `CorePack::parse(bytes)`: `manifest()`, `vertex_count()`, `base_positions()`, `base_indices()`, `base_uvs()`, `targets()`, `quantization_report()` |
| Deltas | Sparse, `i16` max-abs-quantised per-target position deltas (`CorePackTarget::sparse()` / `indices()`) |
| Shipped pack | [`assets/packs/oxihuman-core-v1.ohpk`](../../assets/packs) — 2,093,260 B, 38 CC0 morph targets, 21,833 base vertices, worst-case reconstruction error 0.011 mm |

### Advanced Pipeline Features

| Module | Description |
|--------|-------------|
| `animation` | Generic animation clip export |
| `gltf_anim` | glTF animation and blend shape export |
| `usd_anim` | USD time-sampled animation tracks |
| `vertex_anim` | Morph target / vertex animation sequences |
| `streaming_export` | Chunked position streaming (F16 / F32 / CSV) |
| `realtime_stream` | Real-time frame streaming for live sessions |
| `batch_pipeline` | Parameter grid batch processing |
| `job_queue` | Async export job queue with priority scheduling |
| `asset_bundle` | OXB binary bundle format |
| `morph_delta_bin` | OXMD morph delta binary format |
| `geometry_cache` | OXGC geometry cache format |
| `manifest_json` | Asset manifest generation with SHA-256 checksums |
| `texture` | Procedural texture generation (FBM, Voronoi, marble, wood) |
| `noise_tex` | Noise-based texture utilities |
| `report_html` | HTML pipeline reports with statistics |
| `web_export` | Web GL LOD mesh export |
| `openxr_scene` | OpenXR composition layer scene export |

### Experimental / Stub Formats (Alpha)

These modules are compiled into the crate and expose public APIs, but implementations are stubs pending full development.

**Game Engine Integrations**

| Module | Target |
|--------|--------|
| `unity_export` | Unity asset package |
| `unreal_export` | Unreal Engine asset |
| `babylon_export` | Babylon.js scene |
| `threejs_export` | Three.js JSON scene |
| `cocos_export` | Cocos Creator asset |

**Motion Capture**

| Module | Target |
|--------|--------|
| `mixamo_export` | Mixamo rig / animation |
| `mediapipe_export` | MediaPipe pose landmarks |
| `openpose_export` | OpenPose keypoint format |
| `cmu_motion_export` | CMU Motion Capture database format |

**VFX & Simulation**

| Module | Target |
|--------|--------|
| `particles_export` | Particle system data |
| `fog_export` | Volumetric fog |
| `smoke_export` | Smoke simulation cache |
| `fire_export` | Fire simulation cache |
| `fluid_export` | Fluid simulation cache |
| `lightning_export` | Procedural lightning paths |

**Advanced / Industry**

| Module | Target |
|--------|--------|
| `cgns_export` | CGNS CFD data format |
| `openvdb_export` | OpenVDB sparse volumes |
| `houdini_export` | Houdini geometry (.bgeo) |
| `maya_export` | Maya ASCII / binary (.ma / .mb) |
| `alembic_export` | Alembic (.abc) geometry cache |
| `draco_export` | Google Draco compressed mesh |

## Export Safety Gate

Every human-facing exporter entry point — GLB, glTF-separate, OBJ, STL, VRM,
COLLADA, USD, 3MF, PLY, FBX, X3D, Alembic, LOD packs, and the `auto_export`
router — calls the single gate `export_gate::ensure_export_allowed` before
serialising a mesh. The gate refuses (`Err`) any `oxihuman_mesh::MeshBuffers`
whose `has_suit` flag is `false`, so an unclothed human mesh can never be
written to disk or returned to a caller. The cross-crate regression test
[`invariant_no_nude_mesh_stage`](../oxihuman-tests/tests/invariant_no_nude_mesh_stage.rs)
in `oxihuman-tests` iterates every gated entry point and asserts each one
rejects an unsuited mesh.

Byte-level builders that never touch the filesystem (`build_glb_bytes`,
`build_glb_with_meta_bytes`, `build_glb_with_skeleton_bytes`,
`mesh_to_obj_string`, `mesh_to_stl_ascii`, `encode_stl_binary`,
`VrmExporter::export`, …) sit alongside the `Path`-based `export_*`
convenience wrappers. `oxihuman-wasm`'s browser exporters call these in-memory
builders directly, which is what keeps them working on `wasm32` (no
filesystem, no `std::env::temp_dir()`).

## Feature Flags

None. All modules are unconditionally compiled.

## Quality Notes

- 0 `todo!()` / `unimplemented!()` macro calls
- 0 stub implementations
- Nude-mesh export gate enforced on every human-facing exporter entry point (see [Export Safety Gate](#export-safety-gate))
- 5,579 passing tests

## Dependencies

```toml
[dependencies]
anyhow         = { workspace = true }
thiserror      = { workspace = true }
serde          = { workspace = true }
serde_json     = { workspace = true }
toml           = { workspace = true }
bytemuck       = { workspace = true }
oxiarc-deflate = { workspace = true }
oxiarc-archive = { workspace = true }
sha2           = { workspace = true }
hex            = { workspace = true }
oxihuman-core  = { workspace = true }
oxihuman-morph = { workspace = true }
oxihuman-mesh  = { workspace = true }
```

## License

Apache-2.0 — Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
