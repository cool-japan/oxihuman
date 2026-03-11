// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Programmatic creation and editing of morph targets from mesh pairs.

use oxihuman_core::parser::target::Delta;

/// Configuration for morph target authoring.
#[allow(dead_code)]
pub struct AuthoringConfig {
    /// Minimum delta magnitude to include (default 1e-5).
    pub threshold: f32,
    /// Number of Laplacian smooth iterations before storing (default 0).
    pub smooth_iterations: u32,
    /// Normalize max delta to 1.0 (default false).
    pub normalize: bool,
}

impl Default for AuthoringConfig {
    fn default() -> Self {
        Self {
            threshold: 1e-5,
            smooth_iterations: 0,
            normalize: false,
        }
    }
}

/// A morph target produced by the authoring pipeline.
#[allow(dead_code)]
pub struct AuthoredTarget {
    pub name: String,
    pub deltas: Vec<Delta>,
    pub nonzero_count: usize,
    pub max_magnitude: f32,
    /// `[min_xyz, max_xyz]` bounding box of the delta field.
    pub bounds: [[f32; 3]; 2],
}

// ── helpers ──────────────────────────────────────────────────────────────────

#[inline]
fn magnitude(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn build_authored(name: &str, raw: Vec<[f32; 3]>, cfg: &AuthoringConfig) -> AuthoredTarget {
    // optionally smooth in-place
    let mut field = raw;
    if cfg.smooth_iterations > 0 {
        // We have no topology here so we use a simple averaging over the whole
        // field as a stand-in (zero-index array = no edge info).
        // real callers should use smooth_target_deltas with proper indices.
        for _ in 0..cfg.smooth_iterations {
            let len = field.len();
            if len < 2 {
                break;
            }
            let mut smoothed = field.clone();
            for i in 0..len {
                let prev = if i == 0 { len - 1 } else { i - 1 };
                let next = (i + 1) % len;
                smoothed[i] = [
                    (field[prev][0] + field[i][0] + field[next][0]) / 3.0,
                    (field[prev][1] + field[i][1] + field[next][1]) / 3.0,
                    (field[prev][2] + field[i][2] + field[next][2]) / 3.0,
                ];
            }
            field = smoothed;
        }
    }

    // optional normalize
    if cfg.normalize {
        let max_m = field.iter().map(|v| magnitude(*v)).fold(0.0_f32, f32::max);
        if max_m > 1e-12 {
            for v in &mut field {
                v[0] /= max_m;
                v[1] /= max_m;
                v[2] /= max_m;
            }
        }
    }

    // build deltas with threshold filter
    let mut deltas: Vec<Delta> = Vec::new();
    for (vid, v) in field.iter().enumerate() {
        if magnitude(*v) >= cfg.threshold {
            deltas.push(Delta {
                vid: vid as u32,
                dx: v[0],
                dy: v[1],
                dz: v[2],
            });
        }
    }

    let nonzero_count = deltas.len();
    let max_magnitude = deltas
        .iter()
        .map(|d| magnitude([d.dx, d.dy, d.dz]))
        .fold(0.0_f32, f32::max);
    let bounds = target_delta_bounds(&deltas);

    AuthoredTarget {
        name: name.to_owned(),
        deltas,
        nonzero_count,
        max_magnitude,
        bounds,
    }
}

// ── public API ────────────────────────────────────────────────────────────────

/// Compute per-vertex delta = deformed − base, filter by threshold, then
/// optionally smooth and/or normalize.
#[allow(dead_code)]
pub fn create_target_from_mesh_pair(
    name: &str,
    base: &[[f32; 3]],
    deformed: &[[f32; 3]],
    cfg: &AuthoringConfig,
) -> AuthoredTarget {
    let len = base.len().min(deformed.len());
    let raw: Vec<[f32; 3]> = (0..len)
        .map(|i| {
            [
                deformed[i][0] - base[i][0],
                deformed[i][1] - base[i][1],
                deformed[i][2] - base[i][2],
            ]
        })
        .collect();
    build_authored(name, raw, cfg)
}

/// Build an `AuthoredTarget` directly from a delta array.
#[allow(dead_code)]
pub fn create_target_from_delta_field(
    name: &str,
    deltas: &[[f32; 3]],
    cfg: &AuthoringConfig,
) -> AuthoredTarget {
    build_authored(name, deltas.to_vec(), cfg)
}

/// Weighted blend of two targets' delta fields (union of nonzero vertices).
#[allow(dead_code)]
pub fn merge_targets(a: &AuthoredTarget, b: &AuthoredTarget, blend: f32) -> AuthoredTarget {
    let blend = blend.clamp(0.0, 1.0);
    // determine the maximum vertex index referenced
    let max_vid_a = a.deltas.iter().map(|d| d.vid).max().unwrap_or(0);
    let max_vid_b = b.deltas.iter().map(|d| d.vid).max().unwrap_or(0);
    let n = (max_vid_a.max(max_vid_b) as usize) + 1;

    let mut field = vec![[0.0_f32; 3]; n];
    for d in &a.deltas {
        field[d.vid as usize] = [
            d.dx * (1.0 - blend),
            d.dy * (1.0 - blend),
            d.dz * (1.0 - blend),
        ];
    }
    for d in &b.deltas {
        let v = &mut field[d.vid as usize];
        v[0] += d.dx * blend;
        v[1] += d.dy * blend;
        v[2] += d.dz * blend;
    }

    let cfg = AuthoringConfig {
        threshold: 0.0,
        ..AuthoringConfig::default()
    };
    let mut result = build_authored(&a.name, field, &cfg);
    result.name = format!("{}_merge_{}", a.name, b.name);
    result
}

/// Multiply all deltas by `scale`.
#[allow(dead_code)]
pub fn scale_target(t: &AuthoredTarget, scale: f32) -> AuthoredTarget {
    let field: Vec<[f32; 3]> = {
        let max_vid = t.deltas.iter().map(|d| d.vid).max().unwrap_or(0) as usize;
        let mut v = vec![[0.0_f32; 3]; max_vid + 1];
        for d in &t.deltas {
            v[d.vid as usize] = [d.dx * scale, d.dy * scale, d.dz * scale];
        }
        v
    };
    let cfg = AuthoringConfig {
        threshold: 0.0,
        ..AuthoringConfig::default()
    };
    let mut result = build_authored(&t.name, field, &cfg);
    result.name = t.name.clone();
    result
}

/// Negate all deltas.
#[allow(dead_code)]
pub fn invert_target(t: &AuthoredTarget) -> AuthoredTarget {
    scale_target(t, -1.0)
}

/// Mirror deltas along the X axis using symmetry vertex pairs.
#[allow(dead_code)]
pub fn mirror_target_x(t: &AuthoredTarget, sym_pairs: &[(usize, usize)]) -> AuthoredTarget {
    let max_vid = t.deltas.iter().map(|d| d.vid).max().unwrap_or(0) as usize;
    // also ensure field covers all vertex indices referenced in sym_pairs
    let max_sym_vid = sym_pairs
        .iter()
        .flat_map(|(a, b)| [*a, *b])
        .max()
        .unwrap_or(0);
    let field_len = max_vid.max(max_sym_vid) + 1;
    let mut field = vec![[0.0_f32; 3]; field_len];
    for d in &t.deltas {
        field[d.vid as usize] = [d.dx, d.dy, d.dz];
    }

    let mut mirrored = field.clone();
    for (l, r) in sym_pairs {
        if *l < field.len() && *r < field.len() {
            // mirror X component sign
            mirrored[*r] = [-field[*l][0], field[*l][1], field[*l][2]];
            mirrored[*l] = [-field[*r][0], field[*r][1], field[*r][2]];
        }
    }

    let cfg = AuthoringConfig {
        threshold: 0.0,
        ..AuthoringConfig::default()
    };
    let mut result = build_authored(&t.name, mirrored, &cfg);
    result.name = format!("{}_mirrored", t.name);
    result
}

/// Compute `[min_xyz, max_xyz]` bounds of a delta field.
#[allow(dead_code)]
pub fn target_delta_bounds(deltas: &[Delta]) -> [[f32; 3]; 2] {
    if deltas.is_empty() {
        return [[0.0; 3], [0.0; 3]];
    }
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for d in deltas {
        let v = [d.dx, d.dy, d.dz];
        for i in 0..3 {
            mn[i] = mn[i].min(v[i]);
            mx[i] = mx[i].max(v[i]);
        }
    }
    [mn, mx]
}

/// Laplacian-smooth a delta field in place using mesh indices for adjacency.
/// `indices` is a flat triangle list (groups of 3).
#[allow(dead_code)]
pub fn smooth_target_deltas(deltas: &mut [[f32; 3]], indices: &[u32], iterations: u32) {
    let n = deltas.len();
    if n == 0 || iterations == 0 {
        return;
    }

    // Build adjacency
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if a < n && b < n && c < n {
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
    }

    for _ in 0..iterations {
        let prev = deltas.to_vec();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let mut sum = [0.0_f32; 3];
            for &nb in &adj[i] {
                sum[0] += prev[nb][0];
                sum[1] += prev[nb][1];
                sum[2] += prev[nb][2];
            }
            let cnt = adj[i].len() as f32;
            deltas[i] = [sum[0] / cnt, sum[1] / cnt, sum[2] / cnt];
        }
    }
}

