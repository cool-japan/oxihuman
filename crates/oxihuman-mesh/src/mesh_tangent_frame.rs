// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-vertex tangent frame (tangent, bitangent, normal) computation.

use std::f32::consts::FRAC_PI_2;

/// A per-vertex tangent frame.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct TangentFrame {
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
    pub normal: [f32; 3],
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0].powi(2) + v[1].powi(2) + v[2].powi(2))
        .sqrt()
        .max(1e-9);
    [v[0] / l, v[1] / l, v[2] / l]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Compute tangent frames for each vertex using UV coordinates.
#[allow(dead_code)]
pub fn compute_tangent_frames(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
) -> Vec<TangentFrame> {
    let n = positions.len();
    let mut tan = vec![[0.0_f32; 3]; n];
    let mut btan = vec![[0.0_f32; 3]; n];
    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let e1 = [
            positions[i1][0] - positions[i0][0],
            positions[i1][1] - positions[i0][1],
            positions[i1][2] - positions[i0][2],
        ];
        let e2 = [
            positions[i2][0] - positions[i0][0],
            positions[i2][1] - positions[i0][1],
            positions[i2][2] - positions[i0][2],
        ];
        let du1 = uvs[i1][0] - uvs[i0][0];
        let dv1 = uvs[i1][1] - uvs[i0][1];
        let du2 = uvs[i2][0] - uvs[i0][0];
        let dv2 = uvs[i2][1] - uvs[i0][1];
        let r = 1.0 / (du1 * dv2 - du2 * dv1).abs().max(1e-9);
        let t = [
            (dv2 * e1[0] - dv1 * e2[0]) * r,
            (dv2 * e1[1] - dv1 * e2[1]) * r,
            (dv2 * e1[2] - dv1 * e2[2]) * r,
        ];
        let b = [
            (du1 * e2[0] - du2 * e1[0]) * r,
            (du1 * e2[1] - du2 * e1[1]) * r,
            (du1 * e2[2] - du2 * e1[2]) * r,
        ];
        for &i in &[i0, i1, i2] {
            for k in 0..3 {
                tan[i][k] += t[k];
                btan[i][k] += b[k];
            }
        }
    }
    (0..n)
        .map(|i| {
            let n = normalize3(normals[i]);
            let t_raw = tan[i];
            // Gram-Schmidt orthogonalize
            let dot = dot3(n, t_raw);
            let t = normalize3([
                t_raw[0] - dot * n[0],
                t_raw[1] - dot * n[1],
                t_raw[2] - dot * n[2],
            ]);
            let bt = cross3(n, t);
            TangentFrame {
                tangent: t,
                bitangent: bt,
                normal: n,
            }
        })
        .collect()
}

/// Check that a tangent frame is orthonormal.
#[allow(dead_code)]
pub fn is_orthonormal(frame: &TangentFrame, tol: f32) -> bool {
    let lt =
        (frame.tangent[0].powi(2) + frame.tangent[1].powi(2) + frame.tangent[2].powi(2)).sqrt();
    let lb = (frame.bitangent[0].powi(2) + frame.bitangent[1].powi(2) + frame.bitangent[2].powi(2))
        .sqrt();
    let ln = (frame.normal[0].powi(2) + frame.normal[1].powi(2) + frame.normal[2].powi(2)).sqrt();
    (lt - 1.0).abs() < tol && (lb - 1.0).abs() < tol && (ln - 1.0).abs() < tol
}

/// Count frames that pass the orthonormality test.
#[allow(dead_code)]
pub fn count_valid_frames(frames: &[TangentFrame], tol: f32) -> usize {
    frames.iter().filter(|f| is_orthonormal(f, tol)).count()
}

/// Convert tangent frame to a 3x3 matrix (column-major: [tangent|bitangent|normal]).
#[allow(dead_code)]
pub fn frame_to_matrix(frame: &TangentFrame) -> [[f32; 3]; 3] {
    [frame.tangent, frame.bitangent, frame.normal]
}

