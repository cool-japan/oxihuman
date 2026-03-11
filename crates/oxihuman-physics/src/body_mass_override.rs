#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::HashMap;

/// Mass override storage for physics bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MassOverride {
    overrides: HashMap<u32, (f32, f32)>, // body_id -> (mass, inertia)
}

#[allow(dead_code)]
pub fn new_mass_override() -> MassOverride {
    MassOverride {
        overrides: HashMap::new(),
    }
}

#[allow(dead_code)]
pub fn set_mass_override(mo: &mut MassOverride, body_id: u32, mass: f32) {
    let inertia = mo.overrides.get(&body_id).map_or(0.0, |&(_, i)| i);
    mo.overrides.insert(body_id, (mass, inertia));
}

#[allow(dead_code)]
pub fn get_mass_override(mo: &MassOverride, body_id: u32) -> Option<f32> {
    mo.overrides.get(&body_id).map(|&(m, _)| m)
}

#[allow(dead_code)]
pub fn clear_mass_override(mo: &mut MassOverride, body_id: u32) -> bool {
    mo.overrides.remove(&body_id).is_some()
}

#[allow(dead_code)]
pub fn has_mass_override(mo: &MassOverride, body_id: u32) -> bool {
    mo.overrides.contains_key(&body_id)
}

#[allow(dead_code)]
pub fn override_inertia(mo: &mut MassOverride, body_id: u32, inertia: f32) {
    let mass = mo.overrides.get(&body_id).map_or(1.0, |&(m, _)| m);
    mo.overrides.insert(body_id, (mass, inertia));
}

#[allow(dead_code)]
pub fn override_to_json(mo: &MassOverride) -> String {
    let items: Vec<String> = mo
        .overrides
        .iter()
        .map(|(id, (m, i))| format!("{{\"id\":{},\"mass\":{:.6},\"inertia\":{:.6}}}", id, m, i))
        .collect();
    format!("{{\"overrides\":[{}]}}", items.join(","))
}

#[allow(dead_code)]
pub fn override_count(mo: &MassOverride) -> usize {
    mo.overrides.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mass_override() {
        let mo = new_mass_override();
        assert_eq!(override_count(&mo), 0);
    }

    #[test]
    fn test_set_mass_override() {
        let mut mo = new_mass_override();
        set_mass_override(&mut mo, 1, 10.0);
        assert!(has_mass_override(&mo, 1));
    }

    #[test]
    fn test_get_mass_override() {
        let mut mo = new_mass_override();
        set_mass_override(&mut mo, 1, 10.0);
        assert!((get_mass_override(&mo, 1).unwrap() - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_get_mass_override_missing() {
        let mo = new_mass_override();
        assert!(get_mass_override(&mo, 999).is_none());
    }

    #[test]
    fn test_clear_mass_override() {
        let mut mo = new_mass_override();
        set_mass_override(&mut mo, 1, 10.0);
        assert!(clear_mass_override(&mut mo, 1));
        assert!(!has_mass_override(&mo, 1));
    }

    #[test]
    fn test_clear_missing() {
        let mut mo = new_mass_override();
        assert!(!clear_mass_override(&mut mo, 999));
    }

    #[test]
    fn test_has_mass_override() {
        let mut mo = new_mass_override();
        set_mass_override(&mut mo, 1, 5.0);
        assert!(has_mass_override(&mo, 1));
        assert!(!has_mass_override(&mo, 2));
    }

    #[test]
    fn test_override_inertia() {
        let mut mo = new_mass_override();
        set_mass_override(&mut mo, 1, 10.0);
        override_inertia(&mut mo, 1, 5.0);
        assert!(has_mass_override(&mo, 1));
    }

    #[test]
    fn test_override_to_json() {
        let mut mo = new_mass_override();
        set_mass_override(&mut mo, 1, 10.0);
        let json = override_to_json(&mo);
        assert!(json.contains("\"mass\""));
    }

    #[test]
    fn test_override_count() {
        let mut mo = new_mass_override();
        set_mass_override(&mut mo, 1, 10.0);
        set_mass_override(&mut mo, 2, 20.0);
        assert_eq!(override_count(&mo), 2);
    }
}
