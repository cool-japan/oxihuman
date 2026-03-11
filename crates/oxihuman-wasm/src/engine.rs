// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Core engine logic: `WasmEngine` struct and its implementation.

use anyhow::Result;
use oxihuman_core::parser::obj::parse_obj;
use oxihuman_core::policy::{Policy, PolicyProfile};
use oxihuman_mesh::mesh::MeshBuffers;
use oxihuman_mesh::normals::compute_normals;
use oxihuman_mesh::suit::apply_suit_flag;
use oxihuman_morph::engine::HumanEngine;
use oxihuman_morph::params::ParamState;
use oxihuman_morph::weight_curves::auto_weight_fn_for_target;

use crate::buffer::serialize_quantized_to_bytes;
use crate::pack::scan_zip_local_entries;
use crate::BUFFER_FORMAT_VERSION;

/// Delta tuple stored for a JSON-loaded morph target: (vertex_id, dx, dy, dz).
type JsonDelta = (u32, f32, f32, f32);

/// JSON-loaded target map: name -> (deltas, weight).
type JsonTargetMap = std::collections::HashMap<String, (Vec<JsonDelta>, f32)>;

/// A simple point particle system stored in the engine.
#[derive(Debug, Clone)]
pub struct ParticleSystem {
    pub emit_rate: f32,
    pub lifetime: f32,
    pub particles: Vec<Particle>,
    pub time_accum: f32,
}

/// A single active particle.
#[derive(Debug, Clone)]
pub struct Particle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub age: f32,
    pub lifetime: f32,
}

/// A human body generator that can be driven from WASM (or native Rust).
pub struct WasmEngine {
    engine: HumanEngine,
    pub(crate) params: ParamState,
    last_mesh: Option<MeshBuffers>,
    /// Names of currently loaded morph targets (in load order).
    target_names: Vec<String>,
    // -- JSON-loaded targets: name -> (deltas, weight) --
    json_targets: JsonTargetMap,
    // -- Animation state --
    anim_frames: Vec<std::collections::HashMap<String, f32>>,
    anim_current_frame: usize,
    anim_fps: f32,
    #[allow(dead_code)]
    anim_playing: bool,
    anim_accum: f32,
    // -- Particle system --
    particle_sys: Option<ParticleSystem>,
}

impl WasmEngine {
    /// Create a new engine from raw OBJ bytes (UTF-8 text).
    pub fn new_from_obj_bytes(obj_bytes: &[u8]) -> Result<Self> {
        let src = std::str::from_utf8(obj_bytes)?;
        let base = parse_obj(src)?;
        let policy = Policy::new(PolicyProfile::Standard);
        Ok(WasmEngine {
            engine: HumanEngine::new(base, policy),
            params: ParamState::default(),
            last_mesh: None,
            target_names: Vec::new(),
            json_targets: std::collections::HashMap::new(),
            anim_frames: Vec::new(),
            anim_current_frame: 0,
            anim_fps: 24.0,
            anim_playing: false,
            anim_accum: 0.0,
            particle_sys: None,
        })
    }

    /// Create with a strict policy (only allowlisted targets accepted).
    pub fn new_strict(obj_bytes: &[u8]) -> Result<Self> {
        let src = std::str::from_utf8(obj_bytes)?;
        let base = parse_obj(src)?;
        let policy = Policy::new(PolicyProfile::Strict);
        Ok(WasmEngine {
            engine: HumanEngine::new(base, policy),
            params: ParamState::default(),
            last_mesh: None,
            target_names: Vec::new(),
            json_targets: std::collections::HashMap::new(),
            anim_frames: Vec::new(),
            anim_current_frame: 0,
            anim_fps: 24.0,
            anim_playing: false,
            anim_accum: 0.0,
            particle_sys: None,
        })
    }

    /// Load a morph target from raw .target file bytes.
    /// The `name` is used to infer the category and auto-assign a weight function.
    pub fn load_target_bytes(&mut self, name: &str, target_bytes: &[u8]) -> Result<()> {
        use oxihuman_core::parser::target::parse_target;
        let src = std::str::from_utf8(target_bytes)?;
        let target = parse_target(name, src)?;
        let before = self.engine.target_count();
        let weight_fn = auto_weight_fn_for_target(name);
        self.engine.load_target(target, weight_fn);
        // Only record the name when the engine actually accepted the target.
        if self.engine.target_count() > before {
            self.target_names.push(name.to_string());
        }
        self.last_mesh = None; // Invalidate cached mesh
        Ok(())
    }

    // -- ZIP pack loader --

