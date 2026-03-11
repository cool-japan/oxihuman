// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use oxihuman_mesh::curvature::compute_curvature;
use oxihuman_mesh::decimate::decimate_ratio;
use oxihuman_mesh::lod::{generate_lod, LodLevel};
use oxihuman_mesh::mesh::MeshBuffers;
use oxihuman_mesh::normals::{compute_normals, compute_tangents};
use oxihuman_mesh::weld::weld_by_position;
use oxihuman_morph::engine::MeshBuffers as MB;

// ── mesh factories ─────────────────────────────────────────────────────────

/// Grid mesh with approximately `n` total vertices (side × side grid, 2 triangles per quad).
fn make_large_mesh(n: usize) -> MeshBuffers {
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    let side = (n as f32).sqrt() as usize + 1;
    for row in 0..side {
        for col in 0..side {
            positions.push([col as f32 * 0.1, row as f32 * 0.1, 0.0f32]);
            uvs.push([col as f32 / side as f32, row as f32 / side as f32]);
            normals.push([0.0f32, 0.0, 1.0]);
        }
    }
    for row in 0..side - 1 {
        for col in 0..side - 1 {
            let tl = (row * side + col) as u32;
            let tr = tl + 1;
            let bl = tl + side as u32;
            let br = bl + 1;
            indices.extend_from_slice(&[tl, tr, bl, tr, br, bl]);
        }
    }

    MeshBuffers::from_morph(MB {
        positions,
        normals,
        uvs,
        indices,
        has_suit: false,
    })
}

/// Mesh with intentional duplicate vertices — good input for weld benchmarks.
/// Vertices are emitted per-triangle face so all shared positions are duplicated.
fn make_mesh_with_duplicates(n_unique: usize) -> MeshBuffers {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let side = (n_unique as f32).sqrt() as usize + 1;
    let mut idx = 0u32;

    for row in 0..side - 1 {
        for col in 0..side - 1 {
            let tl = [col as f32 * 0.1, row as f32 * 0.1, 0.0f32];
            let tr = [(col + 1) as f32 * 0.1, row as f32 * 0.1, 0.0f32];
            let bl = [col as f32 * 0.1, (row + 1) as f32 * 0.1, 0.0f32];
            let br = [(col + 1) as f32 * 0.1, (row + 1) as f32 * 0.1, 0.0f32];

            // Triangle 1
            for &p in &[tl, tr, bl] {
                positions.push(p);
                normals.push([0.0, 0.0, 1.0]);
                uvs.push([p[0], p[1]]);
                indices.push(idx);
                idx += 1;
            }
            // Triangle 2 — shares tr and bl with triangle 1 (duplicate positions)
            for &p in &[tr, br, bl] {
                positions.push(p);
                normals.push([0.0, 0.0, 1.0]);
                uvs.push([p[0], p[1]]);
                indices.push(idx);
                idx += 1;
            }
        }
    }

    MeshBuffers::from_morph(MB {
        positions,
        normals,
        uvs,
        indices,
        has_suit: false,
    })
}

// ── original benchmarks ────────────────────────────────────────────────────

fn bench_compute_normals(c: &mut Criterion) {
    let mut mesh = make_large_mesh(19_158);
    c.bench_function("compute_normals_19k_verts", |b| {
        b.iter(|| {
            compute_normals(black_box(&mut mesh));
        })
    });
}

fn bench_compute_tangents(c: &mut Criterion) {
    let mut mesh = make_large_mesh(19_158);
    compute_normals(&mut mesh); // need normals first
    c.bench_function("compute_tangents_19k_verts", |b| {
        b.iter(|| {
            compute_tangents(black_box(&mut mesh));
        })
    });
}

// ── new benchmarks ─────────────────────────────────────────────────────────

/// LOD generation — reduce a 1 000-vertex grid to various ratios.
fn bench_lod_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("lod_generation_1k_verts");

    let mesh = make_large_mesh(1_000);

    for (name, level) in [
        ("lod_half", LodLevel::HALF),
        ("lod_quarter", LodLevel::QUARTER),
        ("lod_eighth", LodLevel::EIGHTH),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| black_box(generate_lod(black_box(&mesh), black_box(level))))
        });
    }

    group.finish();
}

/// Weld vertices — merge a mesh whose vertices are all duplicated.
fn bench_weld_vertices(c: &mut Criterion) {
    c.bench_function("weld_by_position_500_unique", |b| {
        b.iter_batched(
            || make_mesh_with_duplicates(500),
            |mesh| black_box(weld_by_position(black_box(&mesh), 1e-4)),
            BatchSize::SmallInput,
        )
    });
}

/// Curvature computation on a smooth 500-vertex grid mesh.
fn bench_compute_curvature(c: &mut Criterion) {
    let mesh = make_large_mesh(500);
    c.bench_function("compute_curvature_500_verts", |b| {
        b.iter(|| black_box(compute_curvature(black_box(&mesh))))
    });
}

/// Mesh decimation — use Quadric-based decimation to reduce a 19k mesh by half.
fn bench_mesh_decimate(c: &mut Criterion) {
    let mut group = c.benchmark_group("mesh_decimate_19k_verts");

    let mesh = make_large_mesh(19_158);

    for (name, ratio) in [
        ("decimate_75pct", 0.75f32),
        ("decimate_50pct", 0.50f32),
        ("decimate_25pct", 0.25f32),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| black_box(decimate_ratio(black_box(&mesh), black_box(ratio))))
        });
    }

    group.finish();
}

/// LOD generation on the full 19k-vertex body mesh.
fn bench_lod_generation_19k(c: &mut Criterion) {
    let mesh = make_large_mesh(19_158);
    c.bench_function("lod_generation_19k_half", |b| {
        b.iter(|| black_box(generate_lod(black_box(&mesh), LodLevel::HALF)))
    });
}

/// Weld followed by normals recompute — a common post-import pipeline step.
fn bench_weld_then_normals(c: &mut Criterion) {
    c.bench_function("weld_then_normals_500_unique", |b| {
        b.iter_batched(
            || make_mesh_with_duplicates(500),
            |mesh| {
                let (mut welded, _stats) = weld_by_position(&mesh, 1e-4);
                compute_normals(&mut welded);
                black_box(welded)
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_compute_normals,
    bench_compute_tangents,
    bench_lod_generation,
    bench_weld_vertices,
    bench_compute_curvature,
    bench_mesh_decimate,
    bench_lod_generation_19k,
    bench_weld_then_normals,
);
criterion_main!(benches);
