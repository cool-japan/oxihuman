// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Core `WasmEngine` struct definition, constructors, param setters, and mesh build methods.

use anyhow::Result;
use oxihuman_core::parser::obj::{parse_obj, ObjMesh};
use oxihuman_core::policy::{Policy, PolicyProfile};
use oxihuman_mesh::mesh::MeshBuffers;
use oxihuman_mesh::normals::compute_normals;
use oxihuman_mesh::suit::apply_suit_flag;
use oxihuman_morph::engine::HumanEngine;
use oxihuman_morph::params::ParamState;
use oxihuman_morph::weight_curves::auto_weight_fn_for_target;
use oxihuman_physics::{BodyProxies, ClothSim, WindConfig, WindField};

use crate::buffer::serialize_quantized_to_bytes;
use crate::pack::scan_zip_local_entries;
use crate::BUFFER_FORMAT_VERSION;

/// Delta tuple stored for a JSON-loaded morph target: (vertex_id, dx, dy, dz).
pub(crate) type JsonDelta = (u32, f32, f32, f32);

/// JSON-loaded target map: name -> (deltas, weight).
pub(crate) type JsonTargetMap = std::collections::HashMap<String, (Vec<JsonDelta>, f32)>;

/// A morph-target weight function: current param state → blend weight.
type TargetWeightFn = Box<dyn Fn(&ParamState) -> f32 + Send + Sync>;

/// Centimetres per model unit.
///
/// OxiHuman model units follow the MakeHuman convention: one unit is one
/// decimetre (`MODEL_UNIT_MM = 100.0` in `oxihuman_export::core_pack`), i.e.
/// 10 cm.  All WASM measurement APIs report centimetres.
pub const MODEL_UNIT_CM: f32 = oxihuman_export::core_pack::MODEL_UNIT_MM / 10.0;

/// Convert the normalised `age` parameter `[0, 1]` to modelled years.
///
/// Follows the MakeHuman macro-age convention: `0.0` → 1 year,
/// `0.5` → 25 years, `1.0` → 90 years (piecewise linear).
pub fn age_param_to_years(param: f32) -> f32 {
    let p = param.clamp(0.0, 1.0);
    if p <= 0.5 {
        1.0 + p * 48.0
    } else {
        25.0 + (p - 0.5) * 130.0
    }
}

/// Inverse of [`age_param_to_years`]: modelled years to the normalised `age`
/// parameter, clamped to `[0, 1]`.
pub fn age_years_to_param(years: f32) -> f32 {
    if years <= 25.0 {
        ((years - 1.0) / 48.0).clamp(0.0, 1.0)
    } else {
        (0.5 + (years - 25.0) / 130.0).clamp(0.0, 1.0)
    }
}

/// Macro-slider level for a 3-way (min / average / max) MakeHuman modifier.
#[derive(Clone, Copy, PartialEq, Eq)]
enum MacroLevel {
    Min,
    Avg,
    Max,
}

/// Partition-of-unity weight of a 3-level macro modifier at slider value `s`.
///
/// `w_min + w_avg + w_max == 1` for every `s in [0, 1]`:
/// * `Min` ramps 1→0 over `[0, 0.5]`, then 0.
/// * `Max` is 0 over `[0, 0.5]`, then ramps 0→1.
/// * `Avg` is the triangular remainder (1 at `s = 0.5`, 0 at the ends).
fn macro_level_weight(level: MacroLevel, s: f32) -> f32 {
    let s = s.clamp(0.0, 1.0);
    let w_min = (1.0 - 2.0 * s).clamp(0.0, 1.0);
    let w_max = (2.0 * s - 1.0).clamp(0.0, 1.0);
    match level {
        MacroLevel::Min => w_min,
        MacroLevel::Max => w_max,
        MacroLevel::Avg => (1.0 - w_min - w_max).clamp(0.0, 1.0),
    }
}

/// Read the androgyny slider from an extra param (`0` = male, `1` = female).
/// Defaults to `0.5` (balanced) so male/female corner targets contribute
/// equally unless a caller drives `set_param("gender", …)`.
fn gender_slider(p: &ParamState) -> f32 {
    p.extra
        .get("gender")
        .copied()
        .unwrap_or(0.5)
        .clamp(0.0, 1.0)
}

/// Detect the `min|average|max` level for a modifier keyword (e.g. `muscle`)
/// inside a target basename such as `…-maxmuscle-averageweight`.
fn detect_level(basename: &str, keyword: &str) -> Option<MacroLevel> {
    if basename.contains(&format!("max{keyword}")) {
        Some(MacroLevel::Max)
    } else if basename.contains(&format!("min{keyword}")) {
        Some(MacroLevel::Min)
    } else if basename.contains(&format!("average{keyword}")) {
        Some(MacroLevel::Avg)
    } else {
        None
    }
}

