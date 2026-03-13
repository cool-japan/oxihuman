// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Color scale bar legend widget for heat-map and scalar field visualization.

#![allow(dead_code)]

/// Configuration for a color-bar legend widget.
#[derive(Debug, Clone)]
pub struct ColorBarConfig {
    /// Minimum value of the scalar range.
    pub range_min: f64,
    /// Maximum value of the scalar range.
    pub range_max: f64,
    /// Number of label tick marks displayed along the bar.
    pub tick_count: u32,
    /// Optional unit string appended to labels (e.g. "°C", "m/s").
    pub unit: String,
    /// Pretty-print JSON output.
    pub pretty: bool,
}

/// A single colour stop in the gradient.
#[derive(Debug, Clone)]
pub struct ColorStop {
    /// Normalised position in [0, 1].
    pub position: f64,
    /// Red channel [0, 255].
    pub r: u8,
    /// Green channel [0, 255].
    pub g: u8,
    /// Blue channel [0, 255].
    pub b: u8,
    /// Optional label override for this stop.
    pub label: String,
}

/// The color-bar legend state.
#[derive(Debug, Clone)]
pub struct ColorBarLegend {
    /// Colour stops defining the gradient.
    pub stops: Vec<ColorStop>,
    /// Current minimum scalar value.
    pub range_min: f64,
    /// Current maximum scalar value.
    pub range_max: f64,
}

/// Returns the default [`ColorBarConfig`].
pub fn default_color_bar_config() -> ColorBarConfig {
    ColorBarConfig {
        range_min: 0.0,
        range_max: 1.0,
        tick_count: 5,
        unit: String::new(),
        pretty: true,
    }
}

/// Creates a new, empty [`ColorBarLegend`].
pub fn new_color_bar_legend(cfg: &ColorBarConfig) -> ColorBarLegend {
    ColorBarLegend {
        stops: Vec::new(),
        range_min: cfg.range_min,
        range_max: cfg.range_max,
    }
}

/// Adds a colour stop to the legend.
pub fn color_bar_add_stop(legend: &mut ColorBarLegend, stop: ColorStop) {
    legend.stops.push(stop);
    // keep stops sorted by position for correct sampling
    legend
        .stops
        .sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap_or(std::cmp::Ordering::Equal));
}

/// Samples the colour at normalised position `t` ∈ [0, 1] using linear interpolation.
/// Returns `(r, g, b)` as `u8` values.
pub fn color_bar_sample(legend: &ColorBarLegend, t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    if legend.stops.is_empty() {
        return (0, 0, 0);
    }
    if legend.stops.len() == 1 {
        let s = &legend.stops[0];
        return (s.r, s.g, s.b);
    }
    // find surrounding stops
    let first = &legend.stops[0];
    let last = &legend.stops[legend.stops.len() - 1];
    if t <= first.position {
        return (first.r, first.g, first.b);
    }
    if t >= last.position {
        return (last.r, last.g, last.b);
    }
    for i in 0..legend.stops.len() - 1 {
        let lo = &legend.stops[i];
        let hi = &legend.stops[i + 1];
        if t >= lo.position && t <= hi.position {
            let span = hi.position - lo.position;
            let f = if span > 0.0 { (t - lo.position) / span } else { 0.0 };
            let r = (lo.r as f64 + f * (hi.r as f64 - lo.r as f64)).round() as u8;
            let g = (lo.g as f64 + f * (hi.g as f64 - lo.g as f64)).round() as u8;
            let b = (lo.b as f64 + f * (hi.b as f64 - lo.b as f64)).round() as u8;
            return (r, g, b);
        }
    }
    (0, 0, 0)
}

/// Returns the number of colour stops.
pub fn color_bar_stop_count(legend: &ColorBarLegend) -> usize {
    legend.stops.len()
}

/// Returns the label for the tick at index `tick_idx` out of `cfg.tick_count`.
pub fn color_bar_label_at(legend: &ColorBarLegend, cfg: &ColorBarConfig, tick_idx: u32) -> String {
    let n = cfg.tick_count.max(2) - 1;
    let t = tick_idx as f64 / n as f64;
    let value = legend.range_min + t * (legend.range_max - legend.range_min);
    if cfg.unit.is_empty() {
        format!("{:.2}", value)
    } else {
        format!("{:.2} {}", value, cfg.unit)
    }
}

/// Updates the scalar range of the legend.
pub fn color_bar_set_range(legend: &mut ColorBarLegend, min: f64, max: f64) {
    legend.range_min = min;
    legend.range_max = max;
}

/// Serialises the legend as JSON.
pub fn color_bar_to_json(legend: &ColorBarLegend, cfg: &ColorBarConfig) -> String {
    let indent = if cfg.pretty { "  " } else { "" };
    let nl = if cfg.pretty { "\n" } else { "" };
    let mut out = format!(
        "{{{nl}{indent}\"range_min\":{:.6},{nl}{indent}\"range_max\":{:.6},{nl}{indent}\"unit\":\"{}\",{nl}{indent}\"stops\":[{nl}",
        legend.range_min, legend.range_max, cfg.unit
    );
    for (i, s) in legend.stops.iter().enumerate() {
        let comma = if i + 1 < legend.stops.len() { "," } else { "" };
        out.push_str(&format!(
            "{indent}{indent}{{\"pos\":{:.4},\"r\":{},\"g\":{},\"b\":{},\"label\":\"{}\"}}{}{nl}",
            s.position, s.r, s.g, s.b, s.label, comma
        ));
    }
    out.push_str(&format!("{indent}]{nl}}}"));
    out
}

