// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/* New lung mechanics module (v2) with full struct and free functions
as requested for Wave 149A. The existing lung_mechanics.rs models
a simpler Lung struct; this one follows the spec more closely. */

pub struct LungMechanicsV2 {
    pub compliance_l_per_cmh2o: f32,
    pub resistance_cmh2o_per_l_s: f32,
    pub frc_l: f32,
    pub volume_l: f32,
    pub flow_l_per_s: f32,
}

pub fn new_lung_mechanics_v2() -> LungMechanicsV2 {
    LungMechanicsV2 {
        compliance_l_per_cmh2o: 0.2,
        resistance_cmh2o_per_l_s: 2.0,
        frc_l: 2.5,
        volume_l: 2.5,
        flow_l_per_s: 0.0,
    }
}

pub fn lung_v2_driving_pressure(l: &LungMechanicsV2) -> f32 {
    (l.volume_l - l.frc_l) / l.compliance_l_per_cmh2o.max(1e-9)
}

pub fn lung_v2_flow_step(l: &mut LungMechanicsV2, pressure_cmh2o: f32, dt: f32) {
    let elastic_p = lung_v2_driving_pressure(l);
    let net_p = pressure_cmh2o - elastic_p;
    let flow = net_p / l.resistance_cmh2o_per_l_s.max(1e-9);
    l.flow_l_per_s = flow;
    l.volume_l += flow * dt;
    l.volume_l = l.volume_l.max(0.1);
}

pub fn lung_v2_tidal_volume(l: &LungMechanicsV2) -> f32 {
    (l.volume_l - l.frc_l).abs()
}

pub fn lung_v2_is_hyperinflated(l: &LungMechanicsV2) -> bool {
    l.volume_l > 1.5 * l.frc_l
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_volume_equals_frc() {
        /* initial volume is FRC */
        let l = new_lung_mechanics_v2();
        assert!((l.volume_l - l.frc_l).abs() < 1e-6);
    }

    #[test]
    fn driving_pressure_zero_at_frc() {
        /* no driving pressure at FRC */
        let l = new_lung_mechanics_v2();
        assert!((lung_v2_driving_pressure(&l)).abs() < 1e-4);
    }

    #[test]
    fn flow_step_changes_volume() {
        /* positive pressure drives airflow */
        let mut l = new_lung_mechanics_v2();
        lung_v2_flow_step(&mut l, 10.0, 0.1);
        assert!(l.volume_l > 2.5);
    }

    #[test]
    fn tidal_volume_nonneg() {
        /* tidal volume is non-negative */
        let l = new_lung_mechanics_v2();
        assert!(lung_v2_tidal_volume(&l) >= 0.0);
    }

    #[test]
    fn not_hyperinflated_at_rest() {
        /* not hyperinflated at FRC */
        let l = new_lung_mechanics_v2();
        assert!(!lung_v2_is_hyperinflated(&l));
    }

    #[test]
    fn hyperinflated_when_volume_large() {
        /* hyperinflated when volume > 1.5*FRC */
        let mut l = new_lung_mechanics_v2();
        l.volume_l = l.frc_l * 2.0;
        assert!(lung_v2_is_hyperinflated(&l));
    }

    #[test]
    fn volume_stays_positive() {
        /* volume clamped positive under suction */
        let mut l = new_lung_mechanics_v2();
        lung_v2_flow_step(&mut l, -1000.0, 10.0);
        assert!(l.volume_l > 0.0);
    }
}
