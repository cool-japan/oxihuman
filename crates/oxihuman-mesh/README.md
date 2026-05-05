# oxihuman-mesh

Part of the [OxiHuman](../../README.md) workspace — privacy-first, client-side human body generator in pure Rust.

[![Crates.io](https://img.shields.io/crates/v/oxihuman-mesh.svg)](https://crates.io/crates/oxihuman-mesh)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](../../LICENSE)

| Metric | Value |
|--------|-------|
| Status | Stable |
| Tests passing | 5,715 |
| Public API items | 7,381 |
| Source files | 898 `.rs` files |
| Stub coverage | 0 stubs |

## Overview

`oxihuman-mesh` is the geometry backbone of the OxiHuman pipeline. It provides a complete suite of mesh processing algorithms — from raw buffer management and skinning through advanced topology analysis, UV generation, and volumetric methods — all in safe, pure Rust with no C or Fortran dependencies.

## Dependency

```toml
[dependencies]
oxihuman-mesh = "0.1.2"
```

## Module Reference

### Core Mesh Operations

| Module | Description |
|--------|-------------|
| `mesh` | Core `MeshBuffers` definition — positions, normals, UVs, indices |
| `groups` | Vertex groups and group mapping |
| `bounds` | Axis-aligned bounding box computation |
| `measurements` | Body and mesh sizing utilities |
| `normals` | Normal computation and smoothing |
| `tangents` | Tangent frame generation (MikkTSpace-compatible) |
| `skinning` | Linear blend skinning (LBS) pipeline |
| `weights` | Weight painting tools and normalization |

### Geometry Processing

| Module | Description |
|--------|-------------|
| `decimate` | Mesh decimation (quadric error metrics) |
| `catmull_clark` | Catmull-Clark subdivision surfaces |
| `subdivide` | Loop and midpoint subdivision |
| `remesh` | Edge collapse remeshing |
| `convex_hull` | Convex hull generation |
| `marching_cubes` | Volumetric surface extraction (marching cubes) |
| `voxelize` | Solid and surface voxelization |
| `terrain` | Height field terrain generation |

### Topology & Connectivity

| Module | Description |
|--------|-------------|
| `connectivity` | Connected component detection, valence computation |
| `edge_loops` | Edge chain and loop extraction |
| `seam_cut` | UV seam cutting and island splitting |
| `winding` | Winding number calculation |
| `visibility` | Frustum culling and backface testing |

### UV & Texturing

| Module | Description |
|--------|-------------|
| `uvgen` | UV projection and transformation |
| `atlas` | UV island packing and atlasing |
| `uv_quality` | Stretch and overlap analysis |
| `displacement_map` | Displacement map baking |
| `normal_map_bake` | Normal map baking from high-res meshes |

### Deformation & Animation

| Module | Description |
|--------|-------------|
| `dqs` | Dual quaternion skinning (DQS) |
| `skeleton` | Bone hierarchy and skeleton data |
| `pose_library` | Pose storage and blending |
| `ik` | FABRIK and 2-bone IK solver |
| `retarget` | Skeletal retargeting between rigs |
| `ffd` | Free-form deformation (FFD) lattice |
| `spring_deform` | Spring-based secondary deformation |

### Analysis & Advanced

| Module | Description |
|--------|-------------|
| `curvature` | Gaussian and mean curvature estimation |
| `geodesic` | Geodesic distance computation (heat method) |
| `bvh` | Bounding volume hierarchy for ray casting |
| `smooth` | Laplacian and Taubin mesh smoothing |
| `ao_bake` | Ambient occlusion baking |
| `sampling` | Poisson disk and uniform surface sampling |

### Mesh Generation

| Module | Description |
|--------|-------------|
| `shapes` | Geometric primitives: sphere, cone, cylinder, capsule, quad |
| `clothing` | Clothing mesh application and layering |
| `hair_cards` | Hair card generation from guide strands |
| `cloth_sim` | Position-based cloth simulation |

## Feature Flags

None. All modules are unconditionally compiled.

## Quality Notes

- 0 `todo!()` / `unimplemented!()` macro calls in the codebase
- 0 stub implementations
- All public items carry doc comments

## Dependencies

```toml
[dependencies]
anyhow      = { workspace = true }
thiserror   = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
oxihuman-core  = { workspace = true }
oxihuman-morph = { workspace = true }
```

## License

Apache-2.0 — Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
