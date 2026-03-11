// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Clothing overlay pipeline.
//!
//! Deforms a clothing mesh to match a morphed base mesh using barycentric
//! interpolation from `.mhclo` binding data (MakeHuman algorithm).
//!
//! Algorithm (from MakeHuman `mh2proxy.py`):
//!   `clothing_pos[i]` = sum(`weights[j]` * `base_pos[base_verts[j]]`) + offset
//!   for j in 0..3

use anyhow::{bail, Result};
use oxihuman_core::parser::mhclo::{ClothingBinding, VertexBinding};
use oxihuman_core::parser::obj::ObjMesh;

/// A deformed clothing mesh positioned on a morphed base body.
#[derive(Debug, Clone)]
pub struct ClothingMesh {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Apply a clothing binding to a morphed base mesh, deforming the clothing
/// vertex positions to match the current body shape.
///
/// # Arguments
/// * `base_positions` — morphed base mesh vertex positions (19k verts)
/// * `clothing_obj`   — the raw clothing `.obj` mesh
/// * `binding`        — the `.mhclo` binding describing how clothing verts map to base verts
///
/// # Returns
/// A `ClothingMesh` with positions deformed to fit the current body.
pub fn apply_clothing(
    base_positions: &[[f32; 3]],
    clothing_obj: &ObjMesh,
    binding: &ClothingBinding,
) -> Result<ClothingMesh> {
    if binding.vertex_map.len() != clothing_obj.positions.len() {
        bail!(
            "binding has {} entries but clothing obj has {} vertices",
            binding.vertex_map.len(),
            clothing_obj.positions.len()
        );
    }

    let n_base = base_positions.len();
    let mut deformed = Vec::with_capacity(clothing_obj.positions.len());

    for vb in &binding.vertex_map {
        let pos = interpolate_vertex(base_positions, vb, n_base)?;
        deformed.push(pos);
    }

    Ok(ClothingMesh {
        positions: deformed,
        uvs: clothing_obj.uvs.clone(),
        normals: clothing_obj.normals.clone(),
        indices: clothing_obj.indices.clone(),
    })
}

/// Compute a single clothing vertex position via barycentric interpolation.
fn interpolate_vertex(
    base_positions: &[[f32; 3]],
    vb: &VertexBinding,
    n_base: usize,
) -> Result<[f32; 3]> {
    let [i0, i1, i2] = vb.base_verts;
    if i0 as usize >= n_base || i1 as usize >= n_base || i2 as usize >= n_base {
        bail!(
            "base vertex index out of range: ({}, {}, {}) >= {}",
            i0,
            i1,
            i2,
            n_base
        );
    }

    let p0 = base_positions[i0 as usize];
    let p1 = base_positions[i1 as usize];
    let p2 = base_positions[i2 as usize];
    let [w0, w1, w2] = vb.weights;
    let [ox, oy, oz] = vb.offset;

    Ok([
        w0 * p0[0] + w1 * p1[0] + w2 * p2[0] + ox,
        w0 * p0[1] + w1 * p1[1] + w2 * p2[1] + oy,
        w0 * p0[2] + w1 * p1[2] + w2 * p2[2] + oz,
    ])
}

/// Convenience: apply clothing using positions from a `MeshBuffers`.
pub fn apply_clothing_to_mesh(
    mesh_positions: &[[f32; 3]],
    clothing_obj: &ObjMesh,
    binding: &ClothingBinding,
) -> Result<ClothingMesh> {
    apply_clothing(mesh_positions, clothing_obj, binding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_core::parser::mhclo::{ClothingBinding, VertexBinding};
    use oxihuman_core::parser::obj::ObjMesh;

    fn base_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]
    }

    fn simple_binding() -> ClothingBinding {
        ClothingBinding {
            uuid: "test".to_string(),
            basemesh: "hm08".to_string(),
            name: "test_cloth".to_string(),
            obj_file: "test.obj".to_string(),
            vertex_map: vec![
                VertexBinding {
                    base_verts: [0, 1, 2],
                    weights: [1.0, 0.0, 0.0],
                    offset: [0.0, 0.0, 0.0],
                },
                VertexBinding {
                    base_verts: [0, 1, 2],
                    weights: [0.5, 0.5, 0.0],
                    offset: [0.0, 0.1, 0.0],
                },
            ],
        }
    }

    fn simple_clothing_obj() -> ObjMesh {
        ObjMesh {
            positions: vec![[0.0, 0.0, 0.0], [0.5, 0.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
            uvs: vec![[0.0, 0.0], [1.0, 0.0]],
            indices: vec![0, 1, 0],
        }
    }

    #[test]
    fn barycentric_at_vertex_0() {
        let base = base_positions();
        let obj = simple_clothing_obj();
        let binding = simple_binding();
        let cloth = apply_clothing(&base, &obj, &binding).unwrap();
        // First vertex: weights=[1,0,0] → exactly base[0] = [0,0,0]
        assert!((cloth.positions[0][0] - 0.0).abs() < 1e-6);
        assert!((cloth.positions[0][1] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn barycentric_midpoint_with_offset() {
        let base = base_positions();
        let obj = simple_clothing_obj();
        let binding = simple_binding();
        let cloth = apply_clothing(&base, &obj, &binding).unwrap();
        // Second vertex: 0.5*base[0] + 0.5*base[1] + offset[1]=0.1
        // = 0.5*[0,0,0] + 0.5*[1,0,0] + [0,0.1,0] = [0.5, 0.1, 0]
        assert!((cloth.positions[1][0] - 0.5).abs() < 1e-6);
        assert!((cloth.positions[1][1] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn vertex_count_matches_clothing_obj() {
        let base = base_positions();
        let obj = simple_clothing_obj();
        let binding = simple_binding();
        let cloth = apply_clothing(&base, &obj, &binding).unwrap();
        assert_eq!(cloth.positions.len(), obj.positions.len());
    }

    #[test]
    fn mismatched_binding_errors() {
        let base = base_positions();
        let mut obj = simple_clothing_obj();
        obj.positions.push([2.0, 0.0, 0.0]); // 3 verts but binding has 2 entries
        let binding = simple_binding();
        assert!(apply_clothing(&base, &obj, &binding).is_err());
    }

    #[test]
    fn out_of_bounds_base_index_errors() {
        let base = vec![[0.0f32, 0.0, 0.0]]; // only 1 base vert
        let obj = ObjMesh {
            positions: vec![[0.0, 0.0, 0.0]],
            normals: vec![],
            uvs: vec![],
            indices: vec![],
        };
        let binding = ClothingBinding {
            uuid: "t".to_string(),
            basemesh: "hm08".to_string(),
            name: "t".to_string(),
            obj_file: "t.obj".to_string(),
            vertex_map: vec![VertexBinding {
                base_verts: [0, 999, 0], // 999 is out of bounds
                weights: [0.5, 0.5, 0.0],
                offset: [0.0, 0.0, 0.0],
            }],
        };
        assert!(apply_clothing(&base, &obj, &binding).is_err());
    }

    #[test]
    fn clothing_tracks_morphed_base() {
        // Simulate a morph: move base[1] by +0.5 in X
        let mut base = base_positions();
        base[1][0] += 0.5; // base[1] is now [1.5, 0, 0]

        let obj = simple_clothing_obj();
        let binding = simple_binding();
        let cloth = apply_clothing(&base, &obj, &binding).unwrap();
        // Second clothing vert: 0.5*[0,0,0] + 0.5*[1.5,0,0] + [0,0.1,0] = [0.75, 0.1, 0]
        assert!((cloth.positions[1][0] - 0.75).abs() < 1e-6);
    }
}
