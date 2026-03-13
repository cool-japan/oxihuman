// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Cascade shadow map utilities.

/// Shadow cascade configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CascadeShadowConfig {
    pub cascade_count: u32,
    pub max_distance: f32,
    pub split_lambda: f32,
    pub map_size: u32,
}

/// A single cascade split.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CascadeSplit {
    pub near: f32,
    pub far: f32,
    pub index: u32,
}

/// Default cascade shadow config.
#[allow(dead_code)]
pub fn default_cascade_shadow_config() -> CascadeShadowConfig {
    CascadeShadowConfig {
        cascade_count: 4,
        max_distance: 100.0,
        split_lambda: 0.5,
        map_size: 2048,
    }
}

/// Compute cascade splits using logarithmic-linear split scheme.
#[allow(dead_code)]
pub fn compute_cascade_splits(config: &CascadeShadowConfig, near: f32) -> Vec<CascadeSplit> {
    let n = config.cascade_count.max(1);
    let mut splits = Vec::with_capacity(n as usize);
    let lambda = config.split_lambda.clamp(0.0, 1.0);
    let far = config.max_distance;

    for i in 0..n {
        let p = (i as f32 + 1.0) / n as f32;
        let log_split = near * (far / near).powf(p);
        let lin_split = near + (far - near) * p;
        let split_far = lambda * log_split + (1.0 - lambda) * lin_split;

        let p_prev = i as f32 / n as f32;
        let log_near = near * (far / near).powf(p_prev);
        let lin_near = near + (far - near) * p_prev;
        let split_near = if i == 0 { near } else { lambda * log_near + (1.0 - lambda) * lin_near };

        splits.push(CascadeSplit {
            near: split_near,
            far: split_far,
            index: i,
        });
    }
    splits
}

/// Get the cascade index for a given depth.
#[allow(dead_code)]
pub fn cascade_for_depth(splits: &[CascadeSplit], depth: f32) -> Option<u32> {
    for s in splits {
        if depth >= s.near && depth <= s.far {
            return Some(s.index);
        }
    }
    None
}

/// Total shadow map memory estimate in bytes.
#[allow(dead_code)]
pub fn shadow_map_memory(config: &CascadeShadowConfig) -> u64 {
    let per_cascade = (config.map_size as u64) * (config.map_size as u64) * 4;
    per_cascade * config.cascade_count as u64
}

/// Set cascade count.
#[allow(dead_code)]
pub fn set_cascade_count(config: &mut CascadeShadowConfig, count: u32) {
    config.cascade_count = count.clamp(1, 8);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_cascade_shadow_config();
        assert_eq!(c.cascade_count, 4);
    }

    #[test]
    fn test_compute_splits() {
        let c = default_cascade_shadow_config();
        let splits = compute_cascade_splits(&c, 0.1);
        assert_eq!(splits.len(), 4);
    }

    #[test]
    fn test_splits_ordering() {
        let c = default_cascade_shadow_config();
        let splits = compute_cascade_splits(&c, 0.1);
        for i in 1..splits.len() {
            assert!(splits[i].near >= splits[i - 1].near);
        }
    }

    #[test]
    fn test_cascade_for_depth() {
        let c = default_cascade_shadow_config();
        let splits = compute_cascade_splits(&c, 0.1);
        let idx = cascade_for_depth(&splits, 0.5);
        assert!(idx.is_some());
    }

    #[test]
    fn test_cascade_out_of_range() {
        let c = default_cascade_shadow_config();
        let splits = compute_cascade_splits(&c, 0.1);
        let idx = cascade_for_depth(&splits, 1000.0);
        assert!(idx.is_none());
    }

    #[test]
    fn test_shadow_memory() {
        let c = default_cascade_shadow_config();
        let mem = shadow_map_memory(&c);
        assert!(mem > 0);
    }

    #[test]
    fn test_set_cascade_count() {
        let mut c = default_cascade_shadow_config();
        set_cascade_count(&mut c, 2);
        assert_eq!(c.cascade_count, 2);
    }

    #[test]
    fn test_clamp_cascade_count() {
        let mut c = default_cascade_shadow_config();
        set_cascade_count(&mut c, 20);
        assert_eq!(c.cascade_count, 8);
    }

    #[test]
    fn test_first_cascade_starts_at_near() {
        let c = default_cascade_shadow_config();
        let splits = compute_cascade_splits(&c, 0.1);
        assert!((splits[0].near - 0.1).abs() < 1e-4);
    }

    #[test]
    fn test_single_cascade() {
        let mut c = default_cascade_shadow_config();
        c.cascade_count = 1;
        let splits = compute_cascade_splits(&c, 0.1);
        assert_eq!(splits.len(), 1);
    }
}
