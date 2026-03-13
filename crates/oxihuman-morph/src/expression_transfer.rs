// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Transfer expression blend shapes between character topologies using
//! nearest-vertex or barycentric projection.

/// Interpolation strategy for expression transfer.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferInterp {
    /// Nearest-vertex lookup.
    Nearest,
    /// Barycentric projection onto closest triangle.
    Barycentric,
}

/// Configuration for expression transfer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionTransferConfig {
    /// Maximum search radius for closest-point lookup (default 0.1).
    pub max_search_radius: f32,
    /// Scale transferred deltas by mesh scale ratio.
    pub normalize_by_scale: bool,
    /// Interpolation mode.
    pub interpolation: TransferInterp,
}

impl Default for ExpressionTransferConfig {
    fn default() -> Self {
        Self {
            max_search_radius: 0.1,
            normalize_by_scale: false,
            interpolation: TransferInterp::Nearest,
        }
    }
}

/// A single transferred expression.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TransferredExpression {
    /// Name of the expression.
    pub name: String,
    /// Per-vertex delta vectors on the target mesh.
    pub deltas: Vec<[f32; 3]>,
    /// Fraction of target vertices with valid transfer.
    pub coverage: f32,
    /// Maximum delta magnitude across all vertices.
    pub max_delta_magnitude: f32,
}

/// Batch result from transferring multiple expressions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionTransferBatch {
    /// All transferred expressions.
    pub expressions: Vec<TransferredExpression>,
    /// Vertex count of source mesh.
    pub source_vertex_count: usize,
    /// Vertex count of target mesh.
    pub target_vertex_count: usize,
}

// ── Public API ─────────────────────────────────────────────────────────────

/// Transfer a single expression from source topology to target topology.
///
/// For each target vertex, find the closest source vertex (Nearest) or closest
/// triangle (Barycentric) and interpolate the delta.
#[allow(dead_code)]
pub fn transfer_expression(
    name: &str,
    source_verts: &[[f32; 3]],
    source_deltas: &[[f32; 3]],
    target_verts: &[[f32; 3]],
    cfg: &ExpressionTransferConfig,
) -> TransferredExpression {
    let scale = if cfg.normalize_by_scale {
        mesh_scale_ratio(source_verts, target_verts)
    } else {
        1.0
    };

    let mut valid = 0usize;
    let mut deltas: Vec<[f32; 3]> = Vec::with_capacity(target_verts.len());

    for &tv in target_verts {
        let d = match cfg.interpolation {
            TransferInterp::Nearest => {
                transfer_nearest(tv, source_verts, source_deltas, cfg.max_search_radius)
            }
            TransferInterp::Barycentric => {
                // Try to find the closest triangle and use barycentric interpolation.
                // We build triangles implicitly from consecutive triplets of source verts,
                // or fall back to nearest if source doesn't form full triangles.
                transfer_barycentric_or_nearest(
                    tv,
                    source_verts,
                    source_deltas,
                    cfg.max_search_radius,
                )
            }
        };

        if let Some(delta) = d {
            deltas.push(scale3(delta, scale));
            valid += 1;
        } else {
            deltas.push([0.0, 0.0, 0.0]);
        }
    }

    let coverage = if target_verts.is_empty() {
        1.0
    } else {
        valid as f32 / target_verts.len() as f32
    };
    let max_delta_magnitude = delta_magnitude(&deltas);

    TransferredExpression {
        name: name.to_string(),
        deltas,
        coverage,
        max_delta_magnitude,
    }
}

/// Transfer a batch of expressions from source topology to target topology.
#[allow(dead_code)]
pub fn transfer_expression_batch(
    expressions: &[(&str, Vec<[f32; 3]>)],
    source_verts: &[[f32; 3]],
    target_verts: &[[f32; 3]],
    cfg: &ExpressionTransferConfig,
) -> ExpressionTransferBatch {
    let transferred = expressions
        .iter()
        .map(|(name, deltas)| transfer_expression(name, source_verts, deltas, target_verts, cfg))
        .collect();

    ExpressionTransferBatch {
        expressions: transferred,
        source_vertex_count: source_verts.len(),
        target_vertex_count: target_verts.len(),
    }
}

