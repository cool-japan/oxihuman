// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Mesh alignment by principal axes (PCA-based) or iterative closest point (ICP).

#[allow(dead_code)]
pub struct AlignResult {
    pub transformed_positions: Vec<[f32; 3]>,
    pub rotation: [[f32; 3]; 3],
    pub translation: [f32; 3],
    pub scale: f32,
}

#[allow(dead_code)]
pub struct IcpResult {
    pub aligned_positions: Vec<[f32; 3]>,
    pub iterations: usize,
    pub residual: f32,
}

/// Compute the centroid (mean position) of a point cloud.
#[allow(dead_code)]
pub fn compute_centroid(positions: &[[f32; 3]]) -> [f32; 3] {
    if positions.is_empty() {
        return [0.0; 3];
    }
    let n = positions.len() as f32;
    let sum = positions.iter().fold([0.0_f32; 3], |acc, p| {
        [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
    });
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Translate all positions by `offset`.
#[allow(dead_code)]
pub fn translate_mesh(positions: &[[f32; 3]], offset: [f32; 3]) -> Vec<[f32; 3]> {
    positions
        .iter()
        .map(|p| [p[0] + offset[0], p[1] + offset[1], p[2] + offset[2]])
        .collect()
}

/// Apply a 3x3 rotation matrix to all positions.
#[allow(dead_code)]
pub fn apply_rotation(positions: &[[f32; 3]], rotation: &[[f32; 3]; 3]) -> Vec<[f32; 3]> {
    positions
        .iter()
        .map(|p| {
            [
                rotation[0][0] * p[0] + rotation[0][1] * p[1] + rotation[0][2] * p[2],
                rotation[1][0] * p[0] + rotation[1][1] * p[1] + rotation[1][2] * p[2],
                rotation[2][0] * p[0] + rotation[2][1] * p[1] + rotation[2][2] * p[2],
            ]
        })
        .collect()
}

/// Compute the 3x3 covariance matrix of positions around their centroid.
#[allow(dead_code)]
pub fn covariance_matrix_3x3(positions: &[[f32; 3]]) -> [[f32; 3]; 3] {
    if positions.is_empty() {
        return [[0.0; 3]; 3];
    }
    let c = compute_centroid(positions);
    let mut cov = [[0.0_f32; 3]; 3];
    for p in positions {
        let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] += d[i] * d[j];
            }
        }
    }
    let n = positions.len() as f32;
    for row in cov.iter_mut() {
        for v in row.iter_mut() {
            *v /= n;
        }
    }
    cov
}

/// Center the mesh at the origin and normalize its scale to fit a unit sphere.
/// Returns identity rotation (PCA-based rotation is omitted for numerical stability).
#[allow(dead_code)]
pub fn align_to_axes(positions: &[[f32; 3]]) -> AlignResult {
    if positions.is_empty() {
        return AlignResult {
            transformed_positions: Vec::new(),
            rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            translation: [0.0; 3],
            scale: 1.0,
        };
    }
    let centroid = compute_centroid(positions);
    let centered = translate_mesh(positions, [-centroid[0], -centroid[1], -centroid[2]]);
    // Compute max distance from origin for scale normalization
    let max_dist = centered
        .iter()
        .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt())
        .fold(0.0_f32, f32::max);
    let scale = if max_dist > 1e-8 { 1.0 / max_dist } else { 1.0 };
    let transformed = centered
        .iter()
        .map(|p| [p[0] * scale, p[1] * scale, p[2] * scale])
        .collect();
    let identity = [[1.0_f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    AlignResult {
        transformed_positions: transformed,
        rotation: identity,
        translation: [-centroid[0], -centroid[1], -centroid[2]],
        scale,
    }
}

/// Find the index of the nearest point in `targets` to `point`.
#[allow(dead_code)]
pub fn find_nearest(point: [f32; 3], targets: &[[f32; 3]]) -> usize {
    let mut best_idx = 0;
    let mut best_dist = f32::INFINITY;
    for (i, t) in targets.iter().enumerate() {
        let d = (t[0] - point[0]).powi(2) + (t[1] - point[1]).powi(2) + (t[2] - point[2]).powi(2);
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }
    best_idx
}

/// Simplified ICP: iteratively align `source` to `target` via nearest-neighbor
/// correspondence and centroid-based translation.
#[allow(dead_code)]
pub fn icp_align(source: &[[f32; 3]], target: &[[f32; 3]], max_iter: usize, tol: f32) -> IcpResult {
    if source.is_empty() || target.is_empty() {
        return IcpResult {
            aligned_positions: source.to_vec(),
            iterations: 0,
            residual: 0.0,
        };
    }
    let mut current = source.to_vec();
    let mut iters = 0;
    let mut residual = f32::INFINITY;

    for _ in 0..max_iter {
        // Find correspondences
        let correspondences: Vec<[f32; 3]> = current
            .iter()
            .map(|&p| target[find_nearest(p, target)])
            .collect();
        // Compute centroids
        let src_c = compute_centroid(&current);
        let tgt_c = compute_centroid(&correspondences);
        // Translate source to align centroids
        let offset = [
            tgt_c[0] - src_c[0],
            tgt_c[1] - src_c[1],
            tgt_c[2] - src_c[2],
        ];
        let new_positions = translate_mesh(&current, offset);
        // Compute residual
        residual = new_positions
            .iter()
            .zip(correspondences.iter())
            .map(|(p, c)| {
                ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2)).sqrt()
            })
            .sum::<f32>()
            / current.len() as f32;
        current = new_positions;
        iters += 1;
        if residual < tol {
            break;
        }
    }

    IcpResult {
        aligned_positions: current,
        iterations: iters,
        residual,
    }
}

