// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Surface normal visualization — generates line segments from vertex positions along normals.
//!
//! Supports both face normals and vertex normals, with configurable scale and color.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for normal visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NormalVisConfig {
    /// Length of the normal line segments in world units.
    pub scale: f32,
    /// RGB color of normal lines (linear, 0.0–1.0).
    pub color: [f32; 3],
    /// Whether face normals are shown.
    pub show_face_normals: bool,
    /// Whether vertex normals are shown.
    pub show_vertex_normals: bool,
    /// Whether the visualizer is enabled at all.
    pub enabled: bool,
}

/// A single normal visualization line from `start` to `end`.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalLine {
    /// Origin point (vertex or face centroid).
    pub start: [f32; 3],
    /// Tip point (start + normal × scale).
    pub end: [f32; 3],
}

/// Resolved draw call for normal visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NormalVisDrawCall {
    /// All normal line segments to render.
    pub lines: Vec<NormalLine>,
    /// A snapshot of the config at draw-call creation time.
    pub config: NormalVisConfig,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a sensible default `NormalVisConfig`.
#[allow(dead_code)]
pub fn default_normal_vis_config() -> NormalVisConfig {
    NormalVisConfig {
        scale: 0.1,
        color: [0.0, 1.0, 0.0],
        show_face_normals: false,
        show_vertex_normals: true,
        enabled: true,
    }
}

/// Compute per-face normals from vertex positions and triangle indices.
///
/// Each face normal is the cross-product of two edge vectors, normalized.
/// Degenerate triangles produce a zero normal `[0, 0, 0]`.
#[allow(dead_code)]
pub fn compute_face_normals(verts: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<[f32; 3]> {
    faces
        .iter()
        .map(|face| {
            let i0 = face[0] as usize;
            let i1 = face[1] as usize;
            let i2 = face[2] as usize;
            if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
                return [0.0_f32; 3];
            }
            let e1 = sub3(verts[i1], verts[i0]);
            let e2 = sub3(verts[i2], verts[i0]);
            normalize3(cross3(e1, e2))
        })
        .collect()
}

/// Compute per-vertex normals by averaging the face normals of incident triangles.
///
/// Vertices with no incident faces get normal `[0, 0, 0]`.
#[allow(dead_code)]
pub fn compute_vertex_normals(verts: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let mut accum = vec![[0.0_f32; 3]; verts.len()];
    let mut count = vec![0u32; verts.len()];

    for face in faces {
        let i0 = face[0] as usize;
        let i1 = face[1] as usize;
        let i2 = face[2] as usize;
        if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
            continue;
        }
        let e1 = sub3(verts[i1], verts[i0]);
        let e2 = sub3(verts[i2], verts[i0]);
        let n = cross3(e1, e2); // area-weighted, not yet normalized
        for &idx in &[i0, i1, i2] {
            accum[idx] = add3(accum[idx], n);
            count[idx] += 1;
        }
    }

    accum
        .iter()
        .zip(count.iter())
        .map(|(&n, &c)| if c == 0 { [0.0; 3] } else { normalize3(n) })
        .collect()
}

/// Build a `NormalVisDrawCall` from vertex positions, normals, and configuration.
///
/// Each normal line runs from the vertex position to `position + normal * scale`.
#[allow(dead_code)]
pub fn build_normal_lines(
    verts: &[[f32; 3]],
    normals: &[[f32; 3]],
    cfg: &NormalVisConfig,
) -> NormalVisDrawCall {
    let lines = verts
        .iter()
        .zip(normals.iter())
        .map(|(&v, &n)| {
            let end = add3(v, scale3(n, cfg.scale));
            NormalLine { start: v, end }
        })
        .collect();
    NormalVisDrawCall {
        lines,
        config: cfg.clone(),
    }
}

/// Return the number of normal lines in a draw call.
#[allow(dead_code)]
pub fn normal_line_count(call: &NormalVisDrawCall) -> usize {
    call.lines.len()
}

/// Set the scale (length) of normal line segments.
#[allow(dead_code)]
pub fn set_normal_scale(cfg: &mut NormalVisConfig, scale: f32) {
    cfg.scale = scale.max(0.0);
}

/// Set the RGB color of normal lines.
#[allow(dead_code)]
pub fn set_normal_color(cfg: &mut NormalVisConfig, r: f32, g: f32, b: f32) {
    cfg.color = [
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
    ];
}

/// Toggle face-normal display on/off.
#[allow(dead_code)]
pub fn normal_vis_toggle_face_normals(cfg: &mut NormalVisConfig) {
    cfg.show_face_normals = !cfg.show_face_normals;
}

/// Toggle vertex-normal display on/off.
#[allow(dead_code)]
pub fn normal_vis_toggle_vertex_normals(cfg: &mut NormalVisConfig) {
    cfg.show_vertex_normals = !cfg.show_vertex_normals;
}

