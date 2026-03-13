// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mesh surface area measurement.

#[allow(dead_code)]
pub fn am_triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cx = ab[1] * ac[2] - ab[2] * ac[1];
    let cy = ab[2] * ac[0] - ab[0] * ac[2];
    let cz = ab[0] * ac[1] - ab[1] * ac[0];
    (cx * cx + cy * cy + cz * cz).sqrt() / 2.0
}

#[allow(dead_code)]
pub fn am_total_area(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    am_face_areas(positions, indices).iter().sum()
}

#[allow(dead_code)]
pub fn am_face_areas(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Vec<f32> {
    indices.iter().map(|tri| {
        let a = tri[0] as usize;
        let b = tri[1] as usize;
        let c = tri[2] as usize;
        if a < positions.len() && b < positions.len() && c < positions.len() {
            am_triangle_area(positions[a], positions[b], positions[c])
        } else {
            0.0
        }
    }).collect()
}

#[allow(dead_code)]
pub fn am_largest_face(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Option<usize> {
    let areas = am_face_areas(positions, indices);
    if areas.is_empty() { return None; }
    let mut max_i = 0;
    let mut max_a = areas[0];
    for (i, &a) in areas.iter().enumerate() {
        if a > max_a { max_a = a; max_i = i; }
    }
    Some(max_i)
}

#[allow(dead_code)]
pub fn am_smallest_face(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Option<usize> {
    let areas = am_face_areas(positions, indices);
    if areas.is_empty() { return None; }
    let mut min_i = 0;
    let mut min_a = areas[0];
    for (i, &a) in areas.iter().enumerate() {
        if a < min_a { min_a = a; min_i = i; }
    }
    Some(min_i)
}

#[allow(dead_code)]
pub fn am_area_variance(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    let areas = am_face_areas(positions, indices);
    let n = areas.len();
    if n == 0 { return 0.0; }
    let mean = areas.iter().sum::<f32>() / n as f32;
    areas.iter().map(|&a| (a - mean) * (a - mean)).sum::<f32>() / n as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_right_triangle_area() {
        let area = am_triangle_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((area - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_total_area_two_triangles() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        let indices = vec![[0u32, 1, 2], [1, 3, 2]];
        let total = am_total_area(&positions, &indices);
        assert!((total - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_face_areas_count() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![[0u32, 1, 2]];
        let areas = am_face_areas(&positions, &indices);
        assert_eq!(areas.len(), 1);
    }

    #[test]
    fn test_largest_face() {
        let positions = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0],
        ];
        let indices = vec![[0u32, 1, 2], [3, 4, 5]];
        assert_eq!(am_largest_face(&positions, &indices), Some(1));
    }

    #[test]
    fn test_smallest_face() {
        let positions = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0],
        ];
        let indices = vec![[0u32, 1, 2], [3, 4, 5]];
        assert_eq!(am_smallest_face(&positions, &indices), Some(0));
    }

    #[test]
    fn test_largest_empty() {
        assert_eq!(am_largest_face(&[], &[]), None);
    }

    #[test]
    fn test_area_variance_uniform() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![[0u32, 1, 2], [0, 1, 2]];
        let var = am_area_variance(&positions, &indices);
        assert!(var < 1e-5);
    }

    #[test]
    fn test_area_variance_empty() {
        assert_eq!(am_area_variance(&[], &[]), 0.0);
    }
}
