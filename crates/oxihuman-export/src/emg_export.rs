// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// EMG muscle signal channel.
pub struct EmgChannel {
    pub muscle_name: String,
    pub samples: Vec<f32>,
    pub sample_rate_hz: f32,
}

pub fn new_emg_channel(name: &str, rate: f32) -> EmgChannel {
    EmgChannel {
        muscle_name: name.to_string(),
        samples: Vec::new(),
        sample_rate_hz: rate.max(1.0),
    }
}

pub fn emg_push_sample(c: &mut EmgChannel, v: f32) {
    c.samples.push(v);
}

pub fn emg_rms(c: &EmgChannel) -> f32 {
    let n = c.samples.len();
    if n == 0 {
        return 0.0;
    }
    let sum_sq: f32 = c.samples.iter().map(|&v| v * v).sum();
    (sum_sq / n as f32).sqrt()
}

pub fn emg_peak(c: &EmgChannel) -> f32 {
    c.samples.iter().map(|&v| v.abs()).fold(0.0f32, f32::max)
}

pub fn emg_duration_s(c: &EmgChannel) -> f32 {
    if c.sample_rate_hz < 1e-9 {
        return 0.0;
    }
    c.samples.len() as f32 / c.sample_rate_hz
}

pub fn emg_to_csv(c: &EmgChannel) -> String {
    let header = format!(
        "# muscle:{} rate_hz:{}\nindex,value\n",
        c.muscle_name, c.sample_rate_hz
    );
    let rows: Vec<String> = c
        .samples
        .iter()
        .enumerate()
        .map(|(i, &v)| format!("{},{:.6}", i, v))
        .collect();
    format!("{}{}", header, rows.join("\n"))
}

pub fn emg_to_bytes(c: &EmgChannel) -> Vec<u8> {
    let mut out = Vec::with_capacity(c.samples.len() * 4);
    for &v in &c.samples {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_emg_channel() {
        let c = new_emg_channel("bicep", 1000.0);
        assert_eq!(c.muscle_name, "bicep");
        assert!((c.sample_rate_hz - 1000.0).abs() < 1e-4);
    }

    #[test]
    fn test_emg_push_and_rms() {
        let mut c = new_emg_channel("bicep", 1000.0);
        emg_push_sample(&mut c, 1.0);
        emg_push_sample(&mut c, -1.0);
        assert!((emg_rms(&c) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_emg_peak() {
        let mut c = new_emg_channel("bicep", 1000.0);
        emg_push_sample(&mut c, 0.5);
        emg_push_sample(&mut c, -2.0);
        emg_push_sample(&mut c, 1.0);
        assert!((emg_peak(&c) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_emg_duration_s() {
        let mut c = new_emg_channel("tri", 500.0);
        for _ in 0..500 {
            emg_push_sample(&mut c, 0.0);
        }
        assert!((emg_duration_s(&c) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_emg_to_csv_header() {
        let c = new_emg_channel("delt", 100.0);
        let csv = emg_to_csv(&c);
        assert!(csv.contains("delt"));
    }

    #[test]
    fn test_emg_to_bytes_len() {
        let mut c = new_emg_channel("lat", 100.0);
        for _ in 0..10 {
            emg_push_sample(&mut c, 0.0);
        }
        assert_eq!(emg_to_bytes(&c).len(), 40);
    }

    #[test]
    fn test_emg_rms_empty() {
        let c = new_emg_channel("test", 100.0);
        assert!((emg_rms(&c) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_emg_min_rate_clamped() {
        let c = new_emg_channel("test", -100.0);
        assert!(c.sample_rate_hz >= 1.0);
    }
}
