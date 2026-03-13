// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Shadow cascade visualization debug overlay.

/// Configuration for shadow cascade view.
#[derive(Debug, Clone)]
pub struct ShadowCascadeViewConfig {
    pub num_cascades: u32,
    pub opacity: f32,
    pub color_per_cascade: bool,
}

impl Default for ShadowCascadeViewConfig {
    fn default() -> Self {
        Self { num_cascades: 4, opacity: 0.5, color_per_cascade: true }
    }
}

/// State for shadow cascade visualization.
#[derive(Debug, Clone)]
pub struct ShadowCascadeView {
    pub config: ShadowCascadeViewConfig,
    pub enabled: bool,
    pub active_cascade: Option<u32>,
}

impl Default for ShadowCascadeView {
    fn default() -> Self {
        Self { config: ShadowCascadeViewConfig::default(), enabled: false, active_cascade: None }
    }
}

/// Enable shadow cascade visualization.
pub fn scv_enable(view: &mut ShadowCascadeView) {
    view.enabled = true;
}

/// Disable shadow cascade visualization.
pub fn scv_disable(view: &mut ShadowCascadeView) {
    view.enabled = false;
}

/// Select a specific cascade to highlight.
pub fn scv_set_active_cascade(view: &mut ShadowCascadeView, index: Option<u32>) {
    view.active_cascade = index.map(|i| i.min(view.config.num_cascades.saturating_sub(1)));
}

/// Return the colour assigned to a cascade index (RGBA, stub).
pub fn scv_cascade_color(index: u32) -> [f32; 4] {
    let colors = [
        [1.0, 0.2, 0.2, 0.5],
        [0.2, 1.0, 0.2, 0.5],
        [0.2, 0.2, 1.0, 0.5],
        [1.0, 1.0, 0.2, 0.5],
    ];
    colors[(index as usize) % colors.len()]
}

/// Return the split distance for a given cascade (stub).
pub fn scv_split_distance(cascade: u32, near: f32, far: f32) -> f32 {
    let t = (cascade + 1) as f32 / 4.0;
    near + (far - near) * t * t
}

/// Export config to JSON string (stub).
pub fn scv_to_json(view: &ShadowCascadeView) -> String {
    format!(
        r#"{{"num_cascades":{},"opacity":{:.2},"enabled":{}}}"#,
        view.config.num_cascades, view.config.opacity, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = ShadowCascadeView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable and disable should toggle */
        let mut v = ShadowCascadeView::default();
        scv_enable(&mut v);
        assert!(v.enabled);
        scv_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_active_cascade_clamped() {
        /* cascade index should be clamped to num_cascades - 1 */
        let mut v = ShadowCascadeView::default();
        scv_set_active_cascade(&mut v, Some(9999));
        assert!(v.active_cascade.expect("should succeed") < v.config.num_cascades);
    }

    #[test]
    fn test_active_cascade_none() {
        /* None should clear the active cascade */
        let mut v = ShadowCascadeView::default();
        scv_set_active_cascade(&mut v, Some(1));
        scv_set_active_cascade(&mut v, None);
        assert!(v.active_cascade.is_none());
    }

    #[test]
    fn test_cascade_color_valid() {
        /* each cascade should return a valid color */
        let c = scv_cascade_color(0);
        assert_eq!(c.len(), 4);
    }

    #[test]
    fn test_cascade_color_cycles() {
        /* colors should cycle rather than panic */
        let c0 = scv_cascade_color(0);
        let c4 = scv_cascade_color(4);
        assert_eq!(c0, c4);
    }

    #[test]
    fn test_split_distance_ordering() {
        /* split distances should increase with cascade index */
        let d0 = scv_split_distance(0, 0.1, 1000.0);
        let d1 = scv_split_distance(1, 0.1, 1000.0);
        assert!(d1 > d0);
    }

    #[test]
    fn test_to_json_enabled() {
        /* JSON should reflect enabled state */
        let mut v = ShadowCascadeView::default();
        scv_enable(&mut v);
        let json = scv_to_json(&v);
        assert!(json.contains("true"));
    }

    #[test]
    fn test_default_num_cascades() {
        /* default should have 4 cascades */
        let v = ShadowCascadeView::default();
        assert_eq!(v.config.num_cascades, 4);
    }
}
