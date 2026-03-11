// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Volume-preserving morphs using Jacobian correction.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VolumeMorphConfig {
    pub preservation_strength: f32,
    pub iterations: u32,
    pub smooth_correction: bool,
}

impl Default for VolumeMorphConfig {
    fn default() -> Self {
        Self {
            preservation_strength: 0.5,
            iterations: 3,
            smooth_correction: true,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VolumeMorphResult {
    pub corrected_deltas: Vec<[f32; 3]>,
    pub original_volume: f32,
    pub morphed_volume: f32,
    pub corrected_volume: f32,
    pub volume_error_pct: f32,
}

/// Signed volume via divergence theorem (triangle mesh).
#[allow(dead_code)]
pub fn compute_mesh_volume(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    let mut volume = 0.0f32;
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        let v0 = positions[i0];
        let v1 = positions[i1];
        let v2 = positions[i2];
        // Signed volume contribution: (v0 · (v1 × v2)) / 6
        let cross_x = v1[1] * v2[2] - v1[2] * v2[1];
        let cross_y = v1[2] * v2[0] - v1[0] * v2[2];
        let cross_z = v1[0] * v2[1] - v1[1] * v2[0];
        volume += v0[0] * cross_x + v0[1] * cross_y + v0[2] * cross_z;
    }
    volume / 6.0
}

#[allow(dead_code)]
pub fn volume_preserving_delta(
    base: &[[f32; 3]],
    indices: &[u32],
    deltas: &[[f32; 3]],
    cfg: &VolumeMorphConfig,
) -> VolumeMorphResult {
    let n = base.len();

    // Apply deltas to get morphed positions
    let morphed: Vec<[f32; 3]> = base
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let d = if i < deltas.len() {
                deltas[i]
            } else {
                [0.0; 3]
            };
            [b[0] + d[0], b[1] + d[1], b[2] + d[2]]
        })
        .collect();

    let original_volume = compute_mesh_volume(base, indices);
    let morphed_volume = compute_mesh_volume(&morphed, indices);

    // Compute scale factor
    let ratio = mesh_volume_ratio(base, &morphed, indices);
    // Blend with preservation_strength
    let scale = 1.0 + (ratio - 1.0) * cfg.preservation_strength;

    let mut corrected_deltas: Vec<[f32; 3]> = uniform_scale_correction(base, deltas, scale);

    if cfg.smooth_correction {
        corrected_deltas = laplacian_smooth_deltas(&corrected_deltas, indices, cfg.iterations);
    }

    // Recompute corrected volume
    let corrected_positions: Vec<[f32; 3]> = base
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let d = if i < n { corrected_deltas[i] } else { [0.0; 3] };
            [b[0] + d[0], b[1] + d[1], b[2] + d[2]]
        })
        .collect();
    let corrected_volume = compute_mesh_volume(&corrected_positions, indices);
    let volume_error_pct = volume_error_percent(original_volume, corrected_volume);

    VolumeMorphResult {
        corrected_deltas,
        original_volume,
        morphed_volume,
        corrected_volume,
        volume_error_pct,
    }
}

#[allow(dead_code)]
pub fn uniform_scale_correction(
    _base: &[[f32; 3]],
    deltas: &[[f32; 3]],
    scale: f32,
) -> Vec<[f32; 3]> {
    deltas
        .iter()
        .map(|d| [d[0] * scale, d[1] * scale, d[2] * scale])
        .collect()
}

#[allow(dead_code)]
pub fn laplacian_smooth_deltas(
    deltas: &[[f32; 3]],
    indices: &[u32],
    iterations: u32,
) -> Vec<[f32; 3]> {
    let n = deltas.len();
    if n == 0 || indices.is_empty() {
        return deltas.to_vec();
    }

    // Build one-ring adjacency
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        if i0 < n && i1 < n && i2 < n {
            if !neighbors[i0].contains(&i1) {
                neighbors[i0].push(i1);
            }
            if !neighbors[i0].contains(&i2) {
                neighbors[i0].push(i2);
            }
            if !neighbors[i1].contains(&i0) {
                neighbors[i1].push(i0);
            }
            if !neighbors[i1].contains(&i2) {
                neighbors[i1].push(i2);
            }
            if !neighbors[i2].contains(&i0) {
                neighbors[i2].push(i0);
            }
            if !neighbors[i2].contains(&i1) {
                neighbors[i2].push(i1);
            }
        }
    }

    let mut current = deltas.to_vec();
    for _ in 0..iterations {
        let prev = current.clone();
        for i in 0..n {
            let nb = &neighbors[i];
            if nb.is_empty() {
                continue;
            }
            let mut avg = [0.0f32; 3];
            for &j in nb {
                avg[0] += prev[j][0];
                avg[1] += prev[j][1];
                avg[2] += prev[j][2];
            }
            let k = nb.len() as f32;
            current[i] = [avg[0] / k, avg[1] / k, avg[2] / k];
        }
    }
    current
}

#[allow(dead_code)]
pub fn mesh_volume_ratio(
    base_positions: &[[f32; 3]],
    morphed_positions: &[[f32; 3]],
    indices: &[u32],
) -> f32 {
    let v_base = compute_mesh_volume(base_positions, indices);
    let v_morphed = compute_mesh_volume(morphed_positions, indices);
    if v_morphed.abs() < 1e-10 {
        return 1.0;
    }
    v_base / v_morphed
}

