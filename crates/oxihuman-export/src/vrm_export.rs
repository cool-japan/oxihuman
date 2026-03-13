// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! VRM 1.0 export — glTF 2.0 + VRMC extensions for humanoid avatars.
//!
//! Produces a valid GLB binary (`.vrm`) with the `VRMC_vrm` extension
//! containing humanoid bone mapping, avatar metadata, and optional
//! blend shape (expression) data.

use serde_json::json;

// ── GLB constants ────────────────────────────────────────────────────────────

const GLB_MAGIC: u32 = 0x46546C67; // "glTF"
const GLB_VERSION: u32 = 2;
const CHUNK_JSON: u32 = 0x4E4F534A; // "JSON"
const CHUNK_BIN: u32 = 0x004E4942; // "BIN\0"

// ── VRM bone names ───────────────────────────────────────────────────────────

/// All humanoid bone names defined in the VRM 1.0 specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VrmBoneName {
    Hips,
    Spine,
    Chest,
    UpperChest,
    Neck,
    Head,
    LeftUpperArm,
    LeftLowerArm,
    LeftHand,
    RightUpperArm,
    RightLowerArm,
    RightHand,
    LeftUpperLeg,
    LeftLowerLeg,
    LeftFoot,
    RightUpperLeg,
    RightLowerLeg,
    RightFoot,
    LeftThumbProximal,
    LeftThumbIntermediate,
    LeftThumbDistal,
    LeftIndexProximal,
    LeftIndexIntermediate,
    LeftIndexDistal,
    LeftMiddleProximal,
    LeftMiddleIntermediate,
    LeftMiddleDistal,
    LeftRingProximal,
    LeftRingIntermediate,
    LeftRingDistal,
    LeftLittleProximal,
    LeftLittleIntermediate,
    LeftLittleDistal,
    RightThumbProximal,
    RightThumbIntermediate,
    RightThumbDistal,
    RightIndexProximal,
    RightIndexIntermediate,
    RightIndexDistal,
    RightMiddleProximal,
    RightMiddleIntermediate,
    RightMiddleDistal,
    RightRingProximal,
    RightRingIntermediate,
    RightRingDistal,
    RightLittleProximal,
    RightLittleIntermediate,
    RightLittleDistal,
    LeftEye,
    RightEye,
    Jaw,
    LeftShoulder,
    RightShoulder,
    LeftToes,
    RightToes,
}

impl VrmBoneName {
    /// Returns the VRM 1.0 spec camelCase string for this bone.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hips => "hips",
            Self::Spine => "spine",
            Self::Chest => "chest",
            Self::UpperChest => "upperChest",
            Self::Neck => "neck",
            Self::Head => "head",
            Self::LeftUpperArm => "leftUpperArm",
            Self::LeftLowerArm => "leftLowerArm",
            Self::LeftHand => "leftHand",
            Self::RightUpperArm => "rightUpperArm",
            Self::RightLowerArm => "rightLowerArm",
            Self::RightHand => "rightHand",
            Self::LeftUpperLeg => "leftUpperLeg",
            Self::LeftLowerLeg => "leftLowerLeg",
            Self::LeftFoot => "leftFoot",
            Self::RightUpperLeg => "rightUpperLeg",
            Self::RightLowerLeg => "rightLowerLeg",
            Self::RightFoot => "rightFoot",
            Self::LeftThumbProximal => "leftThumbMetacarpal",
            Self::LeftThumbIntermediate => "leftThumbProximal",
            Self::LeftThumbDistal => "leftThumbDistal",
            Self::LeftIndexProximal => "leftIndexProximal",
            Self::LeftIndexIntermediate => "leftIndexIntermediate",
            Self::LeftIndexDistal => "leftIndexDistal",
            Self::LeftMiddleProximal => "leftMiddleProximal",
            Self::LeftMiddleIntermediate => "leftMiddleIntermediate",
            Self::LeftMiddleDistal => "leftMiddleDistal",
            Self::LeftRingProximal => "leftRingProximal",
            Self::LeftRingIntermediate => "leftRingIntermediate",
            Self::LeftRingDistal => "leftRingDistal",
            Self::LeftLittleProximal => "leftLittleProximal",
            Self::LeftLittleIntermediate => "leftLittleIntermediate",
            Self::LeftLittleDistal => "leftLittleDistal",
            Self::RightThumbProximal => "rightThumbMetacarpal",
            Self::RightThumbIntermediate => "rightThumbProximal",
            Self::RightThumbDistal => "rightThumbDistal",
            Self::RightIndexProximal => "rightIndexProximal",
            Self::RightIndexIntermediate => "rightIndexIntermediate",
            Self::RightIndexDistal => "rightIndexDistal",
            Self::RightMiddleProximal => "rightMiddleProximal",
            Self::RightMiddleIntermediate => "rightMiddleIntermediate",
            Self::RightMiddleDistal => "rightMiddleDistal",
            Self::RightRingProximal => "rightRingProximal",
            Self::RightRingIntermediate => "rightRingIntermediate",
            Self::RightRingDistal => "rightRingDistal",
            Self::RightLittleProximal => "rightLittleProximal",
            Self::RightLittleIntermediate => "rightLittleIntermediate",
            Self::RightLittleDistal => "rightLittleDistal",
            Self::LeftEye => "leftEye",
            Self::RightEye => "rightEye",
            Self::Jaw => "jaw",
            Self::LeftShoulder => "leftShoulder",
            Self::RightShoulder => "rightShoulder",
            Self::LeftToes => "leftToes",
            Self::RightToes => "rightToes",
        }
    }

    /// Returns `true` if this bone is required by the VRM 1.0 specification.
    pub fn is_required(&self) -> bool {
        matches!(
            self,
            Self::Hips
                | Self::Spine
                | Self::Chest
                | Self::Neck
                | Self::Head
                | Self::LeftUpperArm
                | Self::LeftLowerArm
                | Self::LeftHand
                | Self::RightUpperArm
                | Self::RightLowerArm
                | Self::RightHand
                | Self::LeftUpperLeg
                | Self::LeftLowerLeg
                | Self::LeftFoot
                | Self::RightUpperLeg
                | Self::RightLowerLeg
                | Self::RightFoot
        )
    }

    /// Returns the complete list of all required VRM bone names.
    pub fn all_required() -> &'static [VrmBoneName] {
        &[
            Self::Hips,
            Self::Spine,
            Self::Chest,
            Self::Neck,
            Self::Head,
            Self::LeftUpperArm,
            Self::LeftLowerArm,
            Self::LeftHand,
            Self::RightUpperArm,
            Self::RightLowerArm,
            Self::RightHand,
            Self::LeftUpperLeg,
            Self::LeftLowerLeg,
            Self::LeftFoot,
            Self::RightUpperLeg,
            Self::RightLowerLeg,
            Self::RightFoot,
        ]
    }
}

