// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[allow(dead_code)]
pub fn smoothstep(a: f64, b: f64, t: f64) -> f64 {
    let t = ((t - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[allow(dead_code)]
pub fn cubic_hermite(p0: f64, p1: f64, m0: f64, m1: f64, t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p0
        + (t3 - 2.0 * t2 + t) * m0
        + (-2.0 * t3 + 3.0 * t2) * p1
        + (t3 - t2) * m1
}

#[allow(dead_code)]
pub fn catmull_rom(p0: f64, p1: f64, p2: f64, p3: f64, t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

#[allow(dead_code)]
pub fn bilinear(tl: f64, tr: f64, bl: f64, br: f64, tx: f64, ty: f64) -> f64 {
    let top = lerp(tl, tr, tx);
    let bottom = lerp(bl, br, tx);
    lerp(top, bottom, ty)
}

#[allow(dead_code)]
pub fn inverse_lerp(a: f64, b: f64, v: f64) -> f64 {
    if (b - a).abs() < 1e-15 {
        return 0.0;
    }
    (v - a) / (b - a)
}

#[allow(dead_code)]
pub fn remap(v: f64, a0: f64, a1: f64, b0: f64, b1: f64) -> f64 {
    let t = inverse_lerp(a0, a1, v);
    lerp(b0, b1, t)
}

// --- f32 versions required by spec ---

/// Linear interpolation (f32 version).
#[allow(dead_code)]
pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Smoothstep (f32 version).
#[allow(dead_code)]
pub fn smoothstep_f32(a: f32, b: f32, t: f32) -> f32 {
    let t = ((t - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smootherstep (Ken Perlin's version, f32).
#[allow(dead_code)]
pub fn smootherstep(a: f32, b: f32, t: f32) -> f32 {
    let t = ((t - a) / (b - a)).clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Cubic Hermite interpolation (f32 version).
#[allow(dead_code)]
pub fn cubic_interp(p0: f32, p1: f32, m0: f32, m1: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p0
        + (t3 - 2.0 * t2 + t) * m0
        + (-2.0 * t3 + 3.0 * t2) * p1
        + (t3 - t2) * m1
}

/// Bilinear interpolation (f32 version).
#[allow(dead_code)]
pub fn bilinear_interp(tl: f32, tr: f32, bl: f32, br: f32, tx: f32, ty: f32) -> f32 {
    let top = lerp_f32(tl, tr, tx);
    let bottom = lerp_f32(bl, br, tx);
    lerp_f32(top, bottom, ty)
}

/// Hermite interpolation between p0 and p1 using tangents m0 and m1 (f32).
#[allow(dead_code)]
pub fn hermite_interp(p0: f32, p1: f32, m0: f32, m1: f32, t: f32) -> f32 {
    cubic_interp(p0, p1, m0, m1, t)
}

/// Inverse linear interpolation (f32 version).
#[allow(dead_code)]
pub fn inverse_lerp_f32(a: f32, b: f32, v: f32) -> f32 {
    if (b - a).abs() < 1e-9 {
        return 0.0;
    }
    (v - a) / (b - a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp_at_0() {
        assert!((lerp(1.0, 5.0, 0.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_lerp_at_1() {
        assert!((lerp(1.0, 5.0, 1.0) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_lerp_at_half() {
        assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_smoothstep_edges() {
        assert!((smoothstep(0.0, 1.0, 0.0)).abs() < 1e-10);
        assert!((smoothstep(0.0, 1.0, 1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_bilinear_corners() {
        assert!((bilinear(1.0, 0.0, 0.0, 0.0, 0.0, 0.0) - 1.0).abs() < 1e-10);
        assert!((bilinear(0.0, 1.0, 0.0, 0.0, 1.0, 0.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_lerp() {
        assert!((inverse_lerp(0.0, 10.0, 5.0) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_remap() {
        let v = remap(5.0, 0.0, 10.0, 0.0, 100.0);
        assert!((v - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_cubic_hermite_endpoints() {
        assert!((cubic_hermite(0.0, 1.0, 0.0, 0.0, 0.0) - 0.0).abs() < 1e-10);
        assert!((cubic_hermite(0.0, 1.0, 0.0, 0.0, 1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_catmull_rom_at_t0() {
        let v = catmull_rom(0.0, 1.0, 2.0, 3.0, 0.0);
        assert!((v - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_catmull_rom_at_t1() {
        let v = catmull_rom(0.0, 1.0, 2.0, 3.0, 1.0);
        assert!((v - 2.0).abs() < 1e-10);
    }
}
