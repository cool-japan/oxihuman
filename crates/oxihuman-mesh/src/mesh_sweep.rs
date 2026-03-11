// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Swept volume — extrude a cross-section profile along a 3-D path.

use std::f32::consts::PI;

// ─── Structures ──────────────────────────────────────────────────────────────

/// 2-D cross-section profile in the XY plane.
#[allow(dead_code)]
pub struct SweepProfile {
    pub points: Vec<[f32; 2]>,
    pub closed: bool,
}

/// 3-D sweep path.
#[allow(dead_code)]
pub struct SweepPath {
    pub points: Vec<[f32; 3]>,
    /// Global up vector used for reference frame computation.
    pub up_vector: [f32; 3],
}

/// Result of a sweep operation.
#[allow(dead_code)]
pub struct SweepResult {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

// ─── Math helpers (module-private unless prefixed) ────────────────────────────

/// Normalize a 3-D vector (returns zero-vector if near zero).
#[allow(dead_code)]
pub fn normalize3_sweep(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Cross product of two 3-D vectors.
#[allow(dead_code)]
pub fn cross3_sweep(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
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
    let d = sub3(b, a);
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

// ─── Frame computation ────────────────────────────────────────────────────────

/// Compute a Frenet-like frame (tangent, normal, binormal) at path point `idx`.
/// Returns `[[tangent], [normal], [binormal]]`.
#[allow(dead_code)]
pub fn frame_at_path_point(path: &SweepPath, idx: usize) -> [[f32; 3]; 3] {
    let n = path.points.len();
    assert!(n >= 2, "path must have at least 2 points");

    // Tangent: forward difference at ends, central difference elsewhere.
    let tangent = if idx == 0 {
        normalize3_sweep(sub3(path.points[1], path.points[0]))
    } else if idx == n - 1 {
        normalize3_sweep(sub3(path.points[n - 1], path.points[n - 2]))
    } else {
        normalize3_sweep(sub3(path.points[idx + 1], path.points[idx - 1]))
    };

    // Binormal: tangent × up; fall back to another axis if degenerate.
    let up = normalize3_sweep(path.up_vector);
    let mut binormal = normalize3_sweep(cross3_sweep(tangent, up));
    if dot3(binormal, binormal) < 1e-6 {
        // tangent is parallel to up; pick a different axis.
        let alt = if tangent[0].abs() < 0.9 {
            [1.0_f32, 0.0, 0.0]
        } else {
            [0.0_f32, 1.0, 0.0]
        };
        binormal = normalize3_sweep(cross3_sweep(tangent, alt));
    }
    let normal = normalize3_sweep(cross3_sweep(binormal, tangent));
    [tangent, normal, binormal]
}

/// Transform a 2-D profile point into 3-D using a Frenet frame and origin.
#[allow(dead_code)]
pub fn transform_profile_point(p2d: [f32; 2], frame: [[f32; 3]; 3], origin: [f32; 3]) -> [f32; 3] {
    let [_tangent, normal, binormal] = frame;
    add3(
        origin,
        add3(scale3(normal, p2d[0]), scale3(binormal, p2d[1])),
    )
}

// ─── Path utilities ───────────────────────────────────────────────────────────

/// Total arc length of the path.
#[allow(dead_code)]
pub fn path_length(path: &SweepPath) -> f32 {
    path.points.windows(2).map(|w| dist3(w[0], w[1])).sum()
}

/// Cumulative arc lengths starting at 0.0 for each path point.
#[allow(dead_code)]
pub fn path_arc_lengths(path: &SweepPath) -> Vec<f32> {
    let mut arc = vec![0.0_f32; path.points.len()];
    for i in 1..path.points.len() {
        arc[i] = arc[i - 1] + dist3(path.points[i - 1], path.points[i]);
    }
    arc
}

// ─── Profile utilities ────────────────────────────────────────────────────────

/// Perimeter of the profile (closed profiles add the last→first segment).
#[allow(dead_code)]
pub fn profile_perimeter(profile: &SweepProfile) -> f32 {
    if profile.points.len() < 2 {
        return 0.0;
    }
    let mut perim: f32 = profile
        .points
        .windows(2)
        .map(|w| {
            let dx = w[1][0] - w[0][0];
            let dy = w[1][1] - w[0][1];
            (dx * dx + dy * dy).sqrt()
        })
        .sum();
    if profile.closed {
        let first = profile.points[0];
        let last = profile.points[profile.points.len() - 1];
        let dx = last[0] - first[0];
        let dy = last[1] - first[1];
        perim += (dx * dx + dy * dy).sqrt();
    }
    perim
}

/// Build a circular profile with `segments` evenly-spaced points.
#[allow(dead_code)]
pub fn circle_profile(radius: f32, segments: u32) -> SweepProfile {
    let pts: Vec<[f32; 2]> = (0..segments)
        .map(|i| {
            let angle = 2.0 * PI * i as f32 / segments as f32;
            [radius * angle.cos(), radius * angle.sin()]
        })
        .collect();
    SweepProfile {
        points: pts,
        closed: true,
    }
}

/// Build a rectangular profile (corners, CCW).
#[allow(dead_code)]
pub fn rectangle_profile(width: f32, height: f32) -> SweepProfile {
    let hw = width * 0.5;
    let hh = height * 0.5;
    SweepProfile {
        points: vec![[-hw, -hh], [hw, -hh], [hw, hh], [-hw, hh]],
        closed: true,
    }
}

// ─── Path constructors ────────────────────────────────────────────────────────

/// A straight-line path from `start` to `end` divided into `segments` equal parts.
#[allow(dead_code)]
pub fn line_path(start: [f32; 3], end: [f32; 3], segments: u32) -> SweepPath {
    let n = (segments + 1) as usize;
    let pts: Vec<[f32; 3]> = (0..n)
        .map(|i| {
            let t = i as f32 / segments as f32;
            [
                start[0] + (end[0] - start[0]) * t,
                start[1] + (end[1] - start[1]) * t,
                start[2] + (end[2] - start[2]) * t,
            ]
        })
        .collect();
    SweepPath {
        points: pts,
        up_vector: [0.0, 1.0, 0.0],
    }
}

/// A helix path: `turns` full revolutions, `radius` in XZ, `height` along Y.
#[allow(dead_code)]
pub fn helix_path(radius: f32, height: f32, turns: f32, segments: u32) -> SweepPath {
    let n = (segments + 1) as usize;
    let pts: Vec<[f32; 3]> = (0..n)
        .map(|i| {
            let t = i as f32 / segments as f32;
            let angle = 2.0 * PI * turns * t;
            [radius * angle.cos(), height * t, radius * angle.sin()]
        })
        .collect();
    SweepPath {
        points: pts,
        up_vector: [0.0, 1.0, 0.0],
    }
}

// ─── Sweep operation ──────────────────────────────────────────────────────────

/// Extrude `profile` along `path` and return the resulting triangle mesh.
#[allow(dead_code)]
pub fn sweep_profile_along_path(profile: &SweepProfile, path: &SweepPath) -> SweepResult {
    let n_path = path.points.len();
    let n_prof = profile.points.len();
    assert!(n_path >= 2, "path needs at least 2 points");
    assert!(!profile.points.is_empty(), "profile must be non-empty");

    // Compute the number of ring segments (for closed profiles, n_prof edges per ring).
    let seg_count = if profile.closed { n_prof } else { n_prof - 1 };

    let arc = path_arc_lengths(path);
    let total_len = *arc.last().unwrap_or(&1.0);
    let total_len = if total_len < 1e-12 { 1.0 } else { total_len };

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    // Compute perimeter of the profile for UV V-axis.
    let perimeter = profile_perimeter(profile);
    let perimeter = if perimeter < 1e-12 { 1.0 } else { perimeter };

    // Emit one ring of vertices per path point.
    for (ring_idx, &origin) in path.points.iter().enumerate() {
        let frame = frame_at_path_point(path, ring_idx);
        let u = arc[ring_idx] / total_len;

        let mut v_accum = 0.0_f32;
        for (prof_idx, &p2d) in profile.points.iter().enumerate() {
            let p3d = transform_profile_point(p2d, frame, origin);
            positions.push(p3d);

            // Outward normal: direction from origin to profile point in the normal/binormal plane.
            let out = normalize3_sweep([p2d[0], 0.0, p2d[1]]); // placeholder magnitude
            let nrm = normalize3_sweep(add3(scale3(frame[1], p2d[0]), scale3(frame[2], p2d[1])));
            let _ = out; // not used directly
            normals.push(nrm);
            uvs.push([u, v_accum / perimeter]);

            // Advance v along the profile perimeter.
            if prof_idx + 1 < profile.points.len() {
                let next = profile.points[prof_idx + 1];
                let dx = next[0] - p2d[0];
                let dy = next[1] - p2d[1];
                v_accum += (dx * dx + dy * dy).sqrt();
            }
        }
    }

    // Build quad-strip indices.
    let mut indices: Vec<u32> = Vec::new();
    for ring in 0..n_path - 1 {
        for seg in 0..seg_count {
            let next_seg = (seg + 1) % n_prof;
            let a = (ring * n_prof + seg) as u32;
            let b = (ring * n_prof + next_seg) as u32;
            let c = ((ring + 1) * n_prof + next_seg) as u32;
            let d = ((ring + 1) * n_prof + seg) as u32;
            // Two triangles per quad.
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }

    SweepResult {
        positions,
        normals,
        uvs,
        indices,
    }
}

// ─── Result accessors ─────────────────────────────────────────────────────────

/// Total number of vertices in the sweep result.
#[allow(dead_code)]
pub fn sweep_result_vertex_count(result: &SweepResult) -> usize {
    result.positions.len()
}

/// Total number of triangular faces in the sweep result.
#[allow(dead_code)]
pub fn sweep_result_face_count(result: &SweepResult) -> usize {
    result.indices.len() / 3
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_profile_point_count() {
        let p = circle_profile(1.0, 8);
        assert_eq!(p.points.len(), 8);
        assert!(p.closed);
    }

    #[test]
    fn test_circle_profile_radius() {
        let r = 2.0_f32;
        let p = circle_profile(r, 16);
        for pt in &p.points {
            let len = (pt[0] * pt[0] + pt[1] * pt[1]).sqrt();
            assert!((len - r).abs() < 1e-5, "point not on circle: {}", len);
        }
    }

    #[test]
    fn test_rectangle_profile_four_corners() {
        let p = rectangle_profile(2.0, 4.0);
        assert_eq!(p.points.len(), 4);
        assert!(p.closed);
    }

    #[test]
    fn test_path_length_line() {
        let path = line_path([0.0, 0.0, 0.0], [0.0, 0.0, 10.0], 10);
        let l = path_length(&path);
        assert!((l - 10.0).abs() < 1e-4, "length = {}", l);
    }

    #[test]
    fn test_helix_path_point_count() {
        let path = helix_path(1.0, 5.0, 2.0, 100);
        assert_eq!(path.points.len(), 101);
    }

    #[test]
    fn test_helix_path_first_last_y() {
        let path = helix_path(1.0, 5.0, 2.0, 100);
        assert!(path.points[0][1].abs() < 1e-5);
        assert!((path.points[100][1] - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_path_arc_lengths_monotone() {
        let path = line_path([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 5);
        let arc = path_arc_lengths(&path);
        for i in 1..arc.len() {
            assert!(arc[i] >= arc[i - 1]);
        }
    }

    #[test]
    fn test_profile_perimeter_circle() {
        let r = 1.0_f32;
        let segments = 360;
        let p = circle_profile(r, segments);
        let perim = profile_perimeter(&p);
        // Approximate 2πr.
        assert!(
            (perim - 2.0 * std::f32::consts::PI * r).abs() < 0.01,
            "perim={}",
            perim
        );
    }

    #[test]
    fn test_sweep_produces_vertices() {
        let profile = circle_profile(0.5, 8);
        let path = line_path([0.0, 0.0, 0.0], [0.0, 5.0, 0.0], 4);
        let result = sweep_profile_along_path(&profile, &path);
        assert!(!result.positions.is_empty());
    }

    #[test]
    fn test_sweep_result_vertex_count() {
        let profile = circle_profile(0.5, 8);
        let path = line_path([0.0, 0.0, 0.0], [0.0, 5.0, 0.0], 4);
        let result = sweep_profile_along_path(&profile, &path);
        // 5 rings × 8 profile points = 40 vertices.
        assert_eq!(sweep_result_vertex_count(&result), 5 * 8);
    }

    #[test]
    fn test_sweep_result_face_count() {
        let profile = circle_profile(0.5, 8);
        let path = line_path([0.0, 0.0, 0.0], [0.0, 5.0, 0.0], 4);
        let result = sweep_profile_along_path(&profile, &path);
        // 4 ring-gaps × 8 quads × 2 triangles = 64 faces.
        assert_eq!(sweep_result_face_count(&result), 64);
    }

    #[test]
    fn test_frame_at_path_point_tangent_direction() {
        let path = line_path([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], 4);
        let frame = frame_at_path_point(&path, 0);
        // Tangent should point along +X.
        assert!(frame[0][0] > 0.9);
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_transform_profile_point_zero() {
        let path = line_path([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 2);
        let frame = frame_at_path_point(&path, 0);
        let p3d = transform_profile_point([0.0, 0.0], frame, path.points[0]);
        // Zero profile point should land exactly at the path origin.
        for (k, val) in p3d.iter().enumerate() {
            assert!(val.abs() < 1e-5, "k={} val={}", k, val);
        }
    }

    #[test]
    fn test_normalize3_sweep_unit() {
        let n = normalize3_sweep([3.0, 4.0, 0.0]);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cross3_sweep_orthogonal() {
        let a = [1.0_f32, 0.0, 0.0];
        let b = [0.0_f32, 1.0, 0.0];
        let c = cross3_sweep(a, b);
        assert!((c[0]).abs() < 1e-6);
        assert!((c[1]).abs() < 1e-6);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sweep_index_count_divisible_by_3() {
        let profile = rectangle_profile(1.0, 1.0);
        let path = line_path([0.0, 0.0, 0.0], [0.0, 3.0, 0.0], 3);
        let result = sweep_profile_along_path(&profile, &path);
        assert_eq!(result.indices.len() % 3, 0);
    }
}
