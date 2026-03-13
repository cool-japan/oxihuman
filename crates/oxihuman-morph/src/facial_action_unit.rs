// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! FACS (Facial Action Coding System) action unit controller.
//!
//! Maps numeric AU codes to morph weights.  This module provides a lightweight
//! runtime state (`FacsCtrlState`) that tracks per-AU weights independently of
//! the enum-based `facs` module.

#[allow(dead_code)]
/// A single action unit with its numeric code, human-readable name and weight.
pub struct ActionUnitEntry {
    pub code: u8,
    pub name: String,
    pub weight: f32,
}

#[allow(dead_code)]
/// Configuration for the FACS controller.
pub struct FacsConfig {
    /// Maximum weight clamp applied to individual AUs.
    pub max_weight: f32,
    /// Minimum weight clamp applied to individual AUs.
    pub min_weight: f32,
}

#[allow(dead_code)]
/// Runtime state holding all registered action units and their current weights.
pub struct FacsCtrlState {
    pub units: Vec<ActionUnitEntry>,
    pub config: FacsConfig,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

#[allow(dead_code)]
/// Returns a sensible default [`FacsConfig`].
pub fn default_facs_config() -> FacsConfig {
    FacsConfig {
        max_weight: 1.0,
        min_weight: 0.0,
    }
}

#[allow(dead_code)]
/// Creates a new [`FacsCtrlState`] using `cfg`.
pub fn new_facs_state(cfg: &FacsConfig) -> FacsCtrlState {
    FacsCtrlState {
        units: Vec::new(),
        config: FacsConfig {
            max_weight: cfg.max_weight,
            min_weight: cfg.min_weight,
        },
    }
}

#[allow(dead_code)]
/// Registers an action unit with the given `code` and `name`.
/// If the code is already registered, the name is updated.
pub fn facs_register_au(state: &mut FacsCtrlState, code: u8, name: &str) {
    if let Some(entry) = state.units.iter_mut().find(|u| u.code == code) {
        entry.name = name.to_string();
    } else {
        state.units.push(ActionUnitEntry {
            code,
            name: name.to_string(),
            weight: 0.0,
        });
    }
}

#[allow(dead_code)]
/// Sets the weight of the AU identified by `code`, clamped to the config range.
pub fn facs_set_au_weight(state: &mut FacsCtrlState, code: u8, weight: f32) {
    let clamped = weight.clamp(state.config.min_weight, state.config.max_weight);
    if let Some(entry) = state.units.iter_mut().find(|u| u.code == code) {
        entry.weight = clamped;
    }
}

#[allow(dead_code)]
/// Returns the weight of the AU identified by `code`, or 0.0 if not found.
pub fn facs_get_au_weight(state: &FacsCtrlState, code: u8) -> f32 {
    state
        .units
        .iter()
        .find(|u| u.code == code)
        .map_or(0.0, |u| u.weight)
}

#[allow(dead_code)]
/// Returns the codes of all AUs with a weight greater than 0.0.
pub fn facs_active_units(state: &FacsCtrlState) -> Vec<u8> {
    state
        .units
        .iter()
        .filter(|u| u.weight > 0.0)
        .map(|u| u.code)
        .collect()
}

#[allow(dead_code)]
/// Returns the total number of registered action units.
pub fn facs_unit_count(state: &FacsCtrlState) -> usize {
    state.units.len()
}

#[allow(dead_code)]
/// Resets all AU weights to 0.0.
pub fn facs_reset(state: &mut FacsCtrlState) {
    for u in &mut state.units {
        u.weight = 0.0;
    }
}

#[allow(dead_code)]
/// Returns a flat vector of weights in registration order.
pub fn facs_to_morph_vector(state: &FacsCtrlState) -> Vec<f32> {
    state.units.iter().map(|u| u.weight).collect()
}

#[allow(dead_code)]
/// Returns the name of the AU identified by `code`, or `None` if not found.
pub fn facs_au_name(state: &FacsCtrlState, code: u8) -> Option<&str> {
    state
        .units
        .iter()
        .find(|u| u.code == code)
        .map(|u| u.name.as_str())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> FacsCtrlState {
        let cfg = default_facs_config();
        let mut s = new_facs_state(&cfg);
        facs_register_au(&mut s, 1, "Inner Brow Raise");
        facs_register_au(&mut s, 2, "Outer Brow Raise");
        facs_register_au(&mut s, 4, "Brow Lowerer");
        s
    }

    #[test]
    fn test_default_config() {
        let cfg = default_facs_config();
        assert_eq!(cfg.max_weight, 1.0);
        assert_eq!(cfg.min_weight, 0.0);
    }

    #[test]
    fn test_register_au() {
        let s = make_state();
        assert_eq!(facs_unit_count(&s), 3);
    }

    #[test]
    fn test_set_get_weight() {
        let mut s = make_state();
        facs_set_au_weight(&mut s, 1, 0.75);
        assert!((facs_get_au_weight(&s, 1) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_weight_clamped() {
        let mut s = make_state();
        facs_set_au_weight(&mut s, 1, 2.0);
        assert_eq!(facs_get_au_weight(&s, 1), 1.0);
        facs_set_au_weight(&mut s, 1, -0.5);
        assert_eq!(facs_get_au_weight(&s, 1), 0.0);
    }

    #[test]
    fn test_active_units() {
        let mut s = make_state();
        assert!(facs_active_units(&s).is_empty());
        facs_set_au_weight(&mut s, 1, 0.5);
        facs_set_au_weight(&mut s, 4, 1.0);
        let active = facs_active_units(&s);
        assert_eq!(active.len(), 2);
        assert!(active.contains(&1));
        assert!(active.contains(&4));
    }

    #[test]
    fn test_facs_reset() {
        let mut s = make_state();
        facs_set_au_weight(&mut s, 1, 0.8);
        facs_reset(&mut s);
        assert_eq!(facs_get_au_weight(&s, 1), 0.0);
        assert!(facs_active_units(&s).is_empty());
    }

    #[test]
    fn test_morph_vector_length() {
        let s = make_state();
        assert_eq!(facs_to_morph_vector(&s).len(), 3);
    }

    #[test]
    fn test_au_name() {
        let s = make_state();
        assert_eq!(facs_au_name(&s, 1), Some("Inner Brow Raise"));
        assert!(facs_au_name(&s, 99).is_none());
    }

    #[test]
    fn test_register_duplicate_updates_name() {
        let mut s = make_state();
        facs_register_au(&mut s, 1, "Updated Name");
        assert_eq!(facs_unit_count(&s), 3); // count unchanged
        assert_eq!(facs_au_name(&s, 1), Some("Updated Name"));
    }

    #[test]
    fn test_get_unknown_code_returns_zero() {
        let s = make_state();
        assert_eq!(facs_get_au_weight(&s, 42), 0.0);
    }
}
