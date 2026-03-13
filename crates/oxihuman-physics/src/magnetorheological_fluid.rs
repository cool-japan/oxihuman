// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Magnetorheological (MR) fluid viscosity model (Bingham model stub).

/// An MR fluid damper.
#[derive(Debug, Clone)]
pub struct MrFluid {
    /// Applied magnetic field strength (kA/m).
    pub field: f32,
    /// Zero-field dynamic viscosity (Pa·s).
    pub base_viscosity: f32,
    /// Field-dependent yield stress coefficient (Pa per kA/m).
    pub yield_stress_coeff: f32,
    /// Current shear rate (1/s).
    pub shear_rate: f32,
}

impl MrFluid {
    pub fn new(base_viscosity: f32) -> Self {
        MrFluid {
            field: 0.0,
            base_viscosity,
            yield_stress_coeff: 5.0, /* Pa per kA/m */
            shear_rate: 0.0,
        }
    }
}

/// Create a new MR fluid model.
pub fn new_mr_fluid(base_viscosity: f32) -> MrFluid {
    MrFluid::new(base_viscosity)
}

/// Field-induced yield stress (Pa).
pub fn mr_yield_stress(m: &MrFluid) -> f32 {
    m.yield_stress_coeff * m.field
}

/// Bingham plastic shear stress (Pa).
pub fn mr_shear_stress(m: &MrFluid) -> f32 {
    let tau_y = mr_yield_stress(m);
    if m.shear_rate.abs() < 1e-12 {
        /* no flow below yield stress */
        return tau_y;
    }
    tau_y + m.base_viscosity * m.shear_rate
}

/// Effective viscosity (Pa·s) = shear stress / shear rate.
pub fn mr_effective_viscosity(m: &MrFluid) -> f32 {
    if m.shear_rate.abs() < 1e-12 {
        return f32::INFINITY;
    }
    mr_shear_stress(m) / m.shear_rate.abs()
}

/// Set the magnetic field (kA/m).
pub fn mr_set_field(m: &mut MrFluid, field: f32) {
    m.field = field.max(0.0);
}

/// Set the shear rate (1/s).
pub fn mr_set_shear_rate(m: &mut MrFluid, rate: f32) {
    m.shear_rate = rate;
}

/// Return the damping force (N) for a piston area `area` (m²).
pub fn mr_damping_force(m: &MrFluid, area: f32, velocity: f32) -> f32 {
    mr_set_shear_rate(&mut m.clone(), velocity * 100.0); /* stub: rate ≈ v/gap */
    let mut mc = m.clone();
    mc.shear_rate = velocity * 100.0;
    mr_shear_stress(&mc) * area
}

/// Return `true` if the fluid is in the "off" (low-viscosity) state.
pub fn mr_is_off(m: &MrFluid) -> bool {
    m.field < 1e-3
}

/// Return `true` if the fluid is in the "on" (high-viscosity) state.
pub fn mr_is_on(m: &MrFluid) -> bool {
    m.field >= 50.0
}

/// Return the power consumed by the electromagnet (W) for coil resistance `r` (Ω) and inductance.
pub fn mr_coil_power(current: f32, resistance: f32) -> f32 {
    current * current * resistance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fluid_off_state() {
        let m = new_mr_fluid(0.3);
        assert!(mr_is_off(&m));
    }

    #[test]
    fn test_yield_stress_zero_without_field() {
        let m = new_mr_fluid(0.3);
        assert!(mr_yield_stress(&m).abs() < 1e-5);
    }

    #[test]
    fn test_yield_stress_increases_with_field() {
        let mut m = new_mr_fluid(0.3);
        mr_set_field(&mut m, 100.0);
        assert!(mr_yield_stress(&m) > 0.0);
    }

    #[test]
    fn test_shear_stress_at_zero_rate() {
        let mut m = new_mr_fluid(0.3);
        mr_set_field(&mut m, 50.0);
        let tau = mr_shear_stress(&m);
        assert!(tau > 0.0);
    }

    #[test]
    fn test_effective_viscosity_increases_with_field() {
        let mut m1 = new_mr_fluid(0.3);
        let mut m2 = new_mr_fluid(0.3);
        mr_set_shear_rate(&mut m1, 100.0);
        mr_set_shear_rate(&mut m2, 100.0);
        mr_set_field(&mut m2, 100.0);
        assert!(mr_effective_viscosity(&m2) > mr_effective_viscosity(&m1));
    }

    #[test]
    fn test_damping_force_positive() {
        let mut m = new_mr_fluid(0.3);
        mr_set_field(&mut m, 100.0);
        let f = mr_damping_force(&m, 0.001, 0.5);
        assert!(f > 0.0);
    }

    #[test]
    fn test_field_clamp_nonnegative() {
        let mut m = new_mr_fluid(0.3);
        mr_set_field(&mut m, -50.0);
        assert_eq!(m.field, 0.0);
    }

    #[test]
    fn test_coil_power() {
        let p = mr_coil_power(2.0, 5.0);
        assert!((p - 20.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_on_when_high_field() {
        let mut m = new_mr_fluid(0.3);
        mr_set_field(&mut m, 100.0);
        assert!(mr_is_on(&m));
    }
}
