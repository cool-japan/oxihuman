// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Offset surface generation — displaces each vertex along its normal by a
//! given distance.
//!
//! Supports both inward and outward offsets and can generate a shell mesh
//! (two offset surfaces connected along open boundaries).

// ── helpers ──────────────────────────────────────────────────────────────────

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

// ── config / result ───────────────────────────────────────────────────────────

/// Configuration for offset surface generation.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct OffsetSurfaceConfig {
    /// Distance to displace each vertex along its normal.
    pub offset_distance: f32,
    /// If `true`, displace inward (opposite normal direction).
    pub inward: bool,
    /// If `true`, clamp normals to unit length before offsetting.
    pub normalize_normals: bool,
}

/// Result returned by [`offset_surface`].
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct OffsetSurfaceResult {
    /// Offset vertex positions.
    pub vertices: Vec<[f32; 3]>,
    /// Maximum displacement magnitude applied to any vertex.
    pub max_displacement: f32,
}

// ── public API ────────────────────────────────────────────────────────────────

/// Return an [`OffsetSurfaceConfig`] with sensible defaults.
#[allow(dead_code)]
pub fn default_offset_surface_config() -> OffsetSurfaceConfig {
    OffsetSurfaceConfig {
        offset_distance: 0.01,
        inward: false,
        normalize_normals: true,
    }
}

/// Displace each vertex along its normal by `cfg.offset_distance`.
///
/// `normals` must be the same length as `verts`.
#[allow(dead_code)]
pub fn offset_surface(
    verts: &[[f32; 3]],
    normals: &[[f32; 3]],
    cfg: &OffsetSurfaceConfig,
) -> OffsetSurfaceResult {
    let sign = if cfg.inward { -1.0f32 } else { 1.0f32 };
    let mut out_verts: Vec<[f32; 3]> = Vec::with_capacity(verts.len());
    let mut max_disp = 0.0f32;

    for (v, n) in verts.iter().zip(normals.iter()) {
        let nhat = if cfg.normalize_normals {
            normalize3(*n)
        } else {
            *n
        };
        let disp = cfg.offset_distance * sign;
        let new_v = [
            v[0] + nhat[0] * disp,
            v[1] + nhat[1] * disp,
            v[2] + nhat[2] * disp,
        ];
        let mag = disp.abs();
        if mag > max_disp {
            max_disp = mag;
        }
        out_verts.push(new_v);
    }

    OffsetSurfaceResult {
        vertices: out_verts,
        max_displacement: max_disp,
    }
}

/// Return the number of vertices in an offset surface result.
#[allow(dead_code)]
pub fn offset_surface_vertex_count(result: &OffsetSurfaceResult) -> usize {
    result.vertices.len()
}

/// Return the maximum displacement recorded in an offset surface result.
#[allow(dead_code)]
pub fn offset_surface_max_displacement(result: &OffsetSurfaceResult) -> f32 {
    result.max_displacement
}

/// Compute per-vertex normals for a triangle mesh by averaging face normals.
#[allow(dead_code)]
pub fn compute_vertex_normals_offset(verts: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let n = verts.len();
    let mut accum = vec![[0.0f32; 3]; n];

    for f in faces {
        let a = verts[f[0] as usize];
        let b = verts[f[1] as usize];
        let c = verts[f[2] as usize];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = cross3(ab, ac);
        for &i in f.iter() {
            accum[i as usize] = add3(accum[i as usize], fn_);
        }
    }

    accum.iter().map(|n| normalize3(*n)).collect()
}

/// Offset each vertex inward (opposite to its normal).
#[allow(dead_code)]
pub fn offset_inward(verts: &[[f32; 3]], normals: &[[f32; 3]], amount: f32) -> Vec<[f32; 3]> {
    verts
        .iter()
        .zip(normals.iter())
        .map(|(v, n)| {
            let nhat = normalize3(*n);
            [
                v[0] - nhat[0] * amount,
                v[1] - nhat[1] * amount,
                v[2] - nhat[2] * amount,
            ]
        })
        .collect()
}

/// Offset each vertex outward (along its normal).
#[allow(dead_code)]
pub fn offset_outward(verts: &[[f32; 3]], normals: &[[f32; 3]], amount: f32) -> Vec<[f32; 3]> {
    verts
        .iter()
        .zip(normals.iter())
        .map(|(v, n)| {
            let nhat = normalize3(*n);
            [
                v[0] + nhat[0] * amount,
                v[1] + nhat[1] * amount,
                v[2] + nhat[2] * amount,
            ]
        })
        .collect()
}

