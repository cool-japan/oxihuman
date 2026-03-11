#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export blend shapes (morph targets with metadata).

#[allow(dead_code)]
pub struct BlendShapeExport {
    pub name: String,
    pub category: String,
    pub value: f32,
    pub min_val: f32,
    pub max_val: f32,
    pub vertex_offsets: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub struct BlendShapesExport {
    pub shapes: Vec<BlendShapeExport>,
}

#[allow(dead_code)]
pub fn new_blend_shapes_export() -> BlendShapesExport {
    BlendShapesExport { shapes: Vec::new() }
}

#[allow(dead_code)]
pub fn add_blend_shape(
    exp: &mut BlendShapesExport,
    name: &str,
    cat: &str,
    val: f32,
    offsets: Vec<[f32; 3]>,
) {
    exp.shapes.push(BlendShapeExport {
        name: name.to_string(),
        category: cat.to_string(),
        value: val,
        min_val: 0.0,
        max_val: 1.0,
        vertex_offsets: offsets,
    });
}

#[allow(dead_code)]
pub fn export_blend_shapes_to_json(exp: &BlendShapesExport) -> String {
    let mut s = "{\"shapes\":[".to_string();
    for (i, sh) in exp.shapes.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"name\":\"{}\",\"category\":\"{}\",\"value\":{},\"min\":{},\"max\":{},\"offsets\":{}}}",
            sh.name, sh.category, sh.value, sh.min_val, sh.max_val, sh.vertex_offsets.len()
        ));
    }
    s.push_str("]}");
    s
}

#[allow(dead_code)]
pub fn shape_count(exp: &BlendShapesExport) -> usize {
    exp.shapes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_offsets() -> Vec<[f32; 3]> {
        vec![[0.1, 0.0, 0.0], [0.0, 0.1, 0.0]]
    }

    #[test]
    fn new_export_empty() {
        let exp = new_blend_shapes_export();
        assert_eq!(shape_count(&exp), 0);
    }

    #[test]
    fn add_shape_increases_count() {
        let mut exp = new_blend_shapes_export();
        add_blend_shape(&mut exp, "smile", "mouth", 0.5, sample_offsets());
        assert_eq!(shape_count(&exp), 1);
    }

    #[test]
    fn shape_name_preserved() {
        let mut exp = new_blend_shapes_export();
        add_blend_shape(&mut exp, "brow_up", "brow", 0.0, vec![]);
        assert_eq!(exp.shapes[0].name, "brow_up");
    }

    #[test]
    fn shape_category_preserved() {
        let mut exp = new_blend_shapes_export();
        add_blend_shape(&mut exp, "x", "face", 0.0, vec![]);
        assert_eq!(exp.shapes[0].category, "face");
    }

    #[test]
    fn shape_value_stored() {
        let mut exp = new_blend_shapes_export();
        add_blend_shape(&mut exp, "x", "y", 0.75, vec![]);
        assert!((exp.shapes[0].value - 0.75).abs() < 1e-6);
    }

    #[test]
    fn default_min_max() {
        let mut exp = new_blend_shapes_export();
        add_blend_shape(&mut exp, "x", "y", 0.5, vec![]);
        assert!((exp.shapes[0].min_val - 0.0).abs() < 1e-6);
        assert!((exp.shapes[0].max_val - 1.0).abs() < 1e-6);
    }

    #[test]
    fn json_contains_shapes_key() {
        let exp = new_blend_shapes_export();
        let json = export_blend_shapes_to_json(&exp);
        assert!(json.contains("shapes"));
    }

    #[test]
    fn json_contains_shape_name() {
        let mut exp = new_blend_shapes_export();
        add_blend_shape(&mut exp, "jaw_open", "jaw", 1.0, sample_offsets());
        let json = export_blend_shapes_to_json(&exp);
        assert!(json.contains("jaw_open"));
    }

    #[test]
    fn vertex_offsets_stored() {
        let mut exp = new_blend_shapes_export();
        add_blend_shape(&mut exp, "x", "y", 0.0, sample_offsets());
        assert_eq!(exp.shapes[0].vertex_offsets.len(), 2);
    }

    #[test]
    fn multiple_shapes() {
        let mut exp = new_blend_shapes_export();
        add_blend_shape(&mut exp, "a", "cat", 0.0, vec![]);
        add_blend_shape(&mut exp, "b", "cat", 0.0, vec![]);
        assert_eq!(shape_count(&exp), 2);
    }
}
