// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Android body proportion morph — weight centred upper body/belly.

/// Configuration for the android proportion morph.
#[derive(Debug, Clone)]
pub struct AndroidProportionConfig {
    pub upper_body_mass: f32,
    pub belly_protrusion: f32,
    pub hip_narrowness: f32,
}

impl Default for AndroidProportionConfig {
    fn default() -> Self {
        AndroidProportionConfig {
            upper_body_mass: 0.75,
            belly_protrusion: 0.65,
            hip_narrowness: 0.55,
        }
    }
}

/// State for the android proportion morph.
#[derive(Debug, Clone)]
pub struct AndroidProportion {
    pub intensity: f32,
    pub config: AndroidProportionConfig,
    pub enabled: bool,
}

/// Create a new android proportion morph.
pub fn new_android_proportion() -> AndroidProportion {
    AndroidProportion {
        intensity: 0.0,
        config: AndroidProportionConfig::default(),
        enabled: true,
    }
}

/// Set intensity [0, 1].
pub fn andr_set_intensity(m: &mut AndroidProportion, v: f32) {
    m.intensity = v.clamp(0.0, 1.0);
}

/// Upper body mass weight.
pub fn andr_upper_mass(m: &AndroidProportion) -> f32 {
    m.intensity * m.config.upper_body_mass
}

/// Belly protrusion weight.
pub fn andr_belly(m: &AndroidProportion) -> f32 {
    m.intensity * m.config.belly_protrusion
}

/// Hip narrowness weight.
pub fn andr_hip_narrow(m: &AndroidProportion) -> f32 {
    m.intensity * m.config.hip_narrowness
}

/// Serialise to JSON.
pub fn andr_to_json(m: &AndroidProportion) -> String {
    format!(
        r#"{{"intensity":{:.3},"belly":{:.3},"enabled":{}}}"#,
        m.intensity,
        andr_belly(m),
        m.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zero_intensity() {
        let m = new_android_proportion();
        assert!((m.intensity - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn clamp_intensity() {
        let mut m = new_android_proportion();
        andr_set_intensity(&mut m, -1.0);
        assert!((m.intensity - 0.0).abs() < 1e-6 /* clamped to 0 */);
    }

    #[test]
    fn upper_mass_proportional() {
        let mut m = new_android_proportion();
        andr_set_intensity(&mut m, 1.0);
        assert!((andr_upper_mass(&m) - m.config.upper_body_mass).abs() < 1e-6 /* correct */);
    }

    #[test]
    fn belly_increases_with_intensity() {
        let mut m = new_android_proportion();
        andr_set_intensity(&mut m, 0.5);
        let b5 = andr_belly(&m);
        andr_set_intensity(&mut m, 1.0);
        let b10 = andr_belly(&m);
        assert!(b10 > b5 /* larger belly at higher intensity */);
    }

    #[test]
    fn hip_narrow_zero_at_zero() {
        let m = new_android_proportion();
        assert!((andr_hip_narrow(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn json_has_belly() {
        let mut m = new_android_proportion();
        andr_set_intensity(&mut m, 0.8);
        assert!(andr_to_json(&m).contains("belly") /* json has field */);
    }

    #[test]
    fn enabled_default() {
        let m = new_android_proportion();
        assert!(m.enabled /* enabled by default */);
    }

    #[test]
    fn config_upper_body_mass_positive() {
        let m = new_android_proportion();
        assert!(m.config.upper_body_mass > 0.0 /* valid */);
    }
}
