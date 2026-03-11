// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hourglass figure proportion morph.

/// Configuration for the hourglass morph.
#[derive(Debug, Clone)]
pub struct HourglassConfig {
    pub bust_fullness: f32,
    pub waist_cinch: f32,
    pub hip_fullness: f32,
}

impl Default for HourglassConfig {
    fn default() -> Self {
        HourglassConfig {
            bust_fullness: 0.7,
            waist_cinch: 0.8,
            hip_fullness: 0.75,
        }
    }
}

/// State for the hourglass proportion morph.
#[derive(Debug, Clone)]
pub struct HourglassProportion {
    pub intensity: f32,
    pub config: HourglassConfig,
    pub enabled: bool,
}

/// Create a new hourglass proportion morph.
pub fn new_hourglass_proportion() -> HourglassProportion {
    HourglassProportion {
        intensity: 0.0,
        config: HourglassConfig::default(),
        enabled: true,
    }
}

/// Set intensity [0, 1].
pub fn hg_set_intensity(m: &mut HourglassProportion, v: f32) {
    m.intensity = v.clamp(0.0, 1.0);
}

/// Bust fullness weight.
pub fn hg_bust(m: &HourglassProportion) -> f32 {
    m.intensity * m.config.bust_fullness
}

/// Waist cinch (narrowing) weight.
pub fn hg_waist(m: &HourglassProportion) -> f32 {
    m.intensity * m.config.waist_cinch
}

/// Hip fullness weight.
pub fn hg_hips(m: &HourglassProportion) -> f32 {
    m.intensity * m.config.hip_fullness
}

/// Waist-to-hip ratio estimate (lower = more hourglass).
pub fn hg_whr(m: &HourglassProportion) -> f32 {
    let base = 0.7_f32;
    base - 0.15 * m.intensity
}

/// Serialise to JSON.
pub fn hg_to_json(m: &HourglassProportion) -> String {
    format!(
        r#"{{"intensity":{:.3},"whr":{:.3},"enabled":{}}}"#,
        m.intensity,
        hg_whr(m),
        m.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zero() {
        let m = new_hourglass_proportion();
        assert!((m.intensity - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn clamp_intensity() {
        let mut m = new_hourglass_proportion();
        hg_set_intensity(&mut m, -1.0);
        assert!((m.intensity - 0.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn bust_at_max() {
        let mut m = new_hourglass_proportion();
        hg_set_intensity(&mut m, 1.0);
        assert!((hg_bust(&m) - m.config.bust_fullness).abs() < 1e-6 /* correct */);
    }

    #[test]
    fn waist_cinch_zero_at_zero() {
        let m = new_hourglass_proportion();
        assert!((hg_waist(&m) - 0.0).abs() < 1e-6 /* zero waist cinch */);
    }

    #[test]
    fn whr_decreases_with_intensity() {
        let mut m = new_hourglass_proportion();
        hg_set_intensity(&mut m, 0.0);
        let whr0 = hg_whr(&m);
        hg_set_intensity(&mut m, 1.0);
        let whr1 = hg_whr(&m);
        assert!(whr1 < whr0 /* more hourglass at higher intensity */);
    }

    #[test]
    fn hips_at_half_intensity() {
        let mut m = new_hourglass_proportion();
        hg_set_intensity(&mut m, 0.5);
        let h = hg_hips(&m);
        assert!(h > 0.0 && h < 1.0 /* partial */);
    }

    #[test]
    fn json_contains_whr() {
        let m = new_hourglass_proportion();
        assert!(hg_to_json(&m).contains("whr") /* json has whr */);
    }

    #[test]
    fn enabled_default() {
        let m = new_hourglass_proportion();
        assert!(m.enabled /* enabled */);
    }
}