// ── VRM humanoid ─────────────────────────────────────────────────────────────

/// A single bone mapping: bone name to glTF node index.
#[derive(Debug, Clone)]
pub struct VrmHumanBone {
    pub name: VrmBoneName,
    pub node_index: usize,
}

/// VRM humanoid bone mapping — maps VRM bone identifiers to glTF node indices.
#[derive(Debug, Clone)]
pub struct VrmHumanoid {
    pub bones: Vec<VrmHumanBone>,
}

impl VrmHumanoid {
    /// Validates that all required bones are present and node indices are unique.
    pub fn validate(&self) -> anyhow::Result<()> {
        for req in VrmBoneName::all_required() {
            if !self.bones.iter().any(|b| b.name == *req) {
                anyhow::bail!(
                    "required VRM bone '{}' is missing from humanoid mapping",
                    req.as_str()
                );
            }
        }
        let mut seen_indices = std::collections::HashSet::new();
        for bone in &self.bones {
            if !seen_indices.insert(bone.node_index) {
                anyhow::bail!(
                    "duplicate node index {} for bone '{}'",
                    bone.node_index,
                    bone.name.as_str()
                );
            }
        }
        Ok(())
    }

    /// Builds the `humanBones` JSON object for VRMC_vrm.
    fn to_json(&self) -> serde_json::Value {
        let mut bones_obj = serde_json::Map::new();
        for bone in &self.bones {
            bones_obj.insert(
                bone.name.as_str().to_string(),
                json!({ "node": bone.node_index }),
            );
        }
        json!({ "humanBones": bones_obj })
    }
}

// ── VRM meta ─────────────────────────────────────────────────────────────────

/// Commercial usage permission level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VrmCommercialUsage {
    PersonalNonProfit,
    PersonalProfit,
    Corporation,
}

impl VrmCommercialUsage {
    fn as_str(&self) -> &'static str {
        match self {
            Self::PersonalNonProfit => "personalNonProfit",
            Self::PersonalProfit => "personalProfit",
            Self::Corporation => "corporation",
        }
    }
}

/// Credit notation requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VrmCreditNotation {
    Required,
    Unnecessary,
}

impl VrmCreditNotation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Unnecessary => "unnecessary",
        }
    }
}

/// Model modification permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VrmModification {
    Prohibited,
    AllowModification,
    AllowModificationRedistribution,
}

impl VrmModification {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Prohibited => "prohibited",
            Self::AllowModification => "allowModification",
            Self::AllowModificationRedistribution => "allowModificationRedistribution",
        }
    }
}

/// VRM 1.0 avatar metadata (the `meta` block inside `VRMC_vrm`).
#[derive(Debug, Clone)]
pub struct VrmMeta {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub license_url: String,
    pub allow_antisocial_actions: bool,
    pub allow_political_or_religious_usage: bool,
    pub allow_excessively_violent_usage: bool,
    pub allow_excessively_sexual_usage: bool,
    pub commercial_usage: VrmCommercialUsage,
    pub credit_notation: VrmCreditNotation,
    pub modification: VrmModification,
}

impl VrmMeta {
    /// Creates a default meta with CC-BY-4.0 license and permissive settings.
    pub fn default_cc_by(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "1.0".to_string(),
            authors: vec!["OxiHuman".to_string()],
            license_url: "https://creativecommons.org/licenses/by/4.0/".to_string(),
            allow_antisocial_actions: false,
            allow_political_or_religious_usage: false,
            allow_excessively_violent_usage: false,
            allow_excessively_sexual_usage: false,
            commercial_usage: VrmCommercialUsage::PersonalProfit,
            credit_notation: VrmCreditNotation::Required,
            modification: VrmModification::AllowModificationRedistribution,
        }
    }

    /// Validates that meta fields are non-empty.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.name.trim().is_empty() {
            anyhow::bail!("VRM meta name must not be empty");
        }
        if self.authors.is_empty() {
            anyhow::bail!("VRM meta must have at least one author");
        }
        if self.license_url.trim().is_empty() {
            anyhow::bail!("VRM meta license_url must not be empty");
        }
        Ok(())
    }

    /// Builds the `meta` JSON object for VRMC_vrm.
    fn to_json(&self) -> serde_json::Value {
        json!({
            "name": self.name,
            "version": self.version,
            "authors": self.authors,
            "licenseUrl": self.license_url,
            "allowAntisocialActions": self.allow_antisocial_actions,
            "allowPoliticalOrReligiousUsage": self.allow_political_or_religious_usage,
            "allowExcessivelyViolentUsage": self.allow_excessively_violent_usage,
            "allowExcessivelySexualUsage": self.allow_excessively_sexual_usage,
            "commercialUsage": self.commercial_usage.as_str(),
            "creditNotation": self.credit_notation.as_str(),
            "modification": self.modification.as_str(),
        })
    }
}

