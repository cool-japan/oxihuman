// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Full wasm-bindgen JavaScript/TypeScript API for OxiHuman.
//!
//! Feature-gated behind `bindgen`. Enable with `--features bindgen` when
//! building for the browser target (`wasm32-unknown-unknown`).
//!
//! # Example (JavaScript)
//! ```js
//! import init, { OxiHumanEngine, set_panic_hook, get_version } from './oxihuman_wasm.js';
//! await init();
//! set_panic_hook();
//! console.log(get_version());
//! const engine = new OxiHumanEngine();
//! engine.set_param("height", 0.7);
//! const meshBytes = engine.build_mesh_bytes();
//! ```

#[cfg(feature = "bindgen")]
mod bindgen_impl {
    use wasm_bindgen::prelude::*;

    use crate::engine::WasmEngine;

    // -----------------------------------------------------------------------
    // Helper: convert anyhow::Error to JsError
    // -----------------------------------------------------------------------
    fn anyhow_to_js(e: anyhow::Error) -> JsError {
        JsError::new(&e.to_string())
    }

    // -----------------------------------------------------------------------
    // Free functions
    // -----------------------------------------------------------------------

    /// Install `console.error` as the Rust panic hook.
    ///
    /// Call this once at startup before any other API call so that Rust panics
    /// appear in the browser developer console rather than as cryptic
    /// `unreachable` WebAssembly traps.
    #[wasm_bindgen]
    pub fn set_panic_hook() {
        console_error_panic_hook::set_once();
    }

    /// Return the crate version string (e.g. `"0.1.0"`).
    #[wasm_bindgen]
    pub fn get_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    // -----------------------------------------------------------------------
    // OxiHumanEngine — main JS class
    // -----------------------------------------------------------------------

    /// The primary OxiHuman engine exposed to JavaScript.
    ///
    /// Wraps [`WasmEngine`] and exposes morphing, mesh export, animation and
    /// measurement APIs through wasm-bindgen.
    ///
    /// # Usage
    /// ```js
    /// const engine = new OxiHumanEngine();
    /// engine.set_param("height", 0.8);
    /// const bytes = engine.build_mesh_bytes();
    /// ```
    #[wasm_bindgen]
    pub struct OxiHumanEngine {
        pub(crate) inner: WasmEngine,
    }

    impl Default for OxiHumanEngine {
        fn default() -> Self {
            Self::new()
        }
    }

    #[wasm_bindgen]
    impl OxiHumanEngine {
        /// Create a new engine with a minimal stub mesh.
        ///
        /// The stub mesh has 3 vertices (one degenerate triangle).  Call
        /// `from_obj_bytes` or `load_zip_pack_bytes` to replace it with a real
        /// base mesh.
        #[wasm_bindgen(constructor)]
        pub fn new() -> OxiHumanEngine {
            // Minimal valid OBJ with a single degenerate triangle so the engine
            // initialises without error.
            const STUB_OBJ: &[u8] =
                b"v 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nvn 0 0 1\nf 1/1/1 2/2/1 3/3/1\n";
            let inner = WasmEngine::new_from_obj_bytes(STUB_OBJ)
                .expect("oxihuman-wasm: stub engine init must not fail");
            OxiHumanEngine { inner }
        }

        /// Create an engine pre-loaded with the given OBJ file bytes.
        ///
        /// `bytes` must be valid UTF-8 OBJ data.
        ///
        /// Throws a JavaScript `Error` if parsing fails.
        #[wasm_bindgen]
        pub fn from_obj_bytes(bytes: &[u8]) -> Result<OxiHumanEngine, JsError> {
            let inner = WasmEngine::new_from_obj_bytes(bytes).map_err(anyhow_to_js)?;
            Ok(OxiHumanEngine { inner })
        }

