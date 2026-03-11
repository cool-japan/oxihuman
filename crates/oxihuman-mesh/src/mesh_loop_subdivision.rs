//! Standalone Loop subdivision (smooth + flat modes) for triangle meshes.
//!
//! Implements Charles Loop's scheme: each triangle is split into four
//! sub-triangles and vertex positions are blended with their one-ring
//! neighbours.  Flat mode skips the blend step, producing a uniformly
//! subdivided mesh without smoothing.

#![allow(dead_code)]

use std::collections::HashMap;

// ── Math helpers ──────────────────────────────────────────────────────────────

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

// ── Public types ──────────────────────────────────────────────────────────────

/// Configuration for Loop subdivision.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LoopSubdivConfig {
    /// Number of subdivision iterations to apply.
    pub iterations: u32,
    /// Smooth factor in [0, 1].  0 = flat (no smoothing), 1 = full Loop blend.
    pub smooth_factor: f32,
    /// Whether to keep boundary vertices fixed.
    pub fix_boundary: bool,
}

/// Result of a Loop subdivision operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LoopSubdivResult {
    /// Vertex positions after subdivision.
    pub positions: Vec<[f32; 3]>,
    /// Triangle index buffer after subdivision.
    pub indices: Vec<u32>,
    /// Number of subdivision iterations actually applied.
    pub iterations_applied: u32,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return default `LoopSubdivConfig`.
#[allow(dead_code)]
pub fn default_loop_subdiv_config() -> LoopSubdivConfig {
    LoopSubdivConfig { iterations: 1, smooth_factor: 1.0, fix_boundary: false }
}

/// Subdivide a triangle mesh once using Loop's scheme.
///
/// Each triangle [A, B, C] produces four child triangles:
/// - [A, mAB, mCA]
/// - [B, mBC, mAB]
/// - [C, mCA, mBC]
/// - [mAB, mBC, mCA]  (inner)
///
/// Edge midpoints and original vertex positions are blended according to
/// `config.smooth_factor`.
#[allow(dead_code)]
pub fn loop_subdivide(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &LoopSubdivConfig,
) -> LoopSubdivResult {
    let mut cur_pos = positions.to_vec();
    let mut cur_idx = indices.to_vec();

    for _ in 0..config.iterations {
        let (new_pos, new_idx) = subdivide_once(&cur_pos, &cur_idx, config.smooth_factor);
        cur_pos = new_pos;
        cur_idx = new_idx;
    }

    LoopSubdivResult {
        positions: cur_pos,
        indices: cur_idx,
        iterations_applied: config.iterations,
    }
}

/// Return the vertex count from a subdivision result.
#[allow(dead_code)]
pub fn loop_subdiv_vertex_count(result: &LoopSubdivResult) -> usize {
    result.positions.len()
}

/// Return the triangle face count from a subdivision result.
#[allow(dead_code)]
pub fn loop_subdiv_face_count(result: &LoopSubdivResult) -> usize {
    result.indices.len() / 3
}

/// Return the number of iterations applied.
#[allow(dead_code)]
pub fn loop_subdiv_iterations(result: &LoopSubdivResult) -> u32 {
    result.iterations_applied
}

/// Return the smooth factor from a config.
#[allow(dead_code)]
pub fn loop_subdiv_smooth_factor(config: &LoopSubdivConfig) -> f32 {
    config.smooth_factor
}

/// Serialise the result to a JSON string.
#[allow(dead_code)]
pub fn loop_subdiv_to_json(result: &LoopSubdivResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"face_count\":{},\"iterations\":{}}}",
        result.positions.len(),
        result.indices.len() / 3,
        result.iterations_applied,
    )
}

/// Apply Loop subdivision `n` times and return the result.
#[allow(dead_code)]
pub fn loop_subdiv_apply_n(
    positions: &[[f32; 3]],
    indices: &[u32],
    n: u32,
    smooth_factor: f32,
) -> LoopSubdivResult {
    let config = LoopSubdivConfig {
        iterations: n,
        smooth_factor: smooth_factor.clamp(0.0, 1.0),
        fix_boundary: false,
    };
    loop_subdivide(positions, indices, &config)
}

