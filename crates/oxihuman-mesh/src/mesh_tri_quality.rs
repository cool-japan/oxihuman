#![allow(dead_code)]
//! Triangle quality metrics.

use std::f32::consts::PI;

/// Triangle quality result.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TriQuality {
    pub aspect_ratio: f32,
    pub min_angle: f32,
    pub max_angle: f32,
    pub quality_score: f32,
}

fn edge_len(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn angle_between(a: f32, b: f32, c: f32) -> f32 {
    // angle opposite to side c, using law of cosines
    let cos_val = (a * a + b * b - c * c) / (2.0 * a * b);
    cos_val.clamp(-1.0, 1.0).acos()
}

/// Compute aspect ratio of a triangle (longest / shortest edge).
#[allow(dead_code)]
pub fn triangle_aspect_ratio(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = edge_len(a, b);
    let bc = edge_len(b, c);
    let ca = edge_len(c, a);
    let max_e = ab.max(bc).max(ca);
    let min_e = ab.min(bc).min(ca);
    if min_e < 1e-12 {
        return f32::MAX;
    }
    max_e / min_e
}

/// Compute skewness of a triangle (deviation from equilateral).
#[allow(dead_code)]
pub fn triangle_skewness(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ideal = PI / 3.0; // 60 degrees
    let ab = edge_len(a, b);
    let bc = edge_len(b, c);
    let ca = edge_len(c, a);
    if ab < 1e-12 || bc < 1e-12 || ca < 1e-12 {
        return 1.0;
    }
    let a1 = angle_between(ab, ca, bc);
    let a2 = angle_between(ab, bc, ca);
    let a3 = angle_between(bc, ca, ab);
    let max_dev = (a1 - ideal).abs().max((a2 - ideal).abs()).max((a3 - ideal).abs());
    max_dev / ideal
}

/// Compute minimum angle of a triangle (radians).
#[allow(dead_code)]
pub fn triangle_min_angle(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = edge_len(a, b);
    let bc = edge_len(b, c);
    let ca = edge_len(c, a);
    if ab < 1e-12 || bc < 1e-12 || ca < 1e-12 {
        return 0.0;
    }
    let a1 = angle_between(ab, ca, bc);
    let a2 = angle_between(ab, bc, ca);
    let a3 = angle_between(bc, ca, ab);
    a1.min(a2).min(a3)
}

/// Compute maximum angle of a triangle (radians).
#[allow(dead_code)]
pub fn triangle_max_angle(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = edge_len(a, b);
    let bc = edge_len(b, c);
    let ca = edge_len(c, a);
    if ab < 1e-12 || bc < 1e-12 || ca < 1e-12 {
        return PI;
    }
    let a1 = angle_between(ab, ca, bc);
    let a2 = angle_between(ab, bc, ca);
    let a3 = angle_between(bc, ca, ab);
    a1.max(a2).max(a3)
}

/// Quality score: ratio of min angle to ideal (60 degrees). 1.0 = perfect.
#[allow(dead_code)]
pub fn triangle_quality_score(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let min_a = triangle_min_angle(a, b, c);
    let ideal = PI / 3.0;
    (min_a / ideal).min(1.0)
}

/// Average quality score across all mesh triangles.
#[allow(dead_code)]
pub fn mesh_avg_quality(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    if indices.is_empty() {
        return 0.0;
    }
    let total: f32 = indices
        .iter()
        .map(|tri| {
            triangle_quality_score(
                positions[tri[0] as usize],
                positions[tri[1] as usize],
                positions[tri[2] as usize],
            )
        })
        .sum();
    total / indices.len() as f32
}

/// Index of the worst quality triangle.
#[allow(dead_code)]
pub fn worst_triangle_index(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Option<usize> {
    if indices.is_empty() {
        return None;
    }
    let mut worst = 0;
    let mut worst_score = f32::MAX;
    for (idx, tri) in indices.iter().enumerate() {
        let score = triangle_quality_score(
            positions[tri[0] as usize],
            positions[tri[1] as usize],
            positions[tri[2] as usize],
        );
        if score < worst_score {
            worst_score = score;
            worst = idx;
        }
    }
    Some(worst)
}

/// Quality histogram: bin quality scores into `bin_count` bins.
#[allow(dead_code)]
pub fn quality_histogram(positions: &[[f32; 3]], indices: &[[u32; 3]], bin_count: usize) -> Vec<u32> {
    if bin_count == 0 {
        return vec![];
    }
    let mut bins = vec![0u32; bin_count];
    for tri in indices {
        let score = triangle_quality_score(
            positions[tri[0] as usize],
            positions[tri[1] as usize],
            positions[tri[2] as usize],
        );
        let idx = (score * (bin_count as f32 - 1.0)).round() as usize;
        let idx = idx.min(bin_count - 1);
        bins[idx] += 1;
    }
    bins
}

#[cfg(test)]
mod tests {
    use super::*;

    fn equilateral() -> ([f32; 3], [f32; 3], [f32; 3]) {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.5, (3.0f32).sqrt() / 2.0, 0.0];
        (a, b, c)
    }

    #[test]
    fn test_aspect_ratio_equilateral() {
        let (a, b, c) = equilateral();
        let ar = triangle_aspect_ratio(a, b, c);
        assert!((ar - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_skewness_equilateral() {
        let (a, b, c) = equilateral();
        let s = triangle_skewness(a, b, c);
        assert!(s.abs() < 0.01);
    }

    #[test]
    fn test_min_angle_equilateral() {
        let (a, b, c) = equilateral();
        let min_a = triangle_min_angle(a, b, c);
        assert!((min_a - PI / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_max_angle_equilateral() {
        let (a, b, c) = equilateral();
        let max_a = triangle_max_angle(a, b, c);
        assert!((max_a - PI / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_quality_score_equilateral() {
        let (a, b, c) = equilateral();
        let q = triangle_quality_score(a, b, c);
        assert!((q - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_mesh_avg_quality() {
        let (a, b, c) = equilateral();
        let p = vec![a, b, c];
        let i = vec![[0u32, 1, 2]];
        let avg = mesh_avg_quality(&p, &i);
        assert!((avg - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_worst_triangle_index() {
        let p = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.866, 0.0],
            [0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.01, 0.01, 0.0],
        ];
        let i = vec![[0u32, 1, 2], [3, 4, 5]];
        let worst = worst_triangle_index(&p, &i);
        assert_eq!(worst, Some(1));
    }

    #[test]
    fn test_worst_triangle_index_empty() {
        assert_eq!(worst_triangle_index(&[], &[]), None);
    }

    #[test]
    fn test_quality_histogram() {
        let (a, b, c) = equilateral();
        let p = vec![a, b, c];
        let i = vec![[0u32, 1, 2]];
        let h = quality_histogram(&p, &i, 4);
        assert_eq!(h.len(), 4);
    }

    #[test]
    fn test_degenerate_triangle() {
        let ar = triangle_aspect_ratio([0.0; 3], [0.0; 3], [0.0; 3]);
        assert_eq!(ar, f32::MAX);
    }
}
