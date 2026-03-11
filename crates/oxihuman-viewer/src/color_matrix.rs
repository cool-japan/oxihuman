// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 4×4 colour matrix post-process effect.

use std::f32::consts::FRAC_PI_4;

/// Row-major 4×4 colour transformation matrix.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorMatrix {
    pub m: [[f32; 4]; 4],
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn cm_identity() -> ColorMatrix {
    ColorMatrix {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn cm_apply(mat: &ColorMatrix, rgba: [f32; 4]) -> [f32; 4] {
    if !mat.enabled {
        return rgba;
    }
    let mut out = [0.0f32; 4];
    #[allow(clippy::needless_range_loop)]
    for row in 0..4 {
        for col in 0..4 {
            out[row] += mat.m[row][col] * rgba[col];
        }
    }
    out
}

#[allow(dead_code)]
pub fn cm_set_enabled(mat: &mut ColorMatrix, v: bool) {
    mat.enabled = v;
}

#[allow(dead_code)]
pub fn cm_is_identity(mat: &ColorMatrix) -> bool {
    for (r, row) in mat.m.iter().enumerate() {
        for (c, &val) in row.iter().enumerate() {
            let expected = if r == c { 1.0f32 } else { 0.0 };
            if (val - expected).abs() > 1e-6 {
                return false;
            }
        }
    }
    true
}

#[allow(dead_code)]
pub fn cm_saturation(s: f32) -> ColorMatrix {
    let r = 0.2126_f32;
    let g = 0.7152_f32;
    let b = 0.0722_f32;
    let sr = (1.0 - s) * r + s;
    let sg = (1.0 - s) * g + s;
    let sb = (1.0 - s) * b + s;
    ColorMatrix {
        m: [
            [sr, (1.0 - s) * g, (1.0 - s) * b, 0.0],
            [(1.0 - s) * r, sg, (1.0 - s) * b, 0.0],
            [(1.0 - s) * r, (1.0 - s) * g, sb, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn cm_determinant(mat: &ColorMatrix) -> f32 {
    // Use only 3×3 sub-matrix for colour channels
    let m = &mat.m;
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

#[allow(dead_code)]
pub fn cm_blend(a: &ColorMatrix, b: &ColorMatrix, t: f32) -> ColorMatrix {
    let t = t.clamp(0.0, 1.0);
    let mut out = cm_identity();
    out.enabled = a.enabled || b.enabled;
    #[allow(clippy::needless_range_loop)]
    for r in 0..4 {
        for c in 0..4 {
            out.m[r][c] = a.m[r][c] * (1.0 - t) + b.m[r][c] * t;
        }
    }
    out
}

#[allow(dead_code)]
pub fn cm_hue_rotate_angle_rad(mat: &ColorMatrix) -> f32 {
    // Proxy: angle from determinant
    cm_determinant(mat).abs().atan().min(FRAC_PI_4)
}

#[allow(dead_code)]
pub fn cm_to_json(mat: &ColorMatrix) -> String {
    format!(
        "{{\"enabled\":{},\"det\":{:.4}}}",
        mat.enabled,
        cm_determinant(mat)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_is_identity() {
        assert!(cm_is_identity(&cm_identity()));
    }
    #[test]
    fn apply_identity_unchanged() {
        let m = cm_identity();
        let c = [1.0, 0.5, 0.25, 1.0];
        let o = cm_apply(&m, c);
        assert!((o[0] - c[0]).abs() < 1e-5);
    }
    #[test]
    fn disabled_matrix_passthrough() {
        let mut m = cm_identity();
        m.m[0][0] = 0.0;
        cm_set_enabled(&mut m, false);
        let c = [1.0, 0.5, 0.25, 1.0];
        let o = cm_apply(&m, c);
        assert!((o[0] - c[0]).abs() < 1e-5);
    }
    #[test]
    fn saturation_zero_greyscale() {
        let m = cm_saturation(0.0);
        let c = cm_apply(&m, [1.0, 0.0, 0.0, 1.0]); /* red -> grey */
        assert!(c[0] > 0.0 && c[0] < 1.0);
    }
    #[test]
    fn saturation_one_unchanged() {
        let m = cm_saturation(1.0);
        let c = [0.8, 0.4, 0.2, 1.0];
        let o = cm_apply(&m, c);
        assert!((o[0] - c[0]).abs() < 1e-4);
    }
    #[test]
    fn identity_det_is_one() {
        assert!((cm_determinant(&cm_identity()) - 1.0).abs() < 1e-4);
    }
    #[test]
    fn blend_identity_with_self() {
        let a = cm_identity();
        let b = cm_identity();
        let r = cm_blend(&a, &b, 0.5);
        assert!(cm_is_identity(&r));
    }
    #[test]
    fn hue_rotate_angle_nonneg() {
        assert!(cm_hue_rotate_angle_rad(&cm_identity()) >= 0.0);
    }
    #[test]
    fn to_json_has_enabled() {
        assert!(cm_to_json(&cm_identity()).contains("\"enabled\""));
    }
    #[test]
    fn set_enabled_false() {
        let mut m = cm_identity();
        cm_set_enabled(&mut m, false);
        assert!(!m.enabled);
    }
}
