// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//
// Fuzz target: UsdaWriter::write_blend_shape_animation with derived-Arbitrary
// structured input.
//
// Exercises the time-sample deduplication, shape-name ordering, and USDA text
// formatting logic inside `write_blend_shape_animation`.  Any panic (rather
// than a returned `Err`) would indicate a bug.

#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use oxihuman_export::{BlendShapeTimeSamples, UsdaWriter};

/// One blend-shape channel with a set of (time_code, weight) keyframes.
#[derive(Arbitrary, Debug)]
struct FuzzSample {
    shape_name: String,
    time_weight_pairs: Vec<(f64, f32)>,
}

/// Top-level structured input for a single `write_blend_shape_animation` call.
#[derive(Arbitrary, Debug)]
struct FuzzInput {
    mesh_path: String,
    samples: Vec<FuzzSample>,
}

fuzz_target!(|input: FuzzInput| {
    // Limit the number of samples and keyframes so the fuzzer spends its budget
    // on interesting input variation rather than pure allocation stress.
    if input.samples.len() > 64 {
        return;
    }
    for s in &input.samples {
        if s.time_weight_pairs.len() > 256 {
            return;
        }
    }

    let samples: Vec<BlendShapeTimeSamples> = input
        .samples
        .into_iter()
        .map(|s| BlendShapeTimeSamples {
            shape_name: s.shape_name,
            time_weight_pairs: s.time_weight_pairs,
        })
        .collect();

    let mut writer = UsdaWriter::new();
    // Must not panic — only return Ok or Err.
    let _ = writer.write_blend_shape_animation(&input.mesh_path, &samples);
});
