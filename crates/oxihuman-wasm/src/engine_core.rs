// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Core `WasmEngine` struct definition, constructors, param setters, and mesh build methods.

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
pub(crate) type JsonDelta = (u32, f32, f32, f32);

/// JSON-loaded target map: name -> (deltas, weight).
pub(crate) type JsonTargetMap = std::collections::HashMap<String, (Vec<JsonDelta>, f32)>;

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
    pub(crate) engine: HumanEngine,
    pub(crate) params: ParamState,
    pub(crate) last_mesh: Option<MeshBuffers>,
    /// Names of currently loaded morph targets (in load order).
    pub(crate) target_names: Vec<String>,
    // -- JSON-loaded targets: name -> (deltas, weight) --
    pub(crate) json_targets: JsonTargetMap,
    // -- Animation state --
    pub(crate) anim_frames: Vec<std::collections::HashMap<String, f32>>,
    pub(crate) anim_current_frame: usize,
    pub(crate) anim_fps: f32,
    #[allow(dead_code)]
    pub(crate) anim_playing: bool,
    pub(crate) anim_accum: f32,
    // -- Particle system --
    pub(crate) particle_sys: Option<ParticleSystem>,
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

    // -- Param setters --

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

    pub(crate) fn _update_param<F: FnOnce(&mut ParamState)>(&mut self, f: F) {
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

    /// Set a strict-mode allowlist on the engine policy.
    ///
    /// After calling this, only targets whose names appear in `names` will be loaded
    /// (the policy is switched to [`PolicyProfile::Strict`]).
    pub fn set_allowlist(&mut self, names: &[&str]) {
        let allowlist: Vec<String> = names.iter().map(|s| s.to_string()).collect();
        let policy = Policy::with_allowlist(PolicyProfile::Strict, allowlist);
        self.engine.set_policy(policy);
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

    /// Number of vertices in the current base mesh.
    pub fn get_vertex_count(&self) -> u32 {
        self.engine.vertex_count() as u32
    }

    /// Number of indices in the current base mesh.
    pub fn get_index_count(&self) -> u32 {
        if let Some(ref m) = self.last_mesh {
            return m.indices.len() as u32;
        }
        // Fall back: build and cache
        0
    }
}
