// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Image-based lighting (IBL) contribution debug view.

/// Which IBL component to isolate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IblComponent {
    Diffuse,
    Specular,
    Combined,
}

/// Configuration for IBL contribution view.
#[derive(Debug, Clone)]
pub struct IblContributionViewConfig {
    pub component: IblComponent,
    pub exposure: f32,
    pub show_probe_coverage: bool,
}

impl Default for IblContributionViewConfig {
    fn default() -> Self {
        Self { component: IblComponent::Combined, exposure: 1.0, show_probe_coverage: false }
    }
}

/// State for IBL contribution visualization.
#[derive(Debug, Clone)]
pub struct IblContributionView {
    pub config: IblContributionViewConfig,
    pub enabled: bool,
}

impl Default for IblContributionView {
    fn default() -> Self {
        Self { config: IblContributionViewConfig::default(), enabled: false }
    }
}

/// Enable IBL contribution view.
pub fn ibl_enable(view: &mut IblContributionView) {
    view.enabled = true;
}

/// Disable IBL contribution view.
pub fn ibl_disable(view: &mut IblContributionView) {
    view.enabled = false;
}

/// Set the IBL component to display.
pub fn ibl_set_component(view: &mut IblContributionView, component: IblComponent) {
    view.config.component = component;
}

/// Set the display exposure.
pub fn ibl_set_exposure(view: &mut IblContributionView, ev: f32) {
    view.config.exposure = ev.clamp(0.0, 32.0);
}

/// Combine diffuse and specular IBL contributions with exposure.
pub fn ibl_combine(diffuse: [f32; 3], specular: [f32; 3], config: &IblContributionViewConfig) -> [f32; 4] {
    let e = config.exposure;
    match config.component {
        IblComponent::Diffuse => [diffuse[0] * e, diffuse[1] * e, diffuse[2] * e, 1.0],
        IblComponent::Specular => [specular[0] * e, specular[1] * e, specular[2] * e, 1.0],
        IblComponent::Combined => [
            (diffuse[0] + specular[0]) * e,
            (diffuse[1] + specular[1]) * e,
            (diffuse[2] + specular[2]) * e,
            1.0,
        ],
    }
}

/// Return probe coverage weight at a normalised screen position (stub).
pub fn ibl_probe_coverage(u: f32, v: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    (u * v).sqrt()
}

/// Export config to JSON string (stub).
pub fn ibl_to_json(view: &IblContributionView) -> String {
    let comp = match view.config.component {
        IblComponent::Diffuse => "diffuse",
        IblComponent::Specular => "specular",
        IblComponent::Combined => "combined",
    };
    format!(r#"{{"component":"{}","exposure":{:.2},"enabled":{}}}"#, comp, view.config.exposure, view.enabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = IblContributionView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable/disable should toggle */
        let mut v = IblContributionView::default();
        ibl_enable(&mut v);
        assert!(v.enabled);
        ibl_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_component() {
        /* component should be updated */
        let mut v = IblContributionView::default();
        ibl_set_component(&mut v, IblComponent::Diffuse);
        assert_eq!(v.config.component, IblComponent::Diffuse);
    }

    #[test]
    fn test_set_exposure() {
        /* exposure should be stored */
        let mut v = IblContributionView::default();
        ibl_set_exposure(&mut v, 2.5);
        assert!((v.config.exposure - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_exposure_clamp() {
        /* exposure above maximum should be clamped */
        let mut v = IblContributionView::default();
        ibl_set_exposure(&mut v, 9999.0);
        assert!((v.config.exposure - 32.0).abs() < 1e-6);
    }

    #[test]
    fn test_combine_diffuse_only() {
        /* diffuse component should return only diffuse */
        let cfg = IblContributionViewConfig { component: IblComponent::Diffuse, exposure: 1.0, show_probe_coverage: false };
        let c = ibl_combine([0.5, 0.5, 0.5], [0.0, 0.0, 0.0], &cfg);
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_combine_alpha_one() {
        /* alpha should always be 1.0 */
        let cfg = IblContributionViewConfig::default();
        let c = ibl_combine([0.5; 3], [0.5; 3], &cfg);
        assert!((c[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_probe_coverage_range() {
        /* coverage should be in [0, 1] */
        let cov = ibl_probe_coverage(0.5, 0.5);
        assert!((0.0..=1.0).contains(&cov));
    }

    #[test]
    fn test_to_json_component() {
        /* JSON should contain component name */
        let v = IblContributionView::default();
        let json = ibl_to_json(&v);
        assert!(json.contains("combined"));
    }
}
