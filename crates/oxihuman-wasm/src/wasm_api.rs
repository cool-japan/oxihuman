// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Full wasm-bindgen JavaScript/TypeScript API for OxiHuman.
//!
//! Feature-gated behind `bindgen` (the canonical browser feature; the legacy
//! `wasm` feature is an alias). Enable with `--features bindgen` when
//! building for the browser target (`wasm32-unknown-unknown`).
//!
//! # Ownership model
//!
//! [`OxiHumanEngine`](crate::wasm_api::OxiHumanEngine) holds the engine state
//! in an `Rc<RefCell<WasmEngine>>`. Helper objects
//! ([`OxiHumanMorphSlider`](crate::wasm_api::OxiHumanMorphSlider),
//! [`OxiHumanAnimPlayer`](crate::wasm_api::OxiHumanAnimPlayer)) hold a
//! *clone* of that `Rc` instead of a raw pointer, so calling
//! `engine.free()` from JS can never leave a dangling pointer behind — the
//! shared engine state stays alive until the last helper is dropped.
//! WASM is single-threaded and no method calls back into JS while holding a
//! borrow, so the `RefCell` borrows are always short-lived and re-entrancy
//! is reported as a catchable JS error rather than a panic.
//!
//! # No-filesystem invariant
//!
//! **Root cause of the historical "recursive use of an object detected"
//! panic:** `export_glb()` used to write the GLB to
//! `std::env::temp_dir()` — on `wasm32-unknown-unknown` there is no
//! filesystem, and `std::env::temp_dir()` *panics* ("no filesystem on this
//! platform"). The panic became a WebAssembly trap that unwound past the
//! wasm-bindgen glue without releasing the object's internal `WasmRefCell`
//! borrow flag, permanently poisoning the JS object: every subsequent method
//! call (e.g. `export_obj()` after `build_mesh_bytes()`) then failed with
//! "recursive use of an object detected which would lead to unsafe aliasing
//! in rust". The fix is structural: **no method on this API may touch
//! `std::fs` / `std::env::temp_dir`** — all exporters go through the
//! in-memory byte builders in `oxihuman-export`
//! (`build_glb_bytes`, `mesh_to_obj_string`, `encode_stl_binary`,
//! `VrmExporter::export`, …).
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
    use std::cell::{Ref, RefCell, RefMut};
    use std::rc::Rc;

    use wasm_bindgen::prelude::*;

    use crate::engine::WasmEngine;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn anyhow_to_js(e: anyhow::Error) -> JsError {
        JsError::new(&e.to_string())
    }

    fn busy_error() -> JsError {
        JsError::new("OxiHumanEngine is busy (unexpected re-entrant call)")
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

    /// Return the crate version string (e.g. `"0.2.1"`).
    #[wasm_bindgen]
    pub fn get_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Return the module's `WebAssembly.Memory` object.
    ///
    /// Needed for the zero-copy geometry views
    /// (`new Float32Array(wasm_memory().buffer, engine.positions_ptr(), engine.positions_len())`)
    /// on targets whose JS glue does not re-export the memory (e.g.
    /// `--target nodejs`). Re-create views whenever `memory.buffer` changes
    /// identity (WebAssembly memory growth detaches old views) or
    /// `engine.mesh_generation()` bumps.
    #[wasm_bindgen]
    pub fn wasm_memory() -> JsValue {
        wasm_bindgen::memory()
    }

    // -----------------------------------------------------------------------
    // OxiHumanEngine — main JS class
    // -----------------------------------------------------------------------

    /// The primary OxiHuman engine exposed to JavaScript.
    ///
    /// Wraps [`WasmEngine`] behind a shared `Rc<RefCell<..>>` handle and
    /// exposes morphing, mesh export, animation and measurement APIs through
    /// wasm-bindgen.
    ///
    /// # Usage
    /// ```js
    /// const engine = new OxiHumanEngine();
    /// engine.set_param("height", 0.8);
    /// const bytes = engine.build_mesh_bytes();
    /// ```
    ///
    /// # Loading the core pack (recommended)
    /// ```js
    /// const bytes = new Uint8Array(await (await fetch(packUrl)).arrayBuffer());
    /// const engine = OxiHumanEngine.from_core_pack_bytes(bytes);
    /// ```
    #[wasm_bindgen]
    pub struct OxiHumanEngine {
        pub(crate) inner: Rc<RefCell<WasmEngine>>,
    }

    impl Default for OxiHumanEngine {
        fn default() -> Self {
            Self::new()
        }
    }

    impl OxiHumanEngine {
        fn wrap(engine: WasmEngine) -> Self {
            OxiHumanEngine {
                inner: Rc::new(RefCell::new(engine)),
            }
        }

        fn eng(&self) -> Result<Ref<'_, WasmEngine>, JsError> {
            self.inner.try_borrow().map_err(|_| busy_error())
        }

        fn eng_mut(&self) -> Result<RefMut<'_, WasmEngine>, JsError> {
            self.inner.try_borrow_mut().map_err(|_| busy_error())
        }
    }

    #[wasm_bindgen]
    impl OxiHumanEngine {
        /// Create a new engine with a minimal stub mesh.
        ///
        /// The stub mesh has 3 vertices (one degenerate triangle).  Call
        /// `from_core_pack_bytes`, `from_obj_bytes` or `load_zip_pack_bytes`
        /// to replace it with a real base mesh.
        #[wasm_bindgen(constructor)]
        pub fn new() -> OxiHumanEngine {
            // Infallible: the stub base mesh is constructed as a literal
            // (no parsing, no Result, no panic path).
            Self::wrap(WasmEngine::new_stub())
        }

        /// Create an engine pre-loaded with the given OBJ file bytes.
        ///
        /// `bytes` must be valid UTF-8 OBJ data.
        ///
        /// Throws a JavaScript `Error` if parsing fails.
        #[wasm_bindgen]
        pub fn from_obj_bytes(bytes: &[u8]) -> Result<OxiHumanEngine, JsError> {
            let inner = WasmEngine::new_from_obj_bytes(bytes).map_err(anyhow_to_js)?;
            Ok(Self::wrap(inner))
        }

        /// Create an engine pre-loaded with an OHPK v1 core pack.
        ///
        /// The pack's base mesh replaces the stub mesh, every pack target is
        /// loaded into the engine target store (driven by
        /// `set_param("height"|"weight"|"muscle"|"age", v)` according to its
        /// category), and `manifest.age_floor_years` is enforced on the `age`
        /// parameter.
        ///
        /// ```js
        /// const bytes = new Uint8Array(await (await fetch(packUrl)).arrayBuffer());
        /// const engine = OxiHumanEngine.from_core_pack_bytes(bytes);
        /// ```
        ///
        /// Throws a JavaScript `Error` if the pack is malformed.
        #[wasm_bindgen]
        pub fn from_core_pack_bytes(bytes: &[u8]) -> Result<OxiHumanEngine, JsError> {
            let inner = WasmEngine::new_from_core_pack_bytes(bytes).map_err(anyhow_to_js)?;
            Ok(Self::wrap(inner))
        }

        /// Load an OHPK v1 core pack into this engine, replacing the base
        /// mesh and the whole morph-target store (see
        /// [`Self::from_core_pack_bytes`]).
        ///
        /// Returns the number of morph targets loaded.
        /// Throws a JavaScript `Error` if the pack is malformed.
        #[wasm_bindgen]
        pub fn load_core_pack_bytes(&self, bytes: &[u8]) -> Result<u32, JsError> {
            self.eng_mut()?
                .load_core_pack_bytes(bytes)
                .map(|n| n as u32)
                .map_err(anyhow_to_js)
        }

        /// The minimum modelled age in years declared by the loaded core
        /// pack, or `undefined` when no floor applies.
        ///
        /// When set, `set_param("age", v)` clamps so the modelled age never
        /// goes below the floor, and `get_param("age")` reflects the clamped
        /// value.
        #[wasm_bindgen]
        pub fn age_floor_years(&self) -> Result<Option<f32>, JsError> {
            Ok(self.eng()?.age_floor_years())
        }

        /// Set a named morphing parameter.
        ///
        /// Well-known names: `"height"`, `"weight"`, `"muscle"`, `"age"`.
        /// Any other name is stored as an extra parameter and may drive a
        /// matching morph target by name.
        ///
        /// Values are typically in `[0.0, 1.0]`. The `age` parameter is
        /// clamped to the core pack's age floor when one is declared.
        #[wasm_bindgen]
        pub fn set_param(&self, name: &str, value: f64) -> Result<(), JsError> {
            let mut e = self.eng_mut()?;
            match name {
                "height" => e.set_height(value as f32),
                "weight" => e.set_weight(value as f32),
                "muscle" => e.set_muscle(value as f32),
                "age" => e.set_age(value as f32),
                _ => e.set_param(name, value as f32),
            }
            Ok(())
        }

        /// Get a named morphing parameter value.
        ///
        /// Returns `NaN` if the parameter name is not recognised.
        #[wasm_bindgen]
        pub fn get_param(&self, name: &str) -> Result<f64, JsError> {
            let e = self.eng()?;
            Ok(match name {
                "height" => e.params.height as f64,
                "weight" => e.params.weight as f64,
                "muscle" => e.params.muscle as f64,
                "age" => e.params.age as f64,
                other => e
                    .params
                    .extra
                    .get(other)
                    .copied()
                    .map(|v| v as f64)
                    .unwrap_or(f64::NAN),
            })
        }

        /// Reset all parameters to their default mid-point values and
        /// invalidate the mesh cache.
        #[wasm_bindgen]
        pub fn reset_params(&self) -> Result<(), JsError> {
            self.eng_mut()?.reset_params();
            Ok(())
        }

        /// Build the morphed mesh and return it as raw binary bytes.
        ///
        /// Uses the engine's incremental build path internally.
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
        pub fn build_mesh_bytes(&self) -> Result<Vec<u8>, JsError> {
            Ok(self.eng_mut()?.build_mesh_bytes())
        }

        /// Number of vertices in the base mesh.
        #[wasm_bindgen]
        pub fn vertex_count(&self) -> Result<u32, JsError> {
            Ok(self.eng()?.get_vertex_count())
        }

        // -------------------------------------------------------------------
        // Zero-copy geometry views (M4)
        // -------------------------------------------------------------------

        /// Recompute the persistent geometry buffers (incremental path,
        /// in-place) and return the current mesh generation.
        ///
        /// Call after `set_param(...)`; then read the buffers through the
        /// `positions_ptr()` / `positions_len()` (etc.) views. No-op when
        /// nothing changed.
        #[wasm_bindgen]
        pub fn refresh_geometry(&self) -> Result<u32, JsError> {
            Ok(self.eng_mut()?.refresh_geometry())
        }

        /// Byte offset of the flat `f32` positions buffer (`3 * n_verts`
        /// elements) inside WASM linear memory. Lazily refreshes the
        /// geometry when dirty.
        ///
        /// ```js
        /// const gen = engine.refresh_geometry();
        /// let view = new Float32Array(memory.buffer, engine.positions_ptr(), engine.positions_len());
        /// // Re-create `view` whenever engine.mesh_generation() != gen or
        /// // memory.buffer changed identity (wasm memory growth detaches views).
        /// ```
        #[wasm_bindgen]
        pub fn positions_ptr(&self) -> Result<u32, JsError> {
            let mut e = self.eng_mut()?;
            e.refresh_geometry();
            Ok(e.positions_ptr())
        }

        /// Number of `f32` elements in the positions buffer (`3 * n_verts`).
        #[wasm_bindgen]
        pub fn positions_len(&self) -> Result<u32, JsError> {
            let mut e = self.eng_mut()?;
            e.refresh_geometry();
            Ok(e.positions_len())
        }

        /// Byte offset of the flat `f32` normals buffer inside WASM linear memory.
        #[wasm_bindgen]
        pub fn normals_ptr(&self) -> Result<u32, JsError> {
            let mut e = self.eng_mut()?;
            e.refresh_geometry();
            Ok(e.normals_ptr())
        }

        /// Number of `f32` elements in the normals buffer (`3 * n_verts`).
        #[wasm_bindgen]
        pub fn normals_len(&self) -> Result<u32, JsError> {
            let mut e = self.eng_mut()?;
            e.refresh_geometry();
            Ok(e.normals_len())
        }

        /// Byte offset of the flat `f32` UV buffer inside WASM linear memory.
        #[wasm_bindgen]
        pub fn uvs_ptr(&self) -> Result<u32, JsError> {
            let mut e = self.eng_mut()?;
            e.refresh_geometry();
            Ok(e.uvs_ptr())
        }

        /// Number of `f32` elements in the UV buffer (`2 * n_verts`).
        #[wasm_bindgen]
        pub fn uvs_len(&self) -> Result<u32, JsError> {
            let mut e = self.eng_mut()?;
            e.refresh_geometry();
            Ok(e.uvs_len())
        }

        /// Byte offset of the `u32` triangle index buffer inside WASM linear memory.
        #[wasm_bindgen]
        pub fn indices_ptr(&self) -> Result<u32, JsError> {
            let mut e = self.eng_mut()?;
            e.refresh_geometry();
            Ok(e.indices_ptr())
        }

        /// Number of `u32` elements in the index buffer.
        #[wasm_bindgen]
        pub fn indices_len(&self) -> Result<u32, JsError> {
            let mut e = self.eng_mut()?;
            e.refresh_geometry();
            Ok(e.indices_len())
        }

        /// Current mesh generation. Bumps whenever the persistent geometry
        /// buffers were (re)allocated (topology / vertex-count change) — JS
        /// must re-create its typed-array views then. Also re-create views
        /// after WebAssembly memory growth.
        #[wasm_bindgen]
        pub fn mesh_generation(&self) -> Result<u32, JsError> {
            Ok(self.eng()?.mesh_generation())
        }

        // -------------------------------------------------------------------
        // Exports (all in-memory; no filesystem — see module docs)
        // -------------------------------------------------------------------

        /// Export the current morphed mesh as a binary GLB (glTF 2.0) byte buffer.
        ///
        /// Built entirely in memory via `oxihuman_export::glb::build_glb_bytes`
        /// — never touches the filesystem (which does not exist on wasm32 and
        /// used to *panic*, poisoning the engine object; see module docs).
        ///
        /// Throws a JavaScript `Error` if GLB serialization fails.
        #[wasm_bindgen]
        pub fn export_glb(&self) -> Result<Vec<u8>, JsError> {
            use oxihuman_export::glb::build_glb_bytes;

            let mesh = self.eng_mut()?.build_mesh_prepared();
            build_glb_bytes(&mesh).map_err(anyhow_to_js)
        }

        /// Export the current morphed mesh as a Wavefront OBJ string.
        ///
        /// Throws a JavaScript `Error` if serialization fails (previously
        /// failures were silently swallowed into an empty string).
        #[wasm_bindgen]
        pub fn export_obj(&self) -> Result<String, JsError> {
            use oxihuman_export::obj::mesh_to_obj_string;

            let mesh = self.eng_mut()?.build_mesh_prepared();
            mesh_to_obj_string(&mesh).map_err(anyhow_to_js)
        }

        /// Export the current morphed mesh as a VRM 1.0 avatar (`.vrm` bytes,
        /// a GLB container with the `VRMC_vrm` extension).
        ///
        /// Uses sensible defaults: name `"OxiHuman Avatar"`, CC-BY-4.0
        /// licence metadata, and a minimal 17-bone required-humanoid
        /// skeleton. Use [`Self::export_vrm_with_options`] to override the
        /// metadata.
        #[wasm_bindgen]
        pub fn export_vrm(&self) -> Result<Vec<u8>, JsError> {
            self.export_vrm_with_options("{}")
        }

        /// Export as VRM 1.0 with metadata overrides from a JSON object.
        ///
        /// Recognised keys (all optional):
        /// `{"name": string, "version": string, "authors": string[],
        ///   "license_url": string,
        ///   "commercial_usage": "personalNonProfit"|"personalProfit"|"corporation",
        ///   "credit_notation": "required"|"unnecessary",
        ///   "modification": "prohibited"|"allowModification"|"allowModificationRedistribution"}`
        ///
        /// Throws a JavaScript `Error` on malformed JSON or export failure.
        #[wasm_bindgen]
        pub fn export_vrm_with_options(&self, options_json: &str) -> Result<Vec<u8>, JsError> {
            use oxihuman_export::vrm_export::{
                VrmBoneName, VrmCommercialUsage, VrmCreditNotation, VrmExporter, VrmHumanBone,
                VrmHumanoid, VrmMeta, VrmModification,
            };

            let opts: serde_json::Value = serde_json::from_str(options_json)
                .map_err(|e| JsError::new(&format!("invalid VRM options JSON: {e}")))?;

            let mesh = self.eng_mut()?.build_mesh_prepared();

            let mut exporter = VrmExporter::new();
            // Gated MeshBuffers entry point (bodysuit invariant).
            exporter.set_mesh_buffers(&mesh).map_err(anyhow_to_js)?;

            // Minimal required-bone skeleton: hips-rooted chain covering the
            // 17 VRM-required humanoid bones with identity bind poses.
            let required = VrmBoneName::all_required();
            let bone_names: Vec<String> = required.iter().map(|b| b.as_str().to_string()).collect();
            let mut bone_parents: Vec<Option<usize>> = vec![Some(0); required.len()];
            bone_parents[0] = None; // hips is the root
            let identity: [f64; 16] = [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ];
            let bind_poses: Vec<[f64; 16]> = vec![identity; required.len()];
            exporter
                .set_skeleton(&bone_names, &bone_parents, &bind_poses)
                .map_err(anyhow_to_js)?;

            let humanoid = VrmHumanoid {
                bones: required
                    .iter()
                    .enumerate()
                    .map(|(i, &name)| VrmHumanBone {
                        name,
                        node_index: i,
                    })
                    .collect(),
            };
            exporter.set_humanoid(&humanoid).map_err(anyhow_to_js)?;

            let name = opts["name"].as_str().unwrap_or("OxiHuman Avatar");
            let mut meta = VrmMeta::default_cc_by(name);
            if let Some(v) = opts["version"].as_str() {
                meta.version = v.to_string();
            }
            if let Some(arr) = opts["authors"].as_array() {
                let authors: Vec<String> = arr
                    .iter()
                    .filter_map(|a| a.as_str().map(|s| s.to_string()))
                    .collect();
                if !authors.is_empty() {
                    meta.authors = authors;
                }
            }
            if let Some(v) = opts["license_url"].as_str() {
                meta.license_url = v.to_string();
            }
            if let Some(v) = opts["commercial_usage"].as_str() {
                meta.commercial_usage = match v {
                    "personalNonProfit" => VrmCommercialUsage::PersonalNonProfit,
                    "corporation" => VrmCommercialUsage::Corporation,
                    _ => VrmCommercialUsage::PersonalProfit,
                };
            }
            if let Some(v) = opts["credit_notation"].as_str() {
                meta.credit_notation = match v {
                    "unnecessary" => VrmCreditNotation::Unnecessary,
                    _ => VrmCreditNotation::Required,
                };
            }
            if let Some(v) = opts["modification"].as_str() {
                meta.modification = match v {
                    "prohibited" => VrmModification::Prohibited,
                    "allowModification" => VrmModification::AllowModification,
                    _ => VrmModification::AllowModificationRedistribution,
                };
            }
            exporter.set_meta(&meta).map_err(anyhow_to_js)?;

            exporter.export().map_err(anyhow_to_js)
        }

        /// Export the current morphed mesh as STL bytes.
        ///
        /// `binary = true` → binary STL; `binary = false` → ASCII STL text
        /// (as UTF-8 bytes). Both are built entirely in memory and pass the
        /// bodysuit export gate.
        #[wasm_bindgen]
        pub fn export_stl(&self, binary: bool) -> Result<Vec<u8>, JsError> {
            use oxihuman_export::stl::{encode_stl_binary, mesh_to_stl_ascii};

            let mesh = self.eng_mut()?.build_mesh_prepared();
            if binary {
                encode_stl_binary(&mesh).map_err(anyhow_to_js)
            } else {
                mesh_to_stl_ascii(&mesh, "oxihuman")
                    .map(String::into_bytes)
                    .map_err(anyhow_to_js)
            }
        }

        /// Return measurements for the current morphed body.
        ///
        /// All linear values are in **centimetres** and come from the precise
        /// cross-section measurer (chest / waist / hip are tape circumferences,
        /// height is stature); `weight_kg` is a mesh-volume mass. Consistent
        /// with `get_measurements_json()`.
        ///
        /// Returns an [`OxiHumanMeasurements`] object.
        #[wasm_bindgen]
        pub fn get_measurements(&self) -> Result<OxiHumanMeasurements, JsError> {
            let summary = self.eng_mut()?.tailoring_summary_cm();
            let s = summary.unwrap_or(oxihuman_morph::measurements::TailoringSummary {
                height_cm: 0.0,
                chest_cm: 0.0,
                waist_cm: 0.0,
                hip_cm: 0.0,
                weight_kg: 0.0,
            });
            Ok(OxiHumanMeasurements {
                height_cm: s.height_cm,
                chest_cm: s.chest_cm,
                waist_cm: s.waist_cm,
                hip_cm: s.hip_cm,
                weight_kg: s.weight_kg,
            })
        }

        /// Fit the engine's macro parameters so the re-measured mesh matches a
        /// set of target measurements, then return a JSON fit report.
        ///
        /// Input JSON accepts any subset of
        /// `{"height_cm":…, "chest_cm":…, "waist_cm":…, "hip_cm":…}` (plus an
        /// optional `{"max_iterations":n}`). The fit runs Nelder–Mead directly
        /// over the engine parameters (`height`, `weight`, `muscle`, `gender`),
        /// re-measuring the morphed mesh at every step, and leaves the engine
        /// set to the fitted parameters.
        ///
        /// Output JSON:
        /// ```json
        /// {"params":{"height":0.55,"weight":0.5,"muscle":0.5,"gender":0.5,"age":0.5},
        ///  "results":[{"name":"height","target_cm":172.0,"measured_cm":171.8,"delta_cm":-0.2}],
        ///  "iterations":47,"converged":true}
        /// ```
        ///
        /// Each `delta_cm` is `measured_cm − target_cm` from a final precise
        /// re-measurement of the fitted geometry — never an echo of the input.
        ///
        /// Throws a JavaScript `Error` on malformed JSON or when no valid
        /// target is supplied.
        #[wasm_bindgen]
        pub fn fit_to_measurements(&self, options_json: &str) -> Result<String, JsError> {
            self.eng_mut()?
                .fit_to_measurements(options_json)
                .map_err(anyhow_to_js)
        }

        /// Load a ZIP asset pack from raw bytes.
        ///
        /// The ZIP must contain one `.obj` file (base mesh) and any number of
        /// `.target` files (morph targets). Prefer OHPK core packs
        /// ([`Self::load_core_pack_bytes`]) for production.
        ///
        /// Returns the number of morph targets loaded.
        /// Throws a JavaScript `Error` if the ZIP is malformed or contains no `.obj`.
        #[wasm_bindgen]
        pub fn load_zip_pack_bytes(&self, bytes: &[u8]) -> Result<u32, JsError> {
            self.eng_mut()?
                .load_zip_pack_bytes(bytes)
                .map(|n| n as u32)
                .map_err(anyhow_to_js)
        }

        /// Load a morph target from raw `.target` file bytes.
        ///
        /// `name` is used to infer the morph category and auto-assign a weight
        /// function.  Throws a JavaScript `Error` if parsing fails.
        #[wasm_bindgen]
        pub fn load_target_bytes(&self, name: &str, bytes: &[u8]) -> Result<(), JsError> {
            self.eng_mut()?
                .load_target_bytes(name, bytes)
                .map_err(anyhow_to_js)
        }

        /// Load a morph target from a JSON descriptor.
        ///
        /// Expected format: `{"deltas":[[vid,dx,dy,dz],...]}`
        ///
        /// The target is applied by every mesh build once its weight is set
        /// via `set_target_weight`.
        ///
        /// Returns `true` on success, `false` on parse error.
        #[wasm_bindgen]
        pub fn load_target_from_json(&self, name: &str, json: &str) -> Result<bool, JsError> {
            Ok(self.eng_mut()?.load_target_from_json(name, json))
        }

        /// Unload a previously JSON-loaded target by name.
        ///
        /// Returns `true` if the target existed.
        #[wasm_bindgen]
        pub fn unload_target(&self, name: &str) -> Result<bool, JsError> {
            Ok(self.eng_mut()?.unload_target(name))
        }

        /// Set the blend weight for a JSON-loaded morph target.
        ///
        /// Returns `true` if the target was found.
        #[wasm_bindgen]
        pub fn set_target_weight(&self, name: &str, weight: f64) -> Result<bool, JsError> {
            Ok(self
                .eng_mut()?
                .set_target_weight_by_name(name, weight as f32))
        }

        /// Get the blend weight of a JSON-loaded morph target.
        ///
        /// Returns `-1.0` if the target is not found.
        #[wasm_bindgen]
        pub fn get_target_weight(&self, name: &str) -> Result<f64, JsError> {
            Ok(self.eng()?.get_target_weight_by_name(name) as f64)
        }

        /// Apply a named body preset (e.g. `"athletic"`, `"average"`, `"slender"`).
        ///
        /// Returns `true` if the preset was recognised and applied.
        #[wasm_bindgen]
        pub fn apply_preset(&self, name: &str) -> Result<bool, JsError> {
            Ok(self.eng_mut()?.apply_preset_by_name(name))
        }

        /// Export current params as a JSON string.
        #[wasm_bindgen]
        pub fn export_params_json(&self) -> Result<String, JsError> {
            Ok(self.eng()?.export_params_json())
        }

        /// Import params from a JSON string previously produced by
        /// `export_params_json`.
        ///
        /// Throws a JavaScript `Error` if the JSON is malformed.
        #[wasm_bindgen]
        pub fn import_params_json(&self, json: &str) -> Result<(), JsError> {
            self.eng_mut()?
                .import_params_json(json)
                .map_err(anyhow_to_js)
        }

        /// Return the number of engine-loaded morph targets.
        #[wasm_bindgen]
        pub fn target_count(&self) -> Result<u32, JsError> {
            Ok(self.eng()?.target_count() as u32)
        }

        /// Return the number of JSON-loaded morph targets.
        #[wasm_bindgen]
        pub fn loaded_target_count(&self) -> Result<u32, JsError> {
            Ok(self.eng()?.loaded_target_count())
        }

        /// Return a JSON array of the names of all JSON-loaded morph targets.
        #[wasm_bindgen]
        pub fn get_loaded_target_names(&self) -> Result<String, JsError> {
            Ok(self.eng()?.get_loaded_target_names())
        }

        /// Return a JSON array of the names of all engine-loaded morph targets.
        #[wasm_bindgen]
        pub fn list_loaded_targets(&self) -> Result<String, JsError> {
            Ok(self.eng()?.list_loaded_targets())
        }

        /// Return a compact JSON summary of current params.
        #[wasm_bindgen]
        pub fn get_param_summary_json(&self) -> Result<String, JsError> {
            Ok(self.eng()?.get_param_summary_json())
        }

        /// Return body proportion ratios as a JSON object.
        #[wasm_bindgen]
        pub fn get_body_proportions_json(&self) -> Result<String, JsError> {
            Ok(self.eng()?.get_body_proportions_json())
        }

        /// Return measurements as a JSON string (all linear values in
        /// centimetres; includes a `"units":"cm"` field).
        #[wasm_bindgen]
        pub fn get_measurements_json(&self) -> Result<String, JsError> {
            Ok(self.eng_mut()?.get_measurements_json())
        }

        /// Return physics collision proxies as a JSON string.
        #[wasm_bindgen]
        pub fn get_physics_proxies_json(&self) -> Result<String, JsError> {
            Ok(self.eng_mut()?.get_physics_proxies_json())
        }

        /// Return physics rig as a JSON string.
        #[wasm_bindgen]
        pub fn get_physics_rig_json(&self) -> Result<String, JsError> {
            Ok(self.eng_mut()?.get_physics_rig_json())
        }

        /// Return capsule chains as a JSON string.
        #[wasm_bindgen]
        pub fn get_capsule_chains_json(&self) -> Result<String, JsError> {
            Ok(self.eng_mut()?.get_capsule_chains_json())
        }

        /// Return the full scene as a JSON string (params + rig + vertex count).
        #[wasm_bindgen]
        pub fn get_scene_json(&self) -> Result<String, JsError> {
            Ok(self.eng_mut()?.get_scene_json())
        }

        /// Return an LOD-reduced scene JSON.
        ///
        /// `lod_level`: `0` = full, `1` = half, `2` = quarter.
        #[wasm_bindgen]
        pub fn get_lod_scene_json(&self, lod_level: u8) -> Result<String, JsError> {
            Ok(self.eng_mut()?.get_lod_scene_json(lod_level))
        }

        /// Return quantized mesh bytes (QMSH format).
        #[wasm_bindgen]
        pub fn export_quantized_bytes(&self) -> Result<Vec<u8>, JsError> {
            Ok(self.eng_mut()?.export_quantized_bytes())
        }

        /// Return per-vertex curvature as a JSON array of floats.
        #[wasm_bindgen]
        pub fn get_curvature_map(&self) -> Result<String, JsError> {
            Ok(self.eng_mut()?.get_curvature_map())
        }

        /// Return geodesic distances from `source_vertex` as a JSON array.
        #[wasm_bindgen]
        pub fn get_geodesic_distances(&self, source_vertex: u32) -> Result<String, JsError> {
            Ok(self.eng()?.get_geodesic_distances(source_vertex as usize))
        }

        /// Return vertex indices within `radius` of the given point as a JSON array.
        #[wasm_bindgen]
        pub fn query_sphere_near_point(
            &self,
            x: f64,
            y: f64,
            z: f64,
            radius: f64,
        ) -> Result<String, JsError> {
            Ok(self
                .eng()?
                .query_sphere_near_point(x as f32, y as f32, z as f32, radius as f32))
        }

        /// Return mesh connectivity segments as a JSON object.
        ///
        /// `mode`: `"connected"` or `"normals"`.
        #[wasm_bindgen]
        pub fn get_mesh_segments(&self, mode: &str) -> Result<String, JsError> {
            Ok(self.eng()?.get_mesh_segments(mode))
        }

        /// Initialise a cloth simulation from the most recently built mesh.
        ///
        /// Does nothing when no mesh has been built yet.
        /// `stiffness` is forwarded to all cloth springs; 0.0 = limp, 1.0 = rigid.
        #[wasm_bindgen]
        pub fn init_cloth(&self, stiffness: f64) -> Result<(), JsError> {
            self.eng_mut()?.init_cloth(stiffness as f32);
            Ok(())
        }

        /// Step physics simulation by `dt` seconds.
        #[wasm_bindgen]
        pub fn step_physics(&self, dt: f64) -> Result<(), JsError> {
            self.eng_mut()?.step_physics(dt as f32);
            Ok(())
        }

        /// Return current cloth simulation state as JSON.
        #[wasm_bindgen]
        pub fn get_cloth_state(&self) -> Result<String, JsError> {
            Ok(self.eng()?.get_cloth_state())
        }

        /// Return physics proxy data as JSON.
        #[wasm_bindgen]
        pub fn get_physics_proxy_json(&self) -> Result<String, JsError> {
            Ok(self.eng()?.get_physics_proxy_json())
        }

        /// Set the wind vector for physics simulation.
        #[wasm_bindgen]
        pub fn set_wind(&self, x: f64, y: f64, z: f64) -> Result<(), JsError> {
            self.eng_mut()?.set_wind(x as f32, y as f32, z as f32);
            Ok(())
        }

        /// Blend two expression presets by weight `t` (0 = a, 1 = b).
        ///
        /// Returns `true` if both preset names are recognised.
        #[wasm_bindgen]
        pub fn apply_expression_blend(
            &self,
            expr_a: &str,
            expr_b: &str,
            t: f64,
        ) -> Result<bool, JsError> {
            Ok(self
                .eng_mut()?
                .apply_expression_blend(expr_a, expr_b, t as f32))
        }

        /// Snapshot the current params as an animation keyframe.
        #[wasm_bindgen]
        pub fn record_anim_frame(&self) -> Result<(), JsError> {
            self.eng_mut()?.record_anim_frame();
            Ok(())
        }

        /// Return the number of recorded animation keyframes.
        #[wasm_bindgen]
        pub fn anim_frame_count(&self) -> Result<u32, JsError> {
            Ok(self.eng()?.anim_frame_count())
        }

        /// Seek to a specific animation frame, restoring its params snapshot.
        #[wasm_bindgen]
        pub fn seek_anim_frame(&self, frame: u32) -> Result<(), JsError> {
            self.eng_mut()?.seek_anim_frame(frame);
            Ok(())
        }

        /// Advance animation by `dt_seconds` and return the new frame index.
        #[wasm_bindgen]
        pub fn play_anim_step(&self, dt_seconds: f64) -> Result<u32, JsError> {
            Ok(self.eng_mut()?.play_anim_step(dt_seconds as f32))
        }

        /// Set animation playback speed in frames per second.
        #[wasm_bindgen]
        pub fn set_anim_fps(&self, fps: f64) -> Result<(), JsError> {
            self.eng_mut()?.set_anim_fps(fps as f32);
            Ok(())
        }

        /// Return the current animation playback speed in FPS.
        #[wasm_bindgen]
        pub fn get_anim_fps(&self) -> Result<f64, JsError> {
            Ok(self.eng()?.get_anim_fps() as f64)
        }

        /// Export all animation keyframes as a JSON array.
        #[wasm_bindgen]
        pub fn export_anim_json(&self) -> Result<String, JsError> {
            Ok(self.eng()?.export_anim_json())
        }

        /// Clear all recorded animation keyframes.
        #[wasm_bindgen]
        pub fn clear_anim_frames(&self) -> Result<(), JsError> {
            self.eng_mut()?.clear_anim_frames();
            Ok(())
        }

        /// Return a list of built-in shader names as a JSON array.
        #[wasm_bindgen]
        pub fn list_builtin_shaders(&self) -> Result<String, JsError> {
            Ok(self.eng()?.list_builtin_shaders())
        }

        /// Create a particle emitter with the given emit rate and particle lifetime.
        #[wasm_bindgen]
        pub fn create_particle_system(
            &self,
            emit_rate: f64,
            lifetime: f64,
        ) -> Result<bool, JsError> {
            Ok(self
                .eng_mut()?
                .create_particle_system(emit_rate as f32, lifetime as f32))
        }

        /// Advance the particle simulation by `dt` seconds.
        ///
        /// Returns JSON: `{"active": N, "positions": [[x,y,z], ...]}`.
        #[wasm_bindgen]
        pub fn step_particles(&self, dt: f64) -> Result<String, JsError> {
            Ok(self.eng_mut()?.step_particles(dt as f32))
        }

        /// Create an [`OxiHumanAnimPlayer`] sharing this engine's state.
        ///
        /// The player holds a shared handle (`Rc`) to the engine state, so it
        /// remains valid even if this `OxiHumanEngine` object is freed from
        /// JS — no dangling pointers.
        #[wasm_bindgen]
        pub fn make_anim_player(&self) -> OxiHumanAnimPlayer {
            OxiHumanAnimPlayer {
                engine: Rc::clone(&self.inner),
            }
        }
    }

    // -----------------------------------------------------------------------
    // OxiHumanMorphSlider — wraps a named param for range-slider UI binding
    // -----------------------------------------------------------------------

    /// A morph slider binding for use in slider-based UIs.
    ///
    /// Obtain a slider from a param name via
    /// [`OxiHumanMorphSlider::for_param`]. The slider holds a shared handle
    /// (`Rc`) to the engine state — freeing the engine object from JS does
    /// not invalidate the slider (no use-after-free is possible).
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
        min_val: f64,
        max_val: f64,
        engine: Rc<RefCell<WasmEngine>>,
    }

    impl OxiHumanMorphSlider {
        fn read_value(&self) -> f64 {
            let Ok(e) = self.engine.try_borrow() else {
                return f64::NAN;
            };
            match self.param_name.as_str() {
                "height" => e.params.height as f64,
                "weight" => e.params.weight as f64,
                "muscle" => e.params.muscle as f64,
                "age" => e.params.age as f64,
                other => e
                    .params
                    .extra
                    .get(other)
                    .copied()
                    .map(|v| v as f64)
                    .unwrap_or(f64::NAN),
            }
        }
    }

    #[wasm_bindgen]
    impl OxiHumanMorphSlider {
        /// Create a slider bound to `param_name` on `engine`.
        ///
        /// Well-known params (`height`, `weight`, `muscle`, `age`) have min=0,
        /// max=1.  Unknown extra params default to min=0, max=1.
        #[wasm_bindgen(js_name = "for_param")]
        pub fn for_param(engine: &OxiHumanEngine, param_name: &str) -> OxiHumanMorphSlider {
            OxiHumanMorphSlider {
                param_name: param_name.to_string(),
                min_val: 0.0,
                max_val: 1.0,
                engine: Rc::clone(&engine.inner),
            }
        }

        /// Return the parameter name this slider is bound to.
        #[wasm_bindgen]
        pub fn name(&self) -> String {
            self.param_name.clone()
        }

        /// Return the current slider value (read live from the engine).
        ///
        /// Returns `NaN` when the param is unknown.
        #[wasm_bindgen]
        pub fn value(&self) -> f64 {
            self.read_value()
        }

        /// Set a new slider value and propagate it to the engine.
        ///
        /// Values outside `[min, max]` are clamped.
        #[wasm_bindgen]
        pub fn set_value(&mut self, v: f64) -> Result<(), JsError> {
            let clamped = v.clamp(self.min_val, self.max_val);
            let mut e = self.engine.try_borrow_mut().map_err(|_| busy_error())?;
            match self.param_name.as_str() {
                "height" => e.set_height(clamped as f32),
                "weight" => e.set_weight(clamped as f32),
                "muscle" => e.set_muscle(clamped as f32),
                "age" => e.set_age(clamped as f32),
                other => {
                    let name = other.to_string();
                    e.set_param(&name, clamped as f32);
                }
            }
            Ok(())
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

        /// Estimated body mass in kilograms (body mesh volume × human mean
        /// density).
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
    /// The player holds a shared handle (`Rc`) to the engine state — freeing
    /// the engine object from JS does not invalidate the player.
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
        engine: Rc<RefCell<WasmEngine>>,
    }

    impl OxiHumanAnimPlayer {
        fn eng_mut(&self) -> Result<RefMut<'_, WasmEngine>, JsError> {
            self.engine.try_borrow_mut().map_err(|_| busy_error())
        }
    }

    #[wasm_bindgen]
    impl OxiHumanAnimPlayer {
        /// Snapshot the engine's current params as an animation keyframe.
        #[wasm_bindgen]
        pub fn record_frame(&mut self) -> Result<(), JsError> {
            self.eng_mut()?.record_anim_frame();
            Ok(())
        }

        /// Return the number of recorded keyframes.
        #[wasm_bindgen]
        pub fn frame_count(&mut self) -> Result<u32, JsError> {
            Ok(self.eng_mut()?.anim_frame_count())
        }

        /// Seek the engine to the given frame index.
        ///
        /// Out-of-range indices are silently ignored.
        #[wasm_bindgen]
        pub fn seek(&mut self, frame: u32) -> Result<(), JsError> {
            self.eng_mut()?.seek_anim_frame(frame);
            Ok(())
        }

        /// Advance playback by `dt_seconds`.
        ///
        /// Returns the new frame index.
        #[wasm_bindgen]
        pub fn step(&mut self, dt_seconds: f64) -> Result<u32, JsError> {
            Ok(self.eng_mut()?.play_anim_step(dt_seconds as f32))
        }

        /// Set animation playback speed in frames per second.
        #[wasm_bindgen]
        pub fn set_fps(&mut self, fps: f64) -> Result<(), JsError> {
            self.eng_mut()?.set_anim_fps(fps as f32);
            Ok(())
        }

        /// Return the current playback FPS.
        #[wasm_bindgen]
        pub fn get_fps(&mut self) -> Result<f64, JsError> {
            Ok(self.eng_mut()?.get_anim_fps() as f64)
        }

        /// Serialize all keyframes to a JSON array.
        ///
        /// Each element is an object of `{param_name: value, ...}`.
        #[wasm_bindgen]
        pub fn export_anim_json(&mut self) -> Result<String, JsError> {
            Ok(self.eng_mut()?.export_anim_json())
        }

        /// Clear all recorded keyframes and reset the playhead.
        #[wasm_bindgen]
        pub fn clear(&mut self) -> Result<(), JsError> {
            self.eng_mut()?.clear_anim_frames();
            Ok(())
        }
    }
}

// Re-export everything from the inner module into the crate namespace.
#[cfg(feature = "bindgen")]
pub use bindgen_impl::*;
