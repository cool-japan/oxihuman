// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Incircle (inscribed circle) computation for triangle mesh faces.

use std::f32::consts::PI;

/// Incircle data for a triangle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Incircle {
    pub center: [f32; 3],
    pub radius: f32,
}

/// Result of incircle analysis for a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IncircleResult {
    pub incircles: Vec<Incircle>,
    pub avg_radius: f32,
    pub min_radius: f32,
    pub max_radius: f32,
}

/// Edge length between two 3D points.
#[allow(dead_code)]
pub fn edge_length_ic(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Compute the incircle of a triangle (center + radius).
/// The incircle center is the weighted average of vertices by opposite edge length.
#[allow(dead_code)]
pub fn triangle_incircle(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Incircle {
    let la = edge_length_ic(b, c);
    let lb = edge_length_ic(a, c);
    let lc = edge_length_ic(a, b);
    let perimeter = la + lb + lc;
    if perimeter < 1e-12 {
        return Incircle {
            center: a,
            radius: 0.0,
        };
    }
    let cx = (la * a[0] + lb * b[0] + lc * c[0]) / perimeter;
    let cy = (la * a[1] + lb * b[1] + lc * c[1]) / perimeter;
    let cz = (la * a[2] + lb * b[2] + lc * c[2]) / perimeter;
    // radius = area / semi-perimeter
    let s = perimeter / 2.0;
    let ea = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let eb = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ea[1] * eb[2] - ea[2] * eb[1],
        ea[2] * eb[0] - ea[0] * eb[2],
        ea[0] * eb[1] - ea[1] * eb[0],
    ];
    let area = 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    let radius = area / s;
    Incircle {
        center: [cx, cy, cz],
        radius,
    }
}

/// Compute incircles for all faces of a triangle mesh.
#[allow(dead_code)]
pub fn compute_incircles(positions: &[[f32; 3]], indices: &[u32]) -> IncircleResult {
    let face_count = indices.len() / 3;
    let mut incircles = Vec::with_capacity(face_count);
    for f in 0..face_count {
        let i0 = indices[f * 3] as usize;
        let i1 = indices[f * 3 + 1] as usize;
        let i2 = indices[f * 3 + 2] as usize;
        incircles.push(triangle_incircle(
            positions[i0],
            positions[i1],
            positions[i2],
        ));
    }
    let avg_radius = if !incircles.is_empty() {
        incircles.iter().map(|ic| ic.radius).sum::<f32>() / incircles.len() as f32
    } else {
        0.0
    };
    let min_radius = incircles
        .iter()
        .map(|ic| ic.radius)
        .fold(f32::MAX, f32::min);
    let max_radius = incircles.iter().map(|ic| ic.radius).fold(0.0_f32, f32::max);
    let min_radius = if incircles.is_empty() {
        0.0
    } else {
        min_radius
    };
    IncircleResult {
        incircles,
        avg_radius,
        min_radius,
        max_radius,
    }
}

/// Count incircles with radius above a threshold.
#[allow(dead_code)]
pub fn count_large_incircles(result: &IncircleResult, threshold: f32) -> usize {
    result
        .incircles
        .iter()
        .filter(|ic| ic.radius > threshold)
        .count()
}

/// Get incircle at index.
#[allow(dead_code)]
pub fn get_incircle(result: &IncircleResult, idx: usize) -> Option<&Incircle> {
    result.incircles.get(idx)
}

/// Export to JSON.
#[allow(dead_code)]
pub fn incircle_result_to_json(result: &IncircleResult) -> String {
    format!(
        "{{\"face_count\":{},\"avg_radius\":{:.6},\"pi\":{:.6}}}",
        result.incircles.len(),
        result.avg_radius,
        PI
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_right_tri() -> ([f32; 3], [f32; 3], [f32; 3]) {
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0])
    }

    #[test]
    fn test_edge_length_ic() {
        let l = edge_length_ic([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((l - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_triangle_incircle_right_tri() {
        let (a, b, c) = unit_right_tri();
        let ic = triangle_incircle(a, b, c);
        // r = (a + b - hyp) / 2 = (1 + 1 - sqrt2) / 2
        let expected = (2.0 - 2.0f32.sqrt()) / 2.0;
        assert!((ic.radius - expected).abs() < 1e-4, "radius: {}", ic.radius);
    }

    #[test]
    fn test_triangle_incircle_degenerate() {
        let ic = triangle_incircle([0.0; 3], [0.0; 3], [0.0; 3]);
        assert!((ic.radius).abs() < 1e-9);
    }

    #[test]
    fn test_compute_incircles_empty() {
        let r = compute_incircles(&[], &[]);
        assert_eq!(r.incircles.len(), 0);
        assert!((r.avg_radius).abs() < 1e-9);
    }

    #[test]
    fn test_compute_incircles_single() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let r = compute_incircles(&pos, &idx);
        assert_eq!(r.incircles.len(), 1);
        assert!(r.avg_radius > 0.0);
    }

    #[test]
    fn test_count_large_incircles() {
        let pos = vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let r = compute_incircles(&pos, &idx);
        assert_eq!(count_large_incircles(&r, 0.01), 1);
    }

    #[test]
    fn test_get_incircle() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let r = compute_incircles(&pos, &idx);
        assert!(get_incircle(&r, 0).is_some());
        assert!(get_incircle(&r, 99).is_none());
    }

    #[test]
    fn test_incircle_result_to_json() {
        let r = compute_incircles(
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[0u32, 1, 2],
        );
        let j = incircle_result_to_json(&r);
        assert!(j.contains("\"face_count\":1"));
    }

    #[test]
    fn test_radius_in_range() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let r = compute_incircles(&pos, &idx);
        assert!((0.0..=1.0).contains(&r.avg_radius));
    }

    #[test]
    fn test_min_max_radius_single() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let r = compute_incircles(&pos, &idx);
        assert!((r.min_radius - r.max_radius).abs() < 1e-9);
    }
}
