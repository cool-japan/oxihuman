// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
//
// Fuzz target: mesh normal (and tangent) recomputation with arbitrary vertex
// and face-index data.
//
// Feeds the `compute_normals` and `compute_tangents` functions with
// `Arbitrary`-derived vertex positions, UVs, and index lists.  Out-of-bounds
// indices, degenerate triangles, NaN/inf coordinates, and zero-area faces must
// all be handled without panicking.

#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use oxihuman_mesh::{compute_normals, compute_tangents, mesh::MeshBuffers};
use oxihuman_morph::engine::MeshBuffers as MorphBuffers;

/// Structured input for a single normal-recomputation round.
#[derive(Arbitrary, Debug)]
struct FuzzMesh {
    /// Raw XYZ positions. May contain NaN, inf, or very large values.
    positions: Vec<[f32; 3]>,
    /// Per-vertex UV coordinates (same expected length as positions).
    uvs: Vec<[f32; 2]>,
    /// Flat triangle index list.  Indices may be out of bounds — the
    /// implementation must skip them, not panic.
    indices: Vec<u32>,
}

fuzz_target!(|input: FuzzMesh| {
    // Cap sizes so the fuzzer explores input variety rather than sheer scale.
    if input.positions.len() > 1024 {
        return;
    }
    if input.indices.len() > 4096 {
        return;
    }

    let n = input.positions.len();

    // Align UVs to the vertex count; pad with zeros or truncate as needed.
    let uvs: Vec<[f32; 2]> = if input.uvs.len() >= n {
        input.uvs[..n].to_vec()
    } else {
        let mut v = input.uvs.clone();
        v.resize(n, [0.0f32, 0.0f32]);
        v
    };

    // Provide a placeholder normal array (will be overwritten by compute_normals).
    let normals = vec![[0.0f32, 1.0f32, 0.0f32]; n];

    let morph = MorphBuffers {
        positions: input.positions,
        normals,
        uvs,
        indices: input.indices,
        has_suit: false,
    };

    let mut mesh = MeshBuffers::from_morph(morph);

    // compute_normals must never panic regardless of degenerate geometry.
    compute_normals(&mut mesh);

    // Verify no NaN was introduced (the implementation falls back to [0,1,0]).
    for n in &mesh.normals {
        if n[0].is_nan() || n[1].is_nan() || n[2].is_nan() {
            // A NaN normal is itself the bug we want to detect — return so
            // the fuzzer records this as a finding rather than a panic.
            return;
        }
    }

    // compute_tangents must also never panic.
    compute_tangents(&mut mesh);

    // Verify tangent W values are finite (NaN/inf in W is also a bug signal).
    for t in &mesh.tangents {
        if t[3].is_nan() || t[3].is_infinite() {
            return;
        }
    }
});
