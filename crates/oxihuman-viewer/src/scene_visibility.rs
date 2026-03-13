// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// Tracks visibility of scene nodes by name.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneVisibility {
    pub nodes: HashMap<String, bool>,
}

/// Create a new visibility tracker.
#[allow(dead_code)]
pub fn new_scene_visibility() -> SceneVisibility {
    SceneVisibility {
        nodes: HashMap::new(),
    }
}

/// Set the visibility of a node.
#[allow(dead_code)]
pub fn set_node_visible(sv: &mut SceneVisibility, name: &str, visible: bool) {
    sv.nodes.insert(name.to_string(), visible);
}

/// Check if a node is visible (defaults to true if not tracked).
#[allow(dead_code)]
pub fn is_node_visible(sv: &SceneVisibility, name: &str) -> bool {
    sv.nodes.get(name).copied().unwrap_or(true)
}

/// Return the number of visible nodes.
#[allow(dead_code)]
pub fn visible_count_sv(sv: &SceneVisibility) -> usize {
    sv.nodes.values().filter(|v| **v).count()
}

/// Return the number of hidden nodes.
#[allow(dead_code)]
pub fn hidden_count(sv: &SceneVisibility) -> usize {
    sv.nodes.values().filter(|v| !**v).count()
}

/// Toggle the visibility of a node.
#[allow(dead_code)]
pub fn toggle_visibility(sv: &mut SceneVisibility, name: &str) {
    let current = sv.nodes.get(name).copied().unwrap_or(true);
    sv.nodes.insert(name.to_string(), !current);
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn visibility_to_json(sv: &SceneVisibility) -> String {
    let mut entries: Vec<String> = sv
        .nodes
        .iter()
        .map(|(k, v)| format!("\"{}\":{}", k, v))
        .collect();
    entries.sort();
    format!("{{{}}}", entries.join(","))
}

/// Reset all nodes to visible.
#[allow(dead_code)]
pub fn reset_visibility(sv: &mut SceneVisibility) {
    for v in sv.nodes.values_mut() {
        *v = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let sv = new_scene_visibility();
        assert_eq!(visible_count_sv(&sv), 0);
    }

    #[test]
    fn set_and_check() {
        let mut sv = new_scene_visibility();
        set_node_visible(&mut sv, "cube", true);
        assert!(is_node_visible(&sv, "cube"));
    }

    #[test]
    fn hide_node() {
        let mut sv = new_scene_visibility();
        set_node_visible(&mut sv, "cube", false);
        assert!(!is_node_visible(&sv, "cube"));
    }

    #[test]
    fn default_visible() {
        let sv = new_scene_visibility();
        assert!(is_node_visible(&sv, "unknown"));
    }

    #[test]
    fn visible_count() {
        let mut sv = new_scene_visibility();
        set_node_visible(&mut sv, "a", true);
        set_node_visible(&mut sv, "b", false);
        assert_eq!(visible_count_sv(&sv), 1);
    }

    #[test]
    fn hidden_count_check() {
        let mut sv = new_scene_visibility();
        set_node_visible(&mut sv, "a", false);
        set_node_visible(&mut sv, "b", false);
        assert_eq!(hidden_count(&sv), 2);
    }

    #[test]
    fn toggle() {
        let mut sv = new_scene_visibility();
        set_node_visible(&mut sv, "x", true);
        toggle_visibility(&mut sv, "x");
        assert!(!is_node_visible(&sv, "x"));
    }

    #[test]
    fn reset_all_visible() {
        let mut sv = new_scene_visibility();
        set_node_visible(&mut sv, "a", false);
        set_node_visible(&mut sv, "b", false);
        reset_visibility(&mut sv);
        assert!(is_node_visible(&sv, "a"));
        assert!(is_node_visible(&sv, "b"));
    }

    #[test]
    fn to_json() {
        let mut sv = new_scene_visibility();
        set_node_visible(&mut sv, "test", true);
        let j = visibility_to_json(&sv);
        assert!(j.contains("test"));
    }

    #[test]
    fn toggle_unknown_hides() {
        let mut sv = new_scene_visibility();
        toggle_visibility(&mut sv, "new");
        assert!(!is_node_visible(&sv, "new"));
    }
}
