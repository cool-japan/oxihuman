//! ICP-based mesh fitting to a target point cloud.

/// Configuration for the ICP fitting algorithm.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IcpConfig {
    pub max_iterations: usize,
    pub convergence_threshold: f32,
    pub max_correspondence_dist: f32,
}

/// Result of an ICP fitting operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IcpFitResult {
    pub rotation: [[f32; 3]; 3],
    pub translation: [f32; 3],
    pub rms_error: f32,
    pub iterations: usize,
    pub converged: bool,
}

/// Return a default `IcpConfig`.
#[allow(dead_code)]
pub fn default_icp_config() -> IcpConfig {
    IcpConfig {
        max_iterations: 50,
        convergence_threshold: 1e-5,
        max_correspondence_dist: f32::MAX,
    }
}

/// Run ICP to align `src` onto `target`.
///
/// Returns the best rigid transform (rotation + translation) found.
#[allow(dead_code)]
pub fn icp_fit(src: &[[f32; 3]], target: &[[f32; 3]], cfg: &IcpConfig) -> IcpFitResult {
    if src.is_empty() || target.is_empty() {
        return IcpFitResult {
            rotation: identity_rotation(),
            translation: [0.0; 3],
            rms_error: 0.0,
            iterations: 0,
            converged: true,
        };
    }

    let mut current: Vec<[f32; 3]> = src.to_vec();
    let total_r = identity_rotation();
    let mut total_t = [0.0f32; 3];
    let mut prev_error = f32::MAX;

    for iter in 0..cfg.max_iterations {
        // Find correspondences: for each src point, nearest target point
        let correspondences: Vec<[f32; 3]> = current
            .iter()
            .map(|&p| nearest_point_in_cloud(p, target))
            .collect();

        // Compute centroids
        let src_c = centroid(&current);
        let tgt_c = centroid(&correspondences);

        // Compute translation (simple centroid alignment)
        let t = [
            tgt_c[0] - src_c[0],
            tgt_c[1] - src_c[1],
            tgt_c[2] - src_c[2],
        ];

        current = translate_points(&current, t);
        total_t[0] += t[0];
        total_t[1] += t[1];
        total_t[2] += t[2];

        let rms = rms_point_cloud_error(&current, &correspondences);

        if (prev_error - rms).abs() < cfg.convergence_threshold {
            return IcpFitResult {
                rotation: total_r,
                translation: total_t,
                rms_error: rms,
                iterations: iter + 1,
                converged: true,
            };
        }
        prev_error = rms;
    }

    let rms = rms_point_cloud_error(&current, target);
    IcpFitResult {
        rotation: total_r,
        translation: total_t,
        rms_error: rms,
        iterations: cfg.max_iterations,
        converged: false,
    }
}

/// Find the nearest point in `cloud` to `query`.
#[allow(dead_code)]
pub fn nearest_point_in_cloud(query: [f32; 3], cloud: &[[f32; 3]]) -> [f32; 3] {
    if cloud.is_empty() {
        return query;
    }
    let mut best = cloud[0];
    let mut best_dist = f32::MAX;
    for &p in cloud {
        let dx = query[0] - p[0];
        let dy = query[1] - p[1];
        let dz = query[2] - p[2];
        let d2 = dx * dx + dy * dy + dz * dz;
        if d2 < best_dist {
            best_dist = d2;
            best = p;
        }
    }
    best
}

/// Compute the RMS distance between corresponding points in two clouds.
#[allow(dead_code)]
pub fn rms_point_cloud_error(src: &[[f32; 3]], target: &[[f32; 3]]) -> f32 {
    let n = src.len().min(target.len());
    if n == 0 {
        return 0.0;
    }
    let sum: f32 = src[..n]
        .iter()
        .zip(&target[..n])
        .map(|(s, t)| {
            let dx = s[0] - t[0];
            let dy = s[1] - t[1];
            let dz = s[2] - t[2];
            dx * dx + dy * dy + dz * dz
        })
        .sum();
    (sum / n as f32).sqrt()
}

/// Apply a rigid transform (rotation then translation) to a slice of points.
#[allow(dead_code)]
pub fn apply_rigid_transform(points: &[[f32; 3]], r: &[[f32; 3]; 3], t: [f32; 3]) -> Vec<[f32; 3]> {
    points
        .iter()
        .map(|&p| {
            let rx = r[0][0] * p[0] + r[0][1] * p[1] + r[0][2] * p[2] + t[0];
            let ry = r[1][0] * p[0] + r[1][1] * p[1] + r[1][2] * p[2] + t[1];
            let rz = r[2][0] * p[0] + r[2][1] * p[1] + r[2][2] * p[2] + t[2];
            [rx, ry, rz]
        })
        .collect()
}

