// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Categories for morph targets (mirrors MakeHuman directory structure).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetCategory {
    Height,
    Weight,
    Muscle,
    Age,
    BodyShapes,
    ArmsLegs,
    Breast,
    Buttocks,
    Cheek,
    Chin,
    Ears,
    Eyebrows,
    Expression,
    /// Any other / user-defined category.
    Other(String),
}

impl TargetCategory {
    #[allow(clippy::should_implement_trait)]
    /// Parse a category from a directory name or target name prefix.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "height" => TargetCategory::Height,
            "weight" => TargetCategory::Weight,
            "muscle" => TargetCategory::Muscle,
            "age" => TargetCategory::Age,
            "bodyshapes" => TargetCategory::BodyShapes,
            "armslegs" => TargetCategory::ArmsLegs,
            "breast" => TargetCategory::Breast,
            "buttocks" => TargetCategory::Buttocks,
            "cheek" => TargetCategory::Cheek,
            "chin" => TargetCategory::Chin,
            "ears" => TargetCategory::Ears,
            "eyebrows" => TargetCategory::Eyebrows,
            "expression" => TargetCategory::Expression,
            other => TargetCategory::Other(other.to_string()),
        }
    }

    /// Returns the canonical string name.
    pub fn as_str(&self) -> &str {
        match self {
            TargetCategory::Height => "height",
            TargetCategory::Weight => "weight",
            TargetCategory::Muscle => "muscle",
            TargetCategory::Age => "age",
            TargetCategory::BodyShapes => "bodyshapes",
            TargetCategory::ArmsLegs => "armslegs",
            TargetCategory::Breast => "breast",
            TargetCategory::Buttocks => "buttocks",
            TargetCategory::Cheek => "cheek",
            TargetCategory::Chin => "chin",
            TargetCategory::Ears => "ears",
            TargetCategory::Eyebrows => "eyebrows",
            TargetCategory::Expression => "expression",
            TargetCategory::Other(s) => s.as_str(),
        }
    }

    /// Returns true if this category is considered safe for all audiences.
    pub fn is_safe(&self) -> bool {
        !matches!(self, TargetCategory::Other(_)) || {
            if let TargetCategory::Other(s) = self {
                let sl = s.to_lowercase();
                !sl.contains("explicit") && !sl.contains("sexual") && !sl.contains("nudity")
            } else {
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_categories() {
        assert_eq!(TargetCategory::from_str("height"), TargetCategory::Height);
        assert_eq!(TargetCategory::from_str("WEIGHT"), TargetCategory::Weight);
        assert_eq!(
            TargetCategory::from_str("armslegs"),
            TargetCategory::ArmsLegs
        );
    }

    #[test]
    fn unknown_becomes_other() {
        let c = TargetCategory::from_str("custom-shape");
        assert!(matches!(c, TargetCategory::Other(_)));
    }

    #[test]
    fn safe_categories() {
        assert!(TargetCategory::Height.is_safe());
        assert!(TargetCategory::Breast.is_safe());
    }

    #[test]
    fn explicit_other_is_unsafe() {
        let c = TargetCategory::Other("explicit-content".to_string());
        assert!(!c.is_safe());
    }
}
