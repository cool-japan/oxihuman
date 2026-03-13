#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Color space definitions and XYZ/Lab conversion utilities.

/// sRGB to XYZ (D65) conversion.
#[allow(dead_code)]
pub fn rgb_to_xyz(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let rl = srgb_lin(r);
    let gl = srgb_lin(g);
    let bl = srgb_lin(b);
    let x = rl * 0.412_456_4 + gl * 0.357_576_1 + bl * 0.180_437_5;
    let y = rl * 0.212_672_9 + gl * 0.715_152_2 + bl * 0.072_175_0;
    let z = rl * 0.019_333_9 + gl * 0.119_192 + bl * 0.950_304_1;
    (x, y, z)
}

fn srgb_lin(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn lin_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// XYZ (D65) to sRGB conversion.
#[allow(dead_code)]
pub fn xyz_to_rgb(x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    let rl = x * 3.240_454_2 - y * 1.537_138_5 - z * 0.498_531_4;
    let gl = -x * 0.969_266 + y * 1.876_010_8 + z * 0.041_556_0;
    let bl = x * 0.055_643_4 - y * 0.204_025_9 + z * 1.057_225_2;
    (lin_srgb(rl.clamp(0.0, 1.0)), lin_srgb(gl.clamp(0.0, 1.0)), lin_srgb(bl.clamp(0.0, 1.0)))
}

fn lab_f(t: f32) -> f32 {
    let delta = 6.0 / 29.0;
    if t > delta * delta * delta {
        t.cbrt()
    } else {
        t / (3.0 * delta * delta) + 4.0 / 29.0
    }
}

fn lab_f_inv(t: f32) -> f32 {
    let delta = 6.0 / 29.0;
    if t > delta {
        t * t * t
    } else {
        3.0 * delta * delta * (t - 4.0 / 29.0)
    }
}

// D65 reference white
const XN: f32 = 0.95047;
const YN: f32 = 1.00000;
const ZN: f32 = 1.08883;

/// XYZ to CIE L*a*b* conversion.
#[allow(dead_code)]
pub fn xyz_to_lab(x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    let fx = lab_f(x / XN);
    let fy = lab_f(y / YN);
    let fz = lab_f(z / ZN);
    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    (l, a, b)
}

/// CIE L*a*b* to XYZ conversion.
#[allow(dead_code)]
pub fn lab_to_xyz(l: f32, a: f32, b_: f32) -> (f32, f32, f32) {
    let fy = (l + 16.0) / 116.0;
    let fx = a / 500.0 + fy;
    let fz = fy - b_ / 200.0;
    (lab_f_inv(fx) * XN, lab_f_inv(fy) * YN, lab_f_inv(fz) * ZN)
}

/// CIE76 delta-E color difference.
#[allow(dead_code)]
pub fn delta_e_cie76(l1: f32, a1: f32, b1_: f32, l2: f32, a2: f32, b2: f32) -> f32 {
    let dl = l1 - l2;
    let da = a1 - a2;
    let db = b1_ - b2;
    (dl * dl + da * da + db * db).sqrt()
}

/// Clamp an RGB triplet to [0, 1].
#[allow(dead_code)]
pub fn clamp_color(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_color_normal() {
        let (r, g, b) = clamp_color(0.5, 0.5, 0.5);
        assert!((r - 0.5).abs() < 1e-6);
        assert!((g - 0.5).abs() < 1e-6);
        assert!((b - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_color_overflow() {
        let (r, g, b) = clamp_color(2.0, -1.0, 0.5);
        assert!((r - 1.0).abs() < 1e-6);
        assert!(g.abs() < 1e-6);
        assert!((b - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_rgb_to_xyz_white() {
        let (x, y, z) = rgb_to_xyz(1.0, 1.0, 1.0);
        assert!((x - 0.95047).abs() < 0.001);
        assert!((y - 1.0).abs() < 0.001);
        assert!((z - 1.08883).abs() < 0.001);
    }

    #[test]
    fn test_rgb_to_xyz_black() {
        let (x, y, z) = rgb_to_xyz(0.0, 0.0, 0.0);
        assert!(x.abs() < 1e-6);
        assert!(y.abs() < 1e-6);
        assert!(z.abs() < 1e-6);
    }

    #[test]
    fn test_xyz_to_lab_white() {
        let (x, y, z) = rgb_to_xyz(1.0, 1.0, 1.0);
        let (l, a, b) = xyz_to_lab(x, y, z);
        assert!((l - 100.0).abs() < 0.1);
        assert!(a.abs() < 0.1);
        assert!(b.abs() < 0.1);
    }

    #[test]
    fn test_delta_e_same() {
        assert!(delta_e_cie76(50.0, 20.0, 10.0, 50.0, 20.0, 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_delta_e_nonzero() {
        let d = delta_e_cie76(50.0, 0.0, 0.0, 60.0, 0.0, 0.0);
        assert!((d - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_lab_roundtrip() {
        let (x0, y0, z0) = rgb_to_xyz(0.5, 0.3, 0.7);
        let (l, a, b) = xyz_to_lab(x0, y0, z0);
        let (x1, y1, z1) = lab_to_xyz(l, a, b);
        assert!((x0 - x1).abs() < 0.001);
        assert!((y0 - y1).abs() < 0.001);
        assert!((z0 - z1).abs() < 0.001);
    }

    #[test]
    fn test_xyz_rgb_roundtrip() {
        let (x, y, z) = rgb_to_xyz(0.6, 0.4, 0.2);
        let (r2, g2, b2) = xyz_to_rgb(x, y, z);
        assert!((r2 - 0.6).abs() < 0.01);
        assert!((g2 - 0.4).abs() < 0.01);
        assert!((b2 - 0.2).abs() < 0.01);
    }
}
