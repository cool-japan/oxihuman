// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Universal Scene Description (USD) export stub.

// ── Enums ────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum UsdUpAxis {
    Y,
    Z,
}

// ── Structs ──────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UsdExportConfig {
    pub version: String,
    pub up_axis: UsdUpAxis,
    pub meters_per_unit: f32,
    pub export_materials: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UsdPrim {
    pub path: String,
    pub prim_type: String,
    pub attributes: Vec<(String, String)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UsdExportScene {
    pub prims: Vec<UsdPrim>,
    pub root_path: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UsdExportResult {
    pub usda_string: String,
    pub prim_count: usize,
    pub success: bool,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_usd_config() -> UsdExportConfig {
    UsdExportConfig {
        version: "1.0".to_string(),
        up_axis: UsdUpAxis::Y,
        meters_per_unit: 1.0,
        export_materials: true,
    }
}

#[allow(dead_code)]
pub fn new_usd_export_scene(root: &str) -> UsdExportScene {
    UsdExportScene {
        prims: Vec::new(),
        root_path: root.to_string(),
    }
}

#[allow(dead_code)]
pub fn add_prim(scene: &mut UsdExportScene, prim: UsdPrim) {
    scene.prims.push(prim);
}

#[allow(dead_code)]
pub fn new_usd_prim(path: &str, prim_type: &str) -> UsdPrim {
    UsdPrim {
        path: path.to_string(),
        prim_type: prim_type.to_string(),
        attributes: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_attribute(prim: &mut UsdPrim, name: &str, value: &str) {
    prim.attributes.push((name.to_string(), value.to_string()));
}

#[allow(dead_code)]
pub fn scene_to_usda(scene: &UsdExportScene, cfg: &UsdExportConfig) -> UsdExportResult {
    let mut out = usda_header(cfg);
    for prim in &scene.prims {
        out.push_str(&usda_prim_block(prim));
    }
    let count = prim_count_usd(scene);
    UsdExportResult {
        usda_string: out,
        prim_count: count,
        success: true,
    }
}

#[allow(dead_code)]
pub fn usda_header(cfg: &UsdExportConfig) -> String {
    format!(
        "#usda {}\n(\n    upAxis = \"{}\"\n    metersPerUnit = {}\n)\n\n",
        cfg.version,
        up_axis_name(cfg),
        cfg.meters_per_unit
    )
}

#[allow(dead_code)]
pub fn usda_prim_block(prim: &UsdPrim) -> String {
    let mut out = format!("def {} \"{}\" {{\n", prim.prim_type, prim.path);
    for (name, value) in &prim.attributes {
        out.push_str(&format!("    {} = {}\n", name, value));
    }
    out.push_str("}\n\n");
    out
}

#[allow(dead_code)]
pub fn prim_count_usd(scene: &UsdExportScene) -> usize {
    scene.prims.len()
}

#[allow(dead_code)]
pub fn up_axis_name(cfg: &UsdExportConfig) -> &'static str {
    match cfg.up_axis {
        UsdUpAxis::Y => "Y",
        UsdUpAxis::Z => "Z",
    }
}

#[allow(dead_code)]
pub fn validate_usd(result: &UsdExportResult) -> bool {
    result.success && !result.usda_string.is_empty()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_usd_config() {
        let cfg = default_usd_config();
        assert_eq!(cfg.version, "1.0");
        assert_eq!(cfg.up_axis, UsdUpAxis::Y);
        assert!((cfg.meters_per_unit - 1.0).abs() < 1e-6);
        assert!(cfg.export_materials);
    }

    #[test]
    fn test_new_usd_export_scene() {
        let scene = new_usd_export_scene("/World");
        assert_eq!(scene.root_path, "/World");
        assert!(scene.prims.is_empty());
    }

    #[test]
    fn test_add_prim() {
        let mut scene = new_usd_export_scene("/World");
        let prim = new_usd_prim("Mesh", "Mesh");
        add_prim(&mut scene, prim);
        assert_eq!(prim_count_usd(&scene), 1);
    }

    #[test]
    fn test_add_attribute() {
        let mut prim = new_usd_prim("Cube", "Cube");
        add_attribute(&mut prim, "xformOp:translate", "(0, 0, 0)");
        assert_eq!(prim.attributes.len(), 1);
        assert_eq!(prim.attributes[0].0, "xformOp:translate");
    }

    #[test]
    fn test_scene_to_usda_header() {
        let cfg = default_usd_config();
        let scene = new_usd_export_scene("/World");
        let result = scene_to_usda(&scene, &cfg);
        assert!(result.usda_string.contains("#usda"));
        assert!(result.usda_string.contains("upAxis"));
        assert!(result.success);
    }

    #[test]
    fn test_scene_to_usda_with_prim() {
        let cfg = default_usd_config();
        let mut scene = new_usd_export_scene("/World");
        let mut prim = new_usd_prim("Sphere", "Sphere");
        add_attribute(&mut prim, "radius", "1.0");
        add_prim(&mut scene, prim);
        let result = scene_to_usda(&scene, &cfg);
        assert_eq!(result.prim_count, 1);
        assert!(result.usda_string.contains("Sphere"));
        assert!(result.usda_string.contains("radius"));
    }

    #[test]
    fn test_up_axis_name_y() {
        let cfg = default_usd_config();
        assert_eq!(up_axis_name(&cfg), "Y");
    }

    #[test]
    fn test_up_axis_name_z() {
        let cfg = UsdExportConfig {
            version: "1.0".to_string(),
            up_axis: UsdUpAxis::Z,
            meters_per_unit: 0.01,
            export_materials: false,
        };
        assert_eq!(up_axis_name(&cfg), "Z");
    }

    #[test]
    fn test_validate_usd_success() {
        let cfg = default_usd_config();
        let scene = new_usd_export_scene("/World");
        let result = scene_to_usda(&scene, &cfg);
        assert!(validate_usd(&result));
    }

    #[test]
    fn test_validate_usd_failure() {
        let result = UsdExportResult {
            usda_string: String::new(),
            prim_count: 0,
            success: false,
        };
        assert!(!validate_usd(&result));
    }

    #[test]
    fn test_prim_count_multiple() {
        let mut scene = new_usd_export_scene("/World");
        add_prim(&mut scene, new_usd_prim("A", "Mesh"));
        add_prim(&mut scene, new_usd_prim("B", "Xform"));
        add_prim(&mut scene, new_usd_prim("C", "Camera"));
        assert_eq!(prim_count_usd(&scene), 3);
    }

    #[test]
    fn test_usda_prim_block_format() {
        let mut prim = new_usd_prim("MyMesh", "Mesh");
        add_attribute(&mut prim, "color", "(1, 0, 0)");
        let block = usda_prim_block(&prim);
        assert!(block.contains("def Mesh"));
        assert!(block.contains("color"));
        assert!(block.contains('}'));
    }
}