/// Generate a shell mesh from the input by producing an inner and outer surface
/// separated by `thickness`, then stitching them together.
///
/// The inner surface faces are reversed so the normals point outward on both
/// sides of the shell. Stitch faces are generated for each boundary edge of the
/// original face list.
#[allow(dead_code)]
pub fn shell_mesh(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    thickness: f32,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let normals = compute_vertex_normals_offset(verts, faces);
    let outer = offset_outward(verts, &normals, thickness * 0.5);
    let inner = offset_inward(verts, &normals, thickness * 0.5);

    let nv = verts.len() as u32;
    let mut shell_verts: Vec<[f32; 3]> = outer;
    shell_verts.extend(inner);

    let mut shell_faces: Vec<[u32; 3]> = Vec::new();

    // Outer faces (original winding).
    shell_faces.extend(faces.iter().copied());

    // Inner faces (reversed winding so normals face inward).
    for f in faces {
        shell_faces.push([f[0] + nv, f[2] + nv, f[1] + nv]);
    }

    // Stitch edges: for each boundary edge of the original, connect outer and inner.
    // Use a simple approach: for each face edge, add two triangles to form a quad.
    for f in faces {
        let edges = [(f[0], f[1]), (f[1], f[2]), (f[2], f[0])];
        for (a, b) in edges {
            // outer a, outer b, inner b, inner a form a quad
            let oa = a;
            let ob = b;
            let ib = b + nv;
            let ia = a + nv;
            shell_faces.push([oa, ob, ib]);
            shell_faces.push([oa, ib, ia]);
        }
    }

    (shell_verts, shell_faces)
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_verts() -> Vec<[f32; 3]> {
        vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }

    fn flat_normals() -> Vec<[f32; 3]> {
        vec![[0.0f32, 0.0, 1.0]; 3]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_offset_surface_config();
        assert!(cfg.offset_distance > 0.0);
        assert!(!cfg.inward);
        assert!(cfg.normalize_normals);
    }

    #[test]
    fn test_offset_surface_vertex_count() {
        let v = triangle_verts();
        let n = flat_normals();
        let cfg = default_offset_surface_config();
        let result = offset_surface(&v, &n, &cfg);
        assert_eq!(offset_surface_vertex_count(&result), 3);
    }

    #[test]
    fn test_offset_surface_max_displacement() {
        let v = triangle_verts();
        let n = flat_normals();
        let mut cfg = default_offset_surface_config();
        cfg.offset_distance = 0.5;
        let result = offset_surface(&v, &n, &cfg);
        assert!((offset_surface_max_displacement(&result) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_offset_outward_moves_along_normal() {
        let v = triangle_verts();
        let n = flat_normals();
        let out = offset_outward(&v, &n, 1.0);
        // All z-components should be 1.0 (moved outward along +z normal).
        for p in &out {
            assert!((p[2] - 1.0).abs() < 1e-5, "z={}", p[2]);
        }
    }

    #[test]
    fn test_offset_inward_moves_opposite_normal() {
        let v = triangle_verts();
        let n = flat_normals();
        let inn = offset_inward(&v, &n, 1.0);
        for p in &inn {
            assert!((p[2] + 1.0).abs() < 1e-5, "z={}", p[2]);
        }
    }

    #[test]
    fn test_compute_vertex_normals_offset_unit_length() {
        let v = triangle_verts();
        let f = vec![[0u32, 1, 2]];
        let normals = compute_vertex_normals_offset(&v, &f);
        assert_eq!(normals.len(), 3);
        for n in &normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-5, "len={len}");
        }
    }

    #[test]
    fn test_shell_mesh_vertex_count() {
        let v = triangle_verts();
        let f = vec![[0u32, 1, 2]];
        let (sv, _sf) = shell_mesh(&v, &f, 0.1);
        // Should have 2 * original vertex count.
        assert_eq!(sv.len(), 2 * v.len());
    }

    #[test]
    fn test_shell_mesh_valid_indices() {
        let v = triangle_verts();
        let f = vec![[0u32, 1, 2]];
        let (sv, sf) = shell_mesh(&v, &f, 0.1);
        let nv = sv.len() as u32;
        for face in &sf {
            assert!(face[0] < nv, "idx {} out of range {}", face[0], nv);
            assert!(face[1] < nv, "idx {} out of range {}", face[1], nv);
            assert!(face[2] < nv, "idx {} out of range {}", face[2], nv);
        }
    }

    #[test]
    fn test_offset_surface_inward() {
        let v = triangle_verts();
        let n = flat_normals();
        let mut cfg = default_offset_surface_config();
        cfg.inward = true;
        cfg.offset_distance = 0.5;
        let result = offset_surface(&v, &n, &cfg);
        // z should be -0.5 for all vertices.
        for p in &result.vertices {
            assert!((p[2] + 0.5).abs() < 1e-5, "z={}", p[2]);
        }
    }
}
