# oxihuman-wasm

Part of the [OxiHuman](../../README.md) workspace — privacy-first, client-side human body generator in pure Rust.

**Status:** Stable | **Tests:** 198 passing | **API:** 78 WasmEngine methods | **Version:** 0.2.2 | **Updated:** 2026-07-13

WebAssembly bindings for OxiHuman — full browser-ready API via wasm-bindgen. `WasmEngine` is the native Rust engine (a thin wrapper around `oxihuman_morph::engine::HumanEngine`) exposing a flat, JS-friendly method surface; built with `--features bindgen`, it is surfaced to JavaScript/TypeScript through the `OxiHumanEngine` wasm-bindgen class (`wasm_api.rs`), enabling privacy-preserving, client-side human body generation with no server round-trips.

---

## Feature Flags

```toml
[features]
default = []
bindgen = ["dep:wasm-bindgen", "dep:console_error_panic_hook"]
wasm    = ["bindgen"]   # deprecated alias of `bindgen`
```

**`bindgen` is the canonical browser feature** — it enables the full
wasm-bindgen JS/TS surface (`OxiHumanEngine`, sliders, anim player,
TypeScript custom sections — `OxiHumanParams`, `MeshBytes`, `OxiHumanSwConfig`,
`OxiHumanCacheEntry`, `OxiHumanAnimFrame`, hand-injected into the generated
`.d.ts` by `ts_types.rs`) plus `console_error_panic_hook` so Rust panics
appear in the browser console. Build with:

```bash
wasm-pack build --release --target web    crates/oxihuman-wasm --no-default-features --features bindgen
wasm-pack build --release --target nodejs crates/oxihuman-wasm --no-default-features --features bindgen
```

The legacy `wasm` feature is now a pure alias of `bindgen` (kept for
compatibility; prefer `bindgen`). Without either feature the crate still
compiles for native testing and CLI pipelines.

---

## Installation

```toml
[dependencies]
oxihuman-wasm = { version = "0.2.2", features = ["wasm"] }
```

---

## Structs

### `WasmEngine`

