// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Curve modifier deformation export for mesh-along-curve operations.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurveModAxis {
    X,
    Y,
    Z,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurveModEntry {
    pub control_points: Vec<[f32; 3]>,
    pub axis: CurveModAxis,
    pub stretch: bool,
    pub bounds_clamp: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurveModifierExport {
    pub modifiers: Vec<CurveModEntry>,
}

#[allow(dead_code)]
pub fn new_curve_modifier_export() -> CurveModifierExport {
    CurveModifierExport {
        modifiers: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_curve_mod(exp: &mut CurveModifierExport, points: Vec<[f32; 3]>, axis: CurveModAxis) {
    exp.modifiers.push(CurveModEntry {
        control_points: points,
        axis,
        stretch: false,
        bounds_clamp: true,
    });
}

#[allow(dead_code)]
pub fn mod_count(exp: &CurveModifierExport) -> usize {
    exp.modifiers.len()
}

#[allow(dead_code)]
pub fn curve_mod_length(entry: &CurveModEntry) -> f32 {
    let mut len = 0.0_f32;
    for pair in entry.control_points.windows(2) {
        let d = [
            pair[1][0] - pair[0][0],
            pair[1][1] - pair[0][1],
            pair[1][2] - pair[0][2],
        ];
        len += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    }
    len
}

#[allow(dead_code)]
pub fn axis_name(axis: CurveModAxis) -> &'static str {
    match axis {
        CurveModAxis::X => "X",
        CurveModAxis::Y => "Y",
        CurveModAxis::Z => "Z",
    }
}

#[allow(dead_code)]
pub fn validate_curve_mod(entry: &CurveModEntry) -> bool {
    entry.control_points.len() >= 2
}

#[allow(dead_code)]
pub fn curve_modifier_to_json(exp: &CurveModifierExport) -> String {
    format!("{{\"modifier_count\":{}}}", exp.modifiers.len())
}

#[allow(dead_code)]
pub fn total_curve_length(exp: &CurveModifierExport) -> f32 {
    exp.modifiers.iter().map(curve_mod_length).sum()
}

#[allow(dead_code)]
pub fn clear_curve_mods(exp: &mut CurveModifierExport) {
    exp.modifiers.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_points() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]
    }

    #[test]
    fn test_new_empty() {
        let exp = new_curve_modifier_export();
        assert_eq!(mod_count(&exp), 0);
    }

    #[test]
    fn test_add_mod() {
        let mut exp = new_curve_modifier_export();
        add_curve_mod(&mut exp, sample_points(), CurveModAxis::Y);
        assert_eq!(mod_count(&exp), 1);
    }

    #[test]
    fn test_curve_length() {
        let entry = CurveModEntry {
            control_points: sample_points(),
            axis: CurveModAxis::Y,
            stretch: false,
            bounds_clamp: true,
        };
        assert!((curve_mod_length(&entry) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_axis_name_y() {
        assert_eq!(axis_name(CurveModAxis::Y), "Y");
    }

    #[test]
    fn test_validate_valid() {
        let entry = CurveModEntry {
            control_points: sample_points(),
            axis: CurveModAxis::X,
            stretch: false,
            bounds_clamp: true,
        };
        assert!(validate_curve_mod(&entry));
    }

    #[test]
    fn test_validate_single_point() {
        let entry = CurveModEntry {
            control_points: vec![[0.0; 3]],
            axis: CurveModAxis::X,
            stretch: false,
            bounds_clamp: true,
        };
        assert!(!validate_curve_mod(&entry));
    }

    #[test]
    fn test_json_output() {
        let exp = new_curve_modifier_export();
        let j = curve_modifier_to_json(&exp);
        assert!(j.contains("modifier_count"));
    }

    #[test]
    fn test_total_length() {
        let mut exp = new_curve_modifier_export();
        add_curve_mod(&mut exp, sample_points(), CurveModAxis::Z);
        assert!(total_curve_length(&exp) > 0.0);
    }

    #[test]
    fn test_clear_mods() {
        let mut exp = new_curve_modifier_export();
        add_curve_mod(&mut exp, sample_points(), CurveModAxis::X);
        clear_curve_mods(&mut exp);
        assert_eq!(mod_count(&exp), 0);
    }

    #[test]
    fn test_axis_x_name() {
        assert_eq!(axis_name(CurveModAxis::X), "X");
    }
}
