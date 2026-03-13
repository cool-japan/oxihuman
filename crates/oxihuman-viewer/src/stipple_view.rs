// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Stipple / pointillism view stub.

/// Stipple placement strategy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StippleStrategy {
    Random,
    Grid,
    Voronoi,
}

/// Stipple view configuration.
#[derive(Debug, Clone)]
pub struct StippleView {
    pub strategy: StippleStrategy,
    pub density: f32,
    pub dot_radius: f32,
    pub jitter: f32,
    pub enabled: bool,
}

impl StippleView {
    pub fn new() -> Self {
        StippleView {
            strategy: StippleStrategy::Random,
            density: 0.5,
            dot_radius: 1.5,
            jitter: 0.3,
            enabled: true,
        }
    }
}

impl Default for StippleView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new stipple view.
pub fn new_stipple_view() -> StippleView {
    StippleView::new()
}

/// Set placement strategy.
pub fn stv_set_strategy(view: &mut StippleView, strategy: StippleStrategy) {
    view.strategy = strategy;
}

/// Set dot density.
pub fn stv_set_density(view: &mut StippleView, density: f32) {
    view.density = density.clamp(0.0, 1.0);
}

/// Set dot radius.
pub fn stv_set_dot_radius(view: &mut StippleView, radius: f32) {
    view.dot_radius = radius.clamp(0.5, 8.0);
}

/// Set jitter amount.
pub fn stv_set_jitter(view: &mut StippleView, jitter: f32) {
    view.jitter = jitter.clamp(0.0, 1.0);
}

/// Enable or disable.
pub fn stv_set_enabled(view: &mut StippleView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn stv_to_json(view: &StippleView) -> String {
    let strategy = match view.strategy {
        StippleStrategy::Random => "random",
        StippleStrategy::Grid => "grid",
        StippleStrategy::Voronoi => "voronoi",
    };
    format!(
        r#"{{"strategy":"{}","density":{},"dot_radius":{},"jitter":{},"enabled":{}}}"#,
        strategy, view.density, view.dot_radius, view.jitter, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_strategy() {
        let v = new_stipple_view();
        assert_eq!(
            v.strategy,
            StippleStrategy::Random /* default strategy must be Random */
        );
    }

    #[test]
    fn test_set_strategy() {
        let mut v = new_stipple_view();
        stv_set_strategy(&mut v, StippleStrategy::Voronoi);
        assert_eq!(
            v.strategy,
            StippleStrategy::Voronoi /* strategy must be set */
        );
    }

    #[test]
    fn test_density_clamped() {
        let mut v = new_stipple_view();
        stv_set_density(&mut v, 2.0);
        assert!((v.density - 1.0).abs() < 1e-6 /* density clamped to 1.0 */);
    }

    #[test]
    fn test_dot_radius_clamped_min() {
        let mut v = new_stipple_view();
        stv_set_dot_radius(&mut v, 0.0);
        assert!((v.dot_radius - 0.5).abs() < 1e-6 /* dot_radius clamped to 0.5 */);
    }

    #[test]
    fn test_dot_radius_clamped_max() {
        let mut v = new_stipple_view();
        stv_set_dot_radius(&mut v, 100.0);
        assert!((v.dot_radius - 8.0).abs() < 1e-6 /* dot_radius clamped to 8.0 */);
    }

    #[test]
    fn test_jitter_clamped() {
        let mut v = new_stipple_view();
        stv_set_jitter(&mut v, -0.5);
        assert!((v.jitter).abs() < 1e-6 /* jitter clamped to 0.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_stipple_view();
        stv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_strategy() {
        let v = new_stipple_view();
        let j = stv_to_json(&v);
        assert!(j.contains("\"strategy\"") /* JSON must have strategy */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_stipple_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_density() {
        let v = new_stipple_view();
        assert!((v.density - 0.5).abs() < 1e-6 /* default density must be 0.5 */);
    }
}
