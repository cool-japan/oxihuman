// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Grease Pencil stroke export.

/* ── legacy API (kept) ── */

#[derive(Debug, Clone)]
pub struct GpStroke {
    pub points: Vec<[f32; 3]>,
    pub pressure: Vec<f32>,
    pub color: [f32; 4],
    pub line_width: f32,
}

#[derive(Debug, Clone)]
pub struct GpLayer {
    pub name: String,
    pub strokes: Vec<GpStroke>,
    pub opacity: f32,
}

#[derive(Debug, Clone)]
pub struct GreasePencilExport {
    pub layers: Vec<GpLayer>,
}

pub fn new_grease_pencil_export() -> GreasePencilExport {
    GreasePencilExport { layers: Vec::new() }
}

/* ── spec functions (wave 150B) ── */

/// Create a new blank GP stroke.
pub fn new_gp_stroke(color: [f32; 4], line_width: f32) -> GpStroke {
    GpStroke {
        points: Vec::new(),
        pressure: Vec::new(),
        color,
        line_width,
    }
}

/// Push a 3D point onto a stroke.
pub fn gp_push_point(stroke: &mut GpStroke, point: [f32; 3], pressure: f32) {
    stroke.points.push(point);
    stroke.pressure.push(pressure);
}

/// Polyline length of a stroke.
pub fn gp_stroke_length(stroke: &GpStroke) -> f32 {
    if stroke.points.len() < 2 {
        return 0.0;
    }
    stroke
        .points
        .windows(2)
        .map(|w| {
            let a = w[0];
            let b = w[1];
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let dz = b[2] - a[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum()
}

/// Serialize a single stroke to JSON.
pub fn gp_stroke_to_json(s: &GpStroke) -> String {
    format!(
        "{{\"points\":{},\"width\":{}}}",
        s.points.len(),
        s.line_width
    )
}

/// Serialize multiple strokes to a JSON array.
pub fn gp_strokes_to_json(strokes: &[GpStroke]) -> String {
    let inner: Vec<String> = strokes.iter().map(gp_stroke_to_json).collect();
    format!("[{}]", inner.join(","))
}

/// Number of points in a stroke.
pub fn gp_point_count(stroke: &GpStroke) -> usize {
    stroke.points.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gp_stroke() {
        let s = new_gp_stroke([1.0, 0.0, 0.0, 1.0], 2.0);
        assert_eq!(gp_point_count(&s), 0);
    }

    #[test]
    fn test_gp_push_point() {
        let mut s = new_gp_stroke([1.0, 1.0, 1.0, 1.0], 1.0);
        gp_push_point(&mut s, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(gp_point_count(&s), 1);
    }

    #[test]
    fn test_gp_stroke_length() {
        let mut s = new_gp_stroke([1.0, 1.0, 1.0, 1.0], 1.0);
        gp_push_point(&mut s, [0.0, 0.0, 0.0], 1.0);
        gp_push_point(&mut s, [1.0, 0.0, 0.0], 1.0);
        assert!((gp_stroke_length(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_gp_stroke_to_json() {
        let s = new_gp_stroke([0.0, 0.0, 0.0, 1.0], 1.5);
        let j = gp_stroke_to_json(&s);
        assert!(j.contains("points"));
    }

    #[test]
    fn test_gp_strokes_to_json() {
        let s = new_gp_stroke([0.0, 0.0, 0.0, 1.0], 1.0);
        let j = gp_strokes_to_json(&[s]);
        assert!(j.starts_with('['));
    }
}
