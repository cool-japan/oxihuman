# OxiHuman TODO

> Last updated: 2026-03-11
> Version: 0.1.0
> Total SLoC: ~938,000 Rust (5,270 source files)

---

## Current Status Summary

| Crate | Files | Status | Completion | Stubs |
|---|---|---|---|---|
| `oxihuman-core` | ~823 | **Stable** | 100% | 0 |
| `oxihuman-morph` | ~898 | **Stable** | 100% | 0 |
| `oxihuman-mesh` | ~892 | **Stable** | 100% | 0 |
| `oxihuman-export` | ~867 | **Stable** | 100% | 0 |
| `oxihuman-physics` | ~895 | **Stable** | 100% | 0 |
| `oxihuman-viewer` | ~875 | **Stable** | 100% | 0 |
| `oxihuman-wasm` | ~18 | **Stable** | 100% | 0 |
| `oxihuman-cli` | ~10 | **Feature-complete** | 100% | 0 |

---

## Phase 1 — Core Libraries (DONE)

- [x] `.target` parser + golden tests
- [x] `.obj` loader + mesh normalization
- [x] Morph engine v0 with SoA buffers + rayon parallelism
- [x] Policy enforcement (Standard + Strict profiles)
- [x] Asset hash (SHA-256) integrity checks
- [x] Asset manifest + allowlist
- [x] GLB exporter prototype
- [x] COLLADA, STL, OBJ export
- [x] criterion benchmarks (morph, mesh, export)
- [x] FACS facial action units
- [x] Pose graph + animation curves
- [x] Body composition (BMI, age, muscle, ethnic)
- [x] 150+ facial control modules
- [x] UV mapping, UV packing, UV quality analysis
- [x] LOD generation + mesh decimation
- [x] Catmull-Clark / Loop subdivision
- [x] Normals, tangents, curvature
- [x] CLI with all subcommands

---

## Phase 2 — WASM + WebGPU Alpha (COMPLETE)

### oxihuman-wasm
- [x] `WasmEngine` core API (new, set_param, get_param, build_mesh_bytes)
- [x] Target loading from bytes and JSON
- [x] Feature-gated `wasm-bindgen` exports
- [x] Zero-copy buffer transfer (buffer_transfer.rs)
- [x] Compressed target loading (compressed_target.rs with LitePack)
- [x] "Lite pack" support (LitePack in compressed_target.rs)
- [x] WASM binary size optimization (feature gates: lite/full, wasm-opt config)
- [x] Browser integration tests (14 tests in wasm_integration.rs)
- [x] lib.rs split into modules (engine.rs, buffer.rs, error.rs, pack.rs)
- [x] Full wasm-bindgen JS/TS API surface (OxiHumanEngine, MorphSlider, Measurements, AnimPlayer)
- [x] Service Worker offline asset caching (CacheFirst/NetworkFirst/StaleWhileRevalidate)
- [x] TypeScript type definitions (.d.ts via typescript_custom_section)

### oxihuman-viewer
- [x] Camera state + orbit/zoom controls
- [x] PBR material definitions
- [x] Scene graph + transform hierarchy
- [x] Mesh upload buffer format
- [x] Viewer config + stats
- [x] Lighting presets (lighting_presets.rs - 6 presets: studio, outdoor, indoor, medical, dramatic, rim_light)
- [x] Wireframe overlay debug mode (wireframe_overlay.rs)
- [x] Screenshot / framebuffer capture (screenshot.rs - software rasterizer, PPM/TGA output)
- [x] lib.rs split into modules (camera.rs, gpu/, scene_state.rs, render_loop.rs)
- [x] Full wgpu render pipeline initialization (pipeline_cache.rs - PBR/wireframe/shadow/fullscreen)
- [x] Shader compilation — WGSL PBR Cook-Torrance + tonemap/shadow/wireframe (wgsl_shaders.rs)
- [x] WebGPU surface configuration (surface_config.rs - 4× MSAA, depth texture, sRGB)
- [x] Mesh GPU buffer upload via bind groups (bind_groups.rs - camera/material/lights/morph compute)
- [x] Real-time slider-driven morph updates (morph_updater.rs - 16ms throttle, dirty tracking)
- [x] Window event loop — winit 0.30 (event_loop.rs - orbit/pan/zoom, ApplicationHandler)
- [x] Multi-LOD rendering (lod_manager_v2.rs - QEM decimation, 5 LOD levels, hysteresis)