        /// Set a named morphing parameter.
        ///
        /// Well-known names: `"height"`, `"weight"`, `"muscle"`, `"age"`.
        /// Any other name is stored as an extra parameter and may drive a
        /// matching morph target by name.
        ///
        /// Values are typically in `[0.0, 1.0]`.
        #[wasm_bindgen]
        pub fn set_param(&mut self, name: &str, value: f64) {
            match name {
                "height" => self.inner.set_height(value as f32),
                "weight" => self.inner.set_weight(value as f32),
                "muscle" => self.inner.set_muscle(value as f32),
                "age" => self.inner.set_age(value as f32),
                _ => self.inner.set_param(name, value as f32),
            }
        }

        /// Get a named morphing parameter value.
        ///
        /// Returns `NaN` if the parameter name is not recognised.
        #[wasm_bindgen]
        pub fn get_param(&self, name: &str) -> f64 {
            match name {
                "height" => self.inner.params.height as f64,
                "weight" => self.inner.params.weight as f64,
                "muscle" => self.inner.params.muscle as f64,
                "age" => self.inner.params.age as f64,
                other => self
                    .inner
                    .params
                    .extra
                    .get(other)
                    .copied()
                    .map(|v| v as f64)
                    .unwrap_or(f64::NAN),
            }
        }

        /// Reset all parameters to their default mid-point values and
        /// invalidate the mesh cache.
        #[wasm_bindgen]
        pub fn reset_params(&mut self) {
            self.inner.reset_params();
        }

        /// Build the morphed mesh and return it as raw binary bytes.
        ///
        /// Binary format — see [`crate::BUFFER_FORMAT_VERSION`] and `MeshBytes`:
        /// - Bytes 0–3:   `version` (u32 LE, currently `1`)
        /// - Bytes 4–7:   `vertex_count` N (u32 LE)
        /// - Bytes 8–11:  `index_count`  M (u32 LE)
        /// - Bytes 12..:  positions  f32\[N\*3\]
        /// - Then:        normals    f32\[N\*3\]
        /// - Then:        uvs        f32\[N\*2\]
        /// - Then:        indices    u32\[M\]
        #[wasm_bindgen]
        pub fn build_mesh_bytes(&mut self) -> Vec<u8> {
            self.inner.build_mesh_bytes()
        }

        /// Number of vertices in the base mesh.
        #[wasm_bindgen]
        pub fn vertex_count(&self) -> u32 {
            self.inner.get_vertex_count()
        }

        /// Export the current morphed mesh as a binary GLB (glTF 2.0) byte buffer.
        ///
        /// The returned bytes can be passed directly to `URL.createObjectURL`
        /// or written to a `.glb` file.
        ///
        /// Throws a JavaScript `Error` if GLB serialization fails.
        #[wasm_bindgen]
        pub fn export_glb(&mut self) -> Result<Vec<u8>, JsError> {
            use oxihuman_export::glb::export_glb;

            let mesh = self.inner.build_mesh_prepared();
            // GLB export writes to a file path — use a temp file.
            let tmp_path = std::env::temp_dir().join("oxihuman_wasm_export.glb");
            export_glb(&mesh, &tmp_path).map_err(|e| JsError::new(&e.to_string()))?;
            std::fs::read(&tmp_path).map_err(|e| JsError::new(&e.to_string()))
        }

        /// Export the current morphed mesh as a Wavefront OBJ string.
        #[wasm_bindgen]
        pub fn export_obj(&mut self) -> String {
            use oxihuman_export::obj::mesh_to_obj_string;

            let mesh = self.inner.build_mesh_prepared();
            mesh_to_obj_string(&mesh).unwrap_or_else(|_| String::new())
        }

