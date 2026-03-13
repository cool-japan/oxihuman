// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Babylon.js scene export.

/// A Babylon.js mesh descriptor.
#[derive(Debug, Clone)]
pub struct BabylonMesh {
    pub name: String,
    pub id: String,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scaling: [f32; 3],
    pub material_id: Option<String>,
}

impl BabylonMesh {
    pub fn new(name: &str, id: &str) -> Self {
        Self {
            name: name.to_string(),
            id: id.to_string(),
            position: [0.0; 3],
            rotation: [0.0; 3],
            scaling: [1.0; 3],
            material_id: None,
        }
    }

    pub fn at(mut self, pos: [f32; 3]) -> Self {
        self.position = pos;
        self
    }

    pub fn with_material(mut self, mat_id: &str) -> Self {
        self.material_id = Some(mat_id.to_string());
        self
    }
}

/// A Babylon.js material.
#[derive(Debug, Clone)]
pub struct BabylonMaterial {
    pub name: String,
    pub id: String,
    pub diffuse_color: [f32; 4],
}

impl BabylonMaterial {
    pub fn new(name: &str, id: &str) -> Self {
        Self {
            name: name.to_string(),
            id: id.to_string(),
            diffuse_color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Babylon.js scene export.
#[derive(Debug, Clone, Default)]
pub struct BabylonExport {
    pub meshes: Vec<BabylonMesh>,
    pub materials: Vec<BabylonMaterial>,
    pub gravity: [f32; 3],
}

impl BabylonExport {
    pub fn new() -> Self {
        Self {
            gravity: [0.0, -9.81, 0.0],
            ..Default::default()
        }
    }

    pub fn add_mesh(&mut self, mesh: BabylonMesh) {
        self.meshes.push(mesh);
    }

    pub fn add_material(&mut self, mat: BabylonMaterial) {
        self.materials.push(mat);
    }
}

/// Serialize to Babylon.js scene JSON.
pub fn to_babylon_json(d: &BabylonExport) -> String {
    let mut meshes_json = String::new();
    for (i, m) in d.meshes.iter().enumerate() {
        if i > 0 {
            meshes_json.push(',');
        }
        let mat = m.material_id.as_deref().unwrap_or("");
        meshes_json.push_str(&format!(
            "{{\"name\":\"{}\",\"id\":\"{}\",\"position\":[{},{},{}],\"materialId\":\"{}\"}}",
            m.name, m.id, m.position[0], m.position[1], m.position[2], mat
        ));
    }
    let mut mats_json = String::new();
    for (i, mat) in d.materials.iter().enumerate() {
        if i > 0 {
            mats_json.push(',');
        }
        mats_json.push_str(&format!(
            "{{\"name\":\"{}\",\"id\":\"{}\"}}",
            mat.name, mat.id
        ));
    }
    format!(
        "{{\"scene\":{{\"gravity\":[{},{},{}],\"meshes\":[{}],\"materials\":[{}]}}}}",
        d.gravity[0], d.gravity[1], d.gravity[2], meshes_json, mats_json
    )
}

/// Count meshes.
pub fn babylon_mesh_count(d: &BabylonExport) -> usize {
    d.meshes.len()
}

/// Count materials.
pub fn babylon_material_count(d: &BabylonExport) -> usize {
    d.materials.len()
}

/// Create a new Babylon export.
pub fn new_babylon_export() -> BabylonExport {
    BabylonExport::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_babylon_export() {
        let d = new_babylon_export();
        assert_eq!(babylon_mesh_count(&d), 0);
    }

    #[test]
    fn test_add_mesh() {
        let mut d = BabylonExport::new();
        d.add_mesh(BabylonMesh::new("Cube", "c1"));
        assert_eq!(babylon_mesh_count(&d), 1);
    }

    #[test]
    fn test_add_material() {
        let mut d = BabylonExport::new();
        d.add_material(BabylonMaterial::new("Mat", "m1"));
        assert_eq!(babylon_material_count(&d), 1);
    }

    #[test]
    fn test_to_babylon_json_structure() {
        let d = BabylonExport::new();
        let s = to_babylon_json(&d);
        assert!(s.contains("scene"));
        assert!(s.contains("gravity"));
    }

    #[test]
    fn test_to_babylon_json_has_mesh() {
        let mut d = BabylonExport::new();
        d.add_mesh(BabylonMesh::new("Sphere", "s1").at([1.0, 2.0, 3.0]));
        let s = to_babylon_json(&d);
        assert!(s.contains("Sphere"));
    }

    #[test]
    fn test_to_babylon_json_has_material() {
        let mut d = BabylonExport::new();
        d.add_material(BabylonMaterial::new("Gold", "g1"));
        let s = to_babylon_json(&d);
        assert!(s.contains("Gold"));
    }

    #[test]
    fn test_babylon_mesh_with_material() {
        let m = BabylonMesh::new("M", "m").with_material("mat1");
        assert_eq!(m.material_id.as_deref(), Some("mat1"));
    }

    #[test]
    fn test_gravity_default() {
        let d = BabylonExport::new();
        assert!((d.gravity[1] - (-9.81)).abs() < 1e-3);
    }

    #[test]
    fn test_babylon_material_default_diffuse() {
        let m = BabylonMaterial::new("White", "w");
        assert!((m.diffuse_color[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_babylon_json_multiple_meshes() {
        let mut d = BabylonExport::new();
        d.add_mesh(BabylonMesh::new("A", "a"));
        d.add_mesh(BabylonMesh::new("B", "b"));
        let s = to_babylon_json(&d);
        assert!(s.contains("\"name\":\"A\""));
        assert!(s.contains("\"name\":\"B\""));
    }
}
