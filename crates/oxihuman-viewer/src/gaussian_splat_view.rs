// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct GaussianSplatConfig {
    pub num_gaussians: usize,
    pub sh_degree: usize,
    pub opacity_threshold: f32,
}

pub fn new_gaussian_splat_config(n: usize) -> GaussianSplatConfig {
    GaussianSplatConfig {
        num_gaussians: n,
        sh_degree: 3,
        opacity_threshold: 0.01,
    }
}

/// (degree+1)^2 SH coefficients per gaussian.
pub fn splat_sh_coeff_count(degree: usize) -> usize {
    (degree + 1) * (degree + 1)
}

/// pos(3) + rot(4) + scale(3) + opacity(1) + sh_coeffs per splat.
pub fn splat_param_count(cfg: &GaussianSplatConfig) -> usize {
    let per_splat = 3 + 4 + 3 + 1 + splat_sh_coeff_count(cfg.sh_degree);
    cfg.num_gaussians * per_splat
}

pub fn splat_memory_mb(cfg: &GaussianSplatConfig) -> f32 {
    (splat_param_count(cfg) * 4) as f32 / (1024.0 * 1024.0)
}

pub fn splat_cull_count(cfg: &GaussianSplatConfig, opacity_field: &[f32]) -> usize {
    opacity_field
        .iter()
        .take(cfg.num_gaussians)
        .filter(|&&o| o >= cfg.opacity_threshold)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config() {
        /* num_gaussians is set */
        let cfg = new_gaussian_splat_config(1000);
        assert_eq!(cfg.num_gaussians, 1000);
    }

    #[test]
    fn test_sh_coeff_degree3() {
        /* degree 3 => 16 coefficients */
        assert_eq!(splat_sh_coeff_count(3), 16);
    }

    #[test]
    fn test_sh_coeff_degree0() {
        /* degree 0 => 1 coefficient */
        assert_eq!(splat_sh_coeff_count(0), 1);
    }

    #[test]
    fn test_param_count_positive() {
        /* param count > 0 */
        let cfg = new_gaussian_splat_config(100);
        assert!(splat_param_count(&cfg) > 0);
    }

    #[test]
    fn test_memory_mb_positive() {
        /* memory > 0 */
        let cfg = new_gaussian_splat_config(100);
        assert!(splat_memory_mb(&cfg) > 0.0);
    }

    #[test]
    fn test_cull_count() {
        /* cull filters by threshold */
        let cfg = new_gaussian_splat_config(3);
        let opacity = vec![0.0, 0.5, 0.001];
        let count = splat_cull_count(&cfg, &opacity);
        assert_eq!(count, 1);
    }
}