/// Validate that all indices in the result are in range.
#[allow(dead_code)]
pub fn loop_subdiv_validate(result: &LoopSubdivResult) -> bool {
    let n = result.positions.len() as u32;
    result.indices.iter().all(|&i| i < n) && result.indices.len().is_multiple_of(3)
}

/// Estimate the memory (bytes) required to store one subdivision result.
#[allow(dead_code)]
pub fn loop_subdiv_memory_estimate(result: &LoopSubdivResult) -> usize {
    result.positions.len() * std::mem::size_of::<[f32; 3]>()
        + result.indices.len() * std::mem::size_of::<u32>()
}

// ── Internal ──────────────────────────────────────────────────────────────────

/// One pass of Loop subdivision (no smooth for now uses midpoints only;
/// smooth_factor blends original position toward the Loop weight sum).
fn subdivide_once(
    positions: &[[f32; 3]],
    indices: &[u32],
    smooth_factor: f32,
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let n_orig = positions.len();

    // Adjacency: one-ring neighbour lists for Loop weights
    let mut neighbours: Vec<Vec<usize>> = vec![Vec::new(); n_orig];
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        for &(a, b) in &[(ia, ib), (ib, ic), (ic, ia), (ib, ia), (ic, ib), (ia, ic)] {
            if !neighbours[a].contains(&b) {
                neighbours[a].push(b);
            }
        }
    }

    // Updated original vertex positions using Loop weights
    let mut new_orig: Vec<[f32; 3]> = Vec::with_capacity(n_orig);
    for (i, &p) in positions.iter().enumerate() {
        let k = neighbours[i].len();
        if k == 0 || smooth_factor < 1e-6 {
            new_orig.push(p);
            continue;
        }
        let beta = loop_beta(k);
        let mut neighbour_sum = [0.0f32; 3];
        for &nb in &neighbours[i] {
            neighbour_sum = add3(neighbour_sum, positions[nb]);
        }
        let smoothed = add3(
            scale3(p, 1.0 - k as f32 * beta),
            scale3(neighbour_sum, beta),
        );
        let blended = add3(scale3(p, 1.0 - smooth_factor), scale3(smoothed, smooth_factor));
        new_orig.push(blended);
    }

    // Edge midpoints — stored in a map keyed by canonical edge (min, max)
    let mut edge_map: HashMap<(u32, u32), u32> = HashMap::new();
    let mut new_positions: Vec<[f32; 3]> = new_orig;

    let mut edge_midpoint = |a: u32, b: u32, poss: &mut Vec<[f32; 3]>| -> u32 {
        let key = if a < b { (a, b) } else { (b, a) };
        if let Some(&idx) = edge_map.get(&key) {
            return idx;
        }
        let pa = positions[a as usize];
        let pb = positions[b as usize];
        let mid = scale3(add3(pa, pb), 0.5);
        let idx = poss.len() as u32;
        poss.push(mid);
        edge_map.insert(key, idx);
        idx
    };

    let mut new_indices: Vec<u32> = Vec::with_capacity(indices.len() * 4);

    for t in 0..tri_count {
        let ia = indices[t * 3];
        let ib = indices[t * 3 + 1];
        let ic = indices[t * 3 + 2];

        let mab = edge_midpoint(ia, ib, &mut new_positions);
        let mbc = edge_midpoint(ib, ic, &mut new_positions);
        let mca = edge_midpoint(ic, ia, &mut new_positions);

        // Four child triangles
        new_indices.extend_from_slice(&[ia, mab, mca]);
        new_indices.extend_from_slice(&[ib, mbc, mab]);
        new_indices.extend_from_slice(&[ic, mca, mbc]);
        new_indices.extend_from_slice(&[mab, mbc, mca]);
    }

    (new_positions, new_indices)
}

