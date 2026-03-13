// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-face normal export for triangle meshes.

/// Per-face normal export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceNormalExport {
    pub normals: Vec<[f32; 3]>,
}

/// Compute normalized face normal for a triangle.
#[allow(dead_code)]
pub fn compute_face_normal_fn(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0] / len, n[1] / len, n[2] / len]
    }
}

/// Build per-face normal export from a triangle mesh.
#[allow(dead_code)]
pub fn export_face_normals(positions: &[[f32; 3]], indices: &[u32]) -> FaceNormalExport {
    let face_count = indices.len() / 3;
    let mut normals = Vec::with_capacity(face_count);
    for f in 0..face_count {
        let i0 = indices[f * 3] as usize;
        let i1 = indices[f * 3 + 1] as usize;
        let i2 = indices[f * 3 + 2] as usize;
        normals.push(compute_face_normal_fn(
            positions[i0],
            positions[i1],
            positions[i2],
        ));
    }
    FaceNormalExport { normals }
}

/// Face count.
#[allow(dead_code)]
pub fn fn_face_count(e: &FaceNormalExport) -> usize {
    e.normals.len()
}

/// Get normal at face index.
#[allow(dead_code)]
pub fn get_face_normal(e: &FaceNormalExport, idx: usize) -> Option<[f32; 3]> {
    e.normals.get(idx).copied()
}

/// Average normal (component-wise).
#[allow(dead_code)]
pub fn avg_face_normal(e: &FaceNormalExport) -> [f32; 3] {
    if e.normals.is_empty() {
        return [0.0; 3];
    }
    let n = e.normals.len() as f32;
    let sum = e.normals.iter().fold([0.0f32; 3], |acc, &n| {
        [acc[0] + n[0], acc[1] + n[1], acc[2] + n[2]]
    });
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Validate: all normals are unit length within tolerance.
#[allow(dead_code)]
pub fn validate_face_normals(e: &FaceNormalExport, tol: f32) -> bool {
    e.normals.iter().all(|&n| {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        (len - 1.0).abs() < tol
    })
}

/// Export to CSV string.
#[allow(dead_code)]
pub fn face_normals_to_csv(e: &FaceNormalExport) -> String {
    let mut s = "face,nx,ny,nz\n".to_string();
    for (i, &n) in e.normals.iter().enumerate() {
        s.push_str(&format!("{},{:.6},{:.6},{:.6}\n", i, n[0], n[1], n[2]));
    }
    s
}

/// Export to JSON.
#[allow(dead_code)]
pub fn face_normal_to_json(e: &FaceNormalExport) -> String {
    format!("{{\"face_count\":{}}}", fn_face_count(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tri_up() -> ([f32; 3], [f32; 3], [f32; 3]) {
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0])
    }

    #[test]
    fn test_compute_face_normal_up() {
        let (a, b, c) = unit_tri_up();
        let n = compute_face_normal_fn(a, b, c);
        assert!((n[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_face_normal_degenerate() {
        let n = compute_face_normal_fn([0.0; 3], [0.0; 3], [0.0; 3]);
        // falls back to [0,0,1]
        assert!((n[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_export_face_normals_empty() {
        let e = export_face_normals(&[], &[]);
        assert_eq!(fn_face_count(&e), 0);
    }

    #[test]
    fn test_export_face_normals_single() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let e = export_face_normals(&pos, &idx);
        assert_eq!(fn_face_count(&e), 1);
    }

    #[test]
    fn test_get_face_normal() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let e = export_face_normals(&pos, &idx);
        assert!(get_face_normal(&e, 0).is_some());
        assert!(get_face_normal(&e, 99).is_none());
    }

    #[test]
    fn test_validate_face_normals() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let e = export_face_normals(&pos, &idx);
        assert!(validate_face_normals(&e, 1e-4));
    }

    #[test]
    fn test_avg_face_normal_single() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let e = export_face_normals(&pos, &idx);
        let avg = avg_face_normal(&e);
        assert!((avg[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_face_normals_to_csv() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let e = export_face_normals(&pos, &idx);
        let csv = face_normals_to_csv(&e);
        assert!(csv.contains("face,nx,ny,nz"));
    }

    #[test]
    fn test_face_normal_to_json() {
        let e = export_face_normals(&[], &[]);
        let j = face_normal_to_json(&e);
        assert!(j.contains("\"face_count\":0"));
    }

    #[test]
    fn test_normals_unit_length() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let e = export_face_normals(&pos, &idx);
        let n = e.normals[0];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }
}
