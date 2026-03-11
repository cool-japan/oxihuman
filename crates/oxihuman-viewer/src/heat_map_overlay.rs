// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Heat-map scalar-field overlay that colorizes mesh surface by a per-vertex float value.

#![allow(dead_code)]

/// Configuration for the heat-map overlay.
#[derive(Debug, Clone)]
pub struct HeatMapConfig {
    /// Minimum value of the display range (mapped to cool color).
    pub range_min: f32,
    /// Maximum value of the display range (mapped to hot color).
    pub range_max: f32,
    /// Whether to clamp out-of-range values (vs wrapping).
    pub clamp: bool,
}

/// A single vertex in the heat-map overlay.
#[derive(Debug, Clone)]
pub struct HeatMapVertex {
    /// Scalar value at this vertex.
    pub value: f32,
}

/// State for the heat-map overlay.
#[derive(Debug, Clone)]
pub struct HeatMapOverlay {
    /// Per-vertex scalar values.
    pub vertices: Vec<HeatMapVertex>,
    /// Byte count of the last serialised output.
    pub total_bytes: usize,
}

/// Returns the default [`HeatMapConfig`].
pub fn default_heat_map_config() -> HeatMapConfig {
    HeatMapConfig {
        range_min: 0.0,
        range_max: 1.0,
        clamp: true,
    }
}

/// Creates a new, empty [`HeatMapOverlay`].
pub fn new_heat_map_overlay() -> HeatMapOverlay {
    HeatMapOverlay {
        vertices: Vec::new(),
        total_bytes: 0,
    }
}

/// Replaces all per-vertex scalar values.
pub fn heat_map_set_values(overlay: &mut HeatMapOverlay, values: &[f32]) {
    overlay.vertices = values.iter().map(|&v| HeatMapVertex { value: v }).collect();
}

/// Samples the RGBA color for a given scalar value using the classic heat-map gradient
/// (blue → cyan → green → yellow → red).
/// Returns `[r, g, b, a]` each in 0.0–1.0.
pub fn heat_map_sample_color(
    value: f32,
    cfg: &HeatMapConfig,
) -> [f32; 4] {
    let range = cfg.range_max - cfg.range_min;
    let t = if range.abs() < 1e-10 {
        0.5_f32
    } else {
        let raw = (value - cfg.range_min) / range;
        if cfg.clamp { raw.clamp(0.0, 1.0) } else { raw.fract().abs() }
    };

    // 4-segment gradient: 0..0.25 blue→cyan, 0.25..0.5 cyan→green,
    //                     0.5..0.75 green→yellow, 0.75..1.0 yellow→red
    let (r, g, b) = if t < 0.25 {
        let s = t / 0.25;
        (0.0_f32, s, 1.0_f32)
    } else if t < 0.5 {
        let s = (t - 0.25) / 0.25;
        (0.0_f32, 1.0_f32, 1.0 - s)
    } else if t < 0.75 {
        let s = (t - 0.5) / 0.25;
        (s, 1.0_f32, 0.0_f32)
    } else {
        let s = (t - 0.75) / 0.25;
        (1.0_f32, 1.0 - s, 0.0_f32)
    };
    [r, g, b, 1.0]
}

/// Returns the minimum scalar value across all vertices, or `0.0` if empty.
pub fn heat_map_min_value(overlay: &HeatMapOverlay) -> f32 {
    overlay
        .vertices
        .iter()
        .map(|v| v.value)
        .fold(f32::INFINITY, f32::min)
        .max(0.0_f32)
        // Return 0.0 for empty (fold returns INFINITY, max(0) gives 0)
        .min(if overlay.vertices.is_empty() { 0.0 } else { f32::INFINITY })
}

/// Returns the maximum scalar value across all vertices, or `0.0` if empty.
pub fn heat_map_max_value(overlay: &HeatMapOverlay) -> f32 {
    overlay
        .vertices
        .iter()
        .map(|v| v.value)
        .fold(f32::NEG_INFINITY, f32::max)
        .min(0.0_f32.max(if overlay.vertices.is_empty() { 0.0 } else { f32::INFINITY }))
}

/// Updates the display range in the config.
pub fn heat_map_set_range(cfg: &mut HeatMapConfig, min: f32, max: f32) {
    cfg.range_min = min;
    cfg.range_max = max;
}

