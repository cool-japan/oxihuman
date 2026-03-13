// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Criterion benchmarks for oxihuman-viewer.
//!
//! Eight benchmarks covering the main viewer subsystems:
//! LOD selection, LOD chain building, morph updater dirty tracking,
//! morph clean path, orbit camera update, render stats snapshot,
//! LOD transition hysteresis, and frame timer ring buffer.

use criterion::{criterion_group, criterion_main, Criterion};
use oxihuman_viewer::{
    build_lod_chain, default_lod_configs, CameraState, LodLevelV2, LodManagerV2, LodTransition,
    Mesh, MorphSlider, MorphTargetDeltas, MorphUpdater, RenderStatsV3,
};
use std::collections::HashMap;
use std::hint::black_box;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a synthetic UV-sphere mesh with the given subdivision count.
///
/// Produces n×n quads = 2·n² triangles.
fn make_sphere_mesh(subdivisions: usize) -> Mesh {
    let n = subdivisions.max(2);
    let mut positions = Vec::with_capacity((n + 1) * (n + 1) * 3);
    let mut indices = Vec::with_capacity(n * n * 6);

    for i in 0..=n {
        for j in 0..=n {
            let theta = std::f32::consts::PI * (i as f32 / n as f32);
            let phi = 2.0 * std::f32::consts::PI * (j as f32 / n as f32);
            positions.push(theta.sin() * phi.cos());
            positions.push(theta.cos());
            positions.push(theta.sin() * phi.sin());
        }
    }
    for i in 0..(n as u32) {
        for j in 0..(n as u32) {
            let row = n as u32 + 1;
            let a = i * row + j;
            let b = a + 1;
            let c = (i + 1) * row + j;
            let d = c + 1;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    Mesh { positions, indices }
}

/// Build a [`MorphUpdater`] with `slider_count` sliders, `dirty_fraction` of
/// which are marked dirty.
///
/// Returns the updater together with a matching morph target map.
fn make_morph_updater(
    vertex_count: usize,
    slider_count: usize,
    dirty_fraction: f32,
) -> (MorphUpdater, HashMap<String, MorphTargetDeltas>) {
    let mut updater = MorphUpdater::new();
    let mut targets: HashMap<String, MorphTargetDeltas> = HashMap::new();

    let dirty_count = (slider_count as f32 * dirty_fraction).ceil() as usize;

    for idx in 0..slider_count {
        let name = format!("morph_{idx:04}");
        updater.add_slider(MorphSlider::new(&name, 0.0, 0.0, 1.0));

        if idx < dirty_count {
            // Mark the slider dirty with a non-zero value.
            updater.set_slider(&name, 0.5 + (idx as f32) * 0.01);

            // One delta every 10 vertices keeps the target realistic but bounded.
            let deltas: MorphTargetDeltas = (0..vertex_count)
                .step_by(10)
                .map(|v| (v as u32, 0.1, -0.05, 0.02))
                .collect();
            targets.insert(name, deltas);
        }
    }

    (updater, targets)
}

// ── Benchmark 1: LOD level selection — ~1 K-face mesh ────────────────────────

/// Bench `LodManagerV2::select_lod` with a ~1 K-face mesh at varying distances.
///
/// Exercises the hot-path LOD selection that runs every frame per visible object.
fn bench_lod_select_1k(c: &mut Criterion) {
    // 22×22×2 = 968 triangles ≈ 1 K.
    let mesh = make_sphere_mesh(22);
    let configs = default_lod_configs(mesh.face_count()).to_vec();
    let lod_meshes = build_lod_chain(&mesh);
    let mgr = LodManagerV2::new(lod_meshes, configs);

    let fov_rad = 60_f32.to_radians();
    let screen_height = 720.0_f32;

    c.bench_function("lod_select_1k", |b| {
        b.iter(|| {
            // Sweep across all LOD boundary distances.
            for dist in [1.0_f32, 5.0, 15.0, 35.0, 70.0, 200.0] {
                black_box(mgr.select_lod(
                    black_box(dist),
                    black_box(screen_height),
                    black_box(fov_rad),
                ));
            }
        });
    });
}

// ── Benchmark 2: LOD level selection — ~100 K-face config ────────────────────

/// Same `select_lod` hot path but with configs scaled for a 100 K-face mesh.
///
/// Verifies O(1) selection is independent of mesh size by using a large
/// `base_face_count` for thresholds while keeping actual mesh data small.
fn bench_lod_select_100k(c: &mut Criterion) {
    // Use distance thresholds for a 100 K-face asset, but a tiny placeholder
    // chain (build_lod_chain cost not included in benchmark).
    let configs = default_lod_configs(100_000).to_vec();
    let placeholder = make_sphere_mesh(4);
    let lod_meshes = build_lod_chain(&placeholder);
    let mgr = LodManagerV2::new(lod_meshes, configs);

    let fov_rad = 60_f32.to_radians();
    let screen_height = 1080.0_f32;

    c.bench_function("lod_select_100k", |b| {
        b.iter(|| {
            for dist in [0.5_f32, 10.0, 25.0, 50.0, 100.0, 500.0] {
                black_box(mgr.select_lod(
                    black_box(dist),
                    black_box(screen_height),
                    black_box(fov_rad),
                ));
            }
        });
    });
}

// ── Benchmark 3: LOD chain build from a ~10 K-face mesh ──────────────────────

/// Measures the full `build_lod_chain` QEM decimation cost for ~10 K faces.
///
/// Representative of the one-time asset-load cost for a face or hand sub-mesh.
fn bench_lod_chain_build(c: &mut Criterion) {
    // 70×70×2 = 9 800 ≈ 10 K triangles.
    let mesh = make_sphere_mesh(70);

    c.bench_function("lod_chain_build_10k", |b| {
        b.iter(|| {
            black_box(build_lod_chain(black_box(&mesh)));
        });
    });
}

// ── Benchmark 4: MorphUpdater — 80 % dirty, 12 K vertices ────────────────────

/// Hot path: 80 % of 40 sliders are dirty; `apply_dirty_to_mesh` accumulates
/// weighted deltas for a 12 K-vertex position buffer.
fn bench_morph_updater_dirty_track(c: &mut Criterion) {
    const VERTEX_COUNT: usize = 12_000;
    const SLIDER_COUNT: usize = 40;

    let (updater, targets) = make_morph_updater(VERTEX_COUNT, SLIDER_COUNT, 0.80);
    let mut positions = vec![0.0_f32; VERTEX_COUNT * 3];

    c.bench_function("morph_updater_dirty_80pct", |b| {
        b.iter(|| {
            // Reset before each application so the measurement is stable.
            positions.fill(0.0);
            updater.apply_dirty_to_mesh(black_box(&mut positions), black_box(&targets));
            black_box(&positions);
        });
    });
}

// ── Benchmark 5: MorphUpdater — fully clean (no-op path) ─────────────────────

/// When no sliders are dirty, `apply_dirty_to_mesh` must return immediately.
///
/// Measures the cheapest possible frame: nothing changed, no GPU upload needed.
fn bench_morph_updater_full_clean(c: &mut Criterion) {
    const VERTEX_COUNT: usize = 12_000;
    const SLIDER_COUNT: usize = 40;

    let (updater, targets) = make_morph_updater(VERTEX_COUNT, SLIDER_COUNT, 0.0);
    let mut positions = vec![0.0_f32; VERTEX_COUNT * 3];

    c.bench_function("morph_updater_clean", |b| {
        b.iter(|| {
            updater.apply_dirty_to_mesh(black_box(&mut positions), black_box(&targets));
            black_box(&positions);
        });
    });
}

// ── Benchmark 6: Orbit camera — 10 K sequential orbit() calls ────────────────

/// Measures `CameraState::orbit` throughput for 10 K sequential calls.
///
/// Simulates a high-frequency mouse-drag stream (e.g., 120 fps with batched
/// input events or a touchpad gesture).
fn bench_camera_orbit_update(c: &mut Criterion) {
    c.bench_function("camera_orbit_10k", |b| {
        b.iter(|| {
            let mut cam = CameraState::default();
            for i in 0_u32..10_000 {
                let yaw = (i % 360) as f32 * 0.01_f32;
                let pitch = ((i % 90) as f32 - 45.0) * 0.005_f32;
                cam.orbit(black_box(yaw), black_box(pitch));
            }
            black_box(cam);
        });
    });
}

// ── Benchmark 7: RenderStatsV3 — end_frame + snapshot ────────────────────────

/// Measures per-frame accounting: recording 100 draw calls and reading back
/// the snapshot, which is the bookkeeping overhead in the main render loop.
fn bench_render_stats_snapshot(c: &mut Criterion) {
    let mut stats = RenderStatsV3::new();

    // Pre-warm the ring buffer so fps() is meaningful from iteration 1.
    for _ in 0..60 {
        stats.begin_frame();
        stats.record_draw(50_000);
        stats.end_frame();
    }

    c.bench_function("render_stats_snapshot", |b| {
        b.iter(|| {
            stats.begin_frame();
            for _ in 0_u32..100 {
                stats.record_draw(black_box(5_000));
            }
            stats.end_frame();
            black_box(stats.snapshot());
        });
    });
}

// ── Benchmark 8: LOD transition hysteresis — 100 request/update cycles ────────

/// Measures `LodTransition::request_lod` + `update` over 100 transitions.
///
/// Simulates an object repeatedly crossing LOD boundaries, exercising the
/// hysteresis blend logic that prevents LOD flickering at threshold distances.
fn bench_lod_transition_hysteresis(c: &mut Criterion) {
    c.bench_function("lod_transition_hysteresis_100", |b| {
        b.iter(|| {
            let mut transition = LodTransition::new(LodLevelV2::Full);
            // Cycle through all levels, including reversals, to cover every
            // branch in the hysteresis state machine.
            let levels = [
                LodLevelV2::Full,
                LodLevelV2::High,
                LodLevelV2::Medium,
                LodLevelV2::Low,
                LodLevelV2::Minimal,
                LodLevelV2::Low,
                LodLevelV2::Medium,
                LodLevelV2::High,
            ];
            // Sub-frame dt: transitions won't snap to completion immediately.
            let dt = 1.0_f32 / 120.0;

            for cycle in 0_u32..100 {
                let target_level = levels[(cycle as usize) % levels.len()];
                transition.request_lod(black_box(target_level));
                transition.update(black_box(dt));
            }
            black_box(&transition);
        });
    });
}

// ── Groups and main ───────────────────────────────────────────────────────────

criterion_group!(
    viewer_benches,
    bench_lod_select_1k,
    bench_lod_select_100k,
    bench_lod_chain_build,
    bench_morph_updater_dirty_track,
    bench_morph_updater_full_clean,
    bench_camera_orbit_update,
    bench_render_stats_snapshot,
    bench_lod_transition_hysteresis,
);
criterion_main!(viewer_benches);
