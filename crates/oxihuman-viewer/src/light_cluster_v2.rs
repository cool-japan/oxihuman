// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Light clustering for clustered shading.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightClusterV2 {
    pub cluster_count: [u32; 3],
    pub near: f32,
    pub far: f32,
    pub lights: Vec<Vec<u32>>,
}

#[allow(dead_code)]
pub fn new_light_cluster_v2(x: u32, y: u32, z: u32, near: f32, far: f32) -> LightClusterV2 {
    let total = (x * y * z) as usize;
    LightClusterV2 {
        cluster_count: [x, y, z],
        near,
        far,
        lights: vec![Vec::new(); total],
    }
}

#[allow(dead_code)]
pub fn lcv2_total_clusters(cluster: &LightClusterV2) -> usize {
    cluster.lights.len()
}

#[allow(dead_code)]
pub fn lcv2_assign_light(cluster: &mut LightClusterV2, idx: usize, light_id: u32) {
    if idx < cluster.lights.len() {
        cluster.lights[idx].push(light_id);
    }
}

#[allow(dead_code)]
pub fn lcv2_lights_in(cluster: &LightClusterV2, idx: usize) -> &[u32] {
    if idx < cluster.lights.len() {
        &cluster.lights[idx]
    } else {
        &[]
    }
}

#[allow(dead_code)]
pub fn lcv2_avg_lights_per_cluster(cluster: &LightClusterV2) -> f32 {
    if cluster.lights.is_empty() {
        return 0.0;
    }
    let total: usize = cluster.lights.iter().map(|l| l.len()).sum();
    total as f32 / cluster.lights.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_clusters() {
        let c = new_light_cluster_v2(4, 4, 4, 0.1, 100.0);
        assert_eq!(lcv2_total_clusters(&c), 64);
    }

    #[test]
    fn test_assign_light() {
        let mut c = new_light_cluster_v2(2, 2, 2, 0.1, 100.0);
        lcv2_assign_light(&mut c, 0, 42);
        assert_eq!(lcv2_lights_in(&c, 0), &[42]);
    }

    #[test]
    fn test_lights_in_empty() {
        let c = new_light_cluster_v2(2, 2, 2, 0.1, 100.0);
        assert_eq!(lcv2_lights_in(&c, 0).len(), 0);
    }

    #[test]
    fn test_avg_lights_per_cluster_zero() {
        let c = new_light_cluster_v2(2, 2, 2, 0.1, 100.0);
        assert!((lcv2_avg_lights_per_cluster(&c) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_avg_lights_per_cluster_nonzero() {
        let mut c = new_light_cluster_v2(2, 1, 1, 0.1, 100.0);
        lcv2_assign_light(&mut c, 0, 1);
        lcv2_assign_light(&mut c, 0, 2);
        let avg = lcv2_avg_lights_per_cluster(&c);
        assert!(avg > 0.0);
    }

    #[test]
    fn test_out_of_range_idx_empty() {
        let c = new_light_cluster_v2(2, 2, 2, 0.1, 100.0);
        assert_eq!(lcv2_lights_in(&c, 9999).len(), 0);
    }

    #[test]
    fn test_cluster_count_stored() {
        let c = new_light_cluster_v2(3, 4, 5, 0.1, 100.0);
        assert_eq!(c.cluster_count, [3, 4, 5]);
    }

    #[test]
    fn test_near_far_stored() {
        let c = new_light_cluster_v2(2, 2, 2, 0.5, 50.0);
        assert!((c.near - 0.5).abs() < 1e-6);
        assert!((c.far - 50.0).abs() < 1e-6);
    }
}