// ── VRM exporter ─────────────────────────────────────────────────────────────

/// VRM 1.0 exporter — builds a GLB binary with VRMC_vrm extensions.
pub struct VrmExporter {
    gltf_json: serde_json::Value,
    binary_buffer: Vec<u8>,
    has_mesh: bool,
    has_skeleton: bool,
    has_humanoid: bool,
    has_meta: bool,
    vertex_count: usize,
    index_count: usize,
    node_count: usize,
}

impl VrmExporter {
    /// Creates a new VRM exporter with an empty glTF document.
    pub fn new() -> Self {
        let gltf_json = json!({
            "asset": {
                "version": "2.0",
                "generator": "OxiHuman VRM Exporter 0.1.0"
            },
            "scene": 0,
            "scenes": [{ "nodes": [] }],
            "nodes": [],
            "meshes": [],
            "accessors": [],
            "bufferViews": [],
            "buffers": [{ "byteLength": 0 }],
            "extensionsUsed": ["VRMC_vrm"],
            "extensions": {
                "VRMC_vrm": {
                    "specVersion": "1.0"
                }
            }
        });
        Self {
            gltf_json,
            binary_buffer: Vec::new(),
            has_mesh: false,
            has_skeleton: false,
            has_humanoid: false,
            has_meta: false,
            vertex_count: 0,
            index_count: 0,
            node_count: 0,
        }
    }

    /// Sets the mesh geometry data.
    pub fn set_mesh(
        &mut self,
        positions: &[[f64; 3]],
        normals: &[[f64; 3]],
        uvs: &[[f64; 2]],
        triangles: &[[usize; 3]],
    ) -> anyhow::Result<()> {
        let n_verts = positions.len();
        if normals.len() != n_verts {
            anyhow::bail!(
                "normals count ({}) must match positions count ({})",
                normals.len(),
                n_verts
            );
        }
        if uvs.len() != n_verts {
            anyhow::bail!(
                "uvs count ({}) must match positions count ({})",
                uvs.len(),
                n_verts
            );
        }
        if n_verts == 0 {
            anyhow::bail!("mesh must have at least one vertex");
        }
        if triangles.is_empty() {
            anyhow::bail!("mesh must have at least one triangle");
        }

        for (ti, tri) in triangles.iter().enumerate() {
            for &idx in tri {
                if idx >= n_verts {
                    anyhow::bail!(
                        "triangle {} has index {} which exceeds vertex count {}",
                        ti,
                        idx,
                        n_verts
                    );
                }
            }
        }

        self.binary_buffer.clear();

        // Write positions as f32
        let pos_offset = self.binary_buffer.len();
        let mut pos_min = [f64::MAX; 3];
        let mut pos_max = [f64::MIN; 3];
        for pos in positions {
            for axis in 0..3 {
                if pos[axis] < pos_min[axis] {
                    pos_min[axis] = pos[axis];
                }
                if pos[axis] > pos_max[axis] {
                    pos_max[axis] = pos[axis];
                }
                let val = pos[axis] as f32;
                self.binary_buffer.extend_from_slice(&val.to_le_bytes());
            }
        }
        let pos_byte_len = self.binary_buffer.len() - pos_offset;

        let norm_offset = self.binary_buffer.len();
        for norm in normals {
            for &component in norm.iter().take(3) {
                let val = component as f32;
                self.binary_buffer.extend_from_slice(&val.to_le_bytes());
            }
        }
        let norm_byte_len = self.binary_buffer.len() - norm_offset;

        let uv_offset = self.binary_buffer.len();
        for uv in uvs {
            for &component in uv.iter().take(2) {
                let val = component as f32;
                self.binary_buffer.extend_from_slice(&val.to_le_bytes());
            }
        }
        let uv_byte_len = self.binary_buffer.len() - uv_offset;

        let idx_offset = self.binary_buffer.len();
        let n_indices = triangles.len() * 3;
        for tri in triangles {
            for &idx in tri {
                let val = idx as u32;
                self.binary_buffer.extend_from_slice(&val.to_le_bytes());
            }
        }
        let idx_byte_len = self.binary_buffer.len() - idx_offset;

        self.gltf_json["bufferViews"] = json!([
            { "buffer": 0, "byteOffset": pos_offset,  "byteLength": pos_byte_len,  "target": 34962 },
            { "buffer": 0, "byteOffset": norm_offset, "byteLength": norm_byte_len, "target": 34962 },
            { "buffer": 0, "byteOffset": uv_offset,   "byteLength": uv_byte_len,   "target": 34962 },
            { "buffer": 0, "byteOffset": idx_offset,  "byteLength": idx_byte_len,  "target": 34963 }
        ]);

        self.gltf_json["accessors"] = json!([
            {
                "bufferView": 0, "componentType": 5126, "count": n_verts, "type": "VEC3",
                "min": [pos_min[0] as f32, pos_min[1] as f32, pos_min[2] as f32],
                "max": [pos_max[0] as f32, pos_max[1] as f32, pos_max[2] as f32]
            },
            { "bufferView": 1, "componentType": 5126, "count": n_verts, "type": "VEC3" },
            { "bufferView": 2, "componentType": 5126, "count": n_verts, "type": "VEC2" },
            { "bufferView": 3, "componentType": 5125, "count": n_indices, "type": "SCALAR" }
        ]);

        self.gltf_json["meshes"] = json!([{
            "name": "VRM_Mesh",
            "primitives": [{
                "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
                "indices": 3,
                "mode": 4
            }]
        }]);

        self.vertex_count = n_verts;
        self.index_count = n_indices;
        self.has_mesh = true;
        Ok(())
    }