Main body generator (native Rust; not itself `#[wasm_bindgen]`). Exposes the full OxiHuman pipeline as 78 public methods — see [WasmEngine API](#wasmengine-api--78-methods) below. Holds base mesh data, loaded morph targets, parameter state, physics proxies, persistent zero-copy geometry buffers, and animation frame buffer.

### `OxiHumanEngine` (feature `bindgen`)

The actual `#[wasm_bindgen]` JS class, defined in `wasm_api.rs`. Wraps a `WasmEngine` behind `Rc<RefCell<..>>` and re-exposes its capabilities as 71 JS methods — some 1:1 (`build_mesh_bytes`, `refresh_geometry`, ...), some collapsed for ergonomics (`set_height` / `set_weight` / `set_muscle` / `set_age` → a single `set_param(name, value)`), plus the in-memory exporters (`export_glb`, `export_vrm`, `export_obj`, `export_stl`) and `fit_to_measurements`, which have no directly-equivalent `WasmEngine` method of the same name. A borrow failure on the inner engine surfaces as a catchable `"OxiHumanEngine is busy"` JS error instead of a panic.

### `OxiHumanMorphSlider` / `OxiHumanMeasurements` / `OxiHumanAnimPlayer` (feature `bindgen`)

Small `#[wasm_bindgen]` helper classes returned by `OxiHumanEngine`:
- `OxiHumanMorphSlider` — a named-parameter slider handle (`OxiHumanMorphSlider.for_param(engine, name)`) with `.name()` / `.value()` / `.set_value(v)` / `.min()` / `.max()`.
- `OxiHumanMeasurements` — the snapshot returned by `engine.get_measurements()`, with `.height_cm()` / `.chest_cm()` / `.waist_cm()` / `.hip_cm()` / `.weight_kg()` getters.
- `OxiHumanAnimPlayer` — a standalone player (`engine.make_anim_player()`) sharing the same underlying engine state, with `.record_frame()` / `.frame_count()` / `.seek(frame)` / `.step(dt)` / `.set_fps()` / `.get_fps()` / `.export_anim_json()` / `.clear()`.

### `ParticleSystem`

Point particle system with configurable `emit_rate` and `lifetime` control. Created via `WasmEngine::create_particle_system()` and stepped via `step_particles()`.

### `Particle`

Represents a single active particle. Carries `position` (Float32Array), `velocity` (Float32Array), and `age` (f32).

---

## WasmEngine API — 78 Methods

### Initialization (8)

| Method | Description |
|--------|-------------|
| `new_from_obj_bytes(bytes)` | Create engine from raw OBJ mesh bytes |
| `new_strict(bytes)` | Create engine from raw OBJ bytes under the strict validation policy profile |
| `new_stub()` | Create engine with a minimal 3-vertex stub mesh (internal default; replace via a real constructor before use) |
| `new_from_core_pack_bytes(bytes)` | Create engine directly from an OHPK core-pack byte buffer |
| `load_core_pack_bytes(bytes)` | Load (or replace) targets from an OHPK core-pack buffer into an existing engine; returns the number of targets loaded |
| `age_floor_years()` | Return the pack's declared minimum modelled age in years, if any |
| `load_target_bytes(name, bytes)` | Load a morph target from raw `.target` file bytes |
| `load_zip_pack_bytes(bytes)` | Load a ZIP asset pack; returns the number of targets loaded |

### Target Management (8)

| Method | Description |
|--------|-------------|
| `list_loaded_targets()` | Return JSON array of all loaded target names |
| `loaded_target_count()` | Return count of currently loaded targets |
| `load_target_from_json(json)` | Load a morph target from JSON descriptor |
| `unload_target(name)` | Remove a named target from the engine |
| `set_target_weight_by_name(name, weight)` | Set blend weight [0.0–1.0] for a named target |
| `get_target_weight_by_name(name)` | Get current blend weight for a named target |
| `target_count()` | Total number of registered targets (loaded + unloaded) |
| `get_loaded_target_names()` | Return JS array of loaded target name strings |

### Parameter Control (7)

| Method | Description |
|--------|-------------|
| `set_height(value)` | Set body height parameter (normalized `[0.0, 1.0]`) |
| `set_weight(value)` | Set body weight parameter (normalized `[0.0, 1.0]`) |
| `set_muscle(value)` | Set musculature parameter (normalized `[0.0, 1.0]`) |
| `set_age(value)` | Set age parameter (normalized `[0.0, 1.0]`) |
| `set_param(key, value)` | Set any named parameter by string key |
| `reset_params()` | Reset all body parameters to defaults |
| `reset_all_weights()` | Reset all morph target blend weights to zero |

### JSON Import/Export (4)

| Method | Description |
|--------|-------------|
| `export_params_json()` | Serialize current parameter state to JSON |
| `import_params_json(json)` | Restore parameter state from JSON |
| `get_measurements_json()` | Return computed body measurements (height, circumferences, etc.) as JSON |
| `export_anim_json()` | Serialize recorded animation frames to JSON |

### Mesh Building (4)

| Method | Description |
|--------|-------------|
| `build_mesh_bytes()` | Build binary mesh buffer containing positions, normals, UVs, and indices |
| `build_mesh_prepared()` | Build and return the prepared `MeshBuffers` (positions/normals/uvs/indices) without serializing to bytes |
| `export_quantized_bytes()` | Export mesh in QMSH quantized binary format (compact, lossy) |
| `get_scene_json()` | Return scene description JSON (nodes, cameras, lights) |

### Physics (6)

| Method | Description |
|--------|-------------|
| `get_physics_proxies_json()` | Return collision proxy shapes as JSON |
| `get_physics_rig_json()` | Return full physics rig descriptor as JSON |
| `get_capsule_chains_json()` | Return capsule chain descriptors for limbs as JSON |
| `step_physics(dt)` | Step physics simulation by `dt` seconds (stub) |
| `set_wind(x, y, z)` | Set wind force vector for cloth/particle systems (stub) |
| `init_cloth(stiffness)` | Initialize a cloth simulation from the last built mesh with the given spring stiffness (no-op until a mesh has been built) |

### Animation (7)

| Method | Description |
|--------|-------------|
| `record_anim_frame()` | Record current parameter state as an animation frame |
| `clear_anim_frames()` | Clear all recorded animation frames |
| `anim_frame_count()` | Return number of recorded frames |
| `seek_anim_frame(index)` | Restore parameter state to a recorded frame by index |
| `play_anim_step()` | Advance playback by one frame, wrapping at end |
| `get_anim_fps()` | Get current animation playback rate (frames per second) |
| `set_anim_fps(fps)` | Set animation playback rate |

### Query & Analysis (8)

| Method | Description |
|--------|-------------|
| `vertex_count()` | Return number of vertices in the base mesh (native `usize`; see also `get_vertex_count()`) |
| `get_vertex_count()` | Return number of vertices in the built mesh (`u32`, JS-friendly) |
| `get_index_count()` | Return number of triangle indices in the built mesh |
| `get_curvature_map()` | Return per-vertex curvature values as Float32Array |
| `get_geodesic_distances(origin_index)` | Return geodesic distances from origin vertex as Float32Array |
| `query_sphere_near_point(x, y, z, radius)` | Return JSON array of vertex indices within sphere |
| `get_mesh_segments()` | Return JSON segment map (body part regions by vertex range) |
| `get_physics_proxy_json()` | Return collision proxy shapes as a single `{"proxies":[...]}` JSON document (see also `get_physics_proxies_json()`) |

### Presets & Proportions (5)

| Method | Description |
|--------|-------------|
| `set_params_from_preset(preset_name)` | Apply a named parameter preset (e.g., `"athletic"`, `"average"`) |
| `apply_preset_by_name(name)` | Apply preset including morph target weights |
| `get_body_proportions_json()` | Return computed proportional ratios as JSON |
| `get_param_summary_json()` | Return a summary of all parameters and their current values as JSON |
| `set_allowlist(names)` | Restrict which targets are eligible for blending |

### LOD & Shaders (2)

| Method | Description |
|--------|-------------|
| `get_lod_scene_json()` | Return multi-LOD scene JSON (LOD0–LOD3 meshes) |
| `list_builtin_shaders()` | Return JSON array of built-in shader descriptor names |

### Cache (2)

| Method | Description |
|--------|-------------|
| `has_cached_mesh()` | Return `true` if a built mesh is cached and parameters are unchanged |
| `reset_incremental_cache()` | Invalidate the incremental mesh build cache |

### Particles (2)

| Method | Description |
|--------|-------------|
| `create_particle_system(emit_rate, lifetime)` | Create a new `ParticleSystem` with given parameters |
| `step_particles(system, dt)` | Advance a particle system by `dt` seconds |

### Expressions (2)

| Method | Description |
|--------|-------------|
| `apply_expression_blend(expression_json)` | Apply a facial expression blend from JSON descriptor |
| `get_cloth_state()` | Return current cloth simulation state as JSON (stub) |

### Zero-Copy Geometry (10)

Persistent geometry buffers for zero-JS-copy WebGL/WebGPU upload (added in
0.2.1, M4); see [Zero-copy per-frame geometry](#zero-copy-per-frame-geometry)
below.

| Method | Description |
|--------|-------------|
| `refresh_geometry()` | Recompute geometry into the persistent buffers if parameters changed; returns the current `mesh_generation` |
| `positions_ptr()` / `positions_len()` | Byte offset and element count of the flat XYZ position buffer |
| `normals_ptr()` / `normals_len()` | Byte offset and element count of the flat XYZ normal buffer |
| `uvs_ptr()` / `uvs_len()` | Byte offset and element count of the flat UV buffer |
| `indices_ptr()` / `indices_len()` | Byte offset and element count of the `u32` triangle index buffer |
| `mesh_generation()` | Current geometry-buffer generation counter; bumps only when the buffers are reallocated (topology or vertex-count change) |

### Measurement Fit (3)

Added in 0.2.1 (M2), implemented in `engine_fit.rs`.

| Method | Description |
|--------|-------------|
| `build_measurer_cm()` | Build (and cache) the centimetre-unit cross-section measurer topology for the current mesh |
| `tailoring_summary_cm()` | Return a summary of tailoring/girth measurements in centimetres |
| `fit_to_measurements(options_json)` | Solve `height` directly (bisection on the monotone stature response), then Nelder–Mead over `weight` / `muscle` / gender, then a lever-gated coordinate-descent refinement of the `measure/` girth weights, so the re-measured mesh matches requested `height_cm` / `chest_cm` / `waist_cm` / `hip_cm` targets; returns a JSON fit report (`params`, `results[]`, `measure_weights`, `iterations`, `converged`) |

---

## Free Functions

| Function | Description |
|----------|-------------|
| `parse_mesh_bytes_header(buffer)` | Parse the format header from a binary mesh buffer; returns JSON with format version and field offsets |
| `wasm_memory()` (feature `bindgen`) | Return the module's `WebAssembly.Memory` object, for building zero-copy typed-array views over `positions_ptr()` / `normals_ptr()` / `uvs_ptr()` / `indices_ptr()` |
| `set_panic_hook()` (feature `bindgen`) | Install `console.error` as the Rust panic hook; call once at startup |
| `get_version()` (feature `bindgen`) | Return the crate version string (e.g. `"0.2.2"`) |

---

## JavaScript/TypeScript Usage

### Recommended: OHPK core pack (3 lines)

```js
import init, { OxiHumanEngine } from "oxihuman-wasm";
await init();
const bytes  = new Uint8Array(await (await fetch(packUrl)).arrayBuffer());
const engine = OxiHumanEngine.from_core_pack_bytes(bytes);
```

The pack's targets are driven directly by
`engine.set_param("height"|"weight"|"muscle"|"age", v)`; a declared
`age_floor_years` clamps the `age` parameter (`engine.age_floor_years()`).

### In-memory exports

```js
const glb = engine.export_glb();          // Uint8Array (GLB 2.0)
const vrm = engine.export_vrm();          // Uint8Array (VRM 1.0: GLB + VRMC_vrm)
const stl = engine.export_stl(true);      // Uint8Array (binary STL; false = ASCII)
const obj = engine.export_obj();          // string (Wavefront OBJ)
```

All exports run entirely in memory (no filesystem — required on wasm32) and
pass the bodysuit export gate. Measurements (`get_measurements()`,
`get_measurements_json()`) are uniformly in **centimetres**.

### Measurement-driven fit

```js
const report = JSON.parse(engine.fit_to_measurements(JSON.stringify({
  height_cm: 172.0,
  chest_cm: 96.0,
  waist_cm: 82.0,
  hip_cm: 98.0,
  max_iterations: 60,          // optional, default 45, clamped to [10, 400]
})));
// report: { params, results: [{ name, target_cm, measured_cm, delta_cm }, ...],
//           measure_weights, iterations, converged }
```

Any subset of `height_cm` / `chest_cm` / `waist_cm` / `hip_cm` may be supplied
(at least one is required). `height` is solved directly by bisection on the
monotone stature response, then Nelder–Mead searches `weight` / `muscle` /
gender, then a lever-gated coordinate-descent pass refines the `measure/`
bust / underbust / waist / hips weights. The optimiser is deterministic (same
input always yields the same fit). The `brief-172` probe fits to `|Δ| ≤ 0.66
cm` in ~0.9 s under Node — see
[`docs/bench/measurement-error.md`](docs/bench/measurement-error.md).

### Zero-copy per-frame geometry

```js
import { wasm_memory } from "oxihuman-wasm";
const memory = wasm_memory();
let gen = engine.refresh_geometry();
let positions = new Float32Array(memory.buffer, engine.positions_ptr(), engine.positions_len());
let normals   = new Float32Array(memory.buffer, engine.normals_ptr(),   engine.normals_len());
let indices   = new Uint32Array (memory.buffer, engine.indices_ptr(),   engine.indices_len());

function frame(t) {
  engine.set_param("weight", 0.5 + 0.5 * Math.sin(t));
  const g = engine.refresh_geometry();  // incremental, writes in place
  if (g !== gen || positions.buffer !== memory.buffer) {
    // topology changed OR wasm memory grew: re-create views
    gen = g;
    positions = new Float32Array(memory.buffer, engine.positions_ptr(), engine.positions_len());
    normals   = new Float32Array(memory.buffer, engine.normals_ptr(),   engine.normals_len());
    indices   = new Uint32Array (memory.buffer, engine.indices_ptr(),   engine.indices_len());
  }
  // upload positions/normals to WebGL/WebGPU with zero JS-side copies
}
```

`positions_ptr()` stays stable across `set_param` calls (no reallocation
while the vertex count is unchanged); `mesh_generation()` bumps whenever the
buffers moved.

### Verification harness

`node scripts/wasm_node_check.mjs [--pkg <pkg-dir>] [--pack <file.ohpk>]`
runs the full end-to-end Node.js check (pack loading, zero-copy views, age
floor, GLB/VRM/STL/OBJ validation, and the poisoning-regression suite).

### Classic OBJ/ZIP flow

```js
import init, { OxiHumanEngine } from "oxihuman-wasm";

await init();

// Load base mesh from an OBJ file
const objBytes = new Uint8Array(await fetch("human_base.obj").then(r => r.arrayBuffer()));
const engine = OxiHumanEngine.from_obj_bytes(objBytes);

// Load a ZIP asset pack with morph targets
const packBytes = new Uint8Array(await fetch("targets.zip").then(r => r.arrayBuffer()));
const count = engine.load_zip_pack_bytes(packBytes);
console.log(`Loaded ${count} targets`);

// Set body parameters (all normalized [0.0, 1.0])
engine.set_param("height", 0.7);
engine.set_param("weight", 0.4);
engine.set_param("muscle", 0.6);
engine.set_param("age", 0.5);

// Build mesh and use binary data
const meshBytes = engine.build_mesh_bytes();

// Export parameters
const paramsJson = engine.export_params_json();
console.log(JSON.parse(paramsJson));

// Query measurements
const measurements = JSON.parse(engine.get_measurements_json());
console.log(measurements);
```

---

## Architecture Notes

- All heavy computation runs in the browser's WebAssembly sandbox — no server communication, no data leakage.
- `WasmEngine` is `!Send` and single-threaded by design. For parallel workloads, use multiple instances in separate Web Workers.
- Physics methods (`step_physics`, `set_wind`) and `get_cloth_state` are currently stubs returning placeholder data; full simulation is implemented in `oxihuman-physics`.
- The `bindgen` feature gate keeps native builds free of wasm-bindgen overhead, allowing the crate to be used in test harnesses and CLI pipelines without a browser target.
- **No method on the JS surface may touch `std::fs` / `std::env::temp_dir`.** On wasm32 the filesystem does not exist and `std::env::temp_dir()` *panics*; the resulting trap used to poison the engine object's wasm-bindgen `WasmRefCell` borrow flag, making every later call fail with "recursive use of an object detected". All exporters therefore use the in-memory byte builders from `oxihuman-export`.
- `OxiHumanMorphSlider` / `OxiHumanAnimPlayer` hold an `Rc` clone of the engine state (no raw pointers): calling `engine.free()` from JS cannot create a use-after-free.

### v0.1.2 Internal Refactor

`engine.rs` is now a thin re-export module (7 lines). The implementation has been split into five focused source files with no public API changes:

| File | Contents |
|------|----------|
| `engine_core.rs` | `WasmEngine`, `ParticleSystem`, `Particle` struct definitions and initialization |
| `engine_anim.rs` | Animation recording, playback, and frame management methods |
| `engine_targets.rs` | Morph target loading, unloading, and weight management methods |
| `engine_io.rs` | JSON import/export, mesh building, physics, query, and preset methods |
| `engine_fit.rs` | Measurement-driven parameter fitting (`fit_to_measurements`, `build_measurer_cm`, `tailoring_summary_cm`) — added in 0.2.1 (M2) |

Tests were moved to `wasm_tests.rs`. All public method signatures are unchanged; the same methods are surfaced to JS through the `OxiHumanEngine` wasm-bindgen wrapper (`wasm_api.rs`) when the `bindgen` feature is enabled.

---

## License

Apache-2.0 — Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
