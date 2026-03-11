// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Occlusion query v2 — async occlusion query result management.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OqState {
    Pending,
    Visible,
    Occluded,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OcclusionQueryV2Entry {
    pub id: u32,
    pub state: OqState,
    pub sample_count: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct OcclusionQueryV2Manager {
    pub entries: Vec<OcclusionQueryV2Entry>,
}

#[allow(dead_code)]
pub fn new_occlusion_query_v2() -> OcclusionQueryV2Manager {
    OcclusionQueryV2Manager::default()
}

#[allow(dead_code)]
pub fn oqv2_register(mgr: &mut OcclusionQueryV2Manager, id: u32) {
    mgr.entries.push(OcclusionQueryV2Entry {
        id,
        state: OqState::Pending,
        sample_count: 0,
    });
}

#[allow(dead_code)]
pub fn oqv2_resolve(mgr: &mut OcclusionQueryV2Manager, id: u32, sample_count: u64) {
    if let Some(e) = mgr.entries.iter_mut().find(|e| e.id == id) {
        e.sample_count = sample_count;
        e.state = if sample_count > 0 {
            OqState::Visible
        } else {
            OqState::Occluded
        };
    }
}

#[allow(dead_code)]
pub fn oqv2_clear(mgr: &mut OcclusionQueryV2Manager) {
    mgr.entries.clear();
}

#[allow(dead_code)]
pub fn oqv2_count(mgr: &OcclusionQueryV2Manager) -> usize {
    mgr.entries.len()
}

#[allow(dead_code)]
pub fn oqv2_visible_count(mgr: &OcclusionQueryV2Manager) -> usize {
    mgr.entries
        .iter()
        .filter(|e| e.state == OqState::Visible)
        .count()
}

#[allow(dead_code)]
pub fn oqv2_occluded_count(mgr: &OcclusionQueryV2Manager) -> usize {
    mgr.entries
        .iter()
        .filter(|e| e.state == OqState::Occluded)
        .count()
}

#[allow(dead_code)]
pub fn oqv2_pending_count(mgr: &OcclusionQueryV2Manager) -> usize {
    mgr.entries
        .iter()
        .filter(|e| e.state == OqState::Pending)
        .count()
}

#[allow(dead_code)]
pub fn oqv2_average_samples(mgr: &OcclusionQueryV2Manager) -> f64 {
    if mgr.entries.is_empty() {
        return 0.0;
    }
    mgr.entries
        .iter()
        .map(|e| e.sample_count as f64)
        .sum::<f64>()
        / mgr.entries.len() as f64
}

#[allow(dead_code)]
pub fn oqv2_visibility_ratio(mgr: &OcclusionQueryV2Manager) -> f32 {
    let n = oqv2_count(mgr);
    if n == 0 {
        return 0.0;
    }
    oqv2_visible_count(mgr) as f32 / n as f32
}

#[allow(dead_code)]
pub fn oqv2_angle_rad(mgr: &OcclusionQueryV2Manager) -> f32 {
    oqv2_visibility_ratio(mgr) * FRAC_PI_4
}

#[allow(dead_code)]
pub fn oqv2_to_json(mgr: &OcclusionQueryV2Manager) -> String {
    format!(
        "{{\"count\":{},\"visible\":{},\"occluded\":{}}}",
        oqv2_count(mgr),
        oqv2_visible_count(mgr),
        oqv2_occluded_count(mgr)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_is_empty() {
        assert_eq!(oqv2_count(&new_occlusion_query_v2()), 0);
    }
    #[test]
    fn register_increments_count() {
        let mut m = new_occlusion_query_v2();
        oqv2_register(&mut m, 0);
        assert_eq!(oqv2_count(&m), 1);
    }
    #[test]
    fn clear_empties() {
        let mut m = new_occlusion_query_v2();
        oqv2_register(&mut m, 0);
        oqv2_clear(&mut m);
        assert_eq!(oqv2_count(&m), 0);
    }
    #[test]
    fn pending_after_register() {
        let mut m = new_occlusion_query_v2();
        oqv2_register(&mut m, 0);
        assert_eq!(oqv2_pending_count(&m), 1);
    }
    #[test]
    fn resolve_visible() {
        let mut m = new_occlusion_query_v2();
        oqv2_register(&mut m, 0);
        oqv2_resolve(&mut m, 0, 100);
        assert_eq!(oqv2_visible_count(&m), 1);
    }
    #[test]
    fn resolve_occluded() {
        let mut m = new_occlusion_query_v2();
        oqv2_register(&mut m, 0);
        oqv2_resolve(&mut m, 0, 0);
        assert_eq!(oqv2_occluded_count(&m), 1);
    }
    #[test]
    fn visibility_ratio_correct() {
        let mut m = new_occlusion_query_v2();
        oqv2_register(&mut m, 0);
        oqv2_resolve(&mut m, 0, 10);
        assert!((oqv2_visibility_ratio(&m) - 1.0).abs() < 1e-5);
    }
    #[test]
    fn average_samples_empty_zero() {
        assert!(oqv2_average_samples(&new_occlusion_query_v2()).abs() < 1e-9);
    }
    #[test]
    fn angle_nonneg() {
        assert!(oqv2_angle_rad(&new_occlusion_query_v2()) >= 0.0);
    }
    #[test]
    fn to_json_has_visible() {
        assert!(oqv2_to_json(&new_occlusion_query_v2()).contains("\"visible\""));
    }
}
