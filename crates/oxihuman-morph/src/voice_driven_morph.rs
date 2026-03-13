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

/// Process an audio frame and return morph weights (stub: zeroed).
pub fn vdm_process(vdm: &VoiceDrivenMorph, _audio_frame: &[f32]) -> Vec<f32> {
    /* Stub: returns zeroed morph weights */
    vec![0.0; vdm.morph_count]
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
}
