// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Occlusion probe — directional ambient occlusion probe sampling.

/// An occlusion probe at a world position.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OcclusionProbe {
    pub position: [f32; 3],
    /// Occlusion per hemisphere direction (6 faces of a cube).
    pub occlusion: [f32; 6],
    pub enabled: bool,
}

/// Manager for occlusion probes.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct OcclusionProbeSet {
    pub probes: Vec<OcclusionProbe>,
}

#[allow(dead_code)]
pub fn new_occlusion_probe(position: [f32; 3]) -> OcclusionProbe {
    OcclusionProbe {
        position,
        occlusion: [1.0; 6],
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn op_set_occlusion(probe: &mut OcclusionProbe, face: usize, v: f32) {
    if face < 6 {
        probe.occlusion[face] = v.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn op_average_occlusion(probe: &OcclusionProbe) -> f32 {
    probe.occlusion.iter().sum::<f32>() / 6.0
}

#[allow(dead_code)]
pub fn op_set_enabled(probe: &mut OcclusionProbe, enabled: bool) {
    probe.enabled = enabled;
}

#[allow(dead_code)]
pub fn ops_add(set: &mut OcclusionProbeSet, probe: OcclusionProbe) {
    set.probes.push(probe);
}

#[allow(dead_code)]
pub fn ops_probe_count(set: &OcclusionProbeSet) -> usize {
    set.probes.len()
}

#[allow(dead_code)]
pub fn ops_enabled_count(set: &OcclusionProbeSet) -> usize {
    set.probes.iter().filter(|p| p.enabled).count()
}

#[allow(dead_code)]
pub fn ops_clear(set: &mut OcclusionProbeSet) {
    set.probes.clear();
}

#[allow(dead_code)]
pub fn ops_nearest_probe(set: &OcclusionProbeSet, pos: [f32; 3]) -> Option<&OcclusionProbe> {
    set.probes.iter().min_by(|a, b| {
        let da = dist_sq(a.position, pos);
        let db = dist_sq(b.position, pos);
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

#[allow(dead_code)]
pub fn ops_to_json(set: &OcclusionProbeSet) -> String {
    format!(
        r#"{{"probe_count":{},"enabled_count":{}}}"#,
        ops_probe_count(set),
        ops_enabled_count(set)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_probe_full_occlusion() {
        let p = new_occlusion_probe([0.0, 0.0, 0.0]);
        assert!((op_average_occlusion(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_occlusion_face() {
        let mut p = new_occlusion_probe([0.0; 3]);
        op_set_occlusion(&mut p, 0, 0.5);
        assert!((p.occlusion[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn set_occlusion_clamps() {
        let mut p = new_occlusion_probe([0.0; 3]);
        op_set_occlusion(&mut p, 1, 2.0);
        assert!((p.occlusion[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_enabled() {
        let mut p = new_occlusion_probe([0.0; 3]);
        op_set_enabled(&mut p, false);
        assert!(!p.enabled);
    }

    #[test]
    fn set_empty() {
        let set = OcclusionProbeSet::default();
        assert_eq!(ops_probe_count(&set), 0);
    }

    #[test]
    fn add_probe() {
        let mut set = OcclusionProbeSet::default();
        ops_add(&mut set, new_occlusion_probe([0.0; 3]));
        assert_eq!(ops_probe_count(&set), 1);
    }

    #[test]
    fn enabled_count() {
        let mut set = OcclusionProbeSet::default();
        let mut p = new_occlusion_probe([0.0; 3]);
        op_set_enabled(&mut p, false);
        ops_add(&mut set, p);
        ops_add(&mut set, new_occlusion_probe([1.0, 0.0, 0.0]));
        assert_eq!(ops_enabled_count(&set), 1);
    }

    #[test]
    fn nearest_probe() {
        let mut set = OcclusionProbeSet::default();
        ops_add(&mut set, new_occlusion_probe([0.0, 0.0, 0.0]));
        ops_add(&mut set, new_occlusion_probe([10.0, 0.0, 0.0]));
        let nearest = ops_nearest_probe(&set, [0.5, 0.0, 0.0]).unwrap();
        assert!((nearest.position[0]).abs() < 1e-6);
    }

    #[test]
    fn to_json_fields() {
        let set = OcclusionProbeSet::default();
        let j = ops_to_json(&set);
        assert!(j.contains("probe_count"));
    }
}