/// Return `true` if the normal visualizer is enabled.
#[allow(dead_code)]
pub fn normal_vis_is_enabled(cfg: &NormalVisConfig) -> bool {
    cfg.enabled
}

// ── Private math helpers ──────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = dot3(v, v).sqrt();
    if len < 1e-10 {
        [0.0; 3]
    } else {
        scale3(v, 1.0 / len)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_verts() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }

    fn triangle_faces() -> Vec<[u32; 3]> {
        vec![[0, 1, 2]]
    }

    #[test]
    fn test_default_normal_vis_config() {
        let cfg = default_normal_vis_config();
        assert!((cfg.scale - 0.1).abs() < 1e-6);
        assert_eq!(cfg.color, [0.0, 1.0, 0.0]);
        assert!(cfg.show_vertex_normals);
        assert!(!cfg.show_face_normals);
        assert!(cfg.enabled);
    }

    #[test]
    fn test_compute_face_normals_count() {
        let verts = triangle_verts();
        let faces = triangle_faces();
        let normals = compute_face_normals(&verts, &faces);
        assert_eq!(normals.len(), faces.len());
    }

    #[test]
    fn test_compute_face_normals_pointing_z() {
        // Triangle in XY plane → normal should point along +Z.
        let verts = triangle_verts();
        let faces = triangle_faces();
        let normals = compute_face_normals(&verts, &faces);
        let n = normals[0];
        assert!(n[2].abs() > 0.9, "face normal should point along Z: {:?}", n);
    }

    #[test]
    fn test_compute_vertex_normals_count() {
        let verts = triangle_verts();
        let faces = triangle_faces();
        let normals = compute_vertex_normals(&verts, &faces);
        assert_eq!(normals.len(), verts.len());
    }

    #[test]
    fn test_compute_vertex_normals_normalized() {
        let verts = triangle_verts();
        let faces = triangle_faces();
        let normals = compute_vertex_normals(&verts, &faces);
        for n in &normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-5, "vertex normal not unit length: {:?}", n);
        }
    }

    #[test]
    fn test_build_normal_lines_count() {
        let verts = triangle_verts();
        let normals = compute_vertex_normals(&verts, &triangle_faces());
        let cfg = default_normal_vis_config();
        let call = build_normal_lines(&verts, &normals, &cfg);
        assert_eq!(normal_line_count(&call), verts.len());
    }

    #[test]
    fn test_build_normal_lines_offset_by_scale() {
        let verts = vec![[0.0_f32, 0.0, 0.0]];
        let normals = vec![[0.0_f32, 0.0, 1.0]]; // pointing +Z
        let mut cfg = default_normal_vis_config();
        cfg.scale = 1.0;
        let call = build_normal_lines(&verts, &normals, &cfg);
        let line = call.lines[0];
        assert!((line.end[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_normal_scale() {
        let mut cfg = default_normal_vis_config();
        set_normal_scale(&mut cfg, 0.5);
        assert!((cfg.scale - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_normal_scale_negative_clamped() {
        let mut cfg = default_normal_vis_config();
        set_normal_scale(&mut cfg, -2.0);
        assert!((cfg.scale - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_normal_color() {
        let mut cfg = default_normal_vis_config();
        set_normal_color(&mut cfg, 1.0, 0.5, 0.0);
        assert_eq!(cfg.color, [1.0, 0.5, 0.0]);
    }

    #[test]
    fn test_normal_vis_toggle_face_normals() {
        let mut cfg = default_normal_vis_config();
        assert!(!cfg.show_face_normals);
        normal_vis_toggle_face_normals(&mut cfg);
        assert!(cfg.show_face_normals);
    }

    #[test]
    fn test_normal_vis_toggle_vertex_normals() {
        let mut cfg = default_normal_vis_config();
        assert!(cfg.show_vertex_normals);
        normal_vis_toggle_vertex_normals(&mut cfg);
        assert!(!cfg.show_vertex_normals);
    }

    #[test]
    fn test_normal_vis_is_enabled() {
        let cfg = default_normal_vis_config();
        assert!(normal_vis_is_enabled(&cfg));
    }

    #[test]
    fn test_compute_face_normals_oob_indices() {
        let verts = vec![[0.0_f32; 3]];
        let faces = vec![[0u32, 5, 10]]; // out-of-bounds
        let normals = compute_face_normals(&verts, &faces);
        assert_eq!(normals.len(), 1);
        assert_eq!(normals[0], [0.0; 3]);
    }

    #[test]
    fn test_compute_vertex_normals_no_faces() {
        let verts = triangle_verts();
        let normals = compute_vertex_normals(&verts, &[]);
        assert_eq!(normals.len(), verts.len());
        for n in &normals {
            assert_eq!(*n, [0.0; 3]);
        }
    }
}
