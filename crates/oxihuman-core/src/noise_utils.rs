// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deterministic noise functions: value noise, smooth noise, octave noise.

#![allow(dead_code)]

/// A deterministic integer hash function.
#[allow(dead_code)]
pub fn hash_u32(x: u32) -> u32 {
    let mut h = x.wrapping_add(0x9e37_79b9);
    h = h.wrapping_mul(0x6c62_272e);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    h
}

/// Returns a pseudo-random float in [0, 1) for an integer lattice position.
fn lattice_val(ix: i32) -> f32 {
    (hash_u32(ix as u32) as f32) / (u32::MAX as f32)
}

/// Returns a pseudo-random float in [0, 1) for a 2D lattice position.
fn lattice_val_2d(ix: i32, iy: i32) -> f32 {
    let h = hash_u32(ix as u32 ^ hash_u32(iy as u32));
    (h as f32) / (u32::MAX as f32)
}

/// Quintic smoothstep: 6t^5 - 15t^4 + 10t^3.
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// 1D value noise in [0, 1).
#[allow(dead_code)]
pub fn value_noise_1d(x: f32) -> f32 {
    let ix = x.floor() as i32;
    let fx = x - ix as f32;
    let v0 = lattice_val(ix);
    let v1 = lattice_val(ix + 1);
    v0 + (v1 - v0) * fx
}

/// 2D value noise in [0, 1).
#[allow(dead_code)]
pub fn value_noise_2d(x: f32, y: f32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - ix as f32;
    let fy = y - iy as f32;
    let v00 = lattice_val_2d(ix, iy);
    let v10 = lattice_val_2d(ix + 1, iy);
    let v01 = lattice_val_2d(ix, iy + 1);
    let v11 = lattice_val_2d(ix + 1, iy + 1);
    let vx0 = v00 + (v10 - v00) * fx;
    let vx1 = v01 + (v11 - v01) * fx;
    vx0 + (vx1 - vx0) * fy
}

/// 1D value noise with quintic smoothing.
#[allow(dead_code)]
pub fn smooth_noise_1d(x: f32) -> f32 {
    let ix = x.floor() as i32;
    let fx = x - ix as f32;
    let u = fade(fx);
    let v0 = lattice_val(ix);
    let v1 = lattice_val(ix + 1);
    v0 + (v1 - v0) * u
}

/// Octave noise: sum of `octaves` layers with decreasing amplitude.
#[allow(dead_code)]
pub fn octave_noise(x: f32, octaves: u32, persistence: f32) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;
    let mut freq = 1.0;
    for _ in 0..octaves {
        total += smooth_noise_1d(x * freq) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        freq *= 2.0;
    }
    if max_value > 0.0 {
        total / max_value
    } else {
        0.0
    }
}

/// Remaps a noise value from [0, 1] to [-1, 1].
#[allow(dead_code)]
pub fn noise_range(v: f32) -> f32 {
    v * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        assert_eq!(hash_u32(42), hash_u32(42));
        assert_ne!(hash_u32(0), hash_u32(1));
    }

    #[test]
    fn test_hash_zero() {
        let h = hash_u32(0);
        assert!(h > 0); // Should produce some non-trivial value
    }

    #[test]
    fn test_value_noise_1d_range() {
        for i in 0..10 {
            let v = value_noise_1d(i as f32 * 0.3);
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn test_value_noise_2d_range() {
        for i in 0..5 {
            for j in 0..5 {
                let v = value_noise_2d(i as f32 * 0.7, j as f32 * 0.5);
                assert!((0.0..=1.0).contains(&v));
            }
        }
    }

    #[test]
    fn test_smooth_noise_1d_range() {
        for i in 0..10 {
            let v = smooth_noise_1d(i as f32 * 0.4);
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn test_octave_noise_single() {
        let v = octave_noise(0.5, 1, 0.5);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn test_octave_noise_zero_octaves() {
        let v = octave_noise(1.0, 0, 0.5);
        assert!(v.abs() < 1e-9);
    }

    #[test]
    fn test_noise_range() {
        assert!((noise_range(0.0) - (-1.0)).abs() < 1e-5);
        assert!((noise_range(1.0) - 1.0).abs() < 1e-5);
        assert!((noise_range(0.5)).abs() < 1e-5);
    }

    #[test]
    fn test_octave_noise_multi_range() {
        let v = octave_noise(2.5, 4, 0.5);
        assert!((0.0..=1.0).contains(&v));
    }
}
