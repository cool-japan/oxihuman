// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Streaming mesh decode benchmarks and LitePack compression benchmarks
//! for the `oxihuman-wasm` crate.
//!
//! Run with:
//! ```text
//! cargo bench -p oxihuman-wasm
//! ```

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use oxihuman_mesh::mesh::MeshBuffers;
use oxihuman_morph::engine::MeshBuffers as MB;
use oxihuman_wasm::buffer::parse_mesh_bytes_header;
use oxihuman_wasm::compressed_target::LitePack;
use oxihuman_wasm::engine::WasmEngine;

// ---------------------------------------------------------------------------
// Minimal synthetic OBJ that parses correctly at any vertex count
// ---------------------------------------------------------------------------

/// Build a minimal OBJ text with `n_verts` vertices arranged in a fan,
/// producing `n_verts - 2` triangles.
fn make_obj_text(n_verts: usize) -> String {
    let mut s = String::with_capacity(n_verts * 60);
    s.push_str("# synthetic OBJ\n");
    for i in 0..n_verts {
        let x = i as f32 * 0.001;
        s.push_str(&format!("v {x:.6} 0.000000 0.000000\n"));
    }
    // UVs — one per vertex
    for _ in 0..n_verts {
        s.push_str("vt 0.000000 0.000000\n");
    }
    // Normals — one shared up-normal
    s.push_str("vn 0.000000 1.000000 0.000000\n");
    // Fan triangles: 0, i, i+1 for i in 1..n_verts-1
    for i in 1..n_verts.saturating_sub(1) {
        // OBJ indices are 1-based
        let a = 1u32;
        let b = (i + 1) as u32;
        let c = (i + 2) as u32;
        s.push_str(&format!("f {a}/1/1 {b}/1/1 {c}/1/1\n"));
    }
    s
}

/// Build a [`WasmEngine`] from a synthetic OBJ of `n_verts` vertices.
/// Panics in test/bench context on failure — acceptable for bench harnesses.
#[allow(dead_code)]
fn make_engine(n_verts: usize) -> WasmEngine {
    let obj = make_obj_text(n_verts);
    WasmEngine::new_from_obj_bytes(obj.as_bytes())
        .expect("bench: failed to build WasmEngine from synthetic OBJ")
}