/// MakeHuman macro-modifier weight function for a *macrodetails* target,
/// implementing the partition-of-unity blend the corner targets were authored
/// for. Returns `None` for targets that are not part of the macro system (the
/// caller then falls back to the category/name heuristic).
///
/// Families handled:
/// * `universal-{gender}-{age}-{min|average|max}muscle-{…}weight` — the body
///   corner targets. Weight = `gender · age · muscle · weight` partition
///   product, so at the neutral slider centre (all `0.5`) every present corner
///   evaluates to ~0 and the mesh stays at the base (average) body instead of
///   summing ~30 corners into a giant.
/// * `…/height/{gender}-…-{min|max}height` — driven by the height slider,
///   split across the two gender corners.
/// * `{ethnicity}-{gender}-{age}` — the ethnicity base targets, blended at an
///   equal `1/3` ethnicity share (there is no ethnicity slider) times the
///   gender/age partition.
fn macro_blend_weight_fn(full_name: &str) -> Option<TargetWeightFn> {
    let basename = full_name
        .rsplit('/')
        .next()
        .unwrap_or(full_name)
        .to_lowercase();
    // "female" contains "male", so test the more specific token first.
    let is_female = basename.contains("female");
    let is_male = !is_female && basename.contains("male");
    let is_young = basename.contains("young");
    let is_old = basename.contains("old");

    let gender_term = move |p: &ParamState| -> f32 {
        let g = gender_slider(p);
        if is_female {
            g
        } else if is_male {
            1.0 - g
        } else {
            1.0
        }
    };
    let age_term = move |p: &ParamState| -> f32 {
        if is_young {
            1.0 - p.age.clamp(0.0, 1.0)
        } else if is_old {
            p.age.clamp(0.0, 1.0)
        } else {
            1.0
        }
    };

    // Height corner targets (…/height/… or a `min|maxheight` suffix).
    //
    // In this core pack *both* the `minheight` and `maxheight` corner deltas
    // increase stature relative to the base mesh (the base is already the
    // shortest configuration — there is no genuine stature-reducing corner in
    // the core tier). A naive partition-of-unity blend is therefore U-shaped
    // (both slider ends taller than the centre), which is unusable for a
    // height slider. We instead drive only the `maxheight` corner with
    // `w_max(height)`, so stature is monotonic non-decreasing: the base body
    // (~170 cm) for `height <= 0.5`, ramping up to the tall corner at
    // `height = 1.0`. `minheight` is left inert (weight 0). Modelling
    // below-base stature would require a dedicated shortening target the core
    // pack does not ship.
    if basename.contains("height") {
        let level = detect_level(&basename, "height")?;
        return Some(Box::new(move |p: &ParamState| match level {
            MacroLevel::Max => gender_term(p) * macro_level_weight(MacroLevel::Max, p.height),
            MacroLevel::Min | MacroLevel::Avg => 0.0,
        }));
    }

    // Universal body corner targets.
    if basename.contains("universal") {
        let muscle = detect_level(&basename, "muscle");
        let weight = detect_level(&basename, "weight");
        if let (Some(ml), Some(wl)) = (muscle, weight) {
            return Some(Box::new(move |p: &ParamState| {
                gender_term(p)
                    * age_term(p)
                    * macro_level_weight(ml, p.muscle)
                    * macro_level_weight(wl, p.weight)
            }));
        }
    }

    // Ethnicity base targets: `{ethnicity}-{gender}-{young}` with no
    // muscle/weight tokens. Equal 1/3 ethnicity share (no ethnicity slider).
    let is_ethnicity = ["african", "asian", "caucasian"]
        .iter()
        .any(|e| basename.contains(e));
    if is_ethnicity && (is_male || is_female) {
        const ETHNICITY_SHARE: f32 = 1.0 / 3.0;
        return Some(Box::new(move |p: &ParamState| {
            ETHNICITY_SHARE * gender_term(p) * age_term(p)
        }));
    }

    None
}

