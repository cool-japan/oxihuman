// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Visible vein/vessel morph stub.

/// Vein prominence region.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VeinRegion {
    Forearm,
    Hand,
    Bicep,
    Temple,
    Neck,
    Foot,
}

/// Vascular visibility morph controller.
#[derive(Debug, Clone)]
pub struct VascularVisibility {
    pub global_visibility: f32,
    pub region_overrides: Vec<(VeinRegion, f32)>,
    pub dilation: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl VascularVisibility {
    pub fn new(morph_count: usize) -> Self {
        VascularVisibility {
            global_visibility: 0.2,
            region_overrides: Vec::new(),
            dilation: 0.5,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new vascular visibility controller.
pub fn new_vascular_visibility(morph_count: usize) -> VascularVisibility {
    VascularVisibility::new(morph_count)
}

/// Set global vein visibility.
pub fn vv_set_visibility(ctrl: &mut VascularVisibility, visibility: f32) {
    ctrl.global_visibility = visibility.clamp(0.0, 1.0);
}

/// Set per-region visibility override.
pub fn vv_set_region(ctrl: &mut VascularVisibility, region: VeinRegion, visibility: f32) {
    let v = visibility.clamp(0.0, 1.0);
    if let Some(e) = ctrl.region_overrides.iter_mut().find(|(r, _)| *r == region) {
        e.1 = v;
    } else {
        ctrl.region_overrides.push((region, v));
    }
}

/// Set vein dilation factor.
pub fn vv_set_dilation(ctrl: &mut VascularVisibility, dilation: f32) {
    ctrl.dilation = dilation.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: visibility × dilation).
pub fn vv_evaluate(ctrl: &VascularVisibility) -> Vec<f32> {
    /* Stub: weight scaled by visibility and dilation */
    if !ctrl.enabled || ctrl.morph_count == 0 {
        return vec![];
    }
    vec![ctrl.global_visibility * ctrl.dilation; ctrl.morph_count]
}

/// Enable or disable.
pub fn vv_set_enabled(ctrl: &mut VascularVisibility, enabled: bool) {
    ctrl.enabled = enabled;
}

/// Return region override count.
pub fn vv_region_count(ctrl: &VascularVisibility) -> usize {
    ctrl.region_overrides.len()
}

/// Serialize to JSON-like string.
pub fn vv_to_json(ctrl: &VascularVisibility) -> String {
    format!(
        r#"{{"global_visibility":{},"dilation":{},"morph_count":{},"enabled":{}}}"#,
        ctrl.global_visibility, ctrl.dilation, ctrl.morph_count, ctrl.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_visibility() {
        let c = new_vascular_visibility(4);
        assert!((c.global_visibility - 0.2).abs() < 1e-6 /* default visibility must be 0.2 */);
    }

    #[test]
    fn test_set_visibility_clamps() {
        let mut c = new_vascular_visibility(4);
        vv_set_visibility(&mut c, 2.0);
        assert!((c.global_visibility - 1.0).abs() < 1e-6 /* visibility clamped to 1.0 */);
    }

    #[test]
    fn test_set_region() {
        let mut c = new_vascular_visibility(4);
        vv_set_region(&mut c, VeinRegion::Forearm, 0.8);
        assert_eq!(
            vv_region_count(&c),
            1 /* one region override must be added */
        );
    }

    #[test]
    fn test_set_dilation() {
        let mut c = new_vascular_visibility(4);
        vv_set_dilation(&mut c, 0.7);
        assert!((c.dilation - 0.7).abs() < 1e-5 /* dilation must be set */);
    }

    #[test]
    fn test_evaluate_length() {
        let c = new_vascular_visibility(6);
        assert_eq!(
            vv_evaluate(&c).len(),
            6 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut c = new_vascular_visibility(4);
        vv_set_enabled(&mut c, false);
        assert!(vv_evaluate(&c).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_visibility() {
        let c = new_vascular_visibility(4);
        let j = vv_to_json(&c);
        assert!(j.contains("\"global_visibility\"") /* JSON must contain global_visibility */);
    }

    #[test]
    fn test_enabled_default() {
        let c = new_vascular_visibility(4);
        assert!(c.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_product() {
        let mut c = new_vascular_visibility(2);
        vv_set_visibility(&mut c, 0.5);
        vv_set_dilation(&mut c, 0.6);
        let out = vv_evaluate(&c);
        assert!((out[0] - 0.3).abs() < 1e-5 /* weight must be visibility * dilation */);
    }
}