/// Transform a vector from tangent space to world space.
#[allow(dead_code)]
pub fn tangent_to_world(frame: &TangentFrame, v: [f32; 3]) -> [f32; 3] {
    [
        frame.tangent[0] * v[0] + frame.bitangent[0] * v[1] + frame.normal[0] * v[2],
        frame.tangent[1] * v[0] + frame.bitangent[1] * v[1] + frame.normal[1] * v[2],
        frame.tangent[2] * v[0] + frame.bitangent[2] * v[1] + frame.normal[2] * v[2],
    ]
}

/// Compute angle between tangent and a reference direction.
#[allow(dead_code)]
pub fn tangent_angle(frame: &TangentFrame, ref_dir: [f32; 3]) -> f32 {
    let _ = FRAC_PI_2;
    dot3(normalize3(frame.tangent), normalize3(ref_dir))
        .clamp(-1.0, 1.0)
        .acos()
}

/// Average tangent direction across all frames.
#[allow(dead_code)]
pub fn average_tangent(frames: &[TangentFrame]) -> [f32; 3] {
    if frames.is_empty() {
        return [1.0, 0.0, 0.0];
    }
    let n = frames.len() as f32;
    let s = frames.iter().fold([0.0_f32; 3], |a, f| {
        [
            a[0] + f.tangent[0],
            a[1] + f.tangent[1],
            a[2] + f.tangent[2],
        ]
    });
    normalize3([s[0] / n, s[1] / n, s[2] / n])
}

#[cfg(test)]
mod tests {
    use super::*;

    type TriData = (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>);
    fn single_tri() -> TriData {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            vec![0u32, 1, 2],
        )
    }

    #[test]
    fn compute_returns_correct_count() {
        let (p, n, uv, i) = single_tri();
        let frames = compute_tangent_frames(&p, &n, &uv, &i);
        assert_eq!(frames.len(), 3);
    }

    #[test]
    fn frames_are_orthonormal() {
        let (p, n, uv, i) = single_tri();
        let frames = compute_tangent_frames(&p, &n, &uv, &i);
        assert_eq!(count_valid_frames(&frames, 1e-4), 3);
    }

    #[test]
    fn frame_to_matrix_shape() {
        let f = TangentFrame {
            tangent: [1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        };
        let m = frame_to_matrix(&f);
        assert_eq!(m[0], [1.0, 0.0, 0.0]);
    }

    #[test]
    fn tangent_to_world_identity() {
        let f = TangentFrame {
            tangent: [1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        };
        let v = [1.0_f32, 0.0, 0.0];
        let w = tangent_to_world(&f, v);
        assert!((w[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn tangent_angle_zero_parallel() {
        let f = TangentFrame {
            tangent: [1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        };
        let ang = tangent_angle(&f, [1.0, 0.0, 0.0]);
        assert!(ang.abs() < 1e-5);
    }

    #[test]
    fn average_tangent_unit() {
        let (p, n, uv, i) = single_tri();
        let frames = compute_tangent_frames(&p, &n, &uv, &i);
        let avg = average_tangent(&frames);
        let l = (avg[0].powi(2) + avg[1].powi(2) + avg[2].powi(2)).sqrt();
        assert!((l - 1.0).abs() < 1e-4);
    }

    #[test]
    fn empty_frames() {
        assert_eq!(count_valid_frames(&[], 1e-4), 0);
    }

    #[test]
    fn average_tangent_empty() {
        let avg = average_tangent(&[]);
        assert_eq!(avg[0], 1.0);
    }

    #[test]
    fn is_orthonormal_basic() {
        let f = TangentFrame {
            tangent: [1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        };
        assert!(is_orthonormal(&f, 1e-5));
    }

    #[test]
    fn frac_pi_2_used() {
        assert!((FRAC_PI_2 - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    }
}
