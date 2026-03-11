// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Clustered lighting — divides the view frustum into 3-D tiles and assigns lights.

use std::f32::consts::FRAC_PI_3;

/// A clustered light.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct ClusterLight {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub radius: f32,
    pub intensity: f32,
    pub enabled: bool,
}

impl Default for ClusterLight {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0],
            radius: 5.0,
            intensity: 1.0,
            enabled: true,
        }
    }
}

/// Cluster grid configuration.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct ClusterConfig {
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub tiles_z: u32,
    pub max_lights_per_tile: usize,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            tiles_x: 16,
            tiles_y: 9,
            tiles_z: 24,
            max_lights_per_tile: 256,
        }
    }
}

/// Cluster light manager.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ClusterLightManager {
    pub config: ClusterConfig,
    pub lights: Vec<ClusterLight>,
}

/// Create a new manager.
#[allow(dead_code)]
pub fn new_cluster_light_manager(cfg: ClusterConfig) -> ClusterLightManager {
    ClusterLightManager {
        config: cfg,
        lights: Vec::new(),
    }
}

/// Add a light.
#[allow(dead_code)]
pub fn add_cluster_light(m: &mut ClusterLightManager, light: ClusterLight) -> usize {
    let idx = m.lights.len();
    m.lights.push(light);
    idx
}

/// Remove a light by index.
#[allow(dead_code)]
pub fn remove_cluster_light(m: &mut ClusterLightManager, idx: usize) {
    if idx < m.lights.len() {
        m.lights.remove(idx);
    }
}

/// Total light count.
#[allow(dead_code)]
pub fn cluster_light_count(m: &ClusterLightManager) -> usize {
    m.lights.len()
}

/// Count enabled lights.
#[allow(dead_code)]
pub fn enabled_light_count(m: &ClusterLightManager) -> usize {
    m.lights.iter().filter(|l| l.enabled).count()
}

/// Compute total tile count.
#[allow(dead_code)]
pub fn total_tile_count(cfg: &ClusterConfig) -> u32 {
    cfg.tiles_x * cfg.tiles_y * cfg.tiles_z
}

/// Estimate cluster depth slice using FRAC_PI_3 as a logarithmic base.
#[allow(dead_code)]
pub fn cluster_depth_slice(z: f32, near: f32, far: f32, slices: u32) -> u32 {
    if z <= near {
        return 0;
    }
    if z >= far {
        return slices - 1;
    }
    let ratio = (z - near) / (far - near);
    let slice = (ratio * slices as f32 * FRAC_PI_3 / std::f32::consts::PI).min((slices - 1) as f32);
    slice as u32
}

/// Compute light influence weight for a tile center.
#[allow(dead_code)]
pub fn light_influence(light: &ClusterLight, tile_center: [f32; 3]) -> f32 {
    let dx = light.position[0] - tile_center[0];
    let dy = light.position[1] - tile_center[1];
    let dz = light.position[2] - tile_center[2];
    let dist2 = dx * dx + dy * dy + dz * dz;
    let r2 = light.radius * light.radius;
    if dist2 >= r2 {
        0.0
    } else {
        (1.0 - dist2 / r2) * light.intensity
    }
}

/// Export to JSON-like string.
#[allow(dead_code)]
pub fn cluster_light_to_json(m: &ClusterLightManager) -> String {
    format!(
        r#"{{"light_count":{},"tiles":{},"enabled":{}}}"#,
        m.lights.len(),
        total_tile_count(&m.config),
        enabled_light_count(m)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count() {
        let mut m = new_cluster_light_manager(ClusterConfig::default());
        add_cluster_light(&mut m, ClusterLight::default());
        assert_eq!(cluster_light_count(&m), 1);
    }

    #[test]
    fn remove_light() {
        let mut m = new_cluster_light_manager(ClusterConfig::default());
        add_cluster_light(&mut m, ClusterLight::default());
        remove_cluster_light(&mut m, 0);
        assert_eq!(cluster_light_count(&m), 0);
    }

    #[test]
    fn enabled_count() {
        let mut m = new_cluster_light_manager(ClusterConfig::default());
        add_cluster_light(
            &mut m,
            ClusterLight {
                enabled: true,
                ..Default::default()
            },
        );
        add_cluster_light(
            &mut m,
            ClusterLight {
                enabled: false,
                ..Default::default()
            },
        );
        assert_eq!(enabled_light_count(&m), 1);
    }

    #[test]
    fn total_tiles() {
        let cfg = ClusterConfig {
            tiles_x: 2,
            tiles_y: 3,
            tiles_z: 4,
            ..Default::default()
        };
        assert_eq!(total_tile_count(&cfg), 24);
    }

    #[test]
    fn depth_slice_clamps_near() {
        assert_eq!(cluster_depth_slice(0.0, 0.1, 100.0, 24), 0);
    }

    #[test]
    fn depth_slice_clamps_far() {
        assert_eq!(cluster_depth_slice(200.0, 0.1, 100.0, 24), 23);
    }

    #[test]
    fn light_influence_inside() {
        let l = ClusterLight {
            position: [0.0, 0.0, 0.0],
            radius: 10.0,
            intensity: 1.0,
            ..Default::default()
        };
        let w = light_influence(&l, [5.0, 0.0, 0.0]);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn light_influence_outside() {
        let l = ClusterLight {
            position: [0.0, 0.0, 0.0],
            radius: 1.0,
            intensity: 1.0,
            ..Default::default()
        };
        assert!((light_influence(&l, [10.0, 0.0, 0.0])).abs() < 1e-6);
    }

    #[test]
    fn json_contains_light_count() {
        let m = new_cluster_light_manager(ClusterConfig::default());
        assert!(cluster_light_to_json(&m).contains("light_count"));
    }

    #[test]
    fn default_config_tiles() {
        let cfg = ClusterConfig::default();
        assert!(cfg.tiles_x > 0 && cfg.tiles_y > 0 && cfg.tiles_z > 0);
    }
}
