// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hair wave and curl pattern morph control.

use std::f32::consts::PI;

/// Hair curl pattern type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurlPattern {
    Straight,
    Wavy,
    Curly,
    Coily,
}

/// Hair wave morph configuration.
#[derive(Debug, Clone)]
pub struct HairWaveMorph {
    pub pattern: CurlPattern,
    pub amplitude: f32,
    pub frequency: f32,
    pub tightness: f32,
}

impl HairWaveMorph {
    pub fn new() -> Self {
        Self {
            pattern: CurlPattern::Straight,
            amplitude: 0.0,
            frequency: 1.0,
            tightness: 0.0,
        }
    }
}

impl Default for HairWaveMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new hair wave morph.
pub fn new_hair_wave_morph() -> HairWaveMorph {
    HairWaveMorph::new()
}

/// Set wave amplitude in normalized range.
pub fn hair_wave_set_amplitude(morph: &mut HairWaveMorph, amplitude: f32) {
    morph.amplitude = amplitude.clamp(0.0, 1.0);
}

/// Set spatial frequency of the wave pattern (cycles per unit length).
pub fn hair_wave_set_frequency(morph: &mut HairWaveMorph, frequency: f32) {
    morph.frequency = frequency.clamp(0.1, 10.0);
}

/// Set curl tightness in normalized range.
pub fn hair_wave_set_tightness(morph: &mut HairWaveMorph, tightness: f32) {
    morph.tightness = tightness.clamp(0.0, 1.0);
}

/// Evaluate wave displacement at normalized position t along strand.
pub fn hair_wave_displacement_at(morph: &HairWaveMorph, t: f32) -> f32 {
    morph.amplitude * (morph.frequency * t * PI * 2.0).sin()
}

/// Serialize to JSON-like string.
pub fn hair_wave_morph_to_json(morph: &HairWaveMorph) -> String {
    let pattern_str = match morph.pattern {
        CurlPattern::Straight => "straight",
        CurlPattern::Wavy => "wavy",
        CurlPattern::Curly => "curly",
        CurlPattern::Coily => "coily",
    };
    format!(
        r#"{{"pattern":"{pattern_str}","amplitude":{:.4},"frequency":{:.4},"tightness":{:.4}}}"#,
        morph.amplitude, morph.frequency, morph.tightness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_hair_wave_morph();
        assert_eq!(m.pattern, CurlPattern::Straight);
        assert_eq!(m.amplitude, 0.0);
    }

    #[test]
    fn test_amplitude_clamp() {
        let mut m = new_hair_wave_morph();
        hair_wave_set_amplitude(&mut m, 5.0);
        assert_eq!(m.amplitude, 1.0);
    }

    #[test]
    fn test_frequency_clamp_low() {
        let mut m = new_hair_wave_morph();
        hair_wave_set_frequency(&mut m, 0.0);
        assert!((m.frequency - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_tightness() {
        let mut m = new_hair_wave_morph();
        hair_wave_set_tightness(&mut m, 0.6);
        assert!((m.tightness - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_displacement_zero_amplitude() {
        let m = new_hair_wave_morph();
        assert_eq!(hair_wave_displacement_at(&m, 0.5), 0.0);
    }

    #[test]
    fn test_json_contains_pattern() {
        let m = new_hair_wave_morph();
        let s = hair_wave_morph_to_json(&m);
        assert!(s.contains("straight"));
    }

    #[test]
    fn test_clone() {
        let m = new_hair_wave_morph();
        let m2 = m.clone();
        assert_eq!(m2.pattern, m.pattern);
    }

    #[test]
    fn test_displacement_nonzero() {
        let mut m = new_hair_wave_morph();
        hair_wave_set_amplitude(&mut m, 1.0);
        hair_wave_set_frequency(&mut m, 1.0);
        let d = hair_wave_displacement_at(&m, 0.25);
        assert!(d.abs() > 0.9);
    }

    #[test]
    fn test_default_trait() {
        let m: HairWaveMorph = Default::default();
        assert!((m.frequency - 1.0).abs() < 1e-6);
    }
}
