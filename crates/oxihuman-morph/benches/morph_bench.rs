// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use oxihuman_core::parser::obj::ObjMesh;
use oxihuman_core::parser::target::Delta;
use oxihuman_core::policy::{Policy, PolicyProfile};
use oxihuman_morph::apply::apply_target;
use oxihuman_morph::character_dna::{decode_dna, encode_dna};
use oxihuman_morph::engine::HumanEngine;
use oxihuman_morph::params::ParamState;
use std::hint::black_box;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Minimal 5-vertex, 2-triangle ObjMesh used as the engine base.
fn minimal_base() -> ObjMesh {
    ObjMesh {
        positions: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ],
        normals: vec![[0.0, 0.0, 1.0]; 5],
        uvs: vec![[0.0, 0.0]; 5],
        indices: vec![0, 1, 2, 1, 3, 2],
    }
}

/// Build an engine pre-loaded with `target_count` synthetic targets,
/// each having `deltas_per_target` deltas spread across the 5-vertex base.
fn build_engine(target_count: usize, deltas_per_target: usize) -> HumanEngine {
    let policy = Policy::new(PolicyProfile::Standard);
    let mut engine = HumanEngine::new(minimal_base(), policy);

    let n_verts = 5u32;
    for t in 0..target_count {
        let deltas: Vec<Delta> = (0..deltas_per_target)
            .map(|i| Delta {
                vid: (i as u32) % n_verts,
                dx: 0.01 * (t as f32 + 1.0),
                dy: 0.02,
                dz: -0.01,
            })
            .collect();
        engine.load_target(
            oxihuman_core::parser::target::TargetFile {
                name: format!("synth_{t:03}"),
                deltas,
            },
            Box::new(move |p: &ParamState| p.height * 0.5 + (t as f32) * 0.05),
        );
    }
    engine.set_params(ParamState::new(0.5, 0.5, 0.5, 0.5));
    engine
}

// ── original benchmarks ───────────────────────────────────────────────────────

fn bench_apply_target_500_deltas(c: &mut Criterion) {
    let n = 19_158usize;
    let mut x = vec![0.0f32; n];
    let mut y = vec![0.0f32; n];
    let mut z = vec![0.0f32; n];

    // 500 sparse deltas (typical for a real target)
    let deltas: Vec<Delta> = (0..500u32)
        .map(|i| Delta {
            vid: i * 38,
            dx: 0.01,
            dy: 0.02,
            dz: -0.01,
        })
        .collect();

    c.bench_function("apply_target_500_deltas", |b| {
        b.iter(|| {
            apply_target(
                black_box(&mut x),
                black_box(&mut y),
                black_box(&mut z),
                black_box(&deltas),
                black_box(0.5f32),
            )
        })
    });
}

fn bench_apply_targets_parallel(c: &mut Criterion) {
    use oxihuman_morph::apply::apply_targets_parallel;
    let n = 19_158usize;
    let mut x = vec![0.0f32; n];
    let mut y = vec![0.0f32; n];
    let mut z = vec![0.0f32; n];

    let targets_data: Vec<Vec<Delta>> = (0..20)
        .map(|offset| {
            let mut v: Vec<Delta> = (0..500u32)
                .map(|i| Delta {
                    vid: (i * 38 + offset) % n as u32,
                    dx: 0.01,
                    dy: 0.02,
                    dz: -0.01,
                })
                .collect();
            v.sort_unstable_by_key(|d| d.vid);
            v
        })
        .collect();

    let targets: Vec<(&[Delta], f32)> = targets_data
        .iter()
        .map(|t| (t.as_slice(), 0.5f32))
        .collect();

    c.bench_function("apply_targets_parallel_20x500", |b| {
        b.iter(|| {
            apply_targets_parallel(
                black_box(&mut x),
                black_box(&mut y),
                black_box(&mut z),
                black_box(&targets),
            )
        })
    });
}

// ── new benchmarks ────────────────────────────────────────────────────────────

/// Full mesh build with 10 synthetic targets (500 deltas each).
/// Setup (engine creation) is outside the timed section via `iter_batched`.
fn bench_engine_full_build(c: &mut Criterion) {
    c.bench_function("engine_full_build_10x500", |b| {
        b.iter_batched(
            || build_engine(10, 500),
            |engine| black_box(engine.build_mesh()),
            BatchSize::SmallInput,
        )
    });
}

/// Incremental build — same 10-target engine, change one param then rebuild.
/// Should be faster than `build_mesh` when the cache is warm.
fn bench_engine_incremental_build(c: &mut Criterion) {
    c.bench_function("engine_incremental_build_10x500", |b| {
        b.iter_batched(
            || {
                let mut engine = build_engine(10, 500);
                // Pre-warm with first build
                let _ = engine.build_mesh();
                // Change a single param so incremental has real work to do
                engine.set_params(ParamState::new(0.6, 0.5, 0.5, 0.5));
                engine
            },
            |mut engine| black_box(engine.build_mesh_incremental()),
            BatchSize::SmallInput,
        )
    });
}

/// Encode a `ParamState` to DNA and decode it back in a tight loop.
fn bench_dna_encode_decode(c: &mut Criterion) {
    let params = ParamState::new(0.3, 0.7, 0.4, 0.6);

    c.bench_function("dna_encode_decode_roundtrip", |b| {
        b.iter(|| {
            let dna = encode_dna(black_box(&params));
            black_box(decode_dna(black_box(&dna)))
        })
    });
}

/// Parallel build with 10 synthetic targets — compare throughput against
/// the sequential `build_mesh` baseline above.
fn bench_build_mesh_parallel(c: &mut Criterion) {
    c.bench_function("engine_parallel_build_10x500", |b| {
        b.iter_batched(
            || build_engine(10, 500),
            |engine| black_box(engine.build_mesh_parallel()),
            BatchSize::SmallInput,
        )
    });
}

/// Encode/decode for a variety of param values to stress the quantisation path.
fn bench_dna_encode_varied(c: &mut Criterion) {
    let param_set: Vec<ParamState> = (0..8)
        .map(|i| {
            let v = i as f32 / 7.0;
            ParamState::new(v, 1.0 - v, v * 0.5, 0.5 + v * 0.25)
        })
        .collect();

    c.bench_function("dna_encode_decode_8_variants", |b| {
        b.iter(|| {
            for p in black_box(&param_set) {
                let dna = encode_dna(p);
                let _ = black_box(decode_dna(&dna));
            }
        })
    });
}

/// Sequential build used as a direct comparison point for the parallel bench.
fn bench_engine_sequential_vs_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_vs_parallel_10x500");

    group.bench_function("sequential", |b| {
        b.iter_batched(
            || build_engine(10, 500),
            |engine| black_box(engine.build_mesh()),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("parallel", |b| {
        b.iter_batched(
            || build_engine(10, 500),
            |engine| black_box(engine.build_mesh_parallel()),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_apply_target_500_deltas,
    bench_apply_targets_parallel,
    bench_engine_full_build,
    bench_engine_incremental_build,
    bench_dna_encode_decode,
    bench_build_mesh_parallel,
    bench_dna_encode_varied,
    bench_engine_sequential_vs_parallel,
);
criterion_main!(benches);
