// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Smooth normal computation with crease angle threshold.
//!
//! Normals are averaged across faces sharing a vertex, but not across
//! edges sharper than the crease angle threshold.  When the angle between
//! two adjacent face normals exceeds the crease angle, the edge is treated
//! as a hard edge and the normals are not blended across it.

// ─── Structures ──────────────────────────────────────────────────────────────

/// Configuration for smooth normal computation.
#[allow(dead_code)]
pub struct SmoothNormalsConfig {
    /// Maximum angle (in degrees) below which face normals are blended.
    /// Edges sharper than this are treated as hard creases.
    pub crease_angle_deg: f32,
    /// Whether to weight contributions by face area.
    pub area_weighted: bool,
}

/// Result of smooth normal computation.
#[allow(dead_code)]
pub struct SmoothNormalsResult {
    /// One unit normal per vertex, in the same order as the input vertex slice.
    pub normals: Vec<[f32; 3]>,
    /// Number of hard edges detected during computation.
    pub hard_edge_count: usize,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Normalize a 3-vector; returns `[0,0,1]` if the length is near zero.
#[allow(dead_code)]
pub fn normalize_vec3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Compute the face normal of a triangle (not necessarily unit length).
#[allow(dead_code)]
pub fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    // Cross product ab × ac
    [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ]
}

/// Angle in degrees between two (possibly non-unit) normals.
#[allow(dead_code)]
pub fn angle_between_normals_deg(a: [f32; 3], b: [f32; 3]) -> f32 {
    let na = normalize_vec3(a);
    let nb = normalize_vec3(b);
    let dot = (na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2]).clamp(-1.0, 1.0);
    dot.acos().to_degrees()
}

fn vec3_len_sq(v: [f32; 3]) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Default configuration: 30° crease angle, area-weighted contributions.
#[allow(dead_code)]
pub fn default_smooth_normals_config() -> SmoothNormalsConfig {
    SmoothNormalsConfig {
        crease_angle_deg: 30.0,
        area_weighted: true,
    }
}

/// Compute smooth normals for a triangle mesh.
///
/// For each vertex the function averages the face normals of all adjacent
/// faces whose normal does not differ by more than `cfg.crease_angle_deg`
/// from any other contributing face normal.  Faces beyond the crease threshold
/// are excluded from the blend.
#[allow(dead_code)]
pub fn compute_smooth_normals(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    cfg: &SmoothNormalsConfig,
) -> SmoothNormalsResult {
    let n = verts.len();
    if n == 0 || faces.is_empty() {
        return SmoothNormalsResult {
            normals: vec![[0.0, 0.0, 1.0]; n],
            hard_edge_count: 0,
        };
    }

    // Compute a (possibly area-weighted) face normal for each face.
    let face_normals: Vec<[f32; 3]> = faces
        .iter()
        .map(|f| {
            let a = verts[f[0] as usize];
            let b = verts[f[1] as usize];
            let c = verts[f[2] as usize];
            let raw = face_normal(a, b, c);
            if cfg.area_weighted {
                // Magnitude == 2 * area, so this naturally weights by area.
                raw
            } else {
                normalize_vec3(raw)
            }
        })
        .collect();

    // Build per-vertex face adjacency list.
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (fi, f) in faces.iter().enumerate() {
        for &vi in f.iter() {
            adj[vi as usize].push(fi);
        }
    }

    let _crease_sq = cfg.crease_angle_deg * cfg.crease_angle_deg;
    let mut hard_edge_count = 0usize;
    let mut normals = vec![[0.0f32; 3]; n];

    for (vi, adj_faces) in adj.iter().enumerate() {
        if adj_faces.is_empty() {
            normals[vi] = [0.0, 0.0, 1.0];
            continue;
        }

        // Use the first face normal as the reference.
        let ref_n = normalize_vec3(face_normals[adj_faces[0]]);
        let mut accum = face_normals[adj_faces[0]];
        let mut included = 1usize;

        for &fi in adj_faces.iter().skip(1) {
            let fn_i = normalize_vec3(face_normals[fi]);
            let angle = angle_between_normals_deg(ref_n, fn_i);
            if angle <= cfg.crease_angle_deg {
                accum = add3(accum, face_normals[fi]);
                included += 1;
            } else {
                hard_edge_count += 1;
            }
        }

        // If nothing was included beyond the reference, accum is already set.
        let _ = included;
        normals[vi] = if vec3_len_sq(accum) < 1e-24 {
            [0.0, 0.0, 1.0]
        } else {
            normalize_vec3(accum)
        };
    }

    SmoothNormalsResult {
        normals,
        hard_edge_count,
    }
}