/// Scale all positions by a uniform factor.
#[allow(dead_code)]
pub fn scale_mesh(positions: &[[f32; 3]], scale: f32) -> Vec<[f32; 3]> {
    positions
        .iter()
        .map(|p| [p[0] * scale, p[1] * scale, p[2] * scale])
        .collect()
}

/// Compute the axis-aligned bounding box: returns (min, max).
#[allow(dead_code)]
pub fn bounding_box(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    if positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = positions[0];
    let mut mx = positions[0];
    for p in positions.iter().skip(1) {
        for i in 0..3 {
            if p[i] < mn[i] {
                mn[i] = p[i];
            }
            if p[i] > mx[i] {
                mx[i] = p[i];
            }
        }
    }
    (mn, mx)
}

/// Normalize positions to fit within [0, 1]^3.
#[allow(dead_code)]
pub fn normalize_to_unit_box(positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
    if positions.is_empty() {
        return Vec::new();
    }
    let (mn, mx) = bounding_box(positions);
    let extent = [
        (mx[0] - mn[0]).max(1e-8),
        (mx[1] - mn[1]).max(1e-8),
        (mx[2] - mn[2]).max(1e-8),
    ];
    positions
        .iter()
        .map(|p| {
            [
                (p[0] - mn[0]) / extent[0],
                (p[1] - mn[1]) / extent[1],
                (p[2] - mn[2]) / extent[2],
            ]
        })
        .collect()
}

