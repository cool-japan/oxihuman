// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Automatic weight function factory based on target category.
//!
//! Maps TargetCategory → a closure over ParamState that returns the blend weight.

use crate::params::ParamState;
use oxihuman_core::category::TargetCategory;

/// Returns a weight function for a given category name.
///
/// The returned closure maps a `ParamState` to a blend weight in [0.0, 1.0].
pub fn auto_weight_fn(category_name: &str) -> Box<dyn Fn(&ParamState) -> f32 + Send + Sync> {
    let cat = TargetCategory::from_str(category_name);
    match cat {
        TargetCategory::Height => Box::new(|p: &ParamState| p.height),
        TargetCategory::Weight => Box::new(|p: &ParamState| p.weight),
        TargetCategory::Muscle => Box::new(|p: &ParamState| p.muscle),
        TargetCategory::Age => Box::new(|p: &ParamState| p.age),
        TargetCategory::BodyShapes => Box::new(|p: &ParamState| p.weight * 0.5 + p.muscle * 0.5),
        TargetCategory::ArmsLegs => Box::new(|p: &ParamState| p.muscle),
        TargetCategory::Breast => Box::new(|p: &ParamState| p.weight),
        TargetCategory::Buttocks => Box::new(|p: &ParamState| p.weight),
        TargetCategory::Cheek => Box::new(|p: &ParamState| p.weight * 0.3 + p.age * 0.7),
        TargetCategory::Chin => Box::new(|p: &ParamState| p.age),
        TargetCategory::Ears => Box::new(|p: &ParamState| p.age),
        TargetCategory::Eyebrows => Box::new(|p: &ParamState| p.age),
        TargetCategory::Expression => {
            Box::new(|p: &ParamState| p.extra.get("expression").copied().unwrap_or(0.0))
        }
        TargetCategory::Other(ref s) => {
            // Try to match against extra params by name
            let key = s.clone();
            Box::new(move |p: &ParamState| p.extra.get(&key).copied().unwrap_or(0.5))
        }
    }
}

/// Score assigned to a category keyword match in a target name.
/// A match preceded by "max" gets a higher score, giving it priority.
fn keyword_score(lower: &str, keyword: &str) -> u32 {
    if lower.contains(&format!("max{keyword}")) {
        2
    } else if lower.contains(keyword) {
        1
    } else {
        0
    }
}

/// Infer the best category name from a target filename.
///
/// MakeHuman targets are named like:
///   `female-young-maxmuscle-averageweight-mincup-minfirmness`
/// We use a scoring heuristic: a keyword preceded by "max" scores higher than
/// a plain keyword match. Breast-specific keywords (cup, firmness) always win
/// if present.
pub fn infer_category_from_name(name: &str) -> TargetCategory {
    let lower = name.to_lowercase();

    // Exclusive high-priority checks (these don't compete)
    if lower.contains("height") {
        return TargetCategory::Height;
    }
    if lower.contains("breast") || lower.contains("cup") || lower.contains("firmness") {
        return TargetCategory::Breast;
    }
    if lower.contains("buttock") || lower.contains("butt") {
        return TargetCategory::Buttocks;
    }
    if lower.contains("cheek") {
        return TargetCategory::Cheek;
    }
    if lower.contains("chin") {
        return TargetCategory::Chin;
    }
    if lower.contains("ear") {
        return TargetCategory::Ears;
    }
    if lower.contains("eyebrow") || lower.contains("brow") {
        return TargetCategory::Eyebrows;
    }
    if lower.contains("expression") || lower.contains("smile") || lower.contains("frown") {
        return TargetCategory::Expression;
    }

    // Competitive scoring between muscle, weight and age
    let muscle_score = keyword_score(&lower, "muscle");
    let weight_score = keyword_score(&lower, "weight")
        .max(if lower.contains("obese") { 1 } else { 0 })
        .max(if lower.contains("thin") { 1 } else { 0 });
    let age_score = keyword_score(&lower, "age")
        .max(if lower.contains("young") { 1 } else { 0 })
        .max(if lower.contains("old") { 1 } else { 0 });

    if muscle_score > weight_score && muscle_score > age_score {
        TargetCategory::Muscle
    } else if weight_score > age_score {
        TargetCategory::Weight
    } else if age_score > 0 {
        TargetCategory::Age
    } else {
        // Fall back to the dir/filename prefix heuristic
        TargetCategory::BodyShapes
    }
}

/// Convenience: get an auto weight function inferred from a target filename.
pub fn auto_weight_fn_for_target(
    target_name: &str,
) -> Box<dyn Fn(&ParamState) -> f32 + Send + Sync> {
    let cat = infer_category_from_name(target_name);
    auto_weight_fn(cat.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::ParamState;

    #[test]
    fn height_category_maps_to_height() {
        let wf = auto_weight_fn("height");
        let p = ParamState::new(0.8, 0.5, 0.5, 0.5);
        assert!((wf(&p) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn weight_category_maps_to_weight() {
        let wf = auto_weight_fn("weight");
        let p = ParamState::new(0.5, 0.3, 0.5, 0.5);
        assert!((wf(&p) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn muscle_category_maps_to_muscle() {
        let wf = auto_weight_fn("muscle");
        let p = ParamState::new(0.5, 0.5, 0.9, 0.5);
        assert!((wf(&p) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn age_category_maps_to_age() {
        let wf = auto_weight_fn("age");
        let p = ParamState::new(0.5, 0.5, 0.5, 0.7);
        assert!((wf(&p) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn infer_muscle_from_target_name() {
        let cat = infer_category_from_name("female-young-maxmuscle-averageweight");
        assert_eq!(cat, TargetCategory::Muscle);
    }

    #[test]
    fn infer_breast_from_target_name() {
        let cat =
            infer_category_from_name("female-young-maxmuscle-averageweight-mincup-minfirmness");
        // "cup" and "firmness" both match breast
        assert_eq!(cat, TargetCategory::Breast);
    }

    #[test]
    fn infer_weight_from_target_name() {
        let cat = infer_category_from_name("male-young-averagemuscle-maxweight");
        assert_eq!(cat, TargetCategory::Weight);
    }

    #[test]
    fn auto_weight_fn_for_target_works() {
        let wf = auto_weight_fn_for_target("height-incr");
        let p = ParamState::new(0.6, 0.5, 0.5, 0.5);
        assert!((wf(&p) - 0.6).abs() < 1e-6);
    }
}
