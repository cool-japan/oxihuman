// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Triangle face area computation and statistics.

#[allow(dead_code)]
pub fn triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0]-a[0], b[1]-a[1], b[2]-a[2]];
    let ac = [c[0]-a[0], c[1]-a[1], c[2]-a[2]];
    let cross = [ab[1]*ac[2]-ab[2]*ac[1], ab[2]*ac[0]-ab[0]*ac[2], ab[0]*ac[1]-ab[1]*ac[0]];
    0.5 * (cross[0]*cross[0]+cross[1]*cross[1]+cross[2]*cross[2]).sqrt()
}

#[allow(dead_code)]
pub fn face_areas(positions: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<f32> {
    faces.iter().map(|f| triangle_area(positions[f[0] as usize], positions[f[1] as usize], positions[f[2] as usize])).collect()
}

#[allow(dead_code)]
pub fn total_surface_area(positions: &[[f32; 3]], faces: &[[u32; 3]]) -> f32 {
    face_areas(positions, faces).iter().sum()
}

#[allow(dead_code)]
pub fn min_face_area(areas: &[f32]) -> f32 {
    areas.iter().copied().fold(f32::MAX, f32::min)
}

#[allow(dead_code)]
pub fn max_face_area(areas: &[f32]) -> f32 {
    areas.iter().copied().fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn mean_face_area(areas: &[f32]) -> f32 {
    if areas.is_empty() { return 0.0; }
    areas.iter().sum::<f32>() / areas.len() as f32
}

#[allow(dead_code)]
pub fn face_area_variance(areas: &[f32]) -> f32 {
    if areas.is_empty() { return 0.0; }
    let mean = mean_face_area(areas);
    areas.iter().map(|&a| (a - mean) * (a - mean)).sum::<f32>() / areas.len() as f32
}

#[allow(dead_code)]
pub fn degenerate_face_count(areas: &[f32], threshold: f32) -> usize {
    areas.iter().filter(|&&a| a < threshold).count()
}

#[allow(dead_code)]
pub fn face_area_to_json(areas: &[f32]) -> String {
    format!("{{\"count\":{},\"total\":{:.4},\"mean\":{:.4},\"min\":{:.4},\"max\":{:.4}}}",
        areas.len(), areas.iter().sum::<f32>(), mean_face_area(areas),
        if areas.is_empty() { 0.0 } else { min_face_area(areas) },
        max_face_area(areas))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        (vec![[0.0,0.0,0.0],[2.0,0.0,0.0],[0.0,2.0,0.0]], vec![[0,1,2]])
    }

    #[test] fn test_area() { assert!((triangle_area([0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]) - 0.5).abs() < 1e-5); }
    #[test] fn test_face_areas() { let(p,f)=tri(); let a=face_areas(&p,&f); assert_eq!(a.len(),1); assert!((a[0]-2.0).abs()<1e-4); }
    #[test] fn test_total() { let(p,f)=tri(); assert!((total_surface_area(&p,&f)-2.0).abs()<1e-4); }
    #[test] fn test_min() { assert!((min_face_area(&[1.0,2.0,3.0]) - 1.0).abs() < 1e-6); }
    #[test] fn test_max() { assert!((max_face_area(&[1.0,2.0,3.0]) - 3.0).abs() < 1e-6); }
    #[test] fn test_mean() { assert!((mean_face_area(&[1.0,2.0,3.0]) - 2.0).abs() < 1e-6); }
    #[test] fn test_variance() { assert!(face_area_variance(&[1.0,1.0,1.0]).abs() < 1e-6); }
    #[test] fn test_degenerate() { assert_eq!(degenerate_face_count(&[0.0001, 1.0, 2.0], 0.001), 1); }
    #[test] fn test_to_json() { assert!(face_area_to_json(&[1.0,2.0]).contains("mean")); }
    #[test] fn test_empty() { assert!((mean_face_area(&[])).abs() < 1e-6); }
}