    /// Sets the skeleton (bone hierarchy) for the avatar.
    pub fn set_skeleton(
        &mut self,
        bone_names: &[String],
        bone_parents: &[Option<usize>],
        bind_poses: &[[f64; 16]],
    ) -> anyhow::Result<()> {
        let n_bones = bone_names.len();
        if bone_parents.len() != n_bones {
            anyhow::bail!(
                "bone_parents length ({}) must match bone_names length ({})",
                bone_parents.len(),
                n_bones
            );
        }
        if bind_poses.len() != n_bones {
            anyhow::bail!(
                "bind_poses length ({}) must match bone_names length ({})",
                bind_poses.len(),
                n_bones
            );
        }
        if n_bones == 0 {
            anyhow::bail!("skeleton must have at least one bone");
        }

        for (i, parent) in bone_parents.iter().enumerate() {
            if let Some(p) = parent {
                if *p >= n_bones {
                    anyhow::bail!(
                        "bone {} has parent index {} which exceeds bone count {}",
                        i,
                        p,
                        n_bones
                    );
                }
                if *p == i {
                    anyhow::bail!("bone {} cannot be its own parent", i);
                }
            }
        }

        let mut children: Vec<Vec<usize>> = vec![Vec::new(); n_bones];
        let mut roots: Vec<usize> = Vec::new();
        for (i, parent) in bone_parents.iter().enumerate() {
            match parent {
                Some(p) => children[*p].push(i),
                None => roots.push(i),
            }
        }

        if roots.is_empty() {
            anyhow::bail!("skeleton must have at least one root bone (no parent)");
        }

        let mut nodes = serde_json::Value::Array(Vec::new());
        let nodes_arr = nodes
            .as_array_mut()
            .ok_or_else(|| anyhow::anyhow!("internal: failed to create nodes array"))?;

        for i in 0..n_bones {
            let (translation, rotation, scale) = decompose_matrix(&bind_poses[i]);
            let mut node = json!({
                "name": bone_names[i],
                "translation": [translation[0] as f32, translation[1] as f32, translation[2] as f32],
                "rotation": [rotation[0] as f32, rotation[1] as f32, rotation[2] as f32, rotation[3] as f32],
                "scale": [scale[0] as f32, scale[1] as f32, scale[2] as f32]
            });
            if !children[i].is_empty() {
                node["children"] = json!(children[i]);
            }
            nodes_arr.push(node);
        }

        let mesh_node_idx = n_bones;
        nodes_arr.push(json!({ "name": "VRM_MeshNode", "mesh": 0, "skin": 0 }));

        let mut scene_nodes: Vec<usize> = roots.clone();
        scene_nodes.push(mesh_node_idx);

        // Write inverse bind matrices to binary buffer
        let ibm_offset = self.binary_buffer.len();
        for bind_pose in bind_poses {
            let inv = invert_matrix_4x4(bind_pose);
            for val in &inv {
                let f = *val as f32;
                self.binary_buffer.extend_from_slice(&f.to_le_bytes());
            }
        }
        let ibm_byte_len = self.binary_buffer.len() - ibm_offset;

        let bv_idx = self.gltf_json["bufferViews"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);
        let acc_idx = self.gltf_json["accessors"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);

        if let Some(bvs) = self.gltf_json["bufferViews"].as_array_mut() {
            bvs.push(json!({
                "buffer": 0, "byteOffset": ibm_offset, "byteLength": ibm_byte_len
            }));
        }
        if let Some(accs) = self.gltf_json["accessors"].as_array_mut() {
            accs.push(json!({
                "bufferView": bv_idx, "componentType": 5126, "count": n_bones, "type": "MAT4"
            }));
        }

        let all_joints: Vec<usize> = (0..n_bones).collect();
        let skeleton_root = roots.first().copied().unwrap_or(0);

        self.gltf_json["skins"] = json!([{
            "joints": all_joints,
            "skeleton": skeleton_root,
            "inverseBindMatrices": acc_idx
        }]);

        self.gltf_json["nodes"] = nodes;
        self.gltf_json["scenes"] = json!([{ "nodes": scene_nodes }]);
        self.node_count = n_bones + 1;
        self.has_skeleton = true;
        Ok(())
    }

    /// Sets the VRM humanoid bone mapping.
    pub fn set_humanoid(&mut self, humanoid: &VrmHumanoid) -> anyhow::Result<()> {
        humanoid.validate()?;
        if self.has_skeleton {
            for bone in &humanoid.bones {
                if bone.node_index >= self.node_count {
                    anyhow::bail!(
                        "humanoid bone '{}' references node index {} but only {} nodes exist",
                        bone.name.as_str(),
                        bone.node_index,
                        self.node_count
                    );
                }
            }
        }
        self.gltf_json["extensions"]["VRMC_vrm"]["humanoid"] = humanoid.to_json();
        self.has_humanoid = true;
        Ok(())
    }

    /// Sets the VRM avatar metadata.
    pub fn set_meta(&mut self, meta: &VrmMeta) -> anyhow::Result<()> {
        meta.validate()?;
        self.gltf_json["extensions"]["VRMC_vrm"]["meta"] = meta.to_json();
        self.has_meta = true;
        Ok(())
    }

