// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use oxihuman_export::glb::export_glb;
use oxihuman_export::mesh_quantize::{quantize_mesh, write_quantized_bin};
use oxihuman_export::morph_delta_bin::{
    write_morph_delta_bin, MorphDeltaBin, MorphDeltaEntry, MorphDeltaTarget,
};
use oxihuman_export::zip_pack::{zip_bytes, ZipEntry};
use oxihuman_mesh::mesh::MeshBuffers;
use oxihuman_morph::engine::MeshBuffers as MB;

// ── mesh factory ─────────────────────────────────────────────────────────────

fn make_suited_mesh(n_verts: usize) -> MeshBuffers {
    let positions: Vec<[f32; 3]> = (0..n_verts).map(|i| [i as f32 * 0.001, 0.0, 0.0]).collect();
    let normals = vec![[0.0f32, 1.0, 0.0]; n_verts];
    let uvs = vec![[0.0f32, 0.0]; n_verts];
    let mut indices = Vec::new();
    for i in 1..n_verts.saturating_sub(1) as u32 {
        indices.extend_from_slice(&[0u32, i, i + 1]);
    }
    MeshBuffers::from_morph(MB {
        positions,
        normals,
        uvs,
        indices,
        has_suit: true,
    })
}

/// Build a synthetic `MorphDeltaBin` with `target_count` targets,
/// each having `deltas_per_target` non-zero entries across `vertex_count` verts.
fn make_morph_delta_bin(
    target_count: usize,
    deltas_per_target: usize,
    vertex_count: u32,
) -> MorphDeltaBin {
    let targets = (0..target_count)
        .map(|t| {
            let deltas = (0..deltas_per_target)
                .map(|i| MorphDeltaEntry {
                    vertex_index: (i as u32 * 37 + t as u32 * 13) % vertex_count,
                    dx: 0.01 * (t as f32 + 1.0),
                    dy: 0.02,
                    dz: -0.01,
                })
                .collect();
            MorphDeltaTarget {
                name: format!("target_{t:03}"),
                deltas,
            }
        })
        .collect();

    MorphDeltaBin {
        vertex_count,
        targets,
    }
}

// ── original benchmark ────────────────────────────────────────────────────────

fn bench_export_glb(c: &mut Criterion) {
    let mesh = make_suited_mesh(19_158);
    let path = std::path::PathBuf::from("/tmp/bench_oxihuman.glb");
    c.bench_function("export_glb_19k_verts", |b| {
        b.iter(|| {
            export_glb(black_box(&mesh), black_box(&path)).unwrap();
        })
    });
    std::fs::remove_file(&path).ok();
}

// ── new benchmarks ────────────────────────────────────────────────────────────

/// Quantize a 19k-vertex mesh and write the binary to /tmp.
fn bench_write_quantized_bin(c: &mut Criterion) {
    let mesh = make_suited_mesh(19_158);
    let path = std::path::Path::new("/tmp/bench_oxihuman_quant.oxq");

    c.bench_function("write_quantized_bin_19k_verts", |b| {
        b.iter(|| {
            let q = quantize_mesh(black_box(&mesh));
            write_quantized_bin(black_box(&q), black_box(path)).unwrap();
        })
    });

    std::fs::remove_file(path).ok();
}

/// Just the quantise step — no file I/O — to isolate CPU cost.
fn bench_quantize_only(c: &mut Criterion) {
    let mesh = make_suited_mesh(19_158);
    c.bench_function("quantize_only_19k_verts", |b| {
        b.iter(|| black_box(quantize_mesh(black_box(&mesh))))
    });
}

/// Build 3 `ZipEntry` items (base mesh bytes + 2 target bytes) and
/// produce the in-memory ZIP with `zip_bytes`.
fn bench_zip_bytes(c: &mut Criterion) {
    // Pre-build byte payloads in setup (not timed).
    let mesh = make_suited_mesh(19_158);
    let q = quantize_mesh(&mesh);

    // Encode quantised positions as raw little-endian u16 bytes.
    let base_bytes: Vec<u8> = q
        .positions
        .iter()
        .flat_map(|p| p.iter().flat_map(|v| v.to_le_bytes()))
        .collect();

    // Normals as raw i8 bytes.
    let target_bytes_a: Vec<u8> = q
        .normals
        .iter()
        .flat_map(|n| n.iter().map(|&v| v as u8))
        .collect();

    // UVs as raw u16 bytes.
    let target_bytes_b: Vec<u8> = q
        .uvs
        .iter()
        .flat_map(|uv| uv.iter().flat_map(|v| v.to_le_bytes()))
        .collect();

    c.bench_function("zip_bytes_3_entries", |b| {
        b.iter_batched(
            || {
                vec![
                    ZipEntry {
                        filename: "base_mesh.oxq".to_string(),
                        data: base_bytes.clone(),
                    },
                    ZipEntry {
                        filename: "target_a.bin".to_string(),
                        data: target_bytes_a.clone(),
                    },
                    ZipEntry {
                        filename: "target_b.bin".to_string(),
                        data: target_bytes_b.clone(),
                    },
                ]
            },
            |entries| black_box(zip_bytes(black_box(&entries))),
            BatchSize::SmallInput,
        )
    });
}

/// Write a `MorphDeltaBin` with 10 synthetic targets (500 deltas each) to /tmp.
fn bench_write_morph_delta_bin(c: &mut Criterion) {
    let bin = make_morph_delta_bin(10, 500, 19_158);
    let path = std::path::Path::new("/tmp/bench_oxihuman_morph.oxmd");

    c.bench_function("write_morph_delta_bin_10x500", |b| {
        b.iter(|| {
            write_morph_delta_bin(black_box(&bin), black_box(path)).unwrap();
        })
    });

    std::fs::remove_file(path).ok();
}

/// Quantize-only at smaller mesh sizes to understand scaling.
fn bench_quantize_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantize_scaling");

    for n in [1_000usize, 5_000, 10_000, 19_158] {
        let mesh = make_suited_mesh(n);
        group.bench_function(format!("{n}v"), |b| {
            b.iter(|| black_box(quantize_mesh(black_box(&mesh))))
        });
    }

    group.finish();
}

/// In-memory morph delta bin write across varying target counts.
fn bench_morph_delta_bin_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("morph_delta_bin_scaling");

    for target_count in [1usize, 5, 10, 20] {
        let bin = make_morph_delta_bin(target_count, 500, 19_158);
        let path_str = format!("/tmp/bench_oxihuman_morph_{target_count}.oxmd");
        let path = std::path::PathBuf::from(&path_str);

        group.bench_function(format!("{target_count}_targets"), |b| {
            b.iter(|| {
                write_morph_delta_bin(black_box(&bin), black_box(&path)).unwrap();
            })
        });

        std::fs::remove_file(&path).ok();
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_export_glb,
    bench_write_quantized_bin,
    bench_quantize_only,
    bench_zip_bytes,
    bench_write_morph_delta_bin,
    bench_quantize_scaling,
    bench_morph_delta_bin_scaling,
);
criterion_main!(benches);
