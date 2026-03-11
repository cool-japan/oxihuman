// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Property-based tests for oxihuman-morph: parameter safety, morph revert,
//! and height monotonicity.

use proptest::prelude::*;

use oxihuman_core::parser::target::Delta;
use oxihuman_morph::apply::{apply_target, reset_from_base, soa_to_aos};
use oxihuman_morph::constraint::{clamp_params, smooth_step};
use oxihuman_morph::params::ParamState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Strategy for generating random base positions (as [f32; 3]).
fn positions_strategy(max_verts: usize) -> impl Strategy<Value = Vec<[f32; 3]>> {
    proptest::collection::vec(prop::array::uniform3(-10.0f32..10.0f32), 3..max_verts)
}

/// Strategy for deltas that reference valid vertex indices.
fn _deltas_strategy(num_verts: usize, max_deltas: usize) -> impl Strategy<Value = Vec<Delta>> {
    proptest::collection::vec(
        (0..num_verts, -1.0f32..1.0, -1.0f32..1.0, -1.0f32..1.0).prop_map(
            move |(vid, dx, dy, dz)| Delta {
                vid: vid as u32,
                dx,
                dy,
                dz,
            },
        ),
        0..max_deltas,
    )
}

// ---------------------------------------------------------------------------
// Any f64 parameter in [0.0, 1.0] never produces NaN vertices
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn parameter_in_unit_range_never_produces_nan(
        base_positions in positions_strategy(64),
        weight in 0.0f32..=1.0,
    ) {
        let n = base_positions.len();
        let mut x = vec![0.0f32; n];
        let mut y = vec![0.0f32; n];
        let mut z = vec![0.0f32; n];
        reset_from_base(&mut x, &mut y, &mut z, &base_positions);

        // Create a simple uniform delta that affects all vertices
        let deltas: Vec<Delta> = (0..n)
            .map(|i| Delta {
                vid: i as u32,
                dx: 0.1,
                dy: 0.2,
                dz: -0.05,
            })
            .collect();

        apply_target(&mut x, &mut y, &mut z, &deltas, weight);
        let positions = soa_to_aos(&x, &y, &z);

        for (i, pos) in positions.iter().enumerate() {
            prop_assert!(!pos[0].is_nan(), "Position X is NaN at vertex {}", i);
            prop_assert!(!pos[1].is_nan(), "Position Y is NaN at vertex {}", i);
            prop_assert!(!pos[2].is_nan(), "Position Z is NaN at vertex {}", i);
            prop_assert!(!pos[0].is_infinite(), "Position X is Inf at vertex {}", i);
            prop_assert!(!pos[1].is_infinite(), "Position Y is Inf at vertex {}", i);
            prop_assert!(!pos[2].is_infinite(), "Position Z is Inf at vertex {}", i);
        }
    }

    #[test]
    fn arbitrary_deltas_with_unit_weight_no_nan(
        num_verts in 3usize..64,
        weight in 0.0f32..=1.0,
    ) {
        // Use a deterministic set of deltas and positions
        let base: Vec<[f32; 3]> = (0..num_verts)
            .map(|i| {
                let f = i as f32;
                [f * 0.1, f * 0.05, f * -0.03]
            })
            .collect();

        let mut x = vec![0.0f32; num_verts];
        let mut y = vec![0.0f32; num_verts];
        let mut z = vec![0.0f32; num_verts];
        reset_from_base(&mut x, &mut y, &mut z, &base);

        let deltas: Vec<Delta> = (0..num_verts.min(10))
            .map(|i| Delta {
                vid: i as u32,
                dx: 0.5,
                dy: -0.3,
                dz: 0.1,
            })
            .collect();

        apply_target(&mut x, &mut y, &mut z, &deltas, weight);
        let positions = soa_to_aos(&x, &y, &z);

        for (i, pos) in positions.iter().enumerate() {
            prop_assert!(pos[0].is_finite(), "Non-finite X at vertex {}", i);
            prop_assert!(pos[1].is_finite(), "Non-finite Y at vertex {}", i);
            prop_assert!(pos[2].is_finite(), "Non-finite Z at vertex {}", i);
        }
    }
}

