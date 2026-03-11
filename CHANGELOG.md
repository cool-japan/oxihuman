# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
