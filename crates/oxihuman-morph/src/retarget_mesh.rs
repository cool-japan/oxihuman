// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Retarget mesh geometry between different topologies using closest-point transfer.

/// Configuration for mesh retargeting.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RetargetMeshConfig {
    /// Max search distance for closest point (default 0.1).
    pub search_radius: f32,
    /// Post-transfer smoothing passes (default 2).
    pub smooth_iterations: u32,
    /// Blend factor: 0 = no transfer, 1 = full transfer (default 1.0).
    pub blend: f32,
}

impl Default for RetargetMeshConfig {
    fn default() -> Self {
        Self {
            search_radius: 0.1,
            smooth_iterations: 2,
            blend: 1.0,
        }
    }
}

/// Result of a mesh retargeting operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RetargetMeshResult {
    /// Retargeted vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Vertices successfully retargeted.
    pub transferred_count: usize,
    /// Vertices with no source within radius.
    pub failed_count: usize,
    /// Mean closest-point distance.
    pub avg_error: f32,
}

/// Find the closest vertex in `positions` to `query` within `max_dist`.
/// Returns `(index, distance)` or `None` if none found within radius.
#[allow(dead_code)]
pub fn closest_vertex(
    query: [f32; 3],
    positions: &[[f32; 3]],
    max_dist: f32,
) -> Option<(usize, f32)> {
    let mut best_idx = None;
    let mut best_dist = max_dist;
    for (i, &p) in positions.iter().enumerate() {
        let d = dist3(query, p);
        if d < best_dist {
            best_dist = d;
            best_idx = Some(i);
        }
    }
    best_idx.map(|i| (i, best_dist))
}

/// Retarget source vertex positions by transferring deformation deltas from a target mesh.
///
/// For each source vertex: find closest vertex in target_base; compute
/// `delta = target_deformed[closest] - target_base[closest]`; apply `delta * blend`
/// to source position. Vertices with no match within `search_radius` are marked failed.
#[allow(dead_code)]
pub fn retarget_mesh_positions(
    source: &[[f32; 3]],
    target_base: &[[f32; 3]],
    target_deformed: &[[f32; 3]],
    cfg: &RetargetMeshConfig,
) -> RetargetMeshResult {
    let n = source.len();
    let mut positions = Vec::with_capacity(n);
    let mut failed_mask = Vec::with_capacity(n);
    let mut transferred_count = 0usize;
    let mut failed_count = 0usize;
    let mut error_sum = 0.0f32;

    for &sv in source.iter() {
        match closest_vertex(sv, target_base, cfg.search_radius) {
            Some((idx, d)) => {
                let delta = sub3(target_deformed[idx], target_base[idx]);
                let scaled = scale3(delta, cfg.blend);
                positions.push(add3(sv, scaled));
                failed_mask.push(false);
                transferred_count += 1;
                error_sum += d;
            }
            None => {
                positions.push(sv);
                failed_mask.push(true);
                failed_count += 1;
            }
        }
    }

    let avg_error = if transferred_count > 0 {
        error_sum / transferred_count as f32
    } else {
        0.0
    };

    // Optional Laplacian smoothing of failed vertices (no adjacency info here, skip smoothing
    // since no index buffer provided — callers should call smooth_transferred_positions separately)
    let _ = cfg.smooth_iterations; // used when caller invokes smooth_transferred_positions

    RetargetMeshResult {
        positions,
        transferred_count,
        failed_count,
        avg_error,
    }
}

/// Transfer morph deltas from target topology to source topology.
///
/// For each source vertex: find closest vertex in target_base; apply the corresponding
/// target delta (scaled by blend) as the source delta.
#[allow(dead_code)]
pub fn transfer_deltas(
    source: &[[f32; 3]],
    source_base: &[[f32; 3]],
    target_base: &[[f32; 3]],
    target_deltas: &[[f32; 3]],
    cfg: &RetargetMeshConfig,
) -> Vec<[f32; 3]> {
    let _ = source_base; // retained for API symmetry / future use
    source
        .iter()
        .map(
            |&sv| match closest_vertex(sv, target_base, cfg.search_radius) {
                Some((idx, _)) => scale3(target_deltas[idx], cfg.blend),
                None => [0.0, 0.0, 0.0],
            },
        )
        .collect()
}

