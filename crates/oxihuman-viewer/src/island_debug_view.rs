// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Physics island debug view — colours each simulation island distinctly.

/// Island debug view configuration.
#[derive(Debug, Clone)]
pub struct IslandDebugView {
    pub enabled: bool,
    pub show_island_id: bool,
    pub alpha: f32,
    pub max_islands: usize,
}

impl IslandDebugView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_island_id: true,
            alpha: 0.4,
            max_islands: 256,
        }
    }
}

impl Default for IslandDebugView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new island debug view.
pub fn new_island_debug_view() -> IslandDebugView {
    IslandDebugView::new()
}

/// Enable or disable island debug display.
pub fn idv_set_enabled(v: &mut IslandDebugView, enabled: bool) {
    v.enabled = enabled;
}

/// Toggle island ID text labels.
pub fn idv_set_show_id(v: &mut IslandDebugView, show: bool) {
    v.show_island_id = show;
}

/// Set island overlay alpha.
pub fn idv_set_alpha(v: &mut IslandDebugView, alpha: f32) {
    v.alpha = alpha.clamp(0.0, 1.0);
}

/// Set maximum number of islands to colour.
pub fn idv_set_max_islands(v: &mut IslandDebugView, max: usize) {
    v.max_islands = max.max(1);
}

/// Generate a deterministic colour for an island index.
pub fn idv_island_color(island_idx: usize) -> [f32; 3] {
    let h = (island_idx as f32 * 0.618_034) % 1.0; /* golden ratio spacing */
    let s = 0.7_f32;
    let l = 0.55_f32;
    /* HSL to RGB — simplified */
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let hk = h;
    [
        hsl_component(p, q, hk + 1.0 / 3.0),
        hsl_component(p, q, hk),
        hsl_component(p, q, hk - 1.0 / 3.0),
    ]
}

fn hsl_component(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 0.5 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// Serialize to JSON-like string.
pub fn island_debug_view_to_json(v: &IslandDebugView) -> String {
    format!(
        r#"{{"enabled":{},"show_island_id":{},"alpha":{:.4},"max_islands":{}}}"#,
        v.enabled, v.show_island_id, v.alpha, v.max_islands
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_island_debug_view();
        assert!(!v.enabled);
        assert_eq!(v.max_islands, 256);
    }

    #[test]
    fn test_enable() {
        let mut v = new_island_debug_view();
        idv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_alpha_clamp() {
        let mut v = new_island_debug_view();
        idv_set_alpha(&mut v, 2.0);
        assert_eq!(v.alpha, 1.0);
    }

    #[test]
    fn test_max_islands_min() {
        let mut v = new_island_debug_view();
        idv_set_max_islands(&mut v, 0);
        assert_eq!(v.max_islands, 1);
    }

    #[test]
    fn test_show_id_toggle() {
        let mut v = new_island_debug_view();
        idv_set_show_id(&mut v, false);
        assert!(!v.show_island_id);
    }

    #[test]
    fn test_island_color_valid_range() {
        let c = idv_island_color(0);
        for ch in c.iter() {
            assert!((0.0..=1.0).contains(ch));
        }
    }

    #[test]
    fn test_island_color_different_indices() {
        let c0 = idv_island_color(0);
        let c1 = idv_island_color(1);
        /* colours should differ between islands */
        let diff: f32 = c0.iter().zip(c1.iter()).map(|(a, b)| (a - b).abs()).sum();
        assert!(diff > 0.01);
    }

    #[test]
    fn test_json_keys() {
        let v = new_island_debug_view();
        let s = island_debug_view_to_json(&v);
        assert!(s.contains("max_islands"));
    }

    #[test]
    fn test_clone() {
        let v = new_island_debug_view();
        let v2 = v.clone();
        assert_eq!(v2.max_islands, v.max_islands);
    }
}
