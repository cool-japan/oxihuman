// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! On-screen text label renderer for HUD overlays (position, content, color, size).

#![allow(dead_code)]

/// Configuration for the on-screen text system.
#[derive(Debug, Clone)]
pub struct OnscreenTextConfig {
    /// Default font size in points.
    pub default_size: f32,
    /// Maximum number of simultaneous labels.
    pub max_labels: usize,
    /// Default text color [r, g, b, a] in 0–255.
    pub default_color: [u8; 4],
}

/// A single on-screen text label.
#[derive(Debug, Clone)]
pub struct TextLabel {
    /// Unique label id.
    pub id: u32,
    /// Screen-space position (x, y) in pixels from top-left.
    pub position: [f32; 2],
    /// Text content to render.
    pub content: String,
    /// Font size in points.
    pub size: f32,
    /// Text color [r, g, b, a].
    pub color: [u8; 4],
    /// Whether this label is visible.
    pub visible: bool,
}

/// State for the on-screen text rendering system.
#[derive(Debug, Clone)]
pub struct OnscreenTextState {
    /// All registered labels.
    pub labels: Vec<TextLabel>,
    /// Next label id.
    next_id: u32,
}

/// Returns the default [`OnscreenTextConfig`].
pub fn default_onscreen_text_config() -> OnscreenTextConfig {
    OnscreenTextConfig {
        default_size: 14.0,
        max_labels: 128,
        default_color: [255, 255, 255, 255],
    }
}

/// Creates a new, empty [`OnscreenTextState`].
pub fn new_onscreen_text() -> OnscreenTextState {
    OnscreenTextState {
        labels: Vec::new(),
        next_id: 1,
    }
}

/// Adds a label, returning its assigned id.
/// Returns 0 if `max_labels` is reached.
pub fn text_add_label(
    state: &mut OnscreenTextState,
    cfg: &OnscreenTextConfig,
    position: [f32; 2],
    content: &str,
) -> u32 {
    if state.labels.len() >= cfg.max_labels {
        return 0;
    }
    let id = state.next_id;
    state.next_id += 1;
    state.labels.push(TextLabel {
        id,
        position,
        content: content.to_string(),
        size: cfg.default_size,
        color: cfg.default_color,
        visible: true,
    });
    id
}

/// Removes a label by id. Returns `true` if found.
pub fn text_remove_label(state: &mut OnscreenTextState, id: u32) -> bool {
    if let Some(pos) = state.labels.iter().position(|l| l.id == id) {
        state.labels.remove(pos);
        true
    } else {
        false
    }
}

/// Updates the content and position of a label by id.
/// Returns `true` if the label was found and updated.
pub fn text_update_label(
    state: &mut OnscreenTextState,
    id: u32,
    position: [f32; 2],
    content: &str,
) -> bool {
    if let Some(label) = state.labels.iter_mut().find(|l| l.id == id) {
        label.position = position;
        label.content = content.to_string();
        true
    } else {
        false
    }
}

/// Returns the number of labels currently registered.
pub fn text_label_count(state: &OnscreenTextState) -> usize {
    state.labels.len()
}

/// Finds a label by id, returning a reference or `None`.
pub fn text_find_label(state: &OnscreenTextState, id: u32) -> Option<&TextLabel> {
    state.labels.iter().find(|l| l.id == id)
}

/// Sets visibility of a label by id.
/// Returns `true` if found.
pub fn text_set_visible(state: &mut OnscreenTextState, id: u32, visible: bool) -> bool {
    if let Some(label) = state.labels.iter_mut().find(|l| l.id == id) {
        label.visible = visible;
        true
    } else {
        false
    }
}

