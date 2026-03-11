// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Material library serialization and management.

// ── Types ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum MatAlphaMode {
    Opaque,
    Mask,
    Blend,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct PbrMaterialDef {
    pub name: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],
    pub alpha_mode: MatAlphaMode,
    pub double_sided: bool,
    pub texture_paths: Vec<String>,
}

#[allow(dead_code)]
pub struct MatLibrary {
    pub name: String,
    pub materials: Vec<PbrMaterialDef>,
    pub version: u32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn new_material_library(name: &str) -> MatLibrary {
    MatLibrary {
        name: name.to_string(),
        materials: Vec::new(),
        version: 1,
    }
}

#[allow(dead_code)]
pub fn add_material(lib: &mut MatLibrary, mat: PbrMaterialDef) -> usize {
    let idx = lib.materials.len();
    lib.materials.push(mat);
    idx
}

#[allow(dead_code)]
pub fn get_material<'a>(lib: &'a MatLibrary, name: &str) -> Option<&'a PbrMaterialDef> {
    lib.materials.iter().find(|m| m.name == name)
}

#[allow(dead_code)]
pub fn default_pbr_material(name: &str) -> PbrMaterialDef {
    PbrMaterialDef {
        name: name.to_string(),
        base_color: [0.5, 0.5, 0.5, 1.0],
        metallic: 0.0,
        roughness: 0.5,
        emissive: [0.0, 0.0, 0.0],
        alpha_mode: MatAlphaMode::Opaque,
        double_sided: false,
        texture_paths: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn serialize_library_json(lib: &MatLibrary) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{{\"name\":\"{}\",\"version\":{},\"materials\":[",
        lib.name, lib.version
    ));
    for (i, m) in lib.materials.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let bc = m.base_color;
        let alpha_str = match m.alpha_mode {
            MatAlphaMode::Opaque => "Opaque",
            MatAlphaMode::Mask => "Mask",
            MatAlphaMode::Blend => "Blend",
        };
        out.push_str(&format!(
            "{{\"name\":\"{}\",\"base_color\":[{},{},{},{}],\"metallic\":{},\"roughness\":{},\"emissive\":[{},{},{}],\"alpha_mode\":\"{}\",\"double_sided\":{}}}",
            m.name,
            bc[0], bc[1], bc[2], bc[3],
            m.metallic,
            m.roughness,
            m.emissive[0], m.emissive[1], m.emissive[2],
            alpha_str,
            m.double_sided
        ));
    }
    out.push_str("]}");
    out
}

#[allow(dead_code)]
pub fn deserialize_library_json(_json: &str) -> Option<MatLibrary> {
    // Stub — full JSON parsing without external deps is out of scope
    None
}

