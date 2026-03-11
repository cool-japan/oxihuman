// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Color temperature — Kelvin to linear RGB conversion.

/// Convert a color temperature in Kelvin (1000..=40000) to a linear RGB triplet.
/// Based on Tanner Helland's approximation.
#[allow(dead_code)]
pub fn kelvin_to_rgb(kelvin: f32) -> [f32; 3] {
    let t = kelvin.clamp(1000.0, 40_000.0) / 100.0;

    let r = if t <= 66.0 {
        1.0
    } else {
        let v = 329.698_73 * (t - 60.0).powf(-0.133_204_76);
        (v / 255.0).clamp(0.0, 1.0)
    };

    let g = if t <= 66.0 {
        let v = 99.470_8 * t.ln() - 161.119_57;
        (v / 255.0).clamp(0.0, 1.0)
    } else {
        let v = 288.122_17 * (t - 60.0).powf(-0.075_514_85);
        (v / 255.0).clamp(0.0, 1.0)
    };

    let b = if t >= 66.0 {
        1.0
    } else if t <= 19.0 {
        0.0
    } else {
        let v = 138.517_73 * (t - 10.0).ln() - 305.044_78;
        (v / 255.0).clamp(0.0, 1.0)
    };

    [r, g, b]
}

/// State for color temperature control.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ColorTemperatureState {
    /// Current color temperature in Kelvin.
    pub kelvin: f32,
    /// Tint offset (green-magenta axis), -1..=1.
    pub tint: f32,
}

impl Default for ColorTemperatureState {
    fn default() -> Self {
        Self {
            kelvin: 6500.0,
            tint: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_color_temperature_state() -> ColorTemperatureState {
    ColorTemperatureState::default()
}

#[allow(dead_code)]
pub fn ct_set_kelvin(state: &mut ColorTemperatureState, k: f32) {
    state.kelvin = k.clamp(1000.0, 40_000.0);
}

#[allow(dead_code)]
pub fn ct_set_tint(state: &mut ColorTemperatureState, v: f32) {
    state.tint = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn ct_reset(state: &mut ColorTemperatureState) {
    *state = ColorTemperatureState::default();
}

/// Returns the RGB white balance multipliers for current temperature.
#[allow(dead_code)]
pub fn ct_white_balance(state: &ColorTemperatureState) -> [f32; 3] {
    let [r, g, b] = kelvin_to_rgb(state.kelvin);
    // Tint shifts green vs magenta
    let g_tinted = (g + state.tint * 0.15).clamp(0.0, 1.0);
    [r, g_tinted, b]
}

/// Is the state at default (daylight 6500K, zero tint)?
#[allow(dead_code)]
pub fn ct_is_default(state: &ColorTemperatureState) -> bool {
    (state.kelvin - 6500.0).abs() < 1.0 && state.tint.abs() < 1e-4
}

#[allow(dead_code)]
pub fn ct_to_json(state: &ColorTemperatureState) -> String {
    format!(
        "{{\"kelvin\":{:.1},\"tint\":{:.4}}}",
        state.kelvin, state.tint
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daylight_roughly_white() {
        let rgb = kelvin_to_rgb(6500.0);
        // All channels should be present
        assert!(rgb[0] > 0.0 && rgb[1] > 0.0 && rgb[2] > 0.0);
    }

    #[test]
    fn warm_has_more_red() {
        let warm = kelvin_to_rgb(2700.0);
        let cool = kelvin_to_rgb(10000.0);
        assert!(warm[0] >= cool[0]);
    }

    #[test]
    fn cool_has_more_blue() {
        let warm = kelvin_to_rgb(2700.0);
        let cool = kelvin_to_rgb(10000.0);
        assert!(cool[2] >= warm[2]);
    }

    #[test]
    fn kelvin_clamps_low() {
        let mut s = new_color_temperature_state();
        ct_set_kelvin(&mut s, 0.0);
        assert!((s.kelvin - 1000.0).abs() < 1.0);
    }

    #[test]
    fn kelvin_clamps_high() {
        let mut s = new_color_temperature_state();
        ct_set_kelvin(&mut s, 100_000.0);
        assert!((s.kelvin - 40_000.0).abs() < 1.0);
    }

    #[test]
    fn tint_clamps() {
        let mut s = new_color_temperature_state();
        ct_set_tint(&mut s, 5.0);
        assert!((s.tint - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_to_default() {
        let mut s = new_color_temperature_state();
        ct_set_kelvin(&mut s, 3000.0);
        ct_reset(&mut s);
        assert!(ct_is_default(&s));
    }

    #[test]
    fn white_balance_rgb_in_range() {
        let s = new_color_temperature_state();
        let wb = ct_white_balance(&s);
        assert!(wb.iter().all(|&v| (0.0..=1.0).contains(&v)));
    }

    #[test]
    fn json_has_keys() {
        let j = ct_to_json(&new_color_temperature_state());
        assert!(j.contains("kelvin") && j.contains("tint"));
    }
}
