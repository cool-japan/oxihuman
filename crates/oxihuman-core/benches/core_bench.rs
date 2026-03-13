use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxihuman_core::{
    add_dep_node, bezier_cubic_sample, bloom_contains, bloom_insert, bloom_reset, cache_get,
    cache_insert, intern, kd3_build, kd3_nearest_id, new_bloom_filter, new_cache,
    new_dependency_graph, new_simple_octree, new_skip_list, new_string_pool, pool_size,
    radix_sort_u32, radix_sort_u64, resolve, resolve_dependencies, skip_find, skip_insert,
    skip_len, Dependency, DependencyNode, OctAabb, SpatialHash2D,
};
use std::hint::black_box;

// ---------------------------------------------------------------------------
// Deterministic pseudo-random helpers (no `rand` crate)
// ---------------------------------------------------------------------------

/// LCG-based pseudo-random u64 sequence seeded with `seed`.
fn lcg_sequence(seed: u64, count: usize) -> Vec<u64> {
    let mut v = Vec::with_capacity(count);
    let mut s = seed;
    for _ in 0..count {
        s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        v.push(s);
    }
    v
}

/// Produce `count` deterministic f32 values in [0, 1).
fn pseudo_rand_f32(seed: u64, count: usize) -> Vec<f32> {
    lcg_sequence(seed, count)
        .into_iter()
        .map(|x| (x >> 32) as f32 / u32::MAX as f32)
        .collect()
}

/// Produce `count` deterministic u32 values.
fn pseudo_rand_u32(seed: u64, count: usize) -> Vec<u32> {
    lcg_sequence(seed, count)
        .into_iter()
        .map(|x| (x >> 32) as u32)
        .collect()
}

// ---------------------------------------------------------------------------
// 1. Bloom filter — bulk insert 10 K elements
// ---------------------------------------------------------------------------
fn bench_bloom_insert_10k(c: &mut Criterion) {
    let items: Vec<Vec<u8>> = (0u32..10_000)
        .map(|i| format!("item_{i}").into_bytes())
        .collect();

    c.bench_function("bloom_insert_10k", |b| {
        b.iter(|| {
            let mut bf = new_bloom_filter(black_box(1 << 17), black_box(4));
            for item in &items {
                bloom_insert(&mut bf, item);
            }
            bf
        });
    });
}

// ---------------------------------------------------------------------------
// 2. Bloom filter — membership queries after 10 K inserts
// ---------------------------------------------------------------------------
fn bench_bloom_query_10k(c: &mut Criterion) {
    let mut bf = new_bloom_filter(1 << 17, 4);
    let items: Vec<Vec<u8>> = (0u32..10_000)
        .map(|i| format!("item_{i}").into_bytes())
        .collect();
    for item in &items {
        bloom_insert(&mut bf, item);
    }
    let queries: Vec<Vec<u8>> = (0u32..10_000)
        .map(|i| {
            if i % 2 == 0 {
                format!("item_{i}").into_bytes()
            } else {
                format!("miss_{i}").into_bytes()
            }
        })
        .collect();

    c.bench_function("bloom_query_10k", |b| {
        b.iter(|| {
            let mut hits = 0usize;
            for q in &queries {
                if bloom_contains(black_box(&bf), q) {
                    hits += 1;
                }
            }
            black_box(hits)
        });
    });
}

// ---------------------------------------------------------------------------
// 3. Bloom filter — reset (clear all bits)
// ---------------------------------------------------------------------------
fn bench_bloom_reset(c: &mut Criterion) {
    let items: Vec<Vec<u8>> = (0u32..10_000)
        .map(|i| format!("item_{i}").into_bytes())
        .collect();

    c.bench_function("bloom_reset", |b| {
        b.iter_batched(
            || {
                let mut bf = new_bloom_filter(1 << 17, 4);
                for item in &items {
                    bloom_insert(&mut bf, item);
                }
                bf
            },
            |mut bf| bloom_reset(black_box(&mut bf)),
            criterion::BatchSize::SmallInput,
        );
    });
}

