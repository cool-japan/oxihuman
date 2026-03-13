// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bilateral mesh smoothing — edge-preserving filter on vertex positions.

/// Configuration for bilateral mesh filter.
#[derive(Debug, Clone)]
pub struct BilateralConfig {
    pub iterations: usize,
    /// Spatial sigma — controls influence by distance.
    pub sigma_s: f32,
    /// Feature sigma — controls influence by normal deviation.
    pub sigma_r: f32,
}

impl Default for BilateralConfig {
    fn default() -> Self {
        Self { iterations: 1, sigma_s: 1.0, sigma_r: 0.5 }
    }
}

/// Apply bilateral smoothing to mesh positions.
pub fn bilateral_smooth(
    positions: &[[f32; 3]],
    indices: &[u32],
    normals: &[[f32; 3]],
    cfg: &BilateralConfig,
) -> Vec<[f32; 3]> {
    let nv = positions.len();
    if nv == 0 { return vec![]; }
    let adj = build_adjacency_bil(indices, nv);
    let mut pos: Vec<[f32; 3]> = positions.to_vec();
    let ss2 = 2.0 * cfg.sigma_s * cfg.sigma_s;
    let sr2 = 2.0 * cfg.sigma_r * cfg.sigma_r;

    for _ in 0..cfg.iterations {
        let old = pos.clone();
        for v in 0..nv {
            let neighbors = &adj[v];
            if neighbors.is_empty() { continue; }
            let nv_norm = if v < normals.len() { normals[v] } else { [0.0, 1.0, 0.0] };
            let mut sum = [0.0f32; 3];
            let mut weight_sum = 0.0f32;
            for &nb in neighbors {
                let d2 = dist_sq(old[v], old[nb]);
                let nb_norm = if nb < normals.len() { normals[nb] } else { [0.0, 1.0, 0.0] };
                let dot = dot3(nv_norm, nb_norm).clamp(-1.0, 1.0);
                let angle = dot.acos();
                let w = (-d2 / ss2).exp() * (-(angle * angle) / sr2).exp();
                sum[0] += old[nb][0] * w;
                sum[1] += old[nb][1] * w;
                sum[2] += old[nb][2] * w;
                weight_sum += w;
            }
            if weight_sum > 1e-12 {
                pos[v] = [sum[0] / weight_sum, sum[1] / weight_sum, sum[2] / weight_sum];
            }
        }
    }
    pos
}

/// Default bilateral filter configuration.
pub fn default_bilateral_config() -> BilateralConfig {
    BilateralConfig::default()
}

/// Average movement magnitude after bilateral filtering.
pub fn bilateral_avg_displacement(original: &[[f32; 3]], filtered: &[[f32; 3]]) -> f32 {
    let n = original.len().min(filtered.len());
    if n == 0 { return 0.0; }
    let sum: f32 = (0..n).map(|i| {
        let dx = filtered[i][0] - original[i][0];
        let dy = filtered[i][1] - original[i][1];
        let dz = filtered[i][2] - original[i][2];
        (dx*dx+dy*dy+dz*dz).sqrt()
    }).sum();
    sum / n as f32
}

/// Check if sigma values are valid.
pub fn bilateral_config_valid(cfg: &BilateralConfig) -> bool {
    cfg.sigma_s > 0.0 && cfg.sigma_r > 0.0 && cfg.iterations > 0
}

fn build_adjacency_bil(indices: &[u32], nv: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![vec![]; nv];
    for tri in indices.chunks(3) {
        if tri.len() < 3 { continue; }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if a < nv && b < nv && c < nv {
            for &(u, v) in &[(a, b), (b, c), (c, a)] {
                if !adj[u].contains(&v) { adj[u].push(v); }
                if !adj[v].contains(&u) { adj[v].push(u); }
            }
        }
    }
    adj
}

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0]-b[0]; let dy = a[1]-b[1]; let dz = a[2]-b[2];
    dx*dx + dy*dy + dz*dz
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0]*b[0] + a[1]*b[1] + a[2]*b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_mesh() -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 3]>) {
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[2.0,0.0,0.0],[1.0,1.0,0.0]];
        let i = vec![0u32,1,3, 1,2,3];
        let n = vec![[0.0,0.0,1.0];4];
        (p, i, n)
    }

    #[test]
    fn test_bilateral_same_count() {
        /* output length matches input */
        let (p, i, n) = flat_mesh();
        let s = bilateral_smooth(&p, &i, &n, &default_bilateral_config());
        assert_eq!(s.len(), p.len());
    }

    #[test]
    fn test_bilateral_empty() {
        /* empty input → empty output */
        let s = bilateral_smooth(&[], &[], &[], &default_bilateral_config());
        assert!(s.is_empty());
    }

    #[test]
    fn test_bilateral_config_valid_default() {
        /* default config is valid */
        assert!(bilateral_config_valid(&default_bilateral_config()));
    }

    #[test]
    fn test_bilateral_config_invalid_sigma() {
        /* zero sigma is invalid */
        let cfg = BilateralConfig { sigma_s: 0.0, ..BilateralConfig::default() };
        assert!(!bilateral_config_valid(&cfg));
    }

    #[test]
    fn test_bilateral_avg_displacement_zero() {
        /* same input → zero displacement */
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0]];
        assert!((bilateral_avg_displacement(&p, &p)).abs() < 1e-6);
    }

    #[test]
    fn test_bilateral_avg_displacement_positive() {
        /* shifted positions → positive avg displacement */
        let orig = vec![[0.0,0.0,0.0]];
        let shifted = vec![[1.0,0.0,0.0]];
        assert!(bilateral_avg_displacement(&orig, &shifted) > 0.0);
    }

    #[test]
    fn test_bilateral_no_crash_zero_iterations() {
        /* zero iterations acts as identity */
        let (p, i, n) = flat_mesh();
        let cfg = BilateralConfig { iterations: 0, ..BilateralConfig::default() };
        let s = bilateral_smooth(&p, &i, &n, &cfg);
        assert_eq!(s.len(), p.len());
    }

    #[test]
    fn test_bilateral_sigma_s_positive() {
        /* default sigma_s > 0 */
        assert!(default_bilateral_config().sigma_s > 0.0);
    }

    #[test]
    fn test_bilateral_sigma_r_positive() {
        /* default sigma_r > 0 */
        assert!(default_bilateral_config().sigma_r > 0.0);
    }

    #[test]
    fn test_bilateral_with_no_normals() {
        /* missing normals should not panic */
        let (p, i, _) = flat_mesh();
        let s = bilateral_smooth(&p, &i, &[], &default_bilateral_config());
        assert_eq!(s.len(), p.len());
    }
}