/// Choose a weight function for a core-pack target.
///
/// Preference order:
/// 1. A MakeHuman macro-modifier blend when the target name matches the
///    macrodetails corner-target naming ([`macro_blend_weight_fn`]) — this
///    composes the corner targets as a partition of unity instead of summing
///    them, so `set_param("height"|"weight"|"muscle"|"age"|"gender")` drives a
///    realistically-scaled body.
/// 2. The pack-declared `category` when it names a known
///    [`oxihuman_core::category::TargetCategory`] (so `set_param("height")`
///    etc. drives the target).
/// 3. When the declared category is unknown, a macro category
///    (height/weight/muscle/age **only**) inferred from the target name
///    (`weight_curves::infer_category_from_name`).
/// 4. Fallback: an extra param keyed by the *target name*, defaulting to
///    `0.0` so unknown targets stay inert until explicitly driven via
///    `set_param(name, w)`.
fn pack_weight_fn(name: &str, category: &str) -> TargetWeightFn {
    use oxihuman_core::category::TargetCategory;
    use oxihuman_morph::weight_curves::{auto_weight_fn, infer_category_from_name};

    // MakeHuman macro corner targets: compose via partition of unity.
    if let Some(wf) = macro_blend_weight_fn(name) {
        return wf;
    }

    let cat = match TargetCategory::from_str(category) {
        // Unknown declared category: trust only a *macro* inference from the
        // name (the generic fallback of `infer_category_from_name` is
        // BodyShapes, which would silently activate the target at default
        // params — packs with unknown categories must stay inert instead).
        TargetCategory::Other(_) => match infer_category_from_name(name) {
            c @ (TargetCategory::Height
            | TargetCategory::Weight
            | TargetCategory::Muscle
            | TargetCategory::Age) => c,
            _ => TargetCategory::Other(name.to_string()),
        },
        known => known,
    };
    match cat {
        TargetCategory::Other(_) => {
            let key = name.to_string();
            Box::new(move |p: &ParamState| p.extra.get(&key).copied().unwrap_or(0.0))
        }
        known => auto_weight_fn(known.as_str()),
    }
}

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
    // -- Physics --
    pub(crate) wind_config: Option<WindConfig>,
    pub(crate) cloth_sim: Option<ClothSim>,
    pub(crate) body_proxies: Option<BodyProxies>,
    /// Accumulated simulation time (seconds), used for wind field sampling.
    pub(crate) sim_time: f32,
    // -- Persistent zero-copy geometry buffers (M4) --
    /// Flat XYZ positions (`3 * n_verts` floats), kept at a stable address
    /// between rebuilds so JS can hold a `Float32Array` view over them.
    pub(crate) geo_positions: Vec<f32>,
    /// Flat XYZ normals (`3 * n_verts` floats).
    pub(crate) geo_normals: Vec<f32>,
    /// Flat UV coordinates (`2 * n_verts` floats).
    pub(crate) geo_uvs: Vec<f32>,
    /// Triangle index list.
    pub(crate) geo_indices: Vec<u32>,
    /// Bumped whenever the persistent buffers are (re)allocated (topology or
    /// vertex-count change) — JS must re-create its typed-array views then.
    pub(crate) geo_generation: u32,
    /// True when params changed since the last `refresh_geometry()`.
    pub(crate) geo_dirty: bool,
    // -- Safety --
    /// Minimum modelled age in years (from a core-pack manifest).  When set,
    /// the `age` parameter is clamped so the modelled age never goes below
    /// this floor (see [`age_years_to_param`]).
    pub(crate) age_floor_years: Option<f32>,
    // -- Measurement acceleration --
    /// Cached positions-independent measurer topology (body boundary +
    /// triangle list). Morphs never change topology, so a fit's dozens of
    /// re-measurements share one derivation; invalidated whenever the base
    /// mesh is replaced (see [`Self::invalidate_geometry_buffers`]).
    pub(crate) measurer_topology: Option<oxihuman_morph::measurements::MeasurerTopology>,
}

impl WasmEngine {
    /// Shared constructor from a parsed base mesh and policy.
    fn from_base(base: ObjMesh, policy: Policy) -> Self {
        WasmEngine {
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
            wind_config: None,
            cloth_sim: None,
            body_proxies: None,
            sim_time: 0.0,
            geo_positions: Vec::new(),
            geo_normals: Vec::new(),
            geo_uvs: Vec::new(),
            geo_indices: Vec::new(),
            geo_generation: 0,
            geo_dirty: true,
            age_floor_years: None,
            measurer_topology: None,
        }
    }

    /// Create a new engine with a built-in minimal stub mesh (one triangle).
    ///
    /// Infallible by construction: the base mesh is built as an [`ObjMesh`]
    /// literal, no parsing involved. Replace it with a real base mesh via
    /// [`Self::load_core_pack_bytes`] or [`Self::load_zip_pack_bytes`].
    pub fn new_stub() -> Self {
        let base = ObjMesh {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
        };
        Self::from_base(base, Policy::new(PolicyProfile::Standard))
    }

    /// Create a new engine from raw OBJ bytes (UTF-8 text).
    pub fn new_from_obj_bytes(obj_bytes: &[u8]) -> Result<Self> {
        let src = std::str::from_utf8(obj_bytes)?;
        let base = parse_obj(src)?;
        Ok(Self::from_base(base, Policy::new(PolicyProfile::Standard)))
    }

