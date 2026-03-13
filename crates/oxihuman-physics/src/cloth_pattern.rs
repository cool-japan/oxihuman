// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! 2D garment pattern to 3D cloth simulation setup.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn len2(a: [f32; 2]) -> f32 {
    (a[0] * a[0] + a[1] * a[1]).sqrt()
}

fn sub2(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l < 1e-10 {
        [0.0, 0.0, 1.0]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

// ---------------------------------------------------------------------------
// data types
// ---------------------------------------------------------------------------

/// A vertex in a garment pattern.
#[derive(Debug, Clone)]
pub struct PatternVertex {
    /// 2D UV coordinate in the pattern.
    pub uv: [f32; 2],
    /// 3D position (set after wrap/drape).
    pub pos3d: [f32; 3],
    /// Whether this vertex is pinned (fixed) during simulation.
    pub pinned: bool,
}

/// Edge kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeKind {
    Seam,
    Structural,
    Shear,
    Bend,
}

/// An edge in a garment pattern.
#[derive(Debug, Clone)]
pub struct PatternEdge {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub kind: EdgeKind,
}

/// A garment pattern consisting of vertices, edges, and triangular faces.
#[derive(Debug, Clone)]
pub struct GarmentPattern {
    pub vertices: Vec<PatternVertex>,
    pub edges: Vec<PatternEdge>,
    pub faces: Vec<[u32; 3]>,
}

/// Configuration for a garment pattern.
#[derive(Debug, Clone)]
pub struct GarmentPatternConfig {
    pub material: String,
    pub subdivisions: usize,
    pub gravity: [f32; 3],
}

// ---------------------------------------------------------------------------
// panel builders
// ---------------------------------------------------------------------------

/// Build a rectangular grid panel of size w x h with `rows` x `cols` quads.
pub fn build_rectangle_panel(w: f32, h: f32, rows: usize, cols: usize) -> GarmentPattern {
    let mut vertices = Vec::new();
    let mut edges = Vec::new();
    let mut faces = Vec::new();

    let nrows = rows + 1;
    let ncols = cols + 1;

    // Create vertices
    for i in 0..nrows {
        for j in 0..ncols {
            let u = j as f32 / cols as f32;
            let v = i as f32 / rows as f32;
            let x = u * w;
            let y = v * h;
            vertices.push(PatternVertex {
                uv: [u, v],
                pos3d: [x, y, 0.0],
                pinned: false,
            });
        }
    }

    // Create faces and structural edges
    let mut edge_set: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();

    for i in 0..rows {
        for j in 0..cols {
            let a = (i * ncols + j) as u32;
            let b = (i * ncols + j + 1) as u32;
            let c = ((i + 1) * ncols + j) as u32;
            let d = ((i + 1) * ncols + j + 1) as u32;

            faces.push([a, b, c]);
            faces.push([b, d, c]);

            // Structural edges
            for (p, q) in [(a, b), (b, c), (a, c), (b, d), (d, c)] {
                let key = ((p as usize).min(q as usize), (p as usize).max(q as usize));
                if edge_set.insert(key) {
                    let rest = len2(sub2(vertices[key.0].uv, vertices[key.1].uv)) * w.max(h);
                    edges.push(PatternEdge {
                        a: key.0,
                        b: key.1,
                        rest_len: rest,
                        kind: EdgeKind::Structural,
                    });
                }
            }

            // Shear diagonal
            let key = ((a as usize).min(d as usize), (a as usize).max(d as usize));
            if edge_set.insert(key) {
                let rest = len2(sub2(vertices[key.0].uv, vertices[key.1].uv)) * w.max(h);
                edges.push(PatternEdge {
                    a: key.0,
                    b: key.1,
                    rest_len: rest,
                    kind: EdgeKind::Shear,
                });
            }
        }
    }

    // Bend edges (skip-one)
    for i in 0..nrows {
        for j in 0..ncols {
            if j + 2 < ncols {
                let p = i * ncols + j;
                let q = i * ncols + j + 2;
                let key = (p.min(q), p.max(q));
                if edge_set.insert(key) {
                    let rest = len2(sub2(vertices[key.0].uv, vertices[key.1].uv)) * w.max(h);
                    edges.push(PatternEdge {
                        a: key.0,
                        b: key.1,
                        rest_len: rest,
                        kind: EdgeKind::Bend,
                    });
                }
            }
            if i + 2 < nrows {
                let p = i * ncols + j;
                let q = (i + 2) * ncols + j;
                let key = (p.min(q), p.max(q));
                if edge_set.insert(key) {
                    let rest = len2(sub2(vertices[key.0].uv, vertices[key.1].uv)) * w.max(h);
                    edges.push(PatternEdge {
                        a: key.0,
                        b: key.1,
                        rest_len: rest,
                        kind: EdgeKind::Bend,
                    });
                }
            }
        }
    }

    GarmentPattern {
        vertices,
        edges,
        faces,
    }
}

/// Build a circular panel with given radius and number of segments.
pub fn build_circular_panel(radius: f32, segments: usize) -> GarmentPattern {
    let mut vertices = Vec::new();
    let mut edges = Vec::new();
    let mut faces = Vec::new();

    // Center vertex
    vertices.push(PatternVertex {
        uv: [0.5, 0.5],
        pos3d: [0.0, 0.0, 0.0],
        pinned: false,
    });

    // Rim vertices
    for i in 0..segments {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        let u = 0.5 + 0.5 * angle.cos();
        let v = 0.5 + 0.5 * angle.sin();
        vertices.push(PatternVertex {
            uv: [u, v],
            pos3d: [radius * angle.cos(), radius * angle.sin(), 0.0],
            pinned: false,
        });
    }

    // Faces: fan from center
    let mut edge_set: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for i in 0..segments {
        let a = 0u32;
        let b = (1 + i) as u32;
        let c = (1 + (i + 1) % segments) as u32;
        faces.push([a, b, c]);

        for (p, q) in [(a, b), (b, c), (a, c)] {
            let key = ((p as usize).min(q as usize), (p as usize).max(q as usize));
            if edge_set.insert(key) {
                let rest = len3(sub3(vertices[key.0].pos3d, vertices[key.1].pos3d));
                edges.push(PatternEdge {
                    a: key.0,
                    b: key.1,
                    rest_len: rest,
                    kind: EdgeKind::Structural,
                });
            }
        }
    }

    GarmentPattern {
        vertices,
        edges,
        faces,
    }
}

// ---------------------------------------------------------------------------
// seam operations
// ---------------------------------------------------------------------------

/// Mark edges between panels as seams.
pub fn sew_panels(
    a: &mut GarmentPattern,
    b: &mut GarmentPattern,
    seam_a: &[usize],
    seam_b: &[usize],
) {
    // Offset for b's vertex indices
    let offset = a.vertices.len();

    // Mark seam edges in panel a
    for edge in a.edges.iter_mut() {
        if seam_a.contains(&edge.a) || seam_a.contains(&edge.b) {
            edge.kind = EdgeKind::Seam;
        }
    }

    // Mark seam edges in panel b
    for edge in b.edges.iter_mut() {
        if seam_b.contains(&edge.a) || seam_b.contains(&edge.b) {
            edge.kind = EdgeKind::Seam;
        }
    }

    // Add cross-panel seam edges between matching pairs
    let n = seam_a.len().min(seam_b.len());
    for i in 0..n {
        let va = seam_a[i];
        let vb = seam_b[i] + offset;
        let rest = len3(sub3(
            a.vertices[seam_a[i]].pos3d,
            b.vertices[seam_b[i]].pos3d,
        ));
        a.edges.push(PatternEdge {
            a: va,
            b: vb,
            rest_len: rest,
            kind: EdgeKind::Seam,
        });
    }
}

// ---------------------------------------------------------------------------
// particle extraction
// ---------------------------------------------------------------------------

/// Extract 3D positions from a pattern.
pub fn pattern_to_cloth_particles(pattern: &GarmentPattern) -> Vec<[f32; 3]> {
    pattern.vertices.iter().map(|v| v.pos3d).collect()
}

/// Count springs (edges) in a pattern.
pub fn pattern_spring_count(pattern: &GarmentPattern) -> usize {
    pattern.edges.len()
}

/// Count faces in a pattern.
pub fn pattern_face_count(pattern: &GarmentPattern) -> usize {
    pattern.faces.len()
}

// ---------------------------------------------------------------------------
// wrap / drape
// ---------------------------------------------------------------------------

/// Wrap a flat panel onto a cylinder of given radius and height.
/// UV.x -> azimuthal angle, UV.y -> height.
pub fn wrap_panel_to_cylinder(pattern: &mut GarmentPattern, radius: f32, height: f32) {
    for v in pattern.vertices.iter_mut() {
        let theta = v.uv[0] * 2.0 * std::f32::consts::PI;
        let y = v.uv[1] * height;
        v.pos3d = [radius * theta.cos(), y, radius * theta.sin()];
    }
}

/// Drape a flat panel onto a sphere of given radius.
/// UV.x -> azimuth [0, 2π], UV.y -> elevation [0, π].
pub fn drape_panel_onto_sphere(pattern: &mut GarmentPattern, radius: f32) {
    for v in pattern.vertices.iter_mut() {
        let theta = v.uv[0] * 2.0 * std::f32::consts::PI;
        let phi = v.uv[1] * std::f32::consts::PI;
        v.pos3d = [
            radius * phi.sin() * theta.cos(),
            radius * phi.cos(),
            radius * phi.sin() * theta.sin(),
        ];
    }
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_rectangle_panel_vertex_count() {
        let panel = build_rectangle_panel(1.0, 1.0, 3, 4);
        // (rows+1) * (cols+1) = 4 * 5 = 20
        assert_eq!(
            panel.vertices.len(),
            20,
            "expected 20 vertices, got {}",
            panel.vertices.len()
        );
    }

    #[test]
    fn test_build_rectangle_panel_face_count() {
        let panel = build_rectangle_panel(1.0, 1.0, 3, 4);
        // 2 tris per quad, rows*cols quads = 3*4 = 12 quads -> 24 tris
        assert_eq!(
            panel.faces.len(),
            24,
            "expected 24 faces, got {}",
            panel.faces.len()
        );
    }

    #[test]
    fn test_build_rectangle_panel_edge_count_nonzero() {
        let panel = build_rectangle_panel(1.0, 1.0, 3, 4);
        assert!(!panel.edges.is_empty(), "panel should have edges");
    }

    #[test]
    fn test_build_circular_panel_vertex_count() {
        let panel = build_circular_panel(1.0, 8);
        // 1 center + 8 rim
        assert_eq!(panel.vertices.len(), 9);
    }

    #[test]
    fn test_build_circular_panel_face_count() {
        let panel = build_circular_panel(1.0, 8);
        assert_eq!(panel.faces.len(), 8);
    }

    #[test]
    fn test_wrap_panel_to_cylinder_no_nan() {
        let mut panel = build_rectangle_panel(1.0, 2.0, 4, 4);
        wrap_panel_to_cylinder(&mut panel, 1.5, 2.0);
        for v in &panel.vertices {
            assert!(
                v.pos3d[0].is_finite() && v.pos3d[1].is_finite() && v.pos3d[2].is_finite(),
                "cylinder wrap produced NaN: {:?}",
                v.pos3d
            );
        }
    }

    #[test]
    fn test_wrap_panel_to_cylinder_radius() {
        let mut panel = build_rectangle_panel(1.0, 1.0, 2, 2);
        let r = 2.0f32;
        wrap_panel_to_cylinder(&mut panel, r, 1.0);
        for v in &panel.vertices {
            let dist = (v.pos3d[0] * v.pos3d[0] + v.pos3d[2] * v.pos3d[2]).sqrt();
            assert!(
                (dist - r).abs() < 1e-5,
                "vertex should be at radius {r}, got {dist}"
            );
        }
    }

    #[test]
    fn test_drape_panel_onto_sphere_no_nan() {
        let mut panel = build_rectangle_panel(1.0, 1.0, 4, 4);
        drape_panel_onto_sphere(&mut panel, 2.0);
        for v in &panel.vertices {
            assert!(
                v.pos3d[0].is_finite() && v.pos3d[1].is_finite() && v.pos3d[2].is_finite(),
                "sphere drape produced NaN: {:?}",
                v.pos3d
            );
        }
    }

    #[test]
    fn test_drape_panel_onto_sphere_radius() {
        let mut panel = build_rectangle_panel(1.0, 1.0, 3, 3);
        let r = 3.0f32;
        drape_panel_onto_sphere(&mut panel, r);
        for v in &panel.vertices {
            let d = len3(v.pos3d);
            assert!(
                (d - r).abs() < 1e-4,
                "sphere drape: vertex at radius {d}, expected {r}"
            );
        }
    }

    #[test]
    fn test_pattern_to_cloth_particles_count() {
        let panel = build_rectangle_panel(1.0, 1.0, 3, 4);
        let particles = pattern_to_cloth_particles(&panel);
        assert_eq!(particles.len(), panel.vertices.len());
    }

    #[test]
    fn test_pattern_spring_count_nonzero() {
        let panel = build_rectangle_panel(1.0, 1.0, 3, 3);
        assert!(pattern_spring_count(&panel) > 0);
    }

    #[test]
    fn test_pattern_face_count_rectangle() {
        let panel = build_rectangle_panel(1.0, 1.0, 2, 2);
        // 2 tris per quad, 2*2 quads = 8
        assert_eq!(pattern_face_count(&panel), 8);
    }

    #[test]
    fn test_sew_panels_marks_seams() {
        let mut a = build_rectangle_panel(1.0, 1.0, 2, 2);
        let mut b = build_rectangle_panel(1.0, 1.0, 2, 2);
        // Sew left column of a to left column of b
        let seam_a: Vec<usize> = (0..3).map(|i| i * 3).collect();
        let seam_b: Vec<usize> = (0..3).map(|i| i * 3).collect();
        sew_panels(&mut a, &mut b, &seam_a, &seam_b);
        let has_seam = a.edges.iter().any(|e| e.kind == EdgeKind::Seam);
        assert!(has_seam, "sewing should mark at least one seam edge");
    }

    #[test]
    fn test_pattern_vertices_in_bounds_rectangle() {
        let w = 2.0f32;
        let h = 3.0f32;
        let panel = build_rectangle_panel(w, h, 4, 4);
        for v in &panel.vertices {
            assert!(
                v.pos3d[0] >= 0.0 && v.pos3d[0] <= w + 1e-5,
                "x={} out of [0,{w}]",
                v.pos3d[0]
            );
            assert!(
                v.pos3d[1] >= 0.0 && v.pos3d[1] <= h + 1e-5,
                "y={} out of [0,{h}]",
                v.pos3d[1]
            );
        }
    }
}
