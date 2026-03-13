// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Luma waveform monitor stub.

/// Channel to monitor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaveformChannel {
    Luma,
    Red,
    Green,
    Blue,
    Rgb,
}

/// A single column sample in the waveform monitor.
#[derive(Debug, Clone)]
pub struct WaveformColumn {
    pub min_val: f32,
    pub max_val: f32,
    pub avg_val: f32,
}

/// Waveform monitor view configuration.
#[derive(Debug, Clone)]
pub struct WaveformMonitorView {
    pub channel: WaveformChannel,
    pub columns: Vec<WaveformColumn>,
    pub width: usize,
    pub ire_graticule: bool,
    pub enabled: bool,
}

impl WaveformMonitorView {
    pub fn new(width: usize) -> Self {
        WaveformMonitorView {
            channel: WaveformChannel::Luma,
            columns: vec![
                WaveformColumn {
                    min_val: 0.0,
                    max_val: 0.0,
                    avg_val: 0.0
                };
                width
            ],
            width,
            ire_graticule: true,
            enabled: true,
        }
    }
}

/// Create a new waveform monitor view.
pub fn new_waveform_monitor_view(width: usize) -> WaveformMonitorView {
    WaveformMonitorView::new(width)
}

/// Update a column with new luma statistics.
pub fn wmv_update_column(wmv: &mut WaveformMonitorView, col: usize, values: &[f32]) {
    /* Stub: computes min, max, avg for the column */
    if col >= wmv.width || values.is_empty() {
        return;
    }
    let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let avg = values.iter().sum::<f32>() / values.len() as f32;
    wmv.columns[col] = WaveformColumn {
        min_val: min.max(0.0),
        max_val: max.min(1.0),
        avg_val: avg.clamp(0.0, 1.0),
    };
}

/// Set the monitored channel.
pub fn wmv_set_channel(wmv: &mut WaveformMonitorView, channel: WaveformChannel) {
    wmv.channel = channel;
}

/// Enable or disable IRE graticule.
pub fn wmv_set_ire_graticule(wmv: &mut WaveformMonitorView, enabled: bool) {
    wmv.ire_graticule = enabled;
}

/// Enable or disable.
pub fn wmv_set_enabled(wmv: &mut WaveformMonitorView, enabled: bool) {
    wmv.enabled = enabled;
}

/// Return width (column count).
pub fn wmv_width(wmv: &WaveformMonitorView) -> usize {
    wmv.width
}

/// Serialize to JSON-like string.
pub fn wmv_to_json(wmv: &WaveformMonitorView) -> String {
    let ch = match wmv.channel {
        WaveformChannel::Luma => "luma",
        WaveformChannel::Red => "red",
        WaveformChannel::Green => "green",
        WaveformChannel::Blue => "blue",
        WaveformChannel::Rgb => "rgb",
    };
    format!(
        r#"{{"channel":"{}","width":{},"ire_graticule":{},"enabled":{}}}"#,
        ch, wmv.width, wmv.ire_graticule, wmv.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_width() {
        let w = new_waveform_monitor_view(64);
        assert_eq!(wmv_width(&w), 64 /* width must match */,);
    }

    #[test]
    fn test_update_column_computes_stats() {
        let mut w = new_waveform_monitor_view(4);
        wmv_update_column(&mut w, 0, &[0.2, 0.4, 0.6, 0.8]);
        assert!((w.columns[0].avg_val - 0.5).abs() < 1e-5, /* avg must be 0.5 */);
    }

    #[test]
    fn test_update_column_min_max() {
        let mut w = new_waveform_monitor_view(4);
        wmv_update_column(&mut w, 0, &[0.1, 0.9]);
        assert!((w.columns[0].min_val - 0.1).abs() < 1e-5, /* min must be 0.1 */);
        assert!((w.columns[0].max_val - 0.9).abs() < 1e-5, /* max must be 0.9 */);
    }

    #[test]
    fn test_update_out_of_bounds_ignored() {
        let mut w = new_waveform_monitor_view(2);
        wmv_update_column(&mut w, 99, &[0.5]);
        assert!((w.columns[0].avg_val).abs() < 1e-6, /* out of bounds update must be ignored */);
    }

    #[test]
    fn test_set_channel() {
        let mut w = new_waveform_monitor_view(4);
        wmv_set_channel(&mut w, WaveformChannel::Red);
        assert_eq!(
            w.channel,
            WaveformChannel::Red, /* channel must be set */
        );
    }

    #[test]
    fn test_set_ire_graticule() {
        let mut w = new_waveform_monitor_view(4);
        wmv_set_ire_graticule(&mut w, false);
        assert!(!w.ire_graticule /* IRE graticule must be disabled */,);
    }

    #[test]
    fn test_set_enabled() {
        let mut w = new_waveform_monitor_view(4);
        wmv_set_enabled(&mut w, false);
        assert!(!w.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_channel() {
        let w = new_waveform_monitor_view(4);
        let j = wmv_to_json(&w);
        assert!(j.contains("\"channel\""), /* json must contain channel */);
    }

    #[test]
    fn test_enabled_default() {
        let w = new_waveform_monitor_view(4);
        assert!(w.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_ire_graticule_default() {
        let w = new_waveform_monitor_view(4);
        assert!(w.ire_graticule, /* IRE graticule must be on by default */);
    }
}