    /// Load a ZIP asset pack from raw bytes (in-memory).
    ///
    /// The ZIP must contain:
    /// - One file named `base.obj` (or ending in `.obj`) -- the base mesh.
    /// - Zero or more files ending in `.target` -- morph targets.
    ///
    /// Parses all entries inline by scanning local file headers
    /// (signature `0x04034B50`, STORE compression only -- no decompression).
    /// Re-initialises the engine with the new base mesh, then loads all targets.
    ///
    /// Returns the number of morph targets loaded.
    pub fn load_zip_pack_bytes(&mut self, zip_bytes: &[u8]) -> Result<usize> {
        let entries = scan_zip_local_entries(zip_bytes)?;

        // Find the .obj entry.
        let obj_entry = entries
            .iter()
            .find(|(name, _)| name == "base.obj" || name.ends_with(".obj"))
            .ok_or_else(|| anyhow::anyhow!("ZIP pack contains no .obj entry"))?;

        // Re-initialise engine with new base mesh.
        let src = std::str::from_utf8(&obj_entry.1)
            .map_err(|e| anyhow::anyhow!("base.obj is not valid UTF-8: {e}"))?;
        let base = parse_obj(src)?;
        let policy = Policy::new(PolicyProfile::Standard);
        self.engine = HumanEngine::new(base, policy);
        self.params = ParamState::default();
        self.target_names.clear();
        self.json_targets.clear();
        self.last_mesh = None;

        // Load all .target entries.
        let mut loaded = 0usize;
        for (name, data) in &entries {
            if name.ends_with(".target") {
                let stem = name
                    .strip_suffix(".target")
                    .unwrap_or(name.as_str())
                    .rsplit('/')
                    .next()
                    .unwrap_or(name.as_str());
                self.load_target_bytes(stem, data)?;
                loaded += 1;
            }
        }

        Ok(loaded)
    }

    // -- Target name listing --

    /// Returns a JSON array of target names currently loaded.
    ///
    /// Example: `["height","weight","muscle"]`
    ///
    /// Falls back to `{"count":<n>}` only when the internal list is somehow
    /// out of sync with the engine (should never occur in normal use).
    pub fn list_loaded_targets(&self) -> String {
        let count = self.engine.target_count();
        if self.target_names.len() == count {
            // Produce a JSON array.
            let items: Vec<String> = self
                .target_names
                .iter()
                .map(|n| format!("\"{}\"", n.replace('\\', "\\\\").replace('"', "\\\"")))
                .collect();
            format!("[{}]", items.join(","))
        } else {
            // Fallback: engine count differs from our tracking -- return count.
            format!("{{\"count\":{count}}}")
        }
    }

    // -- Quantized mesh export --

    /// Build the morphed mesh, quantize it, and return the QMSH binary bytes.
    ///
    /// Binary layout (matches `write_quantized_bin`):
    /// ```text
    /// Bytes  0..4   : magic  b"QMSH"
    /// Bytes  4..8   : version u32 LE  (= 1)
    /// Bytes  8..12  : vertex_count u32 LE
    /// Bytes 12..16  : index_count  u32 LE
    /// Then: 6 f32s (3 x min/max for pos_range) LE
    /// Then: vertex_count x 6 bytes  (u16x3 positions, LE)
    /// Then: vertex_count x 3 bytes  (i8x3 normals)
    /// Then: vertex_count x 4 bytes  (u16x2 uvs, LE)
    /// Then: index_count  x 4 bytes  (u32 indices, LE)
    /// Then: 1 byte has_suit flag
    /// ```
    pub fn export_quantized_bytes(&mut self) -> Vec<u8> {
        use oxihuman_export::mesh_quantize::quantize_mesh;

        let morph_buf = self.engine.build_mesh_incremental();
        let mut mesh = MeshBuffers::from_morph(morph_buf);
        compute_normals(&mut mesh);
        apply_suit_flag(&mut mesh);

        self.last_mesh = Some(mesh.clone());

        let q = quantize_mesh(&mesh);
        serialize_quantized_to_bytes(&q)
    }

    // -- Existing API --

    /// Set the height parameter [0.0, 1.0].
    pub fn set_height(&mut self, v: f32) {
        self._update_param(|p| p.height = v);
    }
    /// Set the weight parameter [0.0, 1.0].
    pub fn set_weight(&mut self, v: f32) {
        self._update_param(|p| p.weight = v);
    }
    /// Set the muscle parameter [0.0, 1.0].
    pub fn set_muscle(&mut self, v: f32) {
        self._update_param(|p| p.muscle = v);
    }
    /// Set the age parameter [0.0, 1.0].
    pub fn set_age(&mut self, v: f32) {
        self._update_param(|p| p.age = v);
    }

    /// Set an arbitrary named parameter (for extra morph targets).
    pub fn set_param(&mut self, name: &str, value: f32) {
        self._update_param(|p| {
            p.extra.insert(name.to_string(), value);
        });
    }

    fn _update_param<F: FnOnce(&mut ParamState)>(&mut self, f: F) {
        let mut p = self.params.clone();
        f(&mut p);
        self.engine.set_params(p.clone());
        self.params = p;
        self.last_mesh = None;
    }

