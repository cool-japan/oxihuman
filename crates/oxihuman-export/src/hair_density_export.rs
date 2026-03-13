// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export per-face hair density data.

/// A hair density entry for a face.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct HairDensityEntry {
    pub face_index: u32,
    /// Hair density as strands per unit area.
    pub density: f32,
    /// Optional length scale factor.
    pub length_scale: f32,
}

/// Hair density export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairDensityExport {
    pub entries: Vec<HairDensityEntry>,
    pub default_density: f32,
}

/// Create a new hair density export.
#[allow(dead_code)]
pub fn new_hair_density_export(default_density: f32) -> HairDensityExport {
    HairDensityExport {
        entries: Vec::new(),
        default_density,
    }
}

/// Add an entry.
#[allow(dead_code)]
pub fn add_hair_density(export: &mut HairDensityExport, entry: HairDensityEntry) {
    export.entries.push(entry);
}

/// Count entries.
#[allow(dead_code)]
pub fn hair_density_count(export: &HairDensityExport) -> usize {
    export.entries.len()
}

/// Get density for a face (uses default if not found).
#[allow(dead_code)]
pub fn density_for_face(export: &HairDensityExport, face: u32) -> f32 {
    export
        .entries
        .iter()
        .find(|e| e.face_index == face)
        .map(|e| e.density)
        .unwrap_or(export.default_density)
}

/// Average density across all entries.
#[allow(dead_code)]
pub fn avg_hair_density(export: &HairDensityExport) -> f32 {
    let n = export.entries.len();
    if n == 0 {
        return export.default_density;
    }
    export.entries.iter().map(|e| e.density).sum::<f32>() / n as f32
}

/// Validate all densities are non-negative.
#[allow(dead_code)]
pub fn validate_hair_densities(export: &HairDensityExport) -> bool {
    export.entries.iter().all(|e| e.density >= 0.0)
}

/// Scale all densities by a factor.
#[allow(dead_code)]
pub fn scale_hair_densities(export: &mut HairDensityExport, factor: f32) {
    for e in &mut export.entries {
        e.density *= factor;
    }
    export.default_density *= factor;
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn hair_density_to_json(export: &HairDensityExport) -> String {
    format!(
        "{{\"entry_count\":{},\"default_density\":{:.4}}}",
        export.entries.len(),
        export.default_density
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(face: u32, density: f32) -> HairDensityEntry {
        HairDensityEntry {
            face_index: face,
            density,
            length_scale: 1.0,
        }
    }

    #[test]
    fn test_add_and_count() {
        let mut e = new_hair_density_export(10.0);
        add_hair_density(&mut e, sample_entry(0, 5.0));
        assert_eq!(hair_density_count(&e), 1);
    }

    #[test]
    fn test_density_for_face_found() {
        let mut e = new_hair_density_export(10.0);
        add_hair_density(&mut e, sample_entry(3, 7.0));
        assert!((density_for_face(&e, 3) - 7.0).abs() < 1e-5);
    }

    #[test]
    fn test_density_for_face_default() {
        let e = new_hair_density_export(10.0);
        assert!((density_for_face(&e, 99) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_avg_density() {
        let mut e = new_hair_density_export(10.0);
        add_hair_density(&mut e, sample_entry(0, 4.0));
        add_hair_density(&mut e, sample_entry(1, 6.0));
        assert!((avg_hair_density(&e) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_avg_density_empty_is_default() {
        let e = new_hair_density_export(8.0);
        assert!((avg_hair_density(&e) - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_validate_valid() {
        let mut e = new_hair_density_export(1.0);
        add_hair_density(&mut e, sample_entry(0, 5.0));
        assert!(validate_hair_densities(&e));
    }

    #[test]
    fn test_validate_invalid() {
        let mut e = new_hair_density_export(1.0);
        add_hair_density(&mut e, sample_entry(0, -1.0));
        assert!(!validate_hair_densities(&e));
    }

    #[test]
    fn test_scale_densities() {
        let mut e = new_hair_density_export(10.0);
        add_hair_density(&mut e, sample_entry(0, 5.0));
        scale_hair_densities(&mut e, 2.0);
        assert!((density_for_face(&e, 0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_hair_density_to_json() {
        let e = new_hair_density_export(10.0);
        let j = hair_density_to_json(&e);
        assert!(j.contains("default_density"));
    }

    #[test]
    fn test_empty_validate() {
        let e = new_hair_density_export(1.0);
        assert!(validate_hair_densities(&e));
    }
}
