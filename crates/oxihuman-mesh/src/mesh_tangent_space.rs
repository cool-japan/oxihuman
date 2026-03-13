// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Tangent and bitangent computation for normal mapping (MikkTSpace-style).
//!
//! Computes per-triangle and per-vertex tangent frames from UV coordinates,
//! orthogonalizes them via Gram-Schmidt, and provides utilities for
//! transforming normals into tangent space.

// ── Types ──────────────────────────────────────────────────────────────────────

/// Configuration for tangent-space computation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TangentConfig {
    /// When `true`, average tangents smoothly over all faces sharing a vertex.
    pub smooth: bool,
    /// Normalize tangents to unit length after averaging.
    pub normalize: bool,
    /// Angle (radians) above which tangents are NOT averaged (hard edge).
    pub smooth_angle_threshold: f32,
}

/// A per-vertex tangent frame: tangent T, bitangent B, normal N, and sign.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TangentFrame {
    /// Tangent vector (unit, in model space).
    pub tangent: [f32; 3],
    /// Bitangent vector (unit, in model space).
    pub bitangent: [f32; 3],
    /// Surface normal (unit, in model space).
    pub normal: [f32; 3],
    /// Handedness sign (+1 or −1).
    pub sign: f32,
}

/// Result of a full tangent computation pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TangentResult {
    /// Per-vertex tangent frames, one per vertex in the mesh.
    pub frames: Vec<TangentFrame>,
    /// Number of vertices processed.
    pub vertex_count: usize,
    /// Number of degenerate UV triangles skipped.
    pub degenerate_count: usize,
}

// ── Default config ────────────────────────────────────────────────────────────

/// Build a [`TangentConfig`] with sensible defaults.
#[allow(dead_code)]
pub fn default_tangent_config() -> TangentConfig {
    TangentConfig {
        smooth: true,
        normalize: true,
        smooth_angle_threshold: std::f32::consts::FRAC_PI_4,
    }
}

// ── Low-level math helpers ────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v).max(1e-10);
    [v[0] / l, v[1] / l, v[2] / l]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

// ── Core computations ─────────────────────────────────────────────────────────

/// Compute the tangent frame for a single triangle from its positions, UVs, and face normal.
///
/// Returns `None` for degenerate (zero-area UV) triangles.
#[allow(dead_code)]
pub fn compute_tangent_frame(
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
    normal: [f32; 3],
) -> Option<TangentFrame> {
    let e1 = sub3(p1, p0);
    let e2 = sub3(p2, p0);
    let du1 = uv1[0] - uv0[0];
    let dv1 = uv1[1] - uv0[1];
    let du2 = uv2[0] - uv0[0];
    let dv2 = uv2[1] - uv0[1];

    let det = du1 * dv2 - du2 * dv1;
    if det.abs() < 1e-10 {
        return None;
    }
    let inv = 1.0 / det;

    let tangent = normalize3([
        inv * (dv2 * e1[0] - dv1 * e2[0]),
        inv * (dv2 * e1[1] - dv1 * e2[1]),
        inv * (dv2 * e1[2] - dv1 * e2[2]),
    ]);
    let bitangent_raw = [
        inv * (du1 * e2[0] - du2 * e1[0]),
        inv * (du1 * e2[1] - du2 * e1[1]),
        inv * (du1 * e2[2] - du2 * e1[2]),
    ];
    let sign = tangent_sign(normal, tangent, bitangent_raw);
    let bitangent = bitangent_from_frame(normal, tangent, sign);

    Some(TangentFrame {
        tangent,
        bitangent,
        normal,
        sign,
    })
}

