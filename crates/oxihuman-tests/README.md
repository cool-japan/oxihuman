# oxihuman-tests

Cross-crate integration test suite for the OxiHuman workspace. This crate is not published (`publish = false`); it exists solely to exercise multi-crate workflows that cannot be tested within individual crates.

**Version:** 0.1.2 | **Tests:** 39 passing | **Updated:** 2026-05-05

## What it tests

The single test binary (`tests/integration_tests.rs`) covers five integration scenarios:

| Module | Coverage |
|---|---|
| `core_morph_mesh_export` | Core -> Morph -> Mesh -> Export pipeline: GLB, OBJ, STL (ASCII + binary), GLB with skeleton, parallel vs. sequential mesh build |
| `core_physics` | Collision proxy generation, proxy JSON serialization, cloth simulation stepping, sphere-sphere contact, physics rig construction |
| `morph_measurements` | Body measurements from morph output (height, shoulder/waist/hip widths), AABB computation, normal recomputation, mesh stats |
| `morph_export_formats` | USDA, 3MF, FBX binary, VRM metadata, COLLADA, X3D, PLY, JSON mesh summary, CSV vertex export |
| `core_morph_viewer` | MeshUploadBuffer construction and binary round-trip, viewer scene/camera defaults, PBR material and pipeline descriptors |
| `cross_cutting` | Full morph -> physics -> export chain, subdivision + Laplacian smoothing, mesh integrity checks, convex hull, event bus, undo/redo stack, scene export, preset -> measurement -> export pipeline |

All file-based tests write to `std::env::temp_dir()` and clean up after themselves.

## Running the tests

```
cargo nextest run -p oxihuman-tests --all-features
```

Standard `cargo test` also works:

```
cargo test -p oxihuman-tests --all-features
```

## Notes

- `publish = false` — this crate is internal to the workspace and will never be published to crates.io.
- Dependencies cover the full OxiHuman crate surface: `oxihuman-core`, `oxihuman-morph`, `oxihuman-mesh`, `oxihuman-export`, `oxihuman-physics`, `oxihuman-viewer`.
