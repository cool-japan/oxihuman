// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Light probe volume debug visualization.

/// Configuration for probe volume view.
#[derive(Debug, Clone)]
pub struct ProbeVolumeViewConfig {
    pub sphere_radius: f32,
    pub show_validity: bool,
    pub max_probes: u32,
}

impl Default for ProbeVolumeViewConfig {
    fn default() -> Self {
        Self { sphere_radius: 0.1, show_validity: true, max_probes: 512 }
    }
}

/// State for probe volume visualization.
#[derive(Debug, Clone)]
pub struct ProbeVolumeView {
    pub config: ProbeVolumeViewConfig,
    pub enabled: bool,
    pub probe_count: u32,
}

impl Default for ProbeVolumeView {
    fn default() -> Self {
        Self { config: ProbeVolumeViewConfig::default(), enabled: false, probe_count: 0 }
    }
}

/// Enable probe volume view.
pub fn pvv_enable(view: &mut ProbeVolumeView) {
    view.enabled = true;
}

/// Disable probe volume view.
pub fn pvv_disable(view: &mut ProbeVolumeView) {
    view.enabled = false;
}

/// Set the display sphere radius for each probe.
pub fn pvv_set_radius(view: &mut ProbeVolumeView, radius: f32) {
    view.config.sphere_radius = radius.clamp(0.001, 10.0);
}

/// Register a probe count for display.
pub fn pvv_set_probe_count(view: &mut ProbeVolumeView, count: u32) {
    view.probe_count = count.min(view.config.max_probes);
}

/// Return the color for a probe based on its validity (0.0=invalid, 1.0=valid).
pub fn pvv_probe_color(validity: f32) -> [f32; 4] {
    let v = validity.clamp(0.0, 1.0);
    [1.0 - v, v, 0.0, 1.0]
}

/// Return whether the probe budget is exceeded.
pub fn pvv_budget_exceeded(view: &ProbeVolumeView) -> bool {
    view.probe_count >= view.config.max_probes
}

/// Export config to JSON string (stub).
pub fn pvv_to_json(view: &ProbeVolumeView) -> String {
    format!(
        r#"{{"sphere_radius":{:.3},"max_probes":{},"probe_count":{},"enabled":{}}}"#,
        view.config.sphere_radius, view.config.max_probes, view.probe_count, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = ProbeVolumeView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable/disable should toggle */
        let mut v = ProbeVolumeView::default();
        pvv_enable(&mut v);
        assert!(v.enabled);
        pvv_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_radius() {
        /* radius should be stored */
        let mut v = ProbeVolumeView::default();
        pvv_set_radius(&mut v, 0.5);
        assert!((v.config.sphere_radius - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_radius_min_clamp() {
        /* radius should have a minimum */
        let mut v = ProbeVolumeView::default();
        pvv_set_radius(&mut v, 0.0);
        assert!(v.config.sphere_radius > 0.0);
    }

    #[test]
    fn test_set_probe_count() {
        /* probe count should be stored */
        let mut v = ProbeVolumeView::default();
        pvv_set_probe_count(&mut v, 100);
        assert_eq!(v.probe_count, 100);
    }

    #[test]
    fn test_probe_count_clamp() {
        /* count should be clamped to max_probes */
        let mut v = ProbeVolumeView::default();
        pvv_set_probe_count(&mut v, 99999);
        assert!(v.probe_count <= v.config.max_probes);
    }

    #[test]
    fn test_probe_color_valid() {
        /* valid probe should be green */
        let c = pvv_probe_color(1.0);
        assert!(c[1] > c[0]);
    }

    #[test]
    fn test_probe_color_invalid() {
        /* invalid probe should be red */
        let c = pvv_probe_color(0.0);
        assert!(c[0] > c[1]);
    }

    #[test]
    fn test_budget_exceeded() {
        /* budget should be exceeded at max */
        let mut v = ProbeVolumeView::default();
        let max = v.config.max_probes;
        pvv_set_probe_count(&mut v, max);
        assert!(pvv_budget_exceeded(&v));
    }
}
