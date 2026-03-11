// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Scatter points on mesh surface using area-weighted per-face sampling.
//! Uses deterministic LCG pseudo-random number generation (no rand crate).

/// A point scattered on the mesh surface.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScatteredPoint {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub face_index: usize,
}

/// LCG state for deterministic pseudo-random sampling.
#[allow(dead_code)]
pub struct LcgRng {
    state: u64,
}

impl LcgRng {
    #[allow(dead_code)]
    pub fn new(seed: u64) -> Self {
        LcgRng {
            state: seed.wrapping_add(1),
        }
    }

    #[allow(dead_code)]
    pub fn next_f32(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 11) as f32 / (1u64 << 53) as f32
    }
}

/// Scatter `count` points uniformly on the mesh surface (area-weighted).
#[allow(dead_code)]
pub fn scatter_points(
    positions: &[[f32; 3]],
    indices: &[u32],
    count: usize,
    seed: u64,
) -> Vec<ScatteredPoint> {
    if indices.len() < 3 || count == 0 {
        return vec![];
    }
    let face_count = indices.len() / 3;
    let areas: Vec<f32> = (0..face_count)
        .map(|fi| {
            let a = positions[indices[fi * 3] as usize];
            let b = positions[indices[fi * 3 + 1] as usize];
            let c = positions[indices[fi * 3 + 2] as usize];
            triangle_area(a, b, c)
        })
        .collect();
    let total_area: f32 = areas.iter().sum();
    if total_area < 1e-12 {
        return vec![];
    }
    let cumulative: Vec<f32> = {
        let mut cum = 0.0f32;
        areas
            .iter()
            .map(|&a| {
                cum += a;
                cum
            })
            .collect()
    };
    let mut rng = LcgRng::new(seed);
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        let r = rng.next_f32() * total_area;
        let fi = cumulative.partition_point(|&c| c < r).min(face_count - 1);
        let ia = indices[fi * 3] as usize;
        let ib = indices[fi * 3 + 1] as usize;
        let ic = indices[fi * 3 + 2] as usize;
        let pa = positions[ia];
        let pb = positions[ib];
        let pc = positions[ic];
        let u = rng.next_f32();
        let v = rng.next_f32();
        let (su, sv) = if u + v > 1.0 {
            (1.0 - u, 1.0 - v)
        } else {
            (u, v)
        };
        let pos = [
            pa[0] + su * (pb[0] - pa[0]) + sv * (pc[0] - pa[0]),
            pa[1] + su * (pb[1] - pa[1]) + sv * (pc[1] - pa[1]),
            pa[2] + su * (pb[2] - pa[2]) + sv * (pc[2] - pa[2]),
        ];
        let normal = face_normal(pa, pb, pc);
        result.push(ScatteredPoint {
            position: pos,
            normal,
            face_index: fi,
        });
    }
    result
}

/// Total surface area of the mesh.
#[allow(dead_code)]
pub fn total_area(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    indices
        .chunks_exact(3)
        .map(|tri| {
            triangle_area(
                positions[tri[0] as usize],
                positions[tri[1] as usize],
                positions[tri[2] as usize],
            )
        })
        .sum()
}

/// Area of a triangle given its three vertices.
#[allow(dead_code)]
pub fn triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cx = ab[1] * ac[2] - ab[2] * ac[1];
    let cy = ab[2] * ac[0] - ab[0] * ac[2];
    let cz = ab[0] * ac[1] - ab[1] * ac[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if l < 1e-8 {
        [0.0, 1.0, 0.0]
    } else {
        [n[0] / l, n[1] / l, n[2] / l]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_square_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn scatter_correct_count() {
        let (pos, idx) = unit_square_mesh();
        let pts = scatter_points(&pos, &idx, 100, 42);
        assert_eq!(pts.len(), 100);
    }

    #[test]
    fn scatter_zero_count() {
        let (pos, idx) = unit_square_mesh();
        let pts = scatter_points(&pos, &idx, 0, 42);
        assert!(pts.is_empty());
    }

    #[test]
    fn scatter_empty_mesh() {
        let pts = scatter_points(&[], &[], 10, 42);
        assert!(pts.is_empty());
    }

    #[test]
    fn scattered_face_indices_valid() {
        let (pos, idx) = unit_square_mesh();
        let pts = scatter_points(&pos, &idx, 50, 7);
        let face_count = idx.len() / 3;
        for pt in &pts {
            assert!(pt.face_index < face_count);
        }
    }

    #[test]
    fn scattered_normals_unit() {
        let (pos, idx) = unit_square_mesh();
        let pts = scatter_points(&pos, &idx, 20, 99);
        for pt in &pts {
            let l = (pt.normal[0] * pt.normal[0]
                + pt.normal[1] * pt.normal[1]
                + pt.normal[2] * pt.normal[2])
                .sqrt();
            assert!((l - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn total_area_unit_square() {
        let (pos, idx) = unit_square_mesh();
        let area = total_area(&pos, &idx);
        assert!((area - 1.0).abs() < 1e-5);
    }

    #[test]
    fn triangle_area_right_triangle() {
        let area = triangle_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((area - 0.5).abs() < 1e-6);
    }

    #[test]
    fn lcg_rng_deterministic() {
        let mut r1 = LcgRng::new(123);
        let mut r2 = LcgRng::new(123);
        for _ in 0..10 {
            assert!((r1.next_f32() - r2.next_f32()).abs() < 1e-9);
        }
    }

    #[test]
    fn lcg_rng_in_range() {
        let mut rng = LcgRng::new(55);
        for _ in 0..100 {
            let v = rng.next_f32();
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn scatter_positions_near_mesh() {
        let (pos, idx) = unit_square_mesh();
        let pts = scatter_points(&pos, &idx, 30, 1);
        for pt in &pts {
            assert!(pt.position[0] >= -0.01 && pt.position[0] <= 1.01);
            assert!(pt.position[2] >= -0.01 && pt.position[2] <= 1.01);
        }
    }
}
