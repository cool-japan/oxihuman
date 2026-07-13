# oxihuman-wasm -- TODO

> Version: 0.2.1 | Updated: 2026-07-13

## Status: Stable

All core features implemented. 0 stubs (no `todo!()`/`unimplemented!()` markers). 198 passing tests. 17 modules across ~8.2k lines.

## Completed

- [x] WasmEngine core (wraps HumanEngine with JS-friendly flat API)
- [x] Engine split architecture (engine_core, engine_anim, engine_targets, engine_io, engine_fit)
- [x] Binary mesh buffer protocol (BUFFER_FORMAT_VERSION, positions/normals/uvs/indices)
- [x] Buffer transfer utilities (buffer_transfer module)
- [x] Compressed morph target support (compressed_target module)
- [x] Error handling (custom error types for WASM boundary)
- [x] Memory profiling (memory_profile module)
- [x] Pack file loading (pack module)
- [x] Service worker support (service_worker module)
- [x] wasm-bindgen JS/TS API surface (wasm_api, gated behind `bindgen` feature) — the `OxiHumanEngine` class plus `OxiHumanMorphSlider` / `OxiHumanMeasurements` / `OxiHumanAnimPlayer` helpers
- [x] TypeScript `.d.ts` type declarations (ts_types, gated behind `bindgen` feature) — `OxiHumanParams`, `MeshBytes`, `OxiHumanSwConfig`, `OxiHumanCacheEntry`, `OxiHumanAnimFrame`
- [x] Particle system (Particle, ParticleSystem exports)
- [x] Animation engine bindings (engine_anim)
- [x] Target loading/management bindings (engine_targets)
- [x] I/O bindings for OBJ/GLB/pack loading (engine_io)
- [x] Comprehensive WASM test suite (wasm_tests, 1163 lines)
- [x] OHPK core-pack loading — `new_from_core_pack_bytes` / `load_core_pack_bytes` (`WasmEngine`) and `from_core_pack_bytes` / `load_core_pack_bytes` (`OxiHumanEngine`), with an `age_floor_years` clamp (0.2.1, M2)
- [x] In-memory browser exporters — `export_glb` / `export_vrm` / `export_vrm_with_options` / `export_stl` / `export_obj` on `OxiHumanEngine`, filesystem-free (0.2.1, M2; fixes the wasm32 `std::fs` / `std::env::temp_dir` panic that used to poison the engine's wasm-bindgen borrow flag)
- [x] Measurement-driven fit solver — `fit_to_measurements` / `build_measurer_cm` / `tailoring_summary_cm` in `engine_fit.rs`: bisection on height, Nelder-Mead over weight/muscle/gender, then a lever-gated coordinate-descent refinement of the `measure/` girth weights (0.2.1, M2)
- [x] Zero-copy per-frame geometry — `wasm_memory`, `refresh_geometry`, `positions_ptr`/`positions_len`, `normals_ptr`/`normals_len`, `uvs_ptr`/`uvs_len`, `indices_ptr`/`indices_len`, `mesh_generation`, for zero-JS-copy WebGL/WebGPU vertex upload (0.2.1, M4)

## Future Work

(No TODO/FIXME markers found in source)
