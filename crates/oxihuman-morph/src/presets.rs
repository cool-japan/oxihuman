// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Predefined ParamState presets for common body types.

use crate::params::ParamState;

/// Named body type presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BodyPreset {
    /// Average adult (all params at 0.5).
    Average,
    /// Athletic build: high muscle, moderate weight, young adult.
    Athletic,
    /// Slender build: low weight, low muscle.
    Slender,
    /// Heavy build: high weight, moderate muscle.
    Heavy,
    /// Tall build: high height, average everything else.
    Tall,
    /// Petite build: low height, low weight.
    Petite,
    /// Older adult: high age param, moderate everything else.
    Senior,
    /// Child-proportioned: very young, low height, low muscle.
    Child,
}

impl BodyPreset {
    /// Return the ParamState for this preset.
    pub fn params(&self) -> ParamState {
        match self {
            BodyPreset::Average => ParamState::new(0.50, 0.50, 0.50, 0.50),
            BodyPreset::Athletic => ParamState::new(0.65, 0.45, 0.80, 0.30),
            BodyPreset::Slender => ParamState::new(0.55, 0.20, 0.25, 0.35),
            BodyPreset::Heavy => ParamState::new(0.50, 0.85, 0.40, 0.50),
            BodyPreset::Tall => ParamState::new(0.85, 0.45, 0.55, 0.35),
            BodyPreset::Petite => ParamState::new(0.20, 0.30, 0.30, 0.30),
            BodyPreset::Senior => ParamState::new(0.45, 0.55, 0.35, 0.85),
            BodyPreset::Child => ParamState::new(0.15, 0.25, 0.15, 0.05),
        }
    }

    /// Parse a preset by name (case-insensitive).
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "average" => Some(BodyPreset::Average),
            "athletic" => Some(BodyPreset::Athletic),
            "slender" => Some(BodyPreset::Slender),
            "heavy" => Some(BodyPreset::Heavy),
            "tall" => Some(BodyPreset::Tall),
            "petite" => Some(BodyPreset::Petite),
            "senior" => Some(BodyPreset::Senior),
            "child" => Some(BodyPreset::Child),
            _ => None,
        }
    }

    /// All available preset names.
    pub fn all_names() -> &'static [&'static str] {
        &[
            "average", "athletic", "slender", "heavy", "tall", "petite", "senior", "child",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_preset_is_midpoint() {
        let p = BodyPreset::Average.params();
        assert!((p.height - 0.5).abs() < 1e-6);
        assert!((p.weight - 0.5).abs() < 1e-6);
        assert!((p.muscle - 0.5).abs() < 1e-6);
        assert!((p.age - 0.5).abs() < 1e-6);
    }

    #[test]
    fn athletic_has_high_muscle() {
        let p = BodyPreset::Athletic.params();
        assert!(p.muscle > 0.7);
    }

    #[test]
    fn child_has_low_age() {
        let p = BodyPreset::Child.params();
        assert!(p.age < 0.1);
        assert!(p.height < 0.3);
    }

    #[test]
    fn all_params_in_range() {
        for name in BodyPreset::all_names() {
            let preset = BodyPreset::from_name(name).unwrap();
            let p = preset.params();
            assert!(
                (0.0..=1.0).contains(&p.height),
                "{} height out of range",
                name
            );
            assert!(
                (0.0..=1.0).contains(&p.weight),
                "{} weight out of range",
                name
            );
            assert!(
                (0.0..=1.0).contains(&p.muscle),
                "{} muscle out of range",
                name
            );
            assert!((0.0..=1.0).contains(&p.age), "{} age out of range", name);
        }
    }

    #[test]
    fn from_name_case_insensitive() {
        assert_eq!(
            BodyPreset::from_name("ATHLETIC"),
            Some(BodyPreset::Athletic)
        );
        assert_eq!(BodyPreset::from_name("Average"), Some(BodyPreset::Average));
        assert_eq!(BodyPreset::from_name("unknown"), None);
    }
}