/// Return a human-readable stats string for an `AuthoredTarget`.
#[allow(dead_code)]
pub fn authored_target_stats(t: &AuthoredTarget) -> String {
    format!(
        "Target '{}': {} nonzero deltas, max_magnitude={:.6}, bounds=min{:?} max{:?}",
        t.name, t.nonzero_count, t.max_magnitude, t.bounds[0], t.bounds[1]
    )
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cfg() -> AuthoringConfig {
        AuthoringConfig::default()
    }

    // 1. create_target_from_mesh_pair produces correct delta count
    #[test]
    fn test_mesh_pair_delta_count() {
        let base = vec![[0.0, 0.0, 0.0]; 10];
        let mut deformed = base.clone();
        deformed[3] = [1.0, 0.0, 0.0];
        deformed[7] = [0.0, 2.0, 0.0];
        let t = create_target_from_mesh_pair("t", &base, &deformed, &default_cfg());
        assert_eq!(t.nonzero_count, 2);
    }

    // 2. threshold filters small deltas
    #[test]
    fn test_threshold_filters() {
        let base = vec![[0.0, 0.0, 0.0]; 5];
        let deformed = vec![
            [1e-6, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let cfg = AuthoringConfig {
            threshold: 1e-5,
            ..Default::default()
        };
        let t = create_target_from_mesh_pair("t", &base, &deformed, &cfg);
        // 1e-6 < 1e-5 should be filtered out; only 1.0 survives
        assert_eq!(t.nonzero_count, 1);
    }

    // 3. create_target_from_delta_field nonzero_count
    #[test]
    fn test_delta_field_nonzero_count() {
        let deltas = vec![[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let t = create_target_from_delta_field("d", &deltas, &default_cfg());
        assert_eq!(t.nonzero_count, 2);
    }

    // 4. merge_targets blend=0 returns a's deltas
    #[test]
    fn test_merge_blend0_is_a() {
        let base = vec![[0.0; 3]; 4];
        let def_a = vec![
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let def_b = vec![
            [0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let a = create_target_from_mesh_pair("a", &base, &def_a, &default_cfg());
        let b = create_target_from_mesh_pair("b", &base, &def_b, &default_cfg());
        let m = merge_targets(&a, &b, 0.0);
        // At blend=0, vertex 0 should have dx≈1.0
        let d0 = m.deltas.iter().find(|d| d.vid == 0).unwrap();
        assert!((d0.dx - 1.0).abs() < 1e-5);
        // vertex 1 should have zero contribution
        assert!(m.deltas.iter().all(|d| d.vid != 1 || d.dy.abs() < 1e-5));
    }

    // 5. merge_targets blend=1 returns b's deltas
    #[test]
    fn test_merge_blend1_is_b() {
        let base = vec![[0.0; 3]; 4];
        let def_a = vec![
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let def_b = vec![
            [0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let a = create_target_from_mesh_pair("a", &base, &def_a, &default_cfg());
        let b = create_target_from_mesh_pair("b", &base, &def_b, &default_cfg());
        let m = merge_targets(&a, &b, 1.0);
        // At blend=1, vertex 1 should have dy≈2.0
        let d1 = m.deltas.iter().find(|d| d.vid == 1).unwrap();
        assert!((d1.dy - 2.0).abs() < 1e-5);
        assert!(m.deltas.iter().all(|d| d.vid != 0 || d.dx.abs() < 1e-5));
    }

    // 6. scale_target doubles
    #[test]
    fn test_scale_target_doubles() {
        let deltas = vec![[1.0, 2.0, 3.0]];
        let t = create_target_from_delta_field("s", &deltas, &default_cfg());
        let scaled = scale_target(&t, 2.0);
        let d = &scaled.deltas[0];
        assert!((d.dx - 2.0).abs() < 1e-5);
        assert!((d.dy - 4.0).abs() < 1e-5);
        assert!((d.dz - 6.0).abs() < 1e-5);
    }

    // 7. invert_target negates
    #[test]
    fn test_invert_target_negates() {
        let deltas = vec![[1.0, -2.0, 3.0]];
        let t = create_target_from_delta_field("i", &deltas, &default_cfg());
        let inv = invert_target(&t);
        let d = &inv.deltas[0];
        assert!((d.dx + 1.0).abs() < 1e-5);
        assert!((d.dy - 2.0).abs() < 1e-5);
        assert!((d.dz + 3.0).abs() < 1e-5);
    }

    // 8. invert twice = identity
    #[test]
    fn test_invert_twice_identity() {
        let deltas = vec![[1.0, -2.0, 3.0], [0.5, 0.5, 0.5]];
        let t = create_target_from_delta_field("i2", &deltas, &default_cfg());
        let inv2 = invert_target(&invert_target(&t));
        for (orig, inv) in t.deltas.iter().zip(inv2.deltas.iter()) {
            assert!((orig.dx - inv.dx).abs() < 1e-4);
            assert!((orig.dy - inv.dy).abs() < 1e-4);
            assert!((orig.dz - inv.dz).abs() < 1e-4);
        }
    }

    // 9. target_delta_bounds min ≤ max
    #[test]
    fn test_bounds_min_le_max() {
        let deltas = vec![[1.0, -2.0, 3.0], [-1.0, 4.0, 0.5]];
        let t = create_target_from_delta_field("b", &deltas, &default_cfg());
        let b = &t.bounds;
        for (i, (mn, mx)) in b[0].iter().zip(b[1].iter()).enumerate() {
            assert!(mn <= mx, "min > max at axis {i}");
        }
    }

    // 10. target_delta_bounds correct values
    #[test]
    fn test_bounds_values() {
        let d = vec![
            Delta {
                vid: 0,
                dx: 1.0,
                dy: -2.0,
                dz: 3.0,
            },
            Delta {
                vid: 1,
                dx: -1.0,
                dy: 4.0,
                dz: 0.5,
            },
        ];
        let b = target_delta_bounds(&d);
        assert!((b[0][0] - (-1.0)).abs() < 1e-6);
        assert!((b[1][0] - 1.0).abs() < 1e-6);
        assert!((b[0][1] - (-2.0)).abs() < 1e-6);
        assert!((b[1][1] - 4.0).abs() < 1e-6);
    }

    // 11. smooth_target_deltas reduces magnitude on a connected mesh
    #[test]
    fn test_smooth_reduces_magnitude() {
        // Triangle: 0-1-2
        let indices = vec![0u32, 1, 2];
        let mut deltas = vec![[10.0_f32, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let before = deltas[0][0];
        smooth_target_deltas(&mut deltas, &indices, 1);
        let after = deltas[0][0];
        assert!(after < before, "smoothing should reduce peak magnitude");
    }

    // 12. authored_target_stats non-empty
    #[test]
    fn test_authored_target_stats_non_empty() {
        let deltas = vec![[1.0, 0.0, 0.0]];
        let t = create_target_from_delta_field("stats", &deltas, &default_cfg());
        let s = authored_target_stats(&t);
        assert!(!s.is_empty());
        assert!(s.contains("stats"));
    }

    // 13. mirror_target_x swaps pairs
    #[test]
    fn test_mirror_target_x_swaps() {
        let deltas = vec![[1.0, 2.0, 3.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let t = create_target_from_delta_field("mx", &deltas, &default_cfg());
        let mirrored = mirror_target_x(&t, &[(0, 1)]);
        // vertex 1 should now have dx = -1.0 (mirrored X)
        let d1 = mirrored.deltas.iter().find(|d| d.vid == 1);
        assert!(d1.is_some(), "mirrored vertex 1 should appear");
        let d1 = d1.unwrap();
        assert!(
            (d1.dx - (-1.0)).abs() < 1e-5,
            "X should be negated: got {}",
            d1.dx
        );
    }

    // 14. normalize flag scales max to 1
    #[test]
    fn test_normalize_flag() {
        let deltas = vec![[0.0, 0.0, 5.0], [0.0, 0.0, 3.0]];
        let cfg = AuthoringConfig {
            normalize: true,
            ..Default::default()
        };
        let t = create_target_from_delta_field("norm", &deltas, &cfg);
        assert!(
            (t.max_magnitude - 1.0).abs() < 1e-5,
            "max_magnitude should be 1.0 after normalize, got {}",
            t.max_magnitude
        );
    }

    // 15. empty bounds on empty delta list
    #[test]
    fn test_bounds_empty() {
        let b = target_delta_bounds(&[]);
        assert_eq!(b, [[0.0; 3], [0.0; 3]]);
    }
}
