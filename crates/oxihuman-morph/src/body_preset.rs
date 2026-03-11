// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Named body type presets for character creation.

#![allow(dead_code)]

use std::collections::HashMap;

/// Parameter map: param name → value in [0.0, 1.0].
pub type BodyParams = HashMap<String, f32>;

// ---------------------------------------------------------------------------
// BodyCategory
// ---------------------------------------------------------------------------

/// Body type categories used to classify presets.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BodyCategory {
    /// Slim, low body fat.
    Ectomorph,
    /// Muscular, athletic.
    Mesomorph,
    /// Rounder, higher body fat.
    Endomorph,
    /// Balanced average.
    Average,
    /// Short stature.
    Petite,
    /// Tall stature.
    Tall,
    /// Child proportions.
    Child,
    /// Elderly proportions.
    Elder,
    /// Custom (user-defined).
    Custom(String),
}

impl BodyCategory {
    /// Short name for the category.
    pub fn name(&self) -> &str {
        match self {
            BodyCategory::Ectomorph => "Ectomorph",
            BodyCategory::Mesomorph => "Mesomorph",
            BodyCategory::Endomorph => "Endomorph",
            BodyCategory::Average => "Average",
            BodyCategory::Petite => "Petite",
            BodyCategory::Tall => "Tall",
            BodyCategory::Child => "Child",
            BodyCategory::Elder => "Elder",
            BodyCategory::Custom(s) => s.as_str(),
        }
    }

    /// Human-readable description of the category.
    pub fn description(&self) -> &str {
        match self {
            BodyCategory::Ectomorph => "Slim body type with low body fat and lean muscle.",
            BodyCategory::Mesomorph => "Muscular and athletic body type.",
            BodyCategory::Endomorph => "Rounder body type with higher body fat.",
            BodyCategory::Average => "Balanced, average body proportions.",
            BodyCategory::Petite => "Short stature with proportionally smaller frame.",
            BodyCategory::Tall => "Tall stature with elongated proportions.",
            BodyCategory::Child => "Child body proportions with low muscle mass.",
            BodyCategory::Elder => "Elderly body proportions with reduced muscle mass.",
            BodyCategory::Custom(_) => "User-defined custom body category.",
        }
    }

    /// All named (non-Custom) categories.
    pub fn all_named() -> Vec<BodyCategory> {
        vec![
            BodyCategory::Ectomorph,
            BodyCategory::Mesomorph,
            BodyCategory::Endomorph,
            BodyCategory::Average,
            BodyCategory::Petite,
            BodyCategory::Tall,
            BodyCategory::Child,
            BodyCategory::Elder,
        ]
    }
}

// ---------------------------------------------------------------------------
// BodyPreset
// ---------------------------------------------------------------------------

/// A named body preset with a full parameter set.
pub struct BodyPreset {
    /// Unique preset name.
    pub name: String,
    /// Body type category.
    pub category: BodyCategory,
    /// Human-readable description.
    pub description: String,
    /// Parameter map.
    pub params: BodyParams,
    /// Tags for filtering and search.
    pub tags: Vec<String>,
}

