// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-face tangent frames (T/B/N) computed from UV coordinates.

/// A per-face tangent frame with tangent, bitangent, and normal.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct TangentFrameV2 {
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
    pub normal: [f32; 3],
    pub face_idx: usize,
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
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

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-9 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

/// Compute per-face tangent frame from position and UV data.
#[allow(dead_code)]
pub fn compute_tangent_frames(
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
) -> Vec<TangentFrameV2> {
    let n_tri = indices.len() / 3;
    let mut frames = Vec::with_capacity(n_tri);
    for t in 0..n_tri {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];
        let uv0 = if i0 < uvs.len() { uvs[i0] } else { [0.0, 0.0] };
        let uv1 = if i1 < uvs.len() { uvs[i1] } else { [1.0, 0.0] };
        let uv2 = if i2 < uvs.len() { uvs[i2] } else { [0.0, 1.0] };
        let e1 = sub3(p1, p0);
        let e2 = sub3(p2, p0);
        let du1 = uv1[0] - uv0[0];
        let dv1 = uv1[1] - uv0[1];
        let du2 = uv2[0] - uv0[0];
        let dv2 = uv2[1] - uv0[1];
        let denom = du1 * dv2 - du2 * dv1;
        let (tangent, bitangent) = if denom.abs() < 1e-9 {
            ([1.0_f32, 0.0, 0.0], [0.0_f32, 1.0, 0.0])
        } else {
            let r = 1.0 / denom;
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
            (t, b)
        };
        let normal = normalize3(cross3(e1, e2));
        let tangent_n = normalize3(tangent);
        let bitangent_n = normalize3(bitangent);
        frames.push(TangentFrameV2 {
            tangent: tangent_n,
            bitangent: bitangent_n,
            normal,
            face_idx: t,
        });
    }
    frames
}

/// Check if all tangent frames are orthonormal within tolerance.
#[allow(dead_code)]
pub fn frames_orthonormal(frames: &[TangentFrameV2], tol: f32) -> bool {
    for f in frames {
        let tt = dot3(f.tangent, f.tangent) - 1.0;
        let bb = dot3(f.bitangent, f.bitangent) - 1.0;
        let nn = dot3(f.normal, f.normal) - 1.0;
        let tn = dot3(f.tangent, f.normal).abs();
        let tb = dot3(f.tangent, f.bitangent).abs();
        if tt.abs() > tol || bb.abs() > tol || nn.abs() > tol || tn > tol || tb > tol {
            return false;
        }
    }
    true
}

/// Average tangent direction across all faces.
#[allow(dead_code)]
pub fn average_tangent_v2(frames: &[TangentFrameV2]) -> [f32; 3] {
    if frames.is_empty() {
        return [0.0; 3];
    }
    let mut avg = [0.0_f32; 3];
    for f in frames {
        avg[0] += f.tangent[0];
        avg[1] += f.tangent[1];
        avg[2] += f.tangent[2];
    }
    let n = frames.len() as f32;
    normalize3([avg[0] / n, avg[1] / n, avg[2] / n])
}

/// Number of frames with tangent and normal nearly parallel (degenerate).
#[allow(dead_code)]
pub fn degenerate_frame_count(frames: &[TangentFrameV2], threshold: f32) -> usize {
    frames
        .iter()
        .filter(|f| dot3(f.tangent, f.normal).abs() > threshold)
        .count()
}

/// Convert tangent frame from world space to local (tangent) space.
#[allow(dead_code)]
pub fn world_to_tangent_space(frame: &TangentFrameV2, world_vec: [f32; 3]) -> [f32; 3] {
    [
        dot3(world_vec, frame.tangent),
        dot3(world_vec, frame.bitangent),
        dot3(world_vec, frame.normal),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_quad_mesh() -> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        (positions, uvs, indices)
    }

    #[test]
    fn frame_count_matches_faces() {
        let (pos, uvs, idx) = flat_quad_mesh();
        let frames = compute_tangent_frames(&pos, &uvs, &idx);
        assert_eq!(frames.len(), 2);
    }

    #[test]
    fn normals_point_up() {
        let (pos, uvs, idx) = flat_quad_mesh();
        let frames = compute_tangent_frames(&pos, &uvs, &idx);
        for f in &frames {
            assert!(
                f.normal[2].abs() > 0.9,
                "normal not pointing up: {:?}",
                f.normal
            );
        }
    }

    #[test]
    fn tangents_unit_length() {
        let (pos, uvs, idx) = flat_quad_mesh();
        let frames = compute_tangent_frames(&pos, &uvs, &idx);
        for f in &frames {
            let len = (f.tangent[0] * f.tangent[0]
                + f.tangent[1] * f.tangent[1]
                + f.tangent[2] * f.tangent[2])
                .sqrt();
            assert!((len - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn frames_orthonormal_flat() {
        let (pos, uvs, idx) = flat_quad_mesh();
        let frames = compute_tangent_frames(&pos, &uvs, &idx);
        assert!(frames_orthonormal(&frames, 0.01));
    }

    #[test]
    fn average_tangent_unit_length() {
        let (pos, uvs, idx) = flat_quad_mesh();
        let frames = compute_tangent_frames(&pos, &uvs, &idx);
        let avg = average_tangent_v2(&frames);
        let len = (avg[0] * avg[0] + avg[1] * avg[1] + avg[2] * avg[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn degenerate_count_flat_mesh_zero() {
        let (pos, uvs, idx) = flat_quad_mesh();
        let frames = compute_tangent_frames(&pos, &uvs, &idx);
        let count = degenerate_frame_count(&frames, 0.1);
        assert_eq!(count, 0);
    }

    #[test]
    fn world_to_tangent_space_normal() {
        let (pos, uvs, idx) = flat_quad_mesh();
        let frames = compute_tangent_frames(&pos, &uvs, &idx);
        let f = &frames[0];
        let ts = world_to_tangent_space(f, f.normal);
        assert!(
            (ts[2] - 1.0).abs() < 0.05,
            "normal in tangent space should be ~(0,0,1)"
        );
    }

    #[test]
    fn empty_frames_average_zero() {
        let avg = average_tangent_v2(&[]);
        assert_eq!(avg, [0.0; 3]);
    }

    #[test]
    fn face_idx_sequential() {
        let (pos, uvs, idx) = flat_quad_mesh();
        let frames = compute_tangent_frames(&pos, &uvs, &idx);
        for (i, f) in frames.iter().enumerate() {
            assert_eq!(f.face_idx, i);
        }
    }

    #[test]
    fn empty_mesh_no_frames() {
        let frames = compute_tangent_frames(&[], &[], &[]);
        assert!(frames.is_empty());
    }
}
