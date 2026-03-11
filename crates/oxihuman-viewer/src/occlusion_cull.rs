// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Occlusion culling helpers (Hi-Z / software rasteriser stub).

/// Occlusion query result.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OcclusionResult {
    Visible,
    Occluded,
    Unknown,
}

/// Object cull state.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct OcclusionEntry {
    pub id: u32,
    pub result: OcclusionResult,
    pub depth_min: f32,
}

/// Manager.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct OcclusionCull {
    pub entries: Vec<OcclusionEntry>,
}

#[allow(dead_code)]
pub fn new_occlusion_cull() -> OcclusionCull {
    OcclusionCull::default()
}

#[allow(dead_code)]
pub fn oc_register(cull: &mut OcclusionCull, id: u32, depth_min: f32) {
    cull.entries.push(OcclusionEntry {
        id,
        result: OcclusionResult::Unknown,
        depth_min,
    });
}

#[allow(dead_code)]
pub fn oc_set_result(cull: &mut OcclusionCull, id: u32, result: OcclusionResult) {
    if let Some(e) = cull.entries.iter_mut().find(|e| e.id == id) {
        e.result = result;
    }
}

#[allow(dead_code)]
pub fn oc_visible_count(cull: &OcclusionCull) -> usize {
    cull.entries
        .iter()
        .filter(|e| e.result == OcclusionResult::Visible)
        .count()
}

#[allow(dead_code)]
pub fn oc_occluded_count(cull: &OcclusionCull) -> usize {
    cull.entries
        .iter()
        .filter(|e| e.result == OcclusionResult::Occluded)
        .count()
}

#[allow(dead_code)]
pub fn oc_clear(cull: &mut OcclusionCull) {
    cull.entries.clear();
}

#[allow(dead_code)]
pub fn oc_count(cull: &OcclusionCull) -> usize {
    cull.entries.len()
}

/// Mark everything visible (conservative).
#[allow(dead_code)]
pub fn oc_mark_all_visible(cull: &mut OcclusionCull) {
    for e in &mut cull.entries {
        e.result = OcclusionResult::Visible;
    }
}

#[allow(dead_code)]
pub fn oc_depth_cull(cull: &mut OcclusionCull, depth_threshold: f32) {
    for e in &mut cull.entries {
        if e.depth_min > depth_threshold {
            e.result = OcclusionResult::Occluded;
        } else {
            e.result = OcclusionResult::Visible;
        }
    }
}

#[allow(dead_code)]
pub fn oc_to_json(cull: &OcclusionCull) -> String {
    format!(
        "{{\"total\":{},\"visible\":{},\"occluded\":{}}}",
        oc_count(cull),
        oc_visible_count(cull),
        oc_occluded_count(cull)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        assert_eq!(oc_count(&new_occlusion_cull()), 0);
    }

    #[test]
    fn register() {
        let mut c = new_occlusion_cull();
        oc_register(&mut c, 1, 0.5);
        assert_eq!(oc_count(&c), 1);
    }

    #[test]
    fn default_unknown() {
        let mut c = new_occlusion_cull();
        oc_register(&mut c, 1, 0.5);
        assert_eq!(oc_visible_count(&c), 0);
        assert_eq!(oc_occluded_count(&c), 0);
    }

    #[test]
    fn set_visible() {
        let mut c = new_occlusion_cull();
        oc_register(&mut c, 1, 0.5);
        oc_set_result(&mut c, 1, OcclusionResult::Visible);
        assert_eq!(oc_visible_count(&c), 1);
    }

    #[test]
    fn set_occluded() {
        let mut c = new_occlusion_cull();
        oc_register(&mut c, 1, 0.5);
        oc_set_result(&mut c, 1, OcclusionResult::Occluded);
        assert_eq!(oc_occluded_count(&c), 1);
    }

    #[test]
    fn clear_empty() {
        let mut c = new_occlusion_cull();
        oc_register(&mut c, 1, 0.5);
        oc_clear(&mut c);
        assert_eq!(oc_count(&c), 0);
    }

    #[test]
    fn mark_all_visible() {
        let mut c = new_occlusion_cull();
        oc_register(&mut c, 1, 0.2);
        oc_register(&mut c, 2, 0.9);
        oc_mark_all_visible(&mut c);
        assert_eq!(oc_visible_count(&c), 2);
    }

    #[test]
    fn depth_cull() {
        let mut c = new_occlusion_cull();
        oc_register(&mut c, 1, 0.2);
        oc_register(&mut c, 2, 0.9);
        oc_depth_cull(&mut c, 0.5);
        assert_eq!(oc_visible_count(&c), 1);
        assert_eq!(oc_occluded_count(&c), 1);
    }

    #[test]
    fn json_has_visible() {
        assert!(oc_to_json(&new_occlusion_cull()).contains("visible"));
    }

    #[test]
    fn two_objects_depth_cull_boundary() {
        let mut c = new_occlusion_cull();
        oc_register(&mut c, 1, 0.5);
        oc_depth_cull(&mut c, 0.5);
        assert_eq!(oc_visible_count(&c), 1);
    }
}
