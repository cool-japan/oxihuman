// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

pub struct CoronaryArtery {
    pub diameter_mm: f32,
    pub length_mm: f32,
    pub stenosis_fraction: f32,
    pub viscosity: f32,
}

pub fn new_coronary_artery(diameter_mm: f32, length_mm: f32) -> CoronaryArtery {
    CoronaryArtery {
        diameter_mm,
        length_mm,
        stenosis_fraction: 0.0,
        viscosity: 0.003,
    }
}

pub fn coronary_resistance(a: &CoronaryArtery) -> f32 {
    /* Poiseuille: R = 8μL / (π r⁴) with stenosis applied to radius */
    let r = (a.diameter_mm * 0.5 * (1.0 - a.stenosis_fraction)) * 1e-3; /* m */
    let l = a.length_mm * 1e-3; /* m */
    if r <= 0.0 {
        return f32::MAX;
    }
    (8.0 * a.viscosity * l) / (PI * r.powi(4))
}

pub fn coronary_flow_ml_per_min(a: &CoronaryArtery, pressure_pa: f32) -> f32 {
    let r = coronary_resistance(a);
    if r <= 0.0 {
        return 0.0;
    }
    let flow_m3_s = pressure_pa / r;
    flow_m3_s * 1e6 * 60.0 /* m³/s → ml/min */
}

pub fn coronary_ffr(a: &CoronaryArtery) -> f32 {
    (1.0 - a.stenosis_fraction * a.stenosis_fraction).max(0.0)
}

pub fn coronary_is_critical(a: &CoronaryArtery) -> bool {
    a.stenosis_fraction > 0.7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resistance_positive() {
        /* resistance is positive for valid artery */
        let a = new_coronary_artery(3.0, 100.0);
        assert!(coronary_resistance(&a) > 0.0);
    }

    #[test]
    fn stenosis_increases_resistance() {
        /* stenosis raises resistance */
        let a0 = new_coronary_artery(3.0, 100.0);
        let mut a1 = new_coronary_artery(3.0, 100.0);
        a1.stenosis_fraction = 0.5;
        assert!(coronary_resistance(&a1) > coronary_resistance(&a0));
    }

    #[test]
    fn flow_positive_with_pressure() {
        /* positive pressure drives positive flow */
        let a = new_coronary_artery(3.0, 100.0);
        assert!(coronary_flow_ml_per_min(&a, 10000.0) > 0.0);
    }

    #[test]
    fn ffr_one_without_stenosis() {
        /* FFR = 1 with no stenosis */
        let a = new_coronary_artery(3.0, 100.0);
        assert!((coronary_ffr(&a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ffr_less_than_one_with_stenosis() {
        /* FFR < 1 with stenosis */
        let mut a = new_coronary_artery(3.0, 100.0);
        a.stenosis_fraction = 0.5;
        assert!(coronary_ffr(&a) < 1.0);
    }

    #[test]
    fn not_critical_without_stenosis() {
        /* no stenosis → not critical */
        let a = new_coronary_artery(3.0, 100.0);
        assert!(!coronary_is_critical(&a));
    }

    #[test]
    fn critical_with_high_stenosis() {
        /* stenosis > 0.7 → critical */
        let mut a = new_coronary_artery(3.0, 100.0);
        a.stenosis_fraction = 0.8;
        assert!(coronary_is_critical(&a));
    }
}
