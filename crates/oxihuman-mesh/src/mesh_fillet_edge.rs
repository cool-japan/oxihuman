// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge fillet (bevel with arc).

use std::f32::consts::PI;

/// Configuration for a fillet operation.
#[derive(Debug, Clone)]
pub struct FilletToolConfig {
    pub radius: f32,
    pub segments: usize,
}

impl Default for FilletToolConfig {
    fn default() -> Self {
        FilletToolConfig {
            radius: 0.1,
            segments: 4,
        }
    }
}

/// Result of a fillet operation.
#[derive(Debug, Clone)]
pub struct FilletToolResult {
    pub new_positions: Vec<[f32; 3]>,
    pub new_indices: Vec<u32>,
    pub filleted_edges: usize,
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-10 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / l, v[1] / l, v[2] / l]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Generate arc points between two tangent directions in the plane of the arc.
pub fn arc_points(
    centre: [f32; 3],
    from_dir: [f32; 3],
    to_dir: [f32; 3],
    radius: f32,
    segments: usize,
) -> Vec<[f32; 3]> {
    let from = normalize3(from_dir);
    let to = normalize3(to_dir);
    let dot = dot3(from, to).clamp(-1.0, 1.0);
    let angle = dot.acos();
    let cross = cross3(from, to);
    let cross_len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    let mut pts = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let a = angle * t;
        if cross_len < 1e-8 {
            pts.push([
                centre[0] + from[0] * radius,
                centre[1] + from[1] * radius,
                centre[2] + from[2] * radius,
            ]);
        } else {
            let axis = [
                cross[0] / cross_len,
                cross[1] / cross_len,
                cross[2] / cross_len,
            ];
            /* Rodrigues rotation */
            let cos_a = a.cos();
            let sin_a = a.sin();
            let d = dot3(axis, from);
            let rotated = [
                from[0] * cos_a
                    + (axis[1] * from[2] - axis[2] * from[1]) * sin_a
                    + axis[0] * d * (1.0 - cos_a),
                from[1] * cos_a
                    + (axis[2] * from[0] - axis[0] * from[2]) * sin_a
                    + axis[1] * d * (1.0 - cos_a),
                from[2] * cos_a
                    + (axis[0] * from[1] - axis[1] * from[0]) * sin_a
                    + axis[2] * d * (1.0 - cos_a),
            ];
            pts.push([
                centre[0] + rotated[0] * radius,
                centre[1] + rotated[1] * radius,
                centre[2] + rotated[2] * radius,
            ]);
        }
    }
    pts
}

/// Fillet a single edge given its two endpoint positions and two face normals.
pub fn fillet_edge_simple(
    v0: [f32; 3],
    v1: [f32; 3],
    n0: [f32; 3],
    n1: [f32; 3],
    config: &FilletToolConfig,
) -> Vec<[f32; 3]> {
    let mid = lerp3(v0, v1, 0.5);
    let from_dir = normalize3(n0);
    let to_dir = normalize3(n1);
    arc_points(mid, from_dir, to_dir, config.radius, config.segments)
}

/// Estimate the number of new vertices from filleting `n` edges.
pub fn fillet_vertex_estimate(n_edges: usize, segments: usize) -> usize {
    n_edges * (segments + 1)
}

/// Compute the arc length for a fillet of given radius and angle.
pub fn fillet_arc_length(radius: f32, angle_rad: f32) -> f32 {
    radius * angle_rad.abs()
}

/// Compute fillet radius from a desired chamfer distance and dihedral angle.
pub fn fillet_radius_from_chamfer(chamfer: f32, dihedral_rad: f32) -> f32 {
    let half = (PI - dihedral_rad) * 0.5;
    chamfer / half.tan().max(1e-8)
}

/// Validate fillet config.
pub fn validate_fillet_config(config: &FilletToolConfig) -> bool {
    config.radius > 0.0 && config.segments > 0
}

/// Apply fillet to a sequence of positions by inserting arc points.
pub fn apply_edge_fillet(
    positions: &[[f32; 3]],
    edge_pairs: &[(usize, usize)],
    config: &FilletToolConfig,
) -> FilletToolResult {
    let mut new_positions = positions.to_vec();
    let mut new_indices: Vec<u32> = Vec::new();
    let mut filleted_edges = 0usize;
    for &(a, b) in edge_pairs {
        if a >= positions.len() || b >= positions.len() {
            continue;
        }
        let v0 = positions[a];
        let v1 = positions[b];
        let dir = normalize3([v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]]);
        let perp = if dir[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let n0 = normalize3(cross3(dir, perp));
        let n1 = normalize3(cross3(perp, dir));
        let arc = fillet_edge_simple(v0, v1, n0, n1, config);
        let base = new_positions.len() as u32;
        for (i, &ap) in arc.iter().enumerate() {
            new_positions.push(ap);
            if i + 1 < arc.len() {
                new_indices.push(base + i as u32);
                new_indices.push(base + i as u32 + 1);
            }
        }
        filleted_edges += 1;
    }
    FilletToolResult {
        new_positions,
        new_indices,
        filleted_edges,
    }
}

/// Default fillet config.
pub fn default_fillet_tool_config() -> FilletToolConfig {
    FilletToolConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /* arc_points count */
    #[test]
    fn test_arc_points_count() {
        let pts = arc_points([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0, 4);
        assert_eq!(pts.len(), 5);
    }

    /* arc_points first point near from_dir */
    #[test]
    fn test_arc_points_first() {
        let pts = arc_points([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0, 4);
        assert!((pts[0][0] - 1.0).abs() < 1e-5);
    }

    /* fillet_arc_length */
    #[test]
    fn test_fillet_arc_length() {
        let l = fillet_arc_length(1.0, PI / 2.0);
        assert!((l - PI / 2.0).abs() < 1e-5);
    }

    /* validate_fillet_config */
    #[test]
    fn test_validate_fillet_config() {
        let cfg = default_fillet_tool_config();
        assert!(validate_fillet_config(&cfg));
        let bad = FilletToolConfig {
            radius: -1.0,
            segments: 4,
        };
        assert!(!validate_fillet_config(&bad));
    }

    /* fillet_vertex_estimate */
    #[test]
    fn test_fillet_vertex_estimate() {
        assert_eq!(fillet_vertex_estimate(2, 4), 10);
    }

    /* apply_edge_fillet */
    #[test]
    fn test_apply_edge_fillet() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let cfg = default_fillet_tool_config();
        let res = apply_edge_fillet(&pos, &[(0, 1)], &cfg);
        assert_eq!(res.filleted_edges, 1);
        assert!(res.new_positions.len() > 2);
    }

    /* fillet_edge_simple returns arc */
    #[test]
    fn test_fillet_edge_simple() {
        let cfg = default_fillet_tool_config();
        let pts = fillet_edge_simple(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            &cfg,
        );
        assert_eq!(pts.len(), cfg.segments + 1);
    }

    /* fillet_radius_from_chamfer */
    #[test]
    fn test_fillet_radius_from_chamfer() {
        let r = fillet_radius_from_chamfer(1.0, PI / 2.0);
        assert!(r > 0.0);
    }
}
