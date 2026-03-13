// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Debug information overlay showing FPS, vertex count, and timing info.
//!
//! Provides a lightweight in-memory overlay that accumulates named metrics
//! and can format them as plain text or JSON for display in a viewer HUD.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for the debug overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DebugOverlayConfig {
    /// Maximum number of FPS samples kept for the rolling average.
    pub fps_history_len: usize,
    /// Whether the overlay starts visible.
    pub visible_on_start: bool,
    /// Column separator string used in text formatting.
    pub column_sep: String,
    /// Whether to include the timestamp in text output.
    pub show_timestamp: bool,
}

/// A single named debug metric with a string value.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DebugMetric {
    /// Metric key (e.g. `"fps"`, `"vertex_count"`).
    pub key: String,
    /// Current value formatted as a string.
    pub value: String,
    /// Optional unit label (e.g. `"ms"`, `"verts"`).
    pub unit: Option<String>,
}

/// The debug overlay state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DebugOverlay {
    /// Configuration snapshot.
    pub config: DebugOverlayConfig,
    /// Named metrics in insertion order.
    metrics: Vec<DebugMetric>,
    /// FPS sample ring buffer.
    fps_history: Vec<f32>,
    /// Current write position in the FPS ring buffer.
    fps_head: usize,
    /// Number of valid entries in fps_history.
    fps_fill: usize,
    /// Whether the overlay is currently visible.
    visible: bool,
    /// Most recent frame time in milliseconds.
    last_frame_ms: f32,
}

// ── Constructors ──────────────────────────────────────────────────────────────

/// Returns a default [`DebugOverlayConfig`].
#[allow(dead_code)]
pub fn default_debug_overlay_config() -> DebugOverlayConfig {
    DebugOverlayConfig {
        fps_history_len: 60,
        visible_on_start: true,
        column_sep: ": ".to_string(),
        show_timestamp: false,
    }
}

/// Creates a new [`DebugOverlay`] with the given config.
#[allow(dead_code)]
pub fn new_debug_overlay(config: DebugOverlayConfig) -> DebugOverlay {
    let cap = config.fps_history_len.max(1);
    let visible = config.visible_on_start;
    DebugOverlay {
        fps_history: vec![0.0; cap],
        fps_head: 0,
        fps_fill: 0,
        visible,
        last_frame_ms: 0.0,
        metrics: Vec::new(),
        config,
    }
}

// ── Metric operations ─────────────────────────────────────────────────────────

/// Sets (or inserts) a metric by key.
#[allow(dead_code)]
pub fn set_metric(overlay: &mut DebugOverlay, key: &str, value: &str) {
    if let Some(m) = overlay.metrics.iter_mut().find(|m| m.key == key) {
        m.value = value.to_string();
    } else {
        overlay.metrics.push(DebugMetric {
            key: key.to_string(),
            value: value.to_string(),
            unit: None,
        });
    }
}

/// Returns a reference to a metric by key, or `None` if not found.
#[allow(dead_code)]
pub fn get_metric<'a>(overlay: &'a DebugOverlay, key: &str) -> Option<&'a DebugMetric> {
    overlay.metrics.iter().find(|m| m.key == key)
}

/// Removes a metric by key. Returns `true` if it existed.
#[allow(dead_code)]
pub fn remove_metric(overlay: &mut DebugOverlay, key: &str) -> bool {
    let before = overlay.metrics.len();
    overlay.metrics.retain(|m| m.key != key);
    overlay.metrics.len() < before
}

/// Clears all metrics.
#[allow(dead_code)]
pub fn clear_metrics(overlay: &mut DebugOverlay) {
    overlay.metrics.clear();
}

/// Returns the number of metrics currently stored.
#[allow(dead_code)]
pub fn metric_count(overlay: &DebugOverlay) -> usize {
    overlay.metrics.len()
}

// ── FPS / frame time ──────────────────────────────────────────────────────────

/// Records a new FPS sample and updates the rolling average metric.
#[allow(dead_code)]
pub fn update_fps(overlay: &mut DebugOverlay, fps: f32) {
    let cap = overlay.fps_history.len();
    overlay.fps_history[overlay.fps_head] = fps;
    overlay.fps_head = (overlay.fps_head + 1) % cap;
    if overlay.fps_fill < cap {
        overlay.fps_fill += 1;
    }
    let avg = fps_value(overlay);
    set_metric(overlay, "fps", &format!("{avg:.1}"));
}

/// Records a new frame time (milliseconds) as a metric.
#[allow(dead_code)]
pub fn update_frame_time(overlay: &mut DebugOverlay, ms: f32) {
    overlay.last_frame_ms = ms;
    set_metric(overlay, "frame_ms", &format!("{ms:.2}"));
}

/// Returns the rolling-average FPS value.
#[allow(dead_code)]
pub fn fps_value(overlay: &DebugOverlay) -> f32 {
    if overlay.fps_fill == 0 {
        return 0.0;
    }
    let sum: f32 = overlay.fps_history[..overlay.fps_fill].iter().sum();
    sum / overlay.fps_fill as f32
}

// ── Visibility ────────────────────────────────────────────────────────────────

/// Shows or hides the overlay.
#[allow(dead_code)]
pub fn set_visible(overlay: &mut DebugOverlay, visible: bool) {
    overlay.visible = visible;
}

