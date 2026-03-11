// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Draw material binding — associate draw calls with material descriptors.

/// Shading model.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadingModel {
    Unlit,
    Lambert,
    Pbr,
    Subsurface,
}

/// A lightweight material descriptor for a draw call.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawMaterial {
    pub id: u32,
    pub name: String,
    pub shading: ShadingModel,
    pub base_color: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
    pub double_sided: bool,
    pub alpha_cutoff: f32,
}

impl DrawMaterial {
    #[allow(dead_code)]
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            shading: ShadingModel::Pbr,
            base_color: [1.0, 1.0, 1.0, 1.0],
            roughness: 0.5,
            metallic: 0.0,
            double_sided: false,
            alpha_cutoff: 0.5,
        }
    }
}

/// Material library.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DrawMaterialLib {
    materials: Vec<DrawMaterial>,
}

#[allow(dead_code)]
pub fn new_draw_material_lib() -> DrawMaterialLib {
    DrawMaterialLib::default()
}

#[allow(dead_code)]
pub fn dml_add(lib: &mut DrawMaterialLib, mat: DrawMaterial) {
    lib.materials.push(mat);
}

#[allow(dead_code)]
pub fn dml_remove(lib: &mut DrawMaterialLib, id: u32) {
    lib.materials.retain(|m| m.id != id);
}

#[allow(dead_code)]
pub fn dml_get(lib: &DrawMaterialLib, id: u32) -> Option<&DrawMaterial> {
    lib.materials.iter().find(|m| m.id == id)
}

#[allow(dead_code)]
pub fn dml_count(lib: &DrawMaterialLib) -> usize {
    lib.materials.len()
}

#[allow(dead_code)]
pub fn dml_clear(lib: &mut DrawMaterialLib) {
    lib.materials.clear();
}

#[allow(dead_code)]
pub fn dml_set_roughness(lib: &mut DrawMaterialLib, id: u32, v: f32) {
    for m in lib.materials.iter_mut() {
        if m.id == id {
            m.roughness = v.clamp(0.0, 1.0);
        }
    }
}

#[allow(dead_code)]
pub fn dml_shading_name(s: ShadingModel) -> &'static str {
    match s {
        ShadingModel::Unlit => "unlit",
        ShadingModel::Lambert => "lambert",
        ShadingModel::Pbr => "pbr",
        ShadingModel::Subsurface => "subsurface",
    }
}

/// Check if a material uses alpha testing.
#[allow(dead_code)]
pub fn dml_uses_alpha_test(mat: &DrawMaterial) -> bool {
    (0.0..=1.0).contains(&mat.alpha_cutoff) && mat.base_color[3] < 1.0
}

#[allow(dead_code)]
pub fn dml_to_json(lib: &DrawMaterialLib) -> String {
    format!("{{\"count\":{}}}", lib.materials.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_lib() {
        assert_eq!(dml_count(&new_draw_material_lib()), 0);
    }

    #[test]
    fn add_and_get() {
        let mut lib = new_draw_material_lib();
        dml_add(&mut lib, DrawMaterial::new(1, "wood"));
        assert!(dml_get(&lib, 1).is_some());
    }

    #[test]
    fn remove_by_id() {
        let mut lib = new_draw_material_lib();
        dml_add(&mut lib, DrawMaterial::new(1, "test"));
        dml_remove(&mut lib, 1);
        assert_eq!(dml_count(&lib), 0);
    }

    #[test]
    fn get_missing_returns_none() {
        let lib = new_draw_material_lib();
        assert!(dml_get(&lib, 99).is_none());
    }

    #[test]
    fn clear_empties() {
        let mut lib = new_draw_material_lib();
        dml_add(&mut lib, DrawMaterial::new(1, "a"));
        dml_add(&mut lib, DrawMaterial::new(2, "b"));
        dml_clear(&mut lib);
        assert_eq!(dml_count(&lib), 0);
    }

    #[test]
    fn shading_name_pbr() {
        assert_eq!(dml_shading_name(ShadingModel::Pbr), "pbr");
    }

    #[test]
    fn set_roughness_clamps() {
        let mut lib = new_draw_material_lib();
        dml_add(&mut lib, DrawMaterial::new(1, "r"));
        dml_set_roughness(&mut lib, 1, 5.0);
        assert!((dml_get(&lib, 1).unwrap().roughness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn alpha_test_when_transparent() {
        let mut m = DrawMaterial::new(1, "t");
        m.base_color[3] = 0.5;
        m.alpha_cutoff = 0.5;
        assert!(dml_uses_alpha_test(&m));
    }

    #[test]
    fn no_alpha_test_when_opaque() {
        let m = DrawMaterial::new(1, "o");
        assert!(!dml_uses_alpha_test(&m));
    }

    #[test]
    fn json_has_count() {
        let j = dml_to_json(&new_draw_material_lib());
        assert!(j.contains("count"));
    }
}
