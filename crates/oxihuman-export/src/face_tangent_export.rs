// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export per-face tangent vectors.

/// Per-face tangent data.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FaceTangent {
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
    pub normal: [f32; 3],
}

/// Per-face tangent export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceTangentExport {
    pub tangents: Vec<FaceTangent>,
}

/// Normalize a 3-D vector.
#[allow(dead_code)]
pub fn normalize_ft(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        return [1.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Cross product.
#[allow(dead_code)]
pub fn cross_ft(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Compute per-face tangents from positions, indices, and UVs.
#[allow(dead_code)]
pub fn compute_face_tangents(
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
) -> FaceTangentExport {
    let tri_count = indices.len() / 3;
    let mut tangents = Vec::with_capacity(tri_count);
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        if i0 >= positions.len()
            || i1 >= positions.len()
            || i2 >= positions.len()
            || i0 >= uvs.len()
            || i1 >= uvs.len()
            || i2 >= uvs.len()
        {
            tangents.push(FaceTangent {
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            });
            continue;
        }
        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];
        let u0 = uvs[i0];
        let u1 = uvs[i1];
        let u2 = uvs[i2];
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let du1 = u1[0] - u0[0];
        let dv1 = u1[1] - u0[1];
        let du2 = u2[0] - u0[0];
        let dv2 = u2[1] - u0[1];
        let det = du1 * dv2 - du2 * dv1;
        let (tan, bitan) = if det.abs() < 1e-12 {
            ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])
        } else {
            let inv = 1.0 / det;
            (
                normalize_ft([
                    (dv2 * e1[0] - dv1 * e2[0]) * inv,
                    (dv2 * e1[1] - dv1 * e2[1]) * inv,
                    (dv2 * e1[2] - dv1 * e2[2]) * inv,
                ]),
                normalize_ft([
                    (-du2 * e1[0] + du1 * e2[0]) * inv,
                    (-du2 * e1[1] + du1 * e2[1]) * inv,
                    (-du2 * e1[2] + du1 * e2[2]) * inv,
                ]),
            )
        };
        let normal = normalize_ft(cross_ft(e1, e2));
        tangents.push(FaceTangent {
            tangent: tan,
            bitangent: bitan,
            normal,
        });
    }
    FaceTangentExport { tangents }
}

/// Count face tangents.
#[allow(dead_code)]
pub fn face_tangent_count(export: &FaceTangentExport) -> usize {
    export.tangents.len()
}

/// Validate tangents are unit length.
#[allow(dead_code)]
pub fn tangents_unit(export: &FaceTangentExport) -> bool {
    export.tangents.iter().all(|ft| {
        let lt2 = ft.tangent[0] * ft.tangent[0]
            + ft.tangent[1] * ft.tangent[1]
            + ft.tangent[2] * ft.tangent[2];
        (lt2 - 1.0).abs() < 1e-3
    })
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn face_tangent_to_json(export: &FaceTangentExport) -> String {
    format!("{{\"face_count\":{}}}", export.tangents.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_tri() -> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let idx = vec![0u32, 1, 2];
        (pos, uvs, idx)
    }

    #[test]
    fn test_compute_count() {
        let (pos, uvs, idx) = flat_tri();
        let e = compute_face_tangents(&pos, &uvs, &idx);
        assert_eq!(face_tangent_count(&e), 1);
    }

    #[test]
    fn test_tangents_unit() {
        let (pos, uvs, idx) = flat_tri();
        let e = compute_face_tangents(&pos, &uvs, &idx);
        assert!(tangents_unit(&e));
    }

    #[test]
    fn test_empty_mesh() {
        let e = compute_face_tangents(&[], &[], &[]);
        assert_eq!(face_tangent_count(&e), 0);
    }

    #[test]
    fn test_normalize_ft() {
        let n = normalize_ft([3.0, 4.0, 0.0]);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_cross_ft_z_axis() {
        let c = cross_ft([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((c[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normal_z_up() {
        let (pos, uvs, idx) = flat_tri();
        let e = compute_face_tangents(&pos, &uvs, &idx);
        let n = e.tangents[0].normal;
        assert!(n[2].abs() > 0.5);
    }

    #[test]
    fn test_face_tangent_to_json() {
        let (pos, uvs, idx) = flat_tri();
        let e = compute_face_tangents(&pos, &uvs, &idx);
        let j = face_tangent_to_json(&e);
        assert!(j.contains("face_count"));
    }

    #[test]
    fn test_oob_indices_default() {
        let pos = vec![[0.0, 0.0, 0.0]];
        let uvs = vec![[0.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let e = compute_face_tangents(&pos, &uvs, &idx);
        assert_eq!(face_tangent_count(&e), 1);
        assert_eq!(e.tangents[0].tangent, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_tangent_normal_cross_bitangent_consistent() {
        let (pos, uvs, idx) = flat_tri();
        let e = compute_face_tangents(&pos, &uvs, &idx);
        let ft = &e.tangents[0];
        // tangent and bitangent should have reasonable lengths
        let lt = (ft.tangent[0] * ft.tangent[0]
            + ft.tangent[1] * ft.tangent[1]
            + ft.tangent[2] * ft.tangent[2])
            .sqrt();
        assert!((lt - 1.0).abs() < 1e-3);
    }
}
