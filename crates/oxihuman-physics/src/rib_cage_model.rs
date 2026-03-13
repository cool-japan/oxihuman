// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct RibCage {
    pub rest_volume_l: f32,
    pub current_volume_l: f32,
    pub compliance: f32,
    pub resistance: f32,
}

pub fn new_rib_cage() -> RibCage {
    RibCage {
        rest_volume_l: 3.0,
        current_volume_l: 3.0,
        compliance: 0.1,
        resistance: 1.0,
    }
}

pub fn rib_expansion(r: &RibCage) -> f32 {
    r.current_volume_l - r.rest_volume_l
}

pub fn rib_elastic_recoil_pressure(r: &RibCage) -> f32 {
    rib_expansion(r) / r.compliance.max(1e-9)
}

pub fn rib_step(r: &mut RibCage, muscle_force: f32, dt: f32) {
    let delta = muscle_force * dt / r.resistance.max(1e-9);
    r.current_volume_l += delta;
    r.current_volume_l = r.current_volume_l.max(0.5);
}

pub fn rib_is_expanded(r: &RibCage) -> bool {
    r.current_volume_l > r.rest_volume_l
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_volume() {
        /* rest volume is 3 L */
        let r = new_rib_cage();
        assert!((r.rest_volume_l - 3.0).abs() < 1e-6);
    }

    #[test]
    fn expansion_zero_at_rest() {
        /* no expansion at rest volume */
        let r = new_rib_cage();
        assert_eq!(rib_expansion(&r), 0.0);
    }

    #[test]
    fn elastic_recoil_zero_at_rest() {
        /* recoil pressure zero at rest */
        let r = new_rib_cage();
        assert_eq!(rib_elastic_recoil_pressure(&r), 0.0);
    }

    #[test]
    fn step_expands_volume() {
        /* positive muscle force expands rib cage */
        let mut r = new_rib_cage();
        rib_step(&mut r, 10.0, 0.1);
        assert!(r.current_volume_l > r.rest_volume_l);
    }

    #[test]
    fn is_expanded_after_step() {
        /* rib_is_expanded true after expansion */
        let mut r = new_rib_cage();
        rib_step(&mut r, 10.0, 0.1);
        assert!(rib_is_expanded(&r));
    }

    #[test]
    fn not_expanded_at_rest() {
        /* not expanded at rest volume */
        let r = new_rib_cage();
        assert!(!rib_is_expanded(&r));
    }

    #[test]
    fn volume_stays_positive() {
        /* volume clamped above minimum */
        let mut r = new_rib_cage();
        rib_step(&mut r, -1000.0, 10.0);
        assert!(r.current_volume_l > 0.0);
    }
}