/// Translate and scale `source` so its bounding box matches that of `target`.
#[allow(dead_code)]
pub fn align_bounding_boxes(source: &[[f32; 3]], target: &[[f32; 3]]) -> Vec<[f32; 3]> {
    if source.is_empty() {
        return Vec::new();
    }
    if target.is_empty() {
        return source.to_vec();
    }
    let (s_mn, s_mx) = bounding_box(source);
    let (t_mn, t_mx) = bounding_box(target);
    let s_ext = [
        (s_mx[0] - s_mn[0]).max(1e-8),
        (s_mx[1] - s_mn[1]).max(1e-8),
        (s_mx[2] - s_mn[2]).max(1e-8),
    ];
    let t_ext = [
        (t_mx[0] - t_mn[0]).max(1e-8),
        (t_mx[1] - t_mn[1]).max(1e-8),
        (t_mx[2] - t_mn[2]).max(1e-8),
    ];
    source
        .iter()
        .map(|p| {
            [
                t_mn[0] + (p[0] - s_mn[0]) / s_ext[0] * t_ext[0],
                t_mn[1] + (p[1] - s_mn[1]) / s_ext[1] * t_ext[1],
                t_mn[2] + (p[2] - s_mn[2]) / s_ext[2] * t_ext[2],
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_positions() -> Vec<[f32; 3]> {
        vec![[1.0, 2.0, 3.0], [3.0, 4.0, 5.0], [5.0, 6.0, 7.0]]
    }

    #[test]
    fn centroid_of_symmetric_set_is_center() {
        let pts = vec![
            [-1.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
        ];
        let c = compute_centroid(&pts);
        assert!((c[0]).abs() < 1e-5);
        assert!((c[1]).abs() < 1e-5);
    }

    #[test]
    fn translate_then_centroid_is_zero() {
        let pts = simple_positions();
        let c = compute_centroid(&pts);
        let translated = translate_mesh(&pts, [-c[0], -c[1], -c[2]]);
        let c2 = compute_centroid(&translated);
        assert!(c2[0].abs() < 1e-5 && c2[1].abs() < 1e-5 && c2[2].abs() < 1e-5);
    }

    #[test]
    fn scale_mesh_doubles_positions() {
        let pts = vec![[1.0f32, 2.0, 3.0]];
        let scaled = scale_mesh(&pts, 2.0);
        assert!((scaled[0][0] - 2.0).abs() < 1e-6);
        assert!((scaled[0][1] - 4.0).abs() < 1e-6);
        assert!((scaled[0][2] - 6.0).abs() < 1e-6);
    }

    #[test]
    fn bounding_box_correct() {
        let pts = vec![[-1.0f32, -2.0, -3.0], [4.0, 5.0, 6.0], [0.0, 0.0, 0.0]];
        let (mn, mx) = bounding_box(&pts);
        assert!((mn[0] - (-1.0)).abs() < 1e-6);
        assert!((mn[1] - (-2.0)).abs() < 1e-6);
        assert!((mn[2] - (-3.0)).abs() < 1e-6);
        assert!((mx[0] - 4.0).abs() < 1e-6);
        assert!((mx[1] - 5.0).abs() < 1e-6);
        assert!((mx[2] - 6.0).abs() < 1e-6);
    }

    #[test]
    fn bounding_box_empty_returns_zeros() {
        let (mn, mx) = bounding_box(&[]);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
    }

    #[test]
    fn normalize_to_unit_box_range() {
        let pts = vec![[0.0f32, 0.0, 0.0], [10.0, 5.0, 20.0], [5.0, 2.5, 10.0]];
        let normed = normalize_to_unit_box(&pts);
        for p in &normed {
            assert!(p[0] >= -1e-6 && p[0] <= 1.0 + 1e-6);
            assert!(p[1] >= -1e-6 && p[1] <= 1.0 + 1e-6);
            assert!(p[2] >= -1e-6 && p[2] <= 1.0 + 1e-6);
        }
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn covariance_matrix_symmetric() {
        let pts = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        let cov = covariance_matrix_3x3(&pts);
        for (i, row) in cov.iter().enumerate() {
            for (j, _) in row.iter().enumerate() {
                assert!((cov[i][j] - cov[j][i]).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn align_to_axes_centroid_zero() {
        let pts = simple_positions();
        let result = align_to_axes(&pts);
        let c = compute_centroid(&result.transformed_positions);
        assert!(c[0].abs() < 1e-5 && c[1].abs() < 1e-5 && c[2].abs() < 1e-5);
    }

    #[test]
    fn align_to_axes_max_dist_unit() {
        let pts = simple_positions();
        let result = align_to_axes(&pts);
        let max_d = result
            .transformed_positions
            .iter()
            .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt())
            .fold(0.0_f32, f32::max);
        assert!((max_d - 1.0).abs() < 1e-4);
    }

    #[test]
    fn icp_identical_meshes_zero_residual() {
        let pts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let result = icp_align(&pts, &pts, 10, 1e-6);
        assert!(result.residual < 1e-5, "residual={}", result.residual);
    }

    #[test]
    fn icp_offset_source_converges() {
        let source = vec![
            [10.0f32, 10.0, 10.0],
            [11.0, 10.0, 10.0],
            [10.0, 11.0, 10.0],
        ];
        let target = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let result = icp_align(&source, &target, 50, 1e-4);
        assert!(
            result.residual < 0.5,
            "ICP did not converge, residual={}",
            result.residual
        );
    }

    #[test]
    fn find_nearest_returns_correct_index() {
        let targets = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [5.0, 0.0, 0.0]];
        let idx = find_nearest([4.8, 0.0, 0.0], &targets);
        assert_eq!(idx, 2);
    }

    #[test]
    fn apply_rotation_identity_unchanged() {
        let pts = vec![[1.0f32, 2.0, 3.0]];
        let id = [[1.0_f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let out = apply_rotation(&pts, &id);
        assert!((out[0][0] - 1.0).abs() < 1e-6);
        assert!((out[0][1] - 2.0).abs() < 1e-6);
        assert!((out[0][2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn align_bounding_boxes_matches_target_extent() {
        let source = vec![[0.0f32, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let target = vec![[10.0f32, 20.0, 30.0], [20.0, 40.0, 60.0]];
        let aligned = align_bounding_boxes(&source, &target);
        let (mn, mx) = bounding_box(&aligned);
        assert!((mn[0] - 10.0).abs() < 1e-4);
        assert!((mx[0] - 20.0).abs() < 1e-4);
        assert!((mn[1] - 20.0).abs() < 1e-4);
        assert!((mx[1] - 40.0).abs() < 1e-4);
    }

    #[test]
    fn empty_mesh_handled_gracefully() {
        let result = align_to_axes(&[]);
        assert!(result.transformed_positions.is_empty());
        let icp_r = icp_align(&[], &[], 10, 0.001);
        assert!(icp_r.aligned_positions.is_empty());
        assert_eq!(icp_r.iterations, 0);
    }
}