/// Laplacian smooth only failed vertices using triangle index data.
///
/// For failed vertices, replace position with average of neighboring vertices
/// derived from the index buffer. Performs `iterations` passes.
#[allow(dead_code)]
pub fn smooth_transferred_positions(
    positions: &[[f32; 3]],
    failed_mask: &[bool],
    indices: &[u32],
    iterations: u32,
) -> Vec<[f32; 3]> {
    let n = positions.len();
    // Build adjacency list from triangle indices
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if !adj[a].contains(&b) {
            adj[a].push(b);
        }
        if !adj[a].contains(&c) {
            adj[a].push(c);
        }
        if !adj[b].contains(&a) {
            adj[b].push(a);
        }
        if !adj[b].contains(&c) {
            adj[b].push(c);
        }
        if !adj[c].contains(&a) {
            adj[c].push(a);
        }
        if !adj[c].contains(&b) {
            adj[c].push(b);
        }
    }

    let mut current: Vec<[f32; 3]> = positions.to_vec();
    for _ in 0..iterations {
        let prev = current.clone();
        for (i, cur_pos) in current.iter_mut().enumerate() {
            if failed_mask.get(i).copied().unwrap_or(false) && !adj[i].is_empty() {
                let mut sum = [0.0f32; 3];
                for &nb in &adj[i] {
                    sum = add3(sum, prev[nb]);
                }
                let cnt = adj[i].len() as f32;
                *cur_pos = [sum[0] / cnt, sum[1] / cnt, sum[2] / cnt];
            }
        }
    }
    current
}

/// Format a human-readable error statistics string from a `RetargetMeshResult`.
#[allow(dead_code)]
pub fn retarget_error_stats(result: &RetargetMeshResult) -> String {
    let total = result.transferred_count + result.failed_count;
    let coverage = if total > 0 {
        result.transferred_count as f32 / total as f32 * 100.0
    } else {
        0.0
    };
    format!(
        "transferred={} failed={} coverage={:.1}% avg_error={:.6}",
        result.transferred_count, result.failed_count, coverage, result.avg_error
    )
}

// ── Inline math helpers ────────────────────────────────────────────────────

