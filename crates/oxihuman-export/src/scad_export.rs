// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! SCAD (generic) geometry export — a simpler, flat representation.

/// A SCAD primitive type.
#[derive(Clone, Debug)]
pub enum ScadPrimType {
    Box,
    Sphere,
    Cylinder,
    Mesh,
}

/// A generic SCAD primitive with transform.
#[derive(Clone, Debug)]
pub struct ScadPrimitive {
    pub prim_type: ScadPrimType,
    pub params: Vec<(String, f32)>,
    pub translation: [f32; 3],
}

/// A generic SCAD document.
#[derive(Clone, Debug, Default)]
pub struct ScadExport {
    pub primitives: Vec<ScadPrimitive>,
    pub metadata: Vec<(String, String)>,
}

/// Create a new SCAD export document.
pub fn new_scad_export() -> ScadExport {
    ScadExport::default()
}

/// Add a box primitive.
pub fn scad_add_box(doc: &mut ScadExport, size: [f32; 3], translation: [f32; 3]) {
    doc.primitives.push(ScadPrimitive {
        prim_type: ScadPrimType::Box,
        params: vec![
            ("sx".to_string(), size[0]),
            ("sy".to_string(), size[1]),
            ("sz".to_string(), size[2]),
        ],
        translation,
    });
}

/// Add a sphere primitive.
pub fn scad_add_sphere(doc: &mut ScadExport, radius: f32, translation: [f32; 3]) {
    doc.primitives.push(ScadPrimitive {
        prim_type: ScadPrimType::Sphere,
        params: vec![("r".to_string(), radius)],
        translation,
    });
}

/// Add a cylinder primitive.
pub fn scad_add_cylinder(doc: &mut ScadExport, height: f32, radius: f32, translation: [f32; 3]) {
    doc.primitives.push(ScadPrimitive {
        prim_type: ScadPrimType::Cylinder,
        params: vec![("h".to_string(), height), ("r".to_string(), radius)],
        translation,
    });
}

/// Add a metadata key-value pair.
pub fn scad_add_metadata(doc: &mut ScadExport, key: &str, value: &str) {
    doc.metadata.push((key.to_string(), value.to_string()));
}

/// Return the primitive count.
pub fn scad_prim_count(doc: &ScadExport) -> usize {
    doc.primitives.len()
}

/// Render the SCAD document to a string.
pub fn render_scad(doc: &ScadExport) -> String {
    let mut out = String::new();
    for (k, v) in &doc.metadata {
        out.push_str(&format!("// {}: {}\n", k, v));
    }
    for prim in &doc.primitives {
        let t = &prim.translation;
        let translate = format!("translate([{:.4},{:.4},{:.4}])", t[0], t[1], t[2]);
        let shape = match prim.prim_type {
            ScadPrimType::Box => {
                let sx = prim
                    .params
                    .iter()
                    .find(|(k, _)| k == "sx")
                    .map(|(_, v)| *v)
                    .unwrap_or(1.0);
                let sy = prim
                    .params
                    .iter()
                    .find(|(k, _)| k == "sy")
                    .map(|(_, v)| *v)
                    .unwrap_or(1.0);
                let sz = prim
                    .params
                    .iter()
                    .find(|(k, _)| k == "sz")
                    .map(|(_, v)| *v)
                    .unwrap_or(1.0);
                format!("cube([{:.4},{:.4},{:.4}])", sx, sy, sz)
            }
            ScadPrimType::Sphere => {
                let r = prim
                    .params
                    .iter()
                    .find(|(k, _)| k == "r")
                    .map(|(_, v)| *v)
                    .unwrap_or(1.0);
                format!("sphere(r={:.4})", r)
            }
            ScadPrimType::Cylinder => {
                let h = prim
                    .params
                    .iter()
                    .find(|(k, _)| k == "h")
                    .map(|(_, v)| *v)
                    .unwrap_or(1.0);
                let r = prim
                    .params
                    .iter()
                    .find(|(k, _)| k == "r")
                    .map(|(_, v)| *v)
                    .unwrap_or(1.0);
                format!("cylinder(h={:.4},r={:.4})", h, r)
            }
            ScadPrimType::Mesh => "polyhedron()".to_string(),
        };
        out.push_str(&format!("{} {};\n", translate, shape));
    }
    out
}

/// Validate the SCAD export document.
pub fn validate_scad(doc: &ScadExport) -> bool {
    doc.primitives
        .iter()
        .all(|p| !p.params.is_empty() || matches!(p.prim_type, ScadPrimType::Mesh))
}

/// Estimate the approximate bounding box of all primitives.
pub fn scad_bounding_box(doc: &ScadExport) -> ([f32; 3], [f32; 3]) {
    if doc.primitives.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for prim in &doc.primitives {
        let t = prim.translation;
        for k in 0..3 {
            min[k] = min[k].min(t[k]);
            max[k] = max[k].max(t[k]);
        }
    }
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scad_empty() {
        let d = new_scad_export();
        assert_eq!(scad_prim_count(&d), 0);
    }

    #[test]
    fn add_sphere_increments_count() {
        let mut d = new_scad_export();
        scad_add_sphere(&mut d, 1.0, [0.0, 0.0, 0.0]);
        assert_eq!(scad_prim_count(&d), 1);
    }

    #[test]
    fn add_box_increments_count() {
        let mut d = new_scad_export();
        scad_add_box(&mut d, [1.0, 2.0, 3.0], [0.0, 0.0, 0.0]);
        assert_eq!(scad_prim_count(&d), 1);
    }

    #[test]
    fn render_contains_translate() {
        let mut d = new_scad_export();
        scad_add_sphere(&mut d, 1.0, [1.0, 2.0, 3.0]);
        let s = render_scad(&d);
        assert!(s.contains("translate"));
    }

    #[test]
    fn render_cylinder_contains_cylinder() {
        let mut d = new_scad_export();
        scad_add_cylinder(&mut d, 2.0, 0.5, [0.0, 0.0, 0.0]);
        let s = render_scad(&d);
        assert!(s.contains("cylinder"));
    }

    #[test]
    fn metadata_in_render() {
        let mut d = new_scad_export();
        scad_add_metadata(&mut d, "author", "test");
        let s = render_scad(&d);
        assert!(s.contains("author"));
    }

    #[test]
    fn bounding_box_min_lte_max() {
        let mut d = new_scad_export();
        scad_add_sphere(&mut d, 1.0, [-1.0, -2.0, 0.0]);
        scad_add_sphere(&mut d, 1.0, [1.0, 2.0, 3.0]);
        let (mn, mx) = scad_bounding_box(&d);
        for k in 0..3 {
            assert!(mn[k] <= mx[k]);
        }
    }

    #[test]
    fn validate_empty_params_mesh() {
        let d = new_scad_export();
        assert!(validate_scad(&d));
    }
}
