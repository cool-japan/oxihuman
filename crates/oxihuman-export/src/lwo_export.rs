// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! LightWave Object (.lwo) format export (stub — text summary).

/// LWO surface definition.
#[derive(Clone, Debug)]
pub struct LwoSurface {
    pub name: String,
    pub color: [f32; 3],
    pub diffuse: f32,
    pub specular: f32,
}

impl Default for LwoSurface {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            color: [0.8, 0.8, 0.8],
            diffuse: 1.0,
            specular: 0.0,
        }
    }
}

/// LWO layer of geometry.
#[derive(Clone, Debug, Default)]
pub struct LwoLayer {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub polygons: Vec<Vec<u32>>,
}

/// LWO export document.
#[derive(Clone, Debug, Default)]
pub struct LwoExport {
    pub layers: Vec<LwoLayer>,
    pub surfaces: Vec<LwoSurface>,
}

/// Create a new LWO export.
pub fn new_lwo_export() -> LwoExport {
    LwoExport::default()
}

/// Add a layer with geometry from a triangle mesh.
pub fn lwo_add_layer(doc: &mut LwoExport, name: &str, positions: Vec<[f32; 3]>, indices: &[u32]) {
    let polygons: Vec<Vec<u32>> = indices.chunks(3).map(|t| t.to_vec()).collect();
    doc.layers.push(LwoLayer {
        name: name.to_string(),
        positions,
        polygons,
    });
}

/// Add a surface definition.
pub fn lwo_add_surface(doc: &mut LwoExport, surface: LwoSurface) {
    doc.surfaces.push(surface);
}

/// Return layer count.
pub fn lwo_layer_count(doc: &LwoExport) -> usize {
    doc.layers.len()
}

/// Return surface count.
pub fn lwo_surface_count(doc: &LwoExport) -> usize {
    doc.surfaces.len()
}

/// Return total vertex count.
pub fn lwo_total_vertex_count(doc: &LwoExport) -> usize {
    doc.layers.iter().map(|l| l.positions.len()).sum()
}

/// Return total polygon count.
pub fn lwo_total_polygon_count(doc: &LwoExport) -> usize {
    doc.layers.iter().map(|l| l.polygons.len()).sum()
}

/// Render the LWO export as a human-readable summary string (not binary).
pub fn render_lwo_summary(doc: &LwoExport) -> String {
    let mut out = String::from("# LWO Export Summary (oxihuman)\n");
    out.push_str(&format!("Layers: {}\n", doc.layers.len()));
    for layer in &doc.layers {
        out.push_str(&format!(
            "  Layer '{}': {} verts, {} polys\n",
            layer.name,
            layer.positions.len(),
            layer.polygons.len()
        ));
    }
    out.push_str(&format!("Surfaces: {}\n", doc.surfaces.len()));
    for surf in &doc.surfaces {
        out.push_str(&format!(
            "  Surface '{}': rgb({:.2},{:.2},{:.2})\n",
            surf.name, surf.color[0], surf.color[1], surf.color[2]
        ));
    }
    out
}

/// Build minimal binary LWO2 chunk header (FORM chunk).
pub fn lwo_form_header(content_size: u32) -> Vec<u8> {
    let mut out = b"FORM".to_vec();
    out.extend_from_slice(&content_size.to_be_bytes());
    out.extend_from_slice(b"LWO2");
    out
}

/// Estimate file size.
pub fn lwo_size_estimate(doc: &LwoExport) -> usize {
    let verts = lwo_total_vertex_count(doc);
    let polys = lwo_total_polygon_count(doc);
    12 + verts * 12 + polys * 12 + doc.surfaces.len() * 64
}

/// Validate the export.
pub fn validate_lwo(doc: &LwoExport) -> bool {
    for layer in &doc.layers {
        let n = layer.positions.len() as u32;
        for poly in &layer.polygons {
            for &i in poly {
                if i >= n {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple() -> LwoExport {
        let mut d = new_lwo_export();
        lwo_add_layer(
            &mut d,
            "Layer1",
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[0, 1, 2],
        );
        lwo_add_surface(&mut d, LwoSurface::default());
        d
    }

    #[test]
    fn layer_count() {
        let d = simple();
        assert_eq!(lwo_layer_count(&d), 1);
    }

    #[test]
    fn surface_count() {
        let d = simple();
        assert_eq!(lwo_surface_count(&d), 1);
    }

    #[test]
    fn total_vertex_count() {
        let d = simple();
        assert_eq!(lwo_total_vertex_count(&d), 3);
    }

    #[test]
    fn total_polygon_count() {
        let d = simple();
        assert_eq!(lwo_total_polygon_count(&d), 1);
    }

    #[test]
    fn render_summary_non_empty() {
        let d = simple();
        assert!(render_lwo_summary(&d).contains("Layer1"));
    }

    #[test]
    fn form_header_starts_with_form() {
        let h = lwo_form_header(100);
        assert_eq!(&h[0..4], b"FORM");
    }

    #[test]
    fn validate_valid() {
        let d = simple();
        assert!(validate_lwo(&d));
    }

    #[test]
    fn size_estimate_positive() {
        let d = simple();
        assert!(lwo_size_estimate(&d) > 0);
    }
}
