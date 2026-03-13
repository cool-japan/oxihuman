// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-instance LOD selection based on screen-space projected size.

use std::f32::consts::FRAC_PI_4;

pub const MAX_LOD_LEVELS: usize = 5;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstanceLodConfig {
    /// Minimum projected radius (pixels) to switch to each LOD level.
    pub thresholds: [f32; MAX_LOD_LEVELS],
}

impl Default for InstanceLodConfig {
    fn default() -> Self {
        Self {
            thresholds: [200.0, 80.0, 30.0, 10.0, 3.0],
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstanceLodEntry {
    pub id: u32,
    pub projected_radius: f32,
    pub lod: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct InstanceLodManager {
    pub config: InstanceLodConfig,
    pub instances: Vec<InstanceLodEntry>,
}

#[allow(dead_code)]
pub fn default_instance_lod_config() -> InstanceLodConfig {
    InstanceLodConfig::default()
}

#[allow(dead_code)]
pub fn new_instance_lod_manager(config: InstanceLodConfig) -> InstanceLodManager {
    InstanceLodManager {
        config,
        instances: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn ilod_select_level(config: &InstanceLodConfig, projected_radius: f32) -> usize {
    for (i, &thresh) in config.thresholds.iter().enumerate() {
        if projected_radius >= thresh {
            return i;
        }
    }
    MAX_LOD_LEVELS - 1
}

#[allow(dead_code)]
pub fn ilod_register(mgr: &mut InstanceLodManager, id: u32, projected_radius: f32) {
    let lod = ilod_select_level(&mgr.config, projected_radius);
    mgr.instances.push(InstanceLodEntry {
        id,
        projected_radius,
        lod,
    });
}

#[allow(dead_code)]
pub fn ilod_clear(mgr: &mut InstanceLodManager) {
    mgr.instances.clear();
}

#[allow(dead_code)]
pub fn ilod_count(mgr: &InstanceLodManager) -> usize {
    mgr.instances.len()
}

#[allow(dead_code)]
pub fn ilod_count_at_level(mgr: &InstanceLodManager, level: usize) -> usize {
    mgr.instances.iter().filter(|e| e.lod == level).count()
}

#[allow(dead_code)]
pub fn ilod_average_lod(mgr: &InstanceLodManager) -> f32 {
    if mgr.instances.is_empty() {
        return 0.0;
    }
    mgr.instances.iter().map(|e| e.lod as f32).sum::<f32>() / mgr.instances.len() as f32
}

#[allow(dead_code)]
pub fn ilod_lod_angle_rad(mgr: &InstanceLodManager) -> f32 {
    ilod_average_lod(mgr) * FRAC_PI_4
}

#[allow(dead_code)]
pub fn ilod_to_json(mgr: &InstanceLodManager) -> String {
    format!(
        "{{\"count\":{},\"avg_lod\":{:.4}}}",
        ilod_count(mgr),
        ilod_average_lod(mgr)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_is_empty() {
        assert_eq!(
            ilod_count(&new_instance_lod_manager(default_instance_lod_config())),
            0
        );
    }
    #[test]
    fn register_increments_count() {
        let mut m = new_instance_lod_manager(default_instance_lod_config());
        ilod_register(&mut m, 0, 250.0);
        assert_eq!(ilod_count(&m), 1);
    }
    #[test]
    fn clear_empties() {
        let mut m = new_instance_lod_manager(default_instance_lod_config());
        ilod_register(&mut m, 0, 50.0);
        ilod_clear(&mut m);
        assert_eq!(ilod_count(&m), 0);
    }
    #[test]
    fn large_radius_lod_zero() {
        assert_eq!(ilod_select_level(&default_instance_lod_config(), 300.0), 0);
    }
    #[test]
    fn tiny_radius_max_lod() {
        assert_eq!(
            ilod_select_level(&default_instance_lod_config(), 1.0),
            MAX_LOD_LEVELS - 1
        );
    }
    #[test]
    fn count_at_level() {
        let mut m = new_instance_lod_manager(default_instance_lod_config());
        ilod_register(&mut m, 0, 300.0);
        ilod_register(&mut m, 1, 300.0);
        assert_eq!(ilod_count_at_level(&m, 0), 2);
    }
    #[test]
    fn average_lod_empty_zero() {
        assert!(
            ilod_average_lod(&new_instance_lod_manager(default_instance_lod_config())).abs() < 1e-6
        );
    }
    #[test]
    fn lod_angle_nonneg() {
        assert!(
            ilod_lod_angle_rad(&new_instance_lod_manager(default_instance_lod_config())) >= 0.0
        );
    }
    #[test]
    fn to_json_has_count() {
        assert!(
            ilod_to_json(&new_instance_lod_manager(default_instance_lod_config()))
                .contains("\"count\"")
        );
    }
    #[test]
    fn max_lod_levels_is_five() {
        assert_eq!(MAX_LOD_LEVELS, 5);
    }
}
