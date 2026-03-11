// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Detection and handling of needle (sliver) triangles.
#[allow(dead_code)]
pub struct NeedleDetectResult {
    pub needle_indices: Vec<usize>,
    pub aspect_ratios: Vec<f32>,
    pub threshold: f32,
}

fn edge_len(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vec_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    vec_len(cross3(ab, ac)) * 0.5
}

/// Aspect ratio = longest_edge / (2 * inradius). Higher = needle.
#[allow(dead_code)]
pub fn triangle_aspect_ratio_needle(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let lab = edge_len(a, b);
    let lbc = edge_len(b, c);
    let lca = edge_len(c, a);
    let longest = lab.max(lbc).max(lca);
    let area = triangle_area(a, b, c);
    let perimeter = lab + lbc + lca;
    if area < 1e-12 || perimeter < 1e-12 {
        return f32::INFINITY;
    }
    let inradius = area / (perimeter * 0.5);
    longest / (2.0 * inradius)
}

/// Detect needle triangles where aspect ratio exceeds threshold.
#[allow(dead_code)]
pub fn detect_needle_triangles(
    positions: &[[f32; 3]],
    indices: &[u32],
    threshold: f32,
) -> NeedleDetectResult {
    let tri_count = indices.len() / 3;
    let mut needle_indices = Vec::new();
    let mut aspect_ratios = Vec::with_capacity(tri_count);

    #[allow(clippy::needless_range_loop)]
    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        if ia >= positions.len() || ib >= positions.len() || ic >= positions.len() {
            aspect_ratios.push(f32::INFINITY);
            continue;
        }
        let ar = triangle_aspect_ratio_needle(positions[ia], positions[ib], positions[ic]);
        aspect_ratios.push(ar);
        if ar > threshold {
            needle_indices.push(t);
        }
    }

    NeedleDetectResult {
        needle_indices,
        aspect_ratios,
        threshold,
    }
}

#[allow(dead_code)]
pub fn needle_count(r: &NeedleDetectResult) -> usize {
    r.needle_indices.len()
}

#[allow(dead_code)]
pub fn needle_ratio(r: &NeedleDetectResult) -> f32 {
    if r.aspect_ratios.is_empty() {
        return 0.0;
    }
    r.needle_indices.len() as f32 / r.aspect_ratios.len() as f32
}

#[allow(dead_code)]
pub fn max_aspect_ratio(r: &NeedleDetectResult) -> f32 {
    r.aspect_ratios
        .iter()
        .cloned()
        .filter(|v| v.is_finite())
        .fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn avg_aspect_ratio(r: &NeedleDetectResult) -> f32 {
    let finite: Vec<f32> = r
        .aspect_ratios
        .iter()
        .cloned()
        .filter(|v| v.is_finite())
        .collect();
    if finite.is_empty() {
        return 0.0;
    }
    finite.iter().sum::<f32>() / finite.len() as f32
}

#[allow(dead_code)]
pub fn needle_result_to_json(r: &NeedleDetectResult) -> String {
    format!(
        "{{\"needle_count\":{},\"total_triangles\":{},\"threshold\":{}}}",
        r.needle_indices.len(),
        r.aspect_ratios.len(),
        r.threshold
    )
}

#[allow(dead_code)]
pub fn is_needle_free(r: &NeedleDetectResult) -> bool {
    r.needle_indices.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn equilateral_triangle() -> (Vec<[f32; 3]>, Vec<u32>) {
        use std::f32::consts::PI;
        let r = 1.0f32;
        let pos = vec![
            [r * (0.0f32).cos(), 0.0, r * (0.0f32).sin()],
            [r * (2.0 * PI / 3.0).cos(), 0.0, r * (2.0 * PI / 3.0).sin()],
            [r * (4.0 * PI / 3.0).cos(), 0.0, r * (4.0 * PI / 3.0).sin()],
        ];
        (pos, vec![0, 1, 2])
    }

    fn needle_triangle() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [50.0, 0.01, 0.0]];
        (pos, vec![0, 1, 2])
    }

    #[test]
    fn test_equilateral_low_aspect_ratio() {
        let (pos, idx) = equilateral_triangle();
        let ar = triangle_aspect_ratio_needle(pos[0], pos[1], pos[2]);
        let _ = idx;
        assert!(ar < 2.0);
    }

    #[test]
    fn test_needle_high_aspect_ratio() {
        let (pos, _) = needle_triangle();
        let ar = triangle_aspect_ratio_needle(pos[0], pos[1], pos[2]);
        assert!(ar > 100.0);
    }

    #[test]
    fn test_detect_no_needles_equilateral() {
        let (pos, idx) = equilateral_triangle();
        let r = detect_needle_triangles(&pos, &idx, 5.0);
        assert_eq!(needle_count(&r), 0);
    }

    #[test]
    fn test_detect_needle_found() {
        let (pos, idx) = needle_triangle();
        let r = detect_needle_triangles(&pos, &idx, 10.0);
        assert_eq!(needle_count(&r), 1);
    }

    #[test]
    fn test_empty_mesh() {
        let r = detect_needle_triangles(&[], &[], 5.0);
        assert_eq!(needle_count(&r), 0);
        assert!(is_needle_free(&r));
    }

    #[test]
    fn test_needle_ratio() {
        let (pos, idx) = needle_triangle();
        let r = detect_needle_triangles(&pos, &idx, 10.0);
        assert!((needle_ratio(&r) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_aspect_ratio() {
        let (pos, idx) = needle_triangle();
        let r = detect_needle_triangles(&pos, &idx, 1.0);
        assert!(max_aspect_ratio(&r) > 100.0);
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = needle_triangle();
        let r = detect_needle_triangles(&pos, &idx, 10.0);
        let j = needle_result_to_json(&r);
        assert!(j.contains("needle_count"));
    }

    #[test]
    fn test_avg_aspect_ratio_positive() {
        let (pos, idx) = needle_triangle();
        let r = detect_needle_triangles(&pos, &idx, 1.0);
        assert!(avg_aspect_ratio(&r) > 0.0);
    }

    #[test]
    fn test_is_needle_free_true() {
        let (pos, idx) = equilateral_triangle();
        let r = detect_needle_triangles(&pos, &idx, 1000.0);
        assert!(is_needle_free(&r));
    }
}
