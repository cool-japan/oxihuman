// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pneumatic body: pressure-volume gas dynamics using ideal gas law.

const R_GAS: f32 = 8.314; // J/(mol·K)

/// A pneumatic (gas-filled) body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PneumaticBody {
    pub volume: f32,
    pub pressure: f32,
    pub temperature: f32,
    pub moles: f32,
    pub max_pressure: f32,
    pub min_volume: f32,
    pub burst: bool,
}

/// Create a new `PneumaticBody`.
#[allow(dead_code)]
pub fn new_pneumatic_body(volume: f32, pressure: f32, temperature: f32) -> PneumaticBody {
    let moles = pressure * volume / (R_GAS * temperature.max(1e-3));
    PneumaticBody {
        volume: volume.max(1e-6),
        pressure,
        temperature: temperature.max(1e-3),
        moles,
        max_pressure: 10.0 * pressure,
        min_volume: volume * 0.01,
        burst: false,
    }
}

/// Update pressure using ideal gas law: P = nRT/V.
#[allow(dead_code)]
pub fn pnb_update_pressure(body: &mut PneumaticBody) {
    if body.burst {
        return;
    }
    body.pressure = body.moles * R_GAS * body.temperature / body.volume.max(body.min_volume);
    if body.pressure > body.max_pressure {
        body.burst = true;
    }
}

/// Compress (reduce volume) and update pressure.
#[allow(dead_code)]
pub fn pnb_compress(body: &mut PneumaticBody, delta_v: f32) {
    body.volume = (body.volume - delta_v.max(0.0)).max(body.min_volume);
    pnb_update_pressure(body);
}

/// Expand (increase volume) and update pressure.
#[allow(dead_code)]
pub fn pnb_expand(body: &mut PneumaticBody, delta_v: f32) {
    body.volume += delta_v.max(0.0);
    pnb_update_pressure(body);
}

/// Add heat (isothermal approximation: temperature increases → pressure increases).
#[allow(dead_code)]
pub fn pnb_heat(body: &mut PneumaticBody, delta_t: f32) {
    body.temperature += delta_t;
    body.temperature = body.temperature.max(1e-3);
    pnb_update_pressure(body);
}

/// Add gas (increase moles).
#[allow(dead_code)]
pub fn pnb_add_gas(body: &mut PneumaticBody, delta_mol: f32) {
    body.moles += delta_mol.max(0.0);
    pnb_update_pressure(body);
}

/// Vent gas (reduce moles).
#[allow(dead_code)]
pub fn pnb_vent(body: &mut PneumaticBody, delta_mol: f32) {
    body.moles = (body.moles - delta_mol.max(0.0)).max(0.0);
    pnb_update_pressure(body);
}

/// Net outward force on a surface of area `area`.
#[allow(dead_code)]
pub fn pnb_force_on_surface(body: &PneumaticBody, area: f32, external_pressure: f32) -> f32 {
    (body.pressure - external_pressure) * area
}

/// Whether the body has burst.
#[allow(dead_code)]
pub fn pnb_is_burst(body: &PneumaticBody) -> bool {
    body.burst
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new_pneumatic() {
        let body = new_pneumatic_body(1.0, 101325.0, 293.0);
        assert!(!pnb_is_burst(&body));
        assert!(body.pressure > 0.0);
    }

    #[test]
    fn test_compress_increases_pressure() {
        let mut body = new_pneumatic_body(1.0, 101325.0, 293.0);
        let p0 = body.pressure;
        pnb_compress(&mut body, 0.1);
        assert!(body.pressure > p0);
    }

    #[test]
    fn test_expand_decreases_pressure() {
        let mut body = new_pneumatic_body(1.0, 101325.0, 293.0);
        let p0 = body.pressure;
        pnb_expand(&mut body, 0.5);
        assert!(body.pressure < p0);
    }

    #[test]
    fn test_heat_increases_pressure() {
        let mut body = new_pneumatic_body(1.0, 101325.0, 293.0);
        let p0 = body.pressure;
        pnb_heat(&mut body, 100.0);
        assert!(body.pressure > p0);
    }

    #[test]
    fn test_add_gas_increases_pressure() {
        let mut body = new_pneumatic_body(1.0, 101325.0, 293.0);
        let p0 = body.pressure;
        pnb_add_gas(&mut body, 0.1);
        assert!(body.pressure > p0);
    }

    #[test]
    fn test_vent_reduces_pressure() {
        let mut body = new_pneumatic_body(1.0, 101325.0, 293.0);
        let p0 = body.pressure;
        pnb_vent(&mut body, 0.01);
        assert!(body.pressure < p0);
    }

    #[test]
    fn test_burst_condition() {
        let mut body = new_pneumatic_body(1.0, 101325.0, 293.0);
        body.max_pressure = 1.0;
        pnb_update_pressure(&mut body);
        assert!(pnb_is_burst(&body));
    }

    #[test]
    fn test_force_on_surface() {
        let body = new_pneumatic_body(1.0, 101325.0, 293.0);
        let f = pnb_force_on_surface(&body, 0.01, 101325.0);
        assert!(f.abs() < 1.0);
    }

    #[test]
    fn test_pi_used() {
        let circle_area = PI * 1.0 * 1.0;
        assert!(circle_area > 3.0);
    }

    #[test]
    fn test_volume_clamp() {
        let body = new_pneumatic_body(1e-9, 1.0, 293.0);
        assert!(body.volume >= 1e-6);
    }
}
