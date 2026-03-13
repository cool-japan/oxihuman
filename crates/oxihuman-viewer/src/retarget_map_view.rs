// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Retarget bone mapping view stub.

/// Mapping match quality.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MappingQuality {
    Exact,
    Good,
    Approximate,
    Unmapped,
}

/// A bone retarget mapping entry.
#[derive(Debug, Clone)]
pub struct RetargetBoneMapping {
    pub source_bone: String,
    pub target_bone: String,
    pub quality: MappingQuality,
    pub weight: f32,
}

/// Retarget map view configuration.
#[derive(Debug, Clone)]
pub struct RetargetMapView {
    pub mappings: Vec<RetargetBoneMapping>,
    pub show_unmapped: bool,
    pub highlight_errors: bool,
    pub enabled: bool,
}

impl RetargetMapView {
    pub fn new() -> Self {
        RetargetMapView {
            mappings: Vec::new(),
            show_unmapped: true,
            highlight_errors: true,
            enabled: true,
        }
    }
}

impl Default for RetargetMapView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new retarget map view.
pub fn new_retarget_map_view() -> RetargetMapView {
    RetargetMapView::new()
}

/// Add a mapping entry.
pub fn rmv_add_mapping(view: &mut RetargetMapView, mapping: RetargetBoneMapping) {
    view.mappings.push(mapping);
}

/// Clear all mappings.
pub fn rmv_clear(view: &mut RetargetMapView) {
    view.mappings.clear();
}

/// Toggle unmapped bone display.
pub fn rmv_show_unmapped(view: &mut RetargetMapView, show: bool) {
    view.show_unmapped = show;
}

/// Toggle error highlighting.
pub fn rmv_highlight_errors(view: &mut RetargetMapView, highlight: bool) {
    view.highlight_errors = highlight;
}

/// Enable or disable.
pub fn rmv_set_enabled(view: &mut RetargetMapView, enabled: bool) {
    view.enabled = enabled;
}

/// Return mapping count.
pub fn rmv_mapping_count(view: &RetargetMapView) -> usize {
    view.mappings.len()
}

/// Return count of unmapped entries.
pub fn rmv_unmapped_count(view: &RetargetMapView) -> usize {
    view.mappings
        .iter()
        .filter(|m| m.quality == MappingQuality::Unmapped)
        .count()
}

/// Serialize to JSON-like string.
pub fn rmv_to_json(view: &RetargetMapView) -> String {
    format!(
        r#"{{"mapping_count":{},"show_unmapped":{},"highlight_errors":{},"enabled":{}}}"#,
        view.mappings.len(),
        view.show_unmapped,
        view.highlight_errors,
        view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mapping(quality: MappingQuality) -> RetargetBoneMapping {
        RetargetBoneMapping {
            source_bone: "hip".to_string(),
            target_bone: "pelvis".to_string(),
            quality,
            weight: 1.0,
        }
    }

    #[test]
    fn test_initial_empty() {
        let v = new_retarget_map_view();
        assert_eq!(rmv_mapping_count(&v), 0 /* no mappings initially */);
    }

    #[test]
    fn test_add_mapping() {
        let mut v = new_retarget_map_view();
        rmv_add_mapping(&mut v, make_mapping(MappingQuality::Exact));
        assert_eq!(rmv_mapping_count(&v), 1 /* one mapping after add */);
    }

    #[test]
    fn test_clear() {
        let mut v = new_retarget_map_view();
        rmv_add_mapping(&mut v, make_mapping(MappingQuality::Good));
        rmv_clear(&mut v);
        assert_eq!(rmv_mapping_count(&v), 0 /* cleared */);
    }

    #[test]
    fn test_unmapped_count() {
        let mut v = new_retarget_map_view();
        rmv_add_mapping(&mut v, make_mapping(MappingQuality::Exact));
        rmv_add_mapping(&mut v, make_mapping(MappingQuality::Unmapped));
        assert_eq!(rmv_unmapped_count(&v), 1 /* one unmapped */);
    }

    #[test]
    fn test_show_unmapped() {
        let mut v = new_retarget_map_view();
        rmv_show_unmapped(&mut v, false);
        assert!(!v.show_unmapped /* unmapped must be hidden */);
    }

    #[test]
    fn test_highlight_errors() {
        let mut v = new_retarget_map_view();
        rmv_highlight_errors(&mut v, false);
        assert!(!v.highlight_errors /* error highlight must be off */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_retarget_map_view();
        rmv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_mapping_count() {
        let v = new_retarget_map_view();
        let j = rmv_to_json(&v);
        assert!(j.contains("\"mapping_count\"") /* JSON must have mapping_count */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_retarget_map_view();
        assert!(v.enabled /* must be enabled by default */);
    }
}