---

## Phase 3 — Physics Integration (COMPLETE)

### oxihuman-physics
- [x] Capsule/sphere/AABB/plane collision detection
- [x] PCA-based capsule fitting to mesh
- [x] `generate_proxies()` → `BodyProxies`
- [x] JSON serialization for proxies
- [x] Distance, bend, volume constraints
- [x] Aerodynamics (drag, wind resistance)
- [x] Cloth simulation v2 (cloth_v2/ - PBD solver, dihedral bend, symplectic Euler)
- [x] Hair strand dynamics v2 (hair_v2/ - Cosserat rods, XPBD, stretch-twist/bend-twist)
- [x] Soft-body tetrahedral simulation v2 (soft_body_v2/ - co-rotational FEM, Neo-Hookean)
- [x] Signed Distance Field (SDF) generation (sdf_gen.rs - spatial hash, pseudo-normal)
- [x] Self-collision detection (self_collision.rs - spatial hash, vertex-triangle)
- [x] Garment fitting with collision response (garment_fit_v2.rs - integrates SDF + cloth + self-collision)
- [x] Real-time constraint solving - XPBD (xpbd_unified/ - trait-based, Gauss-Seidel)
- [x] Integration with OxiRS simulation backend (oxirs_adapter.rs - semi-implicit Euler, sequential impulse solver, sleeping, ray cast, BodyRigMapper)

---

## Phase 4 — Creator Toolkit (COMPLETE)

- [x] Target authoring CLI tools (delta_painter.rs, target_tools.rs)
- [x] Pack signing + distribution pipeline (pack_distribute.rs - OXP format)
- [x] Parameter schema evolution / migration (schema_migration.rs - BFS path, 8 migration ops)
- [x] Target editor vertex delta painting (vertex_paint_state.rs - full undo/redo)
- [x] Documentation generator for custom targets (target_docs.rs - HTML/JSON/CSV/text)
- [ ] Asset pack builder GUI (optional)

---

## Phase 5 — Digital Twin Quality (COMPLETE)

- [x] Advanced constraint system (joint_limits.rs + self_intersection.rs)
- [x] Calibration workflows (calibration.rs - Nelder-Mead optimizer)
- [x] Body measurement outputs (measurements.rs - 24 measurements, cross-section slicing)
- [x] Photogrammetry fitting (body_scan_fit.rs - PLY/OBJ import, ICP, multi-stage fitting)
- [x] Statistical body model (statistical_model.rs - PCA with pure-Rust SVD)
- [x] Population-level validation (population_validate.rs - NHANES + ANSUR, KS test)

---

## Cross-Cutting Tasks

### Export — Completed
- [x] FBX (fbx_ascii.rs + fbx_binary.rs - ASCII + binary FBX 7.4)
- [x] VRM 1.0 (vrm_export.rs - GLB + VRMC extensions, 55 humanoid bones)
- [x] 3MF (three_mf_export.rs - OPC/ZIP via oxiarc-archive)
- [x] Alembic (alembic_ogawa_export.rs - Ogawa binary container)
- [x] USD/USDA (usda_export.rs - text USDA with mesh/material/skeleton)

### Code Quality — Completed
- [x] Zero .unwrap() in non-test production code (was ~2,895, now 0)
- [x] CLI split into modules (10 files, all under 2000 lines)
- [x] Viewer split into modules (gpu/, camera.rs, scene_state.rs, render_loop.rs)
- [x] WASM split into modules (engine.rs, buffer.rs, error.rs, pack.rs)

