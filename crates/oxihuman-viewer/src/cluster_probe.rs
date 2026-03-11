// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cluster probe — clustered light probes for local reflections and GI.

/// A single cluster probe volume.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClusterProbe {
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
    pub intensity: f32,
    pub enabled: bool,
}

/// Cluster probe collection.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ClusterProbeSet {
    pub probes: Vec<ClusterProbe>,
}

#[allow(dead_code)]
pub fn new_cluster_probe(center: [f32; 3], half_extents: [f32; 3]) -> ClusterProbe {
    ClusterProbe {
        center,
        half_extents,
        intensity: 1.0,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn cpr_add(set: &mut ClusterProbeSet, probe: ClusterProbe) {
    set.probes.push(probe);
}

#[allow(dead_code)]
pub fn cpr_remove(set: &mut ClusterProbeSet, index: usize) {
    if index < set.probes.len() {
        set.probes.remove(index);
    }
}

#[allow(dead_code)]
pub fn cpr_count(set: &ClusterProbeSet) -> usize {
    set.probes.len()
}

#[allow(dead_code)]
pub fn cpr_enabled_count(set: &ClusterProbeSet) -> usize {
    set.probes.iter().filter(|p| p.enabled).count()
}

#[allow(dead_code)]
pub fn cpr_contains_point(probe: &ClusterProbe, point: [f32; 3]) -> bool {
    let dx = (point[0] - probe.center[0]).abs();
    let dy = (point[1] - probe.center[1]).abs();
    let dz = (point[2] - probe.center[2]).abs();
    dx <= probe.half_extents[0] && dy <= probe.half_extents[1] && dz <= probe.half_extents[2]
}

#[allow(dead_code)]
pub fn cpr_find_for_point(set: &ClusterProbeSet, point: [f32; 3]) -> Option<usize> {
    set.probes
        .iter()
        .enumerate()
        .find(|(_, p)| p.enabled && cpr_contains_point(p, point))
        .map(|(i, _)| i)
}

#[allow(dead_code)]
pub fn cpr_set_intensity(probe: &mut ClusterProbe, v: f32) {
    probe.intensity = v.clamp(0.0, 10.0);
}

#[allow(dead_code)]
pub fn cpr_set_enabled(probe: &mut ClusterProbe, enabled: bool) {
    probe.enabled = enabled;
}

#[allow(dead_code)]
pub fn cpr_volume(probe: &ClusterProbe) -> f32 {
    8.0 * probe.half_extents[0] * probe.half_extents[1] * probe.half_extents[2]
}

#[allow(dead_code)]
pub fn cpr_clear(set: &mut ClusterProbeSet) {
    set.probes.clear();
}

#[allow(dead_code)]
pub fn cpr_to_json(set: &ClusterProbeSet) -> String {
    format!(
        r#"{{"count":{},"enabled":{}}}"#,
        cpr_count(set),
        cpr_enabled_count(set)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_probe_enabled() {
        let p = new_cluster_probe([0.0; 3], [1.0, 1.0, 1.0]);
        assert!(p.enabled);
    }

    #[test]
    fn add_and_count() {
        let mut set = ClusterProbeSet::default();
        cpr_add(&mut set, new_cluster_probe([0.0; 3], [1.0; 3]));
        cpr_add(&mut set, new_cluster_probe([5.0, 0.0, 0.0], [1.0; 3]));
        assert_eq!(cpr_count(&set), 2);
    }

    #[test]
    fn remove_probe() {
        let mut set = ClusterProbeSet::default();
        cpr_add(&mut set, new_cluster_probe([0.0; 3], [1.0; 3]));
        cpr_remove(&mut set, 0);
        assert_eq!(cpr_count(&set), 0);
    }

    #[test]
    fn contains_point_inside() {
        let p = new_cluster_probe([0.0; 3], [1.0, 1.0, 1.0]);
        assert!(cpr_contains_point(&p, [0.5, 0.5, 0.5]));
    }

    #[test]
    fn contains_point_outside() {
        let p = new_cluster_probe([0.0; 3], [1.0, 1.0, 1.0]);
        assert!(!cpr_contains_point(&p, [2.0, 0.0, 0.0]));
    }

    #[test]
    fn find_for_point() {
        let mut set = ClusterProbeSet::default();
        cpr_add(&mut set, new_cluster_probe([0.0; 3], [1.0; 3]));
        assert!(cpr_find_for_point(&set, [0.0, 0.0, 0.0]).is_some());
        assert!(cpr_find_for_point(&set, [5.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn set_intensity_clamps() {
        let mut p = new_cluster_probe([0.0; 3], [1.0; 3]);
        cpr_set_intensity(&mut p, 100.0);
        assert!((p.intensity - 10.0).abs() < 1e-6);
    }

    #[test]
    fn enabled_count() {
        let mut set = ClusterProbeSet::default();
        let mut p = new_cluster_probe([0.0; 3], [1.0; 3]);
        cpr_set_enabled(&mut p, false);
        cpr_add(&mut set, p);
        cpr_add(&mut set, new_cluster_probe([5.0; 3], [1.0; 3]));
        assert_eq!(cpr_enabled_count(&set), 1);
    }

    #[test]
    fn volume_calculation() {
        let p = new_cluster_probe([0.0; 3], [1.0, 2.0, 3.0]);
        assert!((cpr_volume(&p) - 48.0).abs() < 1e-5);
    }

    #[test]
    fn clear_set() {
        let mut set = ClusterProbeSet::default();
        cpr_add(&mut set, new_cluster_probe([0.0; 3], [1.0; 3]));
        cpr_clear(&mut set);
        assert_eq!(cpr_count(&set), 0);
    }
}
