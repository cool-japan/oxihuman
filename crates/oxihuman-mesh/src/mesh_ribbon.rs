// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Ribbon/trail mesh generation along a sequence of oriented points.
//!
//! A ribbon is a flat strip mesh built by connecting left/right edge
//! vertices at each point along a path. Supports width tapering,
//! twist, and UV coordinate generation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Configuration for ribbon mesh generation.
pub struct RibbonConfig {
    /// Base half-width of the ribbon.
    pub width: f32,
    /// Number of path segments (ignored when building from explicit points).
    pub segments: u32,
    /// Twist angle in radians applied over the full length.
    pub twist: f32,
    /// Whether to taper the ribbon to zero width at the ends.
    pub taper: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// A single oriented point on the ribbon path.
pub struct RibbonPoint {
    /// World-space position of the point.
    pub position: [f32; 3],
    /// Up direction used to orient the ribbon cross-section.
    pub up: [f32; 3],
    /// Per-point width multiplier (1.0 = full width).
    pub width_scale: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Generated ribbon mesh data.
pub struct RibbonMesh {
    /// Vertex positions, two per input point (left, right).
    pub vertices: Vec<[f32; 3]>,
    /// UV coordinates corresponding to each vertex.
    pub uvs: Vec<[f32; 2]>,
    /// Triangle indices (two triangles per segment quad).
    pub indices: Vec<u32>,
    /// Number of source path points used.
    pub point_count: usize,
}

/// Return default ribbon configuration.
#[allow(dead_code)]
pub fn default_ribbon_config() -> RibbonConfig {
    RibbonConfig {
        width: 1.0,
        segments: 16,
        twist: 0.0,
        taper: false,
    }
}

/// Create a new ribbon point with position and up vector; width_scale = 1.
#[allow(dead_code)]
pub fn new_ribbon_point(pos: [f32; 3], up: [f32; 3]) -> RibbonPoint {
    RibbonPoint {
        position: pos,
        up,
        width_scale: 1.0,
    }
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub3(a, b);
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Compute the total arc length of the ribbon path.
#[allow(dead_code)]
pub fn ribbon_length(points: &[RibbonPoint]) -> f32 {
    if points.len() < 2 {
        return 0.0;
    }
    points
        .windows(2)
        .map(|w| dist3(w[1].position, w[0].position))
        .sum()
}

/// Build a ribbon mesh from a sequence of oriented points.
///
/// Each pair of consecutive points produces one quad (two triangles).
/// UV u runs 0→1 across the width; v runs 0→1 along the length.
#[allow(dead_code)]
pub fn build_ribbon(points: &[RibbonPoint], cfg: &RibbonConfig) -> RibbonMesh {
    let n = points.len();
    if n < 2 {
        return RibbonMesh {
            vertices: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            point_count: n,
        };
    }

    let total_len = ribbon_length(points);
    let half = cfg.width * 0.5;

    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(n * 2);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n * 2);
    let mut indices: Vec<u32> = Vec::with_capacity((n - 1) * 6);

    let mut arc = 0.0_f32;

    for (i, pt) in points.iter().enumerate() {
        // Compute forward direction from finite difference
        let fwd = if i + 1 < n {
            normalize3(sub3(points[i + 1].position, pt.position))
        } else {
            normalize3(sub3(pt.position, points[i - 1].position))
        };

        let up = normalize3(pt.up);
        // right = fwd × up, then re-derive up
        let right_raw = cross3(fwd, up);
        let right = if right_raw[0] * right_raw[0]
            + right_raw[1] * right_raw[1]
            + right_raw[2] * right_raw[2]
            < 1e-12
        {
            // degenerate — use world X
            [1.0, 0.0, 0.0]
        } else {
            normalize3(right_raw)
        };

        // Twist accumulation (linear along path)
        let t_frac = if total_len > 1e-9 { arc / total_len } else { 0.0 };
        let twist_angle = cfg.twist * t_frac;
        let cos_t = twist_angle.cos();
        let sin_t = twist_angle.sin();

        // Re-derive up after twist
        let up2 = normalize3(cross3(right, fwd));
        // twisted right = cos*right + sin*up2
        let tr = add3(scale3(right, cos_t), scale3(up2, sin_t));

        // Per-point width scale + optional taper
        let taper_scale = if cfg.taper {
            let tt = t_frac * 2.0;
            if tt < 1.0 {
                tt
            } else {
                2.0 - tt
            }
        } else {
            1.0
        };
        let w = half * pt.width_scale * taper_scale;

        let v_coord = if total_len > 1e-9 { arc / total_len } else { 0.0 };

        // left vertex
        let left_pos = sub3(pt.position, scale3(tr, w));
        vertices.push(left_pos);
        uvs.push([0.0, v_coord]);

        // right vertex
        let right_pos = add3(pt.position, scale3(tr, w));
        vertices.push(right_pos);
        uvs.push([1.0, v_coord]);

        // Accumulate arc length
        if i + 1 < n {
            arc += dist3(points[i + 1].position, pt.position);
        }

        // Emit quad indices for segment [i, i+1]
        if i + 1 < n {
            let base = (i * 2) as u32;
            // left0, right0, right1
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 3);
            // left0, right1, left1
            indices.push(base);
            indices.push(base + 3);
            indices.push(base + 2);
        }
    }