#[inline]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_default() -> RetargetMeshConfig {
        RetargetMeshConfig::default()
    }

    // 1. closest_vertex finds nearest
    #[test]
    fn test_closest_vertex_finds_nearest() {
        let pts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.0, 0.0]];
        let (idx, d) = closest_vertex([0.49, 0.0, 0.0], &pts, 1.0).expect("should succeed");
        assert_eq!(idx, 2);
        assert!(d < 0.02);
    }

    // 2. closest_vertex returns None when max_dist exceeded
    #[test]
    fn test_closest_vertex_none_beyond_radius() {
        let pts = vec![[10.0, 0.0, 0.0]];
        assert!(closest_vertex([0.0, 0.0, 0.0], &pts, 0.5).is_none());
    }

    // 3. closest_vertex exact match (distance = 0)
    #[test]
    fn test_closest_vertex_exact() {
        let pts = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let (idx, d) = closest_vertex([1.0, 2.0, 3.0], &pts, 0.001).expect("should succeed");
        assert_eq!(idx, 0);
        assert!(d < 1e-6);
    }

    // 4. retarget_mesh_positions identity: source == target_base → zero delta
    #[test]
    fn test_retarget_identity_zero_delta() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let cfg = RetargetMeshConfig {
            search_radius: 1.0,
            ..cfg_default()
        };
        let result = retarget_mesh_positions(&verts, &verts, &verts, &cfg);
        for (orig, out) in verts.iter().zip(result.positions.iter()) {
            assert!((orig[0] - out[0]).abs() < 1e-6);
            assert!((orig[1] - out[1]).abs() < 1e-6);
            assert!((orig[2] - out[2]).abs() < 1e-6);
        }
        assert_eq!(result.failed_count, 0);
    }

    // 5. retarget applies delta correctly
    #[test]
    fn test_retarget_applies_delta() {
        let source = vec![[0.0, 0.0, 0.0]];
        let target_base = vec![[0.0, 0.0, 0.0]];
        let target_deformed = vec![[0.0, 1.0, 0.0]];
        let cfg = RetargetMeshConfig {
            search_radius: 0.5,
            blend: 1.0,
            ..cfg_default()
        };
        let result = retarget_mesh_positions(&source, &target_base, &target_deformed, &cfg);
        assert!((result.positions[0][1] - 1.0).abs() < 1e-6);
    }

    // 6. blend=0 → no change
    #[test]
    fn test_retarget_blend_zero_no_change() {
        let source = vec![[0.0, 0.0, 0.0]];
        let target_base = vec![[0.0, 0.0, 0.0]];
        let target_deformed = vec![[0.0, 5.0, 0.0]];
        let cfg = RetargetMeshConfig {
            search_radius: 0.5,
            blend: 0.0,
            ..cfg_default()
        };
        let result = retarget_mesh_positions(&source, &target_base, &target_deformed, &cfg);
        assert!((result.positions[0][1]).abs() < 1e-6);
    }

    // 7. blend=1 → full transfer
    #[test]
    fn test_retarget_blend_one_full() {
        let source = vec![[0.0, 0.0, 0.0]];
        let target_base = vec![[0.0, 0.0, 0.0]];
        let target_deformed = vec![[3.0, 0.0, 0.0]];
        let cfg = RetargetMeshConfig {
            search_radius: 0.5,
            blend: 1.0,
            ..cfg_default()
        };
        let result = retarget_mesh_positions(&source, &target_base, &target_deformed, &cfg);
        assert!((result.positions[0][0] - 3.0).abs() < 1e-6);
    }

    // 8. failed_count for out-of-radius vertex
    #[test]
    fn test_retarget_failed_count() {
        let source = vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0]];
        let target_base = vec![[0.0, 0.0, 0.0]];
        let target_deformed = vec![[0.0, 1.0, 0.0]];
        let cfg = RetargetMeshConfig {
            search_radius: 0.5,
            ..cfg_default()
        };
        let result = retarget_mesh_positions(&source, &target_base, &target_deformed, &cfg);
        assert_eq!(result.failed_count, 1);
        assert_eq!(result.transferred_count, 1);
    }

    // 9. transfer_deltas count matches source
    #[test]
    fn test_transfer_deltas_count_matches_source() {
        let source = vec![[0.0f32; 3]; 5];
        let source_base = vec![[0.0f32; 3]; 5];
        let target_base = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let target_deltas = vec![[0.1, 0.0, 0.0], [0.2, 0.0, 0.0]];
        let cfg = RetargetMeshConfig {
            search_radius: 2.0,
            ..cfg_default()
        };
        let out = transfer_deltas(&source, &source_base, &target_base, &target_deltas, &cfg);
        assert_eq!(out.len(), 5);
    }

    // 10. transfer_deltas correct delta applied
    #[test]
    fn test_transfer_deltas_value() {
        let source = vec![[0.0, 0.0, 0.0]];
        let source_base = vec![[0.0, 0.0, 0.0]];
        let target_base = vec![[0.0, 0.0, 0.0]];
        let target_deltas = vec![[0.5, 0.25, 0.1]];
        let cfg = RetargetMeshConfig {
            search_radius: 1.0,
            blend: 1.0,
            ..cfg_default()
        };
        let out = transfer_deltas(&source, &source_base, &target_base, &target_deltas, &cfg);
        assert!((out[0][0] - 0.5).abs() < 1e-6);
    }

    // 11. smooth_transferred_positions with no failures → no-op
    #[test]
    fn test_smooth_no_failures_noop() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let failed_mask = vec![false, false, false];
        let indices = vec![0u32, 1, 2];
        let out = smooth_transferred_positions(&positions, &failed_mask, &indices, 3);
        for (a, b) in positions.iter().zip(out.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-6);
            assert!((a[1] - b[1]).abs() < 1e-6);
            assert!((a[2] - b[2]).abs() < 1e-6);
        }
    }

    // 12. smooth_transferred_positions with failed vertex moves toward neighbors
    #[test]
    fn test_smooth_failed_vertex_moves() {
        let positions = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [99.0, 99.0, 99.0]];
        let failed_mask = vec![false, false, true];
        let indices = vec![0u32, 1, 2];
        let out = smooth_transferred_positions(&positions, &failed_mask, &indices, 1);
        // vertex 2 should move toward average of 0 and 1 = (1,0,0)
        assert!((out[2][0] - 1.0).abs() < 1e-5);
    }

    // 13. avg_error is computed (non-negative, reasonable)
    #[test]
    fn test_avg_error_computed() {
        let source = vec![[0.05, 0.0, 0.0]];
        let target_base = vec![[0.0, 0.0, 0.0]];
        let target_deformed = vec![[0.0, 0.1, 0.0]];
        let cfg = RetargetMeshConfig {
            search_radius: 1.0,
            ..cfg_default()
        };
        let result = retarget_mesh_positions(&source, &target_base, &target_deformed, &cfg);
        assert!(result.avg_error >= 0.0);
        assert!(result.avg_error < 1.0);
    }

    // 14. retarget_error_stats format check
    #[test]
    fn test_retarget_error_stats_format() {
        let result = RetargetMeshResult {
            positions: vec![],
            transferred_count: 8,
            failed_count: 2,
            avg_error: 0.01234,
        };
        let s = retarget_error_stats(&result);
        assert!(s.contains("transferred=8"));
        assert!(s.contains("failed=2"));
        assert!(s.contains("avg_error"));
    }

    // 15. retarget_error_stats with zero total
    #[test]
    fn test_retarget_error_stats_zero_total() {
        let result = RetargetMeshResult {
            positions: vec![],
            transferred_count: 0,
            failed_count: 0,
            avg_error: 0.0,
        };
        let s = retarget_error_stats(&result);
        assert!(s.contains("coverage=0.0%"));
    }
}
