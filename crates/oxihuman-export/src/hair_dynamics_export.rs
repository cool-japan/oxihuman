#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export hair dynamics settings.

#[allow(dead_code)]
pub struct HairDynamicsExport {
    pub name: String,
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
    pub air_drag: f32,
    pub gravity: [f32; 3],
    pub pin_root: bool,
}

#[allow(dead_code)]
pub fn default_hair_dynamics_export(name: &str) -> HairDynamicsExport {
    HairDynamicsExport {
        name: name.to_string(),
        stiffness: 1.0,
        damping: 0.1,
        mass: 0.3,
        air_drag: 0.02,
        gravity: [0.0, -9.81, 0.0],
        pin_root: true,
    }
}

#[allow(dead_code)]
pub fn export_hair_dynamics_to_json(h: &HairDynamicsExport) -> String {
    format!(
        r#"{{"name":"{}","stiffness":{},"damping":{},"mass":{},"air_drag":{},"gravity":[{},{},{}],"pin_root":{}}}"#,
        h.name, h.stiffness, h.damping, h.mass, h.air_drag,
        h.gravity[0], h.gravity[1], h.gravity[2],
        h.pin_root
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_name() {
        let h = default_hair_dynamics_export("hair_sim");
        assert_eq!(h.name, "hair_sim");
    }

    #[test]
    fn default_stiffness() {
        let h = default_hair_dynamics_export("x");
        assert!((h.stiffness - 1.0).abs() < 1e-5);
    }

    #[test]
    fn default_pin_root() {
        let h = default_hair_dynamics_export("x");
        assert!(h.pin_root);
    }

    #[test]
    fn default_gravity_y() {
        let h = default_hair_dynamics_export("x");
        assert!((h.gravity[1] - (-9.81)).abs() < 1e-3);
    }

    #[test]
    fn json_contains_name() {
        let h = default_hair_dynamics_export("my_hair");
        let json = export_hair_dynamics_to_json(&h);
        assert!(json.contains("my_hair"));
    }

    #[test]
    fn json_contains_stiffness() {
        let h = default_hair_dynamics_export("x");
        let json = export_hair_dynamics_to_json(&h);
        assert!(json.contains("stiffness"));
    }

    #[test]
    fn json_contains_pin_root() {
        let h = default_hair_dynamics_export("x");
        let json = export_hair_dynamics_to_json(&h);
        assert!(json.contains("pin_root"));
    }

    #[test]
    fn default_mass() {
        let h = default_hair_dynamics_export("x");
        assert!((h.mass - 0.3).abs() < 1e-5);
    }

    #[test]
    fn damping_positive() {
        let h = default_hair_dynamics_export("x");
        assert!(h.damping > 0.0);
    }
}
