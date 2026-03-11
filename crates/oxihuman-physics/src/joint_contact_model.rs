// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct JointContact {
    pub gap: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub area: f32,
}

pub fn new_joint_contact(stiffness: f32, area: f32) -> JointContact {
    JointContact {
        gap: 0.0,
        stiffness,
        damping: 0.1 * stiffness,
        area,
    }
}

pub fn joint_contact_force(c: &JointContact, penetration: f32, velocity: f32) -> f32 {
    if penetration <= 0.0 {
        return 0.0;
    }
    let spring = c.stiffness * penetration;
    let damp = c.damping * velocity.abs();
    (spring + damp).max(0.0)
}

pub fn joint_contact_pressure(c: &JointContact, penetration: f32) -> f32 {
    if c.area <= 0.0 {
        return 0.0;
    }
    (c.stiffness * penetration.max(0.0)) / c.area
}

pub fn joint_is_in_contact(c: &JointContact, gap: f32) -> bool {
    gap < c.gap
}

pub fn joint_contact_energy(c: &JointContact, penetration: f32) -> f32 {
    0.5 * c.stiffness * penetration.max(0.0).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_penetration_no_force() {
        /* zero penetration produces zero force */
        let c = new_joint_contact(1000.0, 1.0);
        assert_eq!(joint_contact_force(&c, 0.0, 0.0), 0.0);
    }

    #[test]
    fn force_positive_for_penetration() {
        /* positive penetration yields positive force */
        let c = new_joint_contact(1000.0, 1.0);
        assert!(joint_contact_force(&c, 0.01, 0.0) > 0.0);
    }

    #[test]
    fn pressure_scales_with_penetration() {
        /* pressure increases with penetration */
        let c = new_joint_contact(1000.0, 2.0);
        let p1 = joint_contact_pressure(&c, 0.01);
        let p2 = joint_contact_pressure(&c, 0.02);
        assert!(p2 > p1);
    }

    #[test]
    fn in_contact_when_gap_negative() {
        /* gap below threshold means contact */
        let c = new_joint_contact(100.0, 1.0);
        assert!(joint_is_in_contact(&c, -0.001));
    }

    #[test]
    fn energy_zero_at_zero_penetration() {
        /* energy is zero with no penetration */
        let c = new_joint_contact(500.0, 1.0);
        assert_eq!(joint_contact_energy(&c, 0.0), 0.0);
    }

    #[test]
    fn energy_positive_for_penetration() {
        /* positive penetration stores energy */
        let c = new_joint_contact(500.0, 1.0);
        assert!(joint_contact_energy(&c, 0.01) > 0.0);
    }

    #[test]
    fn pressure_zero_for_zero_area() {
        /* zero area returns zero pressure safely */
        let mut c = new_joint_contact(1000.0, 0.0);
        c.area = 0.0;
        assert_eq!(joint_contact_pressure(&c, 0.01), 0.0);
    }
}
