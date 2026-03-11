// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Select faces by normal angle.

/// Compute the face normal of a triangle (v0, v1, v2).
pub fn face_normal_3(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < f32::EPSILON {
        return [0.0; 3];
    }
    [n[0] / len, n[1] / len, n[2] / len]
}

/// Angle in degrees between two unit normals.
pub fn angle_between_normals_deg(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
    dot.acos().to_degrees()
}

/// Select face indices whose normal is within `max_angle_deg` of `reference_normal`.
pub fn select_faces_by_angle(
    positions: &[[f32; 3]],
    indices: &[u32],
    reference_normal: [f32; 3],
    max_angle_deg: f32,
) -> Vec<usize> {
    indices
        .chunks(3)
        .enumerate()
        .filter_map(|(fi, tri)| {
            if tri.len() < 3 {
                return None;
            }
            let v0 = positions[tri[0] as usize];
            let v1 = positions[tri[1] as usize];
            let v2 = positions[tri[2] as usize];
            let n = face_normal_3(v0, v1, v2);
            let angle = angle_between_normals_deg(n, reference_normal);
            if angle <= max_angle_deg {
                Some(fi)
            } else {
                None
            }
        })
        .collect()
}

/// Select faces that are nearly flat (normal near given axis, threshold in degrees).
pub fn select_flat_faces(
    positions: &[[f32; 3]],
    indices: &[u32],
    up: [f32; 3],
    threshold_deg: f32,
) -> Vec<usize> {
    select_faces_by_angle(positions, indices, up, threshold_deg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_quad() -> (Vec<[f32; 3]>, Vec<u32>) {
        let p = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let i = vec![0, 1, 2, 0, 2, 3];
        (p, i)
    }

    #[test]
    fn test_face_normal_up() {
        /* XY-plane triangle → normal is [0,0,1] */
        let n = face_normal_3([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((n[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_angle_between_same() {
        let a = [0.0f32, 0.0, 1.0];
        assert!(angle_between_normals_deg(a, a) < 1e-4);
    }

    #[test]
    fn test_angle_between_perpendicular() {
        let a = [0.0f32, 0.0, 1.0];
        let b = [1.0, 0.0, 0.0];
        let deg = angle_between_normals_deg(a, b);
        assert!((deg - 90.0).abs() < 1e-3);
    }

    #[test]
    fn test_select_faces_by_angle_all() {
        let (p, i) = flat_quad();
        let sel = select_faces_by_angle(&p, &i, [0.0, 0.0, 1.0], 5.0);
        assert_eq!(sel.len(), 2);
    }

    #[test]
    fn test_select_faces_by_angle_none() {
        let (p, i) = flat_quad();
        /* looking for downward faces → none */
        let sel = select_faces_by_angle(&p, &i, [0.0, 0.0, -1.0], 5.0);
        assert_eq!(sel.len(), 0);
    }

    #[test]
    fn test_select_flat_faces() {
        let (p, i) = flat_quad();
        let sel = select_flat_faces(&p, &i, [0.0, 0.0, 1.0], 1.0);
        assert_eq!(sel.len(), 2);
    }
}
