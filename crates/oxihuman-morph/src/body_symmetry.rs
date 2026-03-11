// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Symmetry enforcement and controlled asymmetry injection.

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Which world axis is the symmetry axis.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymmetryAxis {
    pub axis: u8,
}

impl SymmetryAxis {
    pub const X: Self = Self { axis: 0 };
    pub const Y: Self = Self { axis: 1 };
    pub const Z: Self = Self { axis: 2 };
}

/// Configuration for symmetry enforcement.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SymmetryConfig {
    pub axis: SymmetryAxis,
    /// Position tolerance for pair matching.
    pub tolerance: f32,
    /// 0 = fully asymmetric, 1 = fully symmetric.
    pub blend: f32,
}

impl Default for SymmetryConfig {
    fn default() -> Self {
        Self {
            axis: SymmetryAxis::X,
            tolerance: 0.001,
            blend: 1.0,
        }
    }
}

/// Configuration for asymmetry noise injection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AsymmetryConfig {
    /// Overall magnitude 0..1.
    pub strength: f32,
    /// Spatial frequency of asymmetry noise.
    pub frequency: f32,
    pub seed: u64,
}

impl Default for AsymmetryConfig {
    fn default() -> Self {
        Self {
            strength: 0.05,
            frequency: 1.0,
            seed: 42,
        }
    }
}

/// Symmetry analysis report.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SymmetryReport {
    pub paired_vertices: usize,
    pub unpaired_vertices: usize,
    /// Mean distance between paired positions and their mirror.
    pub symmetry_error: f32,
    /// True when symmetry_error < tolerance.
    pub is_symmetric: bool,
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// Mirror a position through the given axis (flip the component).
#[allow(dead_code)]
pub fn mirror_position(p: [f32; 3], axis: &SymmetryAxis) -> [f32; 3] {
    let mut q = p;
    q[axis.axis as usize] = -q[axis.axis as usize];
    q
}

/// LCG-based deterministic noise for asymmetry.
/// Returns a float in roughly –1..1.
#[allow(dead_code)]
pub fn asymmetry_noise(x: f32, y: f32, z: f32, seed: u64) -> f32 {
    // Encode the three floats into integers and mix with LCG.
    let ix = (x * 1000.0) as i64 as u64;
    let iy = (y * 1000.0) as i64 as u64;
    let iz = (z * 1000.0) as i64 as u64;

    let mut state: u64 = seed
        .wrapping_add(ix.wrapping_mul(2_654_435_761))
        .wrapping_add(iy.wrapping_mul(2_246_822_519))
        .wrapping_add(iz.wrapping_mul(3_266_489_917));

    // Three LCG steps for better mixing.
    state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);

    // Map to –1..1.
    let u = (state >> 11) as f32 / (1u64 << 53) as f32; // 0..1
    u * 2.0 - 1.0
}

/// For each vertex with x > 0 find the best match at (−x, y, z) within tolerance.
///
/// Returns `(pos_x_index, neg_x_index)` pairs (no duplicates).
#[allow(dead_code)]
pub fn find_symmetry_pairs_x(positions: &[[f32; 3]], tolerance: f32) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    let mut used_neg: Vec<bool> = vec![false; positions.len()];

    for (i, &pi) in positions.iter().enumerate() {
        if pi[0] <= 0.0 {
            continue;
        }
        // target: (−x, y, z)
        let tx = -pi[0];
        let ty = pi[1];
        let tz = pi[2];

        let mut best_j = None;
        let mut best_d = tolerance;

        for (j, &pj) in positions.iter().enumerate() {
            if used_neg[j] {
                continue;
            }
            let dx = pj[0] - tx;
            let dy = pj[1] - ty;
            let dz = pj[2] - tz;
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            if d <= best_d {
                best_d = d;
                best_j = Some(j);
            }
        }

        if let Some(j) = best_j {
            used_neg[j] = true;
            pairs.push((i, j));
        }
    }

    pairs
}

/// Enforce symmetry for each pair: average both sides (blended by `cfg.blend`).
#[allow(dead_code)]
pub fn enforce_symmetry(
    positions: &mut [[f32; 3]],
    pairs: &[(usize, usize)],
    cfg: &SymmetryConfig,
) {
    let ax = cfg.axis.axis as usize;
    for &(a, b) in pairs {
        if a >= positions.len() || b >= positions.len() {
            continue;
        }

        // Symmetric target: average the non-axis components, mirror axis.
        let pa = positions[a];
        let pb = positions[b];

        // The non-axis coords should be averaged; the axis coords should be equal/opposite.
        let avg_non_ax_y = if ax != 1 { (pa[1] + pb[1]) * 0.5 } else { 0.0 };
        let avg_non_ax_z = if ax != 2 { (pa[2] + pb[2]) * 0.5 } else { 0.0 };

        // For X-axis symmetry: a is +x side, b is −x side.
        // We want: positions[a][0] = +|avg_x|, positions[b][0] = -|avg_x|.
        let sym_ax = (pa[ax].abs() + pb[ax].abs()) * 0.5;

        let mut new_a = pa;
        let mut new_b = pb;

        // Blend toward symmetric target.
        let b_factor = cfg.blend;

        if ax == 0 {
            new_a[0] = pa[0] * (1.0 - b_factor) + sym_ax * b_factor;
            new_b[0] = pb[0] * (1.0 - b_factor) + (-sym_ax) * b_factor;
            new_a[1] = pa[1] * (1.0 - b_factor) + avg_non_ax_y * b_factor;
            new_b[1] = pb[1] * (1.0 - b_factor) + avg_non_ax_y * b_factor;
            new_a[2] = pa[2] * (1.0 - b_factor) + avg_non_ax_z * b_factor;
            new_b[2] = pb[2] * (1.0 - b_factor) + avg_non_ax_z * b_factor;
        } else {
            // Generic: mirror along the given axis.
            let avg_ax = (pa[ax] + pb[ax]) * 0.5;
            new_a[ax] = pa[ax] * (1.0 - b_factor) + avg_ax * b_factor;
            new_b[ax] = pb[ax] * (1.0 - b_factor) + avg_ax * b_factor;
        }

        positions[a] = new_a;
        positions[b] = new_b;
    }
}