        /// Return measurements for the current morphed body.
        ///
        /// Returns an [`OxiHumanMeasurements`] object.
        #[wasm_bindgen]
        pub fn get_measurements(&mut self) -> OxiHumanMeasurements {
            use oxihuman_mesh::measurements::compute_measurements;

            let mesh = self.inner.build_mesh_prepared();

            let (height_cm, chest_cm, waist_cm, hip_cm, weight_kg) = compute_measurements(&mesh)
                .map(|m| {
                    let h = m.total_height * 100.0;
                    let c = m.max_width * 100.0 * std::f32::consts::PI;
                    let w = m.waist_width * 100.0 * std::f32::consts::PI;
                    let hip = m.hip_width * 100.0 * std::f32::consts::PI;
                    // BMI-21 weight estimate: height_m² × 21
                    let height_m = m.total_height;
                    let kg = height_m * height_m * 21.0;
                    (h, c, w, hip, kg)
                })
                .unwrap_or((0.0, 0.0, 0.0, 0.0, 0.0));

            OxiHumanMeasurements {
                height_cm: height_cm as f64,
                chest_cm: chest_cm as f64,
                waist_cm: waist_cm as f64,
                hip_cm: hip_cm as f64,
                weight_kg: weight_kg as f64,
            }
        }

        /// Load a ZIP asset pack from raw bytes.
        ///
        /// The ZIP must contain one `.obj` file (base mesh) and any number of
        /// `.target` files (morph targets).
        ///
        /// Returns the number of morph targets loaded.
        /// Throws a JavaScript `Error` if the ZIP is malformed or contains no `.obj`.
        #[wasm_bindgen]
        pub fn load_zip_pack_bytes(&mut self, bytes: &[u8]) -> Result<u32, JsError> {
            self.inner
                .load_zip_pack_bytes(bytes)
                .map(|n| n as u32)
                .map_err(anyhow_to_js)
        }

        /// Load a morph target from raw `.target` file bytes.
        ///
        /// `name` is used to infer the morph category and auto-assign a weight
        /// function.  Throws a JavaScript `Error` if parsing fails.
        #[wasm_bindgen]
        pub fn load_target_bytes(&mut self, name: &str, bytes: &[u8]) -> Result<(), JsError> {
            self.inner
                .load_target_bytes(name, bytes)
                .map_err(anyhow_to_js)
        }

        /// Load a morph target from a JSON descriptor.
        ///
        /// Expected format: `{"deltas":[[vid,dx,dy,dz],...]}`
        ///
        /// Returns `true` on success, `false` on parse error.
        #[wasm_bindgen]
        pub fn load_target_from_json(&mut self, name: &str, json: &str) -> bool {
            self.inner.load_target_from_json(name, json)
        }

        /// Unload a previously JSON-loaded target by name.
        ///
        /// Returns `true` if the target existed.
        #[wasm_bindgen]
        pub fn unload_target(&mut self, name: &str) -> bool {
            self.inner.unload_target(name)
        }

        /// Set the blend weight for a JSON-loaded morph target.
        ///
        /// Returns `true` if the target was found.
        #[wasm_bindgen]
        pub fn set_target_weight(&mut self, name: &str, weight: f64) -> bool {
            self.inner.set_target_weight_by_name(name, weight as f32)
        }

        /// Get the blend weight of a JSON-loaded morph target.
        ///
        /// Returns `-1.0` if the target is not found.
        #[wasm_bindgen]
        pub fn get_target_weight(&self, name: &str) -> f64 {
            self.inner.get_target_weight_by_name(name) as f64
        }

        /// Apply a named body preset (e.g. `"athletic"`, `"average"`, `"slender"`).
        ///
        /// Returns `true` if the preset was recognised and applied.
        #[wasm_bindgen]
        pub fn apply_preset(&mut self, name: &str) -> bool {
            self.inner.apply_preset_by_name(name)
        }

        /// Export current params as a JSON string.
        #[wasm_bindgen]
        pub fn export_params_json(&self) -> String {
            self.inner.export_params_json()
        }

        /// Import params from a JSON string previously produced by
        /// `export_params_json`.
        ///
        /// Throws a JavaScript `Error` if the JSON is malformed.
        #[wasm_bindgen]
        pub fn import_params_json(&mut self, json: &str) -> Result<(), JsError> {
            self.inner.import_params_json(json).map_err(anyhow_to_js)
        }

