# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2026-05-05

### Changed
- Test fixtures that depend on the MakeHuman dataset now resolve the data root via the
  `MAKEHUMAN_DATA_DIR` environment variable instead of hard-coded absolute paths.
  Set it to the MakeHuman `data/` directory (the one containing `3dobjs/base.obj` and `targets/`).
  Tests that require this data skip gracefully when the variable is unset.
- Asset pack tests resolve the asset root via `OXIHUMAN_ASSETS_DIR` (the directory containing
  `alpha_pack/oxihuman_assets.toml`). Tests skip gracefully when the variable is unset.

## [0.1.1] - 2026-03-13

### Added
- FABRIK IK solver (`fabrik_ik.rs`) with constrained variant (pole vector, cone angle limit)
- XPBD secondary motion system with Pin/Length/Volume constraints and self-collision detection
- BVH animation retargeting bridge: `parse_bvh_text`, `BvhData`, `BvhJointFrame`, `SkeletonMapping`
- USD time-sampled blend shape animation: `BlendShapeTimeSamples` and `UsdaWriter::write_blend_shape_animation`
- Interactive asset pack builder CLI wizard (`pack-wizard` command)
- Cargo fuzz targets for parser, OXP reader, and USD writer

### Fixed
- Clippy `div_ceil` warning in `fbx_binary.rs:476`

### Refactored
- `oxihuman-wasm` engine split into focused modules: `engine_core`, `engine_anim`, `engine_targets`, `engine_io`
- WASM tests extracted to `wasm_tests.rs` (lib.rs reduced from 1208L to <80L)

### Refactored (Policy compliance — 2 000-line limit)
- **oxihuman-physics** `lib.rs` 3 858 → 117 lines; extracted `proxy_types.rs`, `proxy_gen.rs`, `proxy_tests.rs`, and `modules_a.rs`–`modules_g.rs`
- **oxihuman-morph** `lib.rs` 3 438 → 37 lines; split into `_morph_part1.rs`–`_morph_part6.rs`
- **oxihuman-mesh** `lib.rs` 3 323 → 72 lines; extracted `color_utils.rs` and `mods_a.rs`–`mods_e.rs`
- **oxihuman-export** `lib.rs` 3 137 → 60 lines; split into `_export_part1.rs`–`_export_part6.rs`
- **oxihuman-core** `lib.rs` 2 609 → 43 lines; split into `_core_part1.rs`–`_core_part3.rs`

### Added
- **FBX binary**: zlib-compressed arrays for `I32Array`/`F32Array`/`F64Array` variants (>512 elements, encoding=1) via `miniz_oxide`; new `export_mesh_fbx_binary()` convenience function; ASCII FBX path deprecated since 0.1.1
- **Alembic Ogawa**: `AlembicWriter::from_mesh_buffers()`, `from_mesh_sequence()`, `to_bytes()`, `write_to_file()`, `frame_count()` convenience API; "stub" label removed from docs
- **oxihuman-viewer**: 8 criterion benchmarks (`lod_select`, `lod_chain_build`, `morph_updater_dirty/clean`, `camera_orbit`, `render_stats_snapshot`, `lod_transition_hysteresis`)
- **oxihuman-core**: 17 criterion benchmarks (bloom filter, kd-tree, octree, radix sort, skip list, string pool, asset cache, dependency resolver, spatial hash 2D)
- **Asset pack distribution**: `generate_distribution_manifest()` and `verify_distribution_manifest()` in `asset_pack_builder`; CLI subcommands `pack-dist-manifest` and `pack-verify-dist`; `docs/asset-pack-distribution.md`

## [0.1.0] - 2026-03-11

### Added

#### oxihuman-core
- Core data structures: arena allocator, splay tree, AVL tree, B-tree, bloom filters
- Graph algorithms: BFS shortest path, Bellman-Ford, Floyd-Warshall, Edmonds-Karp max flow, bipartite matching
- Asset management: registry, cache, hash (SHA-256), manifest, versioning
- Codec utilities: base64, base58, arithmetic coding, Huffman coding, CBOR, Avro stubs
- Spatial structures: AABB tree, BVH, spatial hash 2D, HyperLogLog, Count-Min sketch
- String/text: Aho-Corasick, Z-algorithm, indent detector, sentence splitter
- Math primitives: matrix3, color utilities, angle utils, bezier curves

#### oxihuman-morph
- MorphEngine: target-based parametric morphing over thousands of vertices
- Age model, body composition, anthropometry, muscle simulation
- FACS facial action units, pose graph, skin deformation
- Blendshape interpolation, diversity parameters, body symmetry
- Animation curves, param animation, anim retarget
- Constraint system, regions, nasal/hip controls

#### oxihuman-mesh
- Mesh data model: vertex groups, halfedge topology
- Dual contouring (DC) surface extraction
- Sharp feature detection and preservation
- UV mapping, UV packing, UV stitching, UV quality analysis
- LOD generation, mesh decimation
- Parametric surfaces, Gordon surfaces, Coons patches
- Normal map baking, bent normals, cage lattice deform
- Cloth pins, shape key mixing, paint mask

#### oxihuman-export
- GLB/glTF binary export with bytemuck zero-copy packing
- COLLADA export, STL export, SVG projection export
- Vertex animation export, animation layer export
- Texture packing, diffuse color export
- Asset signing and pack verification
- Streaming export, LOD export, morph quantization export

#### oxihuman-physics
- Capsule pair collision detection
- Porous flow (Darcy pressure solver)
- Lattice Boltzmann fluid simulation stub
- Runge-Kutta 4th-order integrator
- Hyperelastic, anisotropic, foam, auxetic material models
- Phase-field fracture, plate bending, creep/fatigue models
- Particle filter, tissue deformation

#### oxihuman-viewer
- wgpu/WebGPU rendering adapter (optional `webgpu` feature)
- LOD manager, depth linearization, instance batching
- Debug views: bent normals, thermal, false-color, histology

#### oxihuman-wasm
- `WasmEngine` — full browser-ready API via `wasm-bindgen`
- Target loading from bytes, ZIP pack loading
- Parameter get/set, mesh export to bytes
- Physics step, animation seek, preset application

#### oxihuman-cli
- `oxihuman generate` — generate mesh from parameter JSON
- `oxihuman export-gltf` / `export-collada` / `export-stl`
- `oxihuman morph-export`, `lod-export`, `proxies`
- `oxihuman asset-bundle`, `zip-pack`, `sign-pack`, `verify-sign`
- `oxihuman stats`, `validate`, `report`
