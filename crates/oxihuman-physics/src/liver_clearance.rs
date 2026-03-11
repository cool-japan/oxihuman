// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct LiverClearance {
    pub extraction_ratio: f32,
    pub blood_flow_ml_per_min: f32,
    pub plasma_protein_binding: f32,
}

pub fn new_liver_clearance(extraction_ratio: f32) -> LiverClearance {
    LiverClearance {
        extraction_ratio: extraction_ratio.clamp(0.0, 1.0),
        blood_flow_ml_per_min: 1500.0,
        plasma_protein_binding: 0.5,
    }
}

pub fn liver_intrinsic_clearance(l: &LiverClearance) -> f32 {
    let e = l.extraction_ratio;
    let denom = (1.0 - e).max(1e-9);
    l.blood_flow_ml_per_min * e / denom
}

pub fn liver_bioavailability(l: &LiverClearance) -> f32 {
    1.0 - l.extraction_ratio
}

pub fn liver_clearance_rate(l: &LiverClearance) -> f32 {
    l.blood_flow_ml_per_min * l.extraction_ratio
}

pub fn liver_half_life(l: &LiverClearance, volume_l: f32) -> f32 {
    let cl = liver_clearance_rate(l);
    if cl <= 0.0 {
        return f32::MAX;
    }
    0.693 * volume_l * 1000.0 / cl /* volume in mL */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bioavailability_complement() {
        /* bioavailability = 1 - extraction */
        let l = new_liver_clearance(0.6);
        assert!((liver_bioavailability(&l) - 0.4).abs() < 1e-5);
    }

    #[test]
    fn clearance_rate_positive() {
        /* clearance rate > 0 for positive extraction */
        let l = new_liver_clearance(0.5);
        assert!(liver_clearance_rate(&l) > 0.0);
    }

    #[test]
    fn intrinsic_clearance_positive() {
        /* intrinsic clearance positive */
        let l = new_liver_clearance(0.5);
        assert!(liver_intrinsic_clearance(&l) > 0.0);
    }

    #[test]
    fn half_life_positive() {
        /* half-life positive for finite clearance */
        let l = new_liver_clearance(0.5);
        assert!(liver_half_life(&l, 50.0) > 0.0);
    }

    #[test]
    fn zero_extraction_full_bioavailability() {
        /* zero extraction → full bioavailability */
        let l = new_liver_clearance(0.0);
        assert!((liver_bioavailability(&l) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn high_extraction_low_bioavailability() {
        /* high extraction → low bioavailability */
        let l = new_liver_clearance(0.9);
        assert!(liver_bioavailability(&l) < 0.2);
    }

    #[test]
    fn clearance_zero_for_zero_extraction() {
        /* zero extraction → zero clearance */
        let l = new_liver_clearance(0.0);
        assert_eq!(liver_clearance_rate(&l), 0.0);
    }
}
