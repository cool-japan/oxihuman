// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Audio waveform oscilloscope view stub.

/// Trigger mode for oscilloscope.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerMode {
    Auto,
    Normal,
    Single,
}

/// Oscilloscope view configuration.
#[derive(Debug, Clone)]
pub struct OscilloscopeView {
    pub trigger_mode: TriggerMode,
    pub trigger_level: f32,
    pub time_div: f32,
    pub volt_div: f32,
    pub waveform: Vec<f32>,
    pub enabled: bool,
}

impl OscilloscopeView {
    pub fn new(buffer_size: usize) -> Self {
        OscilloscopeView {
            trigger_mode: TriggerMode::Auto,
            trigger_level: 0.0,
            time_div: 1.0,
            volt_div: 1.0,
            waveform: vec![0.0; buffer_size],
            enabled: true,
        }
    }
}

/// Create a new oscilloscope view.
pub fn new_oscilloscope_view(buffer_size: usize) -> OscilloscopeView {
    OscilloscopeView::new(buffer_size)
}

/// Feed a new audio sample into the waveform buffer.
pub fn osv_push_sample(osv: &mut OscilloscopeView, sample: f32) {
    /* Stub: shifts the buffer left and appends new sample */
    let n = osv.waveform.len();
    if n == 0 {
        return;
    }
    osv.waveform.rotate_left(1);
    osv.waveform[n - 1] = sample;
}

/// Return peak amplitude of the current waveform.
pub fn osv_peak(osv: &OscilloscopeView) -> f32 {
    osv.waveform
        .iter()
        .cloned()
        .fold(0.0_f32, |a, b| a.max(b.abs()))
}

/// Set trigger mode.
pub fn osv_set_trigger(osv: &mut OscilloscopeView, mode: TriggerMode) {
    osv.trigger_mode = mode;
}

/// Set time division.
pub fn osv_set_time_div(osv: &mut OscilloscopeView, div: f32) {
    osv.time_div = div.max(1e-6);
}

/// Enable or disable.
pub fn osv_set_enabled(osv: &mut OscilloscopeView, enabled: bool) {
    osv.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn osv_to_json(osv: &OscilloscopeView) -> String {
    let trig = match osv.trigger_mode {
        TriggerMode::Auto => "auto",
        TriggerMode::Normal => "normal",
        TriggerMode::Single => "single",
    };
    format!(
        r#"{{"trigger":"{}","buffer_size":{},"time_div":{},"enabled":{}}}"#,
        trig,
        osv.waveform.len(),
        osv.time_div,
        osv.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_waveform_zero() {
        let o = new_oscilloscope_view(64);
        assert!(o.waveform.iter().all(|&v| v.abs() < 1e-6), /* initial waveform must be zero */);
    }

    #[test]
    fn test_push_sample() {
        let mut o = new_oscilloscope_view(4);
        osv_push_sample(&mut o, 1.0);
        assert!((o.waveform[3] - 1.0).abs() < 1e-5, /* last element must be the new sample */);
    }

    #[test]
    fn test_peak_after_push() {
        let mut o = new_oscilloscope_view(4);
        osv_push_sample(&mut o, 0.7);
        assert!((osv_peak(&o) - 0.7).abs() < 1e-5, /* peak must be the pushed value */);
    }

    #[test]
    fn test_peak_initial_zero() {
        let o = new_oscilloscope_view(8);
        assert!((osv_peak(&o)).abs() < 1e-6, /* peak of zero waveform must be 0 */);
    }

    #[test]
    fn test_set_trigger() {
        let mut o = new_oscilloscope_view(8);
        osv_set_trigger(&mut o, TriggerMode::Normal);
        assert_eq!(
            o.trigger_mode,
            TriggerMode::Normal, /* trigger mode must be set */
        );
    }

    #[test]
    fn test_set_time_div() {
        let mut o = new_oscilloscope_view(8);
        osv_set_time_div(&mut o, 2.0);
        assert!((o.time_div - 2.0).abs() < 1e-5, /* time div must be set */);
    }

    #[test]
    fn test_time_div_clamped() {
        let mut o = new_oscilloscope_view(8);
        osv_set_time_div(&mut o, -1.0);
        assert!(o.time_div > 0.0 /* time div must be positive */,);
    }

    #[test]
    fn test_set_enabled() {
        let mut o = new_oscilloscope_view(8);
        osv_set_enabled(&mut o, false);
        assert!(!o.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_trigger() {
        let o = new_oscilloscope_view(16);
        let j = osv_to_json(&o);
        assert!(j.contains("\"trigger\""), /* json must contain trigger */);
    }

    #[test]
    fn test_enabled_default() {
        let o = new_oscilloscope_view(8);
        assert!(o.enabled /* must be enabled by default */,);
    }
}