/// Serialises all labels as JSON.
pub fn text_to_json(state: &OnscreenTextState, _cfg: &OnscreenTextConfig) -> String {
    let mut out = String::from("{\"labels\":[");
    let len = state.labels.len();
    for (i, l) in state.labels.iter().enumerate() {
        let comma = if i + 1 < len { "," } else { "" };
        out.push_str(&format!(
            "{{\"id\":{},\"x\":{:.2},\"y\":{:.2},\"content\":\"{}\",\
             \"size\":{:.2},\"color\":[{},{},{},{}],\"visible\":{}}}{comma}",
            l.id,
            l.position[0], l.position[1],
            l.content,
            l.size,
            l.color[0], l.color[1], l.color[2], l.color[3],
            l.visible
        ));
    }
    out.push_str("]}");
    out
}

/// Removes all labels.
pub fn text_clear(state: &mut OnscreenTextState) {
    state.labels.clear();
    state.next_id = 1;
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_onscreen_text_config();
        assert!((cfg.default_size - 14.0).abs() < 1e-6);
        assert_eq!(cfg.max_labels, 128);
        assert_eq!(cfg.default_color, [255, 255, 255, 255]);
    }

    #[test]
    fn new_state_is_empty() {
        let s = new_onscreen_text();
        assert_eq!(text_label_count(&s), 0);
    }

    #[test]
    fn add_label_increments_count() {
        let mut s = new_onscreen_text();
        let cfg = default_onscreen_text_config();
        text_add_label(&mut s, &cfg, [100.0, 200.0], "Hello");
        assert_eq!(text_label_count(&s), 1);
    }

    #[test]
    fn add_label_returns_unique_ids() {
        let mut s = new_onscreen_text();
        let cfg = default_onscreen_text_config();
        let id1 = text_add_label(&mut s, &cfg, [0.0, 0.0], "A");
        let id2 = text_add_label(&mut s, &cfg, [0.0, 0.0], "B");
        assert_ne!(id1, id2);
        assert!(id1 > 0);
        assert!(id2 > 0);
    }

    #[test]
    fn remove_label_decrements_count() {
        let mut s = new_onscreen_text();
        let cfg = default_onscreen_text_config();
        let id = text_add_label(&mut s, &cfg, [10.0, 20.0], "Test");
        assert!(text_remove_label(&mut s, id));
        assert_eq!(text_label_count(&s), 0);
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let mut s = new_onscreen_text();
        assert!(!text_remove_label(&mut s, 999));
    }

    #[test]
    fn update_label_changes_content() {
        let mut s = new_onscreen_text();
        let cfg = default_onscreen_text_config();
        let id = text_add_label(&mut s, &cfg, [0.0, 0.0], "Old");
        text_update_label(&mut s, id, [5.0, 5.0], "New");
        let found = text_find_label(&s, id).expect("should succeed");
        assert_eq!(found.content, "New");
        assert!((found.position[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn set_visible_false() {
        let mut s = new_onscreen_text();
        let cfg = default_onscreen_text_config();
        let id = text_add_label(&mut s, &cfg, [0.0, 0.0], "Label");
        text_set_visible(&mut s, id, false);
        assert!(!text_find_label(&s, id).expect("should succeed").visible);
    }

    #[test]
    fn json_contains_labels_key() {
        let mut s = new_onscreen_text();
        let cfg = default_onscreen_text_config();
        text_add_label(&mut s, &cfg, [100.0, 50.0], "FPS: 60");
        let json = text_to_json(&s, &cfg);
        assert!(json.contains("\"labels\""));
        assert!(json.contains("FPS: 60"));
    }

    #[test]
    fn max_labels_cap() {
        let mut s = new_onscreen_text();
        let mut cfg = default_onscreen_text_config();
        cfg.max_labels = 2;
        text_add_label(&mut s, &cfg, [0.0, 0.0], "a");
        text_add_label(&mut s, &cfg, [0.0, 0.0], "b");
        let id = text_add_label(&mut s, &cfg, [0.0, 0.0], "c");
        assert_eq!(id, 0);
        assert_eq!(text_label_count(&s), 2);
    }

    #[test]
    fn clear_resets_state() {
        let mut s = new_onscreen_text();
        let cfg = default_onscreen_text_config();
        text_add_label(&mut s, &cfg, [0.0, 0.0], "x");
        text_clear(&mut s);
        assert_eq!(text_label_count(&s), 0);
    }
}