### Testing & Quality
- [x] Cross-crate integration tests (39 tests in oxihuman-tests crate)
- [x] Property tests (proptest) — 23 tests across core/mesh/morph
- [x] WASM headless browser tests (10 wasm_bindgen_test tests, node-compatible)
- [x] Physics simulation accuracy benchmarks (8 throughput + 3 accuracy benchmarks in criterion)
- [x] Viewer rendering regression tests (10 software-rasterizer tests, deterministic)
- [x] `oxihuman-cli` end-to-end tests (9 e2e tests covering generate/export/pack/pipeline)

### Performance
- [x] SIMD morph application (SSE2/AVX2 on x86_64, NEON on aarch64, feature-gated)
- [x] Incremental morph updates (DirtyTracker + IncrementalMorphCache)
- [x] GPU-accelerated morph (compute shader via wgpu - gpu_morph.rs, WGSL 64-thread workgroups)
- [x] Memory pressure profiling (WasmAllocTracker GlobalAlloc, ring-buffer profiler, budget check)
- [x] Streaming mesh decode benchmark (6 criterion benchmarks: encode/decode/chunks/LitePack)

### Documentation
- [x] API documentation (rustdoc) — comprehensive //! and /// docs across all 7 crates
- [x] User guide (asset pack creation, slider reference, export formats, WASM JS example)
- [x] Developer guide (architecture, morph internals, adding formats/constraints, policy system)
- [x] WASM integration tutorial (included in user guide + ts_types.rs TypeScript examples)

### Infrastructure
- [x] GitHub Actions CI (ci.yml - build/test/clippy/fmt/Windows/macOS/bench-dry-run/audit/deny)
- [x] WASM build pipeline in CI (wasm.yml - wasm-pack, wasm-opt, size enforcement)
- [x] Release pipeline (release.yml - validate/WASM/publish-dry-run/create-release)
- [x] Docs deployment (docs.yml - cargo doc → GitHub Pages)
- [x] cargo-deny config (deny.toml - licenses, advisories, COOLJAPAN ecosystem bans)
- [x] crates.io publish preparation (all 8 crates pass --dry-run)
- [ ] Alpha asset pack distribution strategy
- [x] Demo website deployment (demo/ — index.html + app.js WebGPU/wireframe fallback + sw.js service worker)

---

## Stub Reduction — COMPLETE

All 44 stub files replaced with real implementations. Zero `todo!()` and zero `unimplemented!()` in production code.

---

## Release Milestones

### v0.1.0 (current) — Foundation Release
- [x] All 8 workspace crates compile with `--all-features`
- [x] Core morph engine functional
- [x] 50+ export format support
- [x] CLI feature-complete
- [x] Policy enforcement (Standard + Strict)
- [x] Zero .unwrap() in production code
- [x] FBX, VRM, 3MF, Alembic, USD export implementations
- [x] Cloth, hair, soft-body physics (v2 implementations)
- [x] XPBD unified constraint solver
- [x] SDF generation + self-collision detection
- [x] Creator toolkit (target authoring, pack signing, schema migration)
- [x] Digital twin quality (calibration, measurements, body scan fitting, statistical model)
- [ ] Publish to crates.io (awaiting approval)
- [x] Browser-ready WASM build (wasm-pack CI pipeline, bindgen feature)
- [x] WebGPU viewer with real rendering (wgpu pipeline, WGSL PBR shaders, winit event loop)
- [x] Offline asset caching (Service Worker CacheFirst/NetworkFirst/StaleWhileRevalidate)
- [x] Demo website (demo/ — index.html + app.js WebGPU/wireframe fallback + sw.js service worker)
- [x] OxiRS adapter layer (oxirs_adapter.rs - rigid bodies, ray cast, contacts, BodyRigMapper)
- [x] All stubs replaced with real implementations (0 todo!(), 0 unimplemented!())
- [x] Comprehensive test coverage (32,564 tests across all crates — 0 failures)
- [x] Performance benchmarks (SIMD morph, incremental dirty, GPU compute, 11 physics benchmarks)
- [x] Full documentation (rustdoc, user guide, developer guide, TypeScript examples)
- [x] Security audit complete (security.rs: path sanitization, checked arithmetic, magic bytes; 1 low advisory)
