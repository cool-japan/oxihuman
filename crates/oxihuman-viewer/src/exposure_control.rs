// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Camera exposure control: EV, ISO, aperture, and shutter speed.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ExposureMode {
    Manual,
    AutoAverage,
    AutoSpot,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExposureSettings {
    pub mode: ExposureMode,
    pub ev_bias: f32,
    pub iso: f32,
    pub aperture: f32,
    pub shutter_speed: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExposureResult {
    pub exposure_value: f32,
    pub multiplier: f32,
}

#[allow(dead_code)]
pub fn default_exposure_settings() -> ExposureSettings {
    ExposureSettings {
        mode: ExposureMode::AutoAverage,
        ev_bias: 0.0,
        iso: 100.0,
        aperture: 5.6,
        shutter_speed: 1.0 / 125.0,
    }
}

#[allow(dead_code)]
pub fn compute_ev(iso: f32, aperture: f32, shutter: f32) -> f32 {
    if iso <= 0.0 || aperture <= 0.0 || shutter <= 0.0 {
        return 0.0;
    }
    (aperture * aperture / shutter).log2() - (iso / 100.0).log2()
}

#[allow(dead_code)]
pub fn ev_to_multiplier(ev: f32) -> f32 {
    1.0 / 2.0_f32.powf(ev)
}

#[allow(dead_code)]
pub fn evaluate_exposure(settings: &ExposureSettings) -> ExposureResult {
    let ev = compute_ev(settings.iso, settings.aperture, settings.shutter_speed) + settings.ev_bias;
    ExposureResult {
        exposure_value: ev,
        multiplier: ev_to_multiplier(ev),
    }
}

#[allow(dead_code)]
pub fn set_ev_bias(settings: &mut ExposureSettings, bias: f32) {
    settings.ev_bias = bias.clamp(-5.0, 5.0);
}

#[allow(dead_code)]
pub fn set_iso(settings: &mut ExposureSettings, iso: f32) {
    settings.iso = iso.clamp(50.0, 12800.0);
}

#[allow(dead_code)]
pub fn set_aperture(settings: &mut ExposureSettings, f_stop: f32) {
    settings.aperture = f_stop.clamp(1.0, 22.0);
}

#[allow(dead_code)]
pub fn auto_expose_average(avg_luminance: f32, key_value: f32) -> f32 {
    if avg_luminance <= 0.0 {
        return 1.0;
    }
    key_value / avg_luminance
}

#[allow(dead_code)]
pub fn exposure_to_json(settings: &ExposureSettings) -> String {
    let mode_str = match &settings.mode {
        ExposureMode::Manual => "manual",
        ExposureMode::AutoAverage => "auto_average",
        ExposureMode::AutoSpot => "auto_spot",
    };
    format!(
        r#"{{"mode":"{}","ev_bias":{},"iso":{},"aperture":{},"shutter":{}}}"#,
        mode_str, settings.ev_bias, settings.iso, settings.aperture, settings.shutter_speed
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let s = default_exposure_settings();
        assert_eq!(s.mode, ExposureMode::AutoAverage);
    }

    #[test]
    fn test_compute_ev() {
        let ev = compute_ev(100.0, 5.6, 1.0 / 125.0);
        assert!(ev > 0.0);
    }

    #[test]
    fn test_ev_zero_iso() {
        let ev = compute_ev(0.0, 5.6, 1.0 / 125.0);
        assert!(ev.abs() < 1e-6);
    }

    #[test]
    fn test_ev_to_multiplier() {
        let m = ev_to_multiplier(0.0);
        assert!((m - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ev_to_multiplier_positive() {
        let m = ev_to_multiplier(1.0);
        assert!((m - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_evaluate() {
        let s = default_exposure_settings();
        let r = evaluate_exposure(&s);
        assert!(r.multiplier > 0.0);
    }

    #[test]
    fn test_set_ev_bias() {
        let mut s = default_exposure_settings();
        set_ev_bias(&mut s, 2.0);
        assert!((s.ev_bias - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_iso_clamp() {
        let mut s = default_exposure_settings();
        set_iso(&mut s, 50000.0);
        assert!((s.iso - 12800.0).abs() < 1e-6);
    }

    #[test]
    fn test_auto_expose() {
        let m = auto_expose_average(0.5, 0.18);
        assert!((m - 0.36).abs() < 1e-4);
    }

    #[test]
    fn test_to_json() {
        let s = default_exposure_settings();
        let j = exposure_to_json(&s);
        assert!(j.contains("auto_average"));
    }
}
