// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! End-to-end pipeline: base mesh + targets + params → GLB file.

use anyhow::{Context, Result};

use oxihuman_core::parser::obj::parse_obj;
use oxihuman_core::parser::target::parse_target;
use oxihuman_core::policy::{Policy, PolicyProfile};
use oxihuman_mesh::mesh::MeshBuffers;
use oxihuman_mesh::normals::compute_normals;
use oxihuman_mesh::suit::apply_suit_flag;
use oxihuman_morph::engine::HumanEngine;
use oxihuman_morph::params::ParamState;

use crate::glb::export_glb;

/// Configuration for the full build pipeline.
pub struct PipelineConfig {
    /// Path to the base .obj mesh file.
    pub base_obj_path: std::path::PathBuf,
    /// Optional directory of .target files to load.
    pub targets_dir: Option<std::path::PathBuf>,
    /// Maximum number of targets to load (None = all).
    pub max_targets: Option<usize>,
    /// Policy profile for target filtering.
    pub policy: Policy,
    /// Parameters for morphing.
    pub params: ParamState,
    /// Output GLB path.
    pub output_path: std::path::PathBuf,
}

impl PipelineConfig {
    pub fn new(
        base_obj_path: impl Into<std::path::PathBuf>,
        output_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        PipelineConfig {
            base_obj_path: base_obj_path.into(),
            targets_dir: None,
            max_targets: None,
            policy: Policy::new(PolicyProfile::Standard),
            params: ParamState::default(),
            output_path: output_path.into(),
        }
    }
}

