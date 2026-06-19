// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Audio-driven morph target stub.

/// A mapping from an audio frequency band to a morph weight.
#[derive(Debug, Clone)]
pub struct AudioBandMapping {
    pub band_hz_low: f32,
    pub band_hz_high: f32,
    pub morph_index: usize,
    pub gain: f32,
}

/// Voice-driven morph system.
#[derive(Debug, Clone)]
pub struct VoiceDrivenMorph {
    pub mappings: Vec<AudioBandMapping>,
    pub sample_rate: u32,
    pub morph_count: usize,
    pub enabled: bool,
    pub smoothing: f32,
}

impl VoiceDrivenMorph {
    pub fn new(morph_count: usize, sample_rate: u32) -> Self {
        VoiceDrivenMorph {
            mappings: Vec::new(),
            sample_rate,
            morph_count,
            enabled: true,
            smoothing: 0.1,
        }
    }
}

/// Create a new voice-driven morph system.
pub fn new_voice_driven_morph(morph_count: usize, sample_rate: u32) -> VoiceDrivenMorph {
    VoiceDrivenMorph::new(morph_count, sample_rate)
}

/// Add a frequency-band-to-morph mapping.
pub fn vdm_add_mapping(vdm: &mut VoiceDrivenMorph, mapping: AudioBandMapping) {
    vdm.mappings.push(mapping);
}

/// Process an audio frame and return morph weights via Goertzel-based band energy.
///
/// For each [`AudioBandMapping`] the function computes a DFT energy estimate
/// at the band centre frequency using the Goertzel algorithm (single-bin DFT).
/// The resulting power is square-rooted and multiplied by the mapping's
/// `gain` to produce a raw morph weight, which is then attenuated by
/// `(1 - smoothing)` as a single-frame IIR approximation.  All morph slots
/// not referenced by a mapping stay at `0.0`.
pub fn vdm_process(vdm: &VoiceDrivenMorph, audio_frame: &[f32]) -> Vec<f32> {
    let mut weights = vec![0.0_f32; vdm.morph_count];

    if !vdm.enabled || audio_frame.is_empty() {
        return weights;
    }

    let n = audio_frame.len();
    let sample_rate = vdm.sample_rate as f32;
    let n_f = n as f32;

    for mapping in &vdm.mappings {
        if mapping.morph_index >= vdm.morph_count {
            continue;
        }

        // Band-centre frequency and the corresponding Goertzel coefficient.
        let f_c = (mapping.band_hz_low + mapping.band_hz_high) * 0.5_f32;

        // Normalised angular frequency for the Goertzel algorithm.
        let omega = 2.0_f32 * std::f32::consts::PI * f_c / sample_rate;
        let coeff = 2.0_f32 * omega.cos();

        // Single-bin Goertzel recursion: compute s[n] = x[n] + coeff*s[n-1] - s[n-2].
        let mut s_prev2 = 0.0_f32;
        let mut s_prev1 = 0.0_f32;
        for &sample in audio_frame {
            let s = sample + coeff * s_prev1 - s_prev2;
            s_prev2 = s_prev1;
            s_prev1 = s;
        }

        // Power = s[N-1]^2 + s[N-2]^2 - coeff * s[N-1] * s[N-2], normalised by N.
        // This is the standard Goertzel magnitude-squared formula.
        let power = (s_prev1 * s_prev1 + s_prev2 * s_prev2 - coeff * s_prev1 * s_prev2) / n_f;
        let power = power.max(0.0_f32);

        // Map power to a morph weight in [0, 1] and apply IIR gain reduction.
        let raw_weight = (power.sqrt() * mapping.gain).clamp(0.0_f32, 1.0_f32);
        let smoothed = raw_weight * (1.0_f32 - vdm.smoothing);

        // Accumulate — multiple mappings may target the same morph index.
        weights[mapping.morph_index] = (weights[mapping.morph_index] + smoothed).clamp(0.0, 1.0);
    }

    weights
}

