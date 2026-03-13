#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Interpolate arbitrary per-vertex float attributes across a mesh.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub name: String,
    pub values: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_vertex_attribute(name: &str, n_verts: usize) -> VertexAttribute {
    VertexAttribute {
        name: name.to_string(),
        values: vec![0.0; n_verts],
    }
}

#[allow(dead_code)]
pub fn interpolate_attribute(
    attr: &VertexAttribute,
    i0: usize,
    i1: usize,
    i2: usize,
    bary: [f32; 3],
) -> f32 {
    let v0 = attr.values.get(i0).copied().unwrap_or(0.0);
    let v1 = attr.values.get(i1).copied().unwrap_or(0.0);
    let v2 = attr.values.get(i2).copied().unwrap_or(0.0);
    v0 * bary[0] + v1 * bary[1] + v2 * bary[2]
}

#[allow(dead_code)]
pub fn blend_attributes(a: &VertexAttribute, b: &VertexAttribute, t: f32) -> VertexAttribute {
    let len = a.values.len().min(b.values.len());
    let values = (0..len)
        .map(|i| a.values[i] * (1.0 - t) + b.values[i] * t)
        .collect();
    VertexAttribute {
        name: a.name.clone(),
        values,
    }
}

#[allow(dead_code)]
pub fn attribute_range(attr: &VertexAttribute) -> (f32, f32) {
    if attr.values.is_empty() {
        return (0.0, 0.0);
    }
    let mut mn = f32::MAX;
    let mut mx = f32::MIN;
    for &v in &attr.values {
        if v < mn {
            mn = v;
        }
        if v > mx {
            mx = v;
        }
    }
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_attribute_zeros() {
        let a = new_vertex_attribute("skin", 5);
        assert_eq!(a.values.len(), 5);
        assert!(a.values.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn new_attribute_name() {
        let a = new_vertex_attribute("test", 3);
        assert_eq!(a.name, "test");
    }

    #[test]
    fn interpolate_centroid_equal() {
        let mut a = new_vertex_attribute("x", 3);
        a.values = vec![0.0, 3.0, 6.0];
        let v = interpolate_attribute(&a, 0, 1, 2, [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]);
        assert!((v - 3.0).abs() < 1e-5);
    }

    #[test]
    fn interpolate_at_vertex_0() {
        let mut a = new_vertex_attribute("x", 3);
        a.values = vec![1.0, 2.0, 3.0];
        let v = interpolate_attribute(&a, 0, 1, 2, [1.0, 0.0, 0.0]);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn blend_t0_equals_a() {
        let mut a = new_vertex_attribute("a", 2);
        a.values = vec![1.0, 2.0];
        let mut b = new_vertex_attribute("b", 2);
        b.values = vec![3.0, 4.0];
        let r = blend_attributes(&a, &b, 0.0);
        assert!((r.values[0] - 1.0).abs() < 1e-6);
        assert!((r.values[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn blend_t1_equals_b() {
        let mut a = new_vertex_attribute("a", 2);
        a.values = vec![1.0, 2.0];
        let mut b = new_vertex_attribute("b", 2);
        b.values = vec![3.0, 4.0];
        let r = blend_attributes(&a, &b, 1.0);
        assert!((r.values[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn attribute_range_empty() {
        let a = new_vertex_attribute("x", 0);
        let (mn, mx) = attribute_range(&a);
        assert_eq!(mn, 0.0);
        assert_eq!(mx, 0.0);
    }

    #[test]
    fn attribute_range_correct() {
        let mut a = new_vertex_attribute("x", 3);
        a.values = vec![2.0, 5.0, 1.0];
        let (mn, mx) = attribute_range(&a);
        assert!((mn - 1.0).abs() < 1e-6);
        assert!((mx - 5.0).abs() < 1e-6);
    }

    #[test]
    fn interpolate_oob_returns_zero() {
        let a = new_vertex_attribute("x", 1);
        let v = interpolate_attribute(&a, 0, 10, 20, [1.0, 0.0, 0.0]);
        assert!(v.is_finite());
    }
}
