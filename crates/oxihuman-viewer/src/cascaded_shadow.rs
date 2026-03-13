// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! CascadedShadow — cascaded shadow map (CSM) management.

#![allow(dead_code)]

/// Data for a single shadow cascade.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShadowCascade {
    pub near: f32,
    pub far: f32,
    pub bias: f32,
    /// Column-major 4×4 view-projection matrix.
    pub view_proj: [f32; 16],
}

/// Cascaded shadow map state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CascadedShadow {
    pub cascades: Vec<ShadowCascade>,
    pub lambda: f32,
}

/// Create a new `CascadedShadow` with `n` cascades.
#[allow(dead_code)]
pub fn new_cascaded_shadow(n: usize, lambda: f32) -> CascadedShadow {
    CascadedShadow { cascades: Vec::with_capacity(n), lambda }
}

/// Return the number of cascades.
#[allow(dead_code)]
pub fn cascade_count(cs: &CascadedShadow) -> usize {
    cs.cascades.len()
}

/// Return the split distance for cascade `i`.
#[allow(dead_code)]
pub fn cascade_split_at(cs: &CascadedShadow, i: usize) -> f32 {
    cs.cascades.get(i).map(|c| c.far).unwrap_or(0.0)
}

/// Return the near distance for cascade `i`.
#[allow(dead_code)]
pub fn cascade_near(cs: &CascadedShadow, i: usize) -> f32 {
    cs.cascades.get(i).map(|c| c.near).unwrap_or(0.0)
}

/// Return the far distance for cascade `i`.
#[allow(dead_code)]
pub fn cascade_far(cs: &CascadedShadow, i: usize) -> f32 {
    cs.cascades.get(i).map(|c| c.far).unwrap_or(0.0)
}

/// Compute and fill cascade split distances using a log-linear blend.
#[allow(dead_code)]
pub fn compute_cascade_splits(cs: &mut CascadedShadow, near: f32, far: f32, n: usize) {
    cs.cascades.clear();
    let lambda = cs.lambda;
    let mut prev_near = near;
    for i in 0..n {
        let fi = (i + 1) as f32 / n as f32;
        let log_split = near * (far / near.max(f32::EPSILON)).powf(fi);
        let lin_split = near + (far - near) * fi;
        let split = lambda * log_split + (1.0 - lambda) * lin_split;
        let identity = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        cs.cascades.push(ShadowCascade { near: prev_near, far: split, bias: 0.005, view_proj: identity });
        prev_near = split;
    }
}

/// Return the view-projection matrix for cascade `i`.
#[allow(dead_code)]
pub fn cascade_view_proj(cs: &CascadedShadow, i: usize) -> Option<[f32; 16]> {
    cs.cascades.get(i).map(|c| c.view_proj)
}

/// Return the shadow bias for cascade `i`.
#[allow(dead_code)]
pub fn cascade_bias(cs: &CascadedShadow, i: usize) -> f32 {
    cs.cascades.get(i).map(|c| c.bias).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cascaded_shadow_empty() {
        let cs = new_cascaded_shadow(4, 0.5);
        assert_eq!(cascade_count(&cs), 0);
    }

    #[test]
    fn test_compute_cascade_splits_count() {
        let mut cs = new_cascaded_shadow(4, 0.5);
        compute_cascade_splits(&mut cs, 0.1, 100.0, 4);
        assert_eq!(cascade_count(&cs), 4);
    }

    #[test]
    fn test_cascade_splits_increasing() {
        let mut cs = new_cascaded_shadow(4, 0.5);
        compute_cascade_splits(&mut cs, 0.1, 100.0, 4);
        for i in 1..cascade_count(&cs) {
            assert!(cascade_far(&cs, i) > cascade_far(&cs, i - 1));
        }
    }

    #[test]
    fn test_cascade_near_first() {
        let mut cs = new_cascaded_shadow(3, 0.5);
        compute_cascade_splits(&mut cs, 0.1, 50.0, 3);
        assert!((cascade_near(&cs, 0) - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_cascade_split_at_last() {
        let mut cs = new_cascaded_shadow(3, 0.5);
        compute_cascade_splits(&mut cs, 0.1, 50.0, 3);
        let last = cascade_count(&cs) - 1;
        assert!(cascade_split_at(&cs, last) <= 50.0 + 1e-3);
    }

    #[test]
    fn test_cascade_view_proj_identity() {
        let mut cs = new_cascaded_shadow(2, 0.5);
        compute_cascade_splits(&mut cs, 0.1, 10.0, 2);
        let vp = cascade_view_proj(&cs, 0).expect("should succeed");
        assert!((vp[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cascade_bias_default() {
        let mut cs = new_cascaded_shadow(1, 0.5);
        compute_cascade_splits(&mut cs, 0.1, 10.0, 1);
        assert!((cascade_bias(&cs, 0) - 0.005).abs() < 1e-6);
    }

    #[test]
    fn test_cascade_out_of_bounds() {
        let cs = new_cascaded_shadow(4, 0.5);
        assert_eq!(cascade_far(&cs, 99), 0.0);
        assert!(cascade_view_proj(&cs, 99).is_none());
    }
}
