// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// EEG electrode channel.
pub struct EegChannel {
    pub electrode_name: String,
    pub samples: Vec<f32>,
    pub sample_rate_hz: f32,
}

pub fn new_eeg_channel(name: &str, rate: f32) -> EegChannel {
    EegChannel {
        electrode_name: name.to_string(),
        samples: Vec::new(),
        sample_rate_hz: rate.max(1.0),
    }
}

pub fn eeg_push_sample(c: &mut EegChannel, v: f32) {
    c.samples.push(v);
}

/// Stub: returns mean of samples within [low_hz, high_hz] conceptual band.
/// Since we have no FFT, we approximate by mean absolute amplitude.
pub fn eeg_band_power(c: &EegChannel, low_hz: f32, high_hz: f32) -> f32 {
    let _ = low_hz;
    let _ = high_hz;
    let n = c.samples.len();
    if n == 0 {
        return 0.0;
    }
    c.samples.iter().map(|&v| v * v).sum::<f32>() / n as f32
}

pub fn eeg_rms(c: &EegChannel) -> f32 {
    let n = c.samples.len();
    if n == 0 {
        return 0.0;
    }
    let sum_sq: f32 = c.samples.iter().map(|&v| v * v).sum();
    (sum_sq / n as f32).sqrt()
}

pub fn eeg_to_csv(c: &EegChannel) -> String {
    let header = format!(
        "# electrode:{} rate_hz:{}\nindex,uv\n",
        c.electrode_name, c.sample_rate_hz
    );
    let rows: Vec<String> = c
        .samples
        .iter()
        .enumerate()
        .map(|(i, &v)| format!("{},{:.6}", i, v))
        .collect();
    format!("{}{}", header, rows.join("\n"))
}

pub fn eeg_to_bytes(c: &EegChannel) -> Vec<u8> {
    let mut out = Vec::with_capacity(c.samples.len() * 4);
    for &v in &c.samples {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

pub fn eeg_duration_s(c: &EegChannel) -> f32 {
    if c.sample_rate_hz < 1e-9 {
        return 0.0;
    }
    c.samples.len() as f32 / c.sample_rate_hz
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_eeg_channel() {
        let c = new_eeg_channel("Fz", 256.0);
        assert_eq!(c.electrode_name, "Fz");
        assert!((c.sample_rate_hz - 256.0).abs() < 1e-4);
    }

    #[test]
    fn test_eeg_push_and_rms() {
        let mut c = new_eeg_channel("Cz", 256.0);
        eeg_push_sample(&mut c, 2.0);
        eeg_push_sample(&mut c, -2.0);
        assert!((eeg_rms(&c) - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_eeg_band_power_stub() {
        let mut c = new_eeg_channel("Oz", 256.0);
        eeg_push_sample(&mut c, 1.0);
        eeg_push_sample(&mut c, 1.0);
        /* mean sq = 1 */
        assert!((eeg_band_power(&c, 8.0, 12.0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_eeg_to_csv_header() {
        let c = new_eeg_channel("P3", 512.0);
        let csv = eeg_to_csv(&c);
        assert!(csv.contains("P3"));
    }

    #[test]
    fn test_eeg_to_bytes_len() {
        let mut c = new_eeg_channel("T3", 256.0);
        for _ in 0..8 {
            eeg_push_sample(&mut c, 0.0);
        }
        assert_eq!(eeg_to_bytes(&c).len(), 32);
    }

    #[test]
    fn test_eeg_duration_s() {
        let mut c = new_eeg_channel("F4", 100.0);
        for _ in 0..200 {
            eeg_push_sample(&mut c, 0.0);
        }
        assert!((eeg_duration_s(&c) - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_eeg_rms_empty() {
        let c = new_eeg_channel("test", 256.0);
        assert!((eeg_rms(&c) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_eeg_min_rate_clamped() {
        let c = new_eeg_channel("test", 0.0);
        assert!(c.sample_rate_hz >= 1.0);
    }
}
