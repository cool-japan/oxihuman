// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Object highlight/selection outline rendering.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum HighlightStyle {
    Outline,
    Glow,
    Wireframe,
    SolidOverlay,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    pub style: HighlightStyle,
    pub color: [f32; 4],
    pub thickness: f32,
    pub pulse_speed: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HighlightEntry {
    pub object_id: u32,
    pub config: HighlightConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HighlightManager {
    pub entries: Vec<HighlightEntry>,
    pub max_highlights: usize,
}

#[allow(dead_code)]
pub fn default_highlight_config() -> HighlightConfig {
    HighlightConfig {
        style: HighlightStyle::Outline,
        color: [1.0, 0.65, 0.0, 1.0],
        thickness: 2.0,
        pulse_speed: 0.0,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn new_highlight_manager(max: usize) -> HighlightManager {
    HighlightManager {
        entries: Vec::new(),
        max_highlights: max,
    }
}

#[allow(dead_code)]
pub fn add_highlight(mgr: &mut HighlightManager, object_id: u32, config: HighlightConfig) -> bool {
    if mgr.entries.len() >= mgr.max_highlights {
        return false;
    }
    mgr.entries.push(HighlightEntry { object_id, config });
    true
}

#[allow(dead_code)]
pub fn remove_highlight(mgr: &mut HighlightManager, object_id: u32) -> bool {
    let before = mgr.entries.len();
    mgr.entries.retain(|e| e.object_id != object_id);
    mgr.entries.len() < before
}

#[allow(dead_code)]
pub fn is_highlighted(mgr: &HighlightManager, object_id: u32) -> bool {
    mgr.entries.iter().any(|e| e.object_id == object_id)
}

#[allow(dead_code)]
pub fn clear_highlights(mgr: &mut HighlightManager) {
    mgr.entries.clear();
}

#[allow(dead_code)]
pub fn highlight_count(mgr: &HighlightManager) -> usize {
    mgr.entries.len()
}

#[allow(dead_code)]
pub fn pulse_alpha(base_alpha: f32, time: f32, speed: f32) -> f32 {
    if speed <= 0.0 {
        return base_alpha;
    }
    let osc = ((time * speed).sin() + 1.0) * 0.5;
    (base_alpha * (0.5 + 0.5 * osc)).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn highlight_to_json(entry: &HighlightEntry) -> String {
    let s = match &entry.config.style {
        HighlightStyle::Outline => "outline",
        HighlightStyle::Glow => "glow",
        HighlightStyle::Wireframe => "wireframe",
        HighlightStyle::SolidOverlay => "solid",
    };
    format!(
        r#"{{"id":{},"style":"{}","thickness":{},"enabled":{}}}"#,
        entry.object_id, s, entry.config.thickness, entry.config.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_highlight_config();
        assert_eq!(c.style, HighlightStyle::Outline);
        assert!(c.enabled);
    }

    #[test]
    fn test_new_manager() {
        let m = new_highlight_manager(10);
        assert!(m.entries.is_empty());
    }

    #[test]
    fn test_add_highlight() {
        let mut m = new_highlight_manager(5);
        assert!(add_highlight(&mut m, 1, default_highlight_config()));
        assert_eq!(highlight_count(&m), 1);
    }

    #[test]
    fn test_add_over_limit() {
        let mut m = new_highlight_manager(1);
        assert!(add_highlight(&mut m, 1, default_highlight_config()));
        assert!(!add_highlight(&mut m, 2, default_highlight_config()));
    }

    #[test]
    fn test_remove_highlight() {
        let mut m = new_highlight_manager(5);
        add_highlight(&mut m, 1, default_highlight_config());
        assert!(remove_highlight(&mut m, 1));
        assert_eq!(highlight_count(&m), 0);
    }

    #[test]
    fn test_is_highlighted() {
        let mut m = new_highlight_manager(5);
        add_highlight(&mut m, 42, default_highlight_config());
        assert!(is_highlighted(&m, 42));
        assert!(!is_highlighted(&m, 99));
    }

    #[test]
    fn test_clear() {
        let mut m = new_highlight_manager(5);
        add_highlight(&mut m, 1, default_highlight_config());
        add_highlight(&mut m, 2, default_highlight_config());
        clear_highlights(&mut m);
        assert!(m.entries.is_empty());
    }

    #[test]
    fn test_pulse_no_speed() {
        let a = pulse_alpha(0.8, 1.0, 0.0);
        assert!((a - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_pulse_range() {
        let a = pulse_alpha(1.0, 0.5, 2.0);
        assert!((0.0..=1.0).contains(&a));
    }

    #[test]
    fn test_to_json() {
        let e = HighlightEntry {
            object_id: 7,
            config: default_highlight_config(),
        };
        let j = highlight_to_json(&e);
        assert!(j.contains("outline"));
    }
}
