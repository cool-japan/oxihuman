// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Core temperature homeostasis model.

pub struct ThermoCore {
    pub core_temp_c: f32,
    pub skin_temp_c: f32,
    pub metabolic_rate_w: f32,
    pub ambient_temp_c: f32,
    pub thermal_conductance: f32,
}

pub fn new_thermocore() -> ThermoCore {
    ThermoCore {
        core_temp_c: 37.0,
        skin_temp_c: 34.0,
        metabolic_rate_w: 80.0,
        ambient_temp_c: 22.0,
        thermal_conductance: 5.0,
    }
}

pub fn thermo_step(t: &mut ThermoCore, dt: f32) {
    /* heat balance: metabolic production - conductive skin loss */
    let skin_loss = t.thermal_conductance * (t.core_temp_c - t.skin_temp_c);
    let ambient_loss = t.thermal_conductance * 0.5 * (t.skin_temp_c - t.ambient_temp_c);
    /* thermal mass ~ 70 kg * 3500 J/(kg·K) = 245000 J/K */
    let thermal_mass = 245_000.0_f32;
    let net_core = t.metabolic_rate_w - skin_loss;
    t.core_temp_c += net_core / thermal_mass * dt;
    t.skin_temp_c += (skin_loss - ambient_loss) / (thermal_mass * 0.1) * dt;
}

pub fn thermo_is_normal(t: &ThermoCore) -> bool {
    t.core_temp_c >= 36.0 && t.core_temp_c <= 38.0
}

pub fn thermo_is_hyperthermia(t: &ThermoCore) -> bool {
    t.core_temp_c > 38.5
}

pub fn thermo_is_hypothermia(t: &ThermoCore) -> bool {
    t.core_temp_c < 35.0
}

pub fn thermo_heat_loss_w(t: &ThermoCore) -> f32 {
    t.thermal_conductance * (t.skin_temp_c - t.ambient_temp_c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_thermocore() {
        /* normal thermocore starts in normal range */
        let t = new_thermocore();
        assert!(thermo_is_normal(&t));
    }

    #[test]
    fn test_thermo_step_does_not_change_much() {
        /* small dt does not radically change temperature */
        let mut t = new_thermocore();
        thermo_step(&mut t, 1.0);
        assert!((t.core_temp_c - 37.0).abs() < 0.1);
    }

    #[test]
    fn test_thermo_is_normal() {
        /* default state is normal */
        let t = new_thermocore();
        assert!(thermo_is_normal(&t));
        assert!(!thermo_is_hyperthermia(&t));
        assert!(!thermo_is_hypothermia(&t));
    }

    #[test]
    fn test_thermo_is_hyperthermia() {
        /* high core temp triggers hyperthermia */
        let mut t = new_thermocore();
        t.core_temp_c = 39.0;
        assert!(thermo_is_hyperthermia(&t));
    }

    #[test]
    fn test_thermo_is_hypothermia() {
        /* low core temp triggers hypothermia */
        let mut t = new_thermocore();
        t.core_temp_c = 34.0;
        assert!(thermo_is_hypothermia(&t));
    }

    #[test]
    fn test_thermo_heat_loss() {
        /* heat loss is positive when skin is warmer than ambient */
        let t = new_thermocore();
        assert!(thermo_heat_loss_w(&t) > 0.0);
    }

    #[test]
    fn test_thermo_conductance_stored() {
        /* conductance is stored correctly */
        let t = new_thermocore();
        assert!((t.thermal_conductance - 5.0).abs() < 1e-5);
    }
}
