// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct EnzymeKinetics {
    pub vmax: f32,
    pub km: f32,
    pub inhibitor_ki: f32,
}

pub fn new_enzyme_kinetics(vmax: f32, km: f32) -> EnzymeKinetics {
    EnzymeKinetics {
        vmax,
        km,
        inhibitor_ki: 1.0,
    }
}

pub fn enzyme_velocity(e: &EnzymeKinetics, substrate: f32) -> f32 {
    if substrate < 0.0 {
        return 0.0;
    }
    e.vmax * substrate / (e.km + substrate)
}

pub fn enzyme_competitive_inhibition(e: &EnzymeKinetics, substrate: f32, inhibitor: f32) -> f32 {
    if substrate < 0.0 {
        return 0.0;
    }
    let km_app = e.km * (1.0 + inhibitor / e.inhibitor_ki);
    e.vmax * substrate / (km_app + substrate)
}

pub fn enzyme_turnover_number(e: &EnzymeKinetics, total_enzyme: f32) -> f32 {
    if total_enzyme < 1e-12 {
        return 0.0;
    }
    e.vmax / total_enzyme
}

pub fn enzyme_half_saturation(e: &EnzymeKinetics) -> f32 {
    e.km
}

pub fn enzyme_is_saturated(e: &EnzymeKinetics, substrate: f32) -> bool {
    /* S >> Km means S / Km > 10 */
    substrate / e.km > 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_enzyme_kinetics() {
        /* create enzyme with vmax and km */
        let e = new_enzyme_kinetics(10.0, 1.0);
        assert_eq!(e.vmax, 10.0);
        assert_eq!(e.km, 1.0);
    }

    #[test]
    fn test_enzyme_velocity_at_km() {
        /* at S = Km, v = Vmax / 2 */
        let e = new_enzyme_kinetics(10.0, 2.0);
        let v = enzyme_velocity(&e, 2.0);
        assert!((v - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_enzyme_velocity_zero_substrate() {
        /* no substrate, no velocity */
        let e = new_enzyme_kinetics(10.0, 1.0);
        assert_eq!(enzyme_velocity(&e, 0.0), 0.0);
    }

    #[test]
    fn test_enzyme_competitive_inhibition() {
        /* inhibitor reduces velocity */
        let e = new_enzyme_kinetics(10.0, 1.0);
        let v_no_inh = enzyme_velocity(&e, 5.0);
        let v_inh = enzyme_competitive_inhibition(&e, 5.0, 1.0);
        assert!(v_inh < v_no_inh);
    }

    #[test]
    fn test_enzyme_turnover_number() {
        /* kcat = Vmax / Et */
        let e = new_enzyme_kinetics(20.0, 1.0);
        let kcat = enzyme_turnover_number(&e, 4.0);
        assert!((kcat - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_enzyme_half_saturation() {
        /* half saturation returns Km */
        let e = new_enzyme_kinetics(10.0, 3.0);
        assert_eq!(enzyme_half_saturation(&e), 3.0);
    }

    #[test]
    fn test_enzyme_is_saturated() {
        /* saturated when S >> Km */
        let e = new_enzyme_kinetics(10.0, 1.0);
        assert!(enzyme_is_saturated(&e, 20.0));
        assert!(!enzyme_is_saturated(&e, 2.0));
    }
}
