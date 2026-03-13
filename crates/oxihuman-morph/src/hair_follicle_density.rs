// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Scalp follicle density morph stub.

/// Scalp region.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScalpRegion {
    Frontal,
    Parietal,
    Temporal,
    Occipital,
    Crown,
}

/// Hair follicle density controller.
#[derive(Debug, Clone)]
pub struct HairFollicleDensity {
    pub global_density: f32,
    pub region_densities: Vec<(ScalpRegion, f32)>,
    pub miniaturization: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl HairFollicleDensity {
    pub fn new(morph_count: usize) -> Self {
        HairFollicleDensity {
            global_density: 0.8,
            region_densities: Vec::new(),
            miniaturization: 0.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new hair follicle density controller.
pub fn new_hair_follicle_density(morph_count: usize) -> HairFollicleDensity {
    HairFollicleDensity::new(morph_count)
}

/// Set global follicle density.
pub fn hfd_set_density(ctrl: &mut HairFollicleDensity, density: f32) {
    ctrl.global_density = density.clamp(0.0, 1.0);
}

/// Set per-region density.
pub fn hfd_set_region(ctrl: &mut HairFollicleDensity, region: ScalpRegion, density: f32) {
    let v = density.clamp(0.0, 1.0);
    if let Some(e) = ctrl.region_densities.iter_mut().find(|(r, _)| *r == region) {
        e.1 = v;
    } else {
        ctrl.region_densities.push((region, v));
    }
}

/// Set follicle miniaturization (hair thinning).
pub fn hfd_set_miniaturization(ctrl: &mut HairFollicleDensity, value: f32) {
    ctrl.miniaturization = value.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: density * (1 - miniaturization)).
pub fn hfd_evaluate(ctrl: &HairFollicleDensity) -> Vec<f32> {
    /* Stub: effective density reduced by miniaturization */
    if !ctrl.enabled || ctrl.morph_count == 0 {
        return vec![];
    }
    let w = ctrl.global_density * (1.0 - ctrl.miniaturization);
    vec![w; ctrl.morph_count]
}

/// Enable or disable.
pub fn hfd_set_enabled(ctrl: &mut HairFollicleDensity, enabled: bool) {
    ctrl.enabled = enabled;
}

/// Return region count.
pub fn hfd_region_count(ctrl: &HairFollicleDensity) -> usize {
    ctrl.region_densities.len()
}

/// Serialize to JSON-like string.
pub fn hfd_to_json(ctrl: &HairFollicleDensity) -> String {
    format!(
        r#"{{"global_density":{},"miniaturization":{},"morph_count":{},"enabled":{}}}"#,
        ctrl.global_density, ctrl.miniaturization, ctrl.morph_count, ctrl.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_density() {
        let c = new_hair_follicle_density(4);
        assert!((c.global_density - 0.8).abs() < 1e-6 /* default density must be 0.8 */);
    }

    #[test]
    fn test_set_density_clamps() {
        let mut c = new_hair_follicle_density(4);
        hfd_set_density(&mut c, 2.0);
        assert!((c.global_density - 1.0).abs() < 1e-6 /* density clamped to 1.0 */);
    }

    #[test]
    fn test_region_added() {
        let mut c = new_hair_follicle_density(4);
        hfd_set_region(&mut c, ScalpRegion::Frontal, 0.5);
        assert_eq!(hfd_region_count(&c), 1 /* one region added */);
    }

    #[test]
    fn test_miniaturization_clamped() {
        let mut c = new_hair_follicle_density(4);
        hfd_set_miniaturization(&mut c, -0.3);
        assert!((c.miniaturization).abs() < 1e-6 /* miniaturization clamped to 0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let c = new_hair_follicle_density(5);
        assert_eq!(
            hfd_evaluate(&c).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut c = new_hair_follicle_density(4);
        hfd_set_enabled(&mut c, false);
        assert!(hfd_evaluate(&c).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_evaluate_product() {
        let mut c = new_hair_follicle_density(2);
        hfd_set_density(&mut c, 0.8);
        hfd_set_miniaturization(&mut c, 0.5);
        let out = hfd_evaluate(&c);
        assert!((out[0] - 0.4).abs() < 1e-5 /* weight = density * (1-miniaturization) */);
    }

    #[test]
    fn test_to_json_has_fields() {
        let c = new_hair_follicle_density(4);
        let j = hfd_to_json(&c);
        assert!(j.contains("\"global_density\"") /* JSON must have global_density */);
    }

    #[test]
    fn test_enabled_default() {
        let c = new_hair_follicle_density(4);
        assert!(c.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_no_miniaturization_default() {
        let c = new_hair_follicle_density(4);
        assert!((c.miniaturization).abs() < 1e-6 /* default miniaturization must be 0 */);
    }
}
