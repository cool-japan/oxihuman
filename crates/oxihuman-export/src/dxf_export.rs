// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! AutoCAD DXF format export stub for 2D cross-sections and wireframes.

// ── Enums ────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum DxfUnits {
    Millimeters,
    Centimeters,
    Meters,
    Inches,
}

// ── Structs ──────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DxfExportConfig {
    pub version: String,
    pub units: DxfUnits,
    pub layer_name: String,
    pub color_index: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DxfPolyline {
    pub vertices: Vec<[f32; 2]>,
    pub closed: bool,
    pub layer: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DxfScene {
    pub polylines: Vec<DxfPolyline>,
    pub points: Vec<[f32; 2]>,
    pub layer: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DxfExportResult {
    pub dxf_string: String,
    pub entity_count: usize,
    pub layer_count: usize,
}

// ── Functions ────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_dxf_config() -> DxfExportConfig {
    DxfExportConfig {
        version: "AC1015".to_string(),
        units: DxfUnits::Millimeters,
        layer_name: "0".to_string(),
        color_index: 7,
    }
}

#[allow(dead_code)]
pub fn new_dxf_scene() -> DxfScene {
    DxfScene {
        polylines: Vec::new(),
        points: Vec::new(),
        layer: "0".to_string(),
    }
}

#[allow(dead_code)]
pub fn add_polyline(scene: &mut DxfScene, poly: DxfPolyline) {
    scene.polylines.push(poly);
}

#[allow(dead_code)]
pub fn add_point_to_scene(scene: &mut DxfScene, pt: [f32; 2]) {
    scene.points.push(pt);
}

#[allow(dead_code)]
pub fn new_dxf_polyline(vertices: Vec<[f32; 2]>, closed: bool) -> DxfPolyline {
    DxfPolyline {
        vertices,
        closed,
        layer: "0".to_string(),
    }
}

#[allow(dead_code)]
pub fn scene_to_dxf(scene: &DxfScene, cfg: &DxfExportConfig) -> DxfExportResult {
    let mut body = String::new();
    body.push_str(&dxf_header(cfg));
    body.push_str("0\nSECTION\n2\nENTITIES\n");

    for poly in &scene.polylines {
        body.push_str(&dxf_polyline_string(poly, cfg));
    }

    // POINT entities
    for pt in &scene.points {
        body.push_str(&format!(
            "0\nPOINT\n8\n{}\n10\n{:.6}\n20\n{:.6}\n30\n0.0\n",
            cfg.layer_name, pt[0], pt[1]
        ));
    }

    body.push_str("0\nENDSEC\n");
    body.push_str(&dxf_footer());

    let entity_count = entity_count_dxf(scene);
    DxfExportResult {
        dxf_string: body,
        entity_count,
        layer_count: 1,
    }
}

#[allow(dead_code)]
pub fn dxf_header(cfg: &DxfExportConfig) -> String {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n9\n$INSUNITS\n70\n{}\n0\nENDSEC\n",
        cfg.version,
        dxf_units_code(cfg),
    )
}

#[allow(dead_code)]
pub fn dxf_polyline_string(poly: &DxfPolyline, cfg: &DxfExportConfig) -> String {
    if poly.vertices.len() < 2 {
        return String::new();
    }
    let mut out = String::new();
    let n = poly.vertices.len();
    let pairs = if poly.closed { n } else { n - 1 };
    for i in 0..pairs {
        let a = poly.vertices[i];
        let b = poly.vertices[(i + 1) % n];
        out.push_str(&format!(
            "0\nLINE\n8\n{}\n62\n{}\n10\n{:.6}\n20\n{:.6}\n30\n0.0\n11\n{:.6}\n21\n{:.6}\n31\n0.0\n",
            cfg.layer_name, cfg.color_index,
            a[0], a[1], b[0], b[1]
        ));
    }
    out
}

#[allow(dead_code)]
pub fn dxf_footer() -> String {
    "0\nEOF\n".to_string()
}

