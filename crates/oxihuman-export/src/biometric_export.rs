// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A single biometric data sample.
pub struct BiometricSample {
    pub time_s: f32,
    pub heart_rate_bpm: f32,
    pub spo2_percent: f32,
    pub skin_temp_c: f32,
    pub respiratory_rate: f32,
}

pub fn new_biometric_sample(t: f32, hr: f32, spo2: f32) -> BiometricSample {
    BiometricSample {
        time_s: t,
        heart_rate_bpm: hr,
        spo2_percent: spo2,
        skin_temp_c: 36.5,
        respiratory_rate: 15.0,
    }
}

pub fn biometric_to_csv_line(s: &BiometricSample) -> String {
    format!(
        "{:.4},{:.2},{:.2},{:.2},{:.2}",
        s.time_s, s.heart_rate_bpm, s.spo2_percent, s.skin_temp_c, s.respiratory_rate
    )
}

pub fn biometric_sequence_to_csv(samples: &[BiometricSample]) -> String {
    let header = "time_s,heart_rate_bpm,spo2_percent,skin_temp_c,respiratory_rate\n";
    let rows: Vec<String> = samples.iter().map(biometric_to_csv_line).collect();
    format!("{}{}", header, rows.join("\n"))
}

pub fn biometric_average_hr(samples: &[BiometricSample]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().map(|s| s.heart_rate_bpm).sum::<f32>() / samples.len() as f32
}

pub fn biometric_min_spo2(samples: &[BiometricSample]) -> f32 {
    samples
        .iter()
        .map(|s| s.spo2_percent)
        .fold(f32::INFINITY, f32::min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_biometric_sample() {
        let s = new_biometric_sample(0.0, 72.0, 98.5);
        assert!((s.heart_rate_bpm - 72.0).abs() < 1e-5);
        assert!((s.spo2_percent - 98.5).abs() < 1e-5);
    }

    #[test]
    fn test_biometric_to_csv_line_fields() {
        let s = new_biometric_sample(1.0, 80.0, 97.0);
        let line = biometric_to_csv_line(&s);
        assert!(line.contains("1.0000"));
        assert!(line.contains("80.00"));
    }

    #[test]
    fn test_biometric_sequence_to_csv_header() {
        let csv = biometric_sequence_to_csv(&[]);
        assert!(csv.starts_with("time_s"));
    }

    #[test]
    fn test_biometric_average_hr() {
        let samples = vec![
            new_biometric_sample(0.0, 60.0, 98.0),
            new_biometric_sample(1.0, 80.0, 98.0),
        ];
        assert!((biometric_average_hr(&samples) - 70.0).abs() < 1e-4);
    }

    #[test]
    fn test_biometric_min_spo2() {
        let samples = vec![
            new_biometric_sample(0.0, 70.0, 99.0),
            new_biometric_sample(1.0, 70.0, 95.0),
            new_biometric_sample(2.0, 70.0, 97.0),
        ];
        assert!((biometric_min_spo2(&samples) - 95.0).abs() < 1e-4);
    }

    #[test]
    fn test_biometric_average_hr_empty() {
        assert!((biometric_average_hr(&[]) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_biometric_sequence_csv_row_count() {
        let samples = vec![
            new_biometric_sample(0.0, 70.0, 98.0),
            new_biometric_sample(1.0, 75.0, 99.0),
        ];
        let csv = biometric_sequence_to_csv(&samples);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3); /* header + 2 rows */
    }
}
