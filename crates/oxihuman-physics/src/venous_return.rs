// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct VenousSystem {
    pub unstressed_volume_ml: f32,
    pub stressed_volume_ml: f32,
    pub compliance_ml_per_mmhg: f32,
    pub resistance: f32,
}

pub fn new_venous_system() -> VenousSystem {
    VenousSystem {
        unstressed_volume_ml: 2500.0,
        stressed_volume_ml: 500.0,
        compliance_ml_per_mmhg: 100.0,
        resistance: 1.0,
    }
}

pub fn venous_pressure_mmhg(v: &VenousSystem) -> f32 {
    v.stressed_volume_ml / v.compliance_ml_per_mmhg.max(1e-9)
}

pub fn venous_return_flow(v: &VenousSystem, right_atrial_pressure: f32) -> f32 {
    let p_ms = venous_pressure_mmhg(v);
    let delta_p = p_ms - right_atrial_pressure;
    (delta_p / v.resistance.max(1e-9)).max(0.0)
}

pub fn venous_shift_volume(v: &mut VenousSystem, delta_ml: f32) {
    v.stressed_volume_ml += delta_ml;
    v.stressed_volume_ml = v.stressed_volume_ml.max(0.0);
}

pub fn venous_cardiac_output_balance(v: &VenousSystem, co_ml_per_min: f32, dt: f32) -> f32 {
    let vr = venous_return_flow(v, 5.0); /* assume fixed RAP=5 mmHg */
    (vr * 60.0 - co_ml_per_min) * dt / 60.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_pressure_positive() {
        /* mean systemic pressure is positive */
        let v = new_venous_system();
        assert!(venous_pressure_mmhg(&v) > 0.0);
    }

    #[test]
    fn venous_return_positive() {
        /* venous return positive when systemic > atrial pressure */
        let v = new_venous_system();
        let vr = venous_return_flow(&v, 0.0);
        assert!(vr > 0.0);
    }

    #[test]
    fn return_zero_when_pressure_equal() {
        /* no return when systemic pressure equals atrial */
        let mut v = new_venous_system();
        v.stressed_volume_ml = 0.5; /* very low → low pressure */
        let p = venous_pressure_mmhg(&v);
        let vr = venous_return_flow(&v, p);
        assert!(vr <= 0.001);
    }

    #[test]
    fn shift_volume_changes_pressure() {
        /* shifting volume changes pressure */
        let mut v = new_venous_system();
        let p0 = venous_pressure_mmhg(&v);
        venous_shift_volume(&mut v, 100.0);
        let p1 = venous_pressure_mmhg(&v);
        assert!(p1 > p0);
    }

    #[test]
    fn shift_volume_clamped_positive() {
        /* stressed volume cannot go negative */
        let mut v = new_venous_system();
        venous_shift_volume(&mut v, -10000.0);
        assert!(v.stressed_volume_ml >= 0.0);
    }

    #[test]
    fn cardiac_output_balance_finite() {
        /* balance function returns finite value */
        let v = new_venous_system();
        let b = venous_cardiac_output_balance(&v, 5000.0, 0.01);
        assert!(b.is_finite());
    }

    #[test]
    fn compliance_positive() {
        /* compliance is positive */
        let v = new_venous_system();
        assert!(v.compliance_ml_per_mmhg > 0.0);
    }
}
