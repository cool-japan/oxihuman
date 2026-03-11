#![allow(dead_code)]
//! Normal recomputation from positions.

use std::f32::consts::PI;

/// Normal recomputation configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct NormalRecompute {
    pub smoothing_angle: f32,
    pub vertex_count: usize,
}

/// Recompute flat (per-face) normals.
#[allow(dead_code)]
pub fn recompute_flat_normals(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; positions.len()];
    for tri in indices {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        let n = face_normal(a, b, c);
        for &vi in tri {
            normals[vi as usize] = n;
        }
    }
    normals
}

/// Recompute smooth (area-weighted) normals.
#[allow(dead_code)]
pub fn recompute_smooth_normals(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; positions.len()];
    for tri in indices {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        let n = face_normal_unnormalized(a, b, c);
        for &vi in tri {
            normals[vi as usize][0] += n[0];
            normals[vi as usize][1] += n[1];
            normals[vi as usize][2] += n[2];
        }
    }
    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-12 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        }
    }
    normals
}

/// Recompute angle-weighted normals.
#[allow(dead_code)]
pub fn recompute_weighted_normals(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; positions.len()];
    for tri in indices {
        let p = [
            positions[tri[0] as usize],
            positions[tri[1] as usize],
            positions[tri[2] as usize],
        ];
        let fn_val = face_normal_unnormalized(p[0], p[1], p[2]);
        for i in 0..3 {
            let e1 = [
                p[(i + 1) % 3][0] - p[i][0],
                p[(i + 1) % 3][1] - p[i][1],
                p[(i + 1) % 3][2] - p[i][2],
            ];
            let e2 = [
                p[(i + 2) % 3][0] - p[i][0],
                p[(i + 2) % 3][1] - p[i][1],
                p[(i + 2) % 3][2] - p[i][2],
            ];
            let l1 = (e1[0] * e1[0] + e1[1] * e1[1] + e1[2] * e1[2]).sqrt();
            let l2 = (e2[0] * e2[0] + e2[1] * e2[1] + e2[2] * e2[2]).sqrt();
            let angle = if l1 > 1e-12 && l2 > 1e-12 {
                let dot = (e1[0] * e2[0] + e1[1] * e2[1] + e1[2] * e2[2]) / (l1 * l2);
                dot.clamp(-1.0, 1.0).acos()
            } else {
                0.0
            };
            let vi = tri[i] as usize;
            normals[vi][0] += fn_val[0] * angle;
            normals[vi][1] += fn_val[1] * angle;
            normals[vi][2] += fn_val[2] * angle;
        }
    }
    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-12 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        }
    }
    normals
}

/// Return the default smoothing angle threshold (in radians).
#[allow(dead_code)]
pub fn normal_smoothing_angle() -> f32 {
    PI / 3.0
}

/// Smooth normals within face groups sharing edges below angle threshold.
#[allow(dead_code)]
pub fn smooth_normals_by_group(
    positions: &[[f32; 3]],
    indices: &[[u32; 3]],
    angle_threshold: f32,
) -> Vec<[f32; 3]> {
    let face_normals: Vec<[f32; 3]> = indices
        .iter()
        .map(|tri| face_normal(positions[tri[0] as usize], positions[tri[1] as usize], positions[tri[2] as usize]))
        .collect();
    let mut normals = vec![[0.0f32; 3]; positions.len()];
    let cos_threshold = angle_threshold.cos();
    for (fi, tri) in indices.iter().enumerate() {
        for &vi in tri {
            let mut sum = [0.0f32; 3];
            for (fj, tri2) in indices.iter().enumerate() {
                if tri2.contains(&vi) {
                    let dot = face_normals[fi][0] * face_normals[fj][0]
                        + face_normals[fi][1] * face_normals[fj][1]
                        + face_normals[fi][2] * face_normals[fj][2];
                    if dot >= cos_threshold {
                        sum[0] += face_normals[fj][0];
                        sum[1] += face_normals[fj][1];
                        sum[2] += face_normals[fj][2];
                    }
                }
            }
            let len = (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]).sqrt();
            if len > 1e-12 {
                normals[vi as usize] = [sum[0] / len, sum[1] / len, sum[2] / len];
            }
        }
    }
    normals
}

