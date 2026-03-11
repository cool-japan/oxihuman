// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Piercing deformation morph stub.

/// Body location of a piercing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PiercingLocation {
    EarLobe,
    Nostril,
    Septum,
    Eyebrow,
    Lip,
    Navel,
    Other,
}

/// A single piercing entry.
#[derive(Debug, Clone)]
pub struct PiercingEntry {
    pub id: u32,
    pub location: PiercingLocation,
    pub gauge: f32,
    pub deform_radius: f32,
}

/// Piercing deformation morph controller.
#[derive(Debug, Clone)]
pub struct PiercingDeform {
    pub piercings: Vec<PiercingEntry>,
    pub morph_count: usize,
    pub enabled: bool,
}

impl PiercingDeform {
    pub fn new(morph_count: usize) -> Self {
        PiercingDeform {
            piercings: Vec::new(),
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new piercing deform controller.
pub fn new_piercing_deform(morph_count: usize) -> PiercingDeform {
    PiercingDeform::new(morph_count)
}

/// Add a piercing.
pub fn pd_add_piercing(ctrl: &mut PiercingDeform, entry: PiercingEntry) {
    ctrl.piercings.push(entry);
}

/// Remove a piercing by id.
pub fn pd_remove_piercing(ctrl: &mut PiercingDeform, id: u32) {
    ctrl.piercings.retain(|p| p.id != id);
}

/// Evaluate morph weights (stub: sum of deform radii capped at 1).
pub fn pd_evaluate(ctrl: &PiercingDeform) -> Vec<f32> {
    /* Stub: sum deform radii, capped at 1.0 */
    if !ctrl.enabled || ctrl.morph_count == 0 {
        return vec![];
    }
    let w: f32 = ctrl
        .piercings
        .iter()
        .map(|p| p.deform_radius)
        .sum::<f32>()
        .min(1.0);
    vec![w; ctrl.morph_count]
}

/// Enable or disable.
pub fn pd_set_enabled(ctrl: &mut PiercingDeform, enabled: bool) {
    ctrl.enabled = enabled;
}

/// Return piercing count.
pub fn pd_piercing_count(ctrl: &PiercingDeform) -> usize {
    ctrl.piercings.len()
}

/// Serialize to JSON-like string.
pub fn pd_to_json(ctrl: &PiercingDeform) -> String {
    format!(
        r#"{{"piercing_count":{},"morph_count":{},"enabled":{}}}"#,
        ctrl.piercings.len(),
        ctrl.morph_count,
        ctrl.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_piercing(id: u32) -> PiercingEntry {
        PiercingEntry {
            id,
            location: PiercingLocation::EarLobe,
            gauge: 1.0,
            deform_radius: 0.1,
        }
    }

    #[test]
    fn test_initial_empty() {
        let c = new_piercing_deform(4);
        assert_eq!(pd_piercing_count(&c), 0 /* no piercings initially */);
    }

    #[test]
    fn test_add_piercing() {
        let mut c = new_piercing_deform(4);
        pd_add_piercing(&mut c, make_piercing(1));
        assert_eq!(pd_piercing_count(&c), 1 /* one piercing after add */);
    }

    #[test]
    fn test_remove_piercing() {
        let mut c = new_piercing_deform(4);
        pd_add_piercing(&mut c, make_piercing(1));
        pd_remove_piercing(&mut c, 1);
        assert_eq!(pd_piercing_count(&c), 0 /* piercing removed */);
    }

    #[test]
    fn test_evaluate_length() {
        let c = new_piercing_deform(6);
        assert_eq!(
            pd_evaluate(&c).len(),
            6 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut c = new_piercing_deform(4);
        pd_set_enabled(&mut c, false);
        assert!(pd_evaluate(&c).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_evaluate_capped() {
        let mut c = new_piercing_deform(2);
        for i in 0..20 {
            pd_add_piercing(
                &mut c,
                PiercingEntry {
                    id: i,
                    location: PiercingLocation::Other,
                    gauge: 1.0,
                    deform_radius: 0.2,
                },
            );
        }
        let out = pd_evaluate(&c);
        assert!(out[0] <= 1.0 /* weight must not exceed 1.0 */);
    }

    #[test]
    fn test_to_json_has_count() {
        let c = new_piercing_deform(4);
        let j = pd_to_json(&c);
        assert!(j.contains("\"piercing_count\"") /* JSON must contain piercing_count */);
    }

    #[test]
    fn test_enabled_default() {
        let c = new_piercing_deform(4);
        assert!(c.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_remove_nonexistent_is_noop() {
        let mut c = new_piercing_deform(4);
        pd_remove_piercing(&mut c, 999);
        assert_eq!(
            pd_piercing_count(&c),
            0 /* removing nonexistent must be noop */
        );
    }
}
