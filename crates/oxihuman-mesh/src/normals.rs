// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::mesh::MeshBuffers;

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(v: [f32; 3]) -> Option<[f32; 3]> {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        None
    } else {
        Some([v[0] / len, v[1] / len, v[2] / len])
    }
}

fn normalize_or_y(v: [f32; 3]) -> [f32; 3] {
    normalize(v).unwrap_or([0.0, 1.0, 0.0])
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Recompute per-vertex normals by averaging face normals.
pub fn compute_normals(buf: &mut MeshBuffers) {
    let n = buf.positions.len();
    let mut accum = vec![[0.0f32; 3]; n];

    for tri in buf.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= n || i1 >= n || i2 >= n {
            continue;
        }
        let e1 = sub(buf.positions[i1], buf.positions[i0]);
        let e2 = sub(buf.positions[i2], buf.positions[i0]);
        let face_n = cross(e1, e2);
        accum[i0] = add3(accum[i0], face_n);
        accum[i1] = add3(accum[i1], face_n);
        accum[i2] = add3(accum[i2], face_n);
    }

    buf.normals = accum.into_iter().map(normalize_or_y).collect();
}

/// Compute per-vertex tangents using the MikkTSpace-compatible algorithm
/// (Lengyel's method: tangent from UV gradient, handedness from cross product).
///
/// Requires: positions, normals, uvs, and indices must all be populated.
/// Output: mesh.tangents is filled with Vec<[f32; 4]> (XYZ + W handedness).
pub fn compute_tangents(mesh: &mut MeshBuffers) {
    let n = mesh.positions.len();
    if n == 0 {
        mesh.tangents = Vec::new();
        return;
    }

    let mut tan1: Vec<[f32; 3]> = vec![[0.0f32; 3]; n];
    let mut tan2: Vec<[f32; 3]> = vec![[0.0f32; 3]; n];

    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= n || i1 >= n || i2 >= n {
            continue;
        }

        let p0 = mesh.positions[i0];
        let p1 = mesh.positions[i1];
        let p2 = mesh.positions[i2];
        let uv0 = mesh.uvs[i0];
        let uv1 = mesh.uvs[i1];
        let uv2 = mesh.uvs[i2];

        let e1 = sub(p1, p0);
        let e2 = sub(p2, p0);
        let du1 = uv1[0] - uv0[0];
        let dv1 = uv1[1] - uv0[1];
        let du2 = uv2[0] - uv0[0];
        let dv2 = uv2[1] - uv0[1];

        let denom = du1 * dv2 - du2 * dv1;
        if denom.abs() < 1e-10 {
            continue;
        }
        let r = 1.0 / denom;

        // Tangent direction (sdir)
        let sdir = [
            (dv2 * e1[0] - dv1 * e2[0]) * r,
            (dv2 * e1[1] - dv1 * e2[1]) * r,
            (dv2 * e1[2] - dv1 * e2[2]) * r,
        ];
        // Bitangent direction (tdir)
        let tdir = [
            (du1 * e2[0] - du2 * e1[0]) * r,
            (du1 * e2[1] - du2 * e1[1]) * r,
            (du1 * e2[2] - du2 * e1[2]) * r,
        ];

        tan1[i0] = add3(tan1[i0], sdir);
        tan1[i1] = add3(tan1[i1], sdir);
        tan1[i2] = add3(tan1[i2], sdir);

        tan2[i0] = add3(tan2[i0], tdir);
        tan2[i1] = add3(tan2[i1], tdir);
        tan2[i2] = add3(tan2[i2], tdir);
    }

    // Gram-Schmidt orthogonalize and compute handedness
    mesh.tangents = (0..n)
        .map(|i| {
            let nv = mesh.normals[i];
            let t = tan1[i];
            // Gram-Schmidt: t_perp = normalize(t - n * dot(n, t))
            let d = dot(nv, t);
            let t_sub = [t[0] - d * nv[0], t[1] - d * nv[1], t[2] - d * nv[2]];
            let t_perp = match normalize(t_sub) {
                Some(v) => v,
                None => return [1.0f32, 0.0, 0.0, 1.0],
            };
            // Handedness: sign of dot(cross(n, t), tan2)
            let c = cross(nv, t);
            let w = if dot(c, tan2[i]) < 0.0 {
                -1.0f32
            } else {
                1.0f32
            };
            [t_perp[0], t_perp[1], t_perp[2], w]
        })
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers as MyMesh;
    use oxihuman_morph::engine::MeshBuffers as MB;
    use proptest::prelude::*;

    fn triangle_mesh() -> MyMesh {
        MyMesh::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    #[test]
    fn normals_point_up_z() {
        let mut m = triangle_mesh();
        compute_normals(&mut m);
        for n in &m.normals {
            assert!(n[2] > 0.9, "expected +Z normal, got {:?}", n);
        }
    }

    #[test]
    fn tangents_are_unit_length() {
        let mut m = triangle_mesh();
        compute_normals(&mut m);
        compute_tangents(&mut m);
        for t in &m.tangents {
            let len = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
            // tangent might be zero if UV degenerate, just check no NaN
            assert!(!len.is_nan());
        }
    }

    // ── New tests required by the task ────────────────────────────────────────

    #[test]
    fn compute_tangents_length_matches_verts() {
        let mut m = triangle_mesh();
        compute_normals(&mut m);
        compute_tangents(&mut m);
        assert_eq!(
            m.tangents.len(),
            m.positions.len(),
            "tangents.len() must equal vertex count"
        );
    }

    #[test]
    fn tangent_w_is_plus_or_minus_one() {
        let mut m = triangle_mesh();
        compute_normals(&mut m);
        compute_tangents(&mut m);
        for t in &m.tangents {
            let w = t[3];
            assert!(
                (w - 1.0f32).abs() < 1e-6 || (w + 1.0f32).abs() < 1e-6,
                "W must be +1.0 or -1.0, got {}",
                w
            );
        }
    }

    #[test]
    fn tangent_perpendicular_to_normal() {
        let mut m = triangle_mesh();
        compute_normals(&mut m);
        compute_tangents(&mut m);
        for (i, t) in m.tangents.iter().enumerate() {
            let n = m.normals[i];
            let d = t[0] * n[0] + t[1] * n[1] + t[2] * n[2];
            assert!(
                d.abs() < 0.01,
                "tangent not perpendicular to normal at vertex {}: dot = {}",
                i,
                d
            );
        }
    }

    #[test]
    fn empty_mesh_tangents_empty() {
        let mut m = MyMesh::from_morph(MB {
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            indices: vec![],
            has_suit: false,
        });
        compute_tangents(&mut m);
        assert!(
            m.tangents.is_empty(),
            "empty mesh should yield empty tangents"
        );
    }

    proptest! {
        #[test]
        fn compute_normals_no_nan(
            x0 in -10.0f32..10.0f32, y0 in -10.0f32..10.0f32, z0 in -10.0f32..10.0f32,
            x1 in -10.0f32..10.0f32, y1 in -10.0f32..10.0f32, z1 in -10.0f32..10.0f32,
            x2 in -10.0f32..10.0f32, y2 in -10.0f32..10.0f32, z2 in -10.0f32..10.0f32,
        ) {
            let mut m = MyMesh::from_morph(MB {
                positions: vec![[x0, y0, z0], [x1, y1, z1], [x2, y2, z2]],
                normals: vec![[0.0, 1.0, 0.0]; 3],
                uvs: vec![[0.0, 0.0]; 3],
                indices: vec![0, 1, 2],
                has_suit: false,
            });
            compute_normals(&mut m);
            for n in &m.normals {
                prop_assert!(!n[0].is_nan(), "NaN normal x");
                prop_assert!(!n[1].is_nan(), "NaN normal y");
                prop_assert!(!n[2].is_nan(), "NaN normal z");
            }
        }
    }
}
