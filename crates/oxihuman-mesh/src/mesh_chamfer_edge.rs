// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Chamfer edges with configurable segments.

/// Chamfer configuration.
#[derive(Debug, Clone)]
pub struct ChamferToolConfig {
    pub amount: f32,
    pub segments: usize,
    pub profile: f32, // 0 = linear, 1 = convex, -1 = concave
}

impl Default for ChamferToolConfig {
    fn default() -> Self {
        ChamferToolConfig {
            amount: 0.1,
            segments: 1,
            profile: 0.0,
        }
    }
}

/// Result from chamfering edges.
#[derive(Debug, Clone)]
pub struct ChamferToolResult {
    pub new_positions: Vec<[f32; 3]>,
    pub new_indices: Vec<u32>,
    pub chamfered_edge_count: usize,
    pub new_vertex_count: usize,
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-10 {
        [0.0; 3]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Compute the two offset points along each side of an edge for a chamfer.
pub fn chamfer_offset_points(
    v0: [f32; 3],
    v1: [f32; 3],
    face_normal: [f32; 3],
    amount: f32,
) -> ([f32; 3], [f32; 3]) {
    let dir = normalize3([v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]]);
    let side = normalize3(cross3(dir, face_normal));
    let a = [
        v0[0] + side[0] * amount,
        v0[1] + side[1] * amount,
        v0[2] + side[2] * amount,
    ];
    let b = [
        v1[0] + side[0] * amount,
        v1[1] + side[1] * amount,
        v1[2] + side[2] * amount,
    ];
    (a, b)
}

/// Generate chamfer strip positions between two parallel edges.
pub fn chamfer_strip(
    p0: [f32; 3],
    p1: [f32; 3],
    q0: [f32; 3],
    q1: [f32; 3],
    segments: usize,
    profile: f32,
) -> Vec<[f32; 3]> {
    let segs = segments.max(1);
    let mut pts = Vec::with_capacity((segs + 1) * 2);
    for i in 0..=segs {
        let t_raw = i as f32 / segs as f32;
        /* apply profile curve */
        let t = if profile.abs() < 1e-6 {
            t_raw
        } else {
            let sign = profile.signum();
            t_raw.powf(1.0 + sign * profile.abs() * 2.0)
        };
        pts.push(lerp3(p0, p1, t));
        pts.push(lerp3(q0, q1, t));
    }
    pts
}

/// Apply chamfer to a list of edges in a mesh.
pub fn chamfer_edges(
    positions: &[[f32; 3]],
    edge_pairs: &[(usize, usize)],
    config: &ChamferToolConfig,
) -> ChamferToolResult {
    let mut new_positions = positions.to_vec();
    let mut new_indices = Vec::new();
    let mut chamfered_edge_count = 0usize;
    let mut new_vertex_count = 0usize;
    let face_normal = [0.0f32, 0.0, 1.0]; // default, user should override
    for &(a, b) in edge_pairs {
        if a >= positions.len() || b >= positions.len() {
            continue;
        }
        let v0 = positions[a];
        let v1 = positions[b];
        let (off_a, off_b) = chamfer_offset_points(v0, v1, face_normal, config.amount);
        let segs = config.segments.max(1);
        let strip = chamfer_strip(v0, v1, off_a, off_b, segs, config.profile);
        let base = new_positions.len() as u32;
        new_positions.extend_from_slice(&strip);
        new_vertex_count += strip.len();
        /* build quads from strip pairs */
        for i in 0..segs {
            let s = base + (i * 2) as u32;
            new_indices.extend_from_slice(&[s, s + 1, s + 3, s, s + 3, s + 2]);
        }
        chamfered_edge_count += 1;
    }
    ChamferToolResult {
        new_positions,
        new_indices,
        chamfered_edge_count,
        new_vertex_count,
    }
}

/// Validate chamfer config.
pub fn validate_chamfer_config(config: &ChamferToolConfig) -> bool {
    config.amount > 0.0 && config.segments > 0
}

/// Estimate new vertex count from chamfering n edges.
pub fn chamfer_vertex_estimate(n_edges: usize, segments: usize) -> usize {
    n_edges * (segments + 1) * 2
}

/// Default chamfer config.
pub fn default_chamfer_config() -> ChamferToolConfig {
    ChamferToolConfig::default()
}

/// Compute chamfer amount from a bevel width and edge angle (degrees).
pub fn chamfer_from_bevel_width(width: f32, angle_deg: f32) -> f32 {
    let rad = angle_deg * std::f32::consts::PI / 180.0;
    width / (rad / 2.0).sin().max(1e-8)
}

/// Scale all chamfer offsets by a factor.
pub fn scale_chamfer_result(result: &mut ChamferToolResult, factor: f32) {
    for p in result.new_positions.iter_mut() {
        p[0] *= factor;
        p[1] *= factor;
        p[2] *= factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /* validate_chamfer_config */
    #[test]
    fn test_validate_chamfer_config() {
        assert!(validate_chamfer_config(&default_chamfer_config()));
        let bad = ChamferToolConfig {
            amount: 0.0,
            segments: 2,
            profile: 0.0,
        };
        assert!(!validate_chamfer_config(&bad));
    }

    /* chamfer_vertex_estimate */
    #[test]
    fn test_chamfer_vertex_estimate() {
        assert_eq!(chamfer_vertex_estimate(2, 2), 12);
    }

    /* chamfer_strip length */
    #[test]
    fn test_chamfer_strip_length() {
        let s = chamfer_strip(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            2,
            0.0,
        );
        assert_eq!(s.len(), (2 + 1) * 2);
    }

    /* chamfer_offset_points */
    #[test]
    fn test_chamfer_offset_points() {
        let (a, _b) = chamfer_offset_points([0.0; 3], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.1);
        assert!((a[0]).abs() < 0.15);
        assert!(a[1].abs() > 0.05 || a[2].abs() > 0.0);
    }

    /* chamfer_edges produces new verts */
    #[test]
    fn test_chamfer_edges() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let cfg = default_chamfer_config();
        let res = chamfer_edges(&pos, &[(0, 1)], &cfg);
        assert_eq!(res.chamfered_edge_count, 1);
        assert!(res.new_positions.len() > 2);
    }

    /* chamfer_from_bevel_width */
    #[test]
    fn test_chamfer_from_bevel_width() {
        let r = chamfer_from_bevel_width(1.0, 90.0);
        assert!(r > 0.0);
    }

    /* scale_chamfer_result */
    #[test]
    fn test_scale_chamfer_result() {
        let pos = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let cfg = default_chamfer_config();
        let mut res = chamfer_edges(&pos, &[(0, 1)], &cfg);
        let n = res.new_positions.len();
        scale_chamfer_result(&mut res, 2.0);
        assert_eq!(res.new_positions.len(), n);
    }

    /* default_chamfer_config segments */
    #[test]
    fn test_default_config_segments() {
        let cfg = default_chamfer_config();
        assert!(cfg.segments >= 1);
    }

    /* chamfer_strip profile = 1.0 doesn't crash */
    #[test]
    fn test_chamfer_strip_profile() {
        let s = chamfer_strip(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            3,
            1.0,
        );
        assert_eq!(s.len(), (3 + 1) * 2);
    }
}
