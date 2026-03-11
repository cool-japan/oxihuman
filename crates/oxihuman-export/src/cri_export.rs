// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Color Rendering Index data export — exports CRI measurements for light sources.

/// CRI test color sample indices (TCS01..TCS08 for Ra).
pub const RA_SAMPLE_COUNT: usize = 8;

/// A CRI measurement result for a single test color sample.
#[derive(Debug, Clone, Copy)]
pub struct CriSampleResult {
    pub sample_index: u8,
    pub special_ri: f32,
}

/// Full CRI measurement data for a light source.
#[derive(Debug, Clone)]
pub struct CriData {
    pub light_name: String,
    pub cct_kelvin: f32,
    pub ra: f32,
    pub samples: Vec<CriSampleResult>,
}

impl CriData {
    /// Creates a new CRI data entry.
    pub fn new(light_name: impl Into<String>, cct_kelvin: f32, ra: f32) -> Self {
        Self {
            light_name: light_name.into(),
            cct_kelvin,
            ra,
            samples: Vec::new(),
        }
    }

    /// Adds a special CRI sample result.
    pub fn add_sample(&mut self, sample: CriSampleResult) {
        self.samples.push(sample);
    }

    /// Returns the lowest special RI across all samples.
    pub fn min_special_ri(&self) -> f32 {
        self.samples
            .iter()
            .map(|s| s.special_ri)
            .fold(f32::INFINITY, f32::min)
    }
}

/// Checks if a light source achieves a minimum Ra rating.
pub fn passes_ra_threshold(data: &CriData, min_ra: f32) -> bool {
    data.ra >= min_ra
}

/// Exports CRI data to a simple text report.
pub fn export_cri_report(data: &CriData) -> String {
    let mut out = format!(
        "Light: {}\nCCT: {} K\nCRI Ra: {:.1}\n",
        data.light_name, data.cct_kelvin, data.ra
    );
    for s in &data.samples {
        out.push_str(&format!("  R{}: {:.1}\n", s.sample_index, s.special_ri));
    }
    out
}

/// Validates CRI data (Ra must be in 0..=100, CCT must be positive).
pub fn validate_cri_data(data: &CriData) -> bool {
    (0.0..=100.0).contains(&data.ra) && data.cct_kelvin > 0.0
}

/// Computes Ra from eight TCS sample special RI values.
pub fn compute_ra(special_ris: &[f32; 8]) -> f32 {
    let sum: f32 = special_ris.iter().sum();
    (sum / 8.0).clamp(0.0, 100.0)
}

/// Returns a classification string for a Ra value.
pub fn ra_classification(ra: f32) -> &'static str {
    if ra >= 90.0 {
        "Excellent"
    } else if ra >= 80.0 {
        "Good"
    } else if ra >= 60.0 {
        "Fair"
    } else {
        "Poor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cri() -> CriData {
        CriData::new("LED A", 4000.0, 95.0)
    }

    #[test]
    fn test_new_cri_data() {
        /* CCT should be stored correctly */
        assert!((make_cri().cct_kelvin - 4000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_passes_ra_threshold_true() {
        /* Ra 95 should pass threshold of 80 */
        assert!(passes_ra_threshold(&make_cri(), 80.0));
    }

    #[test]
    fn test_passes_ra_threshold_false() {
        /* Ra 95 should fail threshold of 98 */
        assert!(!passes_ra_threshold(&make_cri(), 98.0));
    }

    #[test]
    fn test_validate_cri_data_valid() {
        /* Valid data should pass validation */
        assert!(validate_cri_data(&make_cri()));
    }

    #[test]
    fn test_validate_cri_data_negative_cct() {
        /* Negative CCT should fail validation */
        let mut d = make_cri();
        d.cct_kelvin = -100.0;
        assert!(!validate_cri_data(&d));
    }

    #[test]
    fn test_export_cri_report_contains_name() {
        /* Report should contain light name */
        assert!(export_cri_report(&make_cri()).contains("LED A"));
    }

    #[test]
    fn test_compute_ra_uniform() {
        /* All 100s → Ra = 100 */
        let ris = [100.0f32; 8];
        assert!((compute_ra(&ris) - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compute_ra_clamps() {
        /* Values above 100 should clamp */
        let ris = [110.0f32; 8];
        assert_eq!(compute_ra(&ris), 100.0);
    }

    #[test]
    fn test_ra_classification_excellent() {
        /* Ra >= 90 should be Excellent */
        assert_eq!(ra_classification(95.0), "Excellent");
    }

    #[test]
    fn test_ra_classification_poor() {
        /* Ra < 60 should be Poor */
        assert_eq!(ra_classification(50.0), "Poor");
    }
}