/// Removes all colour stops.
pub fn color_bar_reset(legend: &mut ColorBarLegend) {
    legend.stops.clear();
}

/// Returns `(min, max)` of the current scalar range.
pub fn color_bar_min_max(legend: &ColorBarLegend) -> (f64, f64) {
    (legend.range_min, legend.range_max)
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_stop(position: f64, r: u8, g: u8, b: u8, label: &str) -> ColorStop {
    ColorStop {
        position,
        r,
        g,
        b,
        label: label.to_string(),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_color_bar_config();
        assert!((cfg.range_min - 0.0).abs() < 1e-9);
        assert!((cfg.range_max - 1.0).abs() < 1e-9);
        assert_eq!(cfg.tick_count, 5);
        assert!(cfg.unit.is_empty());
    }

    #[test]
    fn new_legend_is_empty() {
        let cfg = default_color_bar_config();
        let legend = new_color_bar_legend(&cfg);
        assert_eq!(color_bar_stop_count(&legend), 0);
    }

    #[test]
    fn add_stop_increments_count() {
        let cfg = default_color_bar_config();
        let mut legend = new_color_bar_legend(&cfg);
        color_bar_add_stop(&mut legend, make_stop(0.0, 0, 0, 255, ""));
        assert_eq!(color_bar_stop_count(&legend), 1);
    }

    #[test]
    fn stops_sorted_by_position_after_insert() {
        let cfg = default_color_bar_config();
        let mut legend = new_color_bar_legend(&cfg);
        color_bar_add_stop(&mut legend, make_stop(1.0, 255, 0, 0, ""));
        color_bar_add_stop(&mut legend, make_stop(0.0, 0, 0, 255, ""));
        assert!((legend.stops[0].position - 0.0).abs() < 1e-9);
        assert!((legend.stops[1].position - 1.0).abs() < 1e-9);
    }

    #[test]
    fn sample_interpolates_between_stops() {
        let cfg = default_color_bar_config();
        let mut legend = new_color_bar_legend(&cfg);
        color_bar_add_stop(&mut legend, make_stop(0.0, 0, 0, 0, ""));
        color_bar_add_stop(&mut legend, make_stop(1.0, 200, 100, 50, ""));
        let (r, g, b) = color_bar_sample(&legend, 0.5);
        assert_eq!(r, 100);
        assert_eq!(g, 50);
        assert_eq!(b, 25);
    }

    #[test]
    fn sample_clamps_out_of_range() {
        let cfg = default_color_bar_config();
        let mut legend = new_color_bar_legend(&cfg);
        color_bar_add_stop(&mut legend, make_stop(0.0, 10, 20, 30, ""));
        let (r, g, b) = color_bar_sample(&legend, -0.5);
        assert_eq!((r, g, b), (10, 20, 30));
        let (r2, g2, b2) = color_bar_sample(&legend, 1.5);
        assert_eq!((r2, g2, b2), (10, 20, 30));
    }

    #[test]
    fn label_at_returns_formatted_value() {
        let cfg = ColorBarConfig {
            range_min: 0.0,
            range_max: 100.0,
            tick_count: 5,
            unit: "°C".to_string(),
            pretty: true,
        };
        let legend = new_color_bar_legend(&cfg);
        let label = color_bar_label_at(&legend, &cfg, 0);
        assert!(label.contains("0.00"));
        let label_last = color_bar_label_at(&legend, &cfg, 4);
        assert!(label_last.contains("100.00"));
    }

    #[test]
    fn set_range_updates_min_max() {
        let cfg = default_color_bar_config();
        let mut legend = new_color_bar_legend(&cfg);
        color_bar_set_range(&mut legend, -1.0, 5.0);
        let (min, max) = color_bar_min_max(&legend);
        assert!((min - (-1.0)).abs() < 1e-9);
        assert!((max - 5.0).abs() < 1e-9);
    }

    #[test]
    fn json_contains_stops_key() {
        let cfg = default_color_bar_config();
        let mut legend = new_color_bar_legend(&cfg);
        color_bar_add_stop(&mut legend, make_stop(0.0, 0, 0, 255, "cold"));
        let json = color_bar_to_json(&legend, &cfg);
        assert!(json.contains("\"stops\""));
        assert!(json.contains("cold"));
    }

    #[test]
    fn reset_clears_stops() {
        let cfg = default_color_bar_config();
        let mut legend = new_color_bar_legend(&cfg);
        color_bar_add_stop(&mut legend, make_stop(0.0, 255, 0, 0, ""));
        color_bar_reset(&mut legend);
        assert_eq!(color_bar_stop_count(&legend), 0);
    }
}
