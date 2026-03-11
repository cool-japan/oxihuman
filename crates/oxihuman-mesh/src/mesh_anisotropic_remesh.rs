// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Anisotropic remeshing with directional size field.

#[allow(dead_code)]
pub struct SizeField {
    pub sizes: Vec<([f32; 3], f32)>,
}

#[allow(dead_code)]
pub fn new_size_field() -> SizeField {
    SizeField { sizes: Vec::new() }
}

#[allow(dead_code)]
pub fn sf_add(field: &mut SizeField, pos: [f32; 3], size: f32) {
    field.sizes.push((pos, size));
}

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

#[allow(dead_code)]
pub fn sf_query(field: &SizeField, pos: [f32; 3]) -> f32 {
    if field.sizes.is_empty() { return 1.0; }
    let mut best_size = field.sizes[0].1;
    let mut best_dist = dist_sq(field.sizes[0].0, pos);
    for &(p, s) in field.sizes.iter().skip(1) {
        let d = dist_sq(p, pos);
        if d < best_dist {
            best_dist = d;
            best_size = s;
        }
    }
    best_size
}

#[allow(dead_code)]
pub fn sf_count(field: &SizeField) -> usize {
    field.sizes.len()
}

#[allow(dead_code)]
pub struct AnisotropicRemesher {
    pub target_edge_len: f32,
    pub iterations: u32,
}

#[allow(dead_code)]
pub fn new_anisotropic_remesher(target: f32, iters: u32) -> AnisotropicRemesher {
    AnisotropicRemesher { target_edge_len: target, iterations: iters }
}

#[allow(dead_code)]
pub fn anr_target(r: &AnisotropicRemesher) -> f32 {
    r.target_edge_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_field_query_returns_one() {
        let field = new_size_field();
        assert!((sf_query(&field, [0.0, 0.0, 0.0]) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_sf_add_and_count() {
        let mut field = new_size_field();
        sf_add(&mut field, [0.0, 0.0, 0.0], 0.5);
        sf_add(&mut field, [1.0, 0.0, 0.0], 1.5);
        assert_eq!(sf_count(&field), 2);
    }

    #[test]
    fn test_sf_query_nearest() {
        let mut field = new_size_field();
        sf_add(&mut field, [0.0, 0.0, 0.0], 0.1);
        sf_add(&mut field, [10.0, 0.0, 0.0], 2.0);
        assert!((sf_query(&field, [0.1, 0.0, 0.0]) - 0.1).abs() < 1e-5);
        assert!((sf_query(&field, [9.9, 0.0, 0.0]) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_anr_target() {
        let r = new_anisotropic_remesher(0.25, 5);
        assert!((anr_target(&r) - 0.25).abs() < 1e-5);
    }

    #[test]
    fn test_anr_iterations() {
        let r = new_anisotropic_remesher(1.0, 10);
        assert_eq!(r.iterations, 10);
    }

    #[test]
    fn test_sf_count_zero() {
        let field = new_size_field();
        assert_eq!(sf_count(&field), 0);
    }

    #[test]
    fn test_sf_single_entry_query() {
        let mut field = new_size_field();
        sf_add(&mut field, [5.0, 5.0, 5.0], 1.5);
        assert!((sf_query(&field, [0.0, 0.0, 0.0]) - 1.5).abs() < 1e-4);
    }

    #[test]
    fn test_sf_add_many() {
        let mut field = new_size_field();
        for i in 0..8 {
            sf_add(&mut field, [i as f32, 0.0, 0.0], i as f32 * 0.1);
        }
        assert_eq!(sf_count(&field), 8);
    }
}