#[allow(dead_code)]
pub fn entity_count_dxf(scene: &DxfScene) -> usize {
    let line_count: usize = scene
        .polylines
        .iter()
        .map(|p| {
            if p.vertices.len() < 2 {
                0
            } else if p.closed {
                p.vertices.len()
            } else {
                p.vertices.len() - 1
            }
        })
        .sum();
    line_count + scene.points.len()
}

#[allow(dead_code)]
pub fn dxf_units_name(cfg: &DxfExportConfig) -> &'static str {
    match cfg.units {
        DxfUnits::Millimeters => "millimeters",
        DxfUnits::Centimeters => "centimeters",
        DxfUnits::Meters => "meters",
        DxfUnits::Inches => "inches",
    }
}

#[allow(dead_code)]
pub fn validate_dxf(result: &DxfExportResult) -> bool {
    result.dxf_string.contains("EOF") && !result.dxf_string.is_empty()
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn dxf_units_code(cfg: &DxfExportConfig) -> u8 {
    match cfg.units {
        DxfUnits::Millimeters => 4,
        DxfUnits::Centimeters => 5,
        DxfUnits::Meters => 6,
        DxfUnits::Inches => 1,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_version() {
        let cfg = default_dxf_config();
        assert_eq!(cfg.version, "AC1015");
        assert_eq!(cfg.units, DxfUnits::Millimeters);
    }

    #[test]
    fn new_scene_is_empty() {
        let scene = new_dxf_scene();
        assert!(scene.polylines.is_empty());
        assert!(scene.points.is_empty());
    }

    #[test]
    fn add_polyline_increases_count() {
        let mut scene = new_dxf_scene();
        let poly = new_dxf_polyline(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], false);
        add_polyline(&mut scene, poly);
        assert_eq!(scene.polylines.len(), 1);
    }

    #[test]
    fn add_point_increases_count() {
        let mut scene = new_dxf_scene();
        add_point_to_scene(&mut scene, [3.0, 4.0]);
        assert_eq!(scene.points.len(), 1);
    }

    #[test]
    fn scene_to_dxf_contains_eof() {
        let scene = new_dxf_scene();
        let cfg = default_dxf_config();
        let result = scene_to_dxf(&scene, &cfg);
        assert!(result.dxf_string.contains("EOF"));
    }

    #[test]
    fn scene_to_dxf_entity_count_open_polyline() {
        let mut scene = new_dxf_scene();
        // 3 vertices, open => 2 LINE entities
        let poly = new_dxf_polyline(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], false);
        add_polyline(&mut scene, poly);
        let cfg = default_dxf_config();
        let result = scene_to_dxf(&scene, &cfg);
        assert_eq!(result.entity_count, 2);
    }

    #[test]
    fn scene_to_dxf_entity_count_closed_polyline() {
        let mut scene = new_dxf_scene();
        // 3 vertices, closed => 3 LINE entities
        let poly = new_dxf_polyline(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]], true);
        add_polyline(&mut scene, poly);
        let cfg = default_dxf_config();
        let result = scene_to_dxf(&scene, &cfg);
        assert_eq!(result.entity_count, 3);
    }

    #[test]
    fn dxf_footer_is_correct() {
        assert_eq!(dxf_footer(), "0\nEOF\n");
    }

    #[test]
    fn dxf_units_name_all_variants() {
        let mut cfg = default_dxf_config();
        cfg.units = DxfUnits::Millimeters;
        assert_eq!(dxf_units_name(&cfg), "millimeters");
        cfg.units = DxfUnits::Centimeters;
        assert_eq!(dxf_units_name(&cfg), "centimeters");
        cfg.units = DxfUnits::Meters;
        assert_eq!(dxf_units_name(&cfg), "meters");
        cfg.units = DxfUnits::Inches;
        assert_eq!(dxf_units_name(&cfg), "inches");
    }

    #[test]
    fn validate_dxf_empty_scene() {
        let scene = new_dxf_scene();
        let cfg = default_dxf_config();
        let result = scene_to_dxf(&scene, &cfg);
        assert!(validate_dxf(&result));
    }

    #[test]
    fn dxf_header_contains_version() {
        let cfg = default_dxf_config();
        let h = dxf_header(&cfg);
        assert!(h.contains("AC1015"));
    }
}
