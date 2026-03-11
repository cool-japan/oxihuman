// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Transfer scalar/vector attributes from a source mesh to a target mesh
//! using nearest-neighbour or barycentric interpolation.

/// A generic scalar attribute per vertex.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScalarAttr {
    pub name: String,
    pub values: Vec<f32>,
}

/// A generic 3-vector attribute per vertex.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Vec3Attr {
    pub name: String,
    pub values: Vec<[f32; 3]>,
}

/// Transfer method.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferMethod {
    NearestNeighbour,
    Barycentric,
}

/// Result of attribute transfer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TransferResult {
    pub scalar_attrs: Vec<ScalarAttr>,
    pub vec3_attrs: Vec<Vec3Attr>,
    pub target_vertex_count: usize,
}

/// Squared distance between two 3D points.
#[allow(dead_code)]
pub fn sq_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

/// Find the nearest source vertex index for a query point.
#[allow(dead_code)]
pub fn nearest_vertex(query: [f32; 3], source_pos: &[[f32; 3]]) -> usize {
    source_pos
        .iter()
        .enumerate()
        .map(|(i, &p)| (i, sq_dist(query, p)))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Transfer a scalar attribute via nearest-neighbour lookup.
#[allow(dead_code)]
pub fn transfer_scalar_nn(
    src_pos: &[[f32; 3]],
    src_attr: &ScalarAttr,
    tgt_pos: &[[f32; 3]],
) -> ScalarAttr {
    let values = tgt_pos
        .iter()
        .map(|&q| {
            let ni = nearest_vertex(q, src_pos);
            src_attr.values[ni]
        })
        .collect();
    ScalarAttr {
        name: src_attr.name.clone(),
        values,
    }
}

/// Transfer a vec3 attribute via nearest-neighbour lookup.
#[allow(dead_code)]
pub fn transfer_vec3_nn(
    src_pos: &[[f32; 3]],
    src_attr: &Vec3Attr,
    tgt_pos: &[[f32; 3]],
) -> Vec3Attr {
    let values = tgt_pos
        .iter()
        .map(|&q| {
            let ni = nearest_vertex(q, src_pos);
            src_attr.values[ni]
        })
        .collect();
    Vec3Attr {
        name: src_attr.name.clone(),
        values,
    }
}

/// Compute barycentric coordinates of point `p` on triangle (a, b, c).
/// Returns None if the point is too far from the triangle plane.
#[allow(dead_code)]
pub fn barycentric(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Option<[f32; 3]> {
    let v0 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v1 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let v2 = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let d00 = dot(v0, v0);
    let d01 = dot(v0, v1);
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-8 {
        return None;
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    Some([u, v, w])
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Scale a scalar attribute by a constant factor.
#[allow(dead_code)]
pub fn scale_scalar_attr(attr: &mut ScalarAttr, factor: f32) {
    for v in &mut attr.values {
        *v *= factor;
    }
}

/// Check if two scalar attributes have the same name.
#[allow(dead_code)]
pub fn attrs_same_name(a: &ScalarAttr, b: &ScalarAttr) -> bool {
    a.name == b.name
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three_pts() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }
    fn attr_ones() -> ScalarAttr {
        ScalarAttr {
            name: "w".to_string(),
            values: vec![1.0, 2.0, 3.0],
        }
    }

    #[test]
    fn test_sq_dist_zero() {
        assert!((sq_dist([1.0, 2.0, 3.0], [1.0, 2.0, 3.0])).abs() < 1e-6);
    }

    #[test]
    fn test_nearest_vertex() {
        let src = three_pts();
        let idx = nearest_vertex([0.9, 0.0, 0.0], &src);
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_transfer_scalar_nn() {
        let src = three_pts();
        let attr = attr_ones();
        let tgt = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let out = transfer_scalar_nn(&src, &attr, &tgt);
        assert!((out.values[0] - 1.0).abs() < 1e-6);
        assert!((out.values[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_transfer_vec3_nn() {
        let src = three_pts();
        let v3 = Vec3Attr {
            name: "n".to_string(),
            values: vec![[0.0, 0.0, 1.0]; 3],
        };
        let tgt = vec![[0.5, 0.0, 0.0]];
        let out = transfer_vec3_nn(&src, &v3, &tgt);
        assert!((out.values[0][2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_barycentric_centroid() {
        let a = [0.0_f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let p = [1.0 / 3.0, 1.0 / 3.0, 0.0];
        let bary = barycentric(p, a, b, c).unwrap();
        assert!((bary[0] - 1.0 / 3.0).abs() < 1e-5);
        assert!((bary[1] - 1.0 / 3.0).abs() < 1e-5);
        assert!((bary[2] - 1.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale_scalar_attr() {
        let mut a = attr_ones();
        scale_scalar_attr(&mut a, 2.0);
        assert!((a.values[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_attrs_same_name_true() {
        let a = ScalarAttr {
            name: "x".to_string(),
            values: vec![],
        };
        let b = ScalarAttr {
            name: "x".to_string(),
            values: vec![],
        };
        assert!(attrs_same_name(&a, &b));
    }

    #[test]
    fn test_attrs_same_name_false() {
        let a = ScalarAttr {
            name: "x".to_string(),
            values: vec![],
        };
        let b = ScalarAttr {
            name: "y".to_string(),
            values: vec![],
        };
        assert!(!attrs_same_name(&a, &b));
    }

    #[test]
    fn test_transfer_empty_target() {
        let src = three_pts();
        let attr = attr_ones();
        let out = transfer_scalar_nn(&src, &attr, &[]);
        assert!(out.values.is_empty());
    }

    #[test]
    fn test_nearest_vertex_exact_match() {
        let src = three_pts();
        let idx = nearest_vertex([0.0, 1.0, 0.0], &src);
        assert_eq!(idx, 2);
    }
}
