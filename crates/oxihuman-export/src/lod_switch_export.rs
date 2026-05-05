// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export LOD switch distances and mesh references.

/// A single LOD switch entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodSwitchEntry {
    pub level: u32,
    pub switch_in_distance: f32,
    pub switch_out_distance: f32,
    pub mesh_ref: String,
    pub triangle_count: u32,
}

/// LOD switch export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodSwitchExport {
    pub entries: Vec<LodSwitchEntry>,
    pub hysteresis: f32,
}

/// Create a new LOD switch export.
#[allow(dead_code)]
pub fn new_lod_switch_export(hysteresis: f32) -> LodSwitchExport {
    LodSwitchExport {
        entries: Vec::new(),
        hysteresis,
    }
}

/// Add an entry.
#[allow(dead_code)]
pub fn add_lod_switch(export: &mut LodSwitchExport, entry: LodSwitchEntry) {
    export.entries.push(entry);
    export.entries.sort_by_key(|a| a.level);
}

/// Count entries.
#[allow(dead_code)]
pub fn lod_switch_count(export: &LodSwitchExport) -> usize {
    export.entries.len()
}

/// Select the active LOD level for a given camera distance.
#[allow(dead_code)]
pub fn active_lod_for_distance(export: &LodSwitchExport, distance: f32) -> Option<u32> {
    export
        .entries
        .iter()
        .filter(|e| distance >= e.switch_in_distance && distance < e.switch_out_distance)
        .map(|e| e.level)
        .next()
}

/// Validate entries have non-overlapping distance ranges.
#[allow(dead_code)]
pub fn validate_lod_switch(export: &LodSwitchExport) -> bool {
    export
        .entries
        .iter()
        .all(|e| e.switch_out_distance > e.switch_in_distance)
}

/// Total triangle count across all levels.
#[allow(dead_code)]
pub fn total_triangle_count_ls(export: &LodSwitchExport) -> u32 {
    export.entries.iter().map(|e| e.triangle_count).sum()
}

/// Reduction ratio from level 0 to the last level.
#[allow(dead_code)]
pub fn lod_switch_reduction_ratio(export: &LodSwitchExport) -> f32 {
    if export.entries.len() < 2 {
        return 0.0;
    }
    let first = export.entries[0].triangle_count;
    let last = export.entries.last().map_or(0, |e| e.triangle_count);
    if first == 0 {
        return 0.0;
    }
    1.0 - last as f32 / first as f32
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn lod_switch_to_json(export: &LodSwitchExport) -> String {
    format!(
        "{{\"level_count\":{},\"hysteresis\":{:.4}}}",
        export.entries.len(),
        export.hysteresis
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(level: u32, d_in: f32, d_out: f32) -> LodSwitchEntry {
        LodSwitchEntry {
            level,
            switch_in_distance: d_in,
            switch_out_distance: d_out,
            mesh_ref: format!("lod{level}"),
            triangle_count: 1000 / (level + 1),
        }
    }

    fn two_level_export() -> LodSwitchExport {
        let mut e = new_lod_switch_export(0.1);
        add_lod_switch(&mut e, sample_entry(0, 0.0, 50.0));
        add_lod_switch(&mut e, sample_entry(1, 50.0, 200.0));
        e
    }

    #[test]
    fn test_level_count() {
        assert_eq!(lod_switch_count(&two_level_export()), 2);
    }

    #[test]
    fn test_active_lod_close() {
        let e = two_level_export();
        assert_eq!(active_lod_for_distance(&e, 10.0), Some(0));
    }

    #[test]
    fn test_active_lod_far() {
        let e = two_level_export();
        assert_eq!(active_lod_for_distance(&e, 100.0), Some(1));
    }

    #[test]
    fn test_active_lod_none() {
        let e = two_level_export();
        assert_eq!(active_lod_for_distance(&e, 300.0), None);
    }

    #[test]
    fn test_validate_valid() {
        assert!(validate_lod_switch(&two_level_export()));
    }

    #[test]
    fn test_validate_invalid() {
        let mut e = new_lod_switch_export(0.1);
        e.entries.push(LodSwitchEntry {
            level: 0,
            switch_in_distance: 50.0,
            switch_out_distance: 10.0,
            mesh_ref: "x".to_string(),
            triangle_count: 100,
        });
        assert!(!validate_lod_switch(&e));
    }

    #[test]
    fn test_total_triangles() {
        let e = two_level_export();
        assert!(total_triangle_count_ls(&e) > 0);
    }

    #[test]
    fn test_reduction_ratio_positive() {
        let e = two_level_export();
        assert!(lod_switch_reduction_ratio(&e) > 0.0);
    }

    #[test]
    fn test_lod_switch_to_json() {
        let e = two_level_export();
        let j = lod_switch_to_json(&e);
        assert!(j.contains("level_count"));
    }

    #[test]
    fn test_empty_export() {
        let e = new_lod_switch_export(0.1);
        assert_eq!(lod_switch_count(&e), 0);
    }
}
