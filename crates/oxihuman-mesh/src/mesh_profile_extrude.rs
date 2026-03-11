// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A 2D profile to extrude.
#[allow(dead_code)]
pub struct Profile2D {
    pub points: Vec<[f32; 2]>,
}

/// Result of extruding a profile along the Z axis.
#[allow(dead_code)]
pub struct ExtrudeProfileResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub depth: f32,
}

/// Extrude a 2D profile by a given depth along the Z axis.
#[allow(dead_code)]
pub fn extrude_profile(profile: &Profile2D, depth: f32) -> ExtrudeProfileResult {
    let n = profile.points.len();
    assert!(n >= 2);
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n * 2);
    let mut indices: Vec<u32> = Vec::new();

    for &[x, y] in &profile.points {
        positions.push([x, y, 0.0]);
    }
    for &[x, y] in &profile.points {
        positions.push([x, y, depth]);
    }

    for i in 0..n {
        let j = (i + 1) % n;
        let a = i as u32;
        let b = j as u32;
        let c = (j + n) as u32;
        let d = (i + n) as u32;
        indices.extend_from_slice(&[a, b, c, a, c, d]);
    }

    ExtrudeProfileResult {
        positions,
        indices,
        depth,
    }
}

/// Number of side faces in an extrusion.
#[allow(dead_code)]
pub fn extrude_side_face_count(profile_len: usize) -> usize {
    profile_len * 2
}

/// Build a simple square profile.
#[allow(dead_code)]
pub fn square_profile(half: f32) -> Profile2D {
    Profile2D {
        points: vec![[-half, -half], [half, -half], [half, half], [-half, half]],
    }
}

/// Return vertex count of an extruded profile.
#[allow(dead_code)]
pub fn extrude_vertex_count(profile_len: usize) -> usize {
    profile_len * 2
}

/// Compute the centroid of extruded positions.
#[allow(dead_code)]
pub fn extrude_centroid(result: &ExtrudeProfileResult) -> [f32; 3] {
    let n = result.positions.len() as f32;
    let mut s = [0.0_f32; 3];
    for p in &result.positions {
        s[0] += p[0];
        s[1] += p[1];
        s[2] += p[2];
    }
    [s[0] / n, s[1] / n, s[2] / n]
}

/// Validate that extruded result has indices in bounds.
#[allow(dead_code)]
pub fn extrude_indices_valid(result: &ExtrudeProfileResult) -> bool {
    let n = result.positions.len() as u32;
    result.indices.iter().all(|&i| i < n)
}

/// Serialize extrusion metadata to JSON.
#[allow(dead_code)]
pub fn extrude_to_json(result: &ExtrudeProfileResult) -> String {
    format!(
        r#"{{"vertices":{},"triangles":{},"depth":{:.4}}}"#,
        result.positions.len(),
        result.indices.len() / 3,
        result.depth
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_extrude_vertex_count() {
        let p = square_profile(1.0);
        let r = extrude_profile(&p, 2.0);
        assert_eq!(r.positions.len(), extrude_vertex_count(4));
    }

    #[test]
    fn side_face_count() {
        assert_eq!(extrude_side_face_count(4), 8);
    }

    #[test]
    fn indices_in_bounds() {
        let p = square_profile(1.0);
        let r = extrude_profile(&p, 1.0);
        assert!(extrude_indices_valid(&r));
    }

    #[test]
    fn centroid_at_half_depth() {
        let p = square_profile(1.0);
        let r = extrude_profile(&p, 2.0);
        let c = extrude_centroid(&r);
        assert!((c[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn json_contains_depth() {
        let p = square_profile(1.0);
        let r = extrude_profile(&p, 3.0);
        let j = extrude_to_json(&r);
        assert!(j.contains("3.0000"));
    }

    #[test]
    fn depth_stored() {
        let p = square_profile(1.0);
        let r = extrude_profile(&p, 5.0);
        assert!((r.depth - 5.0).abs() < 1e-6);
    }

    #[test]
    fn nonempty_indices() {
        let p = square_profile(1.0);
        let r = extrude_profile(&p, 1.0);
        assert!(!r.indices.is_empty());
    }

    #[test]
    fn triangle_count() {
        let p = square_profile(1.0);
        let r = extrude_profile(&p, 1.0);
        assert_eq!(r.indices.len() % 3, 0);
    }

    #[test]
    fn square_profile_four_points() {
        let p = square_profile(1.0);
        assert_eq!(p.points.len(), 4);
    }

    #[test]
    fn two_point_profile() {
        let p = Profile2D {
            points: vec![[0.0, 0.0], [1.0, 0.0]],
        };
        let r = extrude_profile(&p, 1.0);
        assert!(!r.positions.is_empty());
    }
}
