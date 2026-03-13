// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hair length export: per-strand length data.

/// Per-strand hair length record.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairLengthExport {
    pub lengths: Vec<f32>,
}

/// Create a new hair length export from per-strand lengths.
#[allow(dead_code)]
pub fn new_hair_length_export(lengths: &[f32]) -> HairLengthExport {
    HairLengthExport {
        lengths: lengths.iter().map(|&l| l.max(0.0)).collect(),
    }
}

/// Strand count.
#[allow(dead_code)]
pub fn hair_strand_count(e: &HairLengthExport) -> usize {
    e.lengths.len()
}

/// Average strand length.
#[allow(dead_code)]
pub fn avg_hair_length(e: &HairLengthExport) -> f32 {
    if e.lengths.is_empty() {
        return 0.0;
    }
    e.lengths.iter().sum::<f32>() / e.lengths.len() as f32
}

/// Maximum strand length.
#[allow(dead_code)]
pub fn max_hair_length(e: &HairLengthExport) -> f32 {
    e.lengths.iter().cloned().fold(0.0_f32, f32::max)
}

/// Minimum strand length.
#[allow(dead_code)]
pub fn min_hair_length(e: &HairLengthExport) -> f32 {
    if e.lengths.is_empty() {
        return 0.0;
    }
    e.lengths.iter().cloned().fold(f32::MAX, f32::min)
}

/// Count strands above a threshold length.
#[allow(dead_code)]
pub fn count_long_strands(e: &HairLengthExport, threshold: f32) -> usize {
    e.lengths.iter().filter(|&&l| l > threshold).count()
}

/// Scale all lengths by a factor.
#[allow(dead_code)]
pub fn scale_hair_lengths(e: &mut HairLengthExport, factor: f32) {
    for l in &mut e.lengths {
        *l *= factor.max(0.0);
    }
}

/// Validate: all lengths non-negative.
#[allow(dead_code)]
pub fn validate_hair_lengths(e: &HairLengthExport) -> bool {
    e.lengths.iter().all(|&l| l >= 0.0)
}

/// Export to CSV.
#[allow(dead_code)]
pub fn hair_length_to_csv(e: &HairLengthExport) -> String {
    let mut s = "strand,length\n".to_string();
    for (i, &l) in e.lengths.iter().enumerate() {
        s.push_str(&format!("{},{:.6}\n", i, l));
    }
    s
}

/// Export to JSON.
#[allow(dead_code)]
pub fn hair_length_to_json(e: &HairLengthExport) -> String {
    format!(
        "{{\"strand_count\":{},\"avg_length\":{:.6}}}",
        hair_strand_count(e),
        avg_hair_length(e)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hair_length_export() {
        let e = new_hair_length_export(&[1.0, 2.0, 3.0]);
        assert_eq!(hair_strand_count(&e), 3);
    }

    #[test]
    fn test_negative_clamped() {
        let e = new_hair_length_export(&[-1.0]);
        assert!((e.lengths[0]).abs() < 1e-9);
    }

    #[test]
    fn test_avg_hair_length() {
        let e = new_hair_length_export(&[1.0, 3.0]);
        assert!((avg_hair_length(&e) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_hair_length() {
        let e = new_hair_length_export(&[1.0, 5.0, 3.0]);
        assert!((max_hair_length(&e) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_min_hair_length() {
        let e = new_hair_length_export(&[1.0, 5.0, 3.0]);
        assert!((min_hair_length(&e) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_count_long_strands() {
        let e = new_hair_length_export(&[0.5, 1.5, 2.5]);
        assert_eq!(count_long_strands(&e, 1.0), 2);
    }

    #[test]
    fn test_scale_hair_lengths() {
        let mut e = new_hair_length_export(&[1.0, 2.0]);
        scale_hair_lengths(&mut e, 2.0);
        assert!((e.lengths[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_validate_hair_lengths() {
        let e = new_hair_length_export(&[0.0, 1.0, 2.0]);
        assert!(validate_hair_lengths(&e));
    }

    #[test]
    fn test_hair_length_to_csv() {
        let e = new_hair_length_export(&[1.0]);
        let csv = hair_length_to_csv(&e);
        assert!(csv.contains("strand,length"));
    }

    #[test]
    fn test_hair_length_to_json() {
        let e = new_hair_length_export(&[1.0, 2.0]);
        let j = hair_length_to_json(&e);
        assert!(j.contains("\"strand_count\":2"));
    }
}