/// Set smoothing factor (0 = no smoothing, 1 = full hold).
pub fn vdm_set_smoothing(vdm: &mut VoiceDrivenMorph, smoothing: f32) {
    vdm.smoothing = smoothing.clamp(0.0, 1.0);
}

/// Enable or disable.
pub fn vdm_set_enabled(vdm: &mut VoiceDrivenMorph, enabled: bool) {
    vdm.enabled = enabled;
}

/// Return mapping count.
pub fn vdm_mapping_count(vdm: &VoiceDrivenMorph) -> usize {
    vdm.mappings.len()
}

/// Serialize to JSON-like string.
pub fn vdm_to_json(vdm: &VoiceDrivenMorph) -> String {
    format!(
        r#"{{"morph_count":{},"sample_rate":{},"mappings":{},"smoothing":{},"enabled":{}}}"#,
        vdm.morph_count,
        vdm.sample_rate,
        vdm.mappings.len(),
        vdm.smoothing,
        vdm.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_count() {
        let v = new_voice_driven_morph(8, 44100);
        assert_eq!(v.morph_count, 8 /* morph count must match */,);
    }

    #[test]
    fn test_sample_rate_stored() {
        let v = new_voice_driven_morph(4, 48000);
        assert_eq!(v.sample_rate, 48000 /* sample rate must match */,);
    }

    #[test]
    fn test_process_output_length() {
        let v = new_voice_driven_morph(6, 44100);
        let out = vdm_process(&v, &[0.0; 512]);
        assert_eq!(out.len(), 6 /* output length must match morph_count */,);
    }

    #[test]
    fn test_add_mapping() {
        let mut v = new_voice_driven_morph(4, 44100);
        vdm_add_mapping(
            &mut v,
            AudioBandMapping {
                band_hz_low: 80.0,
                band_hz_high: 200.0,
                morph_index: 0,
                gain: 1.0,
            },
        );
        assert_eq!(vdm_mapping_count(&v), 1 /* one mapping after add */,);
    }

    #[test]
    fn test_smoothing_clamped() {
        let mut v = new_voice_driven_morph(4, 44100);
        vdm_set_smoothing(&mut v, 2.0);
        assert!((v.smoothing - 1.0).abs() < 1e-6, /* smoothing clamped to 1.0 */);
    }

    #[test]
    fn test_smoothing_clamped_low() {
        let mut v = new_voice_driven_morph(4, 44100);
        vdm_set_smoothing(&mut v, -1.0);
        assert!((v.smoothing).abs() < 1e-6, /* smoothing clamped to 0.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_voice_driven_morph(2, 44100);
        vdm_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_morph_count() {
        let v = new_voice_driven_morph(5, 22050);
        let j = vdm_to_json(&v);
        assert!(j.contains("\"morph_count\""), /* json must contain morph_count */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_voice_driven_morph(1, 44100);
        assert!(v.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_default_smoothing() {
        let v = new_voice_driven_morph(1, 44100);
        assert!((v.smoothing - 0.1).abs() < 1e-5, /* default smoothing must be 0.1 */);
    }

    #[test]
    fn vdm_process_nonzero_signal_nonzero_output() {
        // Generate a 440 Hz sinusoidal frame at 44100 Hz sample rate.
        let sample_rate: u32 = 44100;
        let freq = 440.0_f32;
        let frame_len = 1024_usize;
        let audio_frame: Vec<f32> = (0..frame_len)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate as f32).sin())
            .collect();

        let mut v = new_voice_driven_morph(4, sample_rate);
        // Map the 400–480 Hz band (which contains 440 Hz) to morph index 0 with high gain.
        vdm_add_mapping(
            &mut v,
            AudioBandMapping {
                band_hz_low: 400.0,
                band_hz_high: 480.0,
                morph_index: 0,
                gain: 10.0,
            },
        );

        let weights = vdm_process(&v, &audio_frame);
        assert_eq!(weights.len(), 4);
        assert!(
            weights[0] > 0.0,
            "expected non-zero morph weight for 440 Hz signal in 400–480 Hz band, got {}",
            weights[0]
        );
    }
}
