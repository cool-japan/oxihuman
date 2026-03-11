//! Compute per-vertex local coordinate frames (tangent, bitangent, normal)
//! from UV and position data.
//!
//! Each vertex frame is built by accumulating the tangent and bitangent
//! contributions from all triangles that share the vertex, then
//! orthonormalising the result via the Gram-Schmidt process. The handedness
//! of the frame is derived from the sign of the triple product of the
//! tangent, bitangent and geometric normal.

#![allow(dead_code)]

/// Configuration for local-frame computation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LocalFrameConfig {
    /// When `true` the frame is right-handed; when `false` it is left-handed.
    pub right_handed: bool,
    /// Epsilon used during normalisation to avoid division by zero.
    pub epsilon: f32,
}

/// A per-vertex tangent-space frame.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct VertexFrame {
    /// Tangent direction (aligns with increasing U).
    pub tangent: [f32; 3],
    /// Bitangent direction (aligns with increasing V).
    pub bitangent: [f32; 3],
    /// Geometric/shading normal.
    pub normal: [f32; 3],
    /// Handedness: +1.0 for right-handed, −1.0 for left-handed.
    pub handedness: f32,
}

/// Result of a local-frame computation pass.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LocalFrameResult {
    /// One frame per vertex, indexed identically to the position array.
    pub frames: Vec<VertexFrame>,
    /// Whether the frames have been orthonormalised.
    pub orthonormal: bool,
}