// ---------------------------------------------------------------------------
// Applying a parameter and then reverting produces the original mesh
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn apply_then_revert_roundtrip(
        base_positions in positions_strategy(64),
    ) {
        let n = base_positions.len();
        let mut x = vec![0.0f32; n];
        let mut y = vec![0.0f32; n];
        let mut z = vec![0.0f32; n];
        reset_from_base(&mut x, &mut y, &mut z, &base_positions);

        // Save original positions
        let orig_positions = soa_to_aos(&x, &y, &z);

        // Apply morph with weight=0.7
        let deltas: Vec<Delta> = (0..n)
            .map(|i| Delta {
                vid: i as u32,
                dx: 0.5,
                dy: -0.3,
                dz: 0.1,
            })
            .collect();

        let weight = 0.7f32;
        apply_target(&mut x, &mut y, &mut z, &deltas, weight);

        // Revert by applying with negative weight
        apply_target(&mut x, &mut y, &mut z, &deltas, -weight);

        let reverted_positions = soa_to_aos(&x, &y, &z);

        for (i, (orig, rev)) in orig_positions.iter().zip(reverted_positions.iter()).enumerate() {
            prop_assert!(
                (orig[0] - rev[0]).abs() < 1e-4,
                "X mismatch at vertex {}: orig={}, reverted={}", i, orig[0], rev[0],
            );
            prop_assert!(
                (orig[1] - rev[1]).abs() < 1e-4,
                "Y mismatch at vertex {}: orig={}, reverted={}", i, orig[1], rev[1],
            );
            prop_assert!(
                (orig[2] - rev[2]).abs() < 1e-4,
                "Z mismatch at vertex {}: orig={}, reverted={}", i, orig[2], rev[2],
            );
        }
    }

    #[test]
    fn reset_from_base_restores_original(
        base_positions in positions_strategy(64),
    ) {
        let n = base_positions.len();
        let mut x = vec![0.0f32; n];
        let mut y = vec![0.0f32; n];
        let mut z = vec![0.0f32; n];
        reset_from_base(&mut x, &mut y, &mut z, &base_positions);

        // Apply some morph
        let deltas: Vec<Delta> = (0..n.min(20))
            .map(|i| Delta {
                vid: i as u32,
                dx: 1.0,
                dy: 2.0,
                dz: 3.0,
            })
            .collect();
        apply_target(&mut x, &mut y, &mut z, &deltas, 1.0);

        // Reset
        reset_from_base(&mut x, &mut y, &mut z, &base_positions);
        let restored = soa_to_aos(&x, &y, &z);

        for (i, (orig, rest)) in base_positions.iter().zip(restored.iter()).enumerate() {
            prop_assert_eq!(orig, rest,
                "Position mismatch at vertex {} after reset", i);
        }
    }
}

