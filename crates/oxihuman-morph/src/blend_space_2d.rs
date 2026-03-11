//! 2D blend space for locomotion blending (e.g. forward/strafe vs. speed axes).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendSpace2dConfig {
    pub axis_x_name: String,
    pub axis_y_name: String,
    pub max_points: usize,
}

#[allow(dead_code)]
pub fn default_blend_space_2d_config() -> BlendSpace2dConfig {
    BlendSpace2dConfig {
        axis_x_name: "x".to_string(),
        axis_y_name: "y".to_string(),
        max_points: 64,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendSpace2dPoint {
    pub x: f32,
    pub y: f32,
    pub label: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendSpace2d {
    config: BlendSpace2dConfig,
    points: Vec<BlendSpace2dPoint>,
    axis_x_name: String,
    axis_y_name: String,
}

#[allow(dead_code)]
pub fn new_blend_space_2d(config: BlendSpace2dConfig) -> BlendSpace2d {
    let ax = config.axis_x_name.clone();
    let ay = config.axis_y_name.clone();
    BlendSpace2d {
        config,
        points: Vec::new(),
        axis_x_name: ax,
        axis_y_name: ay,
    }
}

#[allow(dead_code)]
pub fn bs2d_add_point(bs: &mut BlendSpace2d, x: f32, y: f32, label: &str) -> bool {
    if bs.points.len() >= bs.config.max_points {
        return false;
    }
    bs.points.push(BlendSpace2dPoint {
        x,
        y,
        label: label.to_string(),
    });
    true
}

/// Evaluate blend weights at query point (qx, qy) using inverse-distance weighting.
/// Returns a Vec of (index, weight) pairs that sum to 1.0.
#[allow(dead_code)]
pub fn bs2d_evaluate(bs: &BlendSpace2d, qx: f32, qy: f32) -> Vec<(usize, f32)> {
    if bs.points.is_empty() {
        return Vec::new();
    }
    let eps = 1e-6_f32;
    let mut inv_dists: Vec<f32> = bs
        .points
        .iter()
        .map(|p| {
            let dx = p.x - qx;
            let dy = p.y - qy;
            let d = (dx * dx + dy * dy).sqrt();
            1.0 / d.max(eps)
        })
        .collect();
    let total: f32 = inv_dists.iter().sum();
    if total > 0.0 {
        for v in &mut inv_dists {
            *v /= total;
        }
    }
    inv_dists.into_iter().enumerate().collect()
}

#[allow(dead_code)]
pub fn bs2d_point_count(bs: &BlendSpace2d) -> usize {
    bs.points.len()
}

#[allow(dead_code)]
pub fn bs2d_nearest_point(bs: &BlendSpace2d, qx: f32, qy: f32) -> Option<usize> {
    if bs.points.is_empty() {
        return None;
    }
    let idx = bs
        .points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let dx = p.x - qx;
            let dy = p.y - qy;
            (i, dx * dx + dy * dy)
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i);
    idx
}

#[allow(dead_code)]
pub fn bs2d_set_axes(bs: &mut BlendSpace2d, axis_x: &str, axis_y: &str) {
    bs.axis_x_name = axis_x.to_string();
    bs.axis_y_name = axis_y.to_string();
}

#[allow(dead_code)]
pub fn bs2d_to_json(bs: &BlendSpace2d) -> String {
    let pts: Vec<String> = bs
        .points
        .iter()
        .map(|p| {
            format!(
                r#"{{"x":{},"y":{},"label":"{}"}}"#,
                p.x, p.y, p.label
            )
        })
        .collect();
    format!(
        r#"{{"axis_x":"{}","axis_y":"{}","points":[{}]}}"#,
        bs.axis_x_name,
        bs.axis_y_name,
        pts.join(",")
    )
}

#[allow(dead_code)]
pub fn bs2d_clear(bs: &mut BlendSpace2d) {
    bs.points.clear();
}

/// Returns normalized weights for all points given query position.
#[allow(dead_code)]
pub fn bs2d_weights_at(bs: &BlendSpace2d, qx: f32, qy: f32) -> Vec<f32> {
    let pairs = bs2d_evaluate(bs, qx, qy);
    let mut out = vec![0.0_f32; bs.points.len()];
    for (i, w) in pairs {
        out[i] = w;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bs() -> BlendSpace2d {
        let cfg = default_blend_space_2d_config();
        let mut bs = new_blend_space_2d(cfg);
        bs2d_add_point(&mut bs, 0.0, 0.0, "idle");
        bs2d_add_point(&mut bs, 1.0, 0.0, "walk_fwd");
        bs2d_add_point(&mut bs, 0.0, 1.0, "walk_right");
        bs2d_add_point(&mut bs, 1.0, 1.0, "run_fwd_right");
        bs
    }

    #[test]
    fn test_point_count() {
        let bs = make_bs();
        assert_eq!(bs2d_point_count(&bs), 4);
    }

    #[test]
    fn test_add_point_over_limit() {
        let cfg = BlendSpace2dConfig {
            axis_x_name: "x".to_string(),
            axis_y_name: "y".to_string(),
            max_points: 2,
        };
        let mut bs = new_blend_space_2d(cfg);
        assert!(bs2d_add_point(&mut bs, 0.0, 0.0, "a"));
        assert!(bs2d_add_point(&mut bs, 1.0, 0.0, "b"));
        assert!(!bs2d_add_point(&mut bs, 0.5, 0.5, "c"));
    }

    #[test]
    fn test_evaluate_weights_sum_to_one() {
        let bs = make_bs();
        let weights = bs2d_evaluate(&bs, 0.5, 0.5);
        let total: f32 = weights.iter().map(|(_, w)| w).sum();
        assert!((total - 1.0).abs() < 1e-5, "weights={}", total);
    }

    #[test]
    fn test_evaluate_empty() {
        let cfg = default_blend_space_2d_config();
        let bs = new_blend_space_2d(cfg);
        let w = bs2d_evaluate(&bs, 0.0, 0.0);
        assert!(w.is_empty());
    }

    #[test]
    fn test_nearest_point() {
        let bs = make_bs();
        let idx = bs2d_nearest_point(&bs, 0.1, 0.1);
        assert_eq!(idx, Some(0)); // closest to (0,0) = idle
    }

    #[test]
    fn test_nearest_point_empty() {
        let cfg = default_blend_space_2d_config();
        let bs = new_blend_space_2d(cfg);
        assert_eq!(bs2d_nearest_point(&bs, 0.0, 0.0), None);
    }

    #[test]
    fn test_set_axes() {
        let mut bs = make_bs();
        bs2d_set_axes(&mut bs, "forward", "lateral");
        let json = bs2d_to_json(&bs);
        assert!(json.contains("forward"));
        assert!(json.contains("lateral"));
    }

    #[test]
    fn test_to_json() {
        let bs = make_bs();
        let json = bs2d_to_json(&bs);
        assert!(json.contains("idle"));
        assert!(json.contains("walk_fwd"));
    }

    #[test]
    fn test_clear() {
        let mut bs = make_bs();
        bs2d_clear(&mut bs);
        assert_eq!(bs2d_point_count(&bs), 0);
    }

    #[test]
    fn test_weights_at_len_matches_points() {
        let bs = make_bs();
        let w = bs2d_weights_at(&bs, 0.3, 0.4);
        assert_eq!(w.len(), 4);
        let sum: f32 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_near_exact_point_dominates() {
        let bs = make_bs();
        let w = bs2d_weights_at(&bs, 1.0 + 1e-7, 0.0);
        // index 1 is (1,0), should dominate
        assert!(w[1] > 0.99);
    }
}