/// Compute a symmetry report for `positions` given known pairs.
#[allow(dead_code)]
pub fn symmetry_report(
    positions: &[[f32; 3]],
    pairs: &[(usize, usize)],
    axis: &SymmetryAxis,
) -> SymmetryReport {
    let tolerance = 0.001;
    let paired_vertices = pairs.len() * 2;
    let unpaired_vertices = positions.len().saturating_sub(paired_vertices);

    let symmetry_error = if pairs.is_empty() {
        0.0
    } else {
        let total: f32 = pairs
            .iter()
            .filter_map(|&(a, b)| {
                if a < positions.len() && b < positions.len() {
                    let mirrored = mirror_position(positions[b], axis);
                    let dx = positions[a][0] - mirrored[0];
                    let dy = positions[a][1] - mirrored[1];
                    let dz = positions[a][2] - mirrored[2];
                    Some((dx * dx + dy * dy + dz * dz).sqrt())
                } else {
                    None
                }
            })
            .sum();
        total / pairs.len() as f32
    };

    SymmetryReport {
        paired_vertices,
        unpaired_vertices,
        symmetry_error,
        is_symmetric: symmetry_error < tolerance,
    }
}

/// Inject subtle LCG-noise asymmetry into vertex positions.
#[allow(dead_code)]
pub fn inject_asymmetry(positions: &mut [[f32; 3]], cfg: &AsymmetryConfig) {
    if cfg.strength <= 0.0 {
        return;
    }
    for p in positions.iter_mut() {
        let nx = asymmetry_noise(
            p[0] * cfg.frequency,
            p[1] * cfg.frequency,
            p[2] * cfg.frequency,
            cfg.seed,
        );
        let ny = asymmetry_noise(
            p[1] * cfg.frequency,
            p[2] * cfg.frequency,
            p[0] * cfg.frequency,
            cfg.seed.wrapping_add(1),
        );
        let nz = asymmetry_noise(
            p[2] * cfg.frequency,
            p[0] * cfg.frequency,
            p[1] * cfg.frequency,
            cfg.seed.wrapping_add(2),
        );
        p[0] += nx * cfg.strength;
        p[1] += ny * cfg.strength;
        p[2] += nz * cfg.strength;
    }
}

