//! Vertex weight/influence visualization — maps skinning weights to colors for heat-map display.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for weight visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightVisConfig {
    /// Name of the active color map (e.g. `"heat"`, `"cool"`, `"mono"`).
    pub colormap: String,
    /// Whether weight visualization is currently enabled.
    pub enabled: bool,
    /// Scale applied to weight values before mapping to color (default 1.0).
    pub scale: f32,
}

/// A single vertex with position and mapped weight color.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightVertex {
    /// 3-D position.
    pub position: [f32; 3],
    /// RGB color derived from the weight value.
    pub color: [f32; 3],
    /// Original weight value \[0, 1\].
    pub weight: f32,
}

/// A GPU-ready draw call payload for weight visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightVisDrawCall {
    /// Vertices ready to be submitted for rendering.
    pub vertices: Vec<WeightVertex>,
    /// Whether the draw call should be rendered.
    pub enabled: bool,
}

// ── Construction ──────────────────────────────────────────────────────────────

/// Returns a default [`WeightVisConfig`].
#[allow(dead_code)]
pub fn default_weight_vis_config() -> WeightVisConfig {
    WeightVisConfig {
        colormap: "heat".to_string(),
        enabled: true,
        scale: 1.0,
    }
}

// ── Color mapping ─────────────────────────────────────────────────────────────

/// Maps a scalar weight in \[0, 1\] to an RGB color according to the config's colormap.
///
/// * `"heat"` — blue → cyan → green → yellow → red
/// * `"cool"` — white → blue
/// * anything else — greyscale
#[allow(dead_code)]
pub fn weight_to_color(weight: f32, cfg: &WeightVisConfig) -> [f32; 3] {
    let w = (weight * cfg.scale).clamp(0.0, 1.0);
    match cfg.colormap.as_str() {
        "heat" => heat_map(w),
        "cool" => {
            let b = 1.0 - w;
            [b, b, 1.0]
        }
        _ => [w, w, w],
    }
}

/// Standard five-stop heat-map (blue → cyan → green → yellow → red).
fn heat_map(t: f32) -> [f32; 3] {
    // Four segments over [0, 1]: each covers 0.25.
    let r;
    let g;
    let b;
    if t < 0.25 {
        let s = t / 0.25;
        r = 0.0;
        g = s;
        b = 1.0;
    } else if t < 0.5 {
        let s = (t - 0.25) / 0.25;
        r = 0.0;
        g = 1.0;
        b = 1.0 - s;
    } else if t < 0.75 {
        let s = (t - 0.5) / 0.25;
        r = s;
        g = 1.0;
        b = 0.0;
    } else {
        let s = (t - 0.75) / 0.25;
        r = 1.0;
        g = 1.0 - s;
        b = 0.0;
    }
    [r, g, b]
}

// ── Build helpers ─────────────────────────────────────────────────────────────

/// Builds a [`Vec<WeightVertex>`] from parallel position and weight slices.
///
/// Positions and weights must have equal length.
#[allow(dead_code)]
pub fn build_weight_vertices(
    positions: &[[f32; 3]],
    weights: &[f32],
    cfg: &WeightVisConfig,
) -> Vec<WeightVertex> {
    positions
        .iter()
        .zip(weights.iter())
        .map(|(pos, &w)| {
            let color = weight_to_color(w, cfg);
            WeightVertex {
                position: *pos,
                color,
                weight: w,
            }
        })
        .collect()
}

/// Creates a [`WeightVisDrawCall`] from position and weight slices.
#[allow(dead_code)]
pub fn weight_vis_draw_call(
    positions: &[[f32; 3]],
    weights: &[f32],
    cfg: &WeightVisConfig,
) -> WeightVisDrawCall {
    WeightVisDrawCall {
        vertices: build_weight_vertices(positions, weights, cfg),
        enabled: cfg.enabled,
    }
}

// ── Query / control ───────────────────────────────────────────────────────────

/// Returns the number of vertices in the draw call.
#[allow(dead_code)]
pub fn weight_vertex_count(call: &WeightVisDrawCall) -> usize {
    call.vertices.len()
}

