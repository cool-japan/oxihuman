// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Physics simulation throughput benchmarks.
//!
//! Covers: cloth PBD, hair XPBD, FEM co-rotational, SDF generation,
//! self-collision spatial hash, garment fitting, proxy generation, and
//! XPBD distance constraint solving.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

use oxihuman_mesh::mesh::MeshBuffers;
use oxihuman_physics::{
    add_xpbd_particle,
    // FEM
    compute_deformation_gradient,
    // Garment
    fit_garment_to_proxies,
    // Proxy generation
    generate_proxies,
    // XPBD
    new_xpbd_world,
    polar_decompose,
    // SDF
    sdf_gen::{SdfConfig, SdfGrid},
    // Self-collision
    self_collision::SelfCollisionDetector,
    strain_energy_density,
    xpbd_add_distance,
    xpbd_step,
    BodyProxies,
    CapsuleProxy,
    // Cloth (from cloth.rs, re-exported via pub mod cloth)
    ClothSim,
    GarmentFitConfig,
    GarmentVertex,
    // Hair (from hair.rs, re-exported via pub mod hair)
    HairConfig,
    HairStrand,
    HairSystem,
    SphereProxy,
};

// ── build helpers ─────────────────────────────────────────────────────────────

/// Build a square grid cloth with `side × side` particles using `ClothSim::from_mesh`.
fn make_cloth_sim(side: usize) -> ClothSim {
    let spacing = 1.0_f32 / side as f32;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(side * side);
    let mut indices: Vec<u32> = Vec::new();

    for row in 0..side {
        for col in 0..side {
            positions.push([col as f32 * spacing, 1.0, row as f32 * spacing]);
        }
    }

    for row in 0..(side - 1) {
        for col in 0..(side - 1) {
            let a = (row * side + col) as u32;
            let b = a + 1;
            let c = a + side as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }

    let sim = ClothSim::from_mesh(&positions, &indices, 0.9);
    // Pin the top row of particles
    sim.pin_by_y(0.95)
}

/// Build a hair system with `n_strands` strands of `segments` segments each.
fn make_hair_system(n_strands: usize, segments: usize) -> HairSystem {
    let cfg = HairConfig {
        gravity: [0.0, -9.81, 0.0],
        damping: 0.98,
        stiffness: 0.1,
        constraint_iters: 4,
    };
    let mut sys = HairSystem::new(cfg);
    for i in 0..n_strands {
        let root = [i as f32 * 0.05, 1.0, 0.0];
        let strand = HairStrand::new(root, [0.0, -1.0, 0.0], 0.3, segments);
        sys.add_strand(strand);
    }
    sys
}

/// Build a unit-cube mesh tiled to approximately `n_faces` triangles.
#[allow(clippy::type_complexity)]
fn unit_cube_mesh_tiled(n_faces: usize) -> (Vec<[f64; 3]>, Vec<[usize; 3]>, Vec<[f64; 3]>) {
    let base_verts: Vec<[f64; 3]> = vec![
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
    ];
    let base_tris: Vec<[usize; 3]> = vec![
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [2, 3, 7],
        [2, 7, 6],
        [0, 4, 7],
        [0, 7, 3],
        [1, 2, 6],
        [1, 6, 5],
    ];

    let mut verts = base_verts.clone();
    let mut tris: Vec<[usize; 3]> = base_tris.clone();
    let mut tile = 1usize;

    while tris.len() < n_faces {
        let v_off = verts.len();
        for v in &base_verts {
            verts.push([v[0] + tile as f64, v[1], v[2]]);
        }
        for t in &base_tris {
            if tris.len() >= n_faces {
                break;
            }
            tris.push([t[0] + v_off, t[1] + v_off, t[2] + v_off]);
        }
        tile += 1;
    }
    tris.truncate(n_faces);

    let mut norms = vec![[0.0f64; 3]; verts.len()];
    for tri in &tris {
        let a = verts[tri[0]];
        let b = verts[tri[1]];
        let c = verts[tri[2]];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        for &vi in tri {
            norms[vi][0] += n[0];
            norms[vi][1] += n[1];
            norms[vi][2] += n[2];
        }
    }
    for nm in &mut norms {
        let len = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
        if len > 1e-12 {
            nm[0] /= len;
            nm[1] /= len;
            nm[2] /= len;
        }
    }
    (verts, tris, norms)
}

/// Build a minimal `MeshBuffers` with `n_verts` vertices arranged in a grid.
fn make_mesh_buffers(n_verts: usize) -> MeshBuffers {
    let side = (n_verts as f32).sqrt().ceil() as usize;
    let spacing = 1.0_f32 / side.max(1) as f32;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n_verts);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_verts);
    let mut tangents: Vec<[f32; 4]> = Vec::with_capacity(n_verts);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n_verts);
    let mut indices: Vec<u32> = Vec::new();

    for i in 0..n_verts {
        let row = i / side;
        let col = i % side;
        let x = col as f32 * spacing - 0.5;
        let y = (i as f32 / n_verts as f32) * 1.8;
        let z = row as f32 * spacing - 0.5;
        positions.push([x, y, z]);
        normals.push([0.0, 1.0, 0.0]);
        tangents.push([1.0, 0.0, 0.0, 1.0]);
        uvs.push([col as f32 / side as f32, row as f32 / side as f32]);
    }
    for row in 0..side.saturating_sub(1) {
        for col in 0..side.saturating_sub(1) {
            let a = (row * side + col) as u32;
            let b = a + 1;
            let c = a + side as u32;
            let d = c + 1;
            if (d as usize) < n_verts {
                indices.extend_from_slice(&[a, b, c, b, d, c]);
            }
        }
    }
    MeshBuffers {
        positions,
        normals,
        tangents,
        uvs,
        indices,
        colors: None,
        has_suit: false,
    }
}