    /// Sets blend shape (morph target) data for the mesh.
    pub fn set_blend_shapes(&mut self, shapes: &[(String, Vec<[f64; 3]>)]) -> anyhow::Result<()> {
        if !self.has_mesh {
            anyhow::bail!("set_mesh must be called before set_blend_shapes");
        }
        if shapes.is_empty() {
            return Ok(());
        }

        for (name, deltas) in shapes {
            if deltas.len() != self.vertex_count {
                anyhow::bail!(
                    "blend shape '{}' has {} deltas but mesh has {} vertices",
                    name,
                    deltas.len(),
                    self.vertex_count
                );
            }
        }

        let mut morph_targets: Vec<serde_json::Value> = Vec::new();

        for (_, deltas) in shapes {
            let delta_offset = self.binary_buffer.len();
            for delta in deltas {
                for &component in delta.iter().take(3) {
                    let val = component as f32;
                    self.binary_buffer.extend_from_slice(&val.to_le_bytes());
                }
            }
            let delta_byte_len = self.binary_buffer.len() - delta_offset;

            let bv_idx = self.gltf_json["bufferViews"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0);
            if let Some(bvs) = self.gltf_json["bufferViews"].as_array_mut() {
                bvs.push(json!({
                    "buffer": 0, "byteOffset": delta_offset, "byteLength": delta_byte_len
                }));
            }

            let acc_idx = self.gltf_json["accessors"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0);
            if let Some(accs) = self.gltf_json["accessors"].as_array_mut() {
                accs.push(json!({
                    "bufferView": bv_idx, "componentType": 5126,
                    "count": self.vertex_count, "type": "VEC3"
                }));
            }

            morph_targets.push(json!({ "POSITION": acc_idx }));
        }

        if let Some(meshes) = self.gltf_json["meshes"].as_array_mut() {
            if let Some(mesh) = meshes.first_mut() {
                if let Some(prims) = mesh["primitives"].as_array_mut() {
                    if let Some(prim) = prims.first_mut() {
                        prim["targets"] = json!(morph_targets);
                    }
                }
                let target_names: Vec<&str> = shapes.iter().map(|(n, _)| n.as_str()).collect();
                mesh["extras"] = json!({ "targetNames": target_names });
            }
        }

        // Build VRMC_vrm expressions
        let mut expressions = serde_json::Map::new();
        let mut preset_map = serde_json::Map::new();
        let mut custom_map = serde_json::Map::new();

        for (i, (name, _)) in shapes.iter().enumerate() {
            let expression_entry = json!({
                "morphTargetBinds": [{ "node": 0, "index": i, "weight": 1.0 }]
            });
            match map_expression_preset(name) {
                Some(preset_name) => {
                    preset_map.insert(preset_name.to_string(), expression_entry);
                }
                None => {
                    custom_map.insert(name.clone(), expression_entry);
                }
            }
        }

        expressions.insert("preset".to_string(), serde_json::Value::Object(preset_map));
        if !custom_map.is_empty() {
            expressions.insert("custom".to_string(), serde_json::Value::Object(custom_map));
        }

        self.gltf_json["extensions"]["VRMC_vrm"]["expressions"] =
            serde_json::Value::Object(expressions);
        Ok(())
    }

    /// Exports the VRM as a GLB binary (`.vrm` file contents).
    pub fn export(&self) -> anyhow::Result<Vec<u8>> {
        if !self.has_mesh {
            anyhow::bail!("cannot export VRM: no mesh data set (call set_mesh first)");
        }
        if !self.has_humanoid {
            anyhow::bail!("cannot export VRM: no humanoid mapping set (call set_humanoid first)");
        }
        if !self.has_meta {
            anyhow::bail!("cannot export VRM: no meta set (call set_meta first)");
        }

        let mut gltf = self.gltf_json.clone();

        let mut bin_data = self.binary_buffer.clone();
        while !bin_data.len().is_multiple_of(4) {
            bin_data.push(0x00);
        }

        gltf["buffers"] = json!([{ "byteLength": bin_data.len() }]);

        let mut json_bytes = serde_json::to_vec(&gltf)?;
        while json_bytes.len() % 4 != 0 {
            json_bytes.push(b' ');
        }

        let json_chunk_len = json_bytes.len() as u32;
        let bin_chunk_len = bin_data.len() as u32;
        let total_len: u32 = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;

        let mut output: Vec<u8> = Vec::with_capacity(total_len as usize);

        // GLB header
        output.extend_from_slice(&GLB_MAGIC.to_le_bytes());
        output.extend_from_slice(&GLB_VERSION.to_le_bytes());
        output.extend_from_slice(&total_len.to_le_bytes());

        // JSON chunk
        output.extend_from_slice(&json_chunk_len.to_le_bytes());
        output.extend_from_slice(&CHUNK_JSON.to_le_bytes());
        output.extend_from_slice(&json_bytes);

        // BIN chunk
        output.extend_from_slice(&bin_chunk_len.to_le_bytes());
        output.extend_from_slice(&CHUNK_BIN.to_le_bytes());
        output.extend_from_slice(&bin_data);

        Ok(output)
    }
}

impl Default for VrmExporter {
    fn default() -> Self {
        Self::new()
    }
}

// ── Expression preset mapping ────────────────────────────────────────────────

fn map_expression_preset(name: &str) -> Option<&'static str> {
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "happy" | "joy" | "smile" => Some("happy"),
        "angry" | "anger" => Some("angry"),
        "sad" | "sorrow" => Some("sad"),
        "relaxed" | "calm" => Some("relaxed"),
        "surprised" | "surprise" => Some("surprised"),
        "aa" | "a" => Some("aa"),
        "ih" | "i" => Some("ih"),
        "ou" | "u" => Some("ou"),
        "ee" | "e" => Some("ee"),
        "oh" | "o" => Some("oh"),
        "blink" => Some("blink"),
        "blinkleft" | "blink_left" | "blink_l" => Some("blinkLeft"),
        "blinkright" | "blink_right" | "blink_r" => Some("blinkRight"),
        "lookup" | "look_up" => Some("lookUp"),
        "lookdown" | "look_down" => Some("lookDown"),
        "lookleft" | "look_left" => Some("lookLeft"),
        "lookright" | "look_right" => Some("lookRight"),
        "neutral" => Some("neutral"),
        _ => None,
    }
}