#[allow(dead_code)]
pub fn volume_error_percent(original: f32, corrected: f32) -> f32 {
    if original.abs() < 1e-10 {
        return 0.0;
    }
    (corrected - original) / original * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a regular tetrahedron with vertices at (1,1,1), (-1,-1,1), (-1,1,-1), (1,-1,-1).
    /// Volume = 8/3.
    fn tetra_positions() -> Vec<[f32; 3]> {
        vec![
            [1.0, 1.0, 1.0],
            [-1.0, -1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [1.0, -1.0, -1.0],
        ]
    }

    fn tetra_indices() -> Vec<u32> {
        // Four faces of the tetrahedron, all wound consistently outward
        vec![0, 1, 2, 0, 2, 3, 0, 3, 1, 1, 3, 2]
    }

    #[test]
    fn test_compute_mesh_volume_tetrahedron() {
        let pos = tetra_positions();
        let idx = tetra_indices();
        let vol = compute_mesh_volume(&pos, &idx).abs();
        // Volume of this regular tetrahedron = 8/3 ≈ 2.667
        assert!((vol - 8.0 / 3.0).abs() < 0.01, "tetra volume: {vol}");
    }

    #[test]
    fn test_compute_mesh_volume_empty() {
        let vol = compute_mesh_volume(&[], &[]);
        assert_eq!(vol, 0.0);
    }

    #[test]
    fn test_compute_mesh_volume_single_triangle() {
        // Degenerate: single triangle contributes a wedge
        let pos = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let idx = vec![0u32, 1, 2];
        let vol = compute_mesh_volume(&pos, &idx).abs();
        // Signed wedge volume = (1/6)|v0·(v1×v2)|
        // v1×v2 = (1,0,0)×(0,0,1)-(0,1,0)×(0,0,1) ... = (1,-1,-1) wait, let's compute:
        // v1=(0,1,0), v2=(0,0,1) => v1×v2=(1*1-0*0, 0*0-0*1, 0*0-1*0)=(1,0,0)
        // v0·(1,0,0) = 1 => vol = 1/6
        assert!((vol - 1.0 / 6.0).abs() < 1e-5, "single tri vol: {vol}");
    }

    #[test]
    fn test_volume_error_percent_formula() {
        let pct = volume_error_percent(100.0, 105.0);
        assert!((pct - 5.0).abs() < 1e-5, "5% error: {pct}");
    }

    #[test]
    fn test_volume_error_percent_zero_original() {
        let pct = volume_error_percent(0.0, 5.0);
        assert_eq!(pct, 0.0);
    }

    #[test]
    fn test_uniform_scale_correction() {
        let base = vec![[0.0, 0.0, 0.0]];
        let deltas = vec![[1.0, 2.0, 3.0]];
        let result = uniform_scale_correction(&base, &deltas, 2.0);
        assert!((result[0][0] - 2.0).abs() < 1e-6);
        assert!((result[0][1] - 4.0).abs() < 1e-6);
        assert!((result[0][2] - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_uniform_scale_correction_identity() {
        let base = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let deltas = vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0]];
        let result = uniform_scale_correction(&base, &deltas, 1.0);
        assert!((result[0][0] - 2.0).abs() < 1e-6);
        assert!((result[1][0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_laplacian_smooth_reduces_magnitude() {
        // A single spike at vertex 0 should be smoothed down
        let deltas = vec![[10.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let indices = vec![0u32, 1, 2];
        let smoothed = laplacian_smooth_deltas(&deltas, &indices, 1);
        // vertex 0 neighbors are 1,2 → both have 0 delta → smoothed[0] = 0
        assert!(
            smoothed[0][0].abs() < deltas[0][0].abs(),
            "spike should be reduced: {}",
            smoothed[0][0]
        );
    }

    #[test]
    fn test_laplacian_smooth_zero_delta_noop() {
        let deltas = vec![[0.0, 0.0, 0.0]; 4];
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        let smoothed = laplacian_smooth_deltas(&deltas, &indices, 3);
        for d in &smoothed {
            for &v in d {
                assert!(v.abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_mesh_volume_ratio_same_mesh() {
        let pos = tetra_positions();
        let idx = tetra_indices();
        let ratio = mesh_volume_ratio(&pos, &pos, &idx);
        assert!((ratio - 1.0).abs() < 1e-5, "same mesh ratio=1: {ratio}");
    }

    #[test]
    fn test_volume_preserving_delta_reduces_error() {
        let pos = tetra_positions();
        let idx = tetra_indices();
        // Inflate all vertices by 20% — this changes volume significantly
        let inflate: Vec<[f32; 3]> = pos
            .iter()
            .map(|v| [v[0] * 0.2, v[1] * 0.2, v[2] * 0.2])
            .collect();
        let cfg = VolumeMorphConfig {
            preservation_strength: 1.0,
            iterations: 1,
            smooth_correction: false,
        };
        let result = volume_preserving_delta(&pos, &idx, &inflate, &cfg);
        // Corrected volume error should be less than uncorrected
        let uncorrected_pct =
            volume_error_percent(result.original_volume, result.morphed_volume).abs();
        let corrected_pct = result.volume_error_pct.abs();
        assert!(
            corrected_pct < uncorrected_pct,
            "corrected error {corrected_pct} should be < uncorrected {uncorrected_pct}"
        );
    }

    #[test]
    fn test_volume_preserving_delta_zero_deltas() {
        let pos = tetra_positions();
        let idx = tetra_indices();
        let deltas = vec![[0.0, 0.0, 0.0]; pos.len()];
        let cfg = VolumeMorphConfig::default();
        let result = volume_preserving_delta(&pos, &idx, &deltas, &cfg);
        assert!(
            (result.original_volume - result.morphed_volume).abs() < 1e-5,
            "zero delta: volumes should match"
        );
    }

    #[test]
    fn test_laplacian_smooth_empty() {
        let result = laplacian_smooth_deltas(&[], &[], 3);
        assert!(result.is_empty());
    }
}