/// Compute per-vertex tangent frames averaged over all incident triangles.
///
/// - `positions`: flat `[x,y,z]` per vertex.
/// - `normals`: flat `[nx,ny,nz]` per vertex.
/// - `uvs`: flat `[u,v]` per vertex.
/// - `indices`: triangle list (must be divisible by 3).
#[allow(dead_code)]
pub fn compute_vertex_tangents(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
    config: &TangentConfig,
) -> TangentResult {
    let n = positions.len();
    let mut sum_tan = vec![[0.0f32; 3]; n];
    let mut sum_bitan = vec![[0.0f32; 3]; n];
    let mut degenerate_count = 0usize;

    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= n || i1 >= n || i2 >= n {
            continue;
        }
        let face_normal = normalize3(add3(add3(normals[i0], normals[i1]), normals[i2]));
        match compute_tangent_frame(
            positions[i0],
            positions[i1],
            positions[i2],
            uvs[i0],
            uvs[i1],
            uvs[i2],
            face_normal,
        ) {
            Some(frame) => {
                for &vi in &[i0, i1, i2] {
                    sum_tan[vi] = add3(sum_tan[vi], frame.tangent);
                    sum_bitan[vi] = add3(sum_bitan[vi], frame.bitangent);
                }
            }
            None => {
                degenerate_count += 1;
            }
        }
    }

    let mut frames = Vec::with_capacity(n);
    for vi in 0..n {
        let n_v = if vi < normals.len() {
            normalize3(normals[vi])
        } else {
            [0.0, 0.0, 1.0]
        };
        let raw_t = sum_tan[vi];
        let raw_b = sum_bitan[vi];

        let (t, b, sign) = if len3(raw_t) < 1e-8 {
            ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0f32)
        } else {
            let t_orth = if config.smooth {
                orthogonalize_tangent(n_v, raw_t)
            } else {
                normalize3(raw_t)
            };
            let sign = tangent_sign(n_v, t_orth, raw_b);
            let b = bitangent_from_frame(n_v, t_orth, sign);
            (t_orth, b, sign)
        };

        frames.push(TangentFrame {
            tangent: if config.normalize { normalize3(t) } else { t },
            bitangent: if config.normalize { normalize3(b) } else { b },
            normal: n_v,
            sign,
        });
    }

    TangentResult {
        vertex_count: n,
        degenerate_count,
        frames,
    }
}

/// Orthogonalize a tangent against a normal using Gram-Schmidt.
///
/// `normal` and `raw_tangent` need not be unit-length.
#[allow(dead_code)]
pub fn orthogonalize_tangent(normal: [f32; 3], raw_tangent: [f32; 3]) -> [f32; 3] {
    let n = normalize3(normal);
    let t = normalize3(raw_tangent);
    let proj = dot3(t, n);
    normalize3(sub3(t, scale3(n, proj)))
}

