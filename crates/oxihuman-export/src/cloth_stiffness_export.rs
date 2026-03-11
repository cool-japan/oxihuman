// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export per-vertex cloth stiffness data.

/// Cloth stiffness type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StiffnessType {
    Structural,
    Shear,
    Bending,
}

/// A cloth stiffness map entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothStiffnessEntry {
    pub vertex_index: u32,
    pub stiffness: f32,
    pub stiffness_type: StiffnessType,
}

/// A cloth stiffness export bundle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothStiffnessExport {
    pub entries: Vec<ClothStiffnessEntry>,
}

/// Create a new cloth stiffness export.
#[allow(dead_code)]
pub fn new_cloth_stiffness_export() -> ClothStiffnessExport {
    ClothStiffnessExport {
        entries: Vec::new(),
    }
}

/// Add an entry.
#[allow(dead_code)]
pub fn add_cloth_stiffness(export: &mut ClothStiffnessExport, entry: ClothStiffnessEntry) {
    export.entries.push(entry);
}

/// Count entries.
#[allow(dead_code)]
pub fn cloth_stiffness_count(export: &ClothStiffnessExport) -> usize {
    export.entries.len()
}

/// Count entries of a given stiffness type.
#[allow(dead_code)]
pub fn count_stiffness_type(export: &ClothStiffnessExport, stype: StiffnessType) -> usize {
    export
        .entries
        .iter()
        .filter(|e| e.stiffness_type == stype)
        .count()
}

/// Average stiffness value.
#[allow(dead_code)]
pub fn avg_stiffness_value(export: &ClothStiffnessExport) -> f32 {
    let n = export.entries.len();
    if n == 0 {
        return 0.0;
    }
    export.entries.iter().map(|e| e.stiffness).sum::<f32>() / n as f32
}

/// Validate all stiffness values are in [0.0, 1.0].
#[allow(dead_code)]
pub fn validate_stiffness(export: &ClothStiffnessExport) -> bool {
    export
        .entries
        .iter()
        .all(|e| (0.0..=1.0).contains(&e.stiffness))
}

/// Clamp all stiffness values to [0.0, 1.0].
#[allow(dead_code)]
pub fn clamp_stiffness(export: &mut ClothStiffnessExport) {
    for e in &mut export.entries {
        e.stiffness = e.stiffness.clamp(0.0, 1.0);
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn cloth_stiffness_to_json(export: &ClothStiffnessExport) -> String {
    format!("{{\"entry_count\":{}}}", export.entries.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(v: u32, s: f32, t: StiffnessType) -> ClothStiffnessEntry {
        ClothStiffnessEntry {
            vertex_index: v,
            stiffness: s,
            stiffness_type: t,
        }
    }

    #[test]
    fn test_add_and_count() {
        let mut e = new_cloth_stiffness_export();
        add_cloth_stiffness(&mut e, sample_entry(0, 0.5, StiffnessType::Structural));
        assert_eq!(cloth_stiffness_count(&e), 1);
    }

    #[test]
    fn test_count_by_type() {
        let mut e = new_cloth_stiffness_export();
        add_cloth_stiffness(&mut e, sample_entry(0, 0.5, StiffnessType::Structural));
        add_cloth_stiffness(&mut e, sample_entry(1, 0.3, StiffnessType::Shear));
        assert_eq!(count_stiffness_type(&e, StiffnessType::Structural), 1);
    }

    #[test]
    fn test_avg_stiffness() {
        let mut e = new_cloth_stiffness_export();
        add_cloth_stiffness(&mut e, sample_entry(0, 0.4, StiffnessType::Structural));
        add_cloth_stiffness(&mut e, sample_entry(1, 0.6, StiffnessType::Structural));
        assert!((avg_stiffness_value(&e) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_validate_valid() {
        let mut e = new_cloth_stiffness_export();
        add_cloth_stiffness(&mut e, sample_entry(0, 0.5, StiffnessType::Structural));
        assert!(validate_stiffness(&e));
    }

    #[test]
    fn test_validate_invalid() {
        let mut e = new_cloth_stiffness_export();
        add_cloth_stiffness(&mut e, sample_entry(0, 1.5, StiffnessType::Structural));
        assert!(!validate_stiffness(&e));
    }

    #[test]
    fn test_clamp_stiffness() {
        let mut e = new_cloth_stiffness_export();
        add_cloth_stiffness(&mut e, sample_entry(0, 2.0, StiffnessType::Structural));
        clamp_stiffness(&mut e);
        assert!(validate_stiffness(&e));
    }

    #[test]
    fn test_cloth_stiffness_to_json() {
        let e = new_cloth_stiffness_export();
        let j = cloth_stiffness_to_json(&e);
        assert!(j.contains("entry_count"));
    }

    #[test]
    fn test_avg_empty() {
        let e = new_cloth_stiffness_export();
        assert!(avg_stiffness_value(&e).abs() < 1e-6);
    }

    #[test]
    fn test_count_stiffness_bending() {
        let mut e = new_cloth_stiffness_export();
        add_cloth_stiffness(&mut e, sample_entry(0, 0.5, StiffnessType::Bending));
        assert_eq!(count_stiffness_type(&e, StiffnessType::Bending), 1);
        assert_eq!(count_stiffness_type(&e, StiffnessType::Shear), 0);
    }

    #[test]
    fn test_validate_empty() {
        let e = new_cloth_stiffness_export();
        assert!(validate_stiffness(&e));
    }
}
