// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Bi-Laplacian (bi-harmonic) mesh smoothing and fairing.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

// ---------------------------------------------------------------------------
// Cotangent weight helpers
// ---------------------------------------------------------------------------

/// Cotangent of the angle at vertex `c` in the triangle (a, b, c),
/// i.e. cot of the angle opposite edge a-b.
pub fn cotangent_weight(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    // Vectors from c to a and c to b
    let ca = sub(a, c);
    let cb = sub(b, c);
    let cos_theta = dot(ca, cb);
    let sin_theta = len([
        ca[1] * cb[2] - ca[2] * cb[1],
        ca[2] * cb[0] - ca[0] * cb[2],
        ca[0] * cb[1] - ca[1] * cb[0],
    ]);
    if sin_theta.abs() < 1e-10 {
        0.0
    } else {
        cos_theta / sin_theta
    }
}

// ---------------------------------------------------------------------------
// Laplacian weight matrix
// ---------------------------------------------------------------------------

/// Build cotangent weight matrix as adjacency list.
/// Returns `weights[i]` = list of `(j, w_ij)` for each vertex `i`.
pub fn build_laplacian_weights(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
) -> Vec<Vec<(usize, f32)>> {
    let n = positions.len();
    let mut weights: Vec<Vec<(usize, f32)>> = vec![Vec::new(); n];
    // Accumulate cotangent weights per edge
    let mut edge_weights: std::collections::HashMap<(usize, usize), f32> =
        std::collections::HashMap::new();

    for &tri in tris {
        let [ai, bi, ci] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if ai >= n || bi >= n || ci >= n {
            continue;
        }
        let pa = positions[ai];
        let pb = positions[bi];
        let pc = positions[ci];

        // cot at c (for edge a-b), cot at a (for edge b-c), cot at b (for edge c-a)
        let cot_c = cotangent_weight(pa, pb, pc);
        let cot_a = cotangent_weight(pb, pc, pa);
        let cot_b = cotangent_weight(pc, pa, pb);

        // Edge ab: weight = 0.5 * (cot_c from this tri + cot from opposite tri)
        for (p, q, w) in [(ai, bi, cot_c), (bi, ci, cot_a), (ci, ai, cot_b)] {
            let key = (p.min(q), p.max(q));
            *edge_weights.entry(key).or_insert(0.0) += 0.5 * w;
        }
    }

    for ((i, j), w) in &edge_weights {
        if w.abs() > 1e-12 {
            weights[*i].push((*j, *w));
            weights[*j].push((*i, *w));
        }
    }

    weights
}

// ---------------------------------------------------------------------------
// Laplacian operator
// ---------------------------------------------------------------------------

/// Apply Laplacian operator once: L(p)_i = sum_j w_ij (p_j - p_i) / sum_j w_ij
pub fn laplacian_operator(positions: &[[f32; 3]], weights: &[Vec<(usize, f32)>]) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut result = vec![[0.0f32; 3]; n];
    for (i, nbrs) in weights.iter().enumerate() {
        if i >= n || nbrs.is_empty() {
            continue;
        }
        let total_w: f32 = nbrs.iter().map(|&(_, w)| w).sum();
        if total_w < 1e-10 {
            continue;
        }
        let mut lap = [0.0f32; 3];
        for &(j, w) in nbrs {
            if j < n {
                let d = sub(positions[j], positions[i]);
                lap = add(lap, scale(d, w));
            }
        }
        result[i] = scale(lap, 1.0 / total_w);
    }
    result
}

// ---------------------------------------------------------------------------
// Bi-Laplacian smoothing
// ---------------------------------------------------------------------------

/// Bi-Laplacian smoothing: new_pos = pos - lambda * L(L(pos)).
pub fn bilaplacian_smooth(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    lambda: f32,
    iters: usize,
) -> Vec<[f32; 3]> {
    let weights = build_laplacian_weights(positions, tris);
    let mut pos = positions.to_vec();

    for _ in 0..iters {
        let lap1 = laplacian_operator(&pos, &weights);
        // Interpret lap1 as "displaced" positions for second application
        let lap1_as_pos: Vec<[f32; 3]> = pos
            .iter()
            .zip(lap1.iter())
            .map(|(&p, &l)| add(p, l))
            .collect();
        let lap2_of_lap1 = laplacian_operator(&lap1_as_pos, &weights);
        // Subtract the bi-Laplacian contribution
        pos = pos
            .iter()
            .zip(lap2_of_lap1.iter())
            .map(|(&p, &l2)| sub(p, scale(l2, lambda)))
            .collect();
    }
    pos
}