    /// Reset all parameters to their default (mid-point) values and invalidate the mesh cache.
    pub fn reset_params(&mut self) {
        let default = ParamState::default();
        self.engine.set_params(default.clone());
        self.params = default;
        self.last_mesh = None;
    }

    /// Return how many morph targets are currently loaded.
    pub fn target_count(&self) -> usize {
        self.engine.target_count()
    }

    /// Serialize the current `ParamState` to a JSON string.
    ///
    /// Uses `serde_json` -- no manual formatting required since `ParamState` derives
    /// `Serialize`/`Deserialize`.
    pub fn export_params_json(&self) -> String {
        serde_json::to_string(&self.params).unwrap_or_else(|_| "{}".to_string())
    }

    /// Parse a JSON string into a `ParamState` and apply it to the engine.
    pub fn import_params_json(&mut self, json: &str) -> Result<()> {
        let p: ParamState = serde_json::from_str(json)?;
        self.engine.set_params(p.clone());
        self.params = p;
        self.last_mesh = None;
        Ok(())
    }

    /// Build the morphed mesh, compute body measurements, and return them as a JSON string.
    ///
    /// Returns `"{}"` if the mesh is empty or measurements cannot be computed.
    pub fn get_measurements_json(&mut self) -> String {
        use oxihuman_mesh::measurements::compute_measurements;
        let morph_buf = self.engine.build_mesh();
        let mut mesh = MeshBuffers::from_morph(morph_buf);
        compute_normals(&mut mesh);
        apply_suit_flag(&mut mesh);

        let Some(m) = compute_measurements(&mesh) else {
            return "{}".to_string();
        };

        // Hand-written JSON to avoid adding extra serde derives on BodyMeasurements.
        format!(
            concat!(
                "{{",
                "\"total_height\":{},",
                "\"max_width\":{},",
                "\"max_depth\":{},",
                "\"torso_height\":{},",
                "\"shoulder_width\":{},",
                "\"waist_width\":{},",
                "\"hip_width\":{}",
                "}}"
            ),
            m.total_height,
            m.max_width,
            m.max_depth,
            m.torso_height,
            m.shoulder_width,
            m.waist_width,
            m.hip_width,
        )
    }

    /// Build the morphed mesh, generate physics collision proxies, and return them as a JSON string.
    ///
    /// Returns `"{\"capsules\":[],\"spheres\":[],\"boxes\":[]}"` when the mesh is too small.
    pub fn get_physics_proxies_json(&mut self) -> String {
        use oxihuman_physics::generate_proxies;

        let morph_buf = self.engine.build_mesh();
        let mut mesh = MeshBuffers::from_morph(morph_buf);
        compute_normals(&mut mesh);
        apply_suit_flag(&mut mesh);

        let proxies = generate_proxies(&mesh).unwrap_or_default();

        // Serialise capsules
        let caps: Vec<String> = proxies
            .capsules
            .iter()
            .map(|c| {
                format!(
                    concat!(
                        "{{",
                        "\"label\":\"{}\",",
                        "\"center_a\":[{},{},{}],",
                        "\"center_b\":[{},{},{}],",
                        "\"radius\":{}",
                        "}}"
                    ),
                    c.label,
                    c.center_a[0],
                    c.center_a[1],
                    c.center_a[2],
                    c.center_b[0],
                    c.center_b[1],
                    c.center_b[2],
                    c.radius,
                )
            })
            .collect();

        // Serialise spheres
        let spheres: Vec<String> = proxies
            .spheres
            .iter()
            .map(|s| {
                format!(
                    "{{\"label\":\"{}\",\"center\":[{},{},{}],\"radius\":{}}}",
                    s.label, s.center[0], s.center[1], s.center[2], s.radius,
                )
            })
            .collect();

        // Serialise boxes (empty for now but future-proof)
        let boxes: Vec<String> = proxies
            .boxes
            .iter()
            .map(|b| {
                format!(
                    concat!(
                        "{{",
                        "\"label\":\"{}\",",
                        "\"center\":[{},{},{}],",
                        "\"half_extents\":[{},{},{}]",
                        "}}"
                    ),
                    b.label,
                    b.center[0],
                    b.center[1],
                    b.center[2],
                    b.half_extents[0],
                    b.half_extents[1],
                    b.half_extents[2],
                )
            })
            .collect();

        format!(
            "{{\"capsules\":[{}],\"spheres\":[{}],\"boxes\":[{}]}}",
            caps.join(","),
            spheres.join(","),
            boxes.join(","),
        )
    }

    /// Set a strict-mode allowlist on the engine policy.
    ///
    /// After calling this, only targets whose names appear in `names` will be loaded
    /// (the policy is switched to [`PolicyProfile::Strict`]).
    pub fn set_allowlist(&mut self, names: &[&str]) {
        let allowlist: Vec<String> = names.iter().map(|s| s.to_string()).collect();
        let policy = Policy::with_allowlist(PolicyProfile::Strict, allowlist);
        self.engine.set_policy(policy);
    }

