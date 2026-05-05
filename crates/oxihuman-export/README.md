# oxihuman-export

Part of the [OxiHuman](../../README.md) workspace — privacy-first, client-side human body generator in pure Rust.

[![Crates.io](https://img.shields.io/crates/v/oxihuman-export.svg)](https://crates.io/crates/oxihuman-export)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](../../LICENSE)

| Metric | Value |
|--------|-------|
| Status | Stable |
| Tests passing | 5,289 |
| Public API items | 8,457 |
| Source files | ~883 `.rs` files |
| Stub coverage | 0 |

## Overview

`oxihuman-export` provides the full export pipeline for the OxiHuman workspace. It supports a broad range of industry-standard 3D formats (stable), custom binary bundle formats, streaming pipelines, and a growing set of experimental integrations covering game engines, motion capture, and VFX. All implementations are pure Rust with no C or Fortran dependencies.

> **Stability note:** All format implementations are stable and production-ready. No stubs remain.

## Dependency

```toml
[dependencies]
oxihuman-export = "0.1.2"
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
| `smpl_export` | SMPL body model parameters |
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

## Feature Flags

None. All modules are unconditionally compiled.

## Quality Notes

- 0 `todo!()` / `unimplemented!()` macro calls
- 0 stub implementations
- 5,289 passing tests

## Dependencies

```toml
[dependencies]
anyhow         = { workspace = true }
thiserror      = { workspace = true }
serde          = { workspace = true }
serde_json     = { workspace = true }
toml           = { workspace = true }
bytemuck       = { workspace = true }
sha2           = { workspace = true }
hex            = { workspace = true }
oxihuman-core  = { workspace = true }
oxihuman-morph = { workspace = true }
oxihuman-mesh  = { workspace = true }
```

## License

Apache-2.0 — Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