        /// Return the number of engine-loaded morph targets.
        #[wasm_bindgen]
        pub fn target_count(&self) -> u32 {
            self.inner.target_count() as u32
        }

        /// Return the number of JSON-loaded morph targets.
        #[wasm_bindgen]
        pub fn loaded_target_count(&self) -> u32 {
            self.inner.loaded_target_count()
        }

        /// Return a JSON array of the names of all JSON-loaded morph targets.
        #[wasm_bindgen]
        pub fn get_loaded_target_names(&self) -> String {
            self.inner.get_loaded_target_names()
        }

        /// Return a JSON array of the names of all engine-loaded morph targets.
        #[wasm_bindgen]
        pub fn list_loaded_targets(&self) -> String {
            self.inner.list_loaded_targets()
        }

        /// Return a compact JSON summary of current params.
        #[wasm_bindgen]
        pub fn get_param_summary_json(&self) -> String {
            self.inner.get_param_summary_json()
        }

        /// Return body proportion ratios as a JSON object.
        #[wasm_bindgen]
        pub fn get_body_proportions_json(&self) -> String {
            self.inner.get_body_proportions_json()
        }

        /// Return measurements as a JSON string.
        #[wasm_bindgen]
        pub fn get_measurements_json(&mut self) -> String {
            self.inner.get_measurements_json()
        }

        /// Return physics collision proxies as a JSON string.
        #[wasm_bindgen]
        pub fn get_physics_proxies_json(&mut self) -> String {
            self.inner.get_physics_proxies_json()
        }

        /// Return physics rig as a JSON string.
        #[wasm_bindgen]
        pub fn get_physics_rig_json(&mut self) -> String {
            self.inner.get_physics_rig_json()
        }

        /// Return capsule chains as a JSON string.
        #[wasm_bindgen]
        pub fn get_capsule_chains_json(&mut self) -> String {
            self.inner.get_capsule_chains_json()
        }

        /// Return the full scene as a JSON string (params + rig + vertex count).
        #[wasm_bindgen]
        pub fn get_scene_json(&mut self) -> String {
            self.inner.get_scene_json()
        }

        /// Return an LOD-reduced scene JSON.
        ///
        /// `lod_level`: `0` = full, `1` = half, `2` = quarter.
        #[wasm_bindgen]
        pub fn get_lod_scene_json(&mut self, lod_level: u8) -> String {
            self.inner.get_lod_scene_json(lod_level)
        }

        /// Return quantized mesh bytes (QMSH format).
        #[wasm_bindgen]
        pub fn export_quantized_bytes(&mut self) -> Vec<u8> {
            self.inner.export_quantized_bytes()
        }

        /// Return per-vertex curvature as a JSON array of floats.
        #[wasm_bindgen]
        pub fn get_curvature_map(&self) -> String {
            self.inner.get_curvature_map()
        }

        /// Return geodesic distances from `source_vertex` as a JSON array.
        #[wasm_bindgen]
        pub fn get_geodesic_distances(&self, source_vertex: u32) -> String {
            self.inner.get_geodesic_distances(source_vertex as usize)
        }

        /// Return vertex indices within `radius` of the given point as a JSON array.
        #[wasm_bindgen]
        pub fn query_sphere_near_point(&self, x: f64, y: f64, z: f64, radius: f64) -> String {
            self.inner
                .query_sphere_near_point(x as f32, y as f32, z as f32, radius as f32)
        }

        /// Return mesh connectivity segments as a JSON object.
        ///
        /// `mode`: `"connected"` or `"normals"`.
        #[wasm_bindgen]
        pub fn get_mesh_segments(&self, mode: &str) -> String {
            self.inner.get_mesh_segments(mode)
        }

        /// Step physics simulation by `dt` seconds.
        #[wasm_bindgen]
        pub fn step_physics(&mut self, dt: f64) {
            self.inner.step_physics(dt as f32);
        }

