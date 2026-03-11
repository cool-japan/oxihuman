//! Voronoi partitioning on mesh surface via BFS from seed vertices.

use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub struct VoronoiCell {
    pub seed_vertex: usize,
    pub member_vertices: Vec<usize>,
    pub centroid: [f32; 3],
    pub area: f32,
}

#[allow(dead_code)]
pub struct VoronoiDiagram {
    pub cells: Vec<VoronoiCell>,
    /// Per-vertex cell assignment (index into cells).
    pub vertex_cell: Vec<usize>,
    pub vertex_count: usize,
}

#[allow(dead_code)]
pub struct VoronoiConfig {
    pub seed_count: usize,
    /// Number of centroidal Voronoi refinement iterations.
    pub iterations: u32,
    pub metric: VoronoiMetric,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum VoronoiMetric {
    Euclidean,
    Geodesic,
}

// ---------------------------------------------------------------------------
// Default config
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn default_voronoi_config(seed_count: usize) -> VoronoiConfig {
    VoronoiConfig {
        seed_count,
        iterations: 5,
        metric: VoronoiMetric::Euclidean,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn euclidean_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

// Simple LCG for deterministic pseudo-random numbers.
fn lcg_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

// ---------------------------------------------------------------------------
// Core functions
// ---------------------------------------------------------------------------

/// Generate `count` deterministic seed vertices using an LCG.
#[allow(dead_code)]
pub fn voronoi_random_seeds(vertex_count: usize, count: usize, seed: u64) -> Vec<usize> {
    if vertex_count == 0 || count == 0 {
        return Vec::new();
    }
    let mut state = seed ^ 0xDEAD_BEEF_CAFE_0001;
    let mut seeds: Vec<usize> = Vec::with_capacity(count);
    let mut attempts = 0usize;
    while seeds.len() < count && attempts < count * 20 + 1000 {
        attempts += 1;
        let val = lcg_next(&mut state);
        let idx = (val as usize) % vertex_count;
        if !seeds.contains(&idx) {
            seeds.push(idx);
        }
    }
    // If we couldn't fill uniquely (very small mesh), just wrap.
    while seeds.len() < count {
        let val = lcg_next(&mut state);
        let idx = (val as usize) % vertex_count;
        seeds.push(idx);
    }
    seeds
}

/// BFS flood-fill from seeds to assign every vertex to the nearest seed cell.
/// Distance is measured as hop-count (BFS), approximating geodesic distance.
#[allow(dead_code)]
pub fn voronoi_from_seeds(
    positions: &[[f32; 3]],
    adjacency: &[Vec<usize>],
    seeds: &[usize],
) -> VoronoiDiagram {
    let n = positions.len();
    let cell_count_n = seeds.len();

    // vertex_cell[v] = cell index (usize::MAX = unvisited)
    let mut vertex_cell_map = vec![usize::MAX; n];
    // BFS queue: (vertex, cell_index)
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    for (ci, &sv) in seeds.iter().enumerate() {
        if sv < n {
            vertex_cell_map[sv] = ci;
            queue.push_back((sv, ci));
        }
    }

    while let Some((v, ci)) = queue.pop_front() {
        if v < adjacency.len() {
            for &nb in &adjacency[v] {
                if nb < n && vertex_cell_map[nb] == usize::MAX {
                    vertex_cell_map[nb] = ci;
                    queue.push_back((nb, ci));
                }
            }
        }
    }

    // Assign any still-unvisited vertex to nearest seed by Euclidean distance.
    for v in 0..n {
        if vertex_cell_map[v] == usize::MAX {
            let pos = positions[v];
            let best = seeds
                .iter()
                .enumerate()
                .min_by(|(_, &sa), (_, &sb)| {
                    let da = euclidean_sq(pos, positions[sa.min(n - 1)]);
                    let db = euclidean_sq(pos, positions[sb.min(n - 1)]);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(ci, _)| ci)
                .unwrap_or(0);
            vertex_cell_map[v] = best;
        }
    }

    // Build cells.
    let mut member_lists: Vec<Vec<usize>> = vec![Vec::new(); cell_count_n];
    for (v, &ci) in vertex_cell_map.iter().enumerate() {
        if ci < cell_count_n {
            member_lists[ci].push(v);
        }
    }

    let cells: Vec<VoronoiCell> = seeds
        .iter()
        .enumerate()
        .map(|(ci, &sv)| {
            let members = member_lists[ci].clone();
            let centroid = compute_member_centroid(positions, &members);
            let area = members.len() as f32; // simplified area = vertex count
            VoronoiCell {
                seed_vertex: sv,
                member_vertices: members,
                centroid,
                area,
            }
        })
        .collect();

    VoronoiDiagram {
        cells,
        vertex_cell: vertex_cell_map,
        vertex_count: n,
    }
}

fn compute_member_centroid(positions: &[[f32; 3]], members: &[usize]) -> [f32; 3] {
    if members.is_empty() {
        return [0.0; 3];
    }
    let mut sum = [0.0f32; 3];
    for &v in members {
        if v < positions.len() {
            sum[0] += positions[v][0];
            sum[1] += positions[v][1];
            sum[2] += positions[v][2];
        }
    }
    let n = members.len() as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Build adjacency list from triangle index buffer.
#[allow(dead_code)]
fn build_adjacency_from_indices(vertex_count: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); vertex_count];
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let a = indices[t * 3] as usize;
        let b = indices[t * 3 + 1] as usize;
        let c = indices[t * 3 + 2] as usize;
        for &(u, v) in &[(a, b), (b, c), (a, c)] {
            if u < vertex_count && v < vertex_count {
                if !adj[u].contains(&v) {
                    adj[u].push(v);
                }
                if !adj[v].contains(&u) {
                    adj[v].push(u);
                }
            }
        }
    }
    adj
}

/// Full pipeline: build adjacency from indices, pick random seeds, BFS flood fill,
/// then run centroidal refinement for `cfg.iterations` steps.
#[allow(dead_code)]
pub fn compute_voronoi(
    positions: &[[f32; 3]],
    indices: &[u32],
    cfg: &VoronoiConfig,
    seed: u64,
) -> VoronoiDiagram {
    let n = positions.len();
    let count = cfg.seed_count.min(n).max(1);
    let seeds = voronoi_random_seeds(n, count, seed);
    let adj = build_adjacency_from_indices(n, indices);
    let mut diagram = voronoi_from_seeds(positions, &adj, &seeds);
    for _ in 0..cfg.iterations {
        centroidal_voronoi_step(positions, &mut diagram);
    }
    diagram
}

/// Move each seed to the centroid of its cell, then re-flood-fill.
/// This approximates Lloyd's algorithm.
#[allow(dead_code)]
pub fn centroidal_voronoi_step(positions: &[[f32; 3]], diagram: &mut VoronoiDiagram) {
    let n = positions.len();
    // Find new seed positions as nearest vertex to centroid.
    let new_seeds: Vec<usize> = diagram
        .cells
        .iter()
        .map(|cell| {
            let centroid = compute_member_centroid(positions, &cell.member_vertices);
            // Find vertex closest to centroid.
            let best = cell
                .member_vertices
                .iter()
                .copied()
                .filter(|&v| v < n)
                .min_by(|&va, &vb| {
                    let da = euclidean_sq(positions[va], centroid);
                    let db = euclidean_sq(positions[vb], centroid);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or(cell.seed_vertex);
            best
        })
        .collect();

    // Rebuild without real adjacency (use Euclidean assignment for simplicity).
    let cell_count_n = new_seeds.len();
    let mut vertex_cell_map = vec![0usize; n];
    for v in 0..n {
        let pos = positions[v];
        let best = new_seeds
            .iter()
            .enumerate()
            .min_by(|(_, &sa), (_, &sb)| {
                let da = euclidean_sq(pos, positions[sa.min(n - 1)]);
                let db = euclidean_sq(pos, positions[sb.min(n - 1)]);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(ci, _)| ci)
            .unwrap_or(0);
        vertex_cell_map[v] = best;
    }

    let mut member_lists: Vec<Vec<usize>> = vec![Vec::new(); cell_count_n];
    for (v, &ci) in vertex_cell_map.iter().enumerate() {
        if ci < cell_count_n {
            member_lists[ci].push(v);
        }
    }

    for (ci, cell) in diagram.cells.iter_mut().enumerate() {
        cell.seed_vertex = new_seeds[ci];
        cell.member_vertices = member_lists[ci].clone();
        cell.centroid = compute_member_centroid(positions, &cell.member_vertices);
        cell.area = cell.member_vertices.len() as f32;
    }
    diagram.vertex_cell = vertex_cell_map;
}

/// Compute the centroid of a single VoronoiCell.
#[allow(dead_code)]
pub fn voronoi_cell_centroid(positions: &[[f32; 3]], cell: &VoronoiCell) -> [f32; 3] {
    compute_member_centroid(positions, &cell.member_vertices)
}

/// Approximate cell area as number of member vertices.
#[allow(dead_code)]
pub fn voronoi_cell_area(positions: &[[f32; 3]], cell: &VoronoiCell) -> f32 {
    let _ = positions; // area = member count for mesh surfaces
    cell.member_vertices.len() as f32
}

/// Return edges (pairs of vertex indices) that cross cell boundaries.
#[allow(dead_code)]
pub fn voronoi_boundary_edges(diagram: &VoronoiDiagram, indices: &[u32]) -> Vec<[u32; 2]> {
    let tri_count = indices.len() / 3;
    let mut edges: Vec<[u32; 2]> = Vec::new();
    for t in 0..tri_count {
        let verts = [
            indices[t * 3] as usize,
            indices[t * 3 + 1] as usize,
            indices[t * 3 + 2] as usize,
        ];
        for &(i, j) in &[(0usize, 1usize), (1, 2), (0, 2)] {
            let va = verts[i];
            let vb = verts[j];
            if va < diagram.vertex_count
                && vb < diagram.vertex_count
                && diagram.vertex_cell[va] != diagram.vertex_cell[vb]
            {
                let e = [va as u32, vb as u32];
                if !edges.contains(&e) {
                    edges.push(e);
                }
            }
        }
    }
    edges
}

/// Number of cells in the diagram.
#[allow(dead_code)]
pub fn cell_count(diagram: &VoronoiDiagram) -> usize {
    diagram.cells.len()
}

/// Cell ID for a given vertex.
#[allow(dead_code)]
pub fn vertex_cell_id(diagram: &VoronoiDiagram, vertex: usize) -> usize {
    if vertex < diagram.vertex_cell.len() {
        diagram.vertex_cell[vertex]
    } else {
        0
    }
}

/// Cell with the most member vertices.
#[allow(dead_code)]
pub fn largest_cell(diagram: &VoronoiDiagram) -> Option<&VoronoiCell> {
    diagram.cells.iter().max_by_key(|c| c.member_vertices.len())
}

/// Cell with the fewest member vertices.
#[allow(dead_code)]
pub fn smallest_cell(diagram: &VoronoiDiagram) -> Option<&VoronoiCell> {
    diagram.cells.iter().min_by_key(|c| c.member_vertices.len())
}

/// Balance score: 1.0 if all cells have the same size, 0.0 if one cell has everything.
#[allow(dead_code)]
pub fn voronoi_balance_score(diagram: &VoronoiDiagram) -> f32 {
    if diagram.cells.is_empty() {
        return 1.0;
    }
    let sizes: Vec<f32> = diagram
        .cells
        .iter()
        .map(|c| c.member_vertices.len() as f32)
        .collect();
    let mean = sizes.iter().sum::<f32>() / sizes.len() as f32;
    if mean < 1e-10 {
        return 1.0;
    }
    let variance = sizes.iter().map(|&s| (s - mean).powi(2)).sum::<f32>() / sizes.len() as f32;
    let std_dev = variance.sqrt();
    // Coefficient of variation: 0 = perfectly balanced.
    let cv = std_dev / mean;
    (1.0 - cv.min(1.0)).max(0.0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
            [0.0, 2.0, 0.0],
            [1.0, 2.0, 0.0],
            [2.0, 2.0, 0.0],
        ]
    }

    fn simple_indices() -> Vec<u32> {
        // 3x3 grid split into triangles
        vec![
            0, 1, 3, 1, 4, 3, 1, 2, 4, 2, 5, 4, 3, 4, 6, 4, 7, 6, 4, 5, 7, 5, 8, 7,
        ]
    }

    fn simple_adjacency(n: usize, indices: &[u32]) -> Vec<Vec<usize>> {
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        let tri_count = indices.len() / 3;
        for t in 0..tri_count {
            let a = indices[t * 3] as usize;
            let b = indices[t * 3 + 1] as usize;
            let c = indices[t * 3 + 2] as usize;
            for &(u, v) in &[(a, b), (b, c), (a, c)] {
                if !adj[u].contains(&v) {
                    adj[u].push(v);
                }
                if !adj[v].contains(&u) {
                    adj[v].push(u);
                }
            }
        }
        adj
    }

    #[test]
    fn test_random_seeds_correct_count() {
        let seeds = voronoi_random_seeds(9, 3, 42);
        assert_eq!(seeds.len(), 3);
    }

    #[test]
    fn test_random_seeds_in_range() {
        let seeds = voronoi_random_seeds(9, 4, 123);
        for &s in &seeds {
            assert!(s < 9, "seed {s} out of range");
        }
    }

    #[test]
    fn test_random_seeds_deterministic() {
        let s1 = voronoi_random_seeds(9, 3, 77);
        let s2 = voronoi_random_seeds(9, 3, 77);
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_voronoi_from_seeds_covers_all_vertices() {
        let pos = simple_positions();
        let idx = simple_indices();
        let adj = simple_adjacency(pos.len(), &idx);
        let seeds = vec![0, 8];
        let diagram = voronoi_from_seeds(&pos, &adj, &seeds);
        // Every vertex should be assigned.
        assert_eq!(diagram.vertex_count, pos.len());
        for &ci in &diagram.vertex_cell {
            assert!(ci < 2, "bad cell index {ci}");
        }
    }

    #[test]
    fn test_cell_count_matches_seeds() {
        let pos = simple_positions();
        let idx = simple_indices();
        let adj = simple_adjacency(pos.len(), &idx);
        let seeds = vec![0, 4, 8];
        let diagram = voronoi_from_seeds(&pos, &adj, &seeds);
        assert_eq!(cell_count(&diagram), 3);
    }

    #[test]
    fn test_vertex_cell_id_valid() {
        let pos = simple_positions();
        let idx = simple_indices();
        let adj = simple_adjacency(pos.len(), &idx);
        let seeds = vec![0, 8];
        let diagram = voronoi_from_seeds(&pos, &adj, &seeds);
        let id = vertex_cell_id(&diagram, 0);
        assert!(id < 2);
        let id2 = vertex_cell_id(&diagram, 8);
        assert!(id2 < 2);
    }

    #[test]
    fn test_boundary_edges_non_empty_for_multi_cell() {
        let pos = simple_positions();
        let idx = simple_indices();
        let adj = simple_adjacency(pos.len(), &idx);
        let seeds = vec![0, 8];
        let diagram = voronoi_from_seeds(&pos, &adj, &seeds);
        let edges = voronoi_boundary_edges(&diagram, &idx);
        assert!(!edges.is_empty(), "expected boundary edges between 2 cells");
    }

    #[test]
    fn test_centroidal_step_moves_seeds() {
        let pos = simple_positions();
        let idx = simple_indices();
        let adj = simple_adjacency(pos.len(), &idx);
        let seeds = vec![0, 8];
        let mut diagram = voronoi_from_seeds(&pos, &adj, &seeds);
        let old_seeds: Vec<usize> = diagram.cells.iter().map(|c| c.seed_vertex).collect();
        centroidal_voronoi_step(&pos, &mut diagram);
        // After a step, at least it ran without panic and cells are consistent.
        assert_eq!(diagram.cells.len(), old_seeds.len());
        for cell in &diagram.cells {
            assert!(cell.seed_vertex < pos.len());
        }
    }

    #[test]
    fn test_voronoi_balance_score_single_cell() {
        let pos = simple_positions();
        let idx = simple_indices();
        let adj = simple_adjacency(pos.len(), &idx);
        let seeds = vec![4]; // single seed — all in one cell
        let diagram = voronoi_from_seeds(&pos, &adj, &seeds);
        let score = voronoi_balance_score(&diagram);
        // Single cell is perfectly "balanced" (no variance).
        assert!((score - 1.0).abs() < 1e-5, "score={score}");
    }

    #[test]
    fn test_voronoi_balance_score_two_cells() {
        let pos = simple_positions();
        let idx = simple_indices();
        let adj = simple_adjacency(pos.len(), &idx);
        let seeds = vec![0, 8];
        let diagram = voronoi_from_seeds(&pos, &adj, &seeds);
        let score = voronoi_balance_score(&diagram);
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn test_largest_smallest_cell() {
        let pos = simple_positions();
        let idx = simple_indices();
        let adj = simple_adjacency(pos.len(), &idx);
        let seeds = vec![0, 4, 8];
        let diagram = voronoi_from_seeds(&pos, &adj, &seeds);
        let lc = largest_cell(&diagram).unwrap();
        let sc = smallest_cell(&diagram).unwrap();
        assert!(lc.member_vertices.len() >= sc.member_vertices.len());
    }

    #[test]
    fn test_compute_voronoi_pipeline() {
        let pos = simple_positions();
        let idx = simple_indices();
        let cfg = default_voronoi_config(3);
        let diagram = compute_voronoi(&pos, &idx, &cfg, 99);
        assert_eq!(cell_count(&diagram), 3);
        assert_eq!(diagram.vertex_count, pos.len());
    }

    #[test]
    fn test_voronoi_cell_centroid() {
        let pos = vec![[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]];
        let cell = VoronoiCell {
            seed_vertex: 0,
            member_vertices: vec![0, 1, 2],
            centroid: [1.0, 0.666, 0.0],
            area: 3.0,
        };
        let c = voronoi_cell_centroid(&pos, &cell);
        assert!((c[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_voronoi_cell_area() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let cell = VoronoiCell {
            seed_vertex: 0,
            member_vertices: vec![0, 1, 2],
            centroid: [0.0; 3],
            area: 0.0,
        };
        assert_eq!(voronoi_cell_area(&pos, &cell), 3.0);
    }

    #[test]
    fn test_default_voronoi_config() {
        let cfg = default_voronoi_config(10);
        assert_eq!(cfg.seed_count, 10);
        assert_eq!(cfg.iterations, 5);
    }
}
