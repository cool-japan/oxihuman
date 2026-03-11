// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A single named morph preset with category and weight pairs.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphPreset {
    pub name: String,
    pub category: String,
    pub weights: Vec<(String, f32)>,
}

/// A library of morph presets.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct MorphPresetLibrary {
    pub presets: Vec<MorphPreset>,
}

/// Create a new empty preset library.
#[allow(dead_code)]
pub fn new_preset_library() -> MorphPresetLibrary {
    MorphPresetLibrary { presets: Vec::new() }
}

/// Add a preset to the library.
#[allow(dead_code)]
pub fn add_preset(lib: &mut MorphPresetLibrary, name: &str, cat: &str, weights: Vec<(String, f32)>) {
    lib.presets.push(MorphPreset {
        name: name.to_string(),
        category: cat.to_string(),
        weights,
    });
}

/// Find a preset by name (case-sensitive).
#[allow(dead_code)]
pub fn find_preset<'a>(lib: &'a MorphPresetLibrary, name: &str) -> Option<&'a MorphPreset> {
    lib.presets.iter().find(|p| p.name == name)
}

/// Return all presets belonging to the given category.
#[allow(dead_code)]
pub fn presets_in_category<'a>(lib: &'a MorphPresetLibrary, cat: &str) -> Vec<&'a MorphPreset> {
    lib.presets.iter().filter(|p| p.category == cat).collect()
}

/// Return the total number of presets in the library.
#[allow(dead_code)]
pub fn preset_count(lib: &MorphPresetLibrary) -> usize {
    lib.presets.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_library_is_empty() {
        let lib = new_preset_library();
        assert_eq!(preset_count(&lib), 0);
    }

    #[test]
    fn add_one_preset() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, "smile", "face", vec![("mouth_corner".to_string(), 0.8)]);
        assert_eq!(preset_count(&lib), 1);
    }

    #[test]
    fn find_existing_preset() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, "angry", "face", vec![]);
        assert!(find_preset(&lib, "angry").is_some());
    }

    #[test]
    fn find_missing_preset_returns_none() {
        let lib = new_preset_library();
        assert!(find_preset(&lib, "unknown").is_none());
    }

    #[test]
    fn presets_in_category_filters() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, "smile", "face", vec![]);
        add_preset(&mut lib, "wave", "body", vec![]);
        add_preset(&mut lib, "wink", "face", vec![]);
        let face = presets_in_category(&lib, "face");
        assert_eq!(face.len(), 2);
    }

    #[test]
    fn presets_in_category_empty_result() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, "smile", "face", vec![]);
        let body = presets_in_category(&lib, "body");
        assert!(body.is_empty());
    }

    #[test]
    fn preset_name_stored_correctly() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, "surprised", "face", vec![]);
        assert_eq!(find_preset(&lib, "surprised").unwrap().name, "surprised");
    }

    #[test]
    fn preset_category_stored_correctly() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, "run", "pose", vec![]);
        assert_eq!(find_preset(&lib, "run").unwrap().category, "pose");
    }

    #[test]
    fn preset_weights_stored_correctly() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, "test", "face", vec![("jaw".to_string(), 0.5)]);
        let p = find_preset(&lib, "test").unwrap();
        assert!(!p.weights.is_empty());
        assert!((p.weights[0].1 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn preset_count_multiple() {
        let mut lib = new_preset_library();
        for i in 0..5 {
            add_preset(&mut lib, &format!("p{i}"), "cat", vec![]);
        }
        assert_eq!(preset_count(&lib), 5);
    }
}