/// Sets the active colormap by name.
#[allow(dead_code)]
pub fn set_weight_colormap(cfg: &mut WeightVisConfig, colormap: &str) {
    cfg.colormap = colormap.to_string();
}

/// Toggles the enabled state of the weight visualizer.
#[allow(dead_code)]
pub fn weight_vis_toggle_enabled(cfg: &mut WeightVisConfig) {
    cfg.enabled = !cfg.enabled;
}

/// Returns whether the weight visualizer is currently enabled.
#[allow(dead_code)]
pub fn weight_vis_is_enabled(cfg: &WeightVisConfig) -> bool {
    cfg.enabled
}

/// Normalises a slice of weights so that they sum to 1.0.
/// Returns a uniform distribution if the sum is zero.
#[allow(dead_code)]
pub fn normalize_weights(weights: &[f32]) -> Vec<f32> {
    let sum: f32 = weights.iter().sum();
    if sum <= 0.0 {
        if weights.is_empty() {
            return Vec::new();
        }
        let uniform = 1.0 / weights.len() as f32;
        return vec![uniform; weights.len()];
    }
    weights.iter().map(|w| w / sum).collect()
}

/// Returns the index of the vertex with the highest weight.
/// Returns 0 if the slice is empty.
#[allow(dead_code)]
pub fn max_weight_vertex(weights: &[f32]) -> usize {
    if weights.is_empty() {
        return 0;
    }
    weights
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_weight_vis_config();
        assert_eq!(cfg.colormap, "heat");
        assert!(cfg.enabled);
        assert!((cfg.scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_weight_to_color_zero_is_blue() {
        let cfg = default_weight_vis_config();
        let c = weight_to_color(0.0, &cfg);
        // At t=0 the heat map gives blue: [0, 0, 1]
        assert!(c[0] < 1e-6);
        assert!(c[1] < 1e-6);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_weight_to_color_one_is_red() {
        let cfg = default_weight_vis_config();
        let c = weight_to_color(1.0, &cfg);
        // At t=1 the heat map gives red: [1, 0, 0]
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!(c[1] < 1e-6);
        assert!(c[2] < 1e-6);
    }

    #[test]
    fn test_build_weight_vertices_count() {
        let cfg = default_weight_vis_config();
        let positions = vec![[0.0f32; 3]; 5];
        let weights = vec![0.2f32; 5];
        let verts = build_weight_vertices(&positions, &weights, &cfg);
        assert_eq!(verts.len(), 5);
    }

    #[test]
    fn test_toggle_enabled() {
        let mut cfg = default_weight_vis_config();
        assert!(weight_vis_is_enabled(&cfg));
        weight_vis_toggle_enabled(&mut cfg);
        assert!(!weight_vis_is_enabled(&cfg));
        weight_vis_toggle_enabled(&mut cfg);
        assert!(weight_vis_is_enabled(&cfg));
    }

    #[test]
    fn test_normalize_weights() {
        let weights = vec![1.0f32, 3.0, 0.0, 0.0];
        let norm = normalize_weights(&weights);
        let sum: f32 = norm.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
        assert!((norm[0] - 0.25).abs() < 1e-5);
        assert!((norm[1] - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_weights_zero_sum() {
        let weights = vec![0.0f32; 4];
        let norm = normalize_weights(&weights);
        let sum: f32 = norm.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_weight_vertex() {
        let weights = vec![0.1f32, 0.9, 0.5, 0.3];
        assert_eq!(max_weight_vertex(&weights), 1);
    }

    #[test]
    fn test_weight_vis_draw_call_count() {
        let cfg = default_weight_vis_config();
        let positions = vec![[1.0f32, 0.0, 0.0]; 3];
        let weights = vec![0.0f32, 0.5, 1.0];
        let call = weight_vis_draw_call(&positions, &weights, &cfg);
        assert_eq!(weight_vertex_count(&call), 3);
    }

    #[test]
    fn test_set_colormap() {
        let mut cfg = default_weight_vis_config();
        set_weight_colormap(&mut cfg, "cool");
        assert_eq!(cfg.colormap, "cool");
    }
}