    /// Create with a strict policy (only allowlisted targets accepted).
    pub fn new_strict(obj_bytes: &[u8]) -> Result<Self> {
        let src = std::str::from_utf8(obj_bytes)?;
        let base = parse_obj(src)?;
        Ok(Self::from_base(base, Policy::new(PolicyProfile::Strict)))
    }

    /// Create a new engine from OHPK core-pack bytes (see
    /// [`Self::load_core_pack_bytes`]).
    pub fn new_from_core_pack_bytes(pack_bytes: &[u8]) -> Result<Self> {
        let mut engine = Self::new_stub();
        engine.load_core_pack_bytes(pack_bytes)?;
        Ok(engine)
    }

    /// Load an OHPK v1 core pack from bytes, replacing the base mesh and the
    /// whole morph-target store.
    ///
    /// - The pack's base positions/indices/UVs become the new base mesh
    ///   (normals are recomputed on every build, so placeholders are fine).
    /// - Every pack target is loaded into the *engine* target store with a
    ///   weight function derived from its category, so
    ///   `set_param("height"|"weight"|"muscle"|"age", v)` drives it (see
    ///   `pack_weight_fn`). Unknown categories are driven by an extra param
    ///   keyed by the target name (inert until set).
    /// - `manifest.age_floor_years` is honoured: the `age` param is clamped
    ///   so the modelled age never drops below the floor.
    ///
    /// Returns the number of targets loaded.
    pub fn load_core_pack_bytes(&mut self, pack_bytes: &[u8]) -> Result<usize> {
        use oxihuman_core::parser::target::{Delta, TargetFile};
        use oxihuman_export::CorePack;

        let pack = CorePack::parse(pack_bytes)?;

        let n_verts = pack.vertex_count();
        let flat = pack.base_positions();
        anyhow::ensure!(
            flat.len() == n_verts * 3,
            "core pack position buffer length {} != 3 * n_verts ({})",
            flat.len(),
            n_verts * 3
        );
        let positions: Vec<[f32; 3]> = flat.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
        let uvs: Vec<[f32; 2]> = match pack.base_uvs() {
            Some(flat_uv) => {
                anyhow::ensure!(
                    flat_uv.len() == n_verts * 2,
                    "core pack UV buffer length {} != 2 * n_verts ({})",
                    flat_uv.len(),
                    n_verts * 2
                );
                flat_uv.chunks_exact(2).map(|c| [c[0], c[1]]).collect()
            }
            None => vec![[0.0, 0.0]; n_verts],
        };
        let base = ObjMesh {
            positions,
            // Placeholder normals: `build_mesh_prepared` / `refresh_geometry`
            // recompute real normals from the morphed positions every build.
            normals: vec![[0.0, 0.0, 1.0]; n_verts],
            uvs,
            indices: pack.base_indices().to_vec(),
        };

        self.engine = HumanEngine::new(base, Policy::new(PolicyProfile::Standard));
        self.params = ParamState::default();
        self.target_names.clear();
        self.json_targets.clear();
        self.last_mesh = None;
        self.body_proxies = None;
        self.cloth_sim = None;
        self.invalidate_geometry_buffers();

        // Safety floor from the manifest, applied before any param commit.
        self.age_floor_years = pack.manifest().age_floor_years;

        let mut loaded = 0usize;
        for target in pack.targets() {
            let deltas: Vec<Delta> = target
                .sparse()
                .into_iter()
                .map(|(vid, d)| Delta {
                    vid,
                    dx: d[0],
                    dy: d[1],
                    dz: d[2],
                })
                .collect();
            let tf = TargetFile {
                name: target.name().to_string(),
                deltas,
            };
            let weight_fn = pack_weight_fn(target.name(), target.category());
            let before = self.engine.target_count();
            self.engine.load_target(tf, weight_fn);
            if self.engine.target_count() > before {
                self.target_names.push(target.name().to_string());
                loaded += 1;
            }
        }

        // Re-commit params so the age floor takes effect immediately.
        let params = self.params.clone();
        self.commit_params(params);
        Ok(loaded)
    }

