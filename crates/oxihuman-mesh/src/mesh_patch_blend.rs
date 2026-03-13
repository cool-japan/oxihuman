// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Blend between two mesh patches using smooth weight interpolation.
#[allow(dead_code)]
pub struct PatchBlendResult {
    pub positions: Vec<[f32; 3]>,
    pub blend_weights: Vec<f32>,
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Smooth step function for weight blending.
#[allow(dead_code)]
pub fn smooth_weight(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Blend two vertex arrays with a scalar weight per vertex in `[0,1]`.
#[allow(dead_code)]
pub fn blend_patches(
    patch_a: &[[f32; 3]],
    patch_b: &[[f32; 3]],
    weights: &[f32],
) -> PatchBlendResult {
    let n = patch_a.len().min(patch_b.len()).min(weights.len());
    let positions: Vec<[f32; 3]> = (0..n)
        .map(|i| lerp3(patch_a[i], patch_b[i], smooth_weight(weights[i])))
        .collect();
    let blend_weights: Vec<f32> = weights[..n].iter().map(|&w| smooth_weight(w)).collect();
    PatchBlendResult {
        positions,
        blend_weights,
    }
}

/// Blend with a uniform scalar weight.
#[allow(dead_code)]
pub fn blend_patches_uniform(
    patch_a: &[[f32; 3]],
    patch_b: &[[f32; 3]],
    t: f32,
) -> PatchBlendResult {
    let n = patch_a.len().min(patch_b.len());
    let weights = vec![t; n];
    blend_patches(patch_a, patch_b, &weights)
}

#[allow(dead_code)]
pub fn patch_blend_vertex_count(r: &PatchBlendResult) -> usize {
    r.positions.len()
}

#[allow(dead_code)]
pub fn patch_blend_avg_weight(r: &PatchBlendResult) -> f32 {
    if r.blend_weights.is_empty() {
        return 0.0;
    }
    r.blend_weights.iter().sum::<f32>() / r.blend_weights.len() as f32
}

#[allow(dead_code)]
pub fn patch_blend_to_json(r: &PatchBlendResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"avg_weight\":{}}}",
        r.positions.len(),
        patch_blend_avg_weight(r)
    )
}

#[allow(dead_code)]
pub fn clamp_blend_weights(weights: &mut [f32]) {
    for w in weights.iter_mut() {
        *w = w.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn patch_distance(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let sum: f32 = (0..n)
        .map(|i| {
            let dx = a[i][0] - b[i][0];
            let dy = a[i][1] - b[i][1];
            let dz = a[i][2] - b[i][2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum();
    sum / n as f32
}

#[allow(dead_code)]
pub fn all_weights_valid(weights: &[f32]) -> bool {
    weights
        .iter()
        .all(|&w| (0.0..=1.0).contains(&w) && w.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_patch(n: usize, y: f32) -> Vec<[f32; 3]> {
        (0..n).map(|i| [i as f32, y, 0.0]).collect()
    }

    #[test]
    fn test_blend_uniform_zero_gives_patch_a() {
        let a = flat_patch(4, 0.0);
        let b = flat_patch(4, 1.0);
        let r = blend_patches_uniform(&a, &b, 0.0);
        for (p, &orig) in r.positions.iter().zip(a.iter()) {
            assert!((p[1] - orig[1]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_blend_uniform_one_gives_patch_b() {
        let a = flat_patch(4, 0.0);
        let b = flat_patch(4, 2.0);
        let r = blend_patches_uniform(&a, &b, 1.0);
        for (p, &orig) in r.positions.iter().zip(b.iter()) {
            assert!((p[1] - orig[1]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_blend_midpoint() {
        let a = flat_patch(4, 0.0);
        let b = flat_patch(4, 2.0);
        let r = blend_patches_uniform(&a, &b, 0.5);
        let avg_w = patch_blend_avg_weight(&r);
        assert!(avg_w > 0.0 && avg_w < 1.0);
    }

    #[test]
    fn test_vertex_count() {
        let a = flat_patch(6, 0.0);
        let b = flat_patch(6, 1.0);
        let r = blend_patches_uniform(&a, &b, 0.5);
        assert_eq!(patch_blend_vertex_count(&r), 6);
    }

    #[test]
    fn test_empty_blend() {
        let r = blend_patches_uniform(&[], &[], 0.5);
        assert_eq!(r.positions.len(), 0);
    }

    #[test]
    fn test_smooth_weight_midpoint() {
        let w = smooth_weight(0.5);
        assert!((w - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_smooth_weight_zero_one() {
        assert!((smooth_weight(0.0)).abs() < 1e-6);
        assert!((smooth_weight(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_all_weights_valid() {
        assert!(all_weights_valid(&[0.0, 0.5, 1.0]));
        assert!(!all_weights_valid(&[0.0, 1.5, 0.5]));
    }

    #[test]
    fn test_patch_distance() {
        let a = flat_patch(4, 0.0);
        let b = flat_patch(4, 1.0);
        let d = patch_distance(&a, &b);
        assert!((d - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let a = flat_patch(4, 0.0);
        let b = flat_patch(4, 1.0);
        let r = blend_patches_uniform(&a, &b, 0.5);
        let j = patch_blend_to_json(&r);
        assert!(j.contains("vertex_count"));
    }
}
