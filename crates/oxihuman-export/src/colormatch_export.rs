// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Color matching function data export — exports CIE CMF tables.

/// A single CMF sample at a given wavelength.
#[derive(Debug, Clone, Copy)]
pub struct CmfSample {
    pub wavelength_nm: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// A color matching function table.
#[derive(Debug, Default, Clone)]
pub struct CmfTable {
    pub samples: Vec<CmfSample>,
    pub observer: String,
}

impl CmfTable {
    /// Creates a new CMF table.
    pub fn new(observer: impl Into<String>) -> Self {
        Self {
            observer: observer.into(),
            samples: Vec::new(),
        }
    }

    /// Adds a sample to the table.
    pub fn push(&mut self, sample: CmfSample) {
        self.samples.push(sample);
    }

    /// Returns the number of wavelength samples.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Returns the wavelength range as (min, max) in nm.
    pub fn wavelength_range(&self) -> (f32, f32) {
        if self.samples.is_empty() {
            return (0.0, 0.0);
        }
        let min = self
            .samples
            .iter()
            .map(|s| s.wavelength_nm)
            .fold(f32::INFINITY, f32::min);
        let max = self
            .samples
            .iter()
            .map(|s| s.wavelength_nm)
            .fold(f32::NEG_INFINITY, f32::max);
        (min, max)
    }
}

/// Generates a minimal CIE 1931 2-degree CMF stub (3 samples only).
pub fn cie1931_stub() -> CmfTable {
    let mut table = CmfTable::new("CIE 1931 2-degree");
    table.push(CmfSample {
        wavelength_nm: 450.0,
        x: 0.3362,
        y: 0.0380,
        z: 1.7721,
    });
    table.push(CmfSample {
        wavelength_nm: 550.0,
        x: 0.4334,
        y: 0.9950,
        z: 0.0087,
    });
    table.push(CmfSample {
        wavelength_nm: 650.0,
        x: 0.2835,
        y: 0.1070,
        z: 0.0000,
    });
    table
}

/// Exports a CMF table to a CSV-like string.
pub fn export_cmf_csv(table: &CmfTable) -> String {
    let mut out = String::from("wavelength_nm,x,y,z\n");
    for s in &table.samples {
        out.push_str(&format!("{},{},{},{}\n", s.wavelength_nm, s.x, s.y, s.z));
    }
    out
}

/// Interpolates X, Y, Z values at a given wavelength.
pub fn interpolate_cmf(table: &CmfTable, wavelength: f32) -> Option<[f32; 3]> {
    if table.samples.len() < 2 {
        return None;
    }
    let idx = table
        .samples
        .partition_point(|s| s.wavelength_nm <= wavelength);
    if idx == 0 {
        let s = &table.samples[0];
        return Some([s.x, s.y, s.z]);
    }
    if idx >= table.samples.len() {
        let s = table.samples.last()?;
        return Some([s.x, s.y, s.z]);
    }
    let a = &table.samples[idx - 1];
    let b = &table.samples[idx];
    let span = (b.wavelength_nm - a.wavelength_nm).max(f32::EPSILON);
    let t = ((wavelength - a.wavelength_nm) / span).clamp(0.0, 1.0);
    Some([
        a.x + (b.x - a.x) * t,
        a.y + (b.y - a.y) * t,
        a.z + (b.z - a.z) * t,
    ])
}

/// Validates that all samples have non-negative XYZ values.
pub fn validate_cmf_table(table: &CmfTable) -> bool {
    table
        .samples
        .iter()
        .all(|s| s.x >= 0.0 && s.y >= 0.0 && s.z >= 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_table_empty() {
        /* New table should have zero samples */
        assert_eq!(CmfTable::new("CIE 1931").sample_count(), 0);
    }

    #[test]
    fn test_push_increases_count() {
        /* Pushing a sample should increase count */
        let mut table = CmfTable::new("test");
        table.push(CmfSample {
            wavelength_nm: 550.0,
            x: 0.4,
            y: 1.0,
            z: 0.0,
        });
        assert_eq!(table.sample_count(), 1);
    }

    #[test]
    fn test_cie1931_stub_count() {
        /* Stub should have exactly 3 samples */
        assert_eq!(cie1931_stub().sample_count(), 3);
    }

    #[test]
    fn test_wavelength_range() {
        /* Range should span 450..650 for the stub */
        let (min, max) = cie1931_stub().wavelength_range();
        assert!((min - 450.0).abs() < 0.1);
        assert!((max - 650.0).abs() < 0.1);
    }

    #[test]
    fn test_export_cmf_csv_header() {
        /* CSV should start with header */
        let csv = export_cmf_csv(&cie1931_stub());
        assert!(csv.starts_with("wavelength_nm"));
    }

    #[test]
    fn test_export_cmf_csv_row_count() {
        /* CSV should have 3 data rows + 1 header */
        let csv = export_cmf_csv(&cie1931_stub());
        assert_eq!(csv.lines().count(), 4);
    }

    #[test]
    fn test_interpolate_cmf_at_sample() {
        /* Interpolation at a known wavelength should return that sample's values */
        let table = cie1931_stub();
        let result = interpolate_cmf(&table, 550.0).expect("should succeed");
        assert!((result[1] - 0.9950).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_cmf_none_for_tiny_table() {
        /* Table with less than 2 samples should return None */
        let mut table = CmfTable::new("x");
        table.push(CmfSample {
            wavelength_nm: 550.0,
            x: 0.4,
            y: 1.0,
            z: 0.0,
        });
        assert!(interpolate_cmf(&table, 550.0).is_none());
    }

    #[test]
    fn test_validate_cmf_table_valid() {
        /* CIE 1931 stub should validate */
        assert!(validate_cmf_table(&cie1931_stub()));
    }

    #[test]
    fn test_wavelength_range_empty() {
        /* Empty table should return (0,0) range */
        let table = CmfTable::new("empty");
        assert_eq!(table.wavelength_range(), (0.0, 0.0));
    }
}
