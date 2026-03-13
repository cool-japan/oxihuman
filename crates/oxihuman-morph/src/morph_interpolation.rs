// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A morph keyframe: a point in time with associated weights.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphKeyframe {
    pub time: f32,
    pub weights: Vec<f32>,
}

/// Linear interpolation between two weight slices.
#[allow(dead_code)]
pub fn linear_morph_interp(a: &[f32], b: &[f32], t: f32) -> Vec<f32> {
    let len = a.len().min(b.len());
    (0..len).map(|i| a[i] + (b[i] - a[i]) * t).collect()
}

/// Catmull-Rom cubic interpolation using four control points.
/// km1, k0, k1, k2 are weights at t=-1, 0, 1, 2 respectively; t is in [0, 1].
#[allow(dead_code)]
pub fn cubic_morph_interp(km1: &[f32], k0: &[f32], k1: &[f32], k2: &[f32], t: f32) -> Vec<f32> {
    let len = km1.len().min(k0.len()).min(k1.len()).min(k2.len());
    (0..len)
        .map(|i| {
            let t2 = t * t;
            let t3 = t2 * t;
            0.5 * ((2.0 * k0[i])
                + (-km1[i] + k1[i]) * t
                + (2.0 * km1[i] - 5.0 * k0[i] + 4.0 * k1[i] - k2[i]) * t2
                + (-km1[i] + 3.0 * k0[i] - 3.0 * k1[i] + k2[i]) * t3)
        })
        .collect()
}

/// Sample a curve defined by keyframes at the given time using linear interpolation.
#[allow(dead_code)]
pub fn morph_curve_sample(keyframes: &[MorphKeyframe], time: f32) -> Vec<f32> {
    if keyframes.is_empty() {
        return Vec::new();
    }
    if time <= keyframes[0].time {
        return keyframes[0].weights.clone();
    }
    if time >= keyframes[keyframes.len() - 1].time {
        return keyframes[keyframes.len() - 1].weights.clone();
    }
    for i in 0..keyframes.len() - 1 {
        let a = &keyframes[i];
        let b = &keyframes[i + 1];
        if time >= a.time && time <= b.time {
            let span = b.time - a.time;
            let t = if span.abs() < 1e-9 { 0.0 } else { (time - a.time) / span };
            return linear_morph_interp(&a.weights, &b.weights, t);
        }
    }
    keyframes[keyframes.len() - 1].weights.clone()
}

/// Return the number of keyframes.
#[allow(dead_code)]
pub fn keyframe_count(kfs: &[MorphKeyframe]) -> usize {
    kfs.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_interp_midpoint() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 1.0];
        let r = linear_morph_interp(&a, &b, 0.5);
        assert!((r[0] - 0.5).abs() < 1e-6);
        assert!((r[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn linear_interp_t0_gives_a() {
        let a = vec![0.2, 0.4];
        let b = vec![0.8, 0.6];
        let r = linear_morph_interp(&a, &b, 0.0);
        assert!((r[0] - 0.2).abs() < 1e-6);
    }

    #[test]
    fn linear_interp_t1_gives_b() {
        let a = vec![0.2];
        let b = vec![0.8];
        let r = linear_morph_interp(&a, &b, 1.0);
        assert!((r[0] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn cubic_interp_at_t0_gives_k0() {
        let k = vec![0.5f32];
        let r = cubic_morph_interp(&k, &k, &k, &k, 0.0);
        assert!((r[0] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn keyframe_count_works() {
        let kfs = vec![
            MorphKeyframe { time: 0.0, weights: vec![0.0] },
            MorphKeyframe { time: 1.0, weights: vec![1.0] },
        ];
        assert_eq!(keyframe_count(&kfs), 2);
    }

    #[test]
    fn morph_curve_sample_empty_returns_empty() {
        let r = morph_curve_sample(&[], 0.5);
        assert!(r.is_empty());
    }

    #[test]
    fn morph_curve_sample_before_start() {
        let kfs = vec![
            MorphKeyframe { time: 1.0, weights: vec![0.2] },
            MorphKeyframe { time: 2.0, weights: vec![0.8] },
        ];
        let r = morph_curve_sample(&kfs, 0.0);
        assert!((r[0] - 0.2).abs() < 1e-5);
    }

    #[test]
    fn morph_curve_sample_after_end() {
        let kfs = vec![
            MorphKeyframe { time: 0.0, weights: vec![0.0] },
            MorphKeyframe { time: 1.0, weights: vec![1.0] },
        ];
        let r = morph_curve_sample(&kfs, 2.0);
        assert!((r[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn morph_curve_sample_interpolates() {
        let kfs = vec![
            MorphKeyframe { time: 0.0, weights: vec![0.0] },
            MorphKeyframe { time: 1.0, weights: vec![1.0] },
        ];
        let r = morph_curve_sample(&kfs, 0.5);
        assert!((r[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn keyframe_count_empty() {
        let kfs: Vec<MorphKeyframe> = vec![];
        assert_eq!(keyframe_count(&kfs), 0);
    }
}
