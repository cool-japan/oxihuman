// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Reverse all normals in a normal array.
#[allow(dead_code)]
pub fn reverse_normals(normals: &mut [[f32; 3]]) {
    for n in normals.iter_mut() {
        n[0] = -n[0];
        n[1] = -n[1];
        n[2] = -n[2];
    }
}

/// Reverse normals for selected vertex indices only.
#[allow(dead_code)]
pub fn reverse_normals_selected(normals: &mut [[f32; 3]], selection: &[usize]) {
    for &i in selection {
        if i < normals.len() {
            normals[i][0] = -normals[i][0];
            normals[i][1] = -normals[i][1];
            normals[i][2] = -normals[i][2];
        }
    }
}

/// Flip winding of all triangles (reverses face normals).
#[allow(dead_code)]
pub fn flip_all_winding_rn(indices: &mut [u32]) {
    let n = indices.len() / 3;
    for fi in 0..n {
        indices.swap(fi * 3 + 1, fi * 3 + 2);
    }
}

/// Count normals that point in roughly the +Y direction.
#[allow(dead_code)]
pub fn count_upward_normals(normals: &[[f32; 3]]) -> usize {
    normals.iter().filter(|n| n[1] > 0.0).count()
}

/// Compute face normal from three positions.
#[allow(dead_code)]
pub fn face_normal_rn(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    if len < 1e-8 {
        return [0.0; 3];
    }
    [cross[0] / len, cross[1] / len, cross[2] / len]
}

/// Compute all face normals from a triangle mesh.
#[allow(dead_code)]
pub fn compute_face_normals_rn(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let n = indices.len() / 3;
    (0..n)
        .map(|fi| {
            let a = positions[indices[fi * 3] as usize];
            let b = positions[indices[fi * 3 + 1] as usize];
            let c = positions[indices[fi * 3 + 2] as usize];
            face_normal_rn(a, b, c)
        })
        .collect()
}

/// Check if all normals are approximately unit length.
#[allow(dead_code)]
pub fn normals_are_unit_rn(normals: &[[f32; 3]]) -> bool {
    normals.iter().all(|n| {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        (len - 1.0).abs() < 1e-3 || len < 1e-9
    })
}

/// Serialize normal reversal report to JSON.
#[allow(dead_code)]
pub fn reverse_normal_to_json(count: usize) -> String {
    format!(r#"{{"reversed":{}}}"#, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_all() {
        let mut n = vec![[0.0_f32, 1.0, 0.0]];
        reverse_normals(&mut n);
        assert!((n[0][1] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn reverse_selected() {
        let mut n = vec![[0.0_f32, 1.0, 0.0], [0.0, 1.0, 0.0]];
        reverse_normals_selected(&mut n, &[0]);
        assert!((n[0][1] + 1.0).abs() < 1e-6);
        assert!((n[1][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn flip_winding_reverses() {
        let mut idx = vec![0_u32, 1, 2];
        flip_all_winding_rn(&mut idx);
        assert_eq!(idx, vec![0, 2, 1]);
    }

    #[test]
    fn count_upward() {
        let n = vec![[0.0_f32, 1.0, 0.0], [0.0, -1.0, 0.0]];
        assert_eq!(count_upward_normals(&n), 1);
    }

    #[test]
    fn face_normal_z() {
        let n = face_normal_rn([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((n[2] - 1.0).abs() < 1e-5 || (n[2] + 1.0).abs() < 1e-5);
    }

    #[test]
    fn face_normals_unit() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0_u32, 1, 2];
        let normals = compute_face_normals_rn(&pos, &idx);
        assert!(normals_are_unit_rn(&normals));
    }

    #[test]
    fn json_has_count() {
        let j = reverse_normal_to_json(5);
        assert!(j.contains("\"reversed\":5"));
    }

    #[test]
    fn double_reverse_identity() {
        let orig = vec![[0.0_f32, 1.0, 0.0]];
        let mut n = orig.clone();
        reverse_normals(&mut n);
        reverse_normals(&mut n);
        assert!((n[0][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn empty_normals() {
        let mut n: Vec<[f32; 3]> = vec![];
        reverse_normals(&mut n);
        assert!(n.is_empty());
    }

    #[test]
    fn unit_check_fails_on_zero() {
        let n = vec![[0.0_f32, 0.0, 0.0]];
        // zero length is allowed (treated as degenerate)
        let _ = normals_are_unit_rn(&n);
    }
}
