// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! UV editor overlay renderer.

#![allow(dead_code)]

/// Configuration for the UV editor overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvOverlayConfig {
    pub grid_size: u32,
    pub show_grid: bool,
    pub show_seams: bool,
    pub opacity: f32,
}

/// A single UV island.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvIsland {
    pub id: u32,
    pub uv_coords: Vec<[f32; 2]>,
    pub selected: bool,
}

/// UV editor overlay state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvEditorOverlay {
    pub config: UvOverlayConfig,
    pub islands: Vec<UvIsland>,
}

#[allow(dead_code)]
pub fn default_uv_overlay_config() -> UvOverlayConfig {
    UvOverlayConfig {
        grid_size: 10,
        show_grid: true,
        show_seams: true,
        opacity: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_uv_editor_overlay() -> UvEditorOverlay {
    UvEditorOverlay {
        config: default_uv_overlay_config(),
        islands: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn uvo_add_island(overlay: &mut UvEditorOverlay, island: UvIsland) {
    overlay.islands.push(island);
}

#[allow(dead_code)]
pub fn uvo_remove_island(overlay: &mut UvEditorOverlay, id: u32) -> bool {
    if let Some(pos) = overlay.islands.iter().position(|i| i.id == id) {
        overlay.islands.remove(pos);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn uvo_island_count(overlay: &UvEditorOverlay) -> usize {
    overlay.islands.len()
}

#[allow(dead_code)]
pub fn uvo_select_island(overlay: &mut UvEditorOverlay, id: u32, selected: bool) -> bool {
    if let Some(island) = overlay.islands.iter_mut().find(|i| i.id == id) {
        island.selected = selected;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn uvo_clear(overlay: &mut UvEditorOverlay) {
    overlay.islands.clear();
}

#[allow(dead_code)]
pub fn uvo_to_json(overlay: &UvEditorOverlay) -> String {
    format!(
        r#"{{"island_count":{},"show_grid":{},"show_seams":{}}}"#,
        overlay.islands.len(),
        overlay.config.show_grid,
        overlay.config.show_seams
    )
}

#[allow(dead_code)]
pub fn uvo_selected_count(overlay: &UvEditorOverlay) -> usize {
    overlay.islands.iter().filter(|i| i.selected).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_uv_overlay_config();
        assert_eq!(cfg.grid_size, 10);
        assert!(cfg.show_grid);
        assert!(cfg.show_seams);
        assert!((cfg.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_overlay_empty() {
        let o = new_uv_editor_overlay();
        assert_eq!(uvo_island_count(&o), 0);
    }

    #[test]
    fn test_add_island() {
        let mut o = new_uv_editor_overlay();
        let island = UvIsland { id: 1, uv_coords: vec![[0.0, 0.0], [1.0, 0.0]], selected: false };
        uvo_add_island(&mut o, island);
        assert_eq!(uvo_island_count(&o), 1);
    }

    #[test]
    fn test_remove_island() {
        let mut o = new_uv_editor_overlay();
        let island = UvIsland { id: 1, uv_coords: Vec::new(), selected: false };
        uvo_add_island(&mut o, island);
        assert!(uvo_remove_island(&mut o, 1));
        assert_eq!(uvo_island_count(&o), 0);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut o = new_uv_editor_overlay();
        assert!(!uvo_remove_island(&mut o, 99));
    }

    #[test]
    fn test_select_island() {
        let mut o = new_uv_editor_overlay();
        let island = UvIsland { id: 1, uv_coords: Vec::new(), selected: false };
        uvo_add_island(&mut o, island);
        uvo_select_island(&mut o, 1, true);
        assert_eq!(uvo_selected_count(&o), 1);
    }

    #[test]
    fn test_clear() {
        let mut o = new_uv_editor_overlay();
        uvo_add_island(&mut o, UvIsland { id: 1, uv_coords: Vec::new(), selected: false });
        uvo_add_island(&mut o, UvIsland { id: 2, uv_coords: Vec::new(), selected: false });
        uvo_clear(&mut o);
        assert_eq!(uvo_island_count(&o), 0);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let o = new_uv_editor_overlay();
        let j = uvo_to_json(&o);
        assert!(j.contains("island_count"));
        assert!(j.contains("show_grid"));
    }
}
