// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct SandView {
    pub show_stress: bool,
    pub show_flow: bool,
    pub repose_angle_deg: f32,
}

pub fn new_sand_view() -> SandView {
    SandView {
        show_stress: true,
        show_flow: false,
        repose_angle_deg: 35.0,
    }
}

pub fn sand_stress_color(stress: f32, max_stress: f32) -> [f32; 3] {
    let t = (stress / max_stress.max(1e-6)).clamp(0.0, 1.0);
    [t, 1.0 - t, 0.0]
}

pub fn sand_is_flowing(slope_deg: f32, repose_deg: f32) -> bool {
    slope_deg > repose_deg
}

pub fn sand_flow_color(velocity: f32) -> [f32; 3] {
    let v = velocity.clamp(0.0, 1.0);
    [0.8 * v, 0.5 * v, 0.2 * v]
}

pub fn sand_packing_fraction(porosity: f32) -> f32 {
    1.0 - porosity.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sand_view() {
        /* repose angle defaults to 35 degrees */
        let v = new_sand_view();
        assert!((v.repose_angle_deg - 35.0).abs() < 1e-6);
    }

    #[test]
    fn test_sand_stress_color_zero() {
        /* zero stress -> green */
        let c = sand_stress_color(0.0, 1.0);
        assert_eq!(c, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_sand_is_flowing_true() {
        /* slope > repose means flowing */
        assert!(sand_is_flowing(40.0, 35.0));
    }

    #[test]
    fn test_sand_is_flowing_false() {
        /* slope <= repose means stable */
        assert!(!sand_is_flowing(30.0, 35.0));
    }

    #[test]
    fn test_sand_packing_fraction() {
        /* packing fraction = 1 - porosity */
        assert!((sand_packing_fraction(0.4) - 0.6).abs() < 1e-6);
    }
}
