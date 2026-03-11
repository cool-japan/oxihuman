// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Discrete Gaussian curvature via angle defect (Descartes theorem).

use std::f32::consts::TAU;

/// Per-vertex angle defect result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AngleDefectResult {
    pub defects: Vec<f32>,
    pub total_defect: f32,
    pub min_defect: f32,
    pub max_defect: f32,
}

/// Compute the interior angle at vertex `v` in the triangle (v, a, b).
#[allow(dead_code)]
pub fn triangle_interior_angle(p: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let va = [a[0] - p[0], a[1] - p[1], a[2] - p[2]];
    let vb = [b[0] - p[0], b[1] - p[1], b[2] - p[2]];
    let d = va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2];
    let la = (va[0] * va[0] + va[1] * va[1] + va[2] * va[2]).sqrt();
    let lb = (vb[0] * vb[0] + vb[1] * vb[1] + vb[2] * vb[2]).sqrt();
    if la < 1e-12 || lb < 1e-12 {
        return 0.0;
    }
    (d / (la * lb)).clamp(-1.0, 1.0).acos()
}

/// Gather the sum of interior angles at vertex `vi` across all incident triangles.
#[allow(dead_code)]
pub fn angle_sum_at_vertex(
    positions: &[[f32; 3]],
    indices: &[u32],
    vi: usize,
) -> f32 {
    let mut total = 0.0f32;
    let tc = indices.len() / 3;
    for t in 0..tc {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        if i0 == vi {
            total += triangle_interior_angle(positions[i0], positions[i1], positions[i2]);
        } else if i1 == vi {
            total += triangle_interior_angle(positions[i1], positions[i2], positions[i0]);
        } else if i2 == vi {
            total += triangle_interior_angle(positions[i2], positions[i0], positions[i1]);
        }
    }
    total
}

/// Compute the angle defect at one vertex: 2*pi - sum_of_angles.
#[allow(dead_code)]
pub fn angle_defect_at(
    positions: &[[f32; 3]],
    indices: &[u32],
    vi: usize,
) -> f32 {
    TAU - angle_sum_at_vertex(positions, indices, vi)
}

/// Check if a vertex is on a boundary (has an edge shared by only one triangle).
#[allow(dead_code)]
pub fn is_boundary_vertex(indices: &[u32], vertex: usize) -> bool {
    use std::collections::HashMap;
    let tc = indices.len() / 3;
    let mut edge_count: HashMap<(usize, usize), u32> = HashMap::new();
    for t in 0..tc {
        let tri = [
            indices[t * 3] as usize,
            indices[t * 3 + 1] as usize,
            indices[t * 3 + 2] as usize,
        ];
        if !tri.contains(&vertex) {
            continue;
        }
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count.values().any(|&c| c == 1)
}

/// Compute angle defect for all vertices.
#[allow(dead_code)]
pub fn compute_angle_defects(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> AngleDefectResult {
    let n = positions.len();
    let mut defects = vec![0.0f32; n];
    for i in 0..n {
        defects[i] = angle_defect_at(positions, indices, i);
    }
    let total = defects.iter().sum::<f32>();
    let min_d = defects.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_d = defects.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    AngleDefectResult {
        defects,
        total_defect: total,
        min_defect: if min_d.is_infinite() { 0.0 } else { min_d },
        max_defect: if max_d.is_infinite() { 0.0 } else { max_d },
    }
}

/// Total Gaussian curvature via Gauss-Bonnet: should equal 2*pi*chi for closed surfaces.
#[allow(dead_code)]
pub fn total_gaussian_curvature(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    let r = compute_angle_defects(positions, indices);
    r.total_defect
}

/// Euler characteristic estimate from angle defect: chi = total_defect / (2*pi).
#[allow(dead_code)]
pub fn euler_characteristic_estimate(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    total_gaussian_curvature(positions, indices) / TAU
}

/// Average angle defect per vertex.
#[allow(dead_code)]
pub fn average_angle_defect(result: &AngleDefectResult) -> f32 {
    if result.defects.is_empty() {
        return 0.0;
    }
    result.total_defect / result.defects.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn equilateral_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 0.866_025_4, 0.0],
        ];
        let indices = vec![0, 1, 2];
        (positions, indices)
    }

    fn tetrahedron() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [1.0, 1.0, 1.0],
            [-1.0, -1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [1.0, -1.0, -1.0],
        ];
        let indices = vec![0, 1, 2, 0, 3, 1, 0, 2, 3, 1, 3, 2];
        (positions, indices)
    }

    #[test]
    fn test_interior_angle_right() {
        let angle = triangle_interior_angle([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((angle - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_angle_sum_equilateral() {
        let (pos, idx) = equilateral_tri();
        let sum = angle_sum_at_vertex(&pos, &idx, 0);
        assert!(sum > 0.0);
        assert!(sum < TAU);
    }

    #[test]
    fn test_angle_defect_single_triangle() {
        let (pos, idx) = equilateral_tri();
        let d = angle_defect_at(&pos, &idx, 0);
        // Only one triangle, so defect = 2*pi - angle_at_vertex
        assert!(d > 0.0);
    }

    #[test]
    fn test_compute_all_defects() {
        let (pos, idx) = equilateral_tri();
        let r = compute_angle_defects(&pos, &idx);
        assert_eq!(r.defects.len(), 3);
    }

    #[test]
    fn test_tetrahedron_curvature() {
        let (pos, idx) = tetrahedron();
        let total = total_gaussian_curvature(&pos, &idx);
        // Closed surface: should be ~ 4*pi
        assert!((total - 4.0 * PI).abs() < 0.5);
    }

    #[test]
    fn test_euler_estimate_tetrahedron() {
        let (pos, idx) = tetrahedron();
        let chi = euler_characteristic_estimate(&pos, &idx);
        // chi(sphere) = 2
        assert!((chi - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_average_defect() {
        let (pos, idx) = equilateral_tri();
        let r = compute_angle_defects(&pos, &idx);
        let avg = average_angle_defect(&r);
        assert!(avg > 0.0);
    }

    #[test]
    fn test_boundary_vertex() {
        let (_, idx) = equilateral_tri();
        assert!(is_boundary_vertex(&idx, 0));
    }

    #[test]
    fn test_min_max_defect() {
        let (pos, idx) = equilateral_tri();
        let r = compute_angle_defects(&pos, &idx);
        assert!(r.min_defect <= r.max_defect);
    }

    #[test]
    fn test_zero_angle_degenerate() {
        let angle = triangle_interior_angle([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((angle - 0.0).abs() < 1e-5);
    }
}
