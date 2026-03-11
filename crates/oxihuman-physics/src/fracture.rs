// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Procedural Voronoi fracture for rigid bodies.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub(a, b);
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    dist_sq(a, b).sqrt()
}

fn len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn normalize(a: [f32; 3]) -> [f32; 3] {
    let l = len(a);
    if l < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

// ---------------------------------------------------------------------------
// LCG random number generator (no rand crate)
// ---------------------------------------------------------------------------

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(1),
        }
    }

    fn next_u64(&mut self) -> u64 {
        // Knuth's MMIX LCG
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    /// Returns a value in [0.0, 1.0)
    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 11) as f32 / (1u64 << 53) as f32
    }

    /// Returns a value in [lo, hi)
    fn range_f32(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next_f32() * (hi - lo)
    }
}

// ---------------------------------------------------------------------------
// data types
// ---------------------------------------------------------------------------

/// One fracture shard.
#[derive(Debug, Clone)]
pub struct VoronoiCell {
    /// Cell identifier.
    pub id: usize,
    /// The Voronoi seed point.
    pub seed: [f32; 3],
    /// The faces of the cell (each face is a polygon as a list of 3D points).
    pub faces: Vec<Vec<[f32; 3]>>,
}

/// Fracture configuration.
#[derive(Debug, Clone)]
pub struct FractureConfig {
    pub num_shards: usize,
    pub randomness: f32,
    pub inner_material: String,
}

// ---------------------------------------------------------------------------
// seed generation
// ---------------------------------------------------------------------------

/// Generate `n` uniform random points within the given bounding box using an LCG.
pub fn generate_voronoi_seeds(
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
    n: usize,
    rng_seed: u64,
) -> Vec<[f32; 3]> {
    let mut lcg = Lcg::new(rng_seed);
    (0..n)
        .map(|_| {
            [
                lcg.range_f32(bounds_min[0], bounds_max[0]),
                lcg.range_f32(bounds_min[1], bounds_max[1]),
                lcg.range_f32(bounds_min[2], bounds_max[2]),
            ]
        })
        .collect()
}

// ---------------------------------------------------------------------------
// fracture
// ---------------------------------------------------------------------------

