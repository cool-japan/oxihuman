// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Proximity pinning constraint stub.

/// A proximity pin entry that pins a vertex to a surface point.
#[derive(Debug, Clone)]
pub struct ProximityPin {
    pub vertex_index: usize,
    pub target_position: [f32; 3],
    pub influence: f32,
    pub enabled: bool,
}

impl ProximityPin {
    pub fn new(vertex_index: usize, target: [f32; 3]) -> Self {
        ProximityPin {
            vertex_index,
            target_position: target,
            influence: 1.0,
            enabled: true,
        }
    }
}

/// A set of proximity pins.
#[derive(Debug, Clone, Default)]
pub struct ProximityPinSet {
    pub pins: Vec<ProximityPin>,
}

/// Create a new empty pin set.
pub fn new_pin_set() -> ProximityPinSet {
    ProximityPinSet::default()
}

/// Add a pin.
pub fn pin_add(set: &mut ProximityPinSet, vertex_index: usize, target: [f32; 3]) {
    set.pins.push(ProximityPin::new(vertex_index, target));
}

/// Remove a pin by index.
pub fn pin_remove(set: &mut ProximityPinSet, pin_index: usize) {
    if pin_index < set.pins.len() {
        set.pins.remove(pin_index);
    }
}

/// Set pin influence.
pub fn pin_set_influence(set: &mut ProximityPinSet, pin_index: usize, influence: f32) {
    if pin_index < set.pins.len() {
        set.pins[pin_index].influence = influence.clamp(0.0, 1.0);
    }
}

/// Enable or disable a pin.
pub fn pin_set_enabled(set: &mut ProximityPinSet, pin_index: usize, enabled: bool) {
    if pin_index < set.pins.len() {
        set.pins[pin_index].enabled = enabled;
    }
}

/// Return pin count.
pub fn pin_count(set: &ProximityPinSet) -> usize {
    set.pins.len()
}

/// Return enabled pin count.
pub fn pin_enabled_count(set: &ProximityPinSet) -> usize {
    set.pins.iter().filter(|p| p.enabled).count()
}

/// Return a JSON-like string.
pub fn pin_set_to_json(set: &ProximityPinSet) -> String {
    format!(
        r#"{{"pins":{},"enabled":{}}}"#,
        set.pins.len(),
        set.pins.iter().filter(|p| p.enabled).count()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pin_set_empty() {
        let s = new_pin_set();
        assert_eq!(pin_count(&s), 0 /* new set should be empty */,);
    }

    #[test]
    fn test_add_pin_increases_count() {
        let mut s = new_pin_set();
        pin_add(&mut s, 0, [0.0, 1.0, 0.0]);
        assert_eq!(pin_count(&s), 1 /* count should increase after add */,);
    }

    #[test]
    fn test_remove_pin_decreases_count() {
        let mut s = new_pin_set();
        pin_add(&mut s, 0, [0.0; 3]);
        pin_remove(&mut s, 0);
        assert_eq!(
            pin_count(&s),
            0, /* count should decrease after remove */
        );
    }

    #[test]
    fn test_pin_enabled_by_default() {
        let mut s = new_pin_set();
        pin_add(&mut s, 0, [0.0; 3]);
        assert!(s.pins[0].enabled /* pins are enabled by default */,);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut s = new_pin_set();
        pin_add(&mut s, 0, [0.0; 3]);
        pin_set_enabled(&mut s, 0, false);
        assert!(!s.pins[0].enabled /* pin should be disabled */,);
    }

    #[test]
    fn test_enabled_count() {
        let mut s = new_pin_set();
        pin_add(&mut s, 0, [0.0; 3]);
        pin_add(&mut s, 1, [1.0, 0.0, 0.0]);
        pin_set_enabled(&mut s, 0, false);
        assert_eq!(
            pin_enabled_count(&s),
            1, /* only one pin should be enabled */
        );
    }

    #[test]
    fn test_set_influence_clamps() {
        let mut s = new_pin_set();
        pin_add(&mut s, 0, [0.0; 3]);
        pin_set_influence(&mut s, 0, 3.0);
        assert!((s.pins[0].influence - 1.0).abs() < 1e-5, /* influence clamped to 1 */);
    }

    #[test]
    fn test_to_json_contains_pins() {
        let s = new_pin_set();
        let j = pin_set_to_json(&s);
        assert!(j.contains("pins") /* JSON must contain pins */,);
    }

    #[test]
    fn test_remove_out_of_bounds_ignored() {
        let mut s = new_pin_set();
        pin_remove(&mut s, 99);
        assert_eq!(pin_count(&s), 0 /* out-of-bounds remove is no-op */,);
    }

    #[test]
    fn test_vertex_index_stored() {
        let mut s = new_pin_set();
        pin_add(&mut s, 42, [0.0; 3]);
        assert_eq!(
            s.pins[0].vertex_index,
            42, /* vertex index must match */
        );
    }
}
