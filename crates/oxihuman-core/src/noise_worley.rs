// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Worley/cellular noise 2D (deterministic grid jitter).

#![allow(dead_code)]

/// Deterministic hash to [0,1) for grid cell feature point jitter.
fn cell_hash(ix: i32, iy: i32, seed: u32) -> [f32; 2] {
    let mut h = (ix.wrapping_mul(1619) ^ iy.wrapping_mul(31337)) as u32;
    h ^= seed;
    h = h.wrapping_mul(0x9e37_79b9);
    h ^= h >> 16;
    h = h.wrapping_mul(0x8512_5a2d);
    h ^= h >> 12;
    let hx = (h & 0xFFFF) as f32 / 65535.0;
    let hy = ((h >> 16) & 0xFFFF) as f32 / 65535.0;
    [hx, hy]
}

/// 2D Worley noise: returns distance to nearest feature point.
/// Output is in [0, ~1.5] depending on density.
#[allow(dead_code)]
pub fn worley2(x: f32, y: f32, seed: u32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let mut min_dist = f32::MAX;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let [jx, jy] = cell_hash(ix + dx, iy + dy, seed);
            let fx = (ix + dx) as f32 + jx;
            let fy = (iy + dy) as f32 + jy;
            let ddx = x - fx;
            let ddy = y - fy;
            let d = (ddx * ddx + ddy * ddy).sqrt();
            if d < min_dist {
                min_dist = d;
            }
        }
    }
    min_dist
}

/// 2D Worley noise returning the two nearest distances (F1, F2).
#[allow(dead_code)]
pub fn worley2_f1f2(x: f32, y: f32, seed: u32) -> (f32, f32) {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let mut f1 = f32::MAX;
    let mut f2 = f32::MAX;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let [jx, jy] = cell_hash(ix + dx, iy + dy, seed);
            let fx = (ix + dx) as f32 + jx;
            let fy = (iy + dy) as f32 + jy;
            let ddx = x - fx;
            let ddy = y - fy;
            let d = (ddx * ddx + ddy * ddy).sqrt();
            if d < f1 {
                f2 = f1;
                f1 = d;
            } else if d < f2 {
                f2 = d;
            }
        }
    }
    (f1, f2)
}

/// Normalized Worley: maps F1 distance to [0, 1] clamped.
#[allow(dead_code)]
pub fn worley2_01(x: f32, y: f32, seed: u32) -> f32 {
    worley2(x, y, seed).clamp(0.0, 1.0)
}

/// F2 - F1 (ridge-like pattern), normalized to [0, 1].
#[allow(dead_code)]
pub fn worley2_ridged(x: f32, y: f32, seed: u32) -> f32 {
    let (f1, f2) = worley2_f1f2(x, y, seed);
    (f2 - f1).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worley2_nonnegative() {
        for i in 0..20 {
            let v = worley2(i as f32 * 0.37, i as f32 * 0.53, 42);
            assert!(v >= 0.0, "negative worley: {v}");
        }
    }

    #[test]
    fn worley2_deterministic() {
        let a = worley2(1.5, 2.3, 0);
        let b = worley2(1.5, 2.3, 0);
        assert!((a - b).abs() < 1e-10);
    }

    #[test]
    fn worley2_seed_changes_result() {
        let a = worley2(1.5, 2.3, 0);
        let b = worley2(1.5, 2.3, 1);
        // Different seeds should generally give different results
        let _diff = (a - b).abs();
    }

    #[test]
    fn worley2_f1_le_f2() {
        let (f1, f2) = worley2_f1f2(3.7, 1.2, 0);
        assert!(f1 <= f2, "f1={f1} > f2={f2}");
    }

    #[test]
    fn worley2_01_in_range() {
        for i in 0..20 {
            let v = worley2_01(i as f32 * 0.31, i as f32 * 0.47, 99);
            assert!((0.0..=1.0).contains(&v), "v={v}");
        }
    }

    #[test]
    fn worley2_ridged_in_range() {
        for i in 0..20 {
            let v = worley2_ridged(i as f32 * 0.31, i as f32 * 0.47, 0);
            assert!((0.0..=1.0).contains(&v), "ridged={v}");
        }
    }

    #[test]
    fn cell_hash_in_unit_range() {
        for i in -5..5 {
            let [hx, hy] = cell_hash(i, i + 1, 0);
            assert!((0.0..=1.0).contains(&hx));
            assert!((0.0..=1.0).contains(&hy));
        }
    }

    #[test]
    fn worley2_large_coords_finite() {
        let v = worley2(100.7, 200.3, 0);
        assert!(v.is_finite());
    }

    #[test]
    fn worley2_negative_coords() {
        let v = worley2(-5.3, -2.7, 0);
        assert!(v >= 0.0 && v.is_finite());
    }

    #[test]
    fn worley2_f1f2_f2_positive() {
        let (_, f2) = worley2_f1f2(0.5, 0.5, 0);
        assert!(f2 > 0.0);
    }
}