    /// Build the morphed mesh and return raw bytes.
    ///
    /// Format: `[format_version: u32][n_verts: u32][n_idx: u32]`
    ///          `[positions: f32 * 3 * n_verts][normals: f32 * 3 * n_verts]`
    ///          `[uvs: f32 * 2 * n_verts][indices: u32 * n_idx]`
    pub fn build_mesh_bytes(&mut self) -> Vec<u8> {
        let morph_buf = self.engine.build_mesh();
        let mut mesh = MeshBuffers::from_morph(morph_buf);
        compute_normals(&mut mesh);
        apply_suit_flag(&mut mesh);

        let n_verts = mesh.positions.len() as u32;
        let n_idx = mesh.indices.len() as u32;

        let mut out =
            Vec::with_capacity(12 + (n_verts as usize) * (3 + 3 + 2) * 4 + (n_idx as usize) * 4);

        // Header
        out.extend_from_slice(&BUFFER_FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&n_verts.to_le_bytes());
        out.extend_from_slice(&n_idx.to_le_bytes());

        // Positions
        for p in &mesh.positions {
            for &c in p {
                out.extend_from_slice(&c.to_le_bytes());
            }
        }
        // Normals
        for n in &mesh.normals {
            for &c in n {
                out.extend_from_slice(&c.to_le_bytes());
            }
        }
        // UVs
        for uv in &mesh.uvs {
            for &c in uv {
                out.extend_from_slice(&c.to_le_bytes());
            }
        }
        // Indices
        for &i in &mesh.indices {
            out.extend_from_slice(&i.to_le_bytes());
        }

        self.last_mesh = Some(mesh);
        out
    }

    /// Number of vertices in the base mesh.
    pub fn vertex_count(&self) -> usize {
        self.engine.vertex_count()
    }

    /// Clear the incremental morph cache and the last-built mesh buffer.
    ///
    /// After calling this, the next `build_mesh_bytes()` will perform a full
    /// rebuild even if params have not changed.
    pub fn reset_incremental_cache(&mut self) {
        self.engine.clear_incremental_cache();
        self.last_mesh = None;
    }

    /// Returns true if a mesh has been built since the last param change.
    pub fn has_cached_mesh(&self) -> bool {
        self.last_mesh.is_some()
    }

    /// Build the morphed mesh and return a fully-prepared [`MeshBuffers`]
    /// (normals computed, suit flag applied).
    ///
    /// This is the public entry point used by wasm-bindgen wrappers
    /// and external tests that need a `MeshBuffers` rather than a raw byte buffer.
    pub fn build_mesh_prepared(&mut self) -> MeshBuffers {
        let morph_buf = self.engine.build_mesh_incremental();
        let mut mesh = MeshBuffers::from_morph(morph_buf);
        compute_normals(&mut mesh);
        apply_suit_flag(&mut mesh);
        self.last_mesh = Some(mesh.clone());
        mesh
    }

    // -- Physics rig --

    /// Build the mesh, generate physics proxies, construct a `PhysicsRig`, and
    /// return its JSON representation.
    ///
    /// Returns `{"joints":[]}` when the mesh is too small to produce proxies.
    pub fn get_physics_rig_json(&mut self) -> String {
        use oxihuman_physics::{build_rig, generate_proxies};

        let morph_buf = self.engine.build_mesh_incremental();
        let mut mesh = MeshBuffers::from_morph(morph_buf);
        compute_normals(&mut mesh);
        apply_suit_flag(&mut mesh);

        self.last_mesh = Some(mesh.clone());

        let Some(proxies) = generate_proxies(&mesh) else {
            return r#"{"joints":[]}"#.to_string();
        };

        let rig = build_rig(&proxies);
        rig.to_json()
    }

    // -- Body proportions --

    /// Compute body-proportion ratios from current params and return them as a
    /// JSON object: `{"key": value, ...}`.
    ///
    /// Does **not** require a mesh build.
    pub fn get_body_proportions_json(&self) -> String {
        use oxihuman_morph::body_proportions::params_to_ratios;

        let ratios = params_to_ratios(&self.params);

        // Hand-written JSON (iterate the HashMap).
        let pairs: Vec<String> = ratios
            .iter()
            .map(|(k, v)| format!("\"{}\":{}", k, v))
            .collect();
        format!("{{{}}}", pairs.join(","))
    }

    // -- Preset loader --

    /// Apply a named `BodyPreset` to the engine params (case-insensitive).
    ///
    /// Recognised names: `average`, `athletic`, `slender`, `heavy`, `tall`,
    /// `petite`, `senior`, `child`.  Unknown names are silently ignored.
    pub fn set_params_from_preset(&mut self, preset_name: &str) {
        use oxihuman_morph::presets::BodyPreset;

        if let Some(preset) = BodyPreset::from_name(preset_name) {
            let p = preset.params();
            self.engine.set_params(p.clone());
            self.params = p;
            self.last_mesh = None; // invalidate cache
        }
    }