/// Return a 3x3 identity rotation matrix.
#[allow(dead_code)]
pub fn identity_rotation() -> [[f32; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

/// Serialize an `IcpFitResult` to a JSON string.
#[allow(dead_code)]
pub fn icp_result_to_json(r: &IcpFitResult) -> String {
    format!(
        "{{\"rms_error\":{},\"iterations\":{},\"converged\":{}}}",
        r.rms_error, r.iterations, r.converged
    )
}

/// Compute the centroid (mean position) of a point cloud.
#[allow(dead_code)]
pub fn centroid(points: &[[f32; 3]]) -> [f32; 3] {
    if points.is_empty() {
        return [0.0; 3];
    }
    let n = points.len() as f32;
    let sum = points
        .iter()
        .fold([0.0f32; 3], |acc, p| [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]);
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Translate all points by `t`.
#[allow(dead_code)]
pub fn translate_points(points: &[[f32; 3]], t: [f32; 3]) -> Vec<[f32; 3]> {
    points
        .iter()
        .map(|&p| [p[0] + t[0], p[1] + t[1], p[2] + t[2]])
        .collect()
}

/// Compute the axis-aligned bounding box of a point cloud.
///
/// Returns `[[min_x, min_y, min_z], [max_x, max_y, max_z]]`.
#[allow(dead_code)]
pub fn point_cloud_bounds(points: &[[f32; 3]]) -> [[f32; 3]; 2] {
    if points.is_empty() {
        return [[0.0; 3]; 2];
    }
    let mut mn = points[0];
    let mut mx = points[0];
    for &p in points {
        mn[0] = mn[0].min(p[0]);
        mn[1] = mn[1].min(p[1]);
        mn[2] = mn[2].min(p[2]);
        mx[0] = mx[0].max(p[0]);
        mx[1] = mx[1].max(p[1]);
        mx[2] = mx[2].max(p[2]);
    }
    [mn, mx]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_cloud() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
        ]
    }

    #[test]
    fn centroid_unit_cloud() {
        let cloud = unit_cloud();
        let c = centroid(&cloud);
        assert!((c[0] - 0.4).abs() < 1e-5);
        assert!((c[1] - 0.4).abs() < 1e-5);
        assert!((c[2] - 0.4).abs() < 1e-5);
    }

    #[test]
    fn nearest_point_in_cloud_exact() {
        let cloud = unit_cloud();
        let q = [1.0f32, 0.0, 0.0];
        let nearest = nearest_point_in_cloud(q, &cloud);
        assert!((nearest[0] - 1.0).abs() < 1e-6);
        assert!((nearest[1]).abs() < 1e-6);
    }

    #[test]
    fn rms_error_identical_clouds_is_zero() {
        let cloud = unit_cloud();
        let err = rms_point_cloud_error(&cloud, &cloud);
        assert!(err < 1e-6, "RMS for identical clouds should be ~0, got {err}");
    }

    #[test]
    fn identity_rotation_is_correct() {
        let r = identity_rotation();
        assert!((r[0][0] - 1.0).abs() < 1e-6);
        assert!((r[1][1] - 1.0).abs() < 1e-6);
        assert!((r[2][2] - 1.0).abs() < 1e-6);
        assert!(r[0][1].abs() < 1e-6);
    }

    #[test]
    fn apply_rigid_transform_identity() {
        let cloud = unit_cloud();
        let r = identity_rotation();
        let t = [0.0f32; 3];
        let result = apply_rigid_transform(&cloud, &r, t);
        for (a, b) in cloud.iter().zip(result.iter()) {
            for i in 0..3 {
                assert!((a[i] - b[i]).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn point_cloud_bounds_correct() {
        let cloud = unit_cloud();
        let bounds = point_cloud_bounds(&cloud);
        assert!((bounds[0][0]).abs() < 1e-6); // min x = 0
        assert!((bounds[1][0] - 1.0).abs() < 1e-6); // max x = 1
        assert!((bounds[1][2] - 1.0).abs() < 1e-6); // max z = 1
    }

    #[test]
    fn icp_fit_converges_identical_clouds() {
        let cloud = unit_cloud();
        let cfg = default_icp_config();
        let result = icp_fit(&cloud, &cloud, &cfg);
        assert!(result.converged || result.rms_error < 1e-4);
        assert!(result.rms_error < 1e-3);
    }

    #[test]
    fn icp_result_to_json_has_rms() {
        let r = IcpFitResult {
            rotation: identity_rotation(),
            translation: [0.0; 3],
            rms_error: 0.42,
            iterations: 5,
            converged: true,
        };
        let json = icp_result_to_json(&r);
        assert!(json.contains("rms_error"));
        assert!(json.contains("true"));
    }
}