// ── bench 1: cloth PBD throughput (≈1000 particles, 100 steps) ───────────────

fn bench_cloth_throughput(c: &mut Criterion) {
    // 32×32 = 1024 particles
    c.bench_function("cloth_pbd_1000p_100steps", |b| {
        b.iter(|| {
            let mut sim = make_cloth_sim(32);
            let dt = black_box(0.016_f32);
            for _ in 0..100 {
                sim.step(dt, 2);
            }
            black_box(sim.particles.len())
        });
    });
}

// ── bench 2: hair dynamics (32 strands × 20 segments, 100 steps) ─────────────

fn bench_hair_dynamics(c: &mut Criterion) {
    c.bench_function("hair_xpbd_32strands_20seg_100steps", |b| {
        b.iter(|| {
            let mut sys = make_hair_system(32, 20);
            let dt = black_box(0.016_f32);
            for _ in 0..100 {
                sys.step(dt);
            }
            black_box(sys.strands.len())
        });
    });
}

// ── bench 3: soft body FEM co-rotational (500-tet mesh, 10 steps) ────────────

fn bench_fem_corotational(c: &mut Criterion) {
    let n_tets = 500usize;
    type TetEntry = ([[f32; 3]; 4], [[f32; 3]; 4]);
    let tet_data: Vec<TetEntry> = (0..n_tets)
        .map(|i| {
            let off = i as f32 * 0.1;
            let rest = [
                [off, 0.0, 0.0],
                [off + 1.0, 0.0, 0.0],
                [off + 0.5, 1.0, 0.0],
                [off + 0.5, 0.3, 1.0],
            ];
            let def = [
                [off, 0.0, 0.0],
                [off + 1.02, 0.0, 0.0],
                [off + 0.50, 1.01, 0.0],
                [off + 0.50, 0.30, 0.99],
            ];
            (rest, def)
        })
        .collect();

    c.bench_function("fem_corotational_500tet_10steps", |b| {
        b.iter(|| {
            let mut total = 0.0_f32;
            for _step in 0..10 {
                for (rest, def) in &tet_data {
                    let f = compute_deformation_gradient(*rest, *def);
                    let (_r, _s) = polar_decompose(&f, 5);
                    total += strain_energy_density(&f, 1e4, 3e3);
                }
            }
            black_box(total)
        });
    });
}

// ── bench 4: SDF generation at 32³, 64³, 128³ ────────────────────────────────

fn bench_sdf_generation(c: &mut Criterion) {
    let (verts, tris, norms) = unit_cube_mesh_tiled(1000);
    let mut group = c.benchmark_group("sdf_generation");

    for &res in &[32usize, 64, 128] {
        let cfg = SdfConfig {
            resolution: [res, res, res],
            padding: 0.2,
        };
        group.bench_with_input(BenchmarkId::new("res", res), &res, |b, _| {
            b.iter(|| {
                let grid = SdfGrid::from_mesh(
                    black_box(&verts),
                    black_box(&tris),
                    black_box(&norms),
                    &cfg,
                );
                black_box(grid.ok().map(|g| g.data().len()))
            });
        });
    }
    group.finish();
}