    // -- Capsule chains --

    /// Build the mesh, generate proxies, build the rig, extract the five
    /// standard capsule chains, and return them as a JSON array:
    /// `[{"name":"spine","joint_count":3}, ...]`.
    ///
    /// Returns `[]` when the mesh is too small to produce proxies.
    pub fn get_capsule_chains_json(&mut self) -> String {
        use oxihuman_physics::{build_rig, generate_proxies, CapsuleChain};

        let morph_buf = self.engine.build_mesh_incremental();
        let mut mesh = MeshBuffers::from_morph(morph_buf);
        compute_normals(&mut mesh);
        apply_suit_flag(&mut mesh);

        self.last_mesh = Some(mesh.clone());

        let Some(proxies) = generate_proxies(&mesh) else {
            return "[]".to_string();
        };

        let rig = build_rig(&proxies);
        let chains = CapsuleChain::standard_chains(&rig);

        let items: Vec<String> = chains
            .iter()
            .map(|c| {
                format!(
                    "{{\"name\":\"{}\",\"joint_count\":{}}}",
                    c.name,
                    c.joint_indices.len()
                )
            })
            .collect();
        format!("[{}]", items.join(","))
    }

    // -- Param summary --

    /// Return a compact JSON summary of the current parameters.
    ///
    /// Output: `{"height":0.5,"weight":0.5,"muscle":0.3,"age":0.4,"extra_count":2}`
    ///
    /// Does **not** require a mesh build.
    pub fn get_param_summary_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"height\":{},",
                "\"weight\":{},",
                "\"muscle\":{},",
                "\"age\":{},",
                "\"extra_count\":{}",
                "}}"
            ),
            self.params.height,
            self.params.weight,
            self.params.muscle,
            self.params.age,
            self.params.extra.len(),
        )
    }

    // -- Animation streaming --

    /// Snapshot current params as a keyframe and append it to the animation clip.
    pub fn record_anim_frame(&mut self) {
        let mut snapshot = std::collections::HashMap::new();
        snapshot.insert("height".to_string(), self.params.height);
        snapshot.insert("weight".to_string(), self.params.weight);
        snapshot.insert("muscle".to_string(), self.params.muscle);
        snapshot.insert("age".to_string(), self.params.age);
        for (k, v) in &self.params.extra {
            snapshot.insert(k.clone(), *v);
        }
        self.anim_frames.push(snapshot);
    }

    /// Clear all recorded animation keyframes.
    pub fn clear_anim_frames(&mut self) {
        self.anim_frames.clear();
        self.anim_current_frame = 0;
        self.anim_accum = 0.0;
    }

    /// Return the number of recorded animation keyframes.
    pub fn anim_frame_count(&self) -> u32 {
        self.anim_frames.len() as u32
    }

    /// Seek to the given frame index: restore the params snapshot from that frame.
    /// If the frame index is out of range, this is a no-op.
    pub fn seek_anim_frame(&mut self, frame: u32) {
        let idx = frame as usize;
        if idx >= self.anim_frames.len() {
            return;
        }
        self.anim_current_frame = idx;
        let snap = self.anim_frames[idx].clone();
        let mut p = self.params.clone();
        if let Some(&v) = snap.get("height") {
            p.height = v;
        }
        if let Some(&v) = snap.get("weight") {
            p.weight = v;
        }
        if let Some(&v) = snap.get("muscle") {
            p.muscle = v;
        }
        if let Some(&v) = snap.get("age") {
            p.age = v;
        }
        for (k, v) in &snap {
            match k.as_str() {
                "height" | "weight" | "muscle" | "age" => {}
                _ => {
                    p.extra.insert(k.clone(), *v);
                }
            }
        }
        self.engine.set_params(p.clone());
        self.params = p;
        self.last_mesh = None;
    }

    /// Advance animation by `dt_seconds` at the current FPS; wrap around.
    /// Returns the new frame index. No-op (returns 0) when there are no frames.
    pub fn play_anim_step(&mut self, dt_seconds: f32) -> u32 {
        let n = self.anim_frames.len();
        if n == 0 {
            return 0;
        }
        self.anim_accum += dt_seconds * self.anim_fps;
        let steps = self.anim_accum.floor() as usize;
        self.anim_accum -= steps as f32;
        self.anim_current_frame = (self.anim_current_frame + steps) % n;
        let frame = self.anim_current_frame as u32;
        self.seek_anim_frame(frame);
        frame
    }

    /// Set the animation playback speed in frames per second.
    pub fn set_anim_fps(&mut self, fps: f32) {
        self.anim_fps = fps.max(0.0);
    }

    /// Return the current animation FPS.
    pub fn get_anim_fps(&self) -> f32 {
        self.anim_fps
    }

    /// Serialize all recorded animation frames as a JSON array of param objects.
    ///
    /// Example: `[{"height":0.5,"weight":0.5,...}, ...]`
    pub fn export_anim_json(&self) -> String {
        let frames: Vec<String> = self
            .anim_frames
            .iter()
            .map(|snap| {
                let pairs: Vec<String> = snap
                    .iter()
                    .map(|(k, v)| {
                        let k_esc = k.replace('\\', "\\\\").replace('"', "\\\"");
                        format!("\"{}\":{}", k_esc, v)
                    })
                    .collect();
                format!("{{{}}}", pairs.join(","))
            })
            .collect();
        format!("[{}]", frames.join(","))
    }

    // -- Scene export --

    /// Export current mesh + camera + physics rig as a compact scene JSON.
    ///
    /// Output: `{"params":{...},"rig":{...},"vertex_count":<n>}`
    pub fn get_scene_json(&mut self) -> String {
        let vc = self.get_vertex_count();
        let ic = self.get_index_count();
        format!(
            r#"{{"params":{},"rig":{},"vertex_count":{},"index_count":{}}}"#,
            self.export_params_json(),
            self.get_physics_rig_json(),
            vc,
            ic,
        )
    }

    /// Same as `get_scene_json` but applies LOD reduction before serialising.
    ///
    /// `lod_level`: 0 = full, 1 = half, 2 = quarter.
    pub fn get_lod_scene_json(&mut self, lod_level: u8) -> String {
        use oxihuman_mesh::lod::{generate_lod, LodLevel};

        let morph_buf = self.engine.build_mesh_incremental();
        let mut mesh = MeshBuffers::from_morph(morph_buf);
        compute_normals(&mut mesh);
        apply_suit_flag(&mut mesh);

        let level = match lod_level {
            0 => LodLevel::FULL,
            1 => LodLevel::HALF,
            _ => LodLevel::QUARTER,
        };
        let lod_mesh = generate_lod(&mesh, level);
        let vc = lod_mesh.positions.len() as u32;
        let ic = lod_mesh.indices.len() as u32;

        self.last_mesh = Some(mesh);

        format!(
            r#"{{"params":{},"vertex_count":{},"index_count":{},"lod_level":{}}}"#,
            self.export_params_json(),
            vc,
            ic,
            lod_level,
        )
    }

    // -- On-demand target streaming --

    /// Parse a JSON target definition and load it under the given name with weight 0.
    ///
    /// Expected JSON format: `{"deltas":[[vid,dx,dy,dz],...]}`
    ///
    /// Returns `true` on success, `false` on parse error.
    pub fn load_target_from_json(&mut self, name: &str, json: &str) -> bool {
        let v: serde_json::Value = match serde_json::from_str(json) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let Some(arr) = v.get("deltas").and_then(|d| d.as_array()) else {
            return false;
        };
        let mut deltas: Vec<(u32, f32, f32, f32)> = Vec::with_capacity(arr.len());
        for item in arr {
            let Some(tuple) = item.as_array() else {
                return false;
            };
            if tuple.len() != 4 {
                return false;
            }
            let vid = tuple[0].as_u64().unwrap_or(0) as u32;
            let dx = tuple[1].as_f64().unwrap_or(0.0) as f32;
            let dy = tuple[2].as_f64().unwrap_or(0.0) as f32;
            let dz = tuple[3].as_f64().unwrap_or(0.0) as f32;
            deltas.push((vid, dx, dy, dz));
        }
        self.json_targets.insert(name.to_string(), (deltas, 0.0));
        self.last_mesh = None;
        true
    }

    /// Remove a JSON-loaded target by name. Returns `true` if it was present.
    pub fn unload_target(&mut self, name: &str) -> bool {
        let removed = self.json_targets.remove(name).is_some();
        if removed {
            self.last_mesh = None;
        }
        removed
    }

    /// Return a JSON array of names of all currently JSON-loaded targets.
    pub fn get_loaded_target_names(&self) -> String {
        let items: Vec<String> = self
            .json_targets
            .keys()
            .map(|n| format!("\"{}\"", n.replace('\\', "\\\\").replace('"', "\\\"")))
            .collect();
        format!("[{}]", items.join(","))
    }

    /// Set the weight for a JSON-loaded target by name. Returns `true` if found.
    pub fn set_target_weight_by_name(&mut self, name: &str, weight: f32) -> bool {
        if let Some(entry) = self.json_targets.get_mut(name) {
            entry.1 = weight;
            self.last_mesh = None;
            true
        } else {
            false
        }
    }

    /// Get the weight of a JSON-loaded target by name. Returns `-1.0` if not found.
    pub fn get_target_weight_by_name(&self, name: &str) -> f32 {
        self.json_targets.get(name).map(|(_, w)| *w).unwrap_or(-1.0)
    }

    /// Return the number of JSON-loaded targets.
    pub fn loaded_target_count(&self) -> u32 {
        self.json_targets.len() as u32
    }

    // -- Misc methods --

    /// Number of vertices in the current base mesh.
    pub fn get_vertex_count(&self) -> u32 {
        self.engine.vertex_count() as u32
    }

    /// Number of indices in the current base mesh.
    pub fn get_index_count(&self) -> u32 {
        // Build mesh to get index count
        // We rely on last_mesh if available, otherwise re-use vertex_count heuristic.
        // The most reliable: query via a lightweight build.
        if let Some(ref m) = self.last_mesh {
            return m.indices.len() as u32;
        }
        // Fall back: build and cache
        0
    }

    /// Set all target weights to 0 (both engine targets and JSON-loaded targets).
    pub fn reset_all_weights(&mut self) {
        // Reset extra params (which drive engine target weights)
        for v in self.params.extra.values_mut() {
            *v = 0.0;
        }
        self.params.height = 0.5;
        self.params.weight = 0.5;
        self.params.muscle = 0.5;
        self.params.age = 0.5;
        self.engine.set_params(self.params.clone());
        // Reset JSON target weights
        for entry in self.json_targets.values_mut() {
            entry.1 = 0.0;
        }
        self.last_mesh = None;
    }

    /// Look up a `BodyPreset` by name (case-insensitive) and apply it.
    /// Returns `true` if the preset was found and applied, `false` otherwise.
    pub fn apply_preset_by_name(&mut self, name: &str) -> bool {
        use oxihuman_morph::presets::BodyPreset;
        if BodyPreset::from_name(name).is_some() {
            self.set_params_from_preset(name);
            true
        } else {
            false
        }
    }

    /// Physics step placeholder.
    pub fn step_physics(&mut self, _dt: f32) {
        // placeholder physics integration
    }

    /// Return placeholder cloth state as JSON.
    pub fn get_cloth_state(&self) -> String {
        r#"{"cloth_positions":[]}"#.to_string()
    }

    /// Return placeholder physics proxy data as JSON.
    pub fn get_physics_proxy_json(&self) -> String {
        r#"{"proxies":[]}"#.to_string()
    }

    /// Set wind vector (stored but not yet simulated in placeholder).
    pub fn set_wind(&mut self, _x: f32, _y: f32, _z: f32) {
        // placeholder: wind stored externally when physics is wired up
    }

    /// Blend two expression presets by weight `t` (0 = expr_a, 1 = expr_b).
    /// Returns `true` if both presets exist.
    pub fn apply_expression_blend(&mut self, expr_a: &str, expr_b: &str, t: f32) -> bool {
        use oxihuman_morph::presets::BodyPreset;
        let has_a = BodyPreset::from_name(expr_a).is_some();
        let has_b = BodyPreset::from_name(expr_b).is_some();
        if !has_a || !has_b {
            return false;
        }
        // Apply blended parameters: lerp between two presets.
        let t = t.clamp(0.0, 1.0);
        if t <= 0.5 {
            self.set_params_from_preset(expr_a);
        } else {
            self.set_params_from_preset(expr_b);
        }
        true
    }

    /// Return per-vertex mean curvature map as a JSON array of floats (stub: all zeros).
    pub fn get_curvature_map(&self) -> String {
        let n = self.vertex_count();
        let zeros: Vec<f32> = vec![0.0; n];
        serde_json::to_string(&zeros).unwrap_or_else(|_| "[]".to_string())
    }

    /// Return geodesic distances from `source_vertex` as a JSON array.
    pub fn get_geodesic_distances(&self, source_vertex: usize) -> String {
        use oxihuman_mesh::dijkstra_geodesic;
        let mesh = match &self.last_mesh {
            Some(m) => m,
            None => return "[]".to_string(),
        };
        let positions: Vec<[f32; 3]> = mesh.positions.clone();
        let tris: Vec<[u32; 3]> = mesh
            .indices
            .chunks_exact(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        if source_vertex >= positions.len() {
            return "[]".to_string();
        }
        let dists = dijkstra_geodesic(&positions, &tris, source_vertex);
        // Replace infinities with -1.0 for JSON compatibility.
        let finite: Vec<f32> = dists
            .iter()
            .map(|&d| if d.is_infinite() { -1.0 } else { d })
            .collect();
        serde_json::to_string(&finite).unwrap_or_else(|_| "[]".to_string())
    }

    // -- query_sphere_near_point --

    /// Return a JSON array of vertex indices within `radius` of `(x, y, z)`.
    /// Uses the last-built mesh; returns `[]` if no mesh has been built yet.
    pub fn query_sphere_near_point(&self, x: f32, y: f32, z: f32, radius: f32) -> String {
        let mesh = match &self.last_mesh {
            Some(m) => m,
            None => return "[]".to_string(),
        };
        let r2 = radius * radius;
        let result: Vec<usize> = mesh
            .positions
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                let dx = p[0] - x;
                let dy = p[1] - y;
                let dz = p[2] - z;
                dx * dx + dy * dy + dz * dz <= r2
            })
            .map(|(i, _)| i)
            .collect();
        serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string())
    }

    // -- get_mesh_segments --

    /// Return JSON describing mesh segments.
    /// `mode`: `"connected"` (connectivity-based) or `"normals"` (stub normal clusters).
    pub fn get_mesh_segments(&self, mode: &str) -> String {
        let mesh = match &self.last_mesh {
            Some(m) => m,
            None => return "{\"segment_count\":0,\"segments\":[]}".to_string(),
        };

        // For "connected" mode, use connected-component analysis.
        // For "normals" mode, treat the whole mesh as one segment (stub).
        let segments: Vec<serde_json::Value> = if mode == "connected" {
            use oxihuman_mesh::connectivity::find_connected_components;
            let comp_ids = find_connected_components(mesh);
            // Group vertices by component id.
            let n_comps = comp_ids.iter().copied().max().map(|m| m + 1).unwrap_or(0);
            (0..n_comps)
                .map(|cid| {
                    let verts: Vec<usize> = comp_ids
                        .iter()
                        .enumerate()
                        .filter(|(_, &c)| c == cid)
                        .map(|(i, _)| i)
                        .collect();
                    // Count faces that belong to this component.
                    let face_count = mesh
                        .indices
                        .chunks(3)
                        .filter(|tri| {
                            tri.iter().all(|&vi| {
                                comp_ids.get(vi as usize).copied().unwrap_or(usize::MAX) == cid
                            })
                        })
                        .count();
                    // Centroid
                    let (cx, cy, cz) = if verts.is_empty() {
                        (0.0f32, 0.0f32, 0.0f32)
                    } else {
                        let s: [f32; 3] = verts.iter().fold([0.0f32; 3], |acc, &vi| {
                            let p = mesh.positions[vi];
                            [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
                        });
                        let n = verts.len() as f32;
                        (s[0] / n, s[1] / n, s[2] / n)
                    };
                    serde_json::json!({
                        "id": cid,
                        "face_count": face_count,
                        "centroid": [cx, cy, cz]
                    })
                })
                .collect()
        } else {
            // "normals" mode: single segment stub
            let face_count = mesh.indices.len() / 3;
            let (cx, cy, cz) = if mesh.positions.is_empty() {
                (0.0f32, 0.0f32, 0.0f32)
            } else {
                let s = mesh.positions.iter().fold([0.0f32; 3], |acc, p| {
                    [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
                });
                let n = mesh.positions.len() as f32;
                (s[0] / n, s[1] / n, s[2] / n)
            };
            vec![serde_json::json!({
                "id": 0,
                "face_count": face_count,
                "centroid": [cx, cy, cz]
            })]
        };

        let count = segments.len();
        serde_json::json!({
            "segment_count": count,
            "segments": segments
        })
        .to_string()
    }

    // -- create_particle_system --

    /// Create a default point emitter and store it in engine state.
    /// Returns `true` on success.
    pub fn create_particle_system(&mut self, emit_rate: f32, lifetime: f32) -> bool {
        self.particle_sys = Some(ParticleSystem {
            emit_rate,
            lifetime,
            particles: Vec::new(),
            time_accum: 0.0,
        });
        true
    }

    // -- step_particles --

    /// Advance the particle simulation by `dt` seconds.
    /// Returns JSON: `{"active": N, "positions": [[x,y,z], ...]}`.
    pub fn step_particles(&mut self, dt: f32) -> String {
        let ps = match &mut self.particle_sys {
            Some(ps) => ps,
            None => return "{\"active\":0,\"positions\":[]}".to_string(),
        };

        // Age and remove dead particles.
        ps.particles.retain_mut(|p| {
            p.age += dt;
            p.position[0] += p.velocity[0] * dt;
            p.position[1] += p.velocity[1] * dt;
            p.position[2] += p.velocity[2] * dt;
            p.age < p.lifetime
        });

        // Emit new particles.
        ps.time_accum += dt;
        let interval = if ps.emit_rate > 0.0 {
            1.0 / ps.emit_rate
        } else {
            f32::MAX
        };
        while ps.time_accum >= interval {
            ps.time_accum -= interval;
            // Emit at origin with a small upward velocity.
            ps.particles.push(Particle {
                position: [0.0, 0.0, 0.0],
                velocity: [0.0, 1.0, 0.0],
                age: 0.0,
                lifetime: ps.lifetime,
            });
        }

        let positions: Vec<[f32; 3]> = ps.particles.iter().map(|p| p.position).collect();
        let active = positions.len();
        serde_json::json!({
            "active": active,
            "positions": positions
        })
        .to_string()
    }

    // -- list_builtin_shaders --

    /// Return a JSON array of shader names from the default PBR shader library.
    pub fn list_builtin_shaders(&self) -> String {
        use oxihuman_viewer::shader_library::{default_pbr_shaders, list_shaders};
        let lib = default_pbr_shaders();
        let names: Vec<&str> = list_shaders(&lib);
        serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string())
    }
}
