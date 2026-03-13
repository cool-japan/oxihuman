// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Triangle aspect ratio analysis for mesh quality assessment.

/// Result of aspect ratio analysis.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AspectRatioResult {
    /// Per-triangle aspect ratios.
    pub ratios: Vec<f32>,
    /// Average aspect ratio.
    pub avg_ratio: f32,
    /// Worst (highest) aspect ratio.
    pub max_ratio: f32,
}

/// Compute distance between two 3D points.
#[allow(dead_code)]
pub fn edge_length(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute area of a triangle from three vertices.
#[allow(dead_code)]
pub fn triangle_area(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> f32 {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let cx = e1[1] * e2[2] - e1[2] * e2[1];
    let cy = e1[2] * e2[0] - e1[0] * e2[2];
    let cz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

/// Compute the aspect ratio of a triangle (longest edge / shortest altitude).
/// A perfect equilateral triangle has ratio ~1.155.
#[allow(dead_code)]
pub fn triangle_aspect_ratio(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> f32 {
    let a = edge_length(v0, v1);
    let b = edge_length(v1, v2);
    let c = edge_length(v2, v0);
    let longest = a.max(b).max(c);
    let area = triangle_area(v0, v1, v2);
    if area < 1e-12 {
        return f32::MAX;
    }
    let shortest_altitude = 2.0 * area / longest;
    longest / shortest_altitude
}

/// Compute aspect ratios for all triangles in a mesh.
#[allow(dead_code)]
pub fn compute_aspect_ratios(positions: &[[f32; 3]], indices: &[u32]) -> AspectRatioResult {
    let tri_count = indices.len() / 3;
    let mut ratios = Vec::with_capacity(tri_count);
    let mut sum = 0.0f32;
    let mut max_r = 0.0f32;

    #[allow(clippy::needless_range_loop)]
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let r = triangle_aspect_ratio(positions[i0], positions[i1], positions[i2]);
        ratios.push(r);
        if r < f32::MAX {
            sum += r;
            if r > max_r {
                max_r = r;
            }
        }
    }

    let valid_count = ratios.iter().filter(|&&r| r < f32::MAX).count();
    let avg = if valid_count > 0 {
        sum / valid_count as f32
    } else {
        0.0
    };
    AspectRatioResult {
        ratios,
        avg_ratio: avg,
        max_ratio: max_r,
    }
}

/// Count triangles exceeding a given aspect ratio threshold.
#[allow(dead_code)]
pub fn count_poor_triangles(result: &AspectRatioResult, threshold: f32) -> usize {
    result.ratios.iter().filter(|&&r| r > threshold).count()
}

/// Minimum aspect ratio in the result.
#[allow(dead_code)]
pub fn min_aspect_ratio(result: &AspectRatioResult) -> f32 {
    result.ratios.iter().copied().fold(f32::MAX, f32::min)
}

/// Get ratio at a specific triangle index.
#[allow(dead_code)]
pub fn ratio_at(result: &AspectRatioResult, idx: usize) -> Option<f32> {
    result.ratios.get(idx).copied()
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn aspect_ratio_to_json(result: &AspectRatioResult) -> String {
    format!(
        "{{\"count\":{},\"avg\":{:.6},\"max\":{:.6}}}",
        result.ratios.len(),
        result.avg_ratio,
        result.max_ratio
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn right_triangle() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }

    #[test]
    fn test_edge_length() {
        let l = edge_length([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((l - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_triangle_area() {
        let a = triangle_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((a - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_aspect_ratio_equilateral() {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.5, 0.866_025_4, 0.0];
        let r = triangle_aspect_ratio(v0, v1, v2);
        assert!(r < 1.2);
        assert!(r > 1.0);
    }

    #[test]
    fn test_degenerate_triangle() {
        let r = triangle_aspect_ratio([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert_eq!(r, f32::MAX);
    }

    #[test]
    fn test_compute_aspect_ratios() {
        let pos = right_triangle();
        let indices = vec![0, 1, 2];
        let result = compute_aspect_ratios(&pos, &indices);
        assert_eq!(result.ratios.len(), 1);
        assert!(result.avg_ratio > 0.0);
    }

    #[test]
    fn test_compute_empty() {
        let result = compute_aspect_ratios(&[], &[]);
        assert!(result.ratios.is_empty());
        assert!((result.avg_ratio).abs() < 1e-9);
    }

    #[test]
    fn test_count_poor_triangles() {
        let result = AspectRatioResult {
            ratios: vec![1.0, 5.0, 10.0],
            avg_ratio: 5.0,
            max_ratio: 10.0,
        };
        assert_eq!(count_poor_triangles(&result, 4.0), 2);
    }

    #[test]
    fn test_min_aspect_ratio() {
        let result = AspectRatioResult {
            ratios: vec![2.0, 1.5, 3.0],
            avg_ratio: 2.0,
            max_ratio: 3.0,
        };
        assert!((min_aspect_ratio(&result) - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_ratio_at() {
        let result = AspectRatioResult {
            ratios: vec![1.5],
            avg_ratio: 1.5,
            max_ratio: 1.5,
        };
        assert!(ratio_at(&result, 0).is_some());
        assert!(ratio_at(&result, 1).is_none());
    }

    #[test]
    fn test_to_json() {
        let result = AspectRatioResult {
            ratios: vec![1.5],
            avg_ratio: 1.5,
            max_ratio: 1.5,
        };
        let j = aspect_ratio_to_json(&result);
        assert!(j.contains("\"count\":1"));
    }
}