/// Number of vertices in the result.
#[allow(dead_code)]
pub fn smooth_normals_vertex_count(result: &SmoothNormalsResult) -> usize {
    result.normals.len()
}

/// Check that all normals in the result are (approximately) unit length.
#[allow(dead_code)]
pub fn smooth_normals_are_unit(result: &SmoothNormalsResult) -> bool {
    result.normals.iter().all(|&n| {
        let len_sq = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
        (len_sq - 1.0).abs() < 1e-4
    })
}

/// Compute flat (per-vertex, face-assigned) normals: each vertex gets the
/// normal of the last face that references it.  Useful as a simple baseline.
#[allow(dead_code)]
pub fn flat_normals(verts: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let n = verts.len();
    let mut normals = vec![[0.0f32, 0.0, 1.0]; n];
    for f in faces {
        let a = verts[f[0] as usize];
        let b = verts[f[1] as usize];
        let c = verts[f[2] as usize];
        let fn_ = normalize_vec3(face_normal(a, b, c));
        normals[f[0] as usize] = fn_;
        normals[f[1] as usize] = fn_;
        normals[f[2] as usize] = fn_;
    }
    normals
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tri_verts() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }
    fn single_tri_faces() -> Vec<[u32; 3]> {
        vec![[0, 1, 2]]
    }

    #[test]
    fn test_normalize_vec3_unit() {
        let v = normalize_vec3([3.0, 0.0, 0.0]);
        assert!((v[0] - 1.0).abs() < 1e-6);
        assert!(v[1].abs() < 1e-6);
        assert!(v[2].abs() < 1e-6);
    }

    #[test]
    fn test_normalize_vec3_zero_returns_default() {
        let v = normalize_vec3([0.0, 0.0, 0.0]);
        assert_eq!(v, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_face_normal_z_up() {
        let n = face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let nu = normalize_vec3(n);
        assert!((nu[2] - 1.0).abs() < 1e-5, "nu={:?}", nu);
    }

    #[test]
    fn test_angle_between_same_normals_is_zero() {
        let angle = angle_between_normals_deg([0.0, 1.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(angle.abs() < 1e-4, "angle={}", angle);
    }

    #[test]
    fn test_angle_between_perpendicular_is_90() {
        let angle = angle_between_normals_deg([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((angle - 90.0).abs() < 1e-3, "angle={}", angle);
    }

    #[test]
    fn test_default_config_crease_angle() {
        let cfg = default_smooth_normals_config();
        assert!((cfg.crease_angle_deg - 30.0).abs() < 1e-6);
        assert!(cfg.area_weighted);
    }

    #[test]
    fn test_smooth_normals_single_tri_count() {
        let verts = single_tri_verts();
        let faces = single_tri_faces();
        let cfg = default_smooth_normals_config();
        let result = compute_smooth_normals(&verts, &faces, &cfg);
        assert_eq!(smooth_normals_vertex_count(&result), 3);
    }

    #[test]
    fn test_smooth_normals_are_unit_single_tri() {
        let verts = single_tri_verts();
        let faces = single_tri_faces();
        let cfg = default_smooth_normals_config();
        let result = compute_smooth_normals(&verts, &faces, &cfg);
        assert!(smooth_normals_are_unit(&result));
    }

    #[test]
    fn test_smooth_normals_direction_z_up() {
        let verts = single_tri_verts();
        let faces = single_tri_faces();
        let cfg = default_smooth_normals_config();
        let result = compute_smooth_normals(&verts, &faces, &cfg);
        for n in &result.normals {
            assert!(n[2] > 0.9, "n={:?}", n);
        }
    }

    #[test]
    fn test_smooth_normals_empty_mesh() {
        let cfg = default_smooth_normals_config();
        let result = compute_smooth_normals(&[], &[], &cfg);
        assert_eq!(result.normals.len(), 0);
    }

    #[test]
    fn test_flat_normals_single_tri() {
        let verts = single_tri_verts();
        let faces = single_tri_faces();
        let normals = flat_normals(&verts, &faces);
        assert_eq!(normals.len(), 3);
        for n in &normals {
            assert!(n[2] > 0.9, "n={:?}", n);
        }
    }

    #[test]
    fn test_smooth_normals_two_coplanar_tris_no_crease() {
        // Two triangles forming a flat quad — normals should be identical.
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let faces = vec![[0u32, 1, 2], [0, 2, 3]];
        let cfg = default_smooth_normals_config();
        let result = compute_smooth_normals(&verts, &faces, &cfg);
        assert!(smooth_normals_are_unit(&result));
        assert_eq!(result.hard_edge_count, 0);
    }
}