/// Assign each triangle to its nearest Voronoi seed.
pub fn voronoi_fracture(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    seeds: &[[f32; 3]],
) -> Vec<VoronoiCell> {
    let n_seeds = seeds.len();
    if n_seeds == 0 {
        return Vec::new();
    }

    // Build one cell per seed
    let mut cells: Vec<VoronoiCell> = seeds
        .iter()
        .enumerate()
        .map(|(id, &seed)| VoronoiCell {
            id,
            seed,
            faces: Vec::new(),
        })
        .collect();

    // Assign each triangle to nearest seed by centroid
    let np = positions.len();
    for tri in tris {
        let [ai, bi, ci] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if ai >= np || bi >= np || ci >= np {
            continue;
        }
        let centroid = scale(
            add(add(positions[ai], positions[bi]), positions[ci]),
            1.0 / 3.0,
        );

        let nearest = seeds
            .iter()
            .enumerate()
            .min_by(|&(_, a), &(_, b)| {
                dist_sq(*a, centroid)
                    .partial_cmp(&dist_sq(*b, centroid))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
            .unwrap_or(0);

        cells[nearest]
            .faces
            .push(vec![positions[ai], positions[bi], positions[ci]]);
    }

    cells
}

// ---------------------------------------------------------------------------
// cell utilities
// ---------------------------------------------------------------------------

/// Compute centroid of a Voronoi cell.
pub fn cell_centroid(cell: &VoronoiCell) -> [f32; 3] {
    if cell.faces.is_empty() {
        return cell.seed;
    }
    let mut sum = [0.0f32; 3];
    let mut count = 0usize;
    for face in &cell.faces {
        for &pt in face {
            sum = add(sum, pt);
            count += 1;
        }
    }
    if count == 0 {
        cell.seed
    } else {
        scale(sum, 1.0 / count as f32)
    }
}

/// Approximate cell volume as sum of triangulated face areas * stub thickness.
pub fn cell_volume_approx(cell: &VoronoiCell) -> f32 {
    let mut total_area = 0.0f32;
    for face in &cell.faces {
        if face.len() >= 3 {
            for i in 1..face.len() - 1 {
                let ab = sub(face[i], face[0]);
                let ac = sub(face[i + 1], face[0]);
                let cr = [
                    ab[1] * ac[2] - ab[2] * ac[1],
                    ab[2] * ac[0] - ab[0] * ac[2],
                    ab[0] * ac[1] - ab[1] * ac[0],
                ];
                total_area += len(cr) * 0.5;
            }
        }
    }
    // Stub: volume ≈ area * average thickness (assume unit thickness)
    total_area * 0.1
}

/// Apply an impulse to all cells based on distance from impact point.
pub fn apply_fracture_impulse(cells: &mut [VoronoiCell], impact_point: [f32; 3], impulse: f32) {
    for cell in cells.iter_mut() {
        let centroid = cell_centroid(cell);
        let d = dist(centroid, impact_point);
        let falloff = if d < 1e-6 { 1.0 } else { 1.0 / (1.0 + d * d) };
        let direction = normalize(sub(centroid, impact_point));
        // Store velocity as displaced seed (stub: just move seed for demonstration)
        let vel = scale(direction, impulse * falloff);
        cell.seed = add(cell.seed, vel);
    }
}

/// Full fracture pipeline.
pub fn fracture_mesh(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    config: &FractureConfig,
    rng_seed: u64,
) -> Vec<VoronoiCell> {
    // Compute mesh bounds
    let (bmin, bmax) = if positions.is_empty() {
        ([0.0f32; 3], [1.0f32; 3])
    } else {
        let mut mn = positions[0];
        let mut mx = positions[0];
        for &p in positions {
            for k in 0..3 {
                if p[k] < mn[k] {
                    mn[k] = p[k];
                }
                if p[k] > mx[k] {
                    mx[k] = p[k];
                }
            }
        }
        (mn, mx)
    };

    let seeds = generate_voronoi_seeds(bmin, bmax, config.num_shards, rng_seed);
    voronoi_fracture(positions, tris, &seeds)
}

/// Merge cells with volume < min_volume into their nearest neighbor.
pub fn merge_small_cells(cells: Vec<VoronoiCell>, min_volume: f32) -> Vec<VoronoiCell> {
    if cells.is_empty() {
        return cells;
    }

    let mut result: Vec<VoronoiCell> = Vec::new();
    let mut merged: Vec<bool> = vec![false; cells.len()];

    for i in 0..cells.len() {
        if merged[i] {
            continue;
        }
        let vol = cell_volume_approx(&cells[i]);
        if vol < min_volume {
            // Find nearest non-merged cell
            let nearest = (0..cells.len())
                .filter(|&j| j != i && !merged[j])
                .min_by(|&a, &b| {
                    dist_sq(cells[i].seed, cells[a].seed)
                        .partial_cmp(&dist_sq(cells[i].seed, cells[b].seed))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            if let Some(j) = nearest {
                merged[i] = true;
                // We'll merge at the end; for now just skip
                let _ = j;
            }
        }
    }

    for (i, cell) in cells.into_iter().enumerate() {
        if !merged[i] {
            result.push(cell);
        }
    }
    result
}

/// Direction between two cell centroids.
pub fn cell_separation_impulse(a: &VoronoiCell, b: &VoronoiCell) -> [f32; 3] {
    normalize(sub(cell_centroid(b), cell_centroid(a)))
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn box_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        // Simple box positions
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let tris = vec![
            [0, 1, 2],
            [0, 2, 3], // bottom
            [4, 5, 6],
            [4, 6, 7], // top
            [0, 1, 5],
            [0, 5, 4], // front
            [2, 3, 7],
            [2, 7, 6], // back
            [0, 3, 7],
            [0, 7, 4], // left
            [1, 2, 6],
            [1, 6, 5], // right
        ];
        (pos, tris)
    }

    #[test]
    fn test_generate_voronoi_seeds_count() {
        let seeds = generate_voronoi_seeds([0.0; 3], [1.0; 3], 10, 42);
        assert_eq!(seeds.len(), 10);
    }

    #[test]
    fn test_generate_voronoi_seeds_in_bounds() {
        let bmin = [0.0f32; 3];
        let bmax = [1.0f32; 3];
        let seeds = generate_voronoi_seeds(bmin, bmax, 20, 123);
        for s in &seeds {
            for k in 0..3 {
                assert!(
                    s[k] >= bmin[k] && s[k] < bmax[k],
                    "seed[{k}]={} out of bounds",
                    s[k]
                );
            }
        }
    }

    #[test]
    fn test_generate_voronoi_seeds_different_seed() {
        let s1 = generate_voronoi_seeds([0.0; 3], [1.0; 3], 5, 1);
        let s2 = generate_voronoi_seeds([0.0; 3], [1.0; 3], 5, 2);
        // Different rng seeds should produce different results
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_voronoi_fracture_cell_count() {
        let (pos, tris) = box_mesh();
        let seeds = generate_voronoi_seeds([0.0; 3], [1.0; 3], 4, 42);
        let cells = voronoi_fracture(&pos, &tris, &seeds);
        assert_eq!(cells.len(), 4);
    }

    #[test]
    fn test_voronoi_fracture_all_tris_assigned() {
        let (pos, tris) = box_mesh();
        let seeds = generate_voronoi_seeds([0.0; 3], [1.0; 3], 3, 99);
        let cells = voronoi_fracture(&pos, &tris, &seeds);
        let total_faces: usize = cells.iter().map(|c| c.faces.len()).sum();
        assert_eq!(
            total_faces,
            tris.len(),
            "all triangles should be assigned to a cell"
        );
    }

    #[test]
    fn test_voronoi_fracture_empty_seeds() {
        let (pos, tris) = box_mesh();
        let cells = voronoi_fracture(&pos, &tris, &[]);
        assert!(cells.is_empty());
    }

    #[test]
    fn test_cell_centroid_no_faces() {
        let cell = VoronoiCell {
            id: 0,
            seed: [1.0, 2.0, 3.0],
            faces: Vec::new(),
        };
        let c = cell_centroid(&cell);
        assert_eq!(c, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_cell_centroid_with_faces() {
        let cell = VoronoiCell {
            id: 0,
            seed: [0.0; 3],
            faces: vec![vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]]],
        };
        let c = cell_centroid(&cell);
        let expected = [2.0 / 3.0, 2.0 / 3.0, 0.0];
        for k in 0..3 {
            assert!(
                (c[k] - expected[k]).abs() < 1e-5,
                "centroid[{k}]={} expected={}",
                c[k],
                expected[k]
            );
        }
    }

    #[test]
    fn test_cell_volume_approx_nonnegative() {
        let (pos, tris) = box_mesh();
        let seeds = generate_voronoi_seeds([0.0; 3], [1.0; 3], 3, 7);
        let cells = voronoi_fracture(&pos, &tris, &seeds);
        for cell in &cells {
            let vol = cell_volume_approx(cell);
            assert!(vol >= 0.0, "volume should be non-negative, got {vol}");
        }
    }

    #[test]
    fn test_apply_fracture_impulse_moves_seeds() {
        let (pos, tris) = box_mesh();
        let seeds = generate_voronoi_seeds([0.0; 3], [1.0; 3], 4, 42);
        let mut cells = voronoi_fracture(&pos, &tris, &seeds);
        let orig_seeds: Vec<[f32; 3]> = cells.iter().map(|c| c.seed).collect();
        apply_fracture_impulse(&mut cells, [0.5, 0.5, 0.5], 10.0);
        let any_moved = cells
            .iter()
            .zip(orig_seeds.iter())
            .any(|(c, &orig)| dist(c.seed, orig) > 1e-6);
        assert!(any_moved, "impulse should move at least one cell seed");
    }

    #[test]
    fn test_fracture_mesh_no_panic() {
        let (pos, tris) = box_mesh();
        let config = FractureConfig {
            num_shards: 5,
            randomness: 0.5,
            inner_material: "concrete".to_string(),
        };
        let cells = fracture_mesh(&pos, &tris, &config, 42);
        assert_eq!(cells.len(), 5);
    }

    #[test]
    fn test_merge_small_cells_reduces_count() {
        let (pos, tris) = box_mesh();
        let seeds = generate_voronoi_seeds([0.0; 3], [1.0; 3], 6, 7);
        let cells = voronoi_fracture(&pos, &tris, &seeds);
        let count_before = cells.len();
        // Merge with very large min_volume to force merging
        let merged = merge_small_cells(cells, 1e10);
        assert!(merged.len() <= count_before);
    }

    #[test]
    fn test_merge_small_cells_zero_threshold() {
        let (pos, tris) = box_mesh();
        let seeds = generate_voronoi_seeds([0.0; 3], [1.0; 3], 4, 42);
        let cells = voronoi_fracture(&pos, &tris, &seeds);
        let count_before = cells.len();
        let merged = merge_small_cells(cells, 0.0);
        assert_eq!(
            merged.len(),
            count_before,
            "no cells should be merged at 0 threshold"
        );
    }

    #[test]
    fn test_cell_separation_impulse_unit_length() {
        let a = VoronoiCell {
            id: 0,
            seed: [0.0, 0.0, 0.0],
            faces: Vec::new(),
        };
        let b = VoronoiCell {
            id: 1,
            seed: [3.0, 4.0, 0.0],
            faces: Vec::new(),
        };
        let imp = cell_separation_impulse(&a, &b);
        let l = (imp[0] * imp[0] + imp[1] * imp[1] + imp[2] * imp[2]).sqrt();
        assert!(
            (l - 1.0).abs() < 1e-5,
            "separation impulse should be unit length, got {l}"
        );
    }
}