impl BodyPreset {
    /// Create a new preset with default empty params.
    pub fn new(name: impl Into<String>, category: BodyCategory) -> Self {
        Self {
            name: name.into(),
            category,
            description: String::new(),
            params: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Builder: set a parameter value.
    pub fn with_param(mut self, key: impl Into<String>, value: f32) -> Self {
        self.params.insert(key.into(), value);
        self
    }

    /// Builder: set description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Builder: add a tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Get a parameter value; returns 0.5 if the key is missing.
    pub fn get_param(&self, key: &str) -> f32 {
        *self.params.get(key).unwrap_or(&0.5)
    }

    /// Number of parameters stored in this preset.
    pub fn param_count(&self) -> usize {
        self.params.len()
    }
}

// ---------------------------------------------------------------------------
// PresetLibrary
// ---------------------------------------------------------------------------

/// A searchable library of named body presets.
pub struct PresetLibrary {
    presets: Vec<BodyPreset>,
}

impl PresetLibrary {
    /// Create an empty library.
    pub fn new() -> Self {
        Self {
            presets: Vec::new(),
        }
    }

    /// Add a preset to the library.
    pub fn add(&mut self, preset: BodyPreset) {
        self.presets.push(preset);
    }

    /// Look up a preset by name (case-sensitive).
    pub fn get(&self, name: &str) -> Option<&BodyPreset> {
        self.presets.iter().find(|p| p.name == name)
    }

    /// Number of presets in the library.
    pub fn preset_count(&self) -> usize {
        self.presets.len()
    }

    /// All presets belonging to a given category.
    pub fn by_category(&self, cat: &BodyCategory) -> Vec<&BodyPreset> {
        self.presets.iter().filter(|p| &p.category == cat).collect()
    }

    /// All presets that have the given tag.
    pub fn with_tag(&self, tag: &str) -> Vec<&BodyPreset> {
        self.presets
            .iter()
            .filter(|p| p.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Names of all presets in the library.
    pub fn names(&self) -> Vec<&str> {
        self.presets.iter().map(|p| p.name.as_str()).collect()
    }

    /// Blend two presets at weight `t` (0 = fully a, 1 = fully b).
    ///
    /// Missing params default to 0.5.  Returns `None` if either name is unknown.
    pub fn blend(&self, name_a: &str, name_b: &str, t: f32) -> Option<BodyParams> {
        let a = self.get(name_a)?;
        let b = self.get(name_b)?;

        // Collect union of all keys
        let mut keys: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for k in a.params.keys() {
            keys.insert(k.as_str());
        }
        for k in b.params.keys() {
            keys.insert(k.as_str());
        }

        let mut result = HashMap::new();
        for key in keys {
            let va = a.get_param(key);
            let vb = b.get_param(key);
            result.insert(key.to_string(), va + (vb - va) * t);
        }
        Some(result)
    }

    /// Find the nearest preset to the given params by L2 distance in param space.
    ///
    /// Missing params in either side default to 0.5.
    pub fn nearest(&self, params: &BodyParams) -> Option<&BodyPreset> {
        // Collect all keys appearing in `params`
        let keys: Vec<&str> = params.keys().map(|k| k.as_str()).collect();

        self.presets.iter().min_by(|a, b| {
            let dist_a = l2_distance(a, params, &keys);
            let dist_b = l2_distance(b, params, &keys);
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

impl Default for PresetLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute L2 distance between a preset and a param map over the given keys.
fn l2_distance(preset: &BodyPreset, params: &BodyParams, keys: &[&str]) -> f32 {
    keys.iter()
        .map(|k| {
            let va = preset.get_param(k);
            let vb = *params.get(*k).unwrap_or(&0.5);
            (va - vb) * (va - vb)
        })
        .sum::<f32>()
        .sqrt()
}

// ---------------------------------------------------------------------------
// Individual preset constructors
// ---------------------------------------------------------------------------

/// Average adult — all params at 0.5 (neutral starting point).
pub fn preset_average() -> BodyPreset {
    BodyPreset::new("average", BodyCategory::Average)
        .with_description("Average adult with balanced proportions.")
        .with_param("height", 0.5)
        .with_param("weight", 0.5)
        .with_param("muscle", 0.5)
        .with_param("age", 0.5)
        .with_param("bmi_factor", 0.5)
        .with_param("shoulder_width", 0.5)
        .with_param("hip_width", 0.5)
        .with_param("leg_length", 0.5)
        .with_param("torso_length", 0.5)
        .with_param("arm_length", 0.5)
        .with_tag("neutral")
        .with_tag("adult")
}

/// Athletic build — high muscle, moderate weight, young adult.
pub fn preset_athletic() -> BodyPreset {
    BodyPreset::new("athletic", BodyCategory::Mesomorph)
        .with_description("Athletic build with high muscle tone and low body fat.")
        .with_param("height", 0.55)
        .with_param("weight", 0.45)
        .with_param("muscle", 0.75)
        .with_param("age", 0.3)
        .with_param("bmi_factor", 0.35)
        .with_param("shoulder_width", 0.65)
        .with_param("hip_width", 0.5)
        .with_param("leg_length", 0.5)
        .with_param("torso_length", 0.5)
        .with_param("arm_length", 0.5)
        .with_tag("fit")
        .with_tag("adult")
        .with_tag("sport")
}

/// Slender build — low weight and muscle, ectomorphic.
pub fn preset_slender() -> BodyPreset {
    BodyPreset::new("slender", BodyCategory::Ectomorph)
        .with_description("Slender build with low body fat and lean muscle.")
        .with_param("height", 0.55)
        .with_param("weight", 0.2)
        .with_param("muscle", 0.3)
        .with_param("age", 0.3)
        .with_param("bmi_factor", 0.15)
        .with_param("shoulder_width", 0.45)
        .with_param("hip_width", 0.5)
        .with_param("leg_length", 0.5)
        .with_param("torso_length", 0.5)
        .with_param("arm_length", 0.5)
        .with_tag("slim")
        .with_tag("adult")
}

/// Heavy build — high weight, endomorphic.
pub fn preset_heavy() -> BodyPreset {
    BodyPreset::new("heavy", BodyCategory::Endomorph)
        .with_description("Heavy build with higher body fat and rounded proportions.")
        .with_param("height", 0.45)
        .with_param("weight", 0.85)
        .with_param("muscle", 0.3)
        .with_param("age", 0.45)
        .with_param("bmi_factor", 0.85)
        .with_param("shoulder_width", 0.5)
        .with_param("hip_width", 0.5)
        .with_param("leg_length", 0.5)
        .with_param("torso_length", 0.5)
        .with_param("arm_length", 0.5)
        .with_tag("heavy")
        .with_tag("adult")
}

/// Muscular build — high muscle and broad shoulders.
pub fn preset_muscular() -> BodyPreset {
    BodyPreset::new("muscular", BodyCategory::Mesomorph)
        .with_description("Highly muscular build with broad shoulders.")
        .with_param("height", 0.6)
        .with_param("weight", 0.6)
        .with_param("muscle", 0.9)
        .with_param("age", 0.35)
        .with_param("bmi_factor", 0.5)
        .with_param("shoulder_width", 0.8)
        .with_param("hip_width", 0.5)
        .with_param("leg_length", 0.5)
        .with_param("torso_length", 0.5)
        .with_param("arm_length", 0.5)
        .with_tag("muscle")
        .with_tag("adult")
        .with_tag("sport")
}

/// Petite build — short stature and small frame.
pub fn preset_petite() -> BodyPreset {
    BodyPreset::new("petite", BodyCategory::Petite)
        .with_description("Petite build with short stature and small proportions.")
        .with_param("height", 0.2)
        .with_param("weight", 0.35)
        .with_param("muscle", 0.4)
        .with_param("age", 0.5)
        .with_param("bmi_factor", 0.5)
        .with_param("shoulder_width", 0.5)
        .with_param("hip_width", 0.5)
        .with_param("leg_length", 0.3)
        .with_param("torso_length", 0.35)
        .with_param("arm_length", 0.5)
        .with_tag("short")
        .with_tag("adult")
}

/// Tall build — elongated legs and arms.
pub fn preset_tall() -> BodyPreset {
    BodyPreset::new("tall", BodyCategory::Tall)
        .with_description("Tall build with elongated limbs and proportions.")
        .with_param("height", 0.85)
        .with_param("weight", 0.5)
        .with_param("muscle", 0.5)
        .with_param("age", 0.5)
        .with_param("bmi_factor", 0.5)
        .with_param("shoulder_width", 0.5)
        .with_param("hip_width", 0.5)
        .with_param("leg_length", 0.75)
        .with_param("torso_length", 0.5)
        .with_param("arm_length", 0.7)
        .with_tag("tall")
        .with_tag("adult")
}

/// Child proportions — young, low muscle, short.
pub fn preset_child() -> BodyPreset {
    BodyPreset::new("child", BodyCategory::Child)
        .with_description("Child body proportions with low muscle mass and short stature.")
        .with_param("height", 0.1)
        .with_param("weight", 0.2)
        .with_param("muscle", 0.2)
        .with_param("age", 0.05)
        .with_param("bmi_factor", 0.5)
        .with_param("shoulder_width", 0.5)
        .with_param("hip_width", 0.5)
        .with_param("leg_length", 0.5)
        .with_param("torso_length", 0.5)
        .with_param("arm_length", 0.5)
        .with_tag("child")
        .with_tag("young")
}

/// Elder proportions — high age, reduced muscle.
pub fn preset_elder() -> BodyPreset {
    BodyPreset::new("elder", BodyCategory::Elder)
        .with_description("Elderly body proportions with reduced muscle mass.")
        .with_param("height", 0.45)
        .with_param("weight", 0.55)
        .with_param("muscle", 0.2)
        .with_param("age", 0.9)
        .with_param("bmi_factor", 0.5)
        .with_param("shoulder_width", 0.5)
        .with_param("hip_width", 0.5)
        .with_param("leg_length", 0.5)
        .with_param("torso_length", 0.5)
        .with_param("arm_length", 0.5)
        .with_tag("elder")
        .with_tag("senior")
}

// ---------------------------------------------------------------------------
// Standard library
// ---------------------------------------------------------------------------

/// Build a standard preset library containing ~12 named presets.
pub fn standard_preset_library() -> PresetLibrary {
    let mut lib = PresetLibrary::new();
    lib.add(preset_average());
    lib.add(preset_athletic());
    lib.add(preset_slender());
    lib.add(preset_heavy());
    lib.add(preset_muscular());
    lib.add(preset_petite());
    lib.add(preset_tall());
    lib.add(preset_child());
    lib.add(preset_elder());

    // Additional presets to reach ~12
    lib.add(
        BodyPreset::new("bodybuilder", BodyCategory::Mesomorph)
            .with_description("Extreme bodybuilder physique.")
            .with_param("height", 0.6)
            .with_param("weight", 0.7)
            .with_param("muscle", 1.0)
            .with_param("age", 0.35)
            .with_param("bmi_factor", 0.5)
            .with_param("shoulder_width", 0.9)
            .with_param("hip_width", 0.55)
            .with_param("leg_length", 0.5)
            .with_param("torso_length", 0.5)
            .with_param("arm_length", 0.5)
            .with_tag("muscle")
            .with_tag("adult")
            .with_tag("sport"),
    );
    lib.add(
        BodyPreset::new("runner", BodyCategory::Ectomorph)
            .with_description("Long-distance runner with lean build and long legs.")
            .with_param("height", 0.6)
            .with_param("weight", 0.25)
            .with_param("muscle", 0.45)
            .with_param("age", 0.3)
            .with_param("bmi_factor", 0.2)
            .with_param("shoulder_width", 0.4)
            .with_param("hip_width", 0.45)
            .with_param("leg_length", 0.7)
            .with_param("torso_length", 0.45)
            .with_param("arm_length", 0.6)
            .with_tag("slim")
            .with_tag("sport")
            .with_tag("adult"),
    );
    lib.add(
        BodyPreset::new("stocky", BodyCategory::Endomorph)
            .with_description("Short and broad with dense musculature.")
            .with_param("height", 0.3)
            .with_param("weight", 0.65)
            .with_param("muscle", 0.55)
            .with_param("age", 0.4)
            .with_param("bmi_factor", 0.7)
            .with_param("shoulder_width", 0.6)
            .with_param("hip_width", 0.55)
            .with_param("leg_length", 0.35)
            .with_param("torso_length", 0.4)
            .with_param("arm_length", 0.4)
            .with_tag("heavy")
            .with_tag("adult"),
    );

    lib
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_category_name() {
        assert_eq!(BodyCategory::Ectomorph.name(), "Ectomorph");
        assert_eq!(BodyCategory::Mesomorph.name(), "Mesomorph");
        assert_eq!(BodyCategory::Endomorph.name(), "Endomorph");
        assert_eq!(BodyCategory::Average.name(), "Average");
        assert_eq!(BodyCategory::Petite.name(), "Petite");
        assert_eq!(BodyCategory::Tall.name(), "Tall");
        assert_eq!(BodyCategory::Child.name(), "Child");
        assert_eq!(BodyCategory::Elder.name(), "Elder");
        assert_eq!(BodyCategory::Custom("MyType".to_string()).name(), "MyType");
    }

    #[test]
    fn test_body_category_all_named() {
        let named = BodyCategory::all_named();
        assert_eq!(named.len(), 8);
        // Custom should not appear
        for cat in &named {
            assert!(!matches!(cat, BodyCategory::Custom(_)));
        }
        assert!(named.contains(&BodyCategory::Ectomorph));
        assert!(named.contains(&BodyCategory::Elder));
    }

    #[test]
    fn test_preset_new() {
        let p = BodyPreset::new("test_preset", BodyCategory::Average);
        assert_eq!(p.name, "test_preset");
        assert_eq!(p.category, BodyCategory::Average);
        assert!(p.params.is_empty());
        assert!(p.tags.is_empty());
        assert!(p.description.is_empty());
    }

    #[test]
    fn test_preset_with_param() {
        let p = BodyPreset::new("x", BodyCategory::Average)
            .with_param("height", 0.7)
            .with_param("muscle", 0.3);
        assert_eq!(p.param_count(), 2);
        assert!((p.get_param("height") - 0.7).abs() < 1e-6);
        assert!((p.get_param("muscle") - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_preset_get_param_missing() {
        let p = BodyPreset::new("x", BodyCategory::Average);
        // Missing key defaults to 0.5
        assert!((p.get_param("nonexistent") - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_library_add_and_get() {
        let mut lib = PresetLibrary::new();
        assert_eq!(lib.preset_count(), 0);
        lib.add(preset_average());
        lib.add(preset_athletic());
        assert_eq!(lib.preset_count(), 2);
        assert!(lib.get("average").is_some());
        assert!(lib.get("athletic").is_some());
        assert!(lib.get("nonexistent").is_none());
    }

    #[test]
    fn test_library_by_category() {
        let lib = standard_preset_library();
        let mesomorphs = lib.by_category(&BodyCategory::Mesomorph);
        // athletic, muscular, bodybuilder
        assert!(mesomorphs.len() >= 2);
        for p in &mesomorphs {
            assert_eq!(p.category, BodyCategory::Mesomorph);
        }
    }

    #[test]
    fn test_library_with_tag() {
        let lib = standard_preset_library();
        let sport = lib.with_tag("sport");
        assert!(!sport.is_empty());
        for p in &sport {
            assert!(p.tags.contains(&"sport".to_string()));
        }

        let adults = lib.with_tag("adult");
        assert!(adults.len() > 3);
    }

    #[test]
    fn test_library_blend() {
        let lib = standard_preset_library();

        // blend at t=0 should give preset_a values
        let blended_0 = lib.blend("average", "athletic", 0.0).expect("blend failed");
        let avg = lib.get("average").unwrap();
        assert!((blended_0["height"] - avg.get_param("height")).abs() < 1e-5);

        // blend at t=1 should give preset_b values
        let blended_1 = lib.blend("average", "athletic", 1.0).expect("blend failed");
        let ath = lib.get("athletic").unwrap();
        assert!((blended_1["height"] - ath.get_param("height")).abs() < 1e-5);

        // blend at t=0.5 should be midpoint
        let blended_half = lib.blend("average", "athletic", 0.5).expect("blend failed");
        let expected_height = (avg.get_param("height") + ath.get_param("height")) / 2.0;
        assert!((blended_half["height"] - expected_height).abs() < 1e-5);

        // unknown name returns None
        assert!(lib.blend("average", "nonexistent", 0.5).is_none());
    }

    #[test]
    fn test_library_nearest() {
        let lib = standard_preset_library();

        // params matching "average" exactly
        let mut params: BodyParams = HashMap::new();
        params.insert("height".to_string(), 0.5);
        params.insert("weight".to_string(), 0.5);
        params.insert("muscle".to_string(), 0.5);
        params.insert("age".to_string(), 0.5);
        let nearest = lib.nearest(&params).expect("should find nearest");
        assert_eq!(nearest.name, "average");

        // params matching "child" closely
        let mut child_params: BodyParams = HashMap::new();
        child_params.insert("height".to_string(), 0.1);
        child_params.insert("muscle".to_string(), 0.2);
        child_params.insert("age".to_string(), 0.05);
        let nearest_child = lib.nearest(&child_params).expect("should find nearest");
        assert_eq!(nearest_child.name, "child");
    }

    #[test]
    fn test_preset_average() {
        let p = preset_average();
        assert_eq!(p.name, "average");
        assert_eq!(p.category, BodyCategory::Average);
        // All standard params at 0.5
        for key in &[
            "height",
            "weight",
            "muscle",
            "age",
            "bmi_factor",
            "shoulder_width",
            "hip_width",
            "leg_length",
            "torso_length",
            "arm_length",
        ] {
            assert!(
                (p.get_param(key) - 0.5).abs() < 1e-6,
                "param '{}' expected 0.5, got {}",
                key,
                p.get_param(key)
            );
        }
    }

    #[test]
    fn test_preset_athletic() {
        let p = preset_athletic();
        assert_eq!(p.name, "athletic");
        assert_eq!(p.category, BodyCategory::Mesomorph);
        assert!((p.get_param("height") - 0.55).abs() < 1e-6);
        assert!((p.get_param("weight") - 0.45).abs() < 1e-6);
        assert!((p.get_param("muscle") - 0.75).abs() < 1e-6);
        assert!((p.get_param("age") - 0.3).abs() < 1e-6);
        assert!((p.get_param("bmi_factor") - 0.35).abs() < 1e-6);
        assert!((p.get_param("shoulder_width") - 0.65).abs() < 1e-6);
        assert!(p.tags.contains(&"sport".to_string()));
    }

    #[test]
    fn test_standard_preset_library() {
        let lib = standard_preset_library();
        assert!(
            lib.preset_count() >= 12,
            "expected at least 12 presets, got {}",
            lib.preset_count()
        );
        let names = lib.names();
        assert!(names.contains(&"average"));
        assert!(names.contains(&"athletic"));
        assert!(names.contains(&"slender"));
        assert!(names.contains(&"heavy"));
        assert!(names.contains(&"muscular"));
        assert!(names.contains(&"petite"));
        assert!(names.contains(&"tall"));
        assert!(names.contains(&"child"));
        assert!(names.contains(&"elder"));
    }

    #[test]
    fn test_preset_child_age() {
        let p = preset_child();
        assert_eq!(p.name, "child");
        assert_eq!(p.category, BodyCategory::Child);
        // age should be very low (0.05)
        assert!(p.get_param("age") < 0.1, "child age param should be < 0.1");
        // height should be very low
        assert!(
            p.get_param("height") < 0.2,
            "child height param should be < 0.2"
        );
        // muscle should be low
        assert!(
            p.get_param("muscle") < 0.3,
            "child muscle param should be < 0.3"
        );
        assert!(p.tags.contains(&"child".to_string()));
    }
}
