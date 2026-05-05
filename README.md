# OxiHuman

**Privacy-first, client-side human body generator — pure Rust, WASM/WebGPU ready.**

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.2-green.svg)](CHANGELOG.md)
[![Rust Edition](https://img.shields.io/badge/rust-edition%202021-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/)

> **Version 0.1.2** — Released 2026-05-05
> **Author**: COOLJAPAN OU (Team Kitasan)
> **Repository**: https://github.com/cool-japan/oxihuman
> **License**: Apache-2.0

---

## Overview

OxiHuman is a pure-Rust parametric human body generator that runs entirely client-side — in the browser via WebAssembly/WebGPU or natively on any platform. It synthesises detailed 3D human meshes from high-level slider parameters without ever transmitting body data to a server. The project spans ~943,000 lines of Rust across 8 workspace crates, with 32,791 passing tests.

### Core Principles

| Principle | Description |
|-----------|-------------|
| **No network by default** | Core crates compile without any HTTP stack; no outbound connections at runtime |
| **Client-side compute** | All morphing and mesh synthesis runs locally — in the browser or on native |
| **Safety by construction** | The base mesh is always a bodysuit; no naked mesh stage exists in memory or exports |
| **Deterministic builds** | Fully reproducible pipelines for both WASM and native targets |

---

## Workspace Crates

All crates are at version **0.1.2**.

| Crate | Status | Tests | Purpose |
|-------|--------|------:|---------|
| `oxihuman-core` | Stable | 5,243 | Arena allocator, graphs, asset cache, spatial index, codec, event bus |
| `oxihuman-morph` | Stable | 5,865 | Parametric morphing engine, FACS, pose graph, age/body model |
| `oxihuman-mesh` | Stable | 5,715 | Mesh processing, topology, UV mapping, LOD, skinning |
| `oxihuman-export` | Stable | 5,289 | glTF/GLB, COLLADA, OBJ, STL, USD, VRM, streaming export |
| `oxihuman-physics` | Stable | 5,217 | Soft-body, cloth, rigid body, FEM, SPH, biomechanics |
| `oxihuman-viewer` | Stable | 4,974 | wgpu/WebGPU rendering, camera systems, 100+ debug views |
| `oxihuman-wasm` | Stable | 168 | WebAssembly bindings (wasm-bindgen), 68-method browser API |
| `oxihuman-cli` | Stable | 134 | 35 subcommands: generate, export, batch, validate, sign |
| `oxihuman-tests` | Stable | 39 | Integration and cross-crate tests |
| **Total** | | **32,791** | |

---

## Feature Highlights

### Morphology (`oxihuman-morph`)

- Parametric MorphEngine with target-based blending over thousands of vertices
- Age progression model with anthropometric scaling
- Body composition: muscle simulation, fat distribution, skeletal proportions
- FACS facial action units (Action Units) for expressive face morphing
- Pose graph with constraint system and skin deformation
- Diversity parameters, body symmetry controls
- Animation curves, parameter animation, animation retargeting
- 30+ morph modules covering nasal, hip, limb, and torso regions

### Mesh Processing (`oxihuman-mesh`)

- Halfedge topology data model with vertex groups
- Dual contouring (DC) surface extraction with sharp feature preservation
- UV mapping, UV packing, UV stitching, UV quality analysis
- LOD generation and mesh decimation
- Parametric surfaces: Gordon surfaces, Coons patches
- Linear Blend Skinning (LBS) and Dual Quaternion Skinning (DQS)
- Normal map baking, bent normals, cage lattice deformation
- Cloth pins, shape key mixing, paint masking
- 60+ mesh algorithms

### Export Pipeline (`oxihuman-export`)

- **GLB/glTF**: binary export with bytemuck zero-copy packing, animated glTF
- **COLLADA**: full scene graph export
- **STL**: solid and ASCII modes
- **USD**: Universal Scene Description export
- **VRM**: avatar format export
- **3MF**: additive manufacturing format
- Vertex animation export, animation layer export
- Texture packing, diffuse color export
- Asset signing and pack verification
- Streaming export, LOD export, morph quantization export
- Batch pipeline for multi-asset processing

### Physics (`oxihuman-physics`)

- Capsule pair collision detection
- Cloth simulation with pins and constraints
- Rigid body dynamics
- Finite Element Method (FEM): hyperelastic, anisotropic, foam, auxetic material models
- Smoothed Particle Hydrodynamics (SPH) fluid simulation
- Porous flow via Darcy pressure solver
- Lattice Boltzmann fluid simulation
- Runge-Kutta 4th-order integrator
- Phase-field fracture, plate bending, creep/fatigue models
- Biomechanics: tissue deformation, particle filter
- 350+ physics modules

### WASM / Browser (`oxihuman-wasm`)

- `WasmEngine` — full browser-ready API via `wasm-bindgen`
- 68-method JavaScript/TypeScript API surface
- Target loading from bytes, ZIP pack loading
- Parameter get/set, mesh export to bytes
- Physics step, animation seek, preset application
- Enable `webgpu` feature for wgpu-based in-browser rendering

### CLI (`oxihuman-cli`)

35 subcommands covering the full generation and export workflow:

```
oxihuman generate          # Generate mesh from parameter JSON
oxihuman export-gltf       # Export to glTF/GLB
oxihuman export-collada    # Export to COLLADA
oxihuman export-stl        # Export to STL
oxihuman morph-export      # Export morph targets
oxihuman lod-export        # Export LOD levels
oxihuman proxies           # Proxy mesh generation
oxihuman asset-bundle      # Bundle assets
oxihuman zip-pack          # Create ZIP pack
oxihuman sign-pack         # Sign asset pack
oxihuman verify-sign       # Verify pack signature
oxihuman batch             # Batch processing pipeline
oxihuman validate          # Validate mesh/asset integrity
oxihuman stats             # Print project/mesh statistics
oxihuman report            # Generate human-readable report
# ... and 17 more subcommands
```

### Viewer (`oxihuman-viewer`)

- wgpu/WebGPU rendering adapter (optional `webgpu` feature)
- Multiple camera systems: orbit, fly, cinematic
- LOD manager, depth linearization, instance batching
- 100+ debug views including: bent normals, thermal overlay, false-color shading, histology visualization

---

## Installation

Add individual crates to your `Cargo.toml` as needed:

```toml
[dependencies]
oxihuman-core    = "0.1.2"
oxihuman-morph   = "0.1.2"
oxihuman-mesh    = "0.1.2"
oxihuman-export  = "0.1.2"
oxihuman-physics = "0.1.2"
oxihuman-viewer  = "0.1.2"
oxihuman-wasm    = "0.1.2"
oxihuman-cli     = "0.1.2"
```

---

## Quick Start

### Native (Rust)

```toml
[dependencies]
oxihuman-morph = "0.1.2"
oxihuman-mesh  = "0.1.2"
```

```rust
use oxihuman_morph::engine::MorphEngine;
use oxihuman_export::gltf::GltfExporter;

let mut engine = MorphEngine::default();
engine.set_param("height", 0.6);
engine.set_param("weight", 0.4);
engine.set_param("age", 0.35);
let mesh = engine.build_mesh();

let exporter = GltfExporter::new();
let glb_bytes = exporter.export_glb(&mesh)?;
```

### Browser (WebAssembly)

```javascript
import init, { WasmEngine } from "./oxihuman_wasm.js";

await init();
const engine = new WasmEngine();
engine.set_param("height", 0.6);
engine.set_param("weight", 0.4);
const mesh_bytes = engine.export_mesh_bytes();
```

### CLI

```bash
# Generate a mesh from parameters and export to GLB
oxihuman generate --params params.json --output human.glb

# Batch-generate with different presets
oxihuman batch --input presets/ --output out/ --format gltf

# Validate an asset pack
oxihuman validate --pack assets.zip
```

---

## Building

```bash
# Native (all features)
cargo build --all-features

# WASM (browser target)
cargo build -p oxihuman-wasm --target wasm32-unknown-unknown --features wasm

# WASM with WebGPU rendering
cargo build -p oxihuman-wasm --target wasm32-unknown-unknown --features wasm,webgpu
```

## Testing

```bash
# Run all 32,791 tests
cargo nextest run --all-features

# Run tests for a specific crate
cargo nextest run -p oxihuman-morph --all-features
```

### Test fixtures

Some tests depend on the [MakeHuman](http://www.makehumancommunity.org/) dataset and oxihuman asset
packs. To run them locally:

| Variable | Purpose | Example |
|---|---|---|
| `MAKEHUMAN_DATA_DIR` | Path to the MakeHuman `data/` directory (containing `3dobjs/base.obj` and `targets/`) | `/path/to/makehuman/data` |
| `OXIHUMAN_ASSETS_DIR` | Path to the oxihuman asset pack root (containing `alpha_pack/oxihuman_assets.toml`) | `/path/to/oxihuman/assets` |

Tests that require these fixtures skip gracefully when the variable is unset, so the standard
`cargo nextest run --all-features` still passes on machines without the dataset.

---

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

Copyright (C) 2026 COOLJAPAN OU (Team Kitasan)
