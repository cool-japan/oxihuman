// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Hash-based white noise in [0, 1] using sine trick.
pub fn white_noise(x: f32, y: f32) -> f32 {
    let v = x * 127.1 + y * 311.7;
    (v.sin() * 43_758.55_f32).fract().abs()
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Bilinear-interpolated value noise in [0, 1].
pub fn value_noise_2d(x: f32, y: f32) -> f32 {
    let ix = x.floor();
    let iy = y.floor();
    let fx = x - ix;
    let fy = y - iy;
    let ux = smooth(fx);
    let uy = smooth(fy);
    let a = white_noise(ix, iy);
    let b = white_noise(ix + 1.0, iy);
    let c = white_noise(ix, iy + 1.0);
    let d = white_noise(ix + 1.0, iy + 1.0);
    let ab = a + ux * (b - a);
    let cd = c + ux * (d - c);
    ab + uy * (cd - ab)
}

/// Fractal Brownian Motion (fBm) noise.
pub fn fractal_noise_2d(x: f32, y: f32, octaves: u32) -> f32 {
    let mut value = 0.0f32;
    let mut amplitude = 0.5f32;
    let mut freq = 1.0f32;
    let mut max_val = 0.0f32;
    for _ in 0..octaves {
        value += value_noise_2d(x * freq, y * freq) * amplitude;
        max_val += amplitude;
        amplitude *= 0.5;
        freq *= 2.0;
    }
    if max_val > 0.0 {
        value / max_val
    } else {
        0.0
    }
}

/// Turbulence noise (absolute value fBm), maps to [0, 1].
pub fn turbulence_2d(x: f32, y: f32, octaves: u32) -> f32 {
    let mut value = 0.0f32;
    let mut amplitude = 0.5f32;
    let mut freq = 1.0f32;
    let mut max_val = 0.0f32;
    for _ in 0..octaves {
        let n = value_noise_2d(x * freq, y * freq) * 2.0 - 1.0;
        value += n.abs() * amplitude;
        max_val += amplitude;
        amplitude *= 0.5;
        freq *= 2.0;
    }
    if max_val > 0.0 {
        value / max_val
    } else {
        0.0
    }
}

/// Ridged multifractal noise.
pub fn ridged_noise_2d(x: f32, y: f32, octaves: u32) -> f32 {
    let mut value = 0.0f32;
    let mut amplitude = 0.5f32;
    let mut freq = 1.0f32;
    let mut max_val = 0.0f32;
    for _ in 0..octaves {
        let n = value_noise_2d(x * freq, y * freq) * 2.0 - 1.0;
        let ridge = 1.0 - n.abs();
        value += ridge * ridge * amplitude;
        max_val += amplitude;
        amplitude *= 0.5;
        freq *= 2.0;
    }
    if max_val > 0.0 {
        value / max_val
    } else {
        0.0
    }
}

/// Checkerboard pattern: returns 0.0 or 1.0.
pub fn checkerboard(x: f32, y: f32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    if (ix + iy) % 2 == 0 {
        0.0
    } else {
        1.0
    }
}

/// Simple 1D gradient noise.
pub fn gradient_noise_1d(x: f32) -> f32 {
    let ix = x.floor();
    let fx = x - ix;
    let g0 = white_noise(ix, 0.0) * 2.0 - 1.0;
    let g1 = white_noise(ix + 1.0, 0.0) * 2.0 - 1.0;
    let v0 = g0 * fx;
    let v1 = g1 * (fx - 1.0);
    let u = smooth(fx);
    (v0 + u * (v1 - v0)) * 0.5 + 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_white_noise_range() {
        /* white noise should be in [0, 1] */
        for i in 0..20 {
            let v = white_noise(i as f32 * 0.73, i as f32 * 0.31);
            assert!((0.0..=1.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn test_white_noise_deterministic() {
        /* same input -> same output */
        let a = white_noise(1.5, 2.7);
        let b = white_noise(1.5, 2.7);
        assert!((a - b).abs() < 1e-9);
    }

    #[test]
    fn test_value_noise_range() {
        /* value noise in [0, 1] */
        for i in 0..20 {
            let v = value_noise_2d(i as f32 * 0.43, i as f32 * 0.19);
            assert!((0.0..=1.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn test_fractal_noise_range() {
        /* fBm result should be in [0, 1] */
        for i in 0..10 {
            let v = fractal_noise_2d(i as f32 * 0.5, i as f32 * 0.3, 4);
            assert!((0.0..=1.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn test_turbulence_range() {
        /* turbulence in [0, 1] */
        for i in 0..10 {
            let v = turbulence_2d(i as f32 * 0.5, i as f32 * 0.7, 3);
            assert!((0.0..=1.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn test_ridged_noise_range() {
        /* ridged noise in [0, 1] */
        for i in 0..10 {
            let v = ridged_noise_2d(i as f32 * 0.4, i as f32 * 0.6, 3);
            assert!((0.0..=1.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn test_checkerboard() {
        /* checkerboard produces 0 or 1 */
        assert!((checkerboard(0.5, 0.5) - 0.0).abs() < 1e-9);
        assert!((checkerboard(1.5, 0.5) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_gradient_noise_1d_range() {
        /* gradient noise in [0, 1] */
        for i in 0..20 {
            let v = gradient_noise_1d(i as f32 * 0.37);
            assert!((0.0..=1.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn test_fractal_zero_octaves() {
        /* zero octaves returns 0 */
        let v = fractal_noise_2d(1.0, 1.0, 0);
        assert!(v.abs() < 1e-9);
    }
}
