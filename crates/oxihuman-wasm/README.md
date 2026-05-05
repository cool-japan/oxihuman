# oxihuman-wasm

Part of the [OxiHuman](../../README.md) workspace — privacy-first, client-side human body generator in pure Rust.

**Status:** Stable | **Tests:** 168 passing | **API:** 68 WasmEngine methods | **Version:** 0.1.2 | **Updated:** 2026-05-05

WebAssembly bindings for OxiHuman — full browser-ready API via wasm-bindgen. A single `WasmEngine` struct exposes the entire OxiHuman pipeline to JavaScript/TypeScript, enabling privacy-preserving, client-side human body generation with no server round-trips.

---

## Feature Flags

```toml
[features]
default = []
wasm = ["dep:wasm-bindgen", "dep:js-sys"]
```

The `wasm` feature is optional. Enable it when targeting browser/Node.js environments via wasm-bindgen. Without it, the crate still compiles for testing and native use.

---

## Installation

```toml
[dependencies]
oxihuman-wasm = { version = "0.1.2", features = ["wasm"] }
```

---

## Structs

### `WasmEngine`

Main body generator. Exposes the full OxiHuman pipeline as 68 public `#[wasm_bindgen]` methods. Holds base mesh data, loaded morph targets, parameter state, physics proxies, and animation frame buffer.

### `ParticleSystem`

Point particle system with configurable `emit_rate` and `lifetime` control. Created via `WasmEngine::create_particle_system()` and stepped via `step_particles()`.

### `Particle`

Represents a single active particle. Carries `position` (Float32Array), `velocity` (Float32Array), and `age` (f32).

---

## WasmEngine API — 68 Methods

### Initialization (4)

| Method | Description |
|--------|-------------|
| `new_from_obj_bytes(bytes)` | Create engine from raw OBJ mesh bytes |
| `new_strict()` | Create engine with strict validation policy profile |
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
| `set_height(value)` | Set body height parameter (metres) |
| `set_weight(value)` | Set body weight parameter (normalized) |
| `set_muscle(value)` | Set musculature parameter |
| `set_age(value)` | Set age parameter |
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

### Mesh Building (3)

| Method | Description |
|--------|-------------|
| `build_mesh_bytes()` | Build binary mesh buffer containing positions, normals, UVs, and indices |
| `export_quantized_bytes()` | Export mesh in QMSH quantized binary format (compact, lossy) |
| `get_scene_json()` | Return scene description JSON (nodes, cameras, lights) |

### Physics (5)

| Method | Description |
|--------|-------------|
| `get_physics_proxies_json()` | Return collision proxy shapes as JSON |
| `get_physics_rig_json()` | Return full physics rig descriptor as JSON |
| `get_capsule_chains_json()` | Return capsule chain descriptors for limbs as JSON |
| `step_physics(dt)` | Step physics simulation by `dt` seconds (stub) |
| `set_wind(x, y, z)` | Set wind force vector for cloth/particle systems (stub) |

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

### Query & Analysis (6)

| Method | Description |
|--------|-------------|
| `get_vertex_count()` | Return number of vertices in the built mesh |
| `get_index_count()` | Return number of triangle indices in the built mesh |
| `get_curvature_map()` | Return per-vertex curvature values as Float32Array |
| `get_geodesic_distances(origin_index)` | Return geodesic distances from origin vertex as Float32Array |
| `query_sphere_near_point(x, y, z, radius)` | Return JSON array of vertex indices within sphere |
| `get_mesh_segments()` | Return JSON segment map (body part regions by vertex range) |

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

---

## Free Functions

| Function | Description |
|----------|-------------|
| `parse_mesh_bytes_header(buffer)` | Parse the format header from a binary mesh buffer; returns JSON with format version and field offsets |

---

## JavaScript/TypeScript Usage

```js
import init, { WasmEngine } from "oxihuman-wasm";

await init();

// Load base mesh from an OBJ file
const objBytes = new Uint8Array(await fetch("human_base.obj").then(r => r.arrayBuffer()));
const engine = WasmEngine.new_from_obj_bytes(objBytes);

// Load a ZIP asset pack with morph targets
const packBytes = new Uint8Array(await fetch("targets.zip").then(r => r.arrayBuffer()));
const count = engine.load_zip_pack_bytes(packBytes);
console.log(`Loaded ${count} targets`);

// Set body parameters
engine.set_height(1.75);
engine.set_weight(0.4);
engine.set_muscle(0.6);
engine.set_age(30.0);

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
- The `wasm` feature gate keeps native builds free of wasm-bindgen overhead, allowing the crate to be used in test harnesses and CLI pipelines without a browser target.

### v0.1.2 Internal Refactor

`engine.rs` is now a thin re-export module (7 lines). The implementation has been split into four focused source files with no public API changes:

| File | Contents |
|------|----------|
| `engine_core.rs` | `WasmEngine`, `ParticleSystem`, `Particle` struct definitions and initialization |
| `engine_anim.rs` | Animation recording, playback, and frame management methods |
| `engine_targets.rs` | Morph target loading, unloading, and weight management methods |
| `engine_io.rs` | JSON import/export, mesh building, physics, query, and preset methods |

Tests were moved to `wasm_tests.rs`. All public `#[wasm_bindgen]` method signatures are unchanged.

---

## License

Apache-2.0 — Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
