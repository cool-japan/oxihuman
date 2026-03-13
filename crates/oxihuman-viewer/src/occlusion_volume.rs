// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Occlusion volume — AABB-based ambient occlusion volumes for indirect lighting.

/// An occlusion volume entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OcclusionVolume {
    pub id: u32,
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
    /// Occlusion strength (0 = transparent, 1 = fully occludes).
    pub strength: f32,
    pub enabled: bool,
}

impl OcclusionVolume {
    #[allow(dead_code)]
    pub fn new(id: u32, min: [f32; 3], max: [f32; 3], strength: f32) -> Self {
        Self {
            id,
            aabb_min: min,
            aabb_max: max,
            strength: strength.clamp(0.0, 1.0),
            enabled: true,
        }
    }

    /// Return the volume in cubic metres.
    #[allow(dead_code)]
    pub fn volume(&self) -> f32 {
        let dx = (self.aabb_max[0] - self.aabb_min[0]).max(0.0);
        let dy = (self.aabb_max[1] - self.aabb_min[1]).max(0.0);
        let dz = (self.aabb_max[2] - self.aabb_min[2]).max(0.0);
        dx * dy * dz
    }

    /// Return true if `point` is inside this volume.
    #[allow(dead_code)]
    pub fn contains(&self, p: [f32; 3]) -> bool {
        p[0] >= self.aabb_min[0]
            && p[0] <= self.aabb_max[0]
            && p[1] >= self.aabb_min[1]
            && p[1] <= self.aabb_max[1]
            && p[2] >= self.aabb_min[2]
            && p[2] <= self.aabb_max[2]
    }
}

/// Volume registry.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct OcclusionVolumeSet {
    volumes: Vec<OcclusionVolume>,
}

#[allow(dead_code)]
pub fn new_occlusion_volume_set() -> OcclusionVolumeSet {
    OcclusionVolumeSet::default()
}

#[allow(dead_code)]
pub fn ovs_add(set: &mut OcclusionVolumeSet, vol: OcclusionVolume) {
    set.volumes.push(vol);
}

#[allow(dead_code)]
pub fn ovs_remove(set: &mut OcclusionVolumeSet, id: u32) {
    set.volumes.retain(|v| v.id != id);
}

#[allow(dead_code)]
pub fn ovs_set_enabled(set: &mut OcclusionVolumeSet, id: u32, en: bool) {
    for v in set.volumes.iter_mut() {
        if v.id == id {
            v.enabled = en;
        }
    }
}

#[allow(dead_code)]
pub fn ovs_count(set: &OcclusionVolumeSet) -> usize {
    set.volumes.len()
}

#[allow(dead_code)]
pub fn ovs_enabled_count(set: &OcclusionVolumeSet) -> usize {
    set.volumes.iter().filter(|v| v.enabled).count()
}

#[allow(dead_code)]
pub fn ovs_clear(set: &mut OcclusionVolumeSet) {
    set.volumes.clear();
}

/// Combined occlusion at a point (max over all enabled volumes containing the point).
#[allow(dead_code)]
pub fn ovs_occlusion_at(set: &OcclusionVolumeSet, point: [f32; 3]) -> f32 {
    set.volumes
        .iter()
        .filter(|v| v.enabled && v.contains(point))
        .map(|v| v.strength)
        .fold(0.0f32, f32::max)
}

/// Total volume of all enabled entries in cubic metres.
#[allow(dead_code)]
pub fn ovs_total_volume(set: &OcclusionVolumeSet) -> f32 {
    set.volumes
        .iter()
        .filter(|v| v.enabled)
        .map(|v| v.volume())
        .sum()
}

#[allow(dead_code)]
pub fn ovs_to_json(set: &OcclusionVolumeSet) -> String {
    format!(
        "{{\"count\":{},\"enabled\":{},\"total_volume\":{:.4}}}",
        ovs_count(set),
        ovs_enabled_count(set),
        ovs_total_volume(set)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vol(id: u32) -> OcclusionVolume {
        OcclusionVolume::new(id, [-1.0; 3], [1.0; 3], 0.5)
    }

    #[test]
    fn empty_set() {
        assert_eq!(ovs_count(&new_occlusion_volume_set()), 0);
    }

    #[test]
    fn add_and_count() {
        let mut s = new_occlusion_volume_set();
        ovs_add(&mut s, make_vol(1));
        assert_eq!(ovs_count(&s), 1);
    }

    #[test]
    fn remove_by_id() {
        let mut s = new_occlusion_volume_set();
        ovs_add(&mut s, make_vol(1));
        ovs_remove(&mut s, 1);
        assert_eq!(ovs_count(&s), 0);
    }

    #[test]
    fn contains_center() {
        let v = make_vol(1);
        assert!(v.contains([0.0; 3]));
    }

    #[test]
    fn not_contains_outside() {
        let v = make_vol(1);
        assert!(!v.contains([5.0, 0.0, 0.0]));
    }

    #[test]
    fn volume_unit_cube() {
        let v = OcclusionVolume::new(1, [0.0; 3], [1.0; 3], 1.0);
        assert!((v.volume() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn occlusion_at_center() {
        let mut s = new_occlusion_volume_set();
        ovs_add(&mut s, make_vol(1));
        assert!((ovs_occlusion_at(&s, [0.0; 3]) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn occlusion_zero_outside() {
        let mut s = new_occlusion_volume_set();
        ovs_add(&mut s, make_vol(1));
        assert!(ovs_occlusion_at(&s, [10.0, 0.0, 0.0]) < 1e-6);
    }

    #[test]
    fn disabled_not_counted() {
        let mut s = new_occlusion_volume_set();
        ovs_add(&mut s, make_vol(1));
        ovs_set_enabled(&mut s, 1, false);
        assert_eq!(ovs_enabled_count(&s), 0);
    }

    #[test]
    fn json_has_count() {
        let j = ovs_to_json(&new_occlusion_volume_set());
        assert!(j.contains("count"));
    }
}