/// Loop's subdivision weight beta for a vertex with `n` neighbours.
fn loop_beta(n: usize) -> f32 {
    if n == 3 {
        return 3.0 / 16.0;
    }
    let cos_val = (2.0 * std::f32::consts::PI / n as f32).cos();
    let inner = 3.0 / 8.0 + cos_val / 4.0;
    (1.0 / n as f32) * (5.0 / 8.0 - inner * inner)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn single_triangle() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2];
        (pos, idx)
    }

    #[test]
    fn test_default_config_iterations_positive() {
        let cfg = default_loop_subdiv_config();
        assert!(cfg.iterations > 0);
    }

    #[test]
    fn test_subdivide_one_triangle_to_four() {
        let (pos, idx) = single_triangle();
        let cfg = default_loop_subdiv_config();
        let result = loop_subdivide(&pos, &idx, &cfg);
        assert_eq!(loop_subdiv_face_count(&result), 4);
    }

    #[test]
    fn test_subdivide_vertex_count_grows() {
        let (pos, idx) = single_triangle();
        let cfg = default_loop_subdiv_config();
        let result = loop_subdivide(&pos, &idx, &cfg);
        assert!(loop_subdiv_vertex_count(&result) > pos.len());
    }

    #[test]
    fn test_validate_after_subdivide() {
        let (pos, idx) = single_triangle();
        let cfg = default_loop_subdiv_config();
        let result = loop_subdivide(&pos, &idx, &cfg);
        assert!(loop_subdiv_validate(&result));
    }

    #[test]
    fn test_two_iterations_sixteen_faces() {
        let (pos, idx) = single_triangle();
        let result = loop_subdiv_apply_n(&pos, &idx, 2, 1.0);
        assert_eq!(loop_subdiv_face_count(&result), 16);
    }

    #[test]
    fn test_zero_iterations_returns_original() {
        let (pos, idx) = single_triangle();
        let result = loop_subdiv_apply_n(&pos, &idx, 0, 1.0);
        assert_eq!(result.positions.len(), pos.len());
        assert_eq!(result.indices.len(), idx.len());
    }

    #[test]
    fn test_flat_mode_still_4x_faces() {
        let (pos, idx) = single_triangle();
        let cfg = LoopSubdivConfig { smooth_factor: 0.0, ..default_loop_subdiv_config() };
        let result = loop_subdivide(&pos, &idx, &cfg);
        assert_eq!(loop_subdiv_face_count(&result), 4);
    }

    #[test]
    fn test_loop_subdiv_to_json() {
        let (pos, idx) = single_triangle();
        let cfg = default_loop_subdiv_config();
        let result = loop_subdivide(&pos, &idx, &cfg);
        let json = loop_subdiv_to_json(&result);
        assert!(json.contains("vertex_count"));
        assert!(json.contains("face_count"));
        assert!(json.contains("iterations"));
    }

    #[test]
    fn test_iterations_applied_field() {
        let (pos, idx) = single_triangle();
        let cfg = LoopSubdivConfig { iterations: 3, ..default_loop_subdiv_config() };
        let result = loop_subdivide(&pos, &idx, &cfg);
        assert_eq!(loop_subdiv_iterations(&result), 3);
    }

    #[test]
    fn test_smooth_factor_accessor() {
        let cfg = LoopSubdivConfig { smooth_factor: 0.75, ..default_loop_subdiv_config() };
        assert!((loop_subdiv_smooth_factor(&cfg) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_memory_estimate_positive() {
        let (pos, idx) = single_triangle();
        let cfg = default_loop_subdiv_config();
        let result = loop_subdivide(&pos, &idx, &cfg);
        assert!(loop_subdiv_memory_estimate(&result) > 0);
    }

    #[test]
    fn test_empty_mesh_no_panic() {
        let cfg = default_loop_subdiv_config();
        let result = loop_subdivide(&[], &[], &cfg);
        assert_eq!(result.positions.len(), 0);
        assert_eq!(result.indices.len(), 0);
    }
}