    /// The minimum modelled age in years, if a core pack declared one.
    pub fn age_floor_years(&self) -> Option<f32> {
        self.age_floor_years
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
        self.geo_dirty = true;
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
        self.invalidate_geometry_buffers();

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

        let mesh = self.build_mesh_prepared();
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

    /// Commit a new parameter state: enforce the age floor, push the params
    /// into the morph engine, and invalidate all cached geometry.
    ///
    /// Every param-mutating path (`set_param`, presets, JSON import,
    /// animation seeking, resets) routes through this single method so the
    /// age floor and geometry invalidation can never be bypassed.
    pub(crate) fn commit_params(&mut self, mut p: ParamState) {
        // D3: the core-pack manifest's `age_floor_years` is only the *trigger* —
        // its presence declares that this pack must enforce the adult age floor.
        // The enforced *value* is the single authoritative policy constant
        // [`oxihuman_core::policy::AGE_ADULT_FLOOR_YR`], not the number embedded
        // in the manifest, so the floor can never silently drift below policy by
        // shipping a weaker manifest. A pack may declare a *stricter* (higher)
        // floor than policy but never a weaker one, so we clamp the modelled age
        // to the stricter of the two.
        if let Some(manifest_floor) = self.age_floor_years {
            let floor_years = manifest_floor.max(oxihuman_core::policy::AGE_ADULT_FLOOR_YR);
            let min_age = age_years_to_param(floor_years);
            if p.age < min_age {
                p.age = min_age;
            }
        }
        self.engine.set_params(p.clone());
        self.params = p;
        self.last_mesh = None;
        self.geo_dirty = true;
    }

    pub(crate) fn _update_param<F: FnOnce(&mut ParamState)>(&mut self, f: F) {
        let mut p = self.params.clone();
        f(&mut p);
        self.commit_params(p);
    }

    /// Reset all parameters to their default (mid-point) values and invalidate the mesh cache.
    pub fn reset_params(&mut self) {
        self.commit_params(ParamState::default());
    }

    /// Return how many morph targets are currently loaded.
    pub fn target_count(&self) -> usize {
        self.engine.target_count()
    }

    /// Scatter-add all JSON-loaded morph targets (weighted) into a position
    /// buffer. This makes `load_target_from_json` / `set_target_weight_by_name`
    /// actually affect every rendered/exported mesh — they were previously
    /// stored but never applied.
    pub(crate) fn apply_json_targets(&self, positions: &mut [[f32; 3]]) {
        for (deltas, weight) in self.json_targets.values() {
            let w = *weight;
            if w == 0.0 {
                continue;
            }
            for &(vid, dx, dy, dz) in deltas {
                if let Some(p) = positions.get_mut(vid as usize) {
                    p[0] += dx * w;
                    p[1] += dy * w;
                    p[2] += dz * w;
                }
            }
        }
    }

    /// Build the morphed mesh and return raw bytes.
    ///
    /// Uses the engine's incremental build path (only re-applies targets
    /// whose weight changed since the previous build) and applies
    /// JSON-loaded targets on top.
    ///
    /// Format: `[format_version: u32][n_verts: u32][n_idx: u32]`
    ///          `[positions: f32 * 3 * n_verts][normals: f32 * 3 * n_verts]`
    ///          `[uvs: f32 * 2 * n_verts][indices: u32 * n_idx]`
    pub fn build_mesh_bytes(&mut self) -> Vec<u8> {
        let mesh = self.build_mesh_prepared();

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
        self.geo_dirty = true;
    }

    /// Returns true if a mesh has been built since the last param change.
    pub fn has_cached_mesh(&self) -> bool {
        self.last_mesh.is_some()
    }

    /// Build the morphed mesh and return a fully-prepared [`MeshBuffers`]
    /// (JSON targets applied, normals computed, suit flag applied).
    ///
    /// This is the single central build path: every WASM-facing method that
    /// needs a mesh (`build_mesh_bytes`, exports, measurements, physics,
    /// LOD, curvature) routes through it, so JSON-loaded targets and the
    /// incremental engine path apply uniformly.
    ///
    /// # `has_suit` truthfulness
    ///
    /// The engine's base mesh is the bodysuit-topology base mesh (the only
    /// base geometry this crate loads), so the suit flag is applied here —
    /// the flag is a statement about the base topology, not a per-build
    /// computation.
    pub fn build_mesh_prepared(&mut self) -> MeshBuffers {
        let morph_buf = self.engine.build_mesh_incremental();
        let mut mesh = MeshBuffers::from_morph(morph_buf);
        self.apply_json_targets(&mut mesh.positions);
        compute_normals(&mut mesh);
        apply_suit_flag(&mut mesh);
        self.last_mesh = Some(mesh.clone());
        mesh
    }

    // -- Persistent zero-copy geometry (M4) -----------------------------------

    /// Drop the persistent geometry buffers and bump the generation counter.
    ///
    /// Called whenever the base mesh (topology) is replaced; JS must
    /// re-create its typed-array views afterwards.
    pub(crate) fn invalidate_geometry_buffers(&mut self) {
        self.geo_positions = Vec::new();
        self.geo_normals = Vec::new();
        self.geo_uvs = Vec::new();
        self.geo_indices = Vec::new();
        self.geo_generation = self.geo_generation.wrapping_add(1);
        self.geo_dirty = true;
        // Base mesh (topology) changed: the cached measurer topology no
        // longer describes the mesh.
        self.measurer_topology = None;
    }

    /// Recompute the persistent geometry buffers via the engine's
    /// incremental build path, writing positions/normals **in place** (the
    /// buffers keep their address unless the vertex count changed).
    ///
    /// Returns the current mesh generation. The generation is bumped only
    /// when the buffers had to be (re)allocated (first build after a
    /// topology change) — JS re-creates its `Float32Array`/`Uint32Array`
    /// views when the returned generation differs from the one it captured,
    /// and additionally after WebAssembly memory growth (see the crate
    /// README / TypeScript docs).
    ///
    /// No-op when the geometry is not dirty.
    pub fn refresh_geometry(&mut self) -> u32 {
        if !self.geo_dirty && !self.geo_positions.is_empty() {
            return self.geo_generation;
        }

        // Incremental engine rebuild (only re-applies changed-weight targets).
        let morph_buf = self.engine.build_mesh_incremental();
        let mut positions = morph_buf.positions;
        self.apply_json_targets(&mut positions);

        let n = positions.len();
        let needs_realloc = self.geo_positions.len() != n * 3;
        if needs_realloc {
            self.geo_positions = vec![0.0; n * 3];
            self.geo_normals = vec![0.0; n * 3];
            self.geo_uvs = Vec::with_capacity(n * 2);
            for uv in &morph_buf.uvs {
                self.geo_uvs.push(uv[0]);
                self.geo_uvs.push(uv[1]);
            }
            self.geo_uvs.resize(n * 2, 0.0);
            self.geo_indices = morph_buf.indices.clone();
            self.geo_generation = self.geo_generation.wrapping_add(1);
        }

        // Positions: flatten in place.
        for (i, p) in positions.iter().enumerate() {
            self.geo_positions[i * 3] = p[0];
            self.geo_positions[i * 3 + 1] = p[1];
            self.geo_positions[i * 3 + 2] = p[2];
        }

        // Normals: area-weighted face-normal accumulation, in place
        // (zero-fill + accumulate + normalise; no per-frame allocation).
        for v in self.geo_normals.iter_mut() {
            *v = 0.0;
        }
        for tri in self.geo_indices.chunks_exact(3) {
            let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
            if i0 >= n || i1 >= n || i2 >= n {
                continue;
            }
            let p0 = positions[i0];
            let p1 = positions[i1];
            let p2 = positions[i2];
            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let fnx = e1[1] * e2[2] - e1[2] * e2[1];
            let fny = e1[2] * e2[0] - e1[0] * e2[2];
            let fnz = e1[0] * e2[1] - e1[1] * e2[0];
            for &vi in &[i0, i1, i2] {
                self.geo_normals[vi * 3] += fnx;
                self.geo_normals[vi * 3 + 1] += fny;
                self.geo_normals[vi * 3 + 2] += fnz;
            }
        }
        for chunk in self.geo_normals.chunks_exact_mut(3) {
            let len = (chunk[0] * chunk[0] + chunk[1] * chunk[1] + chunk[2] * chunk[2]).sqrt();
            if len > 1e-10 {
                chunk[0] /= len;
                chunk[1] /= len;
                chunk[2] /= len;
            } else {
                chunk[0] = 0.0;
                chunk[1] = 1.0;
                chunk[2] = 0.0;
            }
        }

        self.geo_dirty = false;
        self.geo_generation
    }

    /// Byte offset of the flat positions buffer inside WASM linear memory
    /// (`f32` array of [`Self::positions_len`] elements).
    pub fn positions_ptr(&self) -> u32 {
        self.geo_positions.as_ptr() as usize as u32
    }

    /// Number of `f32` elements in the positions buffer (`3 * n_verts`).
    pub fn positions_len(&self) -> u32 {
        self.geo_positions.len() as u32
    }

    /// Byte offset of the flat normals buffer inside WASM linear memory.
    pub fn normals_ptr(&self) -> u32 {
        self.geo_normals.as_ptr() as usize as u32
    }

    /// Number of `f32` elements in the normals buffer (`3 * n_verts`).
    pub fn normals_len(&self) -> u32 {
        self.geo_normals.len() as u32
    }

    /// Byte offset of the flat UV buffer inside WASM linear memory.
    pub fn uvs_ptr(&self) -> u32 {
        self.geo_uvs.as_ptr() as usize as u32
    }

    /// Number of `f32` elements in the UV buffer (`2 * n_verts`).
    pub fn uvs_len(&self) -> u32 {
        self.geo_uvs.len() as u32
    }

    /// Byte offset of the triangle index buffer inside WASM linear memory
    /// (`u32` array of [`Self::indices_len`] elements).
    pub fn indices_ptr(&self) -> u32 {
        self.geo_indices.as_ptr() as usize as u32
    }

    /// Number of `u32` elements in the index buffer.
    pub fn indices_len(&self) -> u32 {
        self.geo_indices.len() as u32
    }

    /// Current mesh generation (bumps when the persistent buffers move).
    pub fn mesh_generation(&self) -> u32 {
        self.geo_generation
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
        let mut p = self.params.clone();
        // Reset extra params (which drive engine target weights)
        for v in p.extra.values_mut() {
            *v = 0.0;
        }
        p.height = 0.5;
        p.weight = 0.5;
        p.muscle = 0.5;
        p.age = 0.5;
        // Reset JSON target weights
        for entry in self.json_targets.values_mut() {
            entry.1 = 0.0;
        }
        self.commit_params(p);
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

    /// Advance the physics simulation by `dt` seconds.
    ///
    /// - Clamps `dt` to at most 1/30 s to prevent large instability.
    /// - Steps the cloth simulation (gravity is built into [`ClothSim::step`]).
    /// - Applies the current wind field to cloth particles.
    /// - Lazily generates body proxies from `last_mesh` when not yet available.
    pub fn step_physics(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 1.0 / 30.0);
        self.sim_time += dt;

        // Lazily build body proxies from the last mesh.
        if self.body_proxies.is_none() {
            if let Some(ref mesh) = self.last_mesh {
                self.body_proxies = oxihuman_physics::generate_proxies(mesh);
            }
        }

        if let Some(ref mut sim) = self.cloth_sim {
            // Apply wind forces before the Verlet step.
            if let Some(ref cfg) = self.wind_config {
                let wind_field = WindField::new(cfg.clone());
                oxihuman_physics::apply_wind_to_cloth(sim, &wind_field, self.sim_time, dt);
            }
            // Verlet integration with gravity already in ClothSim.
            sim.step(dt, 4);
        }
    }

    /// Return current cloth simulation state as JSON.
    ///
    /// Format: `{"cloth_positions":[[x,y,z], ...]}`.
    /// Returns an empty array when no cloth is initialised.
    pub fn get_cloth_state(&self) -> String {
        match self.cloth_sim {
            None => r#"{"cloth_positions":[]}"#.to_string(),
            Some(ref sim) => {
                let positions = sim.positions();
                let mut out = String::with_capacity(positions.len() * 30 + 32);
                out.push_str("{\"cloth_positions\":[");
                for (i, p) in positions.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push_str(&format!("[{},{},{}]", p[0], p[1], p[2]));
                }
                out.push_str("]}");
                out
            }
        }
    }

