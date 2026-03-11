// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Spectral power distribution export — exports SPD data for light sources and materials.

/// A single SPD sample at one wavelength.
#[derive(Debug, Clone, Copy)]
pub struct SpdSample {
    pub wavelength_nm: f32,
    pub power: f32,
}

/// A spectral power distribution.
#[derive(Debug, Default, Clone)]
pub struct Spd {
    pub name: String,
    pub samples: Vec<SpdSample>,
}

impl Spd {
    /// Creates a new SPD.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            samples: Vec::new(),
        }
    }

    /// Adds a sample.
    pub fn push(&mut self, wavelength_nm: f32, power: f32) {
        self.samples.push(SpdSample {
            wavelength_nm,
            power,
        });
    }

    /// Returns the number of samples.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Returns the peak power value.
    pub fn peak_power(&self) -> f32 {
        self.samples.iter().map(|s| s.power).fold(0.0f32, f32::max)
    }

    /// Returns a normalized SPD (peak → 1.0).
    pub fn normalized(&self) -> Spd {
        let peak = self.peak_power();
        let mut out = Spd::new(&self.name);
        if peak < f32::EPSILON {
            return out;
        }
        for s in &self.samples {
            out.push(s.wavelength_nm, s.power / peak);
        }
        out
    }
}

/// Exports an SPD to a simple CSV string.
pub fn export_spd_csv(spd: &Spd) -> String {
    let mut out = format!("# SPD: {}\nwavelength_nm,power\n", spd.name);
    for s in &spd.samples {
        out.push_str(&format!("{},{}\n", s.wavelength_nm, s.power));
    }
    out
}

/// Creates a flat/white SPD across 380..780 nm with `n` equally-spaced samples.
pub fn flat_spd(name: impl Into<String>, n: u32) -> Spd {
    let mut spd = Spd::new(name);
    for i in 0..n {
        let wl = 380.0 + (400.0 * i as f32) / n.max(1) as f32;
        spd.push(wl, 1.0);
    }
    spd
}

/// Validates that all power values are non-negative.
pub fn validate_spd(spd: &Spd) -> bool {
    spd.samples.iter().all(|s| s.power >= 0.0)
}

/// Integrates the SPD (trapezoidal rule).
pub fn integrate_spd(spd: &Spd) -> f32 {
    if spd.samples.len() < 2 {
        return 0.0;
    }
    spd.samples
        .windows(2)
        .map(|w| {
            let dw = w[1].wavelength_nm - w[0].wavelength_nm;
            (w[0].power + w[1].power) * 0.5 * dw
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_spd_empty() {
        /* New SPD should have zero samples */
        assert_eq!(Spd::new("test").sample_count(), 0);
    }

    #[test]
    fn test_push_increases_count() {
        /* Pushing increases count */
        let mut spd = Spd::new("x");
        spd.push(550.0, 1.0);
        assert_eq!(spd.sample_count(), 1);
    }

    #[test]
    fn test_peak_power_empty() {
        /* Empty SPD peak should be 0 */
        assert_eq!(Spd::new("x").peak_power(), 0.0);
    }

    #[test]
    fn test_peak_power_known() {
        /* Peak should return max power */
        let mut spd = Spd::new("x");
        spd.push(450.0, 0.3);
        spd.push(550.0, 1.0);
        assert!((spd.peak_power() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_normalized_peak_is_one() {
        /* After normalization, peak should be 1 */
        let mut spd = Spd::new("x");
        spd.push(450.0, 2.0);
        spd.push(550.0, 4.0);
        let n = spd.normalized();
        assert!((n.peak_power() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_export_csv_header() {
        /* CSV should contain header line */
        let spd = flat_spd("white", 4);
        assert!(export_spd_csv(&spd).contains("wavelength_nm,power"));
    }

    #[test]
    fn test_flat_spd_count() {
        /* flat_spd should produce n samples */
        assert_eq!(flat_spd("f", 10).sample_count(), 10);
    }

    #[test]
    fn test_validate_spd_valid() {
        /* Flat SPD should validate */
        assert!(validate_spd(&flat_spd("f", 5)));
    }

    #[test]
    fn test_integrate_flat_spd() {
        /* Integral of flat SPD over 400 nm should be ~400 */
        let spd = flat_spd("flat", 100);
        let integral = integrate_spd(&spd);
        assert!(integral > 300.0 && integral < 500.0);
    }

    #[test]
    fn test_integrate_single_sample() {
        /* Single sample → zero integral */
        let mut spd = Spd::new("single");
        spd.push(550.0, 1.0);
        assert_eq!(integrate_spd(&spd), 0.0);
    }
}
