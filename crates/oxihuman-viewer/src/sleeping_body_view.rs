// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Sleeping rigid body indicator — dims or tints bodies that have entered sleep state.

/// Sleeping body view configuration.
#[derive(Debug, Clone)]
pub struct SleepingBodyView {
    pub enabled: bool,
    pub sleeping_tint: [f32; 4],
    pub awake_tint: [f32; 4],
    pub show_sleep_timer: bool,
}

impl SleepingBodyView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            sleeping_tint: [0.3, 0.3, 0.6, 0.5],
            awake_tint: [0.2, 0.9, 0.2, 0.3],
            show_sleep_timer: false,
        }
    }
}

impl Default for SleepingBodyView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new sleeping body view.
pub fn new_sleeping_body_view() -> SleepingBodyView {
    SleepingBodyView::new()
}

/// Enable or disable sleeping body indicator.
pub fn sbv_set_enabled(v: &mut SleepingBodyView, enabled: bool) {
    v.enabled = enabled;
}

/// Set the tint colour for sleeping bodies.
pub fn sbv_set_sleeping_tint(v: &mut SleepingBodyView, color: [f32; 4]) {
    v.sleeping_tint = color;
}

/// Set the tint colour for awake bodies.
pub fn sbv_set_awake_tint(v: &mut SleepingBodyView, color: [f32; 4]) {
    v.awake_tint = color;
}

/// Toggle sleep-timer text label.
pub fn sbv_set_show_timer(v: &mut SleepingBodyView, show: bool) {
    v.show_sleep_timer = show;
}

/// Choose tint based on sleep state.
pub fn sbv_tint_for_state(v: &SleepingBodyView, is_sleeping: bool) -> [f32; 4] {
    if is_sleeping {
        v.sleeping_tint
    } else {
        v.awake_tint
    }
}

/// Serialize to JSON-like string.
pub fn sleeping_body_view_to_json(v: &SleepingBodyView) -> String {
    format!(
        r#"{{"enabled":{},"show_sleep_timer":{}}}"#,
        v.enabled, v.show_sleep_timer
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_sleeping_body_view();
        assert!(!v.enabled);
        assert!(!v.show_sleep_timer);
    }

    #[test]
    fn test_enable() {
        let mut v = new_sleeping_body_view();
        sbv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_sleeping_tint() {
        let mut v = new_sleeping_body_view();
        sbv_set_sleeping_tint(&mut v, [1.0, 0.0, 0.0, 1.0]);
        assert!((v.sleeping_tint[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_awake_tint() {
        let mut v = new_sleeping_body_view();
        sbv_set_awake_tint(&mut v, [0.0, 1.0, 0.0, 1.0]);
        assert!((v.awake_tint[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_show_timer_toggle() {
        let mut v = new_sleeping_body_view();
        sbv_set_show_timer(&mut v, true);
        assert!(v.show_sleep_timer);
    }

    #[test]
    fn test_tint_sleeping() {
        let v = new_sleeping_body_view();
        let c = sbv_tint_for_state(&v, true);
        assert!((c[0] - v.sleeping_tint[0]).abs() < 1e-6);
    }

    #[test]
    fn test_tint_awake() {
        let v = new_sleeping_body_view();
        let c = sbv_tint_for_state(&v, false);
        assert!((c[0] - v.awake_tint[0]).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let v = new_sleeping_body_view();
        let s = sleeping_body_view_to_json(&v);
        assert!(s.contains("show_sleep_timer"));
    }

    #[test]
    fn test_clone() {
        let v = new_sleeping_body_view();
        let v2 = v.clone();
        assert_eq!(v2.enabled, v.enabled);
    }
}
