// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Galvanic skin response (GSR) sample.
pub struct GsrSample {
    pub time_s: f32,
    pub conductance_us: f32,
}

pub fn new_gsr_sample(t: f32, conductance: f32) -> GsrSample {
    GsrSample {
        time_s: t,
        conductance_us: conductance.max(0.0),
    }
}

pub fn gsr_mean_conductance(samples: &[GsrSample]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().map(|s| s.conductance_us).sum::<f32>() / samples.len() as f32
}

pub fn gsr_peak_conductance(samples: &[GsrSample]) -> f32 {
    samples
        .iter()
        .map(|s| s.conductance_us)
        .fold(0.0f32, f32::max)
}

pub fn gsr_to_csv(samples: &[GsrSample]) -> String {
    let header = "time_s,conductance_us\n";
    let rows: Vec<String> = samples
        .iter()
        .map(|s| format!("{:.4},{:.6}", s.time_s, s.conductance_us))
        .collect();
    format!("{}{}", header, rows.join("\n"))
}

pub fn gsr_detect_responses(samples: &[GsrSample], threshold: f32) -> Vec<usize> {
    let mut indices = Vec::new();
    for i in 1..samples.len() {
        let delta = samples[i].conductance_us - samples[i - 1].conductance_us;
        if delta >= threshold {
            indices.push(i);
        }
    }
    indices
}

pub fn gsr_to_bytes(samples: &[GsrSample]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 8);
    for s in samples {
        out.extend_from_slice(&s.time_s.to_le_bytes());
        out.extend_from_slice(&s.conductance_us.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gsr_sample() {
        let s = new_gsr_sample(0.5, 3.0);
        assert!((s.conductance_us - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_gsr_conductance_clamped() {
        let s = new_gsr_sample(0.0, -1.0);
        assert!(s.conductance_us >= 0.0);
    }

    #[test]
    fn test_gsr_mean_conductance() {
        let samples = vec![new_gsr_sample(0.0, 2.0), new_gsr_sample(1.0, 4.0)];
        assert!((gsr_mean_conductance(&samples) - 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_gsr_peak_conductance() {
        let samples = vec![
            new_gsr_sample(0.0, 1.0),
            new_gsr_sample(1.0, 10.0),
            new_gsr_sample(2.0, 5.0),
        ];
        assert!((gsr_peak_conductance(&samples) - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_gsr_to_csv_header() {
        let csv = gsr_to_csv(&[]);
        assert!(csv.starts_with("time_s"));
    }

    #[test]
    fn test_gsr_detect_responses() {
        let samples = vec![
            new_gsr_sample(0.0, 1.0),
            new_gsr_sample(1.0, 4.0), /* jump of 3 */
            new_gsr_sample(2.0, 4.5),
        ];
        let resp = gsr_detect_responses(&samples, 2.0);
        assert_eq!(resp, vec![1]);
    }

    #[test]
    fn test_gsr_to_bytes_len() {
        let samples = vec![new_gsr_sample(0.0, 1.0), new_gsr_sample(1.0, 2.0)];
        assert_eq!(gsr_to_bytes(&samples).len(), 16);
    }
}
