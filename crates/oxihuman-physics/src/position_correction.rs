#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Position correction for penetration resolution.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PenetrationCorrection {
    pub depth: f32,
    pub normal: [f32; 3],
    pub inv_mass_a: f32,
    pub inv_mass_b: f32,
    pub slop: f32,
    pub factor: f32,
}

#[allow(dead_code)]
pub fn default_correction(depth: f32, normal: [f32; 3]) -> PenetrationCorrection {
    PenetrationCorrection {
        depth,
        normal,
        inv_mass_a: 1.0,
        inv_mass_b: 1.0,
        slop: 0.01,
        factor: 0.2,
    }
}

#[allow(dead_code)]
pub fn correction_impulse(pc: &PenetrationCorrection) -> [f32; 3] {
    let excess = (pc.depth - pc.slop).max(0.0);
    let total_inv_mass = pc.inv_mass_a + pc.inv_mass_b;
    if total_inv_mass < 1e-12 {
        return [0.0; 3];
    }
    let magnitude = pc.factor * excess / total_inv_mass;
    [
        pc.normal[0] * magnitude,
        pc.normal[1] * magnitude,
        pc.normal[2] * magnitude,
    ]
}

#[allow(dead_code)]
pub fn apply_correction_a(pos: [f32; 3], pc: &PenetrationCorrection) -> [f32; 3] {
    let impulse = correction_impulse(pc);
    [
        pos[0] + impulse[0] * pc.inv_mass_a,
        pos[1] + impulse[1] * pc.inv_mass_a,
        pos[2] + impulse[2] * pc.inv_mass_a,
    ]
}

#[allow(dead_code)]
pub fn apply_correction_b(pos: [f32; 3], pc: &PenetrationCorrection) -> [f32; 3] {
    let impulse = correction_impulse(pc);
    [
        pos[0] - impulse[0] * pc.inv_mass_b,
        pos[1] - impulse[1] * pc.inv_mass_b,
        pos[2] - impulse[2] * pc.inv_mass_b,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_correction_fields() {
        let pc = default_correction(0.1, [0.0, 1.0, 0.0]);
        assert_eq!(pc.depth, 0.1);
        assert_eq!(pc.normal, [0.0, 1.0, 0.0]);
        assert!((pc.factor - 0.2).abs() < 1e-6);
    }

    #[test]
    fn no_penetration_zero_impulse() {
        let pc = default_correction(0.0, [0.0, 1.0, 0.0]);
        let imp = correction_impulse(&pc);
        assert_eq!(imp, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn impulse_within_slop_is_zero() {
        let pc = default_correction(0.005, [0.0, 1.0, 0.0]); // slop = 0.01
        let imp = correction_impulse(&pc);
        assert_eq!(imp[1], 0.0);
    }

    #[test]
    fn impulse_above_slop_nonzero() {
        let pc = default_correction(0.1, [0.0, 1.0, 0.0]);
        let imp = correction_impulse(&pc);
        assert!(imp[1] > 0.0);
    }

    #[test]
    fn apply_correction_a_moves_in_normal_direction() {
        let pc = default_correction(0.1, [0.0, 1.0, 0.0]);
        let pos = [0.0f32, 0.0, 0.0];
        let corrected = apply_correction_a(pos, &pc);
        assert!(corrected[1] > pos[1]);
    }

    #[test]
    fn apply_correction_b_moves_opposite() {
        let pc = default_correction(0.1, [0.0, 1.0, 0.0]);
        let pos = [0.0f32, 0.0, 0.0];
        let corrected = apply_correction_b(pos, &pc);
        assert!(corrected[1] < pos[1]);
    }

    #[test]
    fn infinite_mass_body_no_correction() {
        let mut pc = default_correction(0.2, [1.0, 0.0, 0.0]);
        pc.inv_mass_a = 0.0;
        let pos = [0.0f32, 0.0, 0.0];
        let corrected = apply_correction_a(pos, &pc);
        // inv_mass_a=0 means no displacement for a
        assert_eq!(corrected, pos);
    }

    #[test]
    fn both_zero_inv_mass_no_panic() {
        let mut pc = default_correction(0.5, [0.0, 1.0, 0.0]);
        pc.inv_mass_a = 0.0;
        pc.inv_mass_b = 0.0;
        let imp = correction_impulse(&pc);
        assert_eq!(imp, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn correction_scales_with_depth() {
        let pc1 = default_correction(0.05, [0.0, 1.0, 0.0]);
        let pc2 = default_correction(0.2, [0.0, 1.0, 0.0]);
        let i1 = correction_impulse(&pc1);
        let i2 = correction_impulse(&pc2);
        assert!(i2[1] > i1[1]);
    }

    #[test]
    fn normal_direction_respected() {
        let pc = default_correction(0.1, [1.0, 0.0, 0.0]);
        let imp = correction_impulse(&pc);
        assert!(imp[0] > 0.0);
        assert_eq!(imp[1], 0.0);
        assert_eq!(imp[2], 0.0);
    }
}
