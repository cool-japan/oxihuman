// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Body hair density morph parameter stub.

/// Body region for hair density control.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HairRegion {
    Arms,
    Legs,
    Chest,
    Back,
    Abdomen,
    Face,
}

/// Body hair density controller.
#[derive(Debug, Clone)]
pub struct BodyHairDensity {
    pub global_density: f32,
    pub region_densities: Vec<(HairRegion, f32)>,
    pub coarseness: f32,
    pub enabled: bool,
}

impl BodyHairDensity {
    pub fn new() -> Self {
        BodyHairDensity {
            global_density: 0.5,
            region_densities: Vec::new(),
            coarseness: 0.5,
            enabled: true,
        }
    }
}

impl Default for BodyHairDensity {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new body hair density controller.
pub fn new_body_hair_density() -> BodyHairDensity {
    BodyHairDensity::new()
}

/// Set global hair density.
pub fn bhd_set_density(ctrl: &mut BodyHairDensity, density: f32) {
    ctrl.global_density = density.clamp(0.0, 1.0);
}

/// Set density for a specific region.
pub fn bhd_set_region(ctrl: &mut BodyHairDensity, region: HairRegion, density: f32) {
    let clamped = density.clamp(0.0, 1.0);
    if let Some(entry) = ctrl.region_densities.iter_mut().find(|(r, _)| *r == region) {
        entry.1 = clamped;
    } else {
        ctrl.region_densities.push((region, clamped));
    }
}

/// Set hair coarseness (thickness of individual strands).
pub fn bhd_set_coarseness(ctrl: &mut BodyHairDensity, coarseness: f32) {
    ctrl.coarseness = coarseness.clamp(0.0, 1.0);
}

/// Enable or disable.
pub fn bhd_set_enabled(ctrl: &mut BodyHairDensity, enabled: bool) {
    ctrl.enabled = enabled;
}

/// Return region override count.
pub fn bhd_region_count(ctrl: &BodyHairDensity) -> usize {
    ctrl.region_densities.len()
}

/// Serialize to JSON-like string.
pub fn bhd_to_json(ctrl: &BodyHairDensity) -> String {
    format!(
        r#"{{"global_density":{},"coarseness":{},"regions":{},"enabled":{}}}"#,
        ctrl.global_density,
        ctrl.coarseness,
        ctrl.region_densities.len(),
        ctrl.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_density() {
        let c = new_body_hair_density();
        assert!((c.global_density - 0.5).abs() < 1e-6 /* default density must be 0.5 */);
    }

    #[test]
    fn test_set_density_clamps() {
        let mut c = new_body_hair_density();
        bhd_set_density(&mut c, 2.0);
        assert!((c.global_density - 1.0).abs() < 1e-6 /* density clamped to 1.0 */);
    }

    #[test]
    fn test_set_region_adds_entry() {
        let mut c = new_body_hair_density();
        bhd_set_region(&mut c, HairRegion::Chest, 0.7);
        assert_eq!(
            bhd_region_count(&c),
            1 /* one region entry must be added */
        );
    }

    #[test]
    fn test_set_region_updates_existing() {
        let mut c = new_body_hair_density();
        bhd_set_region(&mut c, HairRegion::Arms, 0.3);
        bhd_set_region(&mut c, HairRegion::Arms, 0.8);
        assert_eq!(
            bhd_region_count(&c),
            1 /* duplicate region must not add new entry */
        );
    }

    #[test]
    fn test_coarseness_clamped() {
        let mut c = new_body_hair_density();
        bhd_set_coarseness(&mut c, -0.5);
        assert!((c.coarseness).abs() < 1e-6 /* coarseness clamped to 0 */);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_body_hair_density();
        bhd_set_enabled(&mut c, false);
        assert!(!c.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_density() {
        let c = new_body_hair_density();
        let j = bhd_to_json(&c);
        assert!(j.contains("\"global_density\"") /* JSON must contain global_density */);
    }

    #[test]
    fn test_enabled_default() {
        let c = new_body_hair_density();
        assert!(c.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_trait() {
        let c = BodyHairDensity::default();
        assert!((c.global_density - 0.5).abs() < 1e-6 /* Default trait must give 0.5 density */);
    }
}
