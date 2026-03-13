// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cascaded shadow maps parameters.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShadowCascadeV2 {
    pub near: f32,
    pub far: f32,
    pub bias: f32,
    pub resolution: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShadowCascadeSetV2 {
    pub cascades: Vec<ShadowCascadeV2>,
    pub num_cascades: usize,
}

#[allow(dead_code)]
pub fn new_shadow_cascade_set_v2(num_cascades: usize, near: f32, far: f32) -> ShadowCascadeSetV2 {
    let n = num_cascades.max(1);
    let range = far - near;
    let step = range / n as f32;
    let cascades = (0..n)
        .map(|i| ShadowCascadeV2 {
            near: near + i as f32 * step,
            far: near + (i + 1) as f32 * step,
            bias: 0.005,
            resolution: 1024,
        })
        .collect();
    ShadowCascadeSetV2 { cascades, num_cascades: n }
}

#[allow(dead_code)]
pub fn scsv2_cascade_count(set: &ShadowCascadeSetV2) -> usize {
    set.cascades.len()
}

#[allow(dead_code)]
pub fn scsv2_cascade_at(set: &ShadowCascadeSetV2, i: usize) -> &ShadowCascadeV2 {
    &set.cascades[i]
}

#[allow(dead_code)]
pub fn scsv2_set_bias(set: &mut ShadowCascadeSetV2, bias: f32) {
    for c in &mut set.cascades {
        c.bias = bias;
    }
}

#[allow(dead_code)]
pub fn scsv2_split_depth(set: &ShadowCascadeSetV2, i: usize) -> f32 {
    set.cascades[i].far
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cascade_count() {
        let s = new_shadow_cascade_set_v2(4, 0.1, 100.0);
        assert_eq!(scsv2_cascade_count(&s), 4);
    }

    #[test]
    fn test_cascade_at() {
        let s = new_shadow_cascade_set_v2(4, 0.0, 100.0);
        let c = scsv2_cascade_at(&s, 0);
        assert!((c.near - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_bias() {
        let mut s = new_shadow_cascade_set_v2(3, 0.1, 100.0);
        scsv2_set_bias(&mut s, 0.01);
        for c in &s.cascades {
            assert!((c.bias - 0.01).abs() < 1e-6);
        }
    }

    #[test]
    fn test_split_depth() {
        let s = new_shadow_cascade_set_v2(4, 0.0, 100.0);
        let depth = scsv2_split_depth(&s, 3);
        assert!((depth - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_near_less_than_far() {
        let s = new_shadow_cascade_set_v2(4, 0.1, 100.0);
        for c in &s.cascades {
            assert!(c.near < c.far);
        }
    }

    #[test]
    fn test_cascades_cover_range() {
        let s = new_shadow_cascade_set_v2(2, 0.0, 100.0);
        let first_near = scsv2_cascade_at(&s, 0).near;
        let last_far = scsv2_split_depth(&s, 1);
        assert!(first_near < 1.0);
        assert!((last_far - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_resolution_default() {
        let s = new_shadow_cascade_set_v2(1, 0.0, 10.0);
        assert_eq!(scsv2_cascade_at(&s, 0).resolution, 1024);
    }

    #[test]
    fn test_single_cascade() {
        let s = new_shadow_cascade_set_v2(1, 1.0, 50.0);
        assert_eq!(scsv2_cascade_count(&s), 1);
        assert!((scsv2_split_depth(&s, 0) - 50.0).abs() < 1e-4);
    }
}