#[allow(dead_code)]
pub fn blend_materials(a: &PbrMaterialDef, b: &PbrMaterialDef, t: f32) -> PbrMaterialDef {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    PbrMaterialDef {
        name: format!("{}_blend_{}", a.name, b.name),
        base_color: [
            lerp(a.base_color[0], b.base_color[0]),
            lerp(a.base_color[1], b.base_color[1]),
            lerp(a.base_color[2], b.base_color[2]),
            lerp(a.base_color[3], b.base_color[3]),
        ],
        metallic: lerp(a.metallic, b.metallic),
        roughness: lerp(a.roughness, b.roughness),
        emissive: [
            lerp(a.emissive[0], b.emissive[0]),
            lerp(a.emissive[1], b.emissive[1]),
            lerp(a.emissive[2], b.emissive[2]),
        ],
        alpha_mode: if t < 0.5 {
            a.alpha_mode.clone()
        } else {
            b.alpha_mode.clone()
        },
        double_sided: if t < 0.5 {
            a.double_sided
        } else {
            b.double_sided
        },
        texture_paths: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn material_is_transparent(mat: &PbrMaterialDef) -> bool {
    matches!(mat.alpha_mode, MatAlphaMode::Blend)
}

#[allow(dead_code)]
pub fn count_textured(lib: &MatLibrary) -> usize {
    lib.materials
        .iter()
        .filter(|m| !m.texture_paths.is_empty())
        .count()
}

#[allow(dead_code)]
pub fn remove_material(lib: &mut MatLibrary, name: &str) -> bool {
    let before = lib.materials.len();
    lib.materials.retain(|m| m.name != name);
    lib.materials.len() < before
}

#[allow(dead_code)]
pub fn list_names(lib: &MatLibrary) -> Vec<&str> {
    lib.materials.iter().map(|m| m.name.as_str()).collect()
}

#[allow(dead_code)]
pub fn material_roughness_category(mat: &PbrMaterialDef) -> &'static str {
    if mat.roughness >= 0.9 {
        "matte"
    } else if mat.roughness >= 0.5 {
        "satin"
    } else if mat.roughness >= 0.1 {
        "glossy"
    } else {
        "mirror"
    }
}

#[allow(dead_code)]
pub fn export_material_ids(lib: &MatLibrary) -> Vec<(usize, String)> {
    lib.materials
        .iter()
        .enumerate()
        .map(|(i, m)| (i, m.name.clone()))
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_library() {
        let lib = new_material_library("TestLib");
        assert_eq!(lib.name, "TestLib");
        assert!(lib.materials.is_empty());
        assert_eq!(lib.version, 1);
    }

    #[test]
    fn test_add_and_get_material() {
        let mut lib = new_material_library("Lib");
        let mat = default_pbr_material("Gray");
        let idx = add_material(&mut lib, mat);
        assert_eq!(idx, 0);
        let found = get_material(&lib, "Gray");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Gray");
    }

    #[test]
    fn test_get_missing_material() {
        let lib = new_material_library("Lib");
        assert!(get_material(&lib, "Missing").is_none());
    }

    #[test]
    fn test_add_multiple_materials() {
        let mut lib = new_material_library("Lib");
        let a = default_pbr_material("A");
        let b = default_pbr_material("B");
        let ia = add_material(&mut lib, a);
        let ib = add_material(&mut lib, b);
        assert_eq!(ia, 0);
        assert_eq!(ib, 1);
        assert_eq!(lib.materials.len(), 2);
    }

    #[test]
    fn test_serialize_nonempty() {
        let mut lib = new_material_library("Lib");
        let mat = default_pbr_material("Metal");
        add_material(&mut lib, mat);
        let json = serialize_library_json(&lib);
        assert!(!json.is_empty());
        assert!(json.contains("Metal"));
        assert!(json.contains("Lib"));
    }

    #[test]
    fn test_serialize_contains_fields() {
        let mut lib = new_material_library("MyLib");
        let mat = default_pbr_material("Base");
        add_material(&mut lib, mat);
        let json = serialize_library_json(&lib);
        assert!(json.contains("base_color"));
        assert!(json.contains("metallic"));
        assert!(json.contains("roughness"));
    }

    #[test]
    fn test_deserialize_stub_returns_none() {
        let result = deserialize_library_json("{\"name\":\"Lib\"}");
        assert!(result.is_none());
    }

    #[test]
    fn test_blend_materials() {
        let a = default_pbr_material("A");
        let mut b = default_pbr_material("B");
        b.metallic = 1.0;
        b.roughness = 1.0;
        let blended = blend_materials(&a, &b, 0.5);
        assert!((blended.metallic - 0.5).abs() < 1e-5);
        assert!((blended.roughness - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_blend_at_zero() {
        let a = default_pbr_material("A");
        let b = default_pbr_material("B");
        let blended = blend_materials(&a, &b, 0.0);
        assert!((blended.metallic - a.metallic).abs() < 1e-5);
    }

    #[test]
    fn test_material_is_transparent() {
        let mut mat = default_pbr_material("Glass");
        mat.alpha_mode = MatAlphaMode::Blend;
        assert!(material_is_transparent(&mat));
        let opaque = default_pbr_material("Opaque");
        assert!(!material_is_transparent(&opaque));
    }

    #[test]
    fn test_count_textured() {
        let mut lib = new_material_library("Lib");
        let mut mat = default_pbr_material("Textured");
        mat.texture_paths.push("tex.png".to_string());
        add_material(&mut lib, mat);
        add_material(&mut lib, default_pbr_material("Plain"));
        assert_eq!(count_textured(&lib), 1);
    }

    #[test]
    fn test_remove_material() {
        let mut lib = new_material_library("Lib");
        add_material(&mut lib, default_pbr_material("ToRemove"));
        add_material(&mut lib, default_pbr_material("Keep"));
        let removed = remove_material(&mut lib, "ToRemove");
        assert!(removed);
        assert_eq!(lib.materials.len(), 1);
        assert_eq!(lib.materials[0].name, "Keep");
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut lib = new_material_library("Lib");
        assert!(!remove_material(&mut lib, "Ghost"));
    }

    #[test]
    fn test_roughness_category() {
        let mut mat = default_pbr_material("M");
        mat.roughness = 0.95;
        assert_eq!(material_roughness_category(&mat), "matte");
        mat.roughness = 0.6;
        assert_eq!(material_roughness_category(&mat), "satin");
        mat.roughness = 0.3;
        assert_eq!(material_roughness_category(&mat), "glossy");
        mat.roughness = 0.05;
        assert_eq!(material_roughness_category(&mat), "mirror");
    }

    #[test]
    fn test_export_material_ids() {
        let mut lib = new_material_library("Lib");
        add_material(&mut lib, default_pbr_material("A"));
        add_material(&mut lib, default_pbr_material("B"));
        let ids = export_material_ids(&lib);
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], (0, "A".to_string()));
        assert_eq!(ids[1], (1, "B".to_string()));
    }

    #[test]
    fn test_list_names() {
        let mut lib = new_material_library("Lib");
        add_material(&mut lib, default_pbr_material("X"));
        add_material(&mut lib, default_pbr_material("Y"));
        let names = list_names(&lib);
        assert_eq!(names, vec!["X", "Y"]);
    }
}