/// Run the full OxiHuman pipeline and write a GLB file.
///
/// Steps:
/// 1. Parse base .obj
/// 2. Create HumanEngine
/// 3. Load targets from directory (if configured)
/// 4. Set params
/// 5. Build morphed mesh
/// 6. Compute normals
/// 7. Apply suit flag
/// 8. Export GLB
pub fn run_pipeline(config: PipelineConfig) -> Result<MeshBuffers> {
    // Step 1: Parse base mesh
    let obj_src = std::fs::read_to_string(&config.base_obj_path)
        .with_context(|| format!("reading base OBJ: {}", config.base_obj_path.display()))?;
    let base = parse_obj(&obj_src).with_context(|| "parsing base OBJ")?;

    // Step 2: Create engine
    let mut engine = HumanEngine::new(base, config.policy);

    // Step 3: Load targets
    if let Some(ref targets_dir) = config.targets_dir {
        if targets_dir.exists() {
            let max = config.max_targets.unwrap_or(usize::MAX);
            let mut loaded = 0usize;
            for entry in std::fs::read_dir(targets_dir)
                .with_context(|| format!("reading targets dir: {}", targets_dir.display()))?
            {
                if loaded >= max {
                    break;
                }
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "target").unwrap_or(false) {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    if let Ok(src) = std::fs::read_to_string(&path) {
                        if let Ok(target) = parse_target(&name, &src) {
                            engine.load_target(target, Box::new(|p: &ParamState| p.weight));
                            loaded += 1;
                        }
                    }
                }
            }
        }
    }

    // Step 4: Set params
    engine.set_params(config.params);

    // Step 5: Build morphed mesh
    let morph_buffers = engine.build_mesh();

    // Step 6: Construct MeshBuffers and compute normals
    let mut mesh = MeshBuffers::from_morph(morph_buffers);
    compute_normals(&mut mesh);

    // Step 7: Apply suit flag
    apply_suit_flag(&mut mesh);

    // Step 8: Export GLB
    export_glb(&mesh, &config.output_path)
        .with_context(|| format!("exporting GLB to {}", config.output_path.display()))?;

    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glb::verify_glb_header;
    use oxihuman_morph::params::ParamState;
    use proptest::prelude::*;
    use std::path::Path;

    const BASE_OBJ: &str =
        "/media/kitasan/Backup/resource/makehuman/makehuman/data/3dobjs/base.obj";
    const TARGETS_DIR: &str =
        "/media/kitasan/Backup/resource/makehuman/makehuman/data/targets/bodyshapes";

    #[test]
    fn pipeline_produces_valid_glb() {
        if !Path::new(BASE_OBJ).exists() {
            return; // skip in CI without assets
        }
        let out = std::path::PathBuf::from("/tmp/oxihuman_pipeline_test.glb");
        let config = PipelineConfig {
            base_obj_path: std::path::PathBuf::from(BASE_OBJ),
            targets_dir: Some(std::path::PathBuf::from(TARGETS_DIR)),
            max_targets: Some(5),
            policy: Policy::new(PolicyProfile::Standard),
            params: ParamState::new(0.6, 0.4, 0.5, 0.3),
            output_path: out.clone(),
        };
        let mesh = run_pipeline(config).expect("pipeline failed");
        assert!(
            mesh.positions.len() > 10_000,
            "base mesh should have many vertices"
        );
        assert!(mesh.has_suit, "suit flag must be set");
        verify_glb_header(&out).expect("GLB header invalid");
        std::fs::remove_file(&out).ok();
    }

    #[test]
    fn pipeline_no_targets_still_works() {
        if !Path::new(BASE_OBJ).exists() {
            return;
        }
        let out = std::path::PathBuf::from("/tmp/oxihuman_notargets.glb");
        let config = PipelineConfig::new(BASE_OBJ, out.clone());
        let mesh = run_pipeline(config).expect("pipeline (no targets) failed");
        assert!(!mesh.positions.is_empty());
        verify_glb_header(&out).unwrap();
        std::fs::remove_file(&out).ok();
    }

    #[test]
    fn pipeline_vertex_positions_finite() {
        if !Path::new(BASE_OBJ).exists() {
            return;
        }
        let out = std::path::PathBuf::from("/tmp/oxihuman_finite.glb");
        let config = PipelineConfig {
            base_obj_path: std::path::PathBuf::from(BASE_OBJ),
            targets_dir: Some(std::path::PathBuf::from(TARGETS_DIR)),
            max_targets: Some(10),
            policy: Policy::new(PolicyProfile::Standard),
            params: ParamState::new(1.0, 1.0, 1.0, 1.0),
            output_path: out.clone(),
        };
        let mesh = run_pipeline(config).unwrap();
        for pos in &mesh.positions {
            assert!(pos[0].is_finite(), "non-finite x at {:?}", pos);
            assert!(pos[1].is_finite(), "non-finite y at {:?}", pos);
            assert!(pos[2].is_finite(), "non-finite z at {:?}", pos);
        }
        std::fs::remove_file(&out).ok();
    }

    proptest! {
        #[test]
        fn random_params_pipeline_finite_positions(
            h in 0.0f32..=1.0f32,
            w in 0.0f32..=1.0f32,
            m in 0.0f32..=1.0f32,
            a in 0.0f32..=1.0f32,
        ) {
            let base_obj_path = std::path::PathBuf::from(BASE_OBJ);
            if !base_obj_path.exists() { return Ok(()); }

            // Use a simple in-memory OBJ (not the 19k vertex one — too slow for proptest)
            use oxihuman_core::parser::obj::parse_obj;
            let simple_obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nvn 0 0 1\nf 1/1/1 2/2/1 3/3/1\n";
            let base = parse_obj(simple_obj).unwrap();

            use oxihuman_morph::engine::HumanEngine;
            use oxihuman_core::policy::{Policy, PolicyProfile};
            let mut engine = HumanEngine::new(base, Policy::new(PolicyProfile::Standard));
            engine.set_params(ParamState::new(h, w, m, a));
            let morph_buf = engine.build_mesh();

            use oxihuman_mesh::mesh::MeshBuffers;
            use oxihuman_mesh::normals::compute_normals;
            use oxihuman_mesh::integrity::check_positions_finite;
            let mut mesh = MeshBuffers::from_morph(morph_buf);
            compute_normals(&mut mesh);

            prop_assert!(
                check_positions_finite(&mesh),
                "non-finite positions with h={} w={} m={} a={}",
                h, w, m, a
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ExportPipeline builder
// ─────────────────────────────────────────────────────────────────────────────

use bytemuck::cast_slice;
use serde_json::json;
use std::io::Write;

use crate::blend_shapes::BlendShape;
use crate::material::PbrMaterial;
use crate::metadata::OxiHumanMeta;
use oxihuman_mesh::skeleton::Skeleton;

// GLB magic constants (re-declared to keep this section self-contained)
const EP_GLB_MAGIC: u32 = 0x46546C67; // "glTF"
const EP_GLB_VER: u32 = 2;
const EP_CHUNK_JSON: u32 = 0x4E4F534A; // "JSON"
const EP_CHUNK_BIN: u32 = 0x004E4942; // "BIN\0"

/// Builder for composing multiple export features into a single GLB file.
///
/// Enables material + metadata + skeleton + blend shapes to be written in one
/// call, eliminating the need to invoke multiple specialised export functions.
#[derive(Default)]
pub struct ExportPipeline {
    material: Option<PbrMaterial>,
    meta: Option<OxiHumanMeta>,
    skeleton: Option<Skeleton>,
    blend_shapes: Vec<BlendShape>,
    /// If true, include TANGENT accessor.
    include_tangents: bool,
    /// If true, include COLOR_0 accessor.
    include_colors: bool,
    /// If true, embed a NORMAL accessor.
    #[allow(dead_code)]
    include_normals: bool,
}

impl ExportPipeline {
    /// Create a new empty pipeline.
    pub fn new() -> Self {
        Self::default()
    }

    /// Include a PBR material.
    pub fn with_material(mut self, m: PbrMaterial) -> Self {
        self.material = Some(m);
        self
    }

    /// Embed OxiHuman metadata in `asset.extras`.
    pub fn with_meta(mut self, m: OxiHumanMeta) -> Self {
        self.meta = Some(m);
        self
    }

    /// Include skeleton / skin data.
    pub fn with_skeleton(mut self, s: Skeleton) -> Self {
        self.skeleton = Some(s);
        self
    }

    /// Add morph targets (blend shapes).
    pub fn with_blend_shapes(mut self, shapes: Vec<BlendShape>) -> Self {
        self.blend_shapes = shapes;
        self
    }

    /// Include TANGENT accessor (requires `mesh.tangents` to be non-empty).
    pub fn with_tangents(mut self) -> Self {
        self.include_tangents = true;
        self
    }

    /// Include COLOR_0 accessor (requires `mesh.colors` to be `Some`).
    pub fn with_colors(mut self) -> Self {
        self.include_colors = true;
        self
    }

    /// Returns a human-readable description of the enabled features.
    pub fn describe(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.material.is_some() {
            parts.push("material");
        }
        if self.meta.is_some() {
            parts.push("metadata");
        }
        if self.skeleton.is_some() {
            parts.push("skeleton");
        }
        if !self.blend_shapes.is_empty() {
            parts.push("blend_shapes");
        }
        if self.include_tangents {
            parts.push("tangents");
        }
        if self.include_colors {
            parts.push("colors");
        }
        if self.include_normals {
            parts.push("normals");
        }
        if parts.is_empty() {
            "ExportPipeline[minimal]".to_string()
        } else {
            format!("ExportPipeline[{}]", parts.join(", "))
        }
    }

    /// Execute the pipeline: build and write a GLB to `path`.
    ///
    /// Writes a self-contained GLB 2.0 binary incorporating all configured
    /// features: positions, normals, UVs, indices, optional tangents, optional
    /// colors, optional morph targets, optional material, optional metadata in
    /// `asset.extras`, and optional skin/skeleton.
    pub fn export(self, mesh: &MeshBuffers, path: &std::path::Path) -> anyhow::Result<()> {
        let n_verts = mesh.positions.len();
        let n_idx = mesh.indices.len();

        // ── 1. Build BIN chunk ────────────────────────────────────────────────
        // Layout:
        //   POSITION   : f32*3 x n_verts
        //   NORMAL     : f32*3 x n_verts
        //   TEXCOORD_0 : f32*2 x n_verts
        //   INDEX      : u32   x n_idx
        //   TANGENT    : f32*4 x n_verts  (optional)
        //   COLOR_0    : f32*4 x n_verts  (optional)
        //   morph_i    : f32*3 x n_verts  (one block per blend shape)

        let pos_bytes: &[u8] = cast_slice(&mesh.positions);
        let norm_bytes: &[u8] = cast_slice(&mesh.normals);
        let uv_bytes: &[u8] = cast_slice(&mesh.uvs);
        let idx_bytes: &[u8] = cast_slice(&mesh.indices);

        let pos_offset = 0usize;
        let norm_offset = pos_offset + pos_bytes.len();
        let uv_offset = norm_offset + norm_bytes.len();
        let idx_offset = uv_offset + uv_bytes.len();
        let mut cursor = idx_offset + idx_bytes.len();

        // Tangents (optional)
        let has_tangents = self.include_tangents && !mesh.tangents.is_empty();
        let tangent_offset = cursor;
        if has_tangents {
            let tb: &[u8] = cast_slice(&mesh.tangents);
            cursor += tb.len();
        }

        // Colors (optional)
        let has_colors = self.include_colors && mesh.colors.is_some();
        let color_offset = cursor;
        if has_colors {
            let cols = mesh.colors.as_ref().unwrap();
            let cb: &[u8] = cast_slice(cols.as_slice());
            cursor += cb.len();
        }

        // Blend shape delta blocks
        let mut morph_offsets: Vec<usize> = Vec::with_capacity(self.blend_shapes.len());
        for shape in &self.blend_shapes {
            morph_offsets.push(cursor);
            cursor += shape.position_deltas.len() * std::mem::size_of::<[f32; 3]>();
        }

        // Assemble BIN buffer
        let mut bin_data: Vec<u8> = Vec::with_capacity(cursor);
        bin_data.extend_from_slice(pos_bytes);
        bin_data.extend_from_slice(norm_bytes);
        bin_data.extend_from_slice(uv_bytes);
        bin_data.extend_from_slice(idx_bytes);
        if has_tangents {
            let tb: &[u8] = cast_slice(&mesh.tangents);
            bin_data.extend_from_slice(tb);
        }
        if has_colors {
            let cols = mesh.colors.as_ref().unwrap();
            let cb: &[u8] = cast_slice(cols.as_slice());
            bin_data.extend_from_slice(cb);
        }
        for shape in &self.blend_shapes {
            let db: &[u8] = cast_slice(&shape.position_deltas);
            bin_data.extend_from_slice(db);
        }

        // Pad BIN to 4-byte boundary
        while !bin_data.len().is_multiple_of(4) {
            bin_data.push(0x00);
        }
        let total_bin = bin_data.len() as u32;

        // ── 2. Build GLTF accessors and buffer views ───────────────────────────
        let mut accessors: Vec<serde_json::Value> = Vec::new();
        let mut buffer_views: Vec<serde_json::Value> = Vec::new();

        // POSITION
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": pos_offset,
            "byteLength": pos_bytes.len()
        }));
        let pos_acc = accessors.len();
        accessors.push(json!({
            "bufferView": buffer_views.len() - 1,
            "componentType": 5126,
            "count": n_verts,
            "type": "VEC3"
        }));

        // NORMAL
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": norm_offset,
            "byteLength": norm_bytes.len()
        }));
        let norm_acc = accessors.len();
        accessors.push(json!({
            "bufferView": buffer_views.len() - 1,
            "componentType": 5126,
            "count": n_verts,
            "type": "VEC3"
        }));

        // TEXCOORD_0
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": uv_offset,
            "byteLength": uv_bytes.len()
        }));
        let uv_acc = accessors.len();
        accessors.push(json!({
            "bufferView": buffer_views.len() - 1,
            "componentType": 5126,
            "count": n_verts,
            "type": "VEC2"
        }));

        // INDEX
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": idx_offset,
            "byteLength": idx_bytes.len()
        }));
        let idx_acc = accessors.len();
        accessors.push(json!({
            "bufferView": buffer_views.len() - 1,
            "componentType": 5125,
            "count": n_idx,
            "type": "SCALAR"
        }));

        // Optional TANGENT
        let tangent_acc: Option<usize> = if has_tangents {
            let tb: &[u8] = cast_slice(&mesh.tangents);
            buffer_views.push(json!({
                "buffer": 0,
                "byteOffset": tangent_offset,
                "byteLength": tb.len()
            }));
            let acc = accessors.len();
            accessors.push(json!({
                "bufferView": buffer_views.len() - 1,
                "componentType": 5126,
                "count": n_verts,
                "type": "VEC4"
            }));
            Some(acc)
        } else {
            None
        };

        // Optional COLOR_0
        let color_acc: Option<usize> = if has_colors {
            let cols = mesh.colors.as_ref().unwrap();
            let cb: &[u8] = cast_slice(cols.as_slice());
            buffer_views.push(json!({
                "buffer": 0,
                "byteOffset": color_offset,
                "byteLength": cb.len()
            }));
            let acc = accessors.len();
            accessors.push(json!({
                "bufferView": buffer_views.len() - 1,
                "componentType": 5126,
                "count": n_verts,
                "type": "VEC4"
            }));
            Some(acc)
        } else {
            None
        };

        // Morph target delta accessors
        let mut morph_targets: Vec<serde_json::Value> = Vec::new();
        for (i, shape) in self.blend_shapes.iter().enumerate() {
            let delta_len = shape.position_deltas.len() * std::mem::size_of::<[f32; 3]>();
            buffer_views.push(json!({
                "buffer": 0,
                "byteOffset": morph_offsets[i],
                "byteLength": delta_len
            }));
            let acc = accessors.len();
            let mut acc_node = json!({
                "bufferView": buffer_views.len() - 1,
                "componentType": 5126,
                "count": n_verts,
                "type": "VEC3"
            });
            if !shape.position_deltas.is_empty() {
                let mut mn = [f32::INFINITY; 3];
                let mut mx = [f32::NEG_INFINITY; 3];
                for d in &shape.position_deltas {
                    for k in 0..3 {
                        mn[k] = mn[k].min(d[k]);
                        mx[k] = mx[k].max(d[k]);
                    }
                }
                acc_node["min"] = json!([mn[0], mn[1], mn[2]]);
                acc_node["max"] = json!([mx[0], mx[1], mx[2]]);
            }
            accessors.push(acc_node);
            morph_targets.push(json!({ "POSITION": acc }));
        }

        // ── 3. Build primitive and mesh node ──────────────────────────────────
        let mut attributes = json!({
            "POSITION": pos_acc,
            "NORMAL": norm_acc,
            "TEXCOORD_0": uv_acc
        });
        if let Some(t) = tangent_acc {
            attributes["TANGENT"] = json!(t);
        }
        if let Some(c) = color_acc {
            attributes["COLOR_0"] = json!(c);
        }

        let mut primitive = json!({
            "attributes": attributes,
            "indices": idx_acc
        });
        if !morph_targets.is_empty() {
            primitive["targets"] = json!(morph_targets);
        }

        let morph_weights: Vec<f32> = vec![0.0; self.blend_shapes.len()];
        let target_names: Vec<&str> = self.blend_shapes.iter().map(|s| s.name.as_str()).collect();

        let mut mesh_node = json!({
            "primitives": [primitive]
        });
        if !morph_weights.is_empty() {
            mesh_node["weights"] = json!(morph_weights);
            mesh_node["extras"] = json!({ "targetNames": target_names });
        }

        // ── 4. Material ────────────────────────────────────────────────────────
        let mut materials_arr: Vec<serde_json::Value> = Vec::new();
        if let Some(ref mat) = self.material {
            materials_arr.push(mat.to_gltf_json());
            mesh_node["primitives"][0]["material"] = json!(0);
        }

        // ── 5. Skeleton / skin ─────────────────────────────────────────────────
        let skins_val: Option<serde_json::Value>;
        let mut nodes_arr: Vec<serde_json::Value> = Vec::new();

        if let Some(ref skel) = self.skeleton {
            let joint_node_start = 1usize; // node 0 = mesh node
            for joint in &skel.joints {
                nodes_arr.push(json!({
                    "name": joint.name,
                    "translation": [
                        joint.translation[0],
                        joint.translation[1],
                        joint.translation[2]
                    ]
                }));
            }
            let joint_indices: Vec<usize> =
                (joint_node_start..joint_node_start + skel.joints.len()).collect();
            nodes_arr.insert(0, json!({ "mesh": 0, "skin": 0 }));
            skins_val = Some(json!([{
                "joints": joint_indices,
                "name": "Armature"
            }]));
        } else {
            nodes_arr.push(json!({ "mesh": 0 }));
            skins_val = None;
        }

        // ── 6. Asset node (with optional extras) ──────────────────────────────
        let mut asset = json!({
            "version": "2.0",
            "generator": "oxihuman-export"
        });
        if let Some(ref m) = self.meta {
            asset["extras"] = m.to_json();
        }

        // ── 7. Assemble GLTF JSON ─────────────────────────────────────────────
        let mut gltf = json!({
            "asset": asset,
            "scene": 0,
            "scenes": [{ "nodes": [0] }],
            "nodes": nodes_arr,
            "meshes": [mesh_node],
            "accessors": accessors,
            "bufferViews": buffer_views,
            "buffers": [{ "byteLength": total_bin }]
        });
        if !materials_arr.is_empty() {
            gltf["materials"] = json!(materials_arr);
        }
        if let Some(s) = skins_val {
            gltf["skins"] = s;
        }

        // ── 8. Write GLB binary ───────────────────────────────────────────────
        let mut json_bytes = serde_json::to_vec(&gltf)?;
        while !json_bytes.len().is_multiple_of(4) {
            json_bytes.push(b' ');
        }

        let json_chunk_len = json_bytes.len() as u32;
        let bin_chunk_len = bin_data.len() as u32;
        let total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;

        let mut file = std::fs::File::create(path)?;
        file.write_all(&EP_GLB_MAGIC.to_le_bytes())?;
        file.write_all(&EP_GLB_VER.to_le_bytes())?;
        file.write_all(&total_len.to_le_bytes())?;
        file.write_all(&json_chunk_len.to_le_bytes())?;
        file.write_all(&EP_CHUNK_JSON.to_le_bytes())?;
        file.write_all(&json_bytes)?;
        file.write_all(&bin_chunk_len.to_le_bytes())?;
        file.write_all(&EP_CHUNK_BIN.to_le_bytes())?;
        file.write_all(&bin_data)?;

        Ok(())
    }
}

