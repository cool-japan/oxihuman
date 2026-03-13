#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export skin weights for a mesh.

#[allow(dead_code)]
pub struct SkinWeightExport {
    pub vertex_idx: u32,
    pub bone_name: String,
    pub weight: f32,
}

#[allow(dead_code)]
pub struct SkinExport {
    pub mesh_name: String,
    pub weights: Vec<SkinWeightExport>,
}

#[allow(dead_code)]
pub fn new_skin_export(mesh: &str) -> SkinExport {
    SkinExport { mesh_name: mesh.to_string(), weights: Vec::new() }
}

#[allow(dead_code)]
pub fn add_weight(s: &mut SkinExport, vert: u32, bone: &str, w: f32) {
    s.weights.push(SkinWeightExport { vertex_idx: vert, bone_name: bone.to_string(), weight: w });
}

#[allow(dead_code)]
pub fn weight_count(s: &SkinExport) -> usize {
    s.weights.len()
}

#[allow(dead_code)]
pub fn export_skin_to_json(s: &SkinExport) -> String {
    let mut out = format!("{{\"mesh\":\"{}\",\"weights\":[", s.mesh_name);
    for (i, w) in s.weights.iter().enumerate() {
        if i > 0 { out.push(','); }
        out.push_str(&format!("{{\"vert\":{},\"bone\":\"{}\",\"weight\":{}}}", w.vertex_idx, w.bone_name, w.weight));
    }
    out.push_str("]}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_skin_empty() {
        let s = new_skin_export("Body");
        assert_eq!(s.mesh_name, "Body");
        assert!(s.weights.is_empty());
    }

    #[test]
    fn add_weight_stored() {
        let mut s = new_skin_export("M");
        add_weight(&mut s, 0, "spine", 1.0);
        assert_eq!(s.weights.len(), 1);
    }

    #[test]
    fn weight_count_correct() {
        let mut s = new_skin_export("M");
        add_weight(&mut s, 0, "spine", 0.5);
        add_weight(&mut s, 0, "hip", 0.5);
        assert_eq!(weight_count(&s), 2);
    }

    #[test]
    fn weight_value_stored() {
        let mut s = new_skin_export("M");
        add_weight(&mut s, 1, "arm", 0.75);
        assert!((s.weights[0].weight - 0.75).abs() < 1e-6);
    }

    #[test]
    fn vertex_idx_stored() {
        let mut s = new_skin_export("M");
        add_weight(&mut s, 42, "bone", 1.0);
        assert_eq!(s.weights[0].vertex_idx, 42);
    }

    #[test]
    fn bone_name_stored() {
        let mut s = new_skin_export("M");
        add_weight(&mut s, 0, "leftArm", 1.0);
        assert_eq!(s.weights[0].bone_name, "leftArm");
    }

    #[test]
    fn export_json_contains_mesh() {
        let s = new_skin_export("BodyMesh");
        let j = export_skin_to_json(&s);
        assert!(j.contains("BodyMesh"));
    }

    #[test]
    fn export_json_contains_bone() {
        let mut s = new_skin_export("M");
        add_weight(&mut s, 0, "rightShoulder", 1.0);
        let j = export_skin_to_json(&s);
        assert!(j.contains("rightShoulder"));
    }

    #[test]
    fn export_json_empty_weights() {
        let s = new_skin_export("X");
        let j = export_skin_to_json(&s);
        assert!(j.contains("weights"));
    }
}