/// Returns whether the overlay is currently visible.
#[allow(dead_code)]
pub fn is_overlay_visible(overlay: &DebugOverlay) -> bool {
    overlay.visible
}

// ── Formatting ────────────────────────────────────────────────────────────────

/// Formats all metrics as a multi-line text string.
///
/// Each line is `key<sep>value[ unit]`.
#[allow(dead_code)]
pub fn overlay_to_text(overlay: &DebugOverlay) -> String {
    let sep = &overlay.config.column_sep;
    overlay
        .metrics
        .iter()
        .map(|m| {
            if let Some(u) = &m.unit {
                format!("{}{}{} {}", m.key, sep, m.value, u)
            } else {
                format!("{}{}{}", m.key, sep, m.value)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Serialises all metrics to a compact JSON string.
#[allow(dead_code)]
pub fn overlay_to_json(overlay: &DebugOverlay) -> String {
    let fields: Vec<String> = overlay
        .metrics
        .iter()
        .map(|m| format!("\"{}\":\"{}\"", m.key, m.value))
        .collect();
    format!("{{{}}}", fields.join(","))
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_overlay() -> DebugOverlay {
        new_debug_overlay(default_debug_overlay_config())
    }

    #[test]
    fn test_default_config_visible() {
        let cfg = default_debug_overlay_config();
        assert!(cfg.visible_on_start);
    }

    #[test]
    fn test_default_config_fps_history_len() {
        let cfg = default_debug_overlay_config();
        assert_eq!(cfg.fps_history_len, 60);
    }

    #[test]
    fn test_new_overlay_empty_metrics() {
        let o = make_overlay();
        assert_eq!(metric_count(&o), 0);
    }

    #[test]
    fn test_new_overlay_visible() {
        let o = make_overlay();
        assert!(is_overlay_visible(&o));
    }

    #[test]
    fn test_set_metric_inserts() {
        let mut o = make_overlay();
        set_metric(&mut o, "verts", "1024");
        assert_eq!(metric_count(&o), 1);
    }

    #[test]
    fn test_set_metric_updates_existing() {
        let mut o = make_overlay();
        set_metric(&mut o, "verts", "512");
        set_metric(&mut o, "verts", "1024");
        assert_eq!(metric_count(&o), 1);
        assert_eq!(get_metric(&o, "verts").expect("should succeed").value, "1024");
    }

    #[test]
    fn test_get_metric_missing_returns_none() {
        let o = make_overlay();
        assert!(get_metric(&o, "missing").is_none());
    }

    #[test]
    fn test_remove_metric_returns_true() {
        let mut o = make_overlay();
        set_metric(&mut o, "x", "1");
        assert!(remove_metric(&mut o, "x"));
        assert_eq!(metric_count(&o), 0);
    }

    #[test]
    fn test_remove_metric_missing_returns_false() {
        let mut o = make_overlay();
        assert!(!remove_metric(&mut o, "ghost"));
    }

    #[test]
    fn test_clear_metrics() {
        let mut o = make_overlay();
        set_metric(&mut o, "a", "1");
        set_metric(&mut o, "b", "2");
        clear_metrics(&mut o);
        assert_eq!(metric_count(&o), 0);
    }

    #[test]
    fn test_update_fps_sets_metric() {
        let mut o = make_overlay();
        update_fps(&mut o, 60.0);
        assert!(get_metric(&o, "fps").is_some());
    }

    #[test]
    fn test_fps_rolling_average() {
        let mut o = make_overlay();
        update_fps(&mut o, 50.0);
        update_fps(&mut o, 70.0);
        let avg = fps_value(&o);
        assert!((avg - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_fps_value_empty_is_zero() {
        let o = make_overlay();
        assert_eq!(fps_value(&o), 0.0);
    }

    #[test]
    fn test_update_frame_time() {
        let mut o = make_overlay();
        update_frame_time(&mut o, 16.67);
        assert!(get_metric(&o, "frame_ms").is_some());
    }

    #[test]
    fn test_set_visible_false() {
        let mut o = make_overlay();
        set_visible(&mut o, false);
        assert!(!is_overlay_visible(&o));
    }

    #[test]
    fn test_overlay_to_text_format() {
        let mut o = make_overlay();
        set_metric(&mut o, "fps", "60.0");
        let text = overlay_to_text(&o);
        assert!(text.contains("fps"));
        assert!(text.contains("60.0"));
    }

    #[test]
    fn test_overlay_to_json_format() {
        let mut o = make_overlay();
        set_metric(&mut o, "fps", "30.0");
        let json = overlay_to_json(&o);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
        assert!(json.contains("\"fps\""));
    }

    #[test]
    fn test_overlay_to_text_empty() {
        let o = make_overlay();
        assert_eq!(overlay_to_text(&o), "");
    }

    #[test]
    fn test_fps_ring_buffer_wraps() {
        let cfg = DebugOverlayConfig {
            fps_history_len: 3,
            ..default_debug_overlay_config()
        };
        let mut o = new_debug_overlay(cfg);
        update_fps(&mut o, 10.0);
        update_fps(&mut o, 20.0);
        update_fps(&mut o, 30.0);
        update_fps(&mut o, 60.0); // overwrites oldest
                                  // fill is 3, average of last 3 seen in ring: 20, 30, 60
        let avg = fps_value(&o);
        assert!((avg - (20.0 + 30.0 + 60.0) / 3.0).abs() < 0.1);
    }
}