        /// Return current cloth simulation state as JSON.
        #[wasm_bindgen]
        pub fn get_cloth_state(&self) -> String {
            self.inner.get_cloth_state()
        }

        /// Return physics proxy data as JSON.
        #[wasm_bindgen]
        pub fn get_physics_proxy_json(&self) -> String {
            self.inner.get_physics_proxy_json()
        }

        /// Set the wind vector for physics simulation.
        #[wasm_bindgen]
        pub fn set_wind(&mut self, x: f64, y: f64, z: f64) {
            self.inner.set_wind(x as f32, y as f32, z as f32);
        }

        /// Blend two expression presets by weight `t` (0 = a, 1 = b).
        ///
        /// Returns `true` if both preset names are recognised.
        #[wasm_bindgen]
        pub fn apply_expression_blend(&mut self, expr_a: &str, expr_b: &str, t: f64) -> bool {
            self.inner.apply_expression_blend(expr_a, expr_b, t as f32)
        }

        /// Snapshot the current params as an animation keyframe.
        #[wasm_bindgen]
        pub fn record_anim_frame(&mut self) {
            self.inner.record_anim_frame();
        }

        /// Return the number of recorded animation keyframes.
        #[wasm_bindgen]
        pub fn anim_frame_count(&self) -> u32 {
            self.inner.anim_frame_count()
        }

        /// Seek to a specific animation frame, restoring its params snapshot.
        #[wasm_bindgen]
        pub fn seek_anim_frame(&mut self, frame: u32) {
            self.inner.seek_anim_frame(frame);
        }

        /// Advance animation by `dt_seconds` and return the new frame index.
        #[wasm_bindgen]
        pub fn play_anim_step(&mut self, dt_seconds: f64) -> u32 {
            self.inner.play_anim_step(dt_seconds as f32)
        }

        /// Set animation playback speed in frames per second.
        #[wasm_bindgen]
        pub fn set_anim_fps(&mut self, fps: f64) {
            self.inner.set_anim_fps(fps as f32);
        }

        /// Return the current animation playback speed in FPS.
        #[wasm_bindgen]
        pub fn get_anim_fps(&self) -> f64 {
            self.inner.get_anim_fps() as f64
        }

        /// Export all animation keyframes as a JSON array.
        #[wasm_bindgen]
        pub fn export_anim_json(&self) -> String {
            self.inner.export_anim_json()
        }

        /// Clear all recorded animation keyframes.
        #[wasm_bindgen]
        pub fn clear_anim_frames(&mut self) {
            self.inner.clear_anim_frames();
        }

        /// Return a list of built-in shader names as a JSON array.
        #[wasm_bindgen]
        pub fn list_builtin_shaders(&self) -> String {
            self.inner.list_builtin_shaders()
        }

        /// Create a particle emitter with the given emit rate and particle lifetime.
        #[wasm_bindgen]
        pub fn create_particle_system(&mut self, emit_rate: f64, lifetime: f64) -> bool {
            self.inner
                .create_particle_system(emit_rate as f32, lifetime as f32)
        }

        /// Advance the particle simulation by `dt` seconds.
        ///
        /// Returns JSON: `{"active": N, "positions": [[x,y,z], ...]}`.
        #[wasm_bindgen]
        pub fn step_particles(&mut self, dt: f64) -> String {
            self.inner.step_particles(dt as f32)
        }

        /// Create an [`OxiHumanAnimPlayer`] wrapping this engine's animation state.
        ///
        /// The player delegates all calls back to `self`, providing a
        /// semantically grouped animation sub-API.
        ///
        /// # Safety / ownership
        /// The player stores a raw pointer to `self`.  It must not outlive
        /// this engine.  Both live on the JS heap in WASM (single-threaded),
        /// so lifetime issues cannot arise in practice.
        #[wasm_bindgen]
        pub fn make_anim_player(&mut self) -> OxiHumanAnimPlayer {
            OxiHumanAnimPlayer {
                engine_ptr: self as *mut OxiHumanEngine,
            }
        }
    }