// ---------------------------------------------------------------------------
// 4. KD-tree 3D — build from 1 K / 5 K / 10 K points
// ---------------------------------------------------------------------------
fn bench_kd_tree_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("kd_tree_3d_build");
    for &n in &[1_000usize, 5_000, 10_000] {
        let coords = pseudo_rand_f32(42, n * 3);
        let points: Vec<[f32; 3]> = (0..n)
            .map(|i| [coords[i * 3], coords[i * 3 + 1], coords[i * 3 + 2]])
            .collect();
        group.bench_with_input(BenchmarkId::from_parameter(n), &points, |b, pts| {
            b.iter(|| kd3_build(black_box(pts.as_slice())));
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// 5. KD-tree 3D — nearest-neighbour queries (1 K queries against 5 K points)
// ---------------------------------------------------------------------------
fn bench_kd_tree_nearest(c: &mut Criterion) {
    let n = 5_000usize;
    let coords = pseudo_rand_f32(99, n * 3);
    let points: Vec<[f32; 3]> = (0..n)
        .map(|i| [coords[i * 3], coords[i * 3 + 1], coords[i * 3 + 2]])
        .collect();
    let tree = kd3_build(points.as_slice());
    let query_coords = pseudo_rand_f32(7, 1_000 * 3);

    c.bench_function("kd_tree_nearest_1k_queries", |b| {
        b.iter(|| {
            let mut found = 0usize;
            for i in 0..1_000 {
                let q = [
                    query_coords[i * 3],
                    query_coords[i * 3 + 1],
                    query_coords[i * 3 + 2],
                ];
                if kd3_nearest_id(black_box(&tree), q).is_some() {
                    found += 1;
                }
            }
            black_box(found)
        });
    });
}

// ---------------------------------------------------------------------------
// 6. SimpleOctree3 — build from 1 K points
// ---------------------------------------------------------------------------
fn bench_octree_build_1k(c: &mut Criterion) {
    let coords = pseudo_rand_f32(13, 3_000);

    c.bench_function("octree_build_1k", |b| {
        b.iter(|| {
            let mut tree = new_simple_octree(black_box(1.0));
            for i in 0..1_000 {
                let p = [
                    coords[i * 3] * 2.0 - 1.0,
                    coords[i * 3 + 1] * 2.0 - 1.0,
                    coords[i * 3 + 2] * 2.0 - 1.0,
                ];
                tree.insert(p);
            }
            black_box(tree.len())
        });
    });
}

// ---------------------------------------------------------------------------
// 7. Octree — sphere range query
// ---------------------------------------------------------------------------
fn bench_octree_sphere_query(c: &mut Criterion) {
    let coords = pseudo_rand_f32(17, 3_000);
    let mut tree = new_simple_octree(1.0);
    for i in 0..1_000 {
        let p = [
            coords[i * 3] * 2.0 - 1.0,
            coords[i * 3 + 1] * 2.0 - 1.0,
            coords[i * 3 + 2] * 2.0 - 1.0,
        ];
        tree.insert(p);
    }

    c.bench_function("octree_sphere_query", |b| {
        b.iter(|| {
            let results = tree.query_sphere(black_box(&[0.0f32, 0.0, 0.0]), black_box(0.5));
            black_box(results.len())
        });
    });
}

// ---------------------------------------------------------------------------
// 8. Octree — AABB range query
// ---------------------------------------------------------------------------
fn bench_octree_aabb_query(c: &mut Criterion) {
    let coords = pseudo_rand_f32(19, 3_000);
    let mut tree = new_simple_octree(1.0);
    for i in 0..1_000 {
        let p = [
            coords[i * 3] * 2.0 - 1.0,
            coords[i * 3 + 1] * 2.0 - 1.0,
            coords[i * 3 + 2] * 2.0 - 1.0,
        ];
        tree.insert(p);
    }
    let query_box = OctAabb::new([-0.3, -0.3, -0.3], [0.3, 0.3, 0.3]);

    c.bench_function("octree_aabb_query", |b| {
        b.iter(|| {
            let results = tree.query_aabb(black_box(&query_box));
            black_box(results.len())
        });
    });
}

// ---------------------------------------------------------------------------
// 9. Radix sort u32 — 50 K values
// ---------------------------------------------------------------------------
fn bench_radix_sort_u32_50k(c: &mut Criterion) {
    let data = pseudo_rand_u32(55, 50_000);

    c.bench_function("radix_sort_u32_50k", |b| {
        b.iter_batched(
            || data.clone(),
            |mut v| {
                radix_sort_u32(black_box(&mut v));
                black_box(v)
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

// ---------------------------------------------------------------------------
// 10. Radix sort u64 — 50 K values
// ---------------------------------------------------------------------------
fn bench_radix_sort_u64_50k(c: &mut Criterion) {
    let data: Vec<u64> = lcg_sequence(77, 50_000);

    c.bench_function("radix_sort_u64_50k", |b| {
        b.iter_batched(
            || data.clone(),
            |mut v| {
                radix_sort_u64(black_box(&mut v));
                black_box(v)
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

// ---------------------------------------------------------------------------
// 11. Bezier cubic sampling — 200-point polyline from one cubic segment
// ---------------------------------------------------------------------------
fn bench_bezier_cubic_sample(c: &mut Criterion) {
    let p0 = [0.0_f32, 0.0];
    let p1 = [0.33_f32, 1.0];
    let p2 = [0.66_f32, -1.0];
    let p3 = [1.0_f32, 0.0];

    c.bench_function("bezier_cubic_sample_200pts", |b| {
        b.iter(|| {
            black_box(bezier_cubic_sample(
                black_box(p0),
                black_box(p1),
                black_box(p2),
                black_box(p3),
                black_box(200),
            ))
        });
    });
}

// ---------------------------------------------------------------------------
// 12. Skip list — 2 K insertions followed by find
// ---------------------------------------------------------------------------
fn bench_skip_list_insert_and_find(c: &mut Criterion) {
    let keys: Vec<i64> = (0i64..2_000).map(|i| (i * 7 + 3) % 10_000).collect();
    let val = "v";

    c.bench_function("skip_list_insert_find_2k", |b| {
        b.iter(|| {
            let mut sl = new_skip_list();
            for &k in &keys {
                skip_insert(&mut sl, k, val);
            }
            let len = skip_len(&sl);
            // Convert to owned to avoid returning a reference into a local
            let found: Option<String> = skip_find(&sl, 4_999).map(str::to_owned);
            black_box((len, found))
        });
    });
}

// ---------------------------------------------------------------------------
// 13. String pool — intern 1 K unique strings
// ---------------------------------------------------------------------------
fn bench_string_pool_intern_1k(c: &mut Criterion) {
    let strings: Vec<String> = (0u32..1_000).map(|i| format!("token_{i:04}")).collect();

    c.bench_function("string_pool_intern_1k", |b| {
        b.iter(|| {
            let mut pool = new_string_pool();
            for s in &strings {
                let _ = intern(&mut pool, s.as_str());
            }
            black_box(pool_size(&pool))
        });
    });
}

// ---------------------------------------------------------------------------
// 14. String pool — resolve 1 K handles
// ---------------------------------------------------------------------------
fn bench_string_pool_resolve_1k(c: &mut Criterion) {
    let strings: Vec<String> = (0u32..1_000).map(|i| format!("token_{i:04}")).collect();
    let mut pool = new_string_pool();
    let handles: Vec<_> = strings
        .iter()
        .map(|s| intern(&mut pool, s.as_str()))
        .collect();

    c.bench_function("string_pool_resolve_1k", |b| {
        b.iter(|| {
            let mut found = 0usize;
            for &h in &handles {
                if resolve(black_box(&pool), h).is_some() {
                    found += 1;
                }
            }
            black_box(found)
        });
    });
}

// ---------------------------------------------------------------------------
// 15. Asset cache — insert / evict cycle (256 KiB budget, 1 KiB items)
// ---------------------------------------------------------------------------
fn bench_asset_cache_insert_evict(c: &mut Criterion) {
    let max_bytes = 256 * 1024;
    let payload: Vec<u8> = (0u8..=255).cycle().take(1024).collect();

    c.bench_function("asset_cache_insert_evict_512items", |b| {
        b.iter(|| {
            let mut cache = new_cache(black_box(max_bytes));
            for i in 0u32..512 {
                let key = format!("asset_{i:04}");
                cache_insert(&mut cache, &key, payload.clone());
            }
            let _ = cache_get(&mut cache, "asset_0128");
            black_box(cache)
        });
    });
}

// ---------------------------------------------------------------------------
// 16. Dependency resolver — resolve a 50-node DAG
// ---------------------------------------------------------------------------
fn bench_dependency_resolve_50_nodes(c: &mut Criterion) {
    c.bench_function("dep_resolve_50_nodes", |b| {
        b.iter(|| {
            let mut graph = new_dependency_graph();
            // Layer 0: 10 root nodes (no deps)
            for i in 0..10u32 {
                add_dep_node(
                    &mut graph,
                    DependencyNode {
                        id: format!("root_{i}"),
                        version: "1.0.0".into(),
                        deps: vec![],
                    },
                );
            }
            // Layer 1: 20 mid nodes each depending on 2 roots
            for i in 0..20u32 {
                add_dep_node(
                    &mut graph,
                    DependencyNode {
                        id: format!("mid_{i}"),
                        version: "1.0.0".into(),
                        deps: vec![
                            Dependency {
                                name: format!("root_{}", i % 10),
                                required: true,
                                version_req: None,
                            },
                            Dependency {
                                name: format!("root_{}", (i + 1) % 10),
                                required: false,
                                version_req: None,
                            },
                        ],
                    },
                );
            }
            // Layer 2: 20 leaf nodes each depending on 2 mid nodes
            for i in 0..20u32 {
                add_dep_node(
                    &mut graph,
                    DependencyNode {
                        id: format!("leaf_{i}"),
                        version: "1.0.0".into(),
                        deps: vec![
                            Dependency {
                                name: format!("mid_{}", i % 20),
                                required: true,
                                version_req: None,
                            },
                            Dependency {
                                name: format!("mid_{}", (i + 3) % 20),
                                required: true,
                                version_req: None,
                            },
                        ],
                    },
                );
            }
            let result = resolve_dependencies(black_box(&graph));
            black_box(result)
        });
    });
}

// ---------------------------------------------------------------------------
// 17. SpatialHash2D — 2 K point insertions + radius query
// ---------------------------------------------------------------------------
fn bench_spatial_hash_2d_insert_query(c: &mut Criterion) {
    let coords = pseudo_rand_f32(31, 2_000 * 2);

    c.bench_function("spatial_hash_2d_insert_query_2k", |b| {
        b.iter(|| {
            let mut grid = SpatialHash2D::new(black_box(5.0));
            for i in 0..2_000 {
                let p = [
                    coords[i * 2] * 200.0 - 100.0,
                    coords[i * 2 + 1] * 200.0 - 100.0,
                ];
                grid.insert(p);
            }
            let hits = grid.query_radius(black_box(0.0), black_box(0.0), black_box(15.0));
            black_box(hits.len())
        });
    });
}

// ---------------------------------------------------------------------------
// Criterion groups
// ---------------------------------------------------------------------------

criterion_group!(
    benches_bloom,
    bench_bloom_insert_10k,
    bench_bloom_query_10k,
    bench_bloom_reset,
);

criterion_group!(
    benches_spatial,
    bench_kd_tree_build,
    bench_kd_tree_nearest,
    bench_octree_build_1k,
    bench_octree_sphere_query,
    bench_octree_aabb_query,
    bench_spatial_hash_2d_insert_query,
);

criterion_group!(
    benches_algorithms,
    bench_radix_sort_u32_50k,
    bench_radix_sort_u64_50k,
    bench_bezier_cubic_sample,
    bench_skip_list_insert_and_find,
);

criterion_group!(
    benches_data_structures,
    bench_string_pool_intern_1k,
    bench_string_pool_resolve_1k,
    bench_asset_cache_insert_evict,
    bench_dependency_resolve_50_nodes,
);

criterion_main!(
    benches_bloom,
    benches_spatial,
    benches_algorithms,
    benches_data_structures,
);
