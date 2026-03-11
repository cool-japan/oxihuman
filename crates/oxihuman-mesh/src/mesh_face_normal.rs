#![allow(dead_code)]
//! Per-face normal computation.

/// A face normal with magnitude.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FaceNormalData {
    pub normal: [f32; 3],
    pub magnitude: f32,
}

/// Compute the normal of a single triangle face.
#[allow(dead_code)]
pub fn compute_face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    if len < 1e-12 {
        return [0.0, 0.0, 0.0];
    }
    [cross[0] / len, cross[1] / len, cross[2] / len]
}

/// Compute all face normals for a mesh.
#[allow(dead_code)]
pub fn all_face_normals(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Vec<[f32; 3]> {
    indices
        .iter()
        .map(|tri| {
            compute_face_normal(
                positions[tri[0] as usize],
                positions[tri[1] as usize],
                positions[tri[2] as usize],
            )
        })
        .collect()
}

/// Check if a face normal is degenerate (zero-length).
#[allow(dead_code)]
pub fn face_normal_is_degenerate(n: [f32; 3]) -> bool {
    let len2 = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
    len2 < 1e-12
}

/// Flip a face normal.
#[allow(dead_code)]
pub fn flip_face_normal(n: [f32; 3]) -> [f32; 3] {
    [-n[0], -n[1], -n[2]]
}

/// Convert face normals to a flat f32 array.
#[allow(dead_code)]
pub fn face_normals_to_array(normals: &[[f32; 3]]) -> Vec<f32> {
    normals.iter().flat_map(|n| n.iter().copied()).collect()
}

/// Compute dot product between two face normals.
#[allow(dead_code)]
pub fn face_normal_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Compute the average face normal of a mesh.
#[allow(dead_code)]
pub fn average_face_normal(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> [f32; 3] {
    let normals = all_face_normals(positions, indices);
    if normals.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let mut sum = [0.0f32; 3];
    for n in &normals {
        sum[0] += n[0];
        sum[1] += n[1];
        sum[2] += n[2];
    }
    let len = (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]).sqrt();
    if len < 1e-12 {
        return [0.0, 0.0, 0.0];
    }
    [sum[0] / len, sum[1] / len, sum[2] / len]
}

/// Compute the variance of face normal directions (1 - avg dot).
#[allow(dead_code)]
pub fn face_normal_variance(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    let normals = all_face_normals(positions, indices);
    if normals.len() < 2 {
        return 0.0;
    }
    let avg = average_face_normal(positions, indices);
    let total: f32 = normals.iter().map(|n| 1.0 - face_normal_dot(*n, avg).abs()).sum();
    total / normals.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_face_normal() {
        let n = compute_face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((n[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_all_face_normals() {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let i = vec![[0u32, 1, 2]];
        let normals = all_face_normals(&p, &i);
        assert_eq!(normals.len(), 1);
    }

    #[test]
    fn test_face_normal_degenerate() {
        assert!(face_normal_is_degenerate([0.0, 0.0, 0.0]));
        assert!(!face_normal_is_degenerate([0.0, 0.0, 1.0]));
    }

    #[test]
    fn test_flip_face_normal() {
        let flipped = flip_face_normal([0.0, 0.0, 1.0]);
        assert!((flipped[2] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_face_normals_to_array() {
        let normals = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let arr = face_normals_to_array(&normals);
        assert_eq!(arr.len(), 6);
        assert!((arr[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_face_normal_dot() {
        let d = face_normal_dot([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]);
        assert!((d - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_average_face_normal() {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let i = vec![[0u32, 1, 2]];
        let avg = average_face_normal(&p, &i);
        assert!((avg[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_average_face_normal_empty() {
        let avg = average_face_normal(&[], &[]);
        assert_eq!(avg, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_face_normal_variance() {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let i = vec![[0u32, 1, 2]];
        let v = face_normal_variance(&p, &i);
        assert!(v >= 0.0);
    }

    #[test]
    fn test_compute_face_normal_degenerate() {
        let n = compute_face_normal([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert_eq!(n, [0.0, 0.0, 0.0]);
    }
}
