// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! WAV/PCM audio stub export.

/// WAV export configuration.
#[derive(Debug, Clone)]
pub struct WavExportConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
}

impl Default for WavExportConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            bits_per_sample: 16,
        }
    }
}

/// WAV export result.
#[derive(Debug, Clone, Default)]
pub struct WavExportResult {
    pub bytes: Vec<u8>,
    pub sample_count: usize,
    pub duration_secs: f64,
}

/// Write a 4-byte little-endian u32 into a byte vec.
fn write_u32_le(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// Write a 2-byte little-endian u16 into a byte vec.
fn write_u16_le(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// Build a minimal WAV file header for a given PCM payload size.
pub fn build_wav_header(cfg: &WavExportConfig, pcm_data_len: usize) -> Vec<u8> {
    /* Standard 44-byte WAV header */
    let mut buf = Vec::with_capacity(44 + pcm_data_len);
    let byte_rate = cfg.sample_rate * cfg.channels as u32 * cfg.bits_per_sample as u32 / 8;
    let block_align = cfg.channels * cfg.bits_per_sample / 8;
    let data_chunk_size = pcm_data_len as u32;
    let riff_size = 36 + data_chunk_size;

    buf.extend_from_slice(b"RIFF");
    write_u32_le(&mut buf, riff_size);
    buf.extend_from_slice(b"WAVE");
    buf.extend_from_slice(b"fmt ");
    write_u32_le(&mut buf, 16); /* PCM fmt chunk size */
    write_u16_le(&mut buf, 1); /* PCM format */
    write_u16_le(&mut buf, cfg.channels);
    write_u32_le(&mut buf, cfg.sample_rate);
    write_u32_le(&mut buf, byte_rate);
    write_u16_le(&mut buf, block_align);
    write_u16_le(&mut buf, cfg.bits_per_sample);
    buf.extend_from_slice(b"data");
    write_u32_le(&mut buf, data_chunk_size);
    buf
}

/// Convert f32 PCM samples in [-1, 1] to i16 PCM bytes.
pub fn f32_to_i16_pcm(samples: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let i16_val = (clamped * i16::MAX as f32) as i16;
        out.extend_from_slice(&i16_val.to_le_bytes());
    }
    out
}

/// Export f32 PCM samples as a WAV file byte stream.
pub fn export_wav_pcm(samples: &[f32], cfg: &WavExportConfig) -> WavExportResult {
    /* Build header + PCM body */
    let pcm_bytes = f32_to_i16_pcm(samples);
    let mut header = build_wav_header(cfg, pcm_bytes.len());
    header.extend_from_slice(&pcm_bytes);
    let sample_count = samples.len();
    let duration_secs = sample_count as f64 / (cfg.sample_rate as f64 * cfg.channels as f64);
    WavExportResult {
        bytes: header,
        sample_count,
        duration_secs,
    }
}

/// Generate a silent WAV of a given duration.
pub fn export_silent_wav(duration_secs: f64, cfg: &WavExportConfig) -> WavExportResult {
    let n = (duration_secs * cfg.sample_rate as f64 * cfg.channels as f64) as usize;
    let samples = vec![0.0f32; n];
    export_wav_pcm(&samples, cfg)
}

/// Validate that a WAV byte stream starts with the RIFF header.
pub fn validate_wav_header(data: &[u8]) -> bool {
    data.len() >= 44 && &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = WavExportConfig::default();
        assert_eq!(cfg.sample_rate, 44100 /* standard rate */);
        assert_eq!(cfg.channels, 2 /* stereo */);
    }

    #[test]
    fn test_wav_header_length() {
        let cfg = WavExportConfig::default();
        let header = build_wav_header(&cfg, 0);
        assert_eq!(header.len(), 44 /* standard WAV header is 44 bytes */);
    }

    #[test]
    fn test_wav_header_riff_magic() {
        let cfg = WavExportConfig::default();
        let header = build_wav_header(&cfg, 100);
        assert_eq!(&header[0..4], b"RIFF" /* RIFF magic */);
    }

    #[test]
    fn test_f32_to_i16_silence() {
        let pcm = f32_to_i16_pcm(&[0.0f32; 4]);
        assert!(pcm.iter().all(|&b| b == 0) /* silence → zero bytes */);
    }

    #[test]
    fn test_f32_to_i16_length() {
        let pcm = f32_to_i16_pcm(&[0.5f32; 3]);
        assert_eq!(pcm.len(), 6 /* 3 samples × 2 bytes */);
    }

    #[test]
    fn test_export_wav_validates() {
        let cfg = WavExportConfig::default();
        let result = export_wav_pcm(&[0.0f32; 100], &cfg);
        assert!(validate_wav_header(&result.bytes) /* valid WAV header */);
    }

    #[test]
    fn test_export_wav_sample_count() {
        let cfg = WavExportConfig::default();
        let result = export_wav_pcm(&[0.0f32; 50], &cfg);
        assert_eq!(result.sample_count, 50 /* correct sample count */);
    }

    #[test]
    fn test_export_silent_wav() {
        let cfg = WavExportConfig {
            sample_rate: 8000,
            channels: 1,
            bits_per_sample: 16,
        };
        let result = export_silent_wav(1.0, &cfg);
        assert!(validate_wav_header(&result.bytes) /* silent WAV is valid */);
    }

    #[test]
    fn test_validate_wav_header_rejects_short() {
        assert!(!validate_wav_header(&[0u8; 10]) /* too short */);
    }
}