/// Build a synthetic [`MeshBuffers`] with `n_verts` vertices.
fn make_mesh_buffers(n_verts: usize) -> MeshBuffers {
    let positions: Vec<[f32; 3]> = (0..n_verts).map(|i| [i as f32 * 0.001, 0.0, 0.0]).collect();
    let normals = vec![[0.0f32, 1.0, 0.0]; n_verts];
    let uvs = vec![[0.0f32, 0.0]; n_verts];
    let mut indices: Vec<u32> = Vec::new();
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

// ---------------------------------------------------------------------------
// Helper: encode mesh to the raw bytes format used by WasmEngine
// ---------------------------------------------------------------------------

/// Encode a [`MeshBuffers`] to the wire format used by `build_mesh_bytes()`.
///
/// Layout (matches `BUFFER_FORMAT_VERSION`):
/// `[version:u32][n_verts:u32][n_idx:u32][positions][normals][uvs][indices]`
fn encode_mesh_to_bytes(mesh: &MeshBuffers) -> Vec<u8> {
    use oxihuman_wasm::BUFFER_FORMAT_VERSION;

    let n_verts = mesh.positions.len() as u32;
    let n_idx = mesh.indices.len() as u32;
    let capacity = 12 + (n_verts as usize) * (3 + 3 + 2) * 4 + (n_idx as usize) * 4;
    let mut out = Vec::with_capacity(capacity);

    out.extend_from_slice(&BUFFER_FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&n_verts.to_le_bytes());
    out.extend_from_slice(&n_idx.to_le_bytes());

    for p in &mesh.positions {
        for &c in p {
            out.extend_from_slice(&c.to_le_bytes());
        }
    }
    for n in &mesh.normals {
        for &c in n {
            out.extend_from_slice(&c.to_le_bytes());
        }
    }
    for uv in &mesh.uvs {
        for &c in uv {
            out.extend_from_slice(&c.to_le_bytes());
        }
    }
    for &i in &mesh.indices {
        out.extend_from_slice(&i.to_le_bytes());
    }

    out
}

// ---------------------------------------------------------------------------
// Helper: decode mesh bytes → (n_verts, n_idx) via header parse
// ---------------------------------------------------------------------------

/// Decode the header and then iterate all position floats from the byte buffer.
/// This simulates the work a real decoder would perform.
fn decode_mesh_bytes(bytes: &[u8]) -> Option<(u32, u32)> {
    let (version, n_verts, n_idx) = parse_mesh_bytes_header(bytes)?;
    // Validate version to ensure the decode path isn't elided.
    if version == 0 {
        return None;
    }
    // Walk positions (3 f32 each) to exercise memory bandwidth.
    let pos_offset = 12usize;
    let pos_bytes = n_verts as usize * 3 * 4;
    if bytes.len() < pos_offset + pos_bytes {
        return None;
    }
    let mut _sum = 0.0f32;
    for chunk in bytes[pos_offset..pos_offset + pos_bytes].chunks_exact(4) {
        let arr: [u8; 4] = chunk.try_into().ok()?;
        _sum += f32::from_le_bytes(arr);
    }
    Some((n_verts, n_idx))
}

// ---------------------------------------------------------------------------
// Helper: build deltas for LitePack benchmarks
// ---------------------------------------------------------------------------

/// Build a sparse delta array of `n` entries (5 % are non-zero).
fn make_sparse_deltas(n: usize) -> Vec<[f64; 3]> {
    (0..n)
        .map(|i| {
            if i % 20 == 0 {
                let t = i as f64 * 0.001;
                [t, -t * 0.5, t * 0.25]
            } else {
                [0.0, 0.0, 0.0]
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Benchmark 1: bench_mesh_bytes_encode
// ---------------------------------------------------------------------------

fn bench_mesh_bytes_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("mesh_bytes_encode");

    for n_verts in [1_000usize, 10_000, 100_000] {
        let mesh = make_mesh_buffers(n_verts);
        group.bench_with_input(BenchmarkId::new("verts", n_verts), &mesh, |b, mesh| {
            b.iter(|| black_box(encode_mesh_to_bytes(black_box(mesh))))
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark 2: bench_mesh_bytes_decode
// ---------------------------------------------------------------------------

fn bench_mesh_bytes_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("mesh_bytes_decode");

    for n_verts in [1_000usize, 10_000, 100_000] {
        let mesh = make_mesh_buffers(n_verts);
        let bytes = encode_mesh_to_bytes(&mesh);
        group.bench_with_input(BenchmarkId::new("verts", n_verts), &bytes, |b, bytes| {
            b.iter(|| black_box(decode_mesh_bytes(black_box(bytes))))
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark 3: bench_compressed_target_pack
// ---------------------------------------------------------------------------

fn bench_compressed_target_pack(c: &mut Criterion) {
    let mut group = c.benchmark_group("litepack_compress");

    for n_deltas in [100usize, 1_000, 10_000] {
        let deltas = make_sparse_deltas(n_deltas);
        group.bench_with_input(
            BenchmarkId::new("deltas", n_deltas),
            &deltas,
            |b, deltas| {
                b.iter_batched(
                    || {
                        // Fresh empty pack each iteration — compress is measured.
                        LitePack::new()
                    },
                    |mut pack| {
                        pack.add_target("target_0".to_string(), black_box(deltas))
                            .expect("bench: litepack add_target failed");
                        black_box(pack)
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark 4: bench_compressed_target_unpack
// ---------------------------------------------------------------------------

fn bench_compressed_target_unpack(c: &mut Criterion) {
    let mut group = c.benchmark_group("litepack_decompress");

    for n_deltas in [100usize, 1_000, 10_000] {
        // Build the pack once outside the timed loop.
        let deltas = make_sparse_deltas(n_deltas);
        let mut pack = LitePack::new();
        pack.add_target("target_0".to_string(), &deltas)
            .expect("bench: litepack add_target failed");
        let packed_bytes = pack.serialize().expect("bench: litepack serialize failed");

        group.bench_with_input(
            BenchmarkId::new("deltas", n_deltas),
            &packed_bytes,
            |b, packed_bytes| {
                b.iter(|| {
                    let pack = LitePack::deserialize(black_box(packed_bytes))
                        .expect("bench: litepack deserialize failed");
                    let out = pack
                        .get_target("target_0")
                        .expect("bench: litepack get_target failed");
                    black_box(out)
                })
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark 5: bench_engine_build_mesh — cold vs warm
// ---------------------------------------------------------------------------

fn bench_engine_build_mesh(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_build_mesh");

    // Cold: cache is always cleared before the measured call.
    // We use a single representative mesh size (10K vertices).
    let obj_bytes = make_obj_text(10_000).into_bytes();

    group.bench_function("cold_10k", |b| {
        b.iter_batched(
            || {
                // Setup: build a fresh engine with no prior build.
                let mut engine = WasmEngine::new_from_obj_bytes(&obj_bytes)
                    .expect("bench: failed to build engine");
                engine.reset_incremental_cache();
                engine
            },
            |mut engine| black_box(engine.build_mesh_bytes()),
            BatchSize::PerIteration,
        )
    });

    // Warm: build once to populate the incremental cache, then rebuild.
    group.bench_function("warm_10k", |b| {
        b.iter_batched(
            || {
                let mut engine = WasmEngine::new_from_obj_bytes(&obj_bytes)
                    .expect("bench: failed to build engine");
                // Prime the cache with a full build.
                let _ = engine.build_mesh_bytes();
                engine
            },
            |mut engine| {
                // Rebuild with same params — should use incremental path.
                black_box(engine.build_mesh_bytes())
            },
            BatchSize::PerIteration,
        )
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark 6: bench_streaming_decode_chunks
// ---------------------------------------------------------------------------

/// Simulate streaming decode: treat `full_bytes` as a contiguous blob and
/// process it in chunks of `chunk_size`, accumulating into an output buffer.
///
/// This models the JavaScript-side pattern where a 1 MB mesh arrives in 64 KB
/// fetch response chunks before being handed off to the decoder.
fn streaming_decode_in_chunks(full_bytes: &[u8], chunk_size: usize) -> Vec<u8> {
    let mut assembled: Vec<u8> = Vec::with_capacity(full_bytes.len());
    for chunk in full_bytes.chunks(chunk_size) {
        // Simulated network chunk arrival — copy into the assembly buffer.
        assembled.extend_from_slice(chunk);
    }
    // Once complete, run the header decode to simulate initial validation.
    let _ = decode_mesh_bytes(&assembled);
    assembled
}

fn bench_streaming_decode_chunks(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_decode_chunks");

    // Build a mesh large enough to be around 1 MB of encoded bytes.
    // 100K verts x (3+3+2) floats x 4 bytes = ~3.2 MB; use 30K for ~1 MB.
    const TARGET_VERTS: usize = 30_000;
    let mesh = make_mesh_buffers(TARGET_VERTS);
    let full_bytes = encode_mesh_to_bytes(&mesh);

    let approx_mb = full_bytes.len() as f64 / (1024.0 * 1024.0);
    // The benchmark runs regardless; log size for informational purposes.
    let _ = approx_mb; // suppress unused warning

    // 64 KB chunks (simulates browser ReadableStream default chunk size).
    group.bench_function("chunk_64k", |b| {
        b.iter(|| {
            black_box(streaming_decode_in_chunks(
                black_box(&full_bytes),
                64 * 1024,
            ))
        })
    });

    // All-at-once (baseline).
    group.bench_function("all_at_once", |b| {
        b.iter(|| {
            black_box(streaming_decode_in_chunks(
                black_box(&full_bytes),
                full_bytes.len().max(1),
            ))
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion wiring
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_mesh_bytes_encode,
    bench_mesh_bytes_decode,
    bench_compressed_target_pack,
    bench_compressed_target_unpack,
    bench_engine_build_mesh,
    bench_streaming_decode_chunks,
);
criterion_main!(benches);