/// Make morph deltas symmetric: `delta[a]` = `delta[b]` mirrored (average both).
#[allow(dead_code)]
pub fn symmetrize_morph_deltas(
    deltas: &mut [[f32; 3]],
    pairs: &[(usize, usize)],
    axis: &SymmetryAxis,
) {
    for &(a, b) in pairs {
        if a >= deltas.len() || b >= deltas.len() {
            continue;
        }
        let da = deltas[a];
        let db = deltas[b];

        // Mirror b's delta, then average.
        let db_mirrored = mirror_position(db, axis);

        let sym_a = [
            (da[0] + db_mirrored[0]) * 0.5,
            (da[1] + db_mirrored[1]) * 0.5,
            (da[2] + db_mirrored[2]) * 0.5,
        ];
        let sym_b = mirror_position(sym_a, axis);

        deltas[a] = sym_a;
        deltas[b] = sym_b;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_position_x_axis() {
        let p = [1.0f32, 2.0, 3.0];
        let m = mirror_position(p, &SymmetryAxis::X);
        assert!((m[0] - (-1.0)).abs() < 1e-6);
        assert!((m[1] - 2.0).abs() < 1e-6);
        assert!((m[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_mirror_position_y_axis() {
        let p = [1.0f32, 2.0, 3.0];
        let m = mirror_position(p, &SymmetryAxis::Y);
        assert!((m[0] - 1.0).abs() < 1e-6);
        assert!((m[1] - (-2.0)).abs() < 1e-6);
        assert!((m[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_mirror_position_z_axis() {
        let p = [1.0f32, 2.0, 3.0];
        let m = mirror_position(p, &SymmetryAxis::Z);
        assert!((m[0] - 1.0).abs() < 1e-6);
        assert!((m[1] - 2.0).abs() < 1e-6);
        assert!((m[2] - (-3.0)).abs() < 1e-6);
    }

    #[test]
    fn test_symmetry_report_no_pairs() {
        let positions = vec![[0.0f32; 3]; 4];
        let report = symmetry_report(&positions, &[], &SymmetryAxis::X);
        assert_eq!(report.paired_vertices, 0);
        assert_eq!(report.symmetry_error, 0.0);
        assert!(report.is_symmetric);
    }

    #[test]
    fn test_find_symmetry_pairs_x_symmetric_mesh() {
        // 4 vertices: two on each side of X=0
        let positions: Vec<[f32; 3]> = vec![
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-2.0, 0.0, 0.0],
        ];
        let pairs = find_symmetry_pairs_x(&positions, 0.01);
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn test_find_symmetry_pairs_x_asymmetric_mesh() {
        // All vertices on same side – no pairs.
        let positions: Vec<[f32; 3]> = vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0]];
        let pairs = find_symmetry_pairs_x(&positions, 0.01);
        assert_eq!(pairs.len(), 0);
    }

    #[test]
    fn test_enforce_symmetry_makes_positions_symmetric() {
        let mut positions: Vec<[f32; 3]> = vec![
            [1.0, 0.0, 0.0],  // index 0: +x side
            [-1.1, 0.0, 0.0], // index 1: −x side (slightly off)
        ];
        let pairs = vec![(0usize, 1usize)];
        let cfg = SymmetryConfig::default(); // blend=1.0
        enforce_symmetry(&mut positions, &pairs, &cfg);
        // After full symmetry: pos[0][0] should be positive, pos[1][0] negative, same abs.
        assert!(positions[0][0] > 0.0);
        assert!(positions[1][0] < 0.0);
        assert!((positions[0][0] + positions[1][0]).abs() < 1e-4);
    }

    #[test]
    fn test_inject_asymmetry_changes_positions() {
        let original = vec![[0.0f32, 0.0, 0.0]; 4];
        let mut positions = original.clone();
        let cfg = AsymmetryConfig {
            strength: 0.1,
            frequency: 1.0,
            seed: 1234,
        };
        inject_asymmetry(&mut positions, &cfg);
        let changed = positions.iter().any(|&p| p != [0.0, 0.0, 0.0]);
        assert!(changed);
    }

    #[test]
    fn test_inject_asymmetry_zero_strength_no_change() {
        let original = vec![[1.0f32, 2.0, 3.0]; 4];
        let mut positions = original.clone();
        let cfg = AsymmetryConfig {
            strength: 0.0,
            frequency: 1.0,
            seed: 99,
        };
        inject_asymmetry(&mut positions, &cfg);
        assert_eq!(positions, original);
    }

    #[test]
    fn test_symmetrize_morph_deltas_makes_deltas_mirror() {
        let mut deltas: Vec<[f32; 3]> = vec![[1.0, 0.0, 0.0], [0.5, 0.0, 0.0]];
        let pairs = vec![(0usize, 1usize)];
        symmetrize_morph_deltas(&mut deltas, &pairs, &SymmetryAxis::X);
        // Algorithm: mirror db=[0.5,0,0] along X → [-0.5,0,0]
        // sym_a = avg(da, db_mirrored) = avg([1,0,0],[-0.5,0,0]) = [0.25, 0, 0]
        // sym_b = mirror(sym_a) = [-0.25, 0, 0]
        assert!(
            (deltas[0][0] - 0.25).abs() < 1e-5,
            "deltas[0]={:?}",
            deltas[0]
        );
        assert!(
            (deltas[1][0] - (-0.25)).abs() < 1e-5,
            "deltas[1]={:?}",
            deltas[1]
        );
        // The result IS mirrored: deltas[1] = -deltas[0] in X component
        assert!((deltas[0][0] + deltas[1][0]).abs() < 1e-5);
    }

    #[test]
    fn test_asymmetry_noise_deterministic() {
        let n1 = asymmetry_noise(1.0, 2.0, 3.0, 42);
        let n2 = asymmetry_noise(1.0, 2.0, 3.0, 42);
        assert_eq!(n1, n2);
    }

    #[test]
    fn test_asymmetry_noise_different_seeds_differ() {
        let n1 = asymmetry_noise(1.0, 2.0, 3.0, 42);
        let n2 = asymmetry_noise(1.0, 2.0, 3.0, 43);
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_asymmetry_noise_in_range() {
        let n = asymmetry_noise(0.5, 1.5, -0.3, 7);
        assert!((-1.0..=1.0).contains(&n), "n={n}");
    }

    #[test]
    fn test_symmetry_report_with_pairs() {
        // Perfectly symmetric: +1 / -1
        let positions: Vec<[f32; 3]> = vec![[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]];
        let pairs = vec![(0usize, 1usize)];
        let report = symmetry_report(&positions, &pairs, &SymmetryAxis::X);
        assert_eq!(report.paired_vertices, 2);
        assert!(
            report.symmetry_error < 0.001,
            "err={}",
            report.symmetry_error
        );
        assert!(report.is_symmetric);
    }
}