// ── bench 5: self-collision spatial hash (10k vertices) ───────────────────────

fn bench_self_collision(c: &mut Criterion) {
    let n_verts = 10_000usize;
    let verts: Vec<[f64; 3]> = (0..n_verts)
        .map(|i| {
            let t = i as f64 / n_verts as f64 * std::f64::consts::TAU;
            let u = (i as f64 / n_verts as f64 * std::f64::consts::PI).sin();
            [
                t.cos() * u,
                (i as f64 / n_verts as f64) * 2.0 - 1.0,
                t.sin() * u,
            ]
        })
        .collect();

    c.bench_function("self_collision_spatial_hash_10k", |b| {
        b.iter(|| {
            if let Ok(mut detector) = SelfCollisionDetector::new(0.005, 0.15) {
                detector.populate_hash(black_box(&verts));
                black_box(detector.thickness());
            }
        });
    });
}

// ── bench 6: garment fitting pipeline ────────────────────────────────────────

fn bench_garment_fitting(c: &mut Criterion) {
    let n_verts = 500usize;
    let garment: Vec<GarmentVertex> = (0..n_verts)
        .map(|i| {
            let angle = i as f32 / n_verts as f32 * std::f32::consts::TAU;
            let y = (i as f32 / n_verts as f32) * 1.5 + 0.3;
            let r = 0.22_f32;
            GarmentVertex {
                position: [angle.cos() * r, y, angle.sin() * r],
                rest_position: [angle.cos() * r, y, angle.sin() * r],
                is_seam: i % 50 == 0,
                layer: 0,
            }
        })
        .collect();

    let mut proxies = BodyProxies::new();
    proxies
        .spheres
        .push(SphereProxy::new([0.0, 1.6, 0.0], 0.12, "head"));
    proxies.capsules.push(CapsuleProxy::new(
        [0.0, 0.8, 0.0],
        [0.0, 1.4, 0.0],
        0.18,
        "torso",
    ));

    let cfg = GarmentFitConfig {
        iterations: 10,
        ..Default::default()
    };

    c.bench_function("garment_fit_pipeline", |b| {
        b.iter(|| {
            let result = fit_garment_to_proxies(black_box(&garment), black_box(&proxies), &cfg);
            black_box(result.max_penetration)
        });
    });
}

// ── bench 7: proxy generation (100, 1000, 10000 vertices) ────────────────────

fn bench_proxy_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("proxy_generation");

    for &n in &[100usize, 1000, 10000] {
        let mesh = make_mesh_buffers(n);
        group.bench_with_input(BenchmarkId::new("verts", n), &n, |b, _| {
            b.iter(|| {
                let result = generate_proxies(black_box(&mesh));
                black_box(result.map(|p| p.capsules.len() + p.spheres.len()))
            });
        });
    }
    group.finish();
}

// ── bench 8: XPBD distance constraints (1000 constraints, 10 iterations) ─────

fn bench_xpbd_constraints(c: &mut Criterion) {
    c.bench_function("xpbd_1000_distance_constraints_10iters", |b| {
        b.iter(|| {
            let mut world = new_xpbd_world();

            // Chain: particle 0 is pinned (inv_mass=0), the rest are free
            add_xpbd_particle(&mut world, [0.0, 0.0, 0.0], 0.0);
            for i in 1..=1000usize {
                add_xpbd_particle(&mut world, [i as f32 * 0.01, 0.0, 0.0], 1.0);
            }
            for i in 0..1000usize {
                xpbd_add_distance(&mut world, i, i + 1, 1e-4);
            }

            let dt = black_box(0.016_f32);
            for _ in 0..10 {
                xpbd_step(&mut world, dt, 1);
            }
            black_box(world.particles.len())
        });
    });
}

// ── criterion wiring ──────────────────────────────────────────────────────────

criterion_group!(
    physics_benches,
    bench_cloth_throughput,
    bench_hair_dynamics,
    bench_fem_corotational,
    bench_sdf_generation,
    bench_self_collision,
    bench_garment_fitting,
    bench_proxy_generation,
    bench_xpbd_constraints,
);
criterion_main!(physics_benches);