/// Return sensible defaults for [`LocalFrameConfig`].
#[allow(dead_code)]
pub fn default_local_frame_config() -> LocalFrameConfig {
    LocalFrameConfig {
        right_handed: true,
        epsilon: 1e-7,
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn v3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn v3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn v3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn v3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn v3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn v3_normalize(v: [f32; 3], eps: f32) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < eps {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute per-vertex local frames from position, UV, and triangle data.
///
/// `positions` — vertex positions as `[x, y, z]`.
/// `uvs`       — vertex texture coordinates as `[u, v]` (one per vertex).
/// `triangles` — triangle indices as `[i0, i1, i2]`.
#[allow(dead_code)]
pub fn compute_local_frames(
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    triangles: &[[usize; 3]],
    config: &LocalFrameConfig,
) -> LocalFrameResult {
    let n_verts = positions.len();
    let eps = config.epsilon;

    let mut tan1 = vec![[0.0_f32; 3]; n_verts];
    let mut tan2 = vec![[0.0_f32; 3]; n_verts];
    let mut norms = vec![[0.0_f32; 3]; n_verts];

    for tri in triangles {
        let i0 = tri[0];
        let i1 = tri[1];
        let i2 = tri[2];

        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];

        let uv0 = uvs[i0];
        let uv1 = uvs[i1];
        let uv2 = uvs[i2];

        let e1 = v3_sub(p1, p0);
        let e2 = v3_sub(p2, p0);

        let du1 = uv1[0] - uv0[0];
        let dv1 = uv1[1] - uv0[1];
        let du2 = uv2[0] - uv0[0];
        let dv2 = uv2[1] - uv0[1];

        let r_denom = du1 * dv2 - du2 * dv1;
        let r = if r_denom.abs() > eps { 1.0 / r_denom } else { 0.0 };

        let t = v3_scale(v3_sub(v3_scale(e1, dv2), v3_scale(e2, dv1)), r);
        let b = v3_scale(v3_sub(v3_scale(e2, du1), v3_scale(e1, du2)), r);

        let fn_ = v3_cross(e1, e2); // face normal (unnormalised)

        for &idx in &[i0, i1, i2] {
            tan1[idx] = v3_add(tan1[idx], t);
            tan2[idx] = v3_add(tan2[idx], b);
            norms[idx] = v3_add(norms[idx], fn_);
        }
    }

    let frames: Vec<VertexFrame> = (0..n_verts)
        .map(|i| {
            let n = v3_normalize(norms[i], eps);
            let t = tan1[i];
            // Gram-Schmidt orthogonalise tangent
            let t_ortho = v3_normalize(
                v3_sub(t, v3_scale(n, v3_dot(n, t))),
                eps,
            );
            // Compute handedness
            let cross = v3_cross(n, t_ortho);
            let hand_sign = if v3_dot(cross, tan2[i]) < 0.0 { -1.0_f32 } else { 1.0_f32 };
            let hand = if config.right_handed { hand_sign } else { -hand_sign };
            // Bitangent = N × T * handedness
            let bt = v3_scale(v3_cross(n, t_ortho), hand);

            VertexFrame {
                tangent: t_ortho,
                bitangent: bt,
                normal: n,
                handedness: hand,
            }
        })
        .collect();

    LocalFrameResult { frames, orthonormal: true }
}

/// Return the number of vertex frames.
#[allow(dead_code)]
pub fn local_frame_count(result: &LocalFrameResult) -> usize {
    result.frames.len()
}

/// Return the tangent of the frame at `index`.
#[allow(dead_code)]
pub fn local_frame_tangent(result: &LocalFrameResult, index: usize) -> [f32; 3] {
    result.frames[index].tangent
}

/// Return the bitangent of the frame at `index`.
#[allow(dead_code)]
pub fn local_frame_bitangent(result: &LocalFrameResult, index: usize) -> [f32; 3] {
    result.frames[index].bitangent
}

/// Return the normal of the frame at `index`.
#[allow(dead_code)]
pub fn local_frame_normal(result: &LocalFrameResult, index: usize) -> [f32; 3] {
    result.frames[index].normal
}

/// Serialise the frames to a compact JSON string.
#[allow(dead_code)]
pub fn local_frames_to_json(result: &LocalFrameResult) -> String {
    let frame_strs: Vec<String> = result
        .frames
        .iter()
        .map(|f| {
            format!(
                r#"{{"t":[{:.4},{:.4},{:.4}],"b":[{:.4},{:.4},{:.4}],"n":[{:.4},{:.4},{:.4}],"h":{:.1}}}"#,
                f.tangent[0], f.tangent[1], f.tangent[2],
                f.bitangent[0], f.bitangent[1], f.bitangent[2],
                f.normal[0], f.normal[1], f.normal[2],
                f.handedness,
            )
        })
        .collect();
    format!(r#"{{"orthonormal":{},"frames":[{}]}}"#, result.orthonormal, frame_strs.join(","))
}

/// Return the handedness value (+1 or −1) for the frame at `index`.
#[allow(dead_code)]
pub fn local_frame_handedness(result: &LocalFrameResult, index: usize) -> f32 {
    result.frames[index].handedness
}

/// Return a new result with all frames explicitly orthonormalised.
#[allow(dead_code)]
pub fn local_frame_orthonormalize(result: &LocalFrameResult, epsilon: f32) -> LocalFrameResult {
    let frames = result
        .frames
        .iter()
        .map(|f| {
            let n = v3_normalize(f.normal, epsilon);
            let t_raw = v3_sub(f.tangent, v3_scale(n, v3_dot(n, f.tangent)));
            let t = v3_normalize(t_raw, epsilon);
            let bt = v3_scale(v3_cross(n, t), f.handedness);
            VertexFrame { tangent: t, bitangent: bt, normal: n, handedness: f.handedness }
        })
        .collect();
    LocalFrameResult { frames, orthonormal: true }
}

/// Return `true` if all frames have unit-length tangent and normal vectors.
#[allow(dead_code)]
pub fn local_frames_valid(result: &LocalFrameResult, tolerance: f32) -> bool {
    result.frames.iter().all(|f| {
        let tn = (f.tangent[0] * f.tangent[0]
            + f.tangent[1] * f.tangent[1]
            + f.tangent[2] * f.tangent[2])
            .sqrt();
        let nn = (f.normal[0] * f.normal[0]
            + f.normal[1] * f.normal[1]
            + f.normal[2] * f.normal[2])
            .sqrt();
        (tn - 1.0).abs() <= tolerance && (nn - 1.0).abs() <= tolerance
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::type_complexity)]
    fn simple_quad() -> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<[usize; 3]>) {
        let positions = vec![
            [0.0_f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let uvs = vec![
            [0.0_f32, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ];
        let triangles = vec![[0, 1, 2], [0, 2, 3]];
        (positions, uvs, triangles)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_local_frame_config();
        assert!(cfg.right_handed);
        assert!(cfg.epsilon > 0.0);
    }

    #[test]
    fn test_frame_count_matches_vertex_count() {
        let (p, uv, t) = simple_quad();
        let cfg = default_local_frame_config();
        let res = compute_local_frames(&p, &uv, &t, &cfg);
        assert_eq!(local_frame_count(&res), p.len());
    }

    #[test]
    fn test_frames_valid_after_compute() {
        let (p, uv, t) = simple_quad();
        let cfg = default_local_frame_config();
        let res = compute_local_frames(&p, &uv, &t, &cfg);
        assert!(local_frames_valid(&res, 1e-4));
    }

    #[test]
    fn test_normal_points_up() {
        let (p, uv, t) = simple_quad();
        let cfg = default_local_frame_config();
        let res = compute_local_frames(&p, &uv, &t, &cfg);
        // XY plane → normal should be ±Z
        for i in 0..local_frame_count(&res) {
            let n = local_frame_normal(&res, i);
            assert!(n[2].abs() > 0.9, "normal z={}", n[2]);
        }
    }

    #[test]
    fn test_tangent_unit_length() {
        let (p, uv, t) = simple_quad();
        let cfg = default_local_frame_config();
        let res = compute_local_frames(&p, &uv, &t, &cfg);
        for i in 0..local_frame_count(&res) {
            let tg = local_frame_tangent(&res, i);
            let len = (tg[0] * tg[0] + tg[1] * tg[1] + tg[2] * tg[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-4, "tangent len={}", len);
        }
    }

    #[test]
    fn test_handedness_is_pm_one() {
        let (p, uv, t) = simple_quad();
        let cfg = default_local_frame_config();
        let res = compute_local_frames(&p, &uv, &t, &cfg);
        for i in 0..local_frame_count(&res) {
            let h = local_frame_handedness(&res, i);
            assert!(h == 1.0 || h == -1.0, "handedness={}", h);
        }
    }

    #[test]
    fn test_to_json_contains_frames() {
        let (p, uv, t) = simple_quad();
        let cfg = default_local_frame_config();
        let res = compute_local_frames(&p, &uv, &t, &cfg);
        let json = local_frames_to_json(&res);
        assert!(json.contains("frames"));
        assert!(json.contains("orthonormal"));
    }

    #[test]
    fn test_orthonormalize_preserves_count() {
        let (p, uv, t) = simple_quad();
        let cfg = default_local_frame_config();
        let res = compute_local_frames(&p, &uv, &t, &cfg);
        let res2 = local_frame_orthonormalize(&res, 1e-7);
        assert_eq!(local_frame_count(&res), local_frame_count(&res2));
    }

    #[test]
    fn test_orthonormalize_still_valid() {
        let (p, uv, t) = simple_quad();
        let cfg = default_local_frame_config();
        let res = compute_local_frames(&p, &uv, &t, &cfg);
        let res2 = local_frame_orthonormalize(&res, 1e-7);
        assert!(local_frames_valid(&res2, 1e-4));
    }

    #[test]
    fn test_bitangent_orthogonal_to_normal() {
        let (p, uv, t) = simple_quad();
        let cfg = default_local_frame_config();
        let res = compute_local_frames(&p, &uv, &t, &cfg);
        for i in 0..local_frame_count(&res) {
            let bt = local_frame_bitangent(&res, i);
            let n = local_frame_normal(&res, i);
            let d = v3_dot(bt, n);
            assert!(d.abs() < 1e-4, "bt·n={}", d);
        }
    }
}