// ---------------------------------------------------------------------------
// Bi-Laplacian fairing (Taubin-style)
// ---------------------------------------------------------------------------

/// Taubin-style alternating λ/μ smoothing to avoid volume shrinkage.
pub fn bilaplacian_fairing(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    iterations: usize,
    lambda: f32,
    mu: f32,
) -> Vec<[f32; 3]> {
    let weights = build_laplacian_weights(positions, tris);
    let mut pos = positions.to_vec();

    for iter_idx in 0..iterations {
        let step = if iter_idx % 2 == 0 { lambda } else { mu };
        let lap = laplacian_operator(&pos, &weights);
        pos = pos
            .iter()
            .zip(lap.iter())
            .map(|(&p, &l)| add(p, scale(l, step)))
            .collect();
    }
    pos
}

// ---------------------------------------------------------------------------
// Energy
// ---------------------------------------------------------------------------

/// Sum of squared Laplacian magnitudes.
pub fn mesh_laplacian_energy(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> f32 {
    let weights = build_laplacian_weights(positions, tris);
    let lap = laplacian_operator(positions, &weights);
    lap.iter().map(|&l| dot(l, l)).sum()
}

/// Sum of squared bi-Laplacian magnitudes.
pub fn bilaplacian_energy(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> f32 {
    let weights = build_laplacian_weights(positions, tris);
    let lap1 = laplacian_operator(positions, &weights);
    let lap1_as_pos: Vec<[f32; 3]> = positions
        .iter()
        .zip(lap1.iter())
        .map(|(&p, &l)| add(p, l))
        .collect();
    let lap2 = laplacian_operator(&lap1_as_pos, &weights);
    lap2.iter().map(|&l| dot(l, l)).sum()
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn equilateral_tri() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        // Equilateral triangle with side length 1
        let s3 = 3.0_f32.sqrt() / 2.0;
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, s3, 0.0]];
        let tris = vec![[0, 1, 2]];
        (pos, tris)
    }

    fn flat_grid(n: usize) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let mut pos = Vec::new();
        let mut tris = Vec::new();
        let step = 1.0 / n as f32;
        for i in 0..=n {
            for j in 0..=n {
                pos.push([i as f32 * step, j as f32 * step, 0.0]);
            }
        }
        let stride = n + 1;
        for i in 0..n {
            for j in 0..n {
                let a = (i * stride + j) as u32;
                let b = (i * stride + j + 1) as u32;
                let c = ((i + 1) * stride + j) as u32;
                let d = ((i + 1) * stride + j + 1) as u32;
                tris.push([a, b, c]);
                tris.push([b, d, c]);
            }
        }
        (pos, tris)
    }

    fn bumpy_grid(n: usize) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let mut pos = Vec::new();
        let mut tris = Vec::new();
        let step = 1.0 / n as f32;
        for i in 0..=n {
            for j in 0..=n {
                let x = i as f32 * step;
                let y = j as f32 * step;
                let z = (x * std::f32::consts::PI * 2.0).sin()
                    * (y * std::f32::consts::PI * 2.0).cos()
                    * 0.3;
                pos.push([x, y, z]);
            }
        }
        let stride = n + 1;
        for i in 0..n {
            for j in 0..n {
                let a = (i * stride + j) as u32;
                let b = (i * stride + j + 1) as u32;
                let c = ((i + 1) * stride + j) as u32;
                let d = ((i + 1) * stride + j + 1) as u32;
                tris.push([a, b, c]);
                tris.push([b, d, c]);
            }
        }
        (pos, tris)
    }

    #[test]
    fn test_cotangent_weight_equilateral() {
        // In an equilateral triangle, all angles are 60°, cot(60°) = 1/sqrt(3) ≈ 0.5774
        let s3 = 3.0_f32.sqrt() / 2.0;
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0f32, 0.0, 0.0];
        let c = [0.5f32, s3, 0.0];
        let w = cotangent_weight(a, b, c); // angle at c
        let expected = 1.0 / 3.0_f32.sqrt();
        assert!(
            (w - expected).abs() < 1e-4,
            "cot(60°)={w}, expected={expected}"
        );
    }

    #[test]
    fn test_cotangent_weight_right_angle() {
        // Right angle at c: cot(90°) = 0
        let a = [0.0f32, 0.0, 0.0];
        let b = [0.0f32, 1.0, 0.0];
        let c = [0.0f32, 0.0, 0.0]; // degenerate, use different
                                    // angle at origin in right triangle
        let a2 = [1.0f32, 0.0, 0.0];
        let b2 = [0.0f32, 1.0, 0.0];
        let c2 = [0.0f32, 0.0, 0.0];
        let w = cotangent_weight(a2, b2, c2);
        // angle at c2 = 90°, cot(90°) = 0
        assert!(
            w.abs() < 1e-4,
            "cot(90°)={w}, expected ~0, a={a:?} b={b:?} c={c:?}"
        );
    }

    #[test]
    fn test_build_laplacian_weights_nonzero() {
        let (pos, tris) = equilateral_tri();
        let w = build_laplacian_weights(&pos, &tris);
        // Each vertex should have neighbors
        let has_weights = w.iter().any(|v| !v.is_empty());
        assert!(has_weights, "Laplacian weights should not all be empty");
    }

    #[test]
    fn test_laplacian_operator_flat_mesh() {
        // For a flat mesh, the Laplacian should be close to 0 for interior vertices
        let (pos, tris) = flat_grid(4);
        let weights = build_laplacian_weights(&pos, &tris);
        let lap = laplacian_operator(&pos, &weights);
        // All vertices in a flat mesh: Laplacian should be near 0 in z
        for l in &lap {
            assert!(
                l[2].abs() < 1e-5,
                "z Laplacian should be 0 for flat mesh, got {}",
                l[2]
            );
        }
    }

    #[test]
    fn test_laplacian_operator_no_nan() {
        let (pos, tris) = flat_grid(3);
        let weights = build_laplacian_weights(&pos, &tris);
        let lap = laplacian_operator(&pos, &weights);
        for l in &lap {
            assert!(l[0].is_finite() && l[1].is_finite() && l[2].is_finite());
        }
    }

    #[test]
    fn test_bilaplacian_smooth_reduces_energy() {
        let (pos, tris) = bumpy_grid(4);
        let before = mesh_laplacian_energy(&pos, &tris);
        let smoothed = bilaplacian_smooth(&pos, &tris, 0.1, 5);
        let after = mesh_laplacian_energy(&smoothed, &tris);
        assert!(
            after <= before + 1e-3,
            "smoothing should reduce energy: before={before} after={after}"
        );
    }

    #[test]
    fn test_bilaplacian_smooth_no_nan() {
        let (pos, tris) = bumpy_grid(4);
        let smoothed = bilaplacian_smooth(&pos, &tris, 0.1, 3);
        for p in &smoothed {
            assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        }
    }

    #[test]
    fn test_bilaplacian_fairing_differs_from_smooth() {
        let (pos, tris) = bumpy_grid(4);
        let smoothed = bilaplacian_smooth(&pos, &tris, 0.1, 4);
        let faired = bilaplacian_fairing(&pos, &tris, 4, 0.1, -0.105);
        // They should produce different results
        let same = smoothed
            .iter()
            .zip(faired.iter())
            .all(|(a, b)| (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs() < 1e-6);
        assert!(!same, "fairing and smooth should produce different results");
    }

    #[test]
    fn test_bilaplacian_fairing_no_nan() {
        let (pos, tris) = bumpy_grid(4);
        let faired = bilaplacian_fairing(&pos, &tris, 6, 0.1, -0.105);
        for p in &faired {
            assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        }
    }

    #[test]
    fn test_mesh_laplacian_energy_positive_bumpy() {
        let (pos, tris) = bumpy_grid(4);
        let energy = mesh_laplacian_energy(&pos, &tris);
        assert!(
            energy > 0.0,
            "energy of bumpy mesh should be positive, got {energy}"
        );
    }

    #[test]
    fn test_mesh_laplacian_energy_flat_less_than_bumpy() {
        // A flat mesh should have strictly less Laplacian energy than a bumpy one
        let (flat_pos, flat_tris) = flat_grid(4);
        let (bumpy_pos, bumpy_tris) = bumpy_grid(4);
        let flat_energy = mesh_laplacian_energy(&flat_pos, &flat_tris);
        let bumpy_energy = mesh_laplacian_energy(&bumpy_pos, &bumpy_tris);
        assert!(
            flat_energy < bumpy_energy,
            "flat energy={flat_energy} should be < bumpy energy={bumpy_energy}"
        );
    }

    #[test]
    fn test_bilaplacian_energy_positive() {
        let (pos, tris) = bumpy_grid(4);
        let energy = bilaplacian_energy(&pos, &tris);
        assert!(energy >= 0.0, "bi-Laplacian energy should be non-negative");
    }

    #[test]
    fn test_bilaplacian_energy_finite() {
        let (pos, tris) = bumpy_grid(4);
        let energy = bilaplacian_energy(&pos, &tris);
        assert!(energy.is_finite(), "bi-Laplacian energy should be finite");
    }
}