/// Return the vertex count for a recomputation.
#[allow(dead_code)]
pub fn normal_recompute_count(positions: &[[f32; 3]]) -> usize {
    positions.len()
}

/// Auto-smooth normals with default angle.
#[allow(dead_code)]
pub fn auto_smooth_normals(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Vec<[f32; 3]> {
    smooth_normals_by_group(positions, indices, normal_smoothing_angle())
}

/// Check if any normal is flipped (pointing opposite to face normal).
#[allow(dead_code)]
pub fn normal_flip_check(positions: &[[f32; 3]], normals: &[[f32; 3]], indices: &[[u32; 3]]) -> Vec<usize> {
    let mut flipped = Vec::new();
    for (fi, tri) in indices.iter().enumerate() {
        let fn_val = face_normal(
            positions[tri[0] as usize],
            positions[tri[1] as usize],
            positions[tri[2] as usize],
        );
        for &vi in tri {
            let vn = normals[vi as usize];
            let dot = fn_val[0] * vn[0] + fn_val[1] * vn[1] + fn_val[2] * vn[2];
            if dot < 0.0 {
                flipped.push(fi);
                break;
            }
        }
    }
    flipped
}

fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let n = face_normal_unnormalized(a, b, c);
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        [n[0] / len, n[1] / len, n[2] / len]
    }
}

fn face_normal_unnormalized(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        let i = vec![[0, 1, 2], [1, 3, 2]];
        (p, i)
    }

    #[test]
    fn test_recompute_flat_normals() {
        let (p, i) = simple_mesh();
        let normals = recompute_flat_normals(&p, &i);
        assert_eq!(normals.len(), p.len());
        assert!((normals[0][2] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_recompute_smooth_normals() {
        let (p, i) = simple_mesh();
        let normals = recompute_smooth_normals(&p, &i);
        for n in &normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-4);
        }
    }

    #[test]
    fn test_recompute_weighted_normals() {
        let (p, i) = simple_mesh();
        let normals = recompute_weighted_normals(&p, &i);
        assert_eq!(normals.len(), p.len());
    }

    #[test]
    fn test_normal_smoothing_angle() {
        let angle = normal_smoothing_angle();
        assert!((angle - PI / 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_smooth_normals_by_group() {
        let (p, i) = simple_mesh();
        let normals = smooth_normals_by_group(&p, &i, PI / 4.0);
        assert_eq!(normals.len(), p.len());
    }

    #[test]
    fn test_normal_recompute_count() {
        let (p, _) = simple_mesh();
        assert_eq!(normal_recompute_count(&p), 4);
    }

    #[test]
    fn test_auto_smooth_normals() {
        let (p, i) = simple_mesh();
        let normals = auto_smooth_normals(&p, &i);
        assert_eq!(normals.len(), p.len());
    }

    #[test]
    fn test_normal_flip_check() {
        let (p, i) = simple_mesh();
        let normals = recompute_smooth_normals(&p, &i);
        let flipped = normal_flip_check(&p, &normals, &i);
        assert!(flipped.is_empty());
    }

    #[test]
    fn test_normal_flip_check_flipped() {
        let (p, i) = simple_mesh();
        let normals = vec![[0.0, 0.0, -1.0]; p.len()];
        let flipped = normal_flip_check(&p, &normals, &i);
        assert!(!flipped.is_empty());
    }

    #[test]
    fn test_empty_mesh_normals() {
        let normals = recompute_smooth_normals(&[], &[]);
        assert!(normals.is_empty());
    }
}
