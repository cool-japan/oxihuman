// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Draw-call grouping by material / pipeline key.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawGroupEntry {
    pub group_id: u32,
    pub draw_id: u32,
    pub sort_key: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DrawGroupManager {
    pub entries: Vec<DrawGroupEntry>,
}

#[allow(dead_code)]
pub fn new_draw_group_manager() -> DrawGroupManager {
    DrawGroupManager::default()
}

#[allow(dead_code)]
pub fn dg_add(mgr: &mut DrawGroupManager, group_id: u32, draw_id: u32, sort_key: u64) {
    mgr.entries.push(DrawGroupEntry {
        group_id,
        draw_id,
        sort_key,
    });
}

#[allow(dead_code)]
pub fn dg_clear(mgr: &mut DrawGroupManager) {
    mgr.entries.clear();
}

#[allow(dead_code)]
pub fn dg_count(mgr: &DrawGroupManager) -> usize {
    mgr.entries.len()
}

#[allow(dead_code)]
pub fn dg_is_empty(mgr: &DrawGroupManager) -> bool {
    mgr.entries.is_empty()
}

#[allow(dead_code)]
pub fn dg_count_by_group(mgr: &DrawGroupManager, group_id: u32) -> usize {
    mgr.entries
        .iter()
        .filter(|e| e.group_id == group_id)
        .count()
}

#[allow(dead_code)]
pub fn dg_sort_by_key(mgr: &mut DrawGroupManager) {
    mgr.entries.sort_by_key(|e| e.sort_key);
}

#[allow(dead_code)]
pub fn dg_group_ids(mgr: &DrawGroupManager) -> Vec<u32> {
    let mut ids: Vec<u32> = mgr.entries.iter().map(|e| e.group_id).collect();
    ids.sort();
    ids.dedup();
    ids
}

#[allow(dead_code)]
pub fn dg_average_sort_key(mgr: &DrawGroupManager) -> f64 {
    if mgr.entries.is_empty() {
        return 0.0;
    }
    mgr.entries.iter().map(|e| e.sort_key as f64).sum::<f64>() / mgr.entries.len() as f64
}

#[allow(dead_code)]
pub fn dg_key_angle_rad(mgr: &DrawGroupManager) -> f32 {
    let a = dg_average_sort_key(mgr) as f32;
    if a > 0.0 {
        (1.0 / a).atan().min(FRAC_PI_4)
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn dg_to_json(mgr: &DrawGroupManager) -> String {
    format!(
        "{{\"count\":{},\"groups\":{}}}",
        dg_count(mgr),
        dg_group_ids(mgr).len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_is_empty() {
        assert!(dg_is_empty(&new_draw_group_manager()));
    }
    #[test]
    fn add_increments_count() {
        let mut m = new_draw_group_manager();
        dg_add(&mut m, 0, 1, 100);
        assert_eq!(dg_count(&m), 1);
    }
    #[test]
    fn clear_empties() {
        let mut m = new_draw_group_manager();
        dg_add(&mut m, 0, 1, 100);
        dg_clear(&mut m);
        assert!(dg_is_empty(&m));
    }
    #[test]
    fn count_by_group() {
        let mut m = new_draw_group_manager();
        dg_add(&mut m, 1, 0, 1);
        dg_add(&mut m, 1, 1, 2);
        dg_add(&mut m, 2, 2, 3);
        assert_eq!(dg_count_by_group(&m, 1), 2);
    }
    #[test]
    fn sort_by_key_orders() {
        let mut m = new_draw_group_manager();
        dg_add(&mut m, 0, 0, 300);
        dg_add(&mut m, 0, 1, 100);
        dg_sort_by_key(&mut m);
        assert!(m.entries[0].sort_key <= m.entries[1].sort_key);
    }
    #[test]
    fn group_ids_deduped() {
        let mut m = new_draw_group_manager();
        dg_add(&mut m, 5, 0, 1);
        dg_add(&mut m, 5, 1, 2);
        let ids = dg_group_ids(&m);
        assert_eq!(ids.len(), 1);
    }
    #[test]
    fn average_sort_key_zero_when_empty() {
        assert!(dg_average_sort_key(&new_draw_group_manager()).abs() < 1e-9);
    }
    #[test]
    fn average_sort_key_correct() {
        let mut m = new_draw_group_manager();
        dg_add(&mut m, 0, 0, 100);
        dg_add(&mut m, 0, 1, 200);
        assert!((dg_average_sort_key(&m) - 150.0).abs() < 1e-9);
    }
    #[test]
    fn key_angle_nonneg() {
        assert!(dg_key_angle_rad(&new_draw_group_manager()) >= 0.0);
    }
    #[test]
    fn to_json_has_count() {
        assert!(dg_to_json(&new_draw_group_manager()).contains("\"count\""));
    }
}