    // -----------------------------------------------------------------------
    // OxiHumanMorphSlider — wraps a named param for range-slider UI binding
    // -----------------------------------------------------------------------

    /// A morph slider binding for use in slider-based UIs.
    ///
    /// Obtain a slider from a param name via
    /// [`OxiHumanMorphSlider::for_param`].
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const slider = OxiHumanMorphSlider.for_param(engine, "height");
    /// console.log(slider.name(), slider.min(), slider.max(), slider.value());
    /// slider.set_value(0.8);
    /// ```
    #[wasm_bindgen]
    pub struct OxiHumanMorphSlider {
        param_name: String,
        current_value: f64,
        min_val: f64,
        max_val: f64,
        /// Raw pointer back to the engine for write-through on `set_value`.
        engine_ptr: *mut OxiHumanEngine,
    }

    // SAFETY: WASM is single-threaded. The raw pointer is only dereferenced
    // on the JS thread that owns both the engine and the slider.
    unsafe impl Send for OxiHumanMorphSlider {}
    unsafe impl Sync for OxiHumanMorphSlider {}

    #[wasm_bindgen]
    impl OxiHumanMorphSlider {
        /// Create a slider bound to `param_name` on `engine`.
        ///
        /// Well-known params (`height`, `weight`, `muscle`, `age`) have min=0,
        /// max=1.  Unknown extra params default to min=0, max=1.
        #[wasm_bindgen(js_name = "for_param")]
        pub fn for_param(engine: &mut OxiHumanEngine, param_name: &str) -> OxiHumanMorphSlider {
            let value = engine.get_param(param_name);
            OxiHumanMorphSlider {
                param_name: param_name.to_string(),
                current_value: value,
                min_val: 0.0,
                max_val: 1.0,
                engine_ptr: engine as *mut OxiHumanEngine,
            }
        }

        /// Return the parameter name this slider is bound to.
        #[wasm_bindgen]
        pub fn name(&self) -> String {
            self.param_name.clone()
        }

        /// Return the current slider value.
        #[wasm_bindgen]
        pub fn value(&self) -> f64 {
            self.current_value
        }

        /// Set a new slider value and propagate it to the engine.
        ///
        /// Values outside `[min, max]` are clamped.
        #[wasm_bindgen]
        pub fn set_value(&mut self, v: f64) {
            let clamped = v.clamp(self.min_val, self.max_val);
            self.current_value = clamped;
            // SAFETY: single-threaded WASM; pointer is always valid while
            // the slider is alive (JS GC keeps the engine alive).
            unsafe {
                (*self.engine_ptr).set_param(&self.param_name, clamped);
            }
        }

        /// Return the minimum allowed value (always `0.0` for standard params).
        #[wasm_bindgen]
        pub fn min(&self) -> f64 {
            self.min_val
        }

        /// Return the maximum allowed value (always `1.0` for standard params).
        #[wasm_bindgen]
        pub fn max(&self) -> f64 {
            self.max_val
        }
    }

    // -----------------------------------------------------------------------
    // OxiHumanMeasurements — returned by engine.get_measurements()
    // -----------------------------------------------------------------------

    /// Body measurements derived from the morphed mesh.
    ///
    /// All linear measurements are in centimetres; `weight_kg` is kilograms.
    ///
    /// Obtained via [`OxiHumanEngine::get_measurements`].
    #[wasm_bindgen]
    pub struct OxiHumanMeasurements {
        height_cm: f64,
        chest_cm: f64,
        waist_cm: f64,
        hip_cm: f64,
        weight_kg: f64,
    }

    #[wasm_bindgen]
    impl OxiHumanMeasurements {
        /// Standing height in centimetres.
        #[wasm_bindgen]
        pub fn height_cm(&self) -> f64 {
            self.height_cm
        }

