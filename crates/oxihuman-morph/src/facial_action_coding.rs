#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A single FACS Action Unit definition.
#[derive(Debug, Clone)]
pub struct FacsActionUnitDef {
    pub au_id: u32,
    pub name: String,
    pub description: String,
    pub morph_index: usize,
}

/// A set of FACS action unit definitions.
#[derive(Debug, Clone)]
pub struct FacsDefSet {
    pub units: Vec<FacsActionUnitDef>,
}

#[allow(dead_code)]
pub fn new_facs_def_set() -> FacsDefSet {
    FacsDefSet { units: Vec::new() }
}

#[allow(dead_code)]
pub fn add_facs_action_unit(
    fs: &mut FacsDefSet,
    au_id: u32,
    name: &str,
    desc: &str,
    morph_idx: usize,
) {
    fs.units.push(FacsActionUnitDef {
        au_id,
        name: name.to_string(),
        description: desc.to_string(),
        morph_index: morph_idx,
    });
}

#[allow(dead_code)]
pub fn get_facs_au(fs: &FacsDefSet, au_id: u32) -> Option<&FacsActionUnitDef> {
    fs.units.iter().find(|u| u.au_id == au_id)
}

#[allow(dead_code)]
pub fn apply_facs_au(fs: &FacsDefSet, au_id: u32, weights: &mut [f32], intensity: f32) {
    if let Some(au) = get_facs_au(fs, au_id) {
        if au.morph_index < weights.len() {
            weights[au.morph_index] = (weights[au.morph_index] + intensity).clamp(0.0, 1.0);
        }
    }
}

#[allow(dead_code)]
pub fn facs_def_au_count(fs: &FacsDefSet) -> usize {
    fs.units.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_facs_set_empty() {
        let fs = new_facs_def_set();
        assert_eq!(facs_def_au_count(&fs), 0);
    }

    #[test]
    fn test_add_au() {
        let mut fs = new_facs_def_set();
        add_facs_action_unit(&mut fs, 1, "AU1", "Inner brow raise", 0);
        assert_eq!(facs_def_au_count(&fs), 1);
    }

    #[test]
    fn test_get_au_found() {
        let mut fs = new_facs_def_set();
        add_facs_action_unit(&mut fs, 6, "AU6", "Cheek raiser", 2);
        let au = get_facs_au(&fs, 6).expect("should succeed");
        assert_eq!(au.name, "AU6");
    }

    #[test]
    fn test_get_au_not_found() {
        let fs = new_facs_def_set();
        assert!(get_facs_au(&fs, 99).is_none());
    }

    #[test]
    fn test_apply_au_sets_weight() {
        let mut fs = new_facs_def_set();
        add_facs_action_unit(&mut fs, 1, "AU1", "desc", 0);
        let mut weights = vec![0.0f32; 4];
        apply_facs_au(&fs, 1, &mut weights, 0.7);
        assert!((weights[0] - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_apply_au_clamps_to_one() {
        let mut fs = new_facs_def_set();
        add_facs_action_unit(&mut fs, 2, "AU2", "desc", 0);
        let mut weights = vec![0.8f32];
        apply_facs_au(&fs, 2, &mut weights, 0.5);
        assert!((weights[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_au_out_of_range_morph() {
        let mut fs = new_facs_def_set();
        add_facs_action_unit(&mut fs, 3, "AU3", "desc", 100);
        let mut weights = vec![0.0f32; 2];
        // Should not panic
        apply_facs_au(&fs, 3, &mut weights, 0.5);
        assert!((weights[0]).abs() < 1e-6);
    }

    #[test]
    fn test_au_description_stored() {
        let mut fs = new_facs_def_set();
        add_facs_action_unit(&mut fs, 4, "AU4", "Brow lowerer", 1);
        let au = get_facs_au(&fs, 4).expect("should succeed");
        assert_eq!(au.description, "Brow lowerer");
    }

    #[test]
    fn test_au_morph_index_stored() {
        let mut fs = new_facs_def_set();
        add_facs_action_unit(&mut fs, 5, "AU5", "desc", 3);
        let au = get_facs_au(&fs, 5).expect("should succeed");
        assert_eq!(au.morph_index, 3);
    }

    #[test]
    fn test_multiple_aus() {
        let mut fs = new_facs_def_set();
        add_facs_action_unit(&mut fs, 1, "AU1", "d1", 0);
        add_facs_action_unit(&mut fs, 2, "AU2", "d2", 1);
        add_facs_action_unit(&mut fs, 4, "AU4", "d4", 2);
        assert_eq!(facs_def_au_count(&fs), 3);
    }
}
