// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// LCG-based deterministic pseudo-random number generator for morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphRng {
    pub state: u64,
}

/// Create a new MorphRng with the given seed.
#[allow(dead_code)]
pub fn new_morph_rng(seed: u64) -> MorphRng {
    MorphRng { state: seed.wrapping_add(1) }
}

/// Advance the LCG and return the next f32 in [0, 1).
#[allow(dead_code)]
pub fn rng_next_f32(rng: &mut MorphRng) -> f32 {
    // LCG: multiplier and increment from Knuth
    rng.state = rng.state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let bits = (rng.state >> 33) as u32;
    bits as f32 / (u32::MAX as f32 + 1.0)
}

/// Return a random f32 in [min, max].
#[allow(dead_code)]
pub fn rng_range(rng: &mut MorphRng, min: f32, max: f32) -> f32 {
    let t = rng_next_f32(rng);
    min + t * (max - min)
}

/// Randomize each weight by adding a random delta scaled by strength.
#[allow(dead_code)]
pub fn randomize_weights(rng: &mut MorphRng, weights: &mut [f32], strength: f32) {
    for w in weights.iter_mut() {
        let delta = rng_range(rng, -1.0, 1.0) * strength;
        *w = (*w + delta).clamp(0.0, 1.0);
    }
}

/// Return the current RNG state (seed).
#[allow(dead_code)]
pub fn rng_seed(rng: &MorphRng) -> u64 {
    rng.state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_morph_rng_not_zero() {
        let rng = new_morph_rng(42);
        assert_ne!(rng.state, 0);
    }

    #[test]
    fn rng_next_f32_in_unit_range() {
        let mut rng = new_morph_rng(1);
        for _ in 0..100 {
            let v = rng_next_f32(&mut rng);
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn rng_next_f32_deterministic_same_seed() {
        let mut a = new_morph_rng(999);
        let mut b = new_morph_rng(999);
        assert_eq!(
            rng_next_f32(&mut a).to_bits(),
            rng_next_f32(&mut b).to_bits()
        );
    }

    #[test]
    fn rng_range_in_bounds() {
        let mut rng = new_morph_rng(7);
        for _ in 0..50 {
            let v = rng_range(&mut rng, 0.2, 0.8);
            assert!((0.2..=0.8).contains(&v));
        }
    }

    #[test]
    fn rng_range_min_le_max() {
        let mut rng = new_morph_rng(3);
        let v = rng_range(&mut rng, 0.5, 0.5);
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn randomize_weights_stays_clamped() {
        let mut rng = new_morph_rng(55);
        let mut weights = vec![0.5, 0.5, 0.5];
        randomize_weights(&mut rng, &mut weights, 1.0);
        for &w in &weights {
            assert!((0.0..=1.0).contains(&w));
        }
    }

    #[test]
    fn randomize_weights_changes_values() {
        let mut rng = new_morph_rng(13);
        let mut weights = vec![0.5, 0.5, 0.5];
        let orig = weights.clone();
        randomize_weights(&mut rng, &mut weights, 0.5);
        // At least one weight should differ with high probability
        let changed = weights.iter().zip(orig.iter()).any(|(a, b)| (a - b).abs() > 1e-9);
        assert!(changed);
    }

    #[test]
    fn rng_seed_returns_state() {
        let rng = new_morph_rng(100);
        assert_eq!(rng_seed(&rng), rng.state);
    }

    #[test]
    fn different_seeds_give_different_values() {
        let mut a = new_morph_rng(1);
        let mut b = new_morph_rng(2);
        // Advance both and collect sequences; they should differ
        let va: Vec<u32> = (0..5).map(|_| rng_next_f32(&mut a).to_bits()).collect();
        let vb: Vec<u32> = (0..5).map(|_| rng_next_f32(&mut b).to_bits()).collect();
        assert_ne!(va, vb);
    }
}
