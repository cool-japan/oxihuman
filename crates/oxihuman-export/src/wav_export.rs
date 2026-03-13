// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! WAV PCM audio export (44-byte header + 16-bit samples).

/// WAV export configuration.
#[allow(dead_code)]
pub struct WavConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
}

impl Default for WavConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 1,
            bits_per_sample: 16,
        }
    }
}

/// A WAV file in memory.
#[allow(dead_code)]
pub struct WavFile {
    pub config: WavConfig,
    pub samples: Vec<i16>,
}

/// Build a 44-byte WAV header.
#[allow(dead_code)]
pub fn build_wav_header(config: &WavConfig, data_bytes: u32) -> Vec<u8> {
    let byte_rate =
        config.sample_rate * config.channels as u32 * (config.bits_per_sample as u32 / 8);
    let block_align = config.channels * (config.bits_per_sample / 8);
    let file_size = 36 + data_bytes;
    let mut hdr = Vec::with_capacity(44);
    hdr.extend_from_slice(b"RIFF");
    hdr.extend_from_slice(&file_size.to_le_bytes());
    hdr.extend_from_slice(b"WAVE");
    hdr.extend_from_slice(b"fmt ");
    hdr.extend_from_slice(&16u32.to_le_bytes());
    hdr.extend_from_slice(&1u16.to_le_bytes());
    hdr.extend_from_slice(&config.channels.to_le_bytes());
    hdr.extend_from_slice(&config.sample_rate.to_le_bytes());
    hdr.extend_from_slice(&byte_rate.to_le_bytes());
    hdr.extend_from_slice(&block_align.to_le_bytes());
    hdr.extend_from_slice(&config.bits_per_sample.to_le_bytes());
    hdr.extend_from_slice(b"data");
    hdr.extend_from_slice(&data_bytes.to_le_bytes());
    hdr
}

/// Encode samples to 16-bit LE bytes.
#[allow(dead_code)]
pub fn encode_samples_i16(samples: &[i16]) -> Vec<u8> {
    samples.iter().flat_map(|s| s.to_le_bytes()).collect()
}

/// Export WAV to byte buffer.
#[allow(dead_code)]
pub fn export_wav(wav: &WavFile) -> Vec<u8> {
    let data = encode_samples_i16(&wav.samples);
    let mut out = build_wav_header(&wav.config, data.len() as u32);
    out.extend_from_slice(&data);
    out
}

/// Create a silent WAV of the given duration.
#[allow(dead_code)]
pub fn silent_wav(config: WavConfig, duration_secs: f32) -> WavFile {
    let n = (config.sample_rate as f32 * duration_secs * config.channels as f32) as usize;
    WavFile {
        config,
        samples: vec![0i16; n],
    }
}

/// Generate a sine wave in a WAV file.
#[allow(dead_code)]
pub fn sine_wav(config: WavConfig, freq_hz: f32, duration_secs: f32, amplitude: f32) -> WavFile {
    let n = (config.sample_rate as f32 * duration_secs) as usize;
    let sr = config.sample_rate as f32;
    let samples: Vec<i16> = (0..n)
        .map(|i| {
            let t = i as f32 / sr;
            let v = amplitude * (2.0 * std::f32::consts::PI * freq_hz * t).sin();
            (v * i16::MAX as f32) as i16
        })
        .collect();
    WavFile { config, samples }
}

/// Duration in seconds.
#[allow(dead_code)]
pub fn wav_duration(wav: &WavFile) -> f32 {
    if wav.config.sample_rate == 0 || wav.config.channels == 0 {
        return 0.0;
    }
    wav.samples.len() as f32 / (wav.config.sample_rate as f32 * wav.config.channels as f32)
}

/// Peak sample value.
#[allow(dead_code)]
pub fn wav_peak_amplitude(wav: &WavFile) -> i16 {
    wav.samples.iter().map(|s| s.abs()).max().unwrap_or(0)
}

/// Byte size of exported WAV.
#[allow(dead_code)]
pub fn wav_export_size(wav: &WavFile) -> usize {
    44 + wav.samples.len() * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_is_44_bytes() {
        let config = WavConfig::default();
        let hdr = build_wav_header(&config, 0);
        assert_eq!(hdr.len(), 44);
    }

    #[test]
    fn header_starts_with_riff() {
        let config = WavConfig::default();
        let hdr = build_wav_header(&config, 0);
        assert_eq!(&hdr[0..4], b"RIFF");
    }

    #[test]
    fn encode_samples_byte_length() {
        let samples = vec![0i16, 100, -100, i16::MAX];
        let bytes = encode_samples_i16(&samples);
        assert_eq!(bytes.len(), 8);
    }

    #[test]
    fn export_wav_total_size() {
        let wav = silent_wav(WavConfig::default(), 0.01);
        let out = export_wav(&wav);
        assert_eq!(out.len(), wav_export_size(&wav));
    }

    #[test]
    fn silent_wav_all_zeros() {
        let wav = silent_wav(WavConfig::default(), 0.01);
        assert!(wav.samples.iter().all(|&s| s == 0));
    }

    #[test]
    fn sine_wav_nonzero() {
        let wav = sine_wav(WavConfig::default(), 440.0, 0.01, 0.5);
        assert!(wav_peak_amplitude(&wav) > 0);
    }

    #[test]
    fn wav_duration_correct() {
        let config = WavConfig::default();
        let dur = 0.5;
        let wav = silent_wav(config, dur);
        assert!((wav_duration(&wav) - dur).abs() < 0.001);
    }

    #[test]
    fn wav_export_size_formula() {
        let wav = silent_wav(WavConfig::default(), 0.01);
        assert_eq!(wav_export_size(&wav), 44 + wav.samples.len() * 2);
    }

    #[test]
    fn wave_contains_wave_marker() {
        let config = WavConfig::default();
        let hdr = build_wav_header(&config, 0);
        assert_eq!(&hdr[8..12], b"WAVE");
    }

    #[test]
    fn default_config_correct() {
        let c = WavConfig::default();
        assert_eq!(c.sample_rate, 44100);
        assert_eq!(c.channels, 1);
        assert_eq!(c.bits_per_sample, 16);
    }
}
