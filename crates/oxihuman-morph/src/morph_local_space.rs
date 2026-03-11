// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Local space morph: apply morph in local bone space.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LocalSpaceMorph {
    pub origin: [f32; 3],
    pub scale: f32,
    pub rotation_deg: f32,
}

#[allow(dead_code)]
pub fn new_local_space_morph(origin: [f32; 3], scale: f32) -> LocalSpaceMorph {
    LocalSpaceMorph { origin, scale, rotation_deg: 0.0 }
}

#[allow(dead_code)]
pub fn lsm_to_local(m: &LocalSpaceMorph, world_pos: [f32; 3]) -> [f32; 3] {
    let s = if m.scale != 0.0 { m.scale } else { 1.0 };
    [
        (world_pos[0] - m.origin[0]) / s,
        (world_pos[1] - m.origin[1]) / s,
        (world_pos[2] - m.origin[2]) / s,
    ]
}

#[allow(dead_code)]
pub fn lsm_to_world(m: &LocalSpaceMorph, local_pos: [f32; 3]) -> [f32; 3] {
    [
        local_pos[0] * m.scale + m.origin[0],
        local_pos[1] * m.scale + m.origin[1],
        local_pos[2] * m.scale + m.origin[2],
    ]
}

#[allow(dead_code)]
pub fn lsm_set_rotation(m: &mut LocalSpaceMorph, deg: f32) {
    m.rotation_deg = deg;
}

#[allow(dead_code)]
pub fn lsm_scale(m: &LocalSpaceMorph) -> f32 {
    m.scale
}

#[allow(dead_code)]
pub fn lsm_origin(m: &LocalSpaceMorph) -> [f32; 3] {
    m.origin
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_to_local_to_world() {
        let m = new_local_space_morph([1.0, 2.0, 3.0], 2.0);
        let world = [5.0, 6.0, 7.0];
        let local = lsm_to_local(&m, world);
        let back = lsm_to_world(&m, local);
        assert!((back[0] - world[0]).abs() < 1e-5);
        assert!((back[1] - world[1]).abs() < 1e-5);
        assert!((back[2] - world[2]).abs() < 1e-5);
    }

    #[test]
    fn test_to_local_at_origin_is_zero() {
        let m = new_local_space_morph([1.0, 2.0, 3.0], 1.0);
        let local = lsm_to_local(&m, [1.0, 2.0, 3.0]);
        for v in &local {
            assert!(v.abs() < 1e-6);
        }
    }

    #[test]
    fn test_scale_getter() {
        let m = new_local_space_morph([0.0, 0.0, 0.0], 3.0);
        assert!((lsm_scale(&m) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_origin_getter() {
        let m = new_local_space_morph([1.0, 2.0, 3.0], 1.0);
        let o = lsm_origin(&m);
        assert!((o[0] - 1.0).abs() < 1e-6);
        assert!((o[1] - 2.0).abs() < 1e-6);
        assert!((o[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_rotation_set() {
        let mut m = new_local_space_morph([0.0; 3], 1.0);
        lsm_set_rotation(&mut m, 45.0);
        assert!((m.rotation_deg - 45.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_world_applies_scale() {
        let m = new_local_space_morph([0.0; 3], 2.0);
        let w = lsm_to_world(&m, [1.0, 0.0, 0.0]);
        assert!((w[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_local_applies_scale() {
        let m = new_local_space_morph([0.0; 3], 2.0);
        let l = lsm_to_local(&m, [2.0, 0.0, 0.0]);
        assert!((l[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_initial_rotation_zero() {
        let m = new_local_space_morph([0.0; 3], 1.0);
        assert_eq!(m.rotation_deg, 0.0);
    }
}