        /// Chest circumference estimate in centimetres.
        #[wasm_bindgen]
        pub fn chest_cm(&self) -> f64 {
            self.chest_cm
        }

        /// Waist circumference estimate in centimetres.
        #[wasm_bindgen]
        pub fn waist_cm(&self) -> f64 {
            self.waist_cm
        }

        /// Hip circumference estimate in centimetres.
        #[wasm_bindgen]
        pub fn hip_cm(&self) -> f64 {
            self.hip_cm
        }

        /// Estimated body mass in kilograms (BMI-21 heuristic).
        #[wasm_bindgen]
        pub fn weight_kg(&self) -> f64 {
            self.weight_kg
        }
    }

    // -----------------------------------------------------------------------
    // OxiHumanAnimPlayer — animation recording and playback
    // -----------------------------------------------------------------------

    /// Animation recording and playback controller.
    ///
    /// Obtain one from [`OxiHumanEngine::make_anim_player`].
    ///
    /// The player stores a raw pointer to its parent engine and delegates all
    /// calls back to it.  The engine must not be dropped while the player is
    /// alive.  In a browser WASM context both live on the JS heap, so GC
    /// ordering is safe as long as the engine reference is kept alive.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const player = engine.make_anim_player();
    /// engine.set_param("height", 0.2); player.record_frame();
    /// engine.set_param("height", 0.8); player.record_frame();
    /// player.set_fps(30);
    /// console.log(player.frame_count()); // 2
    /// const json = player.export_anim_json();
    /// player.clear();
    /// ```
    #[wasm_bindgen]
    pub struct OxiHumanAnimPlayer {
        pub(crate) engine_ptr: *mut OxiHumanEngine,
    }

    // SAFETY: WASM is single-threaded.
    unsafe impl Send for OxiHumanAnimPlayer {}
    unsafe impl Sync for OxiHumanAnimPlayer {}

    impl OxiHumanAnimPlayer {
        fn engine(&mut self) -> &mut OxiHumanEngine {
            // SAFETY: single-threaded WASM; pointer is always valid while
            // the player is alive.
            unsafe { &mut *self.engine_ptr }
        }
    }

    #[wasm_bindgen]
    impl OxiHumanAnimPlayer {
        /// Snapshot the engine's current params as an animation keyframe.
        #[wasm_bindgen]
        pub fn record_frame(&mut self) {
            self.engine().record_anim_frame();
        }

        /// Return the number of recorded keyframes.
        #[wasm_bindgen]
        pub fn frame_count(&mut self) -> u32 {
            self.engine().anim_frame_count()
        }

        /// Seek the engine to the given frame index.
        ///
        /// Out-of-range indices are silently ignored.
        #[wasm_bindgen]
        pub fn seek(&mut self, frame: u32) {
            self.engine().seek_anim_frame(frame);
        }

        /// Advance playback by `dt_seconds`.
        ///
        /// Returns the new frame index.
        #[wasm_bindgen]
        pub fn step(&mut self, dt_seconds: f64) -> u32 {
            self.engine().play_anim_step(dt_seconds)
        }

        /// Set animation playback speed in frames per second.
        #[wasm_bindgen]
        pub fn set_fps(&mut self, fps: f64) {
            self.engine().set_anim_fps(fps);
        }

        /// Return the current playback FPS.
        #[wasm_bindgen]
        pub fn get_fps(&mut self) -> f64 {
            self.engine().get_anim_fps()
        }

        /// Serialize all keyframes to a JSON array.
        ///
        /// Each element is an object of `{param_name: value, ...}`.
        #[wasm_bindgen]
        pub fn export_anim_json(&mut self) -> String {
            self.engine().export_anim_json()
        }

        /// Clear all recorded keyframes and reset the playhead.
        #[wasm_bindgen]
        pub fn clear(&mut self) {
            self.engine().clear_anim_frames();
        }
    }
}

// Re-export everything from the inner module into the crate namespace.
#[cfg(feature = "bindgen")]
pub use bindgen_impl::*;