// ── Matrix math helpers ──────────────────────────────────────────────────────

fn decompose_matrix(m: &[f64; 16]) -> ([f64; 3], [f64; 4], [f64; 3]) {
    let translation = [m[12], m[13], m[14]];

    let col0 = [m[0], m[1], m[2]];
    let col1 = [m[4], m[5], m[6]];
    let col2 = [m[8], m[9], m[10]];

    let sx = vec3_length(&col0);
    let sy = vec3_length(&col1);
    let sz = vec3_length(&col2);
    let scale = [sx, sy, sz];

    let safe_sx = if sx.abs() < 1e-12 { 1.0 } else { sx };
    let safe_sy = if sy.abs() < 1e-12 { 1.0 } else { sy };
    let safe_sz = if sz.abs() < 1e-12 { 1.0 } else { sz };

    let r00 = col0[0] / safe_sx;
    let r10 = col0[1] / safe_sx;
    let r20 = col0[2] / safe_sx;
    let r01 = col1[0] / safe_sy;
    let r11 = col1[1] / safe_sy;
    let r21 = col1[2] / safe_sy;
    let r02 = col2[0] / safe_sz;
    let r12 = col2[1] / safe_sz;
    let r22 = col2[2] / safe_sz;

    let rotation = rotation_matrix_to_quat(r00, r01, r02, r10, r11, r12, r20, r21, r22);
    (translation, rotation, scale)
}

#[allow(clippy::too_many_arguments)]
fn rotation_matrix_to_quat(
    r00: f64,
    r01: f64,
    r02: f64,
    r10: f64,
    r11: f64,
    r12: f64,
    r20: f64,
    r21: f64,
    r22: f64,
) -> [f64; 4] {
    let trace = r00 + r11 + r22;
    let (x, y, z, w) = if trace > 0.0 {
        let s = 0.5 / (trace + 1.0).sqrt();
        ((r21 - r12) * s, (r02 - r20) * s, (r10 - r01) * s, 0.25 / s)
    } else if r00 > r11 && r00 > r22 {
        let s = 2.0 * (1.0 + r00 - r11 - r22).sqrt();
        (0.25 * s, (r01 + r10) / s, (r02 + r20) / s, (r21 - r12) / s)
    } else if r11 > r22 {
        let s = 2.0 * (1.0 + r11 - r00 - r22).sqrt();
        ((r01 + r10) / s, 0.25 * s, (r12 + r21) / s, (r02 - r20) / s)
    } else {
        let s = 2.0 * (1.0 + r22 - r00 - r11).sqrt();
        ((r02 + r20) / s, (r12 + r21) / s, 0.25 * s, (r10 - r01) / s)
    };

    let len = (x * x + y * y + z * z + w * w).sqrt();
    if len.abs() < 1e-12 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [x / len, y / len, z / len, w / len]
}

