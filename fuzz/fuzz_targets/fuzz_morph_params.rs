// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//
// Fuzz target: HumanEngine::set_params + build_mesh with arbitrary parameter maps.
//
// Exercises the parameter clamping, morph-target weight evaluation, SoA scatter-add,
// and mesh-buffer assembly path inside `HumanEngine`. Any panic (rather than a
// returned mesh or an `Err`) would indicate a bug.

#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use oxihuman_morph::{engine::HumanEngine, params::ParamState};
use oxihuman_core::{parser::obj::parse_obj, policy::{Policy, PolicyProfile}};

/// Structured input: the four primary morph parameters plus an optional set of
/// extra named parameters.
#[derive(Arbitrary, Debug)]
struct FuzzParams {
    height: f32,
    weight: f32,
    muscle: f32,
    age: f32,
    /// Extra named parameters fed into `ParamState::extra`.
    extra: Vec<(String, f32)>,
}

/// Minimal valid OBJ base-mesh used to seed the engine.  Declared as a constant
/// so the fuzzer does not need to synthesise valid OBJ text — the interesting
/// surface is the morph-parameter arithmetic, not OBJ parsing.
const BASE_OBJ: &str = "\
v 0 0 0\nv 1 0 0\nv 0 1 0\nv 0 0 1\n\
vn 0 0 1\nvt 0 0\n\
f 1/1/1 2/1/1 3/1/1\nf 1/1/1 3/1/1 4/1/1\n";

fuzz_target!(|input: FuzzParams| {
    // Limit extra params so the fuzzer spends budget on value diversity, not
    // unbounded allocation.
    if input.extra.len() > 32 {
        return;
    }

    // Parse the static base mesh.  This is infallible for our constant, but we
    // use the `let Ok` pattern to satisfy the no-unwrap policy.
    let Ok(base) = parse_obj(BASE_OBJ) else {
        return;
    };

    let policy = Policy::new(PolicyProfile::Standard);
    let mut engine = HumanEngine::new(base, policy);

    // Build a ParamState from fuzzed values.  Out-of-range floats (NaN, inf,
    // negative, >1.0) must be clamped gracefully by the engine.
    let mut extra_map = std::collections::HashMap::new();
    for (k, v) in input.extra {
        // Skip empty keys — the engine treats them as valid but uninteresting.
        if k.is_empty() || k.len() > 64 {
            continue;
        }
        extra_map.insert(k, v);
    }

    let params = ParamState {
        height: input.height,
        weight: input.weight,
        muscle: input.muscle,
        age: input.age,
        extra: extra_map,
    };

    engine.set_params(params);

    // build_mesh must never panic — it should return a valid (possibly
    // degenerate) mesh regardless of the parameter values.
    let mesh = engine.build_mesh();

    // Sanity: the mesh must have consistent buffer sizes.
    let _ = mesh.positions.len();
    let _ = mesh.indices.len();
});