/// Serialises the overlay as JSON.
pub fn heat_map_to_json(overlay: &mut HeatMapOverlay, cfg: &HeatMapConfig) -> String {
    let mut out = format!(
        "{{\"range_min\":{:.6},\"range_max\":{:.6},\"clamp\":{},\"vertex_count\":{}}}",
        cfg.range_min,
        cfg.range_max,
        cfg.clamp,
        overlay.vertices.len()
    );
    // Store total bytes
    overlay.total_bytes = out.len();
    // Append values array for completeness (compact)
    let mut arr = String::from("[");
    for (i, v) in overlay.vertices.iter().enumerate() {
        if i > 0 { arr.push(','); }
        arr.push_str(&format!("{:.6}", v.value));
    }
    arr.push(']');
    out = format!(
        "{{\"range_min\":{:.6},\"range_max\":{:.6},\"clamp\":{},\"values\":{}}}",
        cfg.range_min,
        cfg.range_max,
        cfg.clamp,
        arr
    );
    overlay.total_bytes = out.len();
    out
}

/// Clears all vertex data.
pub fn heat_map_clear(overlay: &mut HeatMapOverlay) {
    overlay.vertices.clear();
    overlay.total_bytes = 0;
}

/// Returns the number of vertices in the overlay.
pub fn heat_map_vertex_count(overlay: &HeatMapOverlay) -> usize {
    overlay.vertices.len()
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_heat_map_config();
        assert!((cfg.range_min - 0.0).abs() < 1e-6);
        assert!((cfg.range_max - 1.0).abs() < 1e-6);
        assert!(cfg.clamp);
    }

    #[test]
    fn new_overlay_is_empty() {
        let o = new_heat_map_overlay();
        assert_eq!(heat_map_vertex_count(&o), 0);
    }

    #[test]
    fn set_values_updates_vertex_count() {
        let mut o = new_heat_map_overlay();
        heat_map_set_values(&mut o, &[0.1, 0.5, 0.9]);
        assert_eq!(heat_map_vertex_count(&o), 3);
    }

    #[test]
    fn sample_color_zero_is_blue() {
        let cfg = default_heat_map_config();
        let c = heat_map_sample_color(0.0, &cfg);
        // t=0 → blue: r=0, g=0, b=1
        assert!(c[0] < 0.01);
        assert!(c[2] > 0.99);
    }

    #[test]
    fn sample_color_one_is_red() {
        let cfg = default_heat_map_config();
        let c = heat_map_sample_color(1.0, &cfg);
        // t=1 → red: r=1, g=0, b=0
        assert!(c[0] > 0.99);
        assert!(c[1] < 0.01);
        assert!(c[2] < 0.01);
    }

    #[test]
    fn sample_color_alpha_is_one() {
        let cfg = default_heat_map_config();
        let c = heat_map_sample_color(0.5, &cfg);
        assert!((c[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_range_updates_config() {
        let mut cfg = default_heat_map_config();
        heat_map_set_range(&mut cfg, -10.0, 10.0);
        assert!((cfg.range_min - -10.0).abs() < 1e-6);
        assert!((cfg.range_max - 10.0).abs() < 1e-6);
    }

    #[test]
    fn json_contains_range_fields() {
        let mut o = new_heat_map_overlay();
        heat_map_set_values(&mut o, &[0.2, 0.8]);
        let cfg = default_heat_map_config();
        let json = heat_map_to_json(&mut o, &cfg);
        assert!(json.contains("range_min"));
        assert!(json.contains("range_max"));
        assert!(json.contains("values"));
    }

    #[test]
    fn clear_resets_overlay() {
        let mut o = new_heat_map_overlay();
        heat_map_set_values(&mut o, &[0.5, 0.5, 0.5]);
        heat_map_clear(&mut o);
        assert_eq!(heat_map_vertex_count(&o), 0);
        assert_eq!(o.total_bytes, 0);
    }

    #[test]
    fn set_values_replaces_previous() {
        let mut o = new_heat_map_overlay();
        heat_map_set_values(&mut o, &[0.1, 0.2, 0.3, 0.4]);
        heat_map_set_values(&mut o, &[1.0]);
        assert_eq!(heat_map_vertex_count(&o), 1);
    }

    #[test]
    fn clamped_out_of_range_maps_to_blue_or_red() {
        let cfg = default_heat_map_config();
        let low = heat_map_sample_color(-5.0, &cfg);
        let high = heat_map_sample_color(5.0, &cfg);
        // clamped low → same as 0 → blue
        assert!(low[2] > 0.99);
        // clamped high → same as 1 → red
        assert!(high[0] > 0.99);
    }
}
