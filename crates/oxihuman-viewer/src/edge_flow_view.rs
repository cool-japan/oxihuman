// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Edge flow visualisation — displays topology edge flow lines.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeFlowEntry {
    pub v0: u32,
    pub v1: u32,
    pub loop_id: u32,
    pub angle_rad: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct EdgeFlowView {
    pub edges: Vec<EdgeFlowEntry>,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn new_edge_flow_view() -> EdgeFlowView {
    EdgeFlowView {
        edges: Vec::new(),
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn efv_add_edge(view: &mut EdgeFlowView, v0: u32, v1: u32, loop_id: u32, angle_rad: f32) {
    view.edges.push(EdgeFlowEntry {
        v0,
        v1,
        loop_id,
        angle_rad,
    });
}

#[allow(dead_code)]
pub fn efv_clear(view: &mut EdgeFlowView) {
    view.edges.clear();
}

#[allow(dead_code)]
pub fn efv_count(view: &EdgeFlowView) -> usize {
    view.edges.len()
}

#[allow(dead_code)]
pub fn efv_is_empty(view: &EdgeFlowView) -> bool {
    view.edges.is_empty()
}

#[allow(dead_code)]
pub fn efv_set_enabled(view: &mut EdgeFlowView, v: bool) {
    view.enabled = v;
}

#[allow(dead_code)]
pub fn efv_count_by_loop(view: &EdgeFlowView, loop_id: u32) -> usize {
    view.edges.iter().filter(|e| e.loop_id == loop_id).count()
}

#[allow(dead_code)]
pub fn efv_average_angle_rad(view: &EdgeFlowView) -> f32 {
    if view.edges.is_empty() {
        return 0.0;
    }
    view.edges.iter().map(|e| e.angle_rad).sum::<f32>() / view.edges.len() as f32
}

#[allow(dead_code)]
pub fn efv_loop_ids(view: &EdgeFlowView) -> Vec<u32> {
    let mut ids: Vec<u32> = view.edges.iter().map(|e| e.loop_id).collect();
    ids.sort();
    ids.dedup();
    ids
}

#[allow(dead_code)]
pub fn efv_flow_angle_rad(view: &EdgeFlowView) -> f32 {
    (efv_average_angle_rad(view) % PI).abs()
}

#[allow(dead_code)]
pub fn efv_to_json(view: &EdgeFlowView) -> String {
    format!(
        "{{\"count\":{},\"enabled\":{}}}",
        efv_count(view),
        view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_4;
    #[test]
    fn new_is_empty() {
        assert!(efv_is_empty(&new_edge_flow_view()));
    }
    #[test]
    fn add_edge_increments_count() {
        let mut v = new_edge_flow_view();
        efv_add_edge(&mut v, 0, 1, 0, FRAC_PI_4);
        assert_eq!(efv_count(&v), 1);
    }
    #[test]
    fn clear_empties() {
        let mut v = new_edge_flow_view();
        efv_add_edge(&mut v, 0, 1, 0, 0.0);
        efv_clear(&mut v);
        assert!(efv_is_empty(&v));
    }
    #[test]
    fn set_enabled_false() {
        let mut v = new_edge_flow_view();
        efv_set_enabled(&mut v, false);
        assert!(!v.enabled);
    }
    #[test]
    fn count_by_loop() {
        let mut v = new_edge_flow_view();
        efv_add_edge(&mut v, 0, 1, 3, 0.0);
        efv_add_edge(&mut v, 1, 2, 3, 0.0);
        efv_add_edge(&mut v, 2, 3, 5, 0.0);
        assert_eq!(efv_count_by_loop(&v, 3), 2);
    }
    #[test]
    fn average_angle_empty_zero() {
        assert!(efv_average_angle_rad(&new_edge_flow_view()).abs() < 1e-6);
    }
    #[test]
    fn average_angle_one_edge() {
        let mut v = new_edge_flow_view();
        efv_add_edge(&mut v, 0, 1, 0, 1.2);
        assert!((efv_average_angle_rad(&v) - 1.2).abs() < 1e-5);
    }
    #[test]
    fn loop_ids_deduped() {
        let mut v = new_edge_flow_view();
        efv_add_edge(&mut v, 0, 1, 7, 0.0);
        efv_add_edge(&mut v, 1, 2, 7, 0.0);
        let ids = efv_loop_ids(&v);
        assert_eq!(ids.len(), 1);
    }
    #[test]
    fn flow_angle_nonneg() {
        assert!(efv_flow_angle_rad(&new_edge_flow_view()) >= 0.0);
    }
    #[test]
    fn to_json_has_enabled() {
        assert!(efv_to_json(&new_edge_flow_view()).contains("\"enabled\""));
    }
}