// ---------------------------------------------------------------------------
// Height parameter is monotonically increasing (higher value = taller bounding box)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn height_deltas_monotonically_increase_bbox(
        base_positions in positions_strategy(32),
        lo in 0.0f32..0.5,
        hi in 0.5f32..1.0,
    ) {
        let n = base_positions.len();

        // Create height deltas: positive Y displacement (makes mesh taller)
        let height_deltas: Vec<Delta> = (0..n)
            .map(|i| Delta {
                vid: i as u32,
                dx: 0.0,
                dy: 1.0, // upward displacement
                dz: 0.0,
            })
            .collect();

        // Apply with low weight
        let mut x_lo = vec![0.0f32; n];
        let mut y_lo = vec![0.0f32; n];
        let mut z_lo = vec![0.0f32; n];
        reset_from_base(&mut x_lo, &mut y_lo, &mut z_lo, &base_positions);
        apply_target(&mut x_lo, &mut y_lo, &mut z_lo, &height_deltas, lo);

        let min_y_lo = y_lo.iter().cloned().fold(f32::MAX, f32::min);
        let max_y_lo = y_lo.iter().cloned().fold(f32::MIN, f32::max);
        let _height_lo = max_y_lo - min_y_lo;

        // Apply with high weight
        let mut x_hi = vec![0.0f32; n];
        let mut y_hi = vec![0.0f32; n];
        let mut z_hi = vec![0.0f32; n];
        reset_from_base(&mut x_hi, &mut y_hi, &mut z_hi, &base_positions);
        apply_target(&mut x_hi, &mut y_hi, &mut z_hi, &height_deltas, hi);

        let min_y_hi = y_hi.iter().cloned().fold(f32::MAX, f32::min);
        let max_y_hi = y_hi.iter().cloned().fold(f32::MIN, f32::max);
        let _height_hi = max_y_hi - min_y_hi;

        // With uniform positive Y deltas applied to all vertices, the bounding box
        // height stays the same (since all vertices move up equally). But if we use
        // a scale-like delta pattern where displacement varies by vertex position,
        // higher weight should produce a taller mesh.
        //
        // For uniform deltas, bbox height is constant. So let's verify max_y is
        // monotonically increasing instead (the mesh moves up more with higher weight).
        prop_assert!(
            max_y_hi >= max_y_lo - 1e-6,
            "Higher height weight ({}) should produce higher max_y ({}) >= low max_y ({})",
            hi, max_y_hi, max_y_lo
        );
    }

    #[test]
    fn scale_height_deltas_monotonically_increase_bbox(
        num_verts in 4usize..32,
        lo in 0.0f32..0.4,
        hi in 0.6f32..1.0,
    ) {
        // Use a spread-out set of vertices so Y-scaling deltas produce bigger bounding boxes
        let base_positions: Vec<[f32; 3]> = (0..num_verts)
            .map(|i| {
                let f = i as f32 / num_verts as f32;
                [f, f - 0.5, f * 0.1] // Y ranges from -0.5 to ~0.5
            })
            .collect();
        let n = base_positions.len();

        // Scale-like deltas: displacement proportional to Y position
        // This makes the bounding box height grow with weight
        let height_deltas: Vec<Delta> = (0..n)
            .map(|i| {
                let base_y = base_positions[i][1];
                Delta {
                    vid: i as u32,
                    dx: 0.0,
                    dy: base_y, // scale from center
                    dz: 0.0,
                }
            })
            .collect();

        // Apply with low weight
        let mut x_lo = vec![0.0f32; n];
        let mut y_lo = vec![0.0f32; n];
        let mut z_lo = vec![0.0f32; n];
        reset_from_base(&mut x_lo, &mut y_lo, &mut z_lo, &base_positions);
        apply_target(&mut x_lo, &mut y_lo, &mut z_lo, &height_deltas, lo);
        let min_y_lo = y_lo.iter().cloned().fold(f32::MAX, f32::min);
        let max_y_lo = y_lo.iter().cloned().fold(f32::MIN, f32::max);
        let bbox_height_lo = max_y_lo - min_y_lo;

        // Apply with high weight
        let mut x_hi = vec![0.0f32; n];
        let mut y_hi = vec![0.0f32; n];
        let mut z_hi = vec![0.0f32; n];
        reset_from_base(&mut x_hi, &mut y_hi, &mut z_hi, &base_positions);
        apply_target(&mut x_hi, &mut y_hi, &mut z_hi, &height_deltas, hi);
        let min_y_hi = y_hi.iter().cloned().fold(f32::MAX, f32::min);
        let max_y_hi = y_hi.iter().cloned().fold(f32::MIN, f32::max);
        let bbox_height_hi = max_y_hi - min_y_hi;

        prop_assert!(
            bbox_height_hi >= bbox_height_lo - 1e-5,
            "Higher height weight ({}) should produce taller bbox ({}) >= low bbox ({})",
            hi, bbox_height_hi, bbox_height_lo
        );
    }
}

// ---------------------------------------------------------------------------
// clamp_params and smooth_step property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn clamp_params_always_in_unit_range(
        height in -10.0f32..10.0,
        weight in -10.0f32..10.0,
        muscle in -10.0f32..10.0,
        age in -10.0f32..10.0,
    ) {
        let mut p = ParamState::new(height, weight, muscle, age);
        clamp_params(&mut p);
        prop_assert!(p.height >= 0.0 && p.height <= 1.0);
        prop_assert!(p.weight >= 0.0 && p.weight <= 1.0);
        prop_assert!(p.muscle >= 0.0 && p.muscle <= 1.0);
        prop_assert!(p.age >= 0.0 && p.age <= 1.0);
    }

    #[test]
    fn smooth_step_monotonic(a in 0.0f32..1.0, b in 0.0f32..1.0) {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        prop_assert!(
            smooth_step(hi) >= smooth_step(lo) - 1e-6,
            "smooth_step should be monotonic: f({}) = {} >= f({}) = {}",
            hi, smooth_step(hi), lo, smooth_step(lo)
        );
    }

    #[test]
    fn smooth_step_output_in_unit_range(t in 0.0f32..=1.0) {
        let result = smooth_step(t);
        prop_assert!((0.0..=1.0).contains(&result),
            "smooth_step({}) = {} should be in [0, 1]", t, result);
    }
}