    /// Return physics proxy data as JSON.
    ///
    /// Wraps [`oxihuman_physics::proxies_to_json`] output under a `"proxies"` key
    /// to preserve backward compatibility with callers expecting that key.
    /// Falls back to `{"proxies":[]}` when no proxies are available.
    pub fn get_physics_proxy_json(&self) -> String {
        match self.body_proxies {
            None => r#"{"proxies":[]}"#.to_string(),
            Some(ref proxies) => {
                let inner = oxihuman_physics::proxies_to_json(proxies);
                format!("{{\"proxies\":{inner}}}")
            }
        }
    }

    /// Set the wind vector for physics simulation.
    ///
    /// Normalises the vector internally and stores a [`WindConfig`].
    /// Passing a zero vector disables wind.
    pub fn set_wind(&mut self, x: f32, y: f32, z: f32) {
        let speed = (x * x + y * y + z * z).sqrt();
        if speed < 1e-6 {
            self.wind_config = None;
            return;
        }
        self.wind_config = Some(WindConfig {
            base_direction: [x / speed, y / speed, z / speed],
            base_speed: speed,
            turbulence: 0.3,
            gust_frequency: 0.5,
            vortex_strength: 0.2,
            seed: 42,
        });
    }

    /// Initialise a cloth simulation from the last built mesh.
    ///
    /// Does nothing when no mesh has been built yet.
    /// `stiffness` is forwarded directly to all springs (0 = limp, 1 = rigid).
    pub fn init_cloth(&mut self, stiffness: f32) {
        if let Some(ref mesh) = self.last_mesh {
            let positions: Vec<[f32; 3]> = mesh.positions.clone();
            let indices: Vec<u32> = mesh.indices.clone();
            self.cloth_sim = Some(ClothSim::from_mesh(&positions, &indices, stiffness));
        }
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

// ── Physics unit tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_OBJ: &[u8] = b"\
v 0.0 0.0 0.0\n\
v 1.0 0.0 0.0\n\
v 0.0 1.0 0.0\n\
vt 0.0 0.0\n\
vt 1.0 0.0\n\
vt 0.0 1.0\n\
vn 0.0 0.0 1.0\n\
f 1/1/1 2/2/1 3/3/1\n";

    #[test]
    fn test_set_wind_stores_config() {
        let mut engine = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
        engine.set_wind(1.0, 0.0, 0.0);
        assert!(engine.wind_config.is_some());
    }

    #[test]
    fn test_set_wind_zero_clears() {
        let mut engine = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
        engine.set_wind(1.0, 0.0, 0.0);
        engine.set_wind(0.0, 0.0, 0.0);
        assert!(engine.wind_config.is_none());
    }

    #[test]
    fn test_set_wind_normalises_direction() {
        let mut engine = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
        engine.set_wind(3.0, 0.0, 4.0);
        let cfg = engine
            .wind_config
            .as_ref()
            .expect("wind_config must be Some");
        // Direction should be normalised: magnitude == 1
        let mag = (cfg.base_direction[0].powi(2)
            + cfg.base_direction[1].powi(2)
            + cfg.base_direction[2].powi(2))
        .sqrt();
        assert!(
            (mag - 1.0).abs() < 1e-5,
            "direction magnitude should be 1, got {mag}"
        );
        // Base speed should be the original magnitude: sqrt(9+16) = 5
        assert!(
            (cfg.base_speed - 5.0).abs() < 1e-5,
            "base_speed should be 5, got {}",
            cfg.base_speed
        );
    }

    #[test]
    fn test_step_physics_no_cloth_no_op() {
        let mut engine = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
        // Must not panic
        engine.step_physics(1.0 / 60.0);
    }

    #[test]
    fn test_step_physics_advances_sim_time() {
        let mut engine = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
        engine.step_physics(0.1);
        assert!(engine.sim_time > 0.0, "sim_time should advance");
    }

    #[test]
    fn test_cloth_state_empty_before_init() {
        let engine = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
        assert!(engine.get_cloth_state().contains("cloth_positions"));
    }

    #[test]
    fn test_cloth_state_empty_json_before_init() {
        let engine = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
        let v: serde_json::Value =
            serde_json::from_str(&engine.get_cloth_state()).expect("must be valid JSON");
        let arr = v["cloth_positions"].as_array().expect("must be array");
        assert!(arr.is_empty(), "no cloth sim yet, array should be empty");
    }

    #[test]
    fn test_init_cloth_requires_built_mesh() {
        // init_cloth silently does nothing when last_mesh is None.
        let mut engine = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
        engine.init_cloth(0.5);
        assert!(
            engine.cloth_sim.is_none(),
            "cloth_sim should remain None without a built mesh"
        );
    }

    #[test]
    fn test_init_cloth_then_step() {
        let mut engine = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
        // Build a mesh first so init_cloth has something to work from.
        let _ = engine.build_mesh_bytes();
        engine.init_cloth(0.5);
        assert!(
            engine.cloth_sim.is_some(),
            "cloth_sim should be Some after init_cloth"
        );

        // Step 5 frames — must not panic.
        for _ in 0..5 {
            engine.step_physics(1.0 / 60.0);
        }

        // Cloth state should now have positions.
        let v: serde_json::Value =
            serde_json::from_str(&engine.get_cloth_state()).expect("must be valid JSON");
        let arr = v["cloth_positions"].as_array().expect("must be array");
        assert!(
            !arr.is_empty(),
            "cloth_positions should be non-empty after init"
        );
    }

    #[test]
    fn test_init_cloth_with_wind_no_panic() {
        let mut engine = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
        let _ = engine.build_mesh_bytes();
        engine.init_cloth(0.8);
        engine.set_wind(1.0, 0.0, 0.5);
        for _ in 0..10 {
            engine.step_physics(1.0 / 60.0);
        }
        // All particle positions must be finite.
        if let Some(ref sim) = engine.cloth_sim {
            for p in sim.positions() {
                assert!(
                    p[0].is_finite() && p[1].is_finite() && p[2].is_finite(),
                    "particle position is not finite: {p:?}"
                );
            }
        }
    }

    #[test]
    fn test_step_physics_dt_clamp() {
        let mut engine = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
        let _ = engine.build_mesh_bytes();
        engine.init_cloth(0.5);
        // Very large dt should be clamped and not cause a NaN cascade.
        engine.step_physics(100.0);
        if let Some(ref sim) = engine.cloth_sim {
            for p in sim.positions() {
                assert!(
                    p[0].is_finite() && p[1].is_finite() && p[2].is_finite(),
                    "particle position is not finite after large dt: {p:?}"
                );
            }
        }
    }
}