fn vec3_length(v: &[f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn invert_matrix_4x4(m: &[f64; 16]) -> [f64; 16] {
    let (a00, a01, a02, a03) = (m[0], m[1], m[2], m[3]);
    let (a10, a11, a12, a13) = (m[4], m[5], m[6], m[7]);
    let (a20, a21, a22, a23) = (m[8], m[9], m[10], m[11]);
    let (a30, a31, a32, a33) = (m[12], m[13], m[14], m[15]);

    let b00 = a00 * a11 - a01 * a10;
    let b01 = a00 * a12 - a02 * a10;
    let b02 = a00 * a13 - a03 * a10;
    let b03 = a01 * a12 - a02 * a11;
    let b04 = a01 * a13 - a03 * a11;
    let b05 = a02 * a13 - a03 * a12;
    let b06 = a20 * a31 - a21 * a30;
    let b07 = a20 * a32 - a22 * a30;
    let b08 = a20 * a33 - a23 * a30;
    let b09 = a21 * a32 - a22 * a31;
    let b10 = a21 * a33 - a23 * a31;
    let b11 = a22 * a33 - a23 * a32;

    let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;

    if det.abs() < 1e-14 {
        return [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
    }

    let inv_det = 1.0 / det;
    [
        (a11 * b11 - a12 * b10 + a13 * b09) * inv_det,
        (a02 * b10 - a01 * b11 - a03 * b09) * inv_det,
        (a31 * b05 - a32 * b04 + a33 * b03) * inv_det,
        (a22 * b04 - a21 * b05 - a23 * b03) * inv_det,
        (a12 * b08 - a10 * b11 - a13 * b07) * inv_det,
        (a00 * b11 - a02 * b08 + a03 * b07) * inv_det,
        (a32 * b02 - a30 * b05 - a33 * b01) * inv_det,
        (a20 * b05 - a22 * b02 + a23 * b01) * inv_det,
        (a10 * b10 - a11 * b08 + a13 * b06) * inv_det,
        (a01 * b08 - a00 * b10 - a03 * b06) * inv_det,
        (a30 * b04 - a31 * b02 + a33 * b00) * inv_det,
        (a21 * b02 - a20 * b04 - a23 * b00) * inv_det,
        (a11 * b07 - a10 * b09 - a12 * b06) * inv_det,
        (a00 * b09 - a01 * b07 + a02 * b06) * inv_det,
        (a31 * b01 - a30 * b03 - a32 * b00) * inv_det,
        (a20 * b03 - a21 * b01 + a22 * b00) * inv_det,
    ]
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_humanoid() -> VrmHumanoid {
        let required = VrmBoneName::all_required();
        VrmHumanoid {
            bones: required
                .iter()
                .enumerate()
                .map(|(i, &name)| VrmHumanBone {
                    name,
                    node_index: i,
                })
                .collect(),
        }
    }

    fn identity_matrix() -> [f64; 16] {
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
    }

    fn minimal_exporter() -> VrmExporter {
        let mut exporter = VrmExporter::new();
        exporter
            .set_mesh(
                &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                &[[0.0, 0.0, 1.0]; 3],
                &[[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                &[[0, 1, 2]],
            )
            .expect("set_mesh failed");

        let humanoid = minimal_humanoid();
        let n_bones = humanoid.bones.len();
        let bone_names: Vec<String> = humanoid
            .bones
            .iter()
            .map(|b| b.name.as_str().to_string())
            .collect();
        let mut bone_parents: Vec<Option<usize>> = vec![Some(0); n_bones];
        bone_parents[0] = None;
        let bind_poses: Vec<[f64; 16]> = vec![identity_matrix(); n_bones];

        exporter
            .set_skeleton(&bone_names, &bone_parents, &bind_poses)
            .expect("set_skeleton failed");
        exporter
            .set_humanoid(&humanoid)
            .expect("set_humanoid failed");
        exporter
            .set_meta(&VrmMeta::default_cc_by("TestAvatar"))
            .expect("set_meta failed");
        exporter
    }

    #[test]
    fn bone_name_as_str_hips() {
        assert_eq!(VrmBoneName::Hips.as_str(), "hips");
    }

    #[test]
    fn bone_name_as_str_head() {
        assert_eq!(VrmBoneName::Head.as_str(), "head");
    }

    #[test]
    fn bone_name_required_hips() {
        assert!(VrmBoneName::Hips.is_required());
    }

    #[test]
    fn bone_name_optional_jaw() {
        assert!(!VrmBoneName::Jaw.is_required());
    }

    #[test]
    fn all_required_count() {
        assert_eq!(VrmBoneName::all_required().len(), 17);
    }

    #[test]
    fn humanoid_validate_ok() {
        assert!(minimal_humanoid().validate().is_ok());
    }

    #[test]
    fn humanoid_validate_missing_bone() {
        let h = VrmHumanoid {
            bones: vec![VrmHumanBone {
                name: VrmBoneName::Hips,
                node_index: 0,
            }],
        };
        assert!(h.validate().is_err());
    }

    #[test]
    fn humanoid_validate_duplicate_node() {
        let bones: Vec<VrmHumanBone> = VrmBoneName::all_required()
            .iter()
            .map(|&name| VrmHumanBone {
                name,
                node_index: 0,
            })
            .collect();
        assert!(VrmHumanoid { bones }.validate().is_err());
    }

    #[test]
    fn meta_validate_ok() {
        assert!(VrmMeta::default_cc_by("Test").validate().is_ok());
    }

    #[test]
    fn meta_validate_empty_name() {
        let mut meta = VrmMeta::default_cc_by("Test");
        meta.name = "  ".to_string();
        assert!(meta.validate().is_err());
    }

    #[test]
    fn meta_validate_no_authors() {
        let mut meta = VrmMeta::default_cc_by("Test");
        meta.authors.clear();
        assert!(meta.validate().is_err());
    }

    #[test]
    fn new_exporter_defaults() {
        let exp = VrmExporter::new();
        assert!(!exp.has_mesh);
        assert!(!exp.has_skeleton);
        assert!(!exp.has_humanoid);
        assert!(!exp.has_meta);
    }

    #[test]
    fn set_mesh_basic() {
        let mut exp = VrmExporter::new();
        let result = exp.set_mesh(
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[[0.0, 0.0, 1.0]; 3],
            &[[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            &[[0, 1, 2]],
        );
        assert!(result.is_ok());
        assert!(exp.has_mesh);
    }

    #[test]
    fn set_mesh_mismatched_normals() {
        let mut exp = VrmExporter::new();
        let result = exp.set_mesh(
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            &[[0.0, 0.0, 1.0]],
            &[[0.0, 0.0], [1.0, 0.0]],
            &[[0, 1, 0]],
        );
        assert!(result.is_err());
    }

    #[test]
    fn set_mesh_invalid_index() {
        let mut exp = VrmExporter::new();
        let result = exp.set_mesh(
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[[0.0, 0.0, 1.0]; 3],
            &[[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            &[[0, 1, 99]],
        );
        assert!(result.is_err());
    }

    #[test]
    fn export_without_mesh_fails() {
        assert!(VrmExporter::new().export().is_err());
    }

    #[test]
    fn export_minimal_vrm_produces_valid_glb() {
        let bytes = minimal_exporter().export().expect("export failed");
        assert!(bytes.len() >= 12);
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(magic, GLB_MAGIC);
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(version, 2);
        let total_len = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        assert_eq!(total_len as usize, bytes.len());
    }

    #[test]
    fn export_contains_vrmc_vrm_extension() {
        let bytes = minimal_exporter().export().expect("export failed");
        let json_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let json_str = std::str::from_utf8(&bytes[20..20 + json_len])
            .expect("invalid utf8")
            .trim_end_matches(' ');
        let parsed: serde_json::Value = serde_json::from_str(json_str).expect("invalid JSON");
        assert!(parsed["extensions"]["VRMC_vrm"].is_object());
        assert_eq!(
            parsed["extensions"]["VRMC_vrm"]["specVersion"]
                .as_str()
                .unwrap_or(""),
            "1.0"
        );
    }

    #[test]
    fn export_contains_humanoid_bones() {
        let bytes = minimal_exporter().export().expect("export failed");
        let json_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let json_str = std::str::from_utf8(&bytes[20..20 + json_len])
            .expect("invalid utf8")
            .trim_end_matches(' ');
        let parsed: serde_json::Value = serde_json::from_str(json_str).expect("invalid JSON");
        let humanoid = &parsed["extensions"]["VRMC_vrm"]["humanoid"];
        assert!(humanoid["humanBones"]["hips"].is_object());
        assert!(humanoid["humanBones"]["head"].is_object());
    }

    #[test]
    fn export_contains_meta() {
        let bytes = minimal_exporter().export().expect("export failed");
        let json_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let json_str = std::str::from_utf8(&bytes[20..20 + json_len])
            .expect("invalid utf8")
            .trim_end_matches(' ');
        let parsed: serde_json::Value = serde_json::from_str(json_str).expect("invalid JSON");
        let meta = &parsed["extensions"]["VRMC_vrm"]["meta"];
        assert_eq!(meta["name"].as_str().unwrap_or(""), "TestAvatar");
    }

    #[test]
    fn export_has_extensions_used() {
        let bytes = minimal_exporter().export().expect("export failed");
        let json_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let json_str = std::str::from_utf8(&bytes[20..20 + json_len])
            .expect("invalid utf8")
            .trim_end_matches(' ');
        let parsed: serde_json::Value = serde_json::from_str(json_str).expect("invalid JSON");
        let ext_used = parsed["extensionsUsed"]
            .as_array()
            .expect("extensionsUsed missing");
        assert!(ext_used.iter().any(|v| v.as_str() == Some("VRMC_vrm")));
    }

    #[test]
    fn export_has_skin_with_joints() {
        let bytes = minimal_exporter().export().expect("export failed");
        let json_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let json_str = std::str::from_utf8(&bytes[20..20 + json_len])
            .expect("invalid utf8")
            .trim_end_matches(' ');
        let parsed: serde_json::Value = serde_json::from_str(json_str).expect("invalid JSON");
        let skins = parsed["skins"].as_array().expect("skins missing");
        assert!(!skins.is_empty());
        assert!(!skins[0]["joints"]
            .as_array()
            .expect("joints missing")
            .is_empty());
    }

    #[test]
    fn export_with_blend_shapes() {
        let mut exp = minimal_exporter();
        let shapes = vec![
            ("happy".to_string(), vec![[0.0, 0.01, 0.0]; 3]),
            ("angry".to_string(), vec![[0.0, -0.01, 0.0]; 3]),
            ("custom_face".to_string(), vec![[0.01, 0.0, 0.0]; 3]),
        ];
        exp.set_blend_shapes(&shapes)
            .expect("set_blend_shapes failed");
        let bytes = exp.export().expect("export failed");
        let json_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let json_str = std::str::from_utf8(&bytes[20..20 + json_len])
            .expect("invalid utf8")
            .trim_end_matches(' ');
        let parsed: serde_json::Value = serde_json::from_str(json_str).expect("invalid JSON");

        let targets = &parsed["meshes"][0]["primitives"][0]["targets"];
        assert_eq!(targets.as_array().map(|a| a.len()).unwrap_or(0), 3);

        let expressions = &parsed["extensions"]["VRMC_vrm"]["expressions"];
        assert!(expressions["preset"]["happy"].is_object());
        assert!(expressions["preset"]["angry"].is_object());
        assert!(expressions["custom"]["custom_face"].is_object());
    }

    #[test]
    fn blend_shapes_before_mesh_fails() {
        let mut exp = VrmExporter::new();
        assert!(exp
            .set_blend_shapes(&[("test".to_string(), vec![[0.0; 3]])])
            .is_err());
    }

    #[test]
    fn write_vrm_to_file() {
        let bytes = minimal_exporter().export().expect("export failed");
        let path = std::env::temp_dir().join("test_oxihuman_vrm_export.vrm");
        std::fs::write(&path, &bytes).expect("write failed");
        assert!(path.exists());
        assert_eq!(
            std::fs::read(&path).expect("read failed").len(),
            bytes.len()
        );
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn decompose_identity_matrix_test() {
        let (t, r, s) = decompose_matrix(&identity_matrix());
        assert!((t[0]).abs() < 1e-6);
        assert!((r[3] - 1.0).abs() < 1e-6);
        assert!((s[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn invert_identity_returns_identity() {
        let inv = invert_matrix_4x4(&identity_matrix());
        for (i, &val) in inv.iter().enumerate() {
            let expected = if i % 5 == 0 { 1.0 } else { 0.0 };
            assert!((val - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn expression_preset_mapping_test() {
        assert_eq!(map_expression_preset("happy"), Some("happy"));
        assert_eq!(map_expression_preset("Happy"), Some("happy"));
        assert_eq!(map_expression_preset("custom_thing"), None);
    }

    #[test]
    fn default_exporter_impl() {
        assert!(!VrmExporter::default().has_mesh);
    }

    #[test]
    fn set_skeleton_self_parent() {
        let mut exp = VrmExporter::new();
        assert!(exp
            .set_skeleton(&["Root".to_string()], &[Some(0)], &[identity_matrix()])
            .is_err());
    }

    #[test]
    fn commercial_usage_str_values() {
        assert_eq!(
            VrmCommercialUsage::PersonalNonProfit.as_str(),
            "personalNonProfit"
        );
        assert_eq!(VrmCommercialUsage::Corporation.as_str(), "corporation");
    }

    #[test]
    fn modification_str_values() {
        assert_eq!(VrmModification::Prohibited.as_str(), "prohibited");
        assert_eq!(
            VrmModification::AllowModificationRedistribution.as_str(),
            "allowModificationRedistribution"
        );
    }
}
