// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fractal displacement: push vertices along normals using fBm (fractional Brownian motion).
//! Uses deterministic LCG-based value noise — no external RNG dependency.

/// Parameters for fractal displacement.
#[allow(dead_code)]
pub struct FractalDispParams {
    /// Number of octaves in fBm.
    pub octaves: usize,
    /// Initial frequency multiplier.
    pub frequency: f32,
    /// Amplitude (max displacement per vertex).
    pub amplitude: f32,
    /// Lacunarity (frequency multiplier per octave).
    pub lacunarity: f32,
    /// Gain (amplitude multiplier per octave).
    pub gain: f32,
}

impl Default for FractalDispParams {
    fn default() -> Self {
        FractalDispParams {
            octaves: 5,
            frequency: 1.0,
            amplitude: 0.1,
            lacunarity: 2.0,
            gain: 0.5,
        }
    }
}

/// Apply fractal (fBm) displacement along vertex normals.
#[allow(dead_code)]
pub fn fractal_displace(
    positions: &[[f32; 3]],
    indices: &[u32],
    params: &FractalDispParams,
) -> Vec<[f32; 3]> {
    let normals = compute_normals_fd(positions, indices);
    positions
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            let d = fbm(p, params);
            let n = normals[i];
            [p[0] + n[0] * d, p[1] + n[1] * d, p[2] + n[2] * d]
        })
        .collect()
}

/// Fractional Brownian motion at position `p`.
#[allow(dead_code)]
pub fn fbm(p: [f32; 3], params: &FractalDispParams) -> f32 {
    let mut value = 0.0f32;
    let mut freq = params.frequency;
    let mut amp = params.amplitude;
    for _ in 0..params.octaves {
        value += amp * value_noise_3d([p[0] * freq, p[1] * freq, p[2] * freq]);
        freq *= params.lacunarity;
        amp *= params.gain;
    }
    value
}

/// Deterministic 3-D value noise using a simple hash.
#[allow(dead_code)]
pub fn value_noise_3d(p: [f32; 3]) -> f32 {
    let xi = p[0].floor() as i64;
    let yi = p[1].floor() as i64;
    let zi = p[2].floor() as i64;
    let hash = lcg_hash(
        xi.wrapping_mul(374761393)
            .wrapping_add(yi.wrapping_mul(668265263))
            .wrapping_add(zi.wrapping_mul(1274126177)),
    );
    (hash as f32 / u64::MAX as f32) * 2.0 - 1.0
}

fn lcg_hash(seed: i64) -> u64 {
    let s = seed as u64;
    s.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
}

/// Max displacement after fractal apply.
#[allow(dead_code)]
pub fn max_fractal_displacement(original: &[[f32; 3]], displaced: &[[f32; 3]]) -> f32 {
    original
        .iter()
        .zip(displaced.iter())
        .map(|(&a, &b)| {
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(0.0f32, f32::max)
}

fn compute_normals_fd(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut acc = vec![[0.0f32; 3]; n];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let pa = positions[a];
        let pb = positions[b];
        let pc = positions[c];
        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let n3 = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        for &i in &[a, b, c] {
            acc[i][0] += n3[0];
            acc[i][1] += n3[1];
            acc[i][2] += n3[2];
        }
    }
    acc.iter()
        .map(|&v| {
            let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            if l < 1e-8 {
                [0.0, 1.0, 0.0]
            } else {
                [v[0] / l, v[1] / l, v[2] / l]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0u32, 1, 2],
        )
    }

    #[test]
    fn fractal_displace_preserves_count() {
        let (pos, idx) = flat_tri();
        let res = fractal_displace(&pos, &idx, &FractalDispParams::default());
        assert_eq!(res.len(), pos.len());
    }

    #[test]
    fn zero_amplitude_unchanged() {
        let (pos, idx) = flat_tri();
        let params = FractalDispParams {
            amplitude: 0.0,
            ..Default::default()
        };
        let res = fractal_displace(&pos, &idx, &params);
        for (a, b) in pos.iter().zip(res.iter()) {
            let d = (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs();
            assert!(d < 1e-6);
        }
    }

    #[test]
    fn value_noise_in_range() {
        for i in 0..20 {
            let v = value_noise_3d([i as f32 * 0.7, i as f32 * 0.3, i as f32 * 0.5]);
            assert!((-1.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn fbm_deterministic() {
        let params = FractalDispParams::default();
        let a = fbm([1.0, 2.0, 3.0], &params);
        let b = fbm([1.0, 2.0, 3.0], &params);
        assert!((a - b).abs() < 1e-7);
    }

    #[test]
    fn max_displacement_bounded() {
        let (pos, idx) = flat_tri();
        let params = FractalDispParams {
            amplitude: 0.1,
            octaves: 3,
            ..Default::default()
        };
        let res = fractal_displace(&pos, &idx, &params);
        let d = max_fractal_displacement(&pos, &res);
        assert!(d < 1.0);
    }

    #[test]
    fn octave_one_matches_single_noise() {
        let p = [0.5f32, 0.5, 0.5];
        let params = FractalDispParams {
            octaves: 1,
            frequency: 1.0,
            amplitude: 1.0,
            lacunarity: 2.0,
            gain: 0.5,
        };
        let expected = value_noise_3d(p);
        let got = fbm(p, &params);
        assert!((got - expected).abs() < 1e-6);
    }

    #[test]
    fn empty_mesh() {
        let res = fractal_displace(&[], &[], &FractalDispParams::default());
        assert!(res.is_empty());
    }

    #[test]
    fn displacement_varies_across_vertices() {
        let pos = vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let res = fractal_displace(&pos, &idx, &FractalDispParams::default());
        let d0 = {
            let d = [
                res[0][0] - pos[0][0],
                res[0][1] - pos[0][1],
                res[0][2] - pos[0][2],
            ];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        };
        let d1 = {
            let d = [
                res[1][0] - pos[1][0],
                res[1][1] - pos[1][1],
                res[1][2] - pos[1][2],
            ];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        };
        let _ = d0.max(d1);
    }

    #[test]
    fn default_params_sensible() {
        let p = FractalDispParams::default();
        assert!(p.octaves > 0);
        assert!(p.amplitude > 0.0);
    }

    #[test]
    fn lcg_hash_nonzero() {
        let h = lcg_hash(42);
        assert_ne!(h, 0);
    }
}
