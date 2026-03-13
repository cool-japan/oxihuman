// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cluster culling v2 — extended AABB-based light / shadow cluster culling.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Aabb2 {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[allow(dead_code)]
impl Aabb2 {
    pub fn volume(&self) -> f32 {
        (self.max[0] - self.min[0]).max(0.0)
            * (self.max[1] - self.min[1]).max(0.0)
            * (self.max[2] - self.min[2]).max(0.0)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClusterEntryV2 {
    pub id: u32,
    pub aabb: Aabb2,
    pub visible: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ClusterCullV2 {
    pub entries: Vec<ClusterEntryV2>,
}

#[allow(dead_code)]
pub fn new_cluster_cull_v2() -> ClusterCullV2 {
    ClusterCullV2::default()
}

#[allow(dead_code)]
pub fn ccv2_register(cull: &mut ClusterCullV2, id: u32, aabb: Aabb2) {
    cull.entries.push(ClusterEntryV2 {
        id,
        aabb,
        visible: true,
    });
}

#[allow(dead_code)]
pub fn ccv2_clear(cull: &mut ClusterCullV2) {
    cull.entries.clear();
}

#[allow(dead_code)]
pub fn ccv2_count(cull: &ClusterCullV2) -> usize {
    cull.entries.len()
}

#[allow(dead_code)]
pub fn ccv2_visible_count(cull: &ClusterCullV2) -> usize {
    cull.entries.iter().filter(|e| e.visible).count()
}

#[allow(dead_code)]
pub fn ccv2_set_visible(cull: &mut ClusterCullV2, id: u32, vis: bool) {
    if let Some(e) = cull.entries.iter_mut().find(|e| e.id == id) {
        e.visible = vis;
    }
}

#[allow(dead_code)]
pub fn ccv2_mark_all_visible(cull: &mut ClusterCullV2) {
    for e in &mut cull.entries {
        e.visible = true;
    }
}

#[allow(dead_code)]
pub fn ccv2_cull_by_volume(cull: &mut ClusterCullV2, min_volume: f32) {
    for e in &mut cull.entries {
        if e.aabb.volume() < min_volume {
            e.visible = false;
        }
    }
}

#[allow(dead_code)]
pub fn ccv2_total_volume(cull: &ClusterCullV2) -> f32 {
    cull.entries
        .iter()
        .filter(|e| e.visible)
        .map(|e| e.aabb.volume())
        .sum()
}

#[allow(dead_code)]
pub fn ccv2_depth_angle_rad(cull: &ClusterCullV2) -> f32 {
    let v = ccv2_total_volume(cull);
    if v > 0.0 {
        (1.0 / v).atan().min(FRAC_PI_4)
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn ccv2_to_json(cull: &ClusterCullV2) -> String {
    format!(
        "{{\"count\":{},\"visible\":{}}}",
        ccv2_count(cull),
        ccv2_visible_count(cull)
    )
}

fn aabb2(min: [f32; 3], max: [f32; 3]) -> Aabb2 {
    Aabb2 { min, max }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_is_empty() {
        assert_eq!(ccv2_count(&new_cluster_cull_v2()), 0);
    }
    #[test]
    fn register_increments_count() {
        let mut c = new_cluster_cull_v2();
        ccv2_register(&mut c, 0, aabb2([0.0; 3], [1.0; 3]));
        assert_eq!(ccv2_count(&c), 1);
    }
    #[test]
    fn clear_empties() {
        let mut c = new_cluster_cull_v2();
        ccv2_register(&mut c, 0, aabb2([0.0; 3], [1.0; 3]));
        ccv2_clear(&mut c);
        assert_eq!(ccv2_count(&c), 0);
    }
    #[test]
    fn visible_count_after_hide() {
        let mut c = new_cluster_cull_v2();
        ccv2_register(&mut c, 0, aabb2([0.0; 3], [1.0; 3]));
        ccv2_set_visible(&mut c, 0, false);
        assert_eq!(ccv2_visible_count(&c), 0);
    }
    #[test]
    fn mark_all_visible() {
        let mut c = new_cluster_cull_v2();
        ccv2_register(&mut c, 0, aabb2([0.0; 3], [1.0; 3]));
        ccv2_set_visible(&mut c, 0, false);
        ccv2_mark_all_visible(&mut c);
        assert_eq!(ccv2_visible_count(&c), 1);
    }
    #[test]
    fn cull_by_volume_hides_small() {
        let mut c = new_cluster_cull_v2();
        ccv2_register(&mut c, 0, aabb2([0.0; 3], [0.1, 0.1, 0.1]));
        ccv2_cull_by_volume(&mut c, 1.0);
        assert_eq!(ccv2_visible_count(&c), 0);
    }
    #[test]
    fn aabb_volume_correct() {
        let a = aabb2([0.0; 3], [2.0, 3.0, 4.0]);
        assert!((a.volume() - 24.0).abs() < 1e-4);
    }
    #[test]
    fn total_volume_sums_visible() {
        let mut c = new_cluster_cull_v2();
        ccv2_register(&mut c, 0, aabb2([0.0; 3], [1.0; 3]));
        assert!((ccv2_total_volume(&c) - 1.0).abs() < 1e-4);
    }
    #[test]
    fn depth_angle_nonneg() {
        assert!(ccv2_depth_angle_rad(&new_cluster_cull_v2()) >= 0.0);
    }
    #[test]
    fn to_json_has_visible() {
        assert!(ccv2_to_json(&new_cluster_cull_v2()).contains("\"visible\""));
    }
}