#[cfg(test)]
mod export_pipeline_tests {
    use super::*;

    fn triangle() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            colors: Some(vec![[1.0, 1.0, 1.0, 1.0]; 3]),
            indices: vec![0, 1, 2],
            has_suit: true,
        }
    }

    #[test]
    fn pipeline_minimal_export() {
        let mesh = triangle();
        let path = std::path::Path::new("/tmp/test_pipeline_minimal.glb");
        ExportPipeline::new()
            .export(&mesh, path)
            .expect("minimal export failed");
        assert!(path.exists(), "GLB file not created");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn pipeline_with_material() {
        let mesh = triangle();
        let path = std::path::Path::new("/tmp/test_pipeline_material.glb");
        ExportPipeline::new()
            .with_material(PbrMaterial::skin())
            .export(&mesh, path)
            .expect("material export failed");
        assert!(path.exists(), "GLB file not created");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn pipeline_with_meta() {
        let mesh = triangle();
        let path = std::path::Path::new("/tmp/test_pipeline_meta.glb");
        ExportPipeline::new()
            .with_meta(OxiHumanMeta::minimal())
            .export(&mesh, path)
            .expect("meta export failed");
        assert!(path.exists(), "GLB file not created");
        let bytes = std::fs::read(path).unwrap();
        assert!(bytes.len() >= 4);
        assert_eq!(&bytes[0..4], b"glTF", "invalid GLB magic");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn pipeline_describe_empty() {
        let desc = ExportPipeline::new().describe();
        assert!(
            !desc.is_empty(),
            "describe() should return non-empty string"
        );
    }

    #[test]
    fn pipeline_describe_with_features() {
        let desc = ExportPipeline::new()
            .with_material(PbrMaterial::skin())
            .with_meta(OxiHumanMeta::minimal())
            .describe();
        assert!(
            desc.contains("material"),
            "describe() should mention 'material', got: {desc}"
        );
    }

    #[test]
    fn pipeline_with_blend_shapes() {
        let mesh = triangle();
        let path = std::path::Path::new("/tmp/test_pipeline_blend_shapes.glb");
        let shapes = vec![BlendShape::zero("neutral", 3)];
        ExportPipeline::new()
            .with_blend_shapes(shapes)
            .export(&mesh, path)
            .expect("blend shapes export failed");
        assert!(path.exists(), "GLB file not created");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn pipeline_with_tangents_and_colors() {
        let mesh = triangle();
        let path = std::path::Path::new("/tmp/test_pipeline_tangents_colors.glb");
        ExportPipeline::new()
            .with_tangents()
            .with_colors()
            .export(&mesh, path)
            .expect("tangents+colors export failed");
        assert!(path.exists(), "GLB file not created");
        std::fs::remove_file(path).ok();
    }
}