/// Compute barycentric coordinates (u, v, w) for point `p` projected onto
/// triangle (a, b, c). Returns `None` if the triangle is degenerate.
#[allow(dead_code)]
pub fn barycentric_coords(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Option<[f32; 3]> {
    let v0 = sub3(b, a);
    let v1 = sub3(c, a);
    let v2 = sub3(p, a);

    let d00 = dot3(v0, v0);
    let d01 = dot3(v0, v1);
    let d11 = dot3(v1, v1);
    let d20 = dot3(v2, v0);
    let d21 = dot3(v2, v1);

    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-10 {
        return None; // degenerate
    }

    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    Some([u, v, w])
}

/// Interpolate a delta using barycentric coordinates (u, v, w).
#[allow(dead_code)]
pub fn interpolate_delta_barycentric(
    d0: [f32; 3],
    d1: [f32; 3],
    d2: [f32; 3],
    uvw: [f32; 3],
) -> [f32; 3] {
    [
        d0[0] * uvw[0] + d1[0] * uvw[1] + d2[0] * uvw[2],
        d0[1] * uvw[0] + d1[1] * uvw[1] + d2[1] * uvw[2],
        d0[2] * uvw[0] + d1[2] * uvw[1] + d2[2] * uvw[2],
    ]
}

/// Compute ratio of bounding-box diagonal sizes: source_diag / target_diag.
/// Returns 1.0 when either mesh is empty or degenerate.
#[allow(dead_code)]
pub fn mesh_scale_ratio(source_verts: &[[f32; 3]], target_verts: &[[f32; 3]]) -> f32 {
    let sd = bbox_diagonal(source_verts);
    let td = bbox_diagonal(target_verts);
    if td < 1e-10 || sd < 1e-10 {
        return 1.0;
    }
    sd / td
}

/// Maximum delta magnitude (L2 norm) over all vertices.
#[allow(dead_code)]
pub fn delta_magnitude(deltas: &[[f32; 3]]) -> f32 {
    deltas
        .iter()
        .map(|&d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
        .fold(0.0f32, f32::max)
}

// ── Private helpers ────────────────────────────────────────────────────────

fn transfer_nearest(
    tv: [f32; 3],
    source_verts: &[[f32; 3]],
    source_deltas: &[[f32; 3]],
    max_dist: f32,
) -> Option<[f32; 3]> {
    let mut best_idx = None;
    let mut best_dist = max_dist;
    for (i, &sv) in source_verts.iter().enumerate() {
        let d = dist3(tv, sv);
        if d < best_dist {
            best_dist = d;
            best_idx = Some(i);
        }
    }
    best_idx.map(|i| source_deltas[i])
}

fn transfer_barycentric_or_nearest(
    tv: [f32; 3],
    source_verts: &[[f32; 3]],
    source_deltas: &[[f32; 3]],
    max_dist: f32,
) -> Option<[f32; 3]> {
    // Try barycentric over consecutive triangles (triplets)
    let n_tris = source_verts.len() / 3;
    let mut best: Option<[f32; 3]> = None;
    let mut best_dist = max_dist;

    for t in 0..n_tris {
        let i0 = t * 3;
        let i1 = i0 + 1;
        let i2 = i0 + 2;
        if let Some(uvw) =
            barycentric_coords(tv, source_verts[i0], source_verts[i1], source_verts[i2])
        {
            // Compute projected point and distance
            let proj = interpolate_delta_barycentric(
                source_verts[i0],
                source_verts[i1],
                source_verts[i2],
                uvw,
            );
            let d = dist3(tv, proj);
            if d < best_dist {
                best_dist = d;
                best = Some(interpolate_delta_barycentric(
                    source_deltas[i0],
                    source_deltas[i1],
                    source_deltas[i2],
                    uvw,
                ));
            }
        }
    }

    if best.is_some() {
        return best;
    }
    // fallback to nearest
    transfer_nearest(tv, source_verts, source_deltas, max_dist)
}

fn bbox_diagonal(verts: &[[f32; 3]]) -> f32 {
    if verts.is_empty() {
        return 0.0;
    }
    let mut mn = verts[0];
    let mut mx = verts[0];
    for &v in verts {
        if v[0] < mn[0] {
            mn[0] = v[0];
        }
        if v[1] < mn[1] {
            mn[1] = v[1];
        }
        if v[2] < mn[2] {
            mn[2] = v[2];
        }
        if v[0] > mx[0] {
            mx[0] = v[0];
        }
        if v[1] > mx[1] {
            mx[1] = v[1];
        }
        if v[2] > mx[2] {
            mx[2] = v[2];
        }
    }
    dist3(mn, mx)
}

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
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_nearest() -> ExpressionTransferConfig {
        ExpressionTransferConfig {
            max_search_radius: 10.0,
            normalize_by_scale: false,
            interpolation: TransferInterp::Nearest,
        }
    }

    // 1. barycentric_coords centroid returns (1/3, 1/3, 1/3)
    #[test]
    fn test_barycentric_centroid() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let centroid = [1.0 / 3.0, 1.0 / 3.0, 0.0];
        let uvw = barycentric_coords(centroid, a, b, c).expect("should succeed");
        assert!((uvw[0] - 1.0 / 3.0).abs() < 1e-5, "uvw[0]={}", uvw[0]);
        assert!((uvw[1] - 1.0 / 3.0).abs() < 1e-5, "uvw[1]={}", uvw[1]);
        assert!((uvw[2] - 1.0 / 3.0).abs() < 1e-5, "uvw[2]={}", uvw[2]);
    }

    // 2. barycentric_coords at corner A returns (1, 0, 0)
    #[test]
    fn test_barycentric_corner_a() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let uvw = barycentric_coords(a, a, b, c).expect("should succeed");
        assert!((uvw[0] - 1.0).abs() < 1e-5);
        assert!(uvw[1].abs() < 1e-5);
        assert!(uvw[2].abs() < 1e-5);
    }

    // 3. barycentric_coords degenerate triangle returns None
    #[test]
    fn test_barycentric_degenerate() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [2.0, 0.0, 0.0]; // collinear
                                 // denom approaches zero for degenerate triangles
                                 // a and b and c are collinear so degenerate
        assert!(barycentric_coords([0.5, 0.0, 0.0], a, b, c).is_none());
    }

    // 4. interpolate_delta_barycentric at corner returns that corner's delta
    #[test]
    fn test_interpolate_delta_at_corner() {
        let d0 = [1.0f32, 0.0, 0.0];
        let d1 = [0.0, 1.0, 0.0];
        let d2 = [0.0, 0.0, 1.0];
        let uvw = [1.0, 0.0, 0.0];
        let r = interpolate_delta_barycentric(d0, d1, d2, uvw);
        assert!((r[0] - 1.0).abs() < 1e-6);
        assert!(r[1].abs() < 1e-6);
        assert!(r[2].abs() < 1e-6);
    }

    // 5. interpolate_delta_barycentric at centroid returns average
    #[test]
    fn test_interpolate_delta_at_centroid() {
        let d0 = [3.0f32, 0.0, 0.0];
        let d1 = [0.0, 3.0, 0.0];
        let d2 = [0.0, 0.0, 3.0];
        let uvw = [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
        let r = interpolate_delta_barycentric(d0, d1, d2, uvw);
        assert!((r[0] - 1.0).abs() < 1e-5);
        assert!((r[1] - 1.0).abs() < 1e-5);
        assert!((r[2] - 1.0).abs() < 1e-5);
    }

    // 6. mesh_scale_ratio identical meshes = 1.0
    #[test]
    fn test_mesh_scale_ratio_identical() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let r = mesh_scale_ratio(&verts, &verts);
        assert!((r - 1.0).abs() < 1e-5);
    }

    // 7. mesh_scale_ratio different scales
    #[test]
    fn test_mesh_scale_ratio_double() {
        let src = vec![[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let tgt = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let r = mesh_scale_ratio(&src, &tgt);
        assert!((r - 2.0).abs() < 1e-5);
    }

    // 8. transfer_expression identity topology → same deltas
    #[test]
    fn test_transfer_expression_identity_topology() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let deltas = vec![[0.1f32, 0.2, 0.3], [0.4, 0.5, 0.6]];
        let cfg = cfg_nearest();
        let result = transfer_expression("test", &verts, &deltas, &verts, &cfg);
        for (a, b) in deltas.iter().zip(result.deltas.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-5);
            assert!((a[1] - b[1]).abs() < 1e-5);
            assert!((a[2] - b[2]).abs() < 1e-5);
        }
    }

    // 9. coverage = 1.0 for same topology within radius
    #[test]
    fn test_transfer_expression_full_coverage() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let deltas = vec![[0.1f32, 0.0, 0.0], [0.2, 0.0, 0.0]];
        let cfg = cfg_nearest();
        let result = transfer_expression("test", &verts, &deltas, &verts, &cfg);
        assert!((result.coverage - 1.0).abs() < 1e-6);
    }

    // 10. coverage < 1.0 when target verts are far from source
    #[test]
    fn test_transfer_expression_partial_coverage() {
        let source_verts = vec![[0.0f32, 0.0, 0.0]];
        let source_deltas = vec![[0.1f32, 0.0, 0.0]];
        let target_verts = vec![[0.0f32, 0.0, 0.0], [100.0, 0.0, 0.0]];
        let cfg = ExpressionTransferConfig {
            max_search_radius: 0.5,
            ..Default::default()
        };
        let result =
            transfer_expression("test", &source_verts, &source_deltas, &target_verts, &cfg);
        assert!(result.coverage < 1.0);
    }

    // 11. batch count matches input
    #[test]
    fn test_transfer_batch_count_matches() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let exprs: Vec<(&str, Vec<[f32; 3]>)> = vec![
            ("smile", vec![[0.1, 0.0, 0.0], [0.2, 0.0, 0.0]]),
            ("frown", vec![[0.0, 0.1, 0.0], [0.0, 0.2, 0.0]]),
            ("blink", vec![[0.0, 0.0, 0.1], [0.0, 0.0, 0.2]]),
        ];
        let cfg = cfg_nearest();
        let batch = transfer_expression_batch(&exprs, &verts, &verts, &cfg);
        assert_eq!(batch.expressions.len(), 3);
        assert_eq!(batch.source_vertex_count, 2);
        assert_eq!(batch.target_vertex_count, 2);
    }

    // 12. delta_magnitude empty returns 0.0
    #[test]
    fn test_delta_magnitude_empty() {
        assert_eq!(delta_magnitude(&[]), 0.0);
    }

    // 13. delta_magnitude correct value
    #[test]
    fn test_delta_magnitude_value() {
        let d = vec![[3.0f32, 4.0, 0.0]];
        assert!((delta_magnitude(&d) - 5.0).abs() < 1e-5);
    }

    // 14. barycentric_coords at corner B returns (0, 1, 0)
    #[test]
    fn test_barycentric_corner_b() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let uvw = barycentric_coords(b, a, b, c).expect("should succeed");
        assert!(uvw[0].abs() < 1e-5);
        assert!((uvw[1] - 1.0).abs() < 1e-5);
        assert!(uvw[2].abs() < 1e-5);
    }
}