    RibbonMesh {
        vertices,
        uvs,
        indices,
        point_count: n,
    }
}

/// Return the total number of vertices in the ribbon mesh.
#[allow(dead_code)]
pub fn ribbon_vertex_count(mesh: &RibbonMesh) -> usize {
    mesh.vertices.len()
}

/// Return the total number of indices in the ribbon mesh.
#[allow(dead_code)]
pub fn ribbon_index_count(mesh: &RibbonMesh) -> usize {
    mesh.indices.len()
}

/// Serialize ribbon mesh to a compact JSON string.
#[allow(dead_code)]
pub fn ribbon_to_json(mesh: &RibbonMesh) -> String {
    format!(
        "{{\"point_count\":{},\"vertex_count\":{},\"index_count\":{}}}",
        mesh.point_count,
        mesh.vertices.len(),
        mesh.indices.len()
    )
}

/// Flip ribbon normals by swapping the V UV coordinate (1 - v).
///
/// This effectively mirrors the ribbon so the opposite face is the front face.
#[allow(dead_code)]
pub fn flip_ribbon_normals(mesh: &mut RibbonMesh) {
    for uv in mesh.uvs.iter_mut() {
        uv[1] = 1.0 - uv[1];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_straight_points(n: usize) -> Vec<RibbonPoint> {
        (0..n)
            .map(|i| new_ribbon_point([i as f32, 0.0, 0.0], [0.0, 1.0, 0.0]))
            .collect()
    }

    #[test]
    fn test_default_config() {
        let cfg = default_ribbon_config();
        assert!((cfg.width - 1.0).abs() < 1e-6);
        assert_eq!(cfg.segments, 16);
        assert!(!cfg.taper);
    }

    #[test]
    fn test_new_ribbon_point() {
        let p = new_ribbon_point([1.0, 2.0, 3.0], [0.0, 1.0, 0.0]);
        assert_eq!(p.position, [1.0, 2.0, 3.0]);
        assert!((p.width_scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ribbon_length_zero() {
        let pts = vec![new_ribbon_point([0.0, 0.0, 0.0], [0.0, 1.0, 0.0])];
        assert!((ribbon_length(&pts)).abs() < 1e-6);
    }

    #[test]
    fn test_ribbon_length_straight() {
        let pts = make_straight_points(5);
        let len = ribbon_length(&pts);
        assert!((len - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_build_ribbon_empty() {
        let cfg = default_ribbon_config();
        let mesh = build_ribbon(&[], &cfg);
        assert_eq!(mesh.vertices.len(), 0);
        assert_eq!(mesh.indices.len(), 0);
    }

    #[test]
    fn test_build_ribbon_single_point() {
        let pts = vec![new_ribbon_point([0.0, 0.0, 0.0], [0.0, 1.0, 0.0])];
        let cfg = default_ribbon_config();
        let mesh = build_ribbon(&pts, &cfg);
        assert_eq!(mesh.vertices.len(), 0);
        assert_eq!(mesh.indices.len(), 0);
    }

    #[test]
    fn test_build_ribbon_two_points() {
        let pts = make_straight_points(2);
        let cfg = default_ribbon_config();
        let mesh = build_ribbon(&pts, &cfg);
        // 2 points → 4 vertices, 1 quad = 6 indices
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.indices.len(), 6);
        assert_eq!(mesh.point_count, 2);
    }

    #[test]
    fn test_build_ribbon_counts() {
        let pts = make_straight_points(5);
        let cfg = default_ribbon_config();
        let mesh = build_ribbon(&pts, &cfg);
        assert_eq!(ribbon_vertex_count(&mesh), 10);
        assert_eq!(ribbon_index_count(&mesh), 24);
    }

    #[test]
    fn test_build_ribbon_taper() {
        let pts = make_straight_points(4);
        let mut cfg = default_ribbon_config();
        cfg.taper = true;
        let mesh = build_ribbon(&pts, &cfg);
        assert_eq!(mesh.vertices.len(), 8);
    }

    #[test]
    fn test_build_ribbon_twist() {
        let pts = make_straight_points(4);
        let mut cfg = default_ribbon_config();
        cfg.twist = std::f32::consts::PI;
        let mesh = build_ribbon(&pts, &cfg);
        assert_eq!(mesh.vertices.len(), 8);
    }

    #[test]
    fn test_flip_ribbon_normals() {
        let pts = make_straight_points(3);
        let cfg = default_ribbon_config();
        let mut mesh = build_ribbon(&pts, &cfg);
        flip_ribbon_normals(&mut mesh);
        // All v coordinates should be flipped
        for uv in &mesh.uvs {
            assert!(uv[1] >= 0.0 && uv[1] <= 1.0);
        }
    }

    #[test]
    fn test_ribbon_to_json() {
        let pts = make_straight_points(3);
        let cfg = default_ribbon_config();
        let mesh = build_ribbon(&pts, &cfg);
        let json = ribbon_to_json(&mesh);
        assert!(json.contains("vertex_count"));
        assert!(json.contains("index_count"));
    }

    #[test]
    fn test_ribbon_uv_range() {
        let pts = make_straight_points(5);
        let cfg = default_ribbon_config();
        let mesh = build_ribbon(&pts, &cfg);
        for uv in &mesh.uvs {
            assert!(uv[0] >= 0.0 && uv[0] <= 1.0);
            assert!(uv[1] >= 0.0 && uv[1] <= 1.0 + 1e-6);
        }
    }

    #[test]
    fn test_ribbon_indices_in_range() {
        let pts = make_straight_points(6);
        let cfg = default_ribbon_config();
        let mesh = build_ribbon(&pts, &cfg);
        let vcount = mesh.vertices.len() as u32;
        for &idx in &mesh.indices {
            assert!(idx < vcount);
        }
    }
}