/// Compute the handedness sign (+1 / −1) of the tangent basis.
///
/// The sign is +1 when (T × B) · N > 0 (right-handed), −1 otherwise.
#[allow(dead_code)]
pub fn tangent_sign(normal: [f32; 3], tangent: [f32; 3], bitangent: [f32; 3]) -> f32 {
    if dot3(cross3(tangent, bitangent), normal) >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

/// Reconstruct the bitangent from a normal, tangent, and handedness sign.
///
/// `bitangent = sign * (normal × tangent)`
#[allow(dead_code)]
pub fn bitangent_from_frame(normal: [f32; 3], tangent: [f32; 3], sign: f32) -> [f32; 3] {
    scale3(cross3(normal, tangent), sign)
}

/// Validate that a [`TangentFrame`] is well-formed (all vectors unit, orthogonal, finite).
///
/// Returns `true` if the frame passes basic sanity checks.
#[allow(dead_code)]
pub fn validate_tangent_frame(frame: &TangentFrame) -> bool {
    let tol = 1e-3f32;
    let t = frame.tangent;
    let b = frame.bitangent;
    let n = frame.normal;
    if !t.iter().all(|v| v.is_finite()) {
        return false;
    }
    if !b.iter().all(|v| v.is_finite()) {
        return false;
    }
    if !n.iter().all(|v| v.is_finite()) {
        return false;
    }
    (len3(t) - 1.0).abs() < tol
        && (len3(b) - 1.0).abs() < tol
        && (len3(n) - 1.0).abs() < tol
        && dot3(t, n).abs() < tol
        && dot3(t, b).abs() < tol
}

/// Smooth tangent frames by averaging over a neighbourhood list.
///
/// `adjacency[i]` is the list of vertex indices adjacent to vertex `i`.
#[allow(dead_code)]
pub fn smooth_tangents(frames: &mut [TangentFrame], adjacency: &[Vec<usize>]) {
    let n = frames.len().min(adjacency.len());
    let orig = frames.to_vec();
    for i in 0..n {
        let mut sum_t = orig[i].tangent;
        let mut sum_b = orig[i].bitangent;
        let mut sum_n = orig[i].normal;
        for &j in &adjacency[i] {
            if j < orig.len() {
                sum_t = add3(sum_t, orig[j].tangent);
                sum_b = add3(sum_b, orig[j].bitangent);
                sum_n = add3(sum_n, orig[j].normal);
            }
        }
        let n_v = normalize3(sum_n);
        let t_v = orthogonalize_tangent(n_v, sum_t);
        let sign = tangent_sign(n_v, t_v, sum_b);
        let b_v = bitangent_from_frame(n_v, t_v, sign);
        frames[i] = TangentFrame {
            tangent: t_v,
            bitangent: b_v,
            normal: n_v,
            sign,
        };
    }
}

/// Transform a tangent-space normal (from a normal map) into model space.
///
/// `ts_normal` is the decoded normal map vector (should be unit-length).
#[allow(dead_code)]
pub fn tangent_space_normal(frame: &TangentFrame, ts_normal: [f32; 3]) -> [f32; 3] {
    let t = frame.tangent;
    let b = frame.bitangent;
    let n = frame.normal;
    normalize3([
        ts_normal[0] * t[0] + ts_normal[1] * b[0] + ts_normal[2] * n[0],
        ts_normal[0] * t[1] + ts_normal[1] * b[1] + ts_normal[2] * n[1],
        ts_normal[0] * t[2] + ts_normal[1] * b[2] + ts_normal[2] * n[2],
    ])
}

/// Build a 3×3 TBN matrix (column-major as flat array) from a [`TangentFrame`].
///
/// Layout: `[T0, T1, T2, B0, B1, B2, N0, N1, N2]`.
#[allow(dead_code)]
pub fn tangent_frame_matrix(frame: &TangentFrame) -> [f32; 9] {
    let t = frame.tangent;
    let b = frame.bitangent;
    let n = frame.normal;
    [t[0], t[1], t[2], b[0], b[1], b[2], n[0], n[1], n[2]]
}

/// Return the number of vertices in a [`TangentResult`].
#[allow(dead_code)]
pub fn tangent_vertex_count(result: &TangentResult) -> usize {
    result.vertex_count
}

/// Invert the handedness of all tangent frames in place.
#[allow(dead_code)]
pub fn invert_tangent_handedness(frames: &mut [TangentFrame]) {
    for f in frames.iter_mut() {
        f.sign = -f.sign;
        f.bitangent = scale3(f.bitangent, -1.0);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::type_complexity)]
    fn flat_quad_data() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
        // Unit square in XY plane, normal = +Z
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let nrm = vec![[0.0, 0.0, 1.0]; 4];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        (pos, nrm, uvs, idx)
    }

    #[test]
    fn default_tangent_config_smooth_true() {
        let c = default_tangent_config();
        assert!(c.smooth);
    }

    #[test]
    fn compute_tangent_frame_returns_some_for_valid_triangle() {
        let frame = compute_tangent_frame(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.0, 0.0, 1.0],
        );
        assert!(frame.is_some(), "valid triangle should produce a frame");
    }

    #[test]
    fn compute_tangent_frame_none_for_degenerate_uv() {
        let frame = compute_tangent_frame(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0, 1.0],
        );
        assert!(frame.is_none(), "zero UV triangle should return None");
    }

    #[test]
    fn compute_vertex_tangents_count_matches() {
        let (pos, nrm, uvs, idx) = flat_quad_data();
        let cfg = default_tangent_config();
        let result = compute_vertex_tangents(&pos, &nrm, &uvs, &idx, &cfg);
        assert_eq!(result.vertex_count, 4);
        assert_eq!(result.frames.len(), 4);
    }

    #[test]
    fn compute_vertex_tangents_no_degenerate_for_valid_quad() {
        let (pos, nrm, uvs, idx) = flat_quad_data();
        let result = compute_vertex_tangents(&pos, &nrm, &uvs, &idx, &default_tangent_config());
        assert_eq!(result.degenerate_count, 0);
    }

    #[test]
    fn orthogonalize_tangent_perpendicular_to_normal() {
        let n = [0.0, 0.0, 1.0];
        let t_raw = [1.0, 0.5, 0.8]; // not perp to n
        let t_orth = orthogonalize_tangent(n, t_raw);
        let dp = dot3(t_orth, n);
        assert!(
            dp.abs() < 1e-5,
            "orthogonalized tangent must be perp to normal, dot={dp}"
        );
    }

    #[test]
    fn tangent_sign_right_handed() {
        let n = [0.0, 0.0, 1.0];
        let t = [1.0, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        assert_eq!(tangent_sign(n, t, b), 1.0);
    }

    #[test]
    fn tangent_sign_left_handed() {
        let n = [0.0, 0.0, 1.0];
        let t = [1.0, 0.0, 0.0];
        let b = [0.0, -1.0, 0.0];
        assert_eq!(tangent_sign(n, t, b), -1.0);
    }

    #[test]
    fn bitangent_from_frame_unit_length() {
        let n = [0.0, 0.0, 1.0];
        let t = [1.0, 0.0, 0.0];
        let b = bitangent_from_frame(n, t, 1.0);
        let l = len3(b);
        assert!((l - 1.0).abs() < 1e-5, "bitangent length={l}");
    }

    #[test]
    fn validate_tangent_frame_passes_for_valid() {
        let frame = TangentFrame {
            tangent: [1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            sign: 1.0,
        };
        assert!(validate_tangent_frame(&frame));
    }

    #[test]
    fn validate_tangent_frame_fails_for_non_unit() {
        let frame = TangentFrame {
            tangent: [2.0, 0.0, 0.0], // non-unit
            bitangent: [0.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            sign: 1.0,
        };
        assert!(!validate_tangent_frame(&frame));
    }

    #[test]
    fn tangent_space_normal_round_trip_identity() {
        let frame = TangentFrame {
            tangent: [1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            sign: 1.0,
        };
        // (0,0,1) in tangent space → should give normal direction
        let result = tangent_space_normal(&frame, [0.0, 0.0, 1.0]);
        assert!((result[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn tangent_frame_matrix_correct_layout() {
        let frame = TangentFrame {
            tangent: [1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            sign: 1.0,
        };
        let m = tangent_frame_matrix(&frame);
        assert_eq!(m[0], 1.0); // T.x
        assert_eq!(m[4], 1.0); // B.y
        assert_eq!(m[8], 1.0); // N.z
    }

    #[test]
    fn tangent_vertex_count_matches_result() {
        let (pos, nrm, uvs, idx) = flat_quad_data();
        let result = compute_vertex_tangents(&pos, &nrm, &uvs, &idx, &default_tangent_config());
        assert_eq!(tangent_vertex_count(&result), result.vertex_count);
    }

    #[test]
    fn invert_tangent_handedness_flips_sign() {
        let (pos, nrm, uvs, idx) = flat_quad_data();
        let result = compute_vertex_tangents(&pos, &nrm, &uvs, &idx, &default_tangent_config());
        let mut frames = result.frames;
        let original_sign = frames[0].sign;
        invert_tangent_handedness(&mut frames);
        assert_eq!(frames[0].sign, -original_sign);
    }

    #[test]
    fn smooth_tangents_preserves_count() {
        let (pos, nrm, uvs, idx) = flat_quad_data();
        let result = compute_vertex_tangents(&pos, &nrm, &uvs, &idx, &default_tangent_config());
        let mut frames = result.frames;
        let adjacency: Vec<Vec<usize>> = vec![vec![1, 3], vec![0, 2], vec![1, 3], vec![2, 0]];
        smooth_tangents(&mut frames, &adjacency);
        assert_eq!(frames.len(), 4);
    }
}
