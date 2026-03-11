// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Film slate overlay view for production annotation.

/// Film slate overlay configuration.
#[derive(Debug, Clone)]
pub struct SlateView {
    pub production: String,
    pub scene: String,
    pub take: u32,
    pub roll: String,
    pub director: String,
    pub camera: String,
    pub date: String,
    pub enabled: bool,
    pub opacity: f32,
}

impl SlateView {
    pub fn new() -> Self {
        Self {
            production: String::from("Production"),
            scene: String::from("1"),
            take: 1,
            roll: String::from("A001"),
            director: String::new(),
            camera: String::from("A"),
            date: String::new(),
            enabled: false,
            opacity: 1.0,
        }
    }
}

impl Default for SlateView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new slate view.
pub fn new_slate_view() -> SlateView {
    SlateView::new()
}

/// Set scene identifier.
pub fn slv_set_scene(view: &mut SlateView, scene: &str) {
    view.scene = scene.to_string();
}

/// Set take number.
pub fn slv_set_take(view: &mut SlateView, take: u32) {
    view.take = take.max(1);
}

/// Set production title.
pub fn slv_set_production(view: &mut SlateView, title: &str) {
    view.production = title.to_string();
}

/// Increment take counter.
pub fn slv_next_take(view: &mut SlateView) {
    view.take = view.take.saturating_add(1);
}

/// Set overlay opacity.
pub fn slv_set_opacity(view: &mut SlateView, opacity: f32) {
    view.opacity = opacity.clamp(0.0, 1.0);
}

/// Serialize to JSON-like string.
pub fn slate_view_to_json(view: &SlateView) -> String {
    format!(
        r#"{{"production":"{}","scene":"{}","take":{},"roll":"{}","enabled":{}}}"#,
        view.production, view.scene, view.take, view.roll, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_slate_view();
        assert_eq!(v.take, 1);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_scene() {
        let mut v = new_slate_view();
        slv_set_scene(&mut v, "42A");
        assert_eq!(v.scene, "42A");
    }

    #[test]
    fn test_set_take() {
        let mut v = new_slate_view();
        slv_set_take(&mut v, 5);
        assert_eq!(v.take, 5);
    }

    #[test]
    fn test_take_min_one() {
        let mut v = new_slate_view();
        slv_set_take(&mut v, 0);
        assert_eq!(v.take, 1);
    }

    #[test]
    fn test_next_take() {
        let mut v = new_slate_view();
        slv_next_take(&mut v);
        assert_eq!(v.take, 2);
    }

    #[test]
    fn test_set_production() {
        let mut v = new_slate_view();
        slv_set_production(&mut v, "Epic Film");
        assert_eq!(v.production, "Epic Film");
    }

    #[test]
    fn test_opacity_clamp() {
        let mut v = new_slate_view();
        slv_set_opacity(&mut v, 2.0);
        assert_eq!(v.opacity, 1.0);
    }

    #[test]
    fn test_json() {
        let v = new_slate_view();
        let s = slate_view_to_json(&v);
        assert!(s.contains("production"));
        assert!(s.contains("take"));
    }

    #[test]
    fn test_clone() {
        let v = new_slate_view();
        let v2 = v.clone();
        assert_eq!(v2.take, v.take);
    }
}
