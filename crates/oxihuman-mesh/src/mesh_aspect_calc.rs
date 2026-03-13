// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Triangle aspect ratio metrics.

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn cross_mag(ab: [f32; 3], ac: [f32; 3]) -> f32 {
    let cx = ab[1] * ac[2] - ab[2] * ac[1];
    let cy = ab[2] * ac[0] - ab[0] * ac[2];
    let cz = ab[0] * ac[1] - ab[1] * ac[0];
    (cx * cx + cy * cy + cz * cz).sqrt()
}

#[allow(dead_code)]
pub fn ar_triangle_calc(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let lab = dist(a, b);
    let lbc = dist(b, c);
    let lac = dist(a, c);
    let s = (lab + lbc + lac) / 2.0;
    let area = {
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        cross_mag(ab, ac) / 2.0
    };
    if area < 1e-10 { return f32::MAX; }
    let circumr = (lab * lbc * lac) / (4.0 * area);
    let inr = area / s;
    if inr < 1e-10 { return f32::MAX; }
    circumr / inr
}

#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn ar_mesh_stats_calc(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> (f32, f32, f32) {
    if indices.is_empty() { return (0.0, 0.0, 0.0); }
    let mut min_ar = f32::MAX;
    let mut max_ar = 0.0f32;
    let mut sum = 0.0f32;
    let n = indices.len();
    for i in 0..n {
        let tri = &indices[i];
        let a = tri[0] as usize;
        let b = tri[1] as usize;
        let c = tri[2] as usize;
        if a < positions.len() && b < positions.len() && c < positions.len() {
            let ar = ar_triangle_calc(positions[a], positions[b], positions[c]);
            if ar < min_ar { min_ar = ar; }
            if ar > max_ar { max_ar = ar; }
            sum += ar;
        }
    }
    (min_ar, max_ar, sum / n as f32)
}

#[allow(dead_code)]
pub fn ar_is_equilateral_calc(a: [f32; 3], b: [f32; 3], c: [f32; 3], tol: f32) -> bool {
    let ar = ar_triangle_calc(a, b, c);
    (ar - 2.0).abs() < tol
}

#[allow(dead_code)]
pub fn ar_worst_triangle_calc(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Option<usize> {
    if indices.is_empty() { return None; }
    let mut worst_idx = 0;
    let mut worst_ar = 0.0f32;
    for (i, tri) in indices.iter().enumerate() {
        let a = tri[0] as usize;
        let b = tri[1] as usize;
        let c = tri[2] as usize;
        if a < positions.len() && b < positions.len() && c < positions.len() {
            let ar = ar_triangle_calc(positions[a], positions[b], positions[c]);
            if ar > worst_ar {
                worst_ar = ar;
                worst_idx = i;
            }
        }
    }
    Some(worst_idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn equilateral() -> ([f32; 3], [f32; 3], [f32; 3]) {
        let h = (3.0f32).sqrt() / 2.0;
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, h, 0.0])
    }

    #[test]
    fn test_equilateral_ar_is_two() {
        let (a, b, c) = equilateral();
        let ar = ar_triangle_calc(a, b, c);
        assert!((ar - 2.0).abs() < 0.01, "Expected AR~2 for equilateral, got {ar}");
    }

    #[test]
    fn test_equilateral_detection() {
        let (a, b, c) = equilateral();
        assert!(ar_is_equilateral_calc(a, b, c, 0.05));
    }

    #[test]
    fn test_degenerate_triangle() {
        let ar = ar_triangle_calc([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert!(ar >= f32::MAX || ar > 1000.0);
    }

    #[test]
    fn test_worst_triangle() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.866, 0.0],
                             [0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [0.0, 0.001, 0.0]];
        let indices = vec![[0u32, 1, 2], [3, 4, 5]];
        let worst = ar_worst_triangle_calc(&positions, &indices);
        assert_eq!(worst, Some(1));
    }

    #[test]
    fn test_worst_triangle_empty() {
        assert_eq!(ar_worst_triangle_calc(&[], &[]), None);
    }

    #[test]
    fn test_mesh_stats_single_equilateral() {
        let (a, b, c) = equilateral();
        let positions = vec![a, b, c];
        let indices = vec![[0u32, 1, 2]];
        let (mn, mx, avg) = ar_mesh_stats_calc(&positions, &indices);
        assert!((mn - mx).abs() < 0.01);
        assert!((avg - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_mesh_stats_empty() {
        let (mn, mx, avg) = ar_mesh_stats_calc(&[], &[]);
        assert_eq!((mn, mx, avg), (0.0, 0.0, 0.0));
    }

    #[test]
    fn test_right_triangle_ar_gt_equilateral() {
        let ar_right = ar_triangle_calc([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let (a, b, c) = equilateral();
        let ar_eq = ar_triangle_calc(a, b, c);
        assert!(ar_right >= ar_eq, "Right triangle AR {ar_right} should be >= equilateral AR {ar_eq}");
    }

    #[test]
    fn test_not_equilateral() {
        assert!(!ar_is_equilateral_calc([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.5, 0.0], 0.05));
    }
}
