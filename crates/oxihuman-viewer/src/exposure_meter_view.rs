// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Exposure metering visualization stub.

/// Metering mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MeteringMode {
    Average,
    CenterWeighted,
    Spot,
}

/// Exposure meter view config.
#[derive(Debug, Clone)]
pub struct ExposureMeterViewConfig {
    pub mode: MeteringMode,
    pub ev_target: f32,
    pub enabled: bool,
    pub show_histogram: bool,
}

impl Default for ExposureMeterViewConfig {
    fn default() -> Self {
        ExposureMeterViewConfig {
            mode: MeteringMode::Average,
            ev_target: 0.0,
            enabled: true,
            show_histogram: false,
        }
    }
}

/// Create a new exposure meter view config.
pub fn new_exposure_meter_view() -> ExposureMeterViewConfig {
    ExposureMeterViewConfig::default()
}

/// Set the metering mode.
pub fn emv2_set_mode(cfg: &mut ExposureMeterViewConfig, mode: MeteringMode) {
    cfg.mode = mode;
}

/// Set EV target.
pub fn emv2_set_ev_target(cfg: &mut ExposureMeterViewConfig, ev: f32) {
    cfg.ev_target = ev.clamp(-10.0, 10.0);
}

/// Enable or disable.
pub fn emv2_set_enabled(cfg: &mut ExposureMeterViewConfig, enabled: bool) {
    cfg.enabled = enabled;
}

/// Toggle histogram display.
pub fn emv2_toggle_histogram(cfg: &mut ExposureMeterViewConfig) {
    cfg.show_histogram = !cfg.show_histogram;
}

/// Compute EV from average luminance (stub).
pub fn emv2_luminance_to_ev(avg_luminance: f32) -> f32 {
    if avg_luminance <= 0.0 {
        return -10.0;
    }
    avg_luminance.log2()
}

/// Return exposure correction delta.
pub fn emv2_correction(cfg: &ExposureMeterViewConfig, measured_ev: f32) -> f32 {
    cfg.ev_target - measured_ev
}

/// Return a JSON-like string.
pub fn emv2_to_json(cfg: &ExposureMeterViewConfig) -> String {
    format!(
        r#"{{"mode":"{}","ev_target":{:.4},"enabled":{}}}"#,
        match cfg.mode {
            MeteringMode::Average => "average",
            MeteringMode::CenterWeighted => "center_weighted",
            MeteringMode::Spot => "spot",
        },
        cfg.ev_target,
        cfg.enabled
    )
}

/// Return the mode name.
pub fn emv2_mode_name(cfg: &ExposureMeterViewConfig) -> &'static str {
    match cfg.mode {
        MeteringMode::Average => "average",
        MeteringMode::CenterWeighted => "center_weighted",
        MeteringMode::Spot => "spot",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode_average() {
        let c = new_exposure_meter_view();
        assert_eq!(
            c.mode,
            MeteringMode::Average, /* default mode is Average */
        );
    }

    #[test]
    fn test_set_mode_spot() {
        let mut c = new_exposure_meter_view();
        emv2_set_mode(&mut c, MeteringMode::Spot);
        assert_eq!(c.mode, MeteringMode::Spot /* mode must be Spot */,);
    }

    #[test]
    fn test_set_ev_target() {
        let mut c = new_exposure_meter_view();
        emv2_set_ev_target(&mut c, 1.5);
        assert!((c.ev_target - 1.5).abs() < 1e-5, /* EV target must match */);
    }

    #[test]
    fn test_set_ev_target_clamps() {
        let mut c = new_exposure_meter_view();
        emv2_set_ev_target(&mut c, 20.0);
        assert!((c.ev_target - 10.0).abs() < 1e-5, /* EV clamped to 10 */);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_exposure_meter_view();
        emv2_set_enabled(&mut c, false);
        assert!(!c.enabled /* should be disabled */,);
    }

    #[test]
    fn test_toggle_histogram() {
        let mut c = new_exposure_meter_view();
        emv2_toggle_histogram(&mut c);
        assert!(c.show_histogram /* histogram toggled on */,);
    }

    #[test]
    fn test_luminance_to_ev_zero_luminance() {
        let ev = emv2_luminance_to_ev(0.0);
        assert!((ev - (-10.0)).abs() < 1e-5, /* zero luminance returns -10 EV */);
    }

    #[test]
    fn test_luminance_to_ev_one() {
        let ev = emv2_luminance_to_ev(1.0);
        assert!((ev).abs() < 1e-5 /* luminance=1 gives EV=0 */,);
    }

    #[test]
    fn test_correction_positive() {
        let mut c = new_exposure_meter_view();
        emv2_set_ev_target(&mut c, 2.0);
        let corr = emv2_correction(&c, 1.0);
        assert!((corr - 1.0).abs() < 1e-5, /* correction is target - measured */);
    }

    #[test]
    fn test_to_json_contains_mode() {
        let c = new_exposure_meter_view();
        let j = emv2_to_json(&c);
        assert!(j.contains("mode") /* JSON must contain mode */,);
    }
}
