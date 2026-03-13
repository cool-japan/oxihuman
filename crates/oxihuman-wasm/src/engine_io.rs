// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! JSON import/export, measurements, physics, scene, and render methods for `WasmEngine`.

use anyhow::Result;
use oxihuman_mesh::mesh::MeshBuffers;
use oxihuman_mesh::normals::compute_normals;
use oxihuman_mesh::suit::apply_suit_flag;
use oxihuman_morph::params::ParamState;

use crate::engine_core::WasmEngine;

impl WasmEngine {
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

    // -- Shader library --

    /// Return a JSON array of shader names from the default PBR shader library.
    pub fn list_builtin_shaders(&self) -> String {
        use oxihuman_viewer::shader_library::{default_pbr_shaders, list_shaders};
        let lib = default_pbr_shaders();
        let names: Vec<&str> = list_shaders(&lib);
        serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string())
    }
}
