// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Outliner filter panel view.

/// Filter mode for the outliner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlinerFilterMode {
    All,
    Selected,
    Visible,
    Custom,
}

/// An outliner filter entry.
#[derive(Debug, Clone)]
pub struct OutlinerFilterEntry {
    pub name: String,
    pub active: bool,
}

/// State for the outliner filter panel.
#[derive(Debug, Clone)]
pub struct OutlinerFilterView {
    pub mode: OutlinerFilterMode,
    pub search_text: String,
    pub entries: Vec<OutlinerFilterEntry>,
    pub enabled: bool,
}

/// Create a new outliner filter view.
pub fn new_outliner_filter_view() -> OutlinerFilterView {
    OutlinerFilterView {
        mode: OutlinerFilterMode::All,
        search_text: String::new(),
        entries: Vec::new(),
        enabled: true,
    }
}

/// Set the search text.
pub fn ofv_set_search(v: &mut OutlinerFilterView, text: &str) {
    v.search_text = text.to_string();
}

/// Set filter mode.
pub fn ofv_set_mode(v: &mut OutlinerFilterView, mode: OutlinerFilterMode) {
    v.mode = mode;
}

/// Add a filter entry.
pub fn ofv_add_entry(v: &mut OutlinerFilterView, name: &str) {
    v.entries.push(OutlinerFilterEntry {
        name: name.to_string(),
        active: true,
    });
}

/// Count active filter entries.
pub fn ofv_active_count(v: &OutlinerFilterView) -> usize {
    v.entries.iter().filter(|e| e.active).count()
}

/// Serialise to JSON.
pub fn ofv_to_json(v: &OutlinerFilterView) -> String {
    format!(
        r#"{{"mode":"{:?}","search":"{}","entry_count":{},"enabled":{}}}"#,
        v.mode,
        v.search_text,
        v.entries.len(),
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_all_mode() {
        let v = new_outliner_filter_view();
        assert_eq!(v.mode, OutlinerFilterMode::All /* default mode */);
    }

    #[test]
    fn set_search() {
        let mut v = new_outliner_filter_view();
        ofv_set_search(&mut v, "Armature");
        assert_eq!(v.search_text, "Armature" /* search set */);
    }

    #[test]
    fn set_mode_selected() {
        let mut v = new_outliner_filter_view();
        ofv_set_mode(&mut v, OutlinerFilterMode::Selected);
        assert_eq!(v.mode, OutlinerFilterMode::Selected /* mode set */);
    }

    #[test]
    fn add_entry() {
        let mut v = new_outliner_filter_view();
        ofv_add_entry(&mut v, "Meshes");
        assert_eq!(v.entries.len(), 1 /* one entry */);
    }

    #[test]
    fn active_count() {
        let mut v = new_outliner_filter_view();
        ofv_add_entry(&mut v, "A");
        ofv_add_entry(&mut v, "B");
        v.entries[0].active = false;
        assert_eq!(ofv_active_count(&v), 1 /* one active */);
    }

    #[test]
    fn json_has_mode() {
        let v = new_outliner_filter_view();
        assert!(ofv_to_json(&v).contains("mode") /* json has mode */);
    }

    #[test]
    fn empty_search_by_default() {
        let v = new_outliner_filter_view();
        assert!(v.search_text.is_empty() /* empty search */);
    }

    #[test]
    fn enabled_default() {
        let v = new_outliner_filter_view();
        assert!(v.enabled /* enabled */);
    }
}
