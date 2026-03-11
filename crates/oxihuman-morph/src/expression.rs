// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Facial expression preset system.
//!
//! An expression preset is a named combination of expression target weights.
//! These map to MakeHuman expression target filenames under the `expression/` subdirectory.
//! Actual target stems are drawn from `expression/units/{ethnicity}/` files.

use crate::engine::HumanEngine;
use crate::params::ParamState;

/// A single component of an expression: a target filename stem and its weight.
#[derive(Debug, Clone)]
pub struct ExpressionComponent {
    /// Target filename stem (without .target extension), e.g. "mouth-open"
    pub target_name: String,
    /// Blend weight for this component (0.0..=1.0)
    pub weight: f32,
}

impl ExpressionComponent {
    fn new(target_name: &str, weight: f32) -> Self {
        Self {
            target_name: target_name.to_string(),
            weight,
        }
    }
}

/// A named facial expression preset composed of multiple target components.
#[derive(Debug, Clone)]
pub struct ExpressionPreset {
    pub name: String,
    pub components: Vec<ExpressionComponent>,
}

impl ExpressionPreset {
    fn new(name: &str, components: Vec<ExpressionComponent>) -> Self {
        Self {
            name: name.to_string(),
            components,
        }
    }

    /// Return all built-in expression presets.
    ///
    /// Target stems correspond to actual MakeHuman expression unit targets
    /// found under `expression/units/{ethnicity}/`.
    pub fn all() -> Vec<ExpressionPreset> {
        vec![
            // neutral — baseline, no components
            ExpressionPreset::new("neutral", vec![]),
            // smile — corner puller + mouth elevation
            ExpressionPreset::new(
                "smile",
                vec![
                    ExpressionComponent::new("mouth-corner-puller", 0.8),
                    ExpressionComponent::new("mouth-elevation", 0.5),
                ],
            ),
            // frown — mouth depression + eyebrows down
            ExpressionPreset::new(
                "frown",
                vec![
                    ExpressionComponent::new("mouth-depression", 0.7),
                    ExpressionComponent::new("eyebrows-left-down", 0.5),
                    ExpressionComponent::new("eyebrows-right-down", 0.5),
                ],
            ),
            // surprised — mouth open + eyebrows up + eyes opened up
            ExpressionPreset::new(
                "surprised",
                vec![
                    ExpressionComponent::new("mouth-open", 0.7),
                    ExpressionComponent::new("eyebrows-left-up", 0.8),
                    ExpressionComponent::new("eyebrows-right-up", 0.8),
                    ExpressionComponent::new("eye-left-opened-up", 0.6),
                    ExpressionComponent::new("eye-right-opened-up", 0.6),
                ],
            ),
            // angry — eyebrows inner down + mouth compression
            ExpressionPreset::new(
                "angry",
                vec![
                    ExpressionComponent::new("eyebrows-left-inner-up", 0.0),
                    ExpressionComponent::new("eyebrows-right-inner-up", 0.0),
                    ExpressionComponent::new("eyebrows-left-down", 0.8),
                    ExpressionComponent::new("eyebrows-right-down", 0.8),
                    ExpressionComponent::new("mouth-compression", 0.6),
                    ExpressionComponent::new("mouth-retraction", 0.3),
                ],
            ),
            // sad — mouth depression + eyebrows inner up (classic sad brow) + eye slit
            ExpressionPreset::new(
                "sad",
                vec![
                    ExpressionComponent::new("mouth-depression", 0.6),
                    ExpressionComponent::new("eyebrows-left-inner-up", 0.7),
                    ExpressionComponent::new("eyebrows-right-inner-up", 0.7),
                    ExpressionComponent::new("eye-left-slit", 0.3),
                    ExpressionComponent::new("eye-right-slit", 0.3),
                ],
            ),
        ]
    }

    /// Find a preset by name (case-insensitive).
    pub fn from_name(name: &str) -> Option<ExpressionPreset> {
        let lower = name.to_lowercase();
        ExpressionPreset::all()
            .into_iter()
            .find(|p| p.name == lower)
    }

    /// Return all preset names.
    pub fn all_names() -> Vec<&'static str> {
        vec!["neutral", "smile", "frown", "surprised", "angry", "sad"]
    }
}

/// Load expression targets from a directory and apply a preset to a HumanEngine.
///
/// Targets are looked up as `{expression_dir}/{component.target_name}.target`.
/// Each matched target is loaded with a constant weight function returning the component weight.
/// Returns the count of successfully applied targets.
///
/// Missing target files are silently skipped (graceful degradation).
pub fn apply_expression_to_engine(
    engine: &mut HumanEngine,
    preset: &ExpressionPreset,
    expression_dir: &std::path::Path,
) -> usize {
    use oxihuman_core::parser::target::parse_target;

    if !expression_dir.exists() {
        return 0;
    }

    let mut count = 0usize;
    for component in &preset.components {
        let target_path = expression_dir.join(format!("{}.target", component.target_name));
        if !target_path.exists() {
            continue;
        }
        let src = match std::fs::read_to_string(&target_path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let target = match parse_target(&component.target_name, &src) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let w = component.weight;
        engine.load_target(target, Box::new(move |_p: &ParamState| w));
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_presets_have_names() {
        assert!(
            ExpressionPreset::all_names().len() >= 5,
            "expected at least 5 expression presets"
        );
    }

    #[test]
    fn neutral_preset_has_no_components() {
        let preset = ExpressionPreset::from_name("neutral").expect("neutral must exist");
        assert!(
            preset.components.is_empty(),
            "neutral should have no components"
        );
    }

    #[test]
    fn from_name_case_insensitive() {
        let lower = ExpressionPreset::from_name("smile").expect("smile must exist");
        let upper = ExpressionPreset::from_name("SMILE").expect("SMILE must exist");
        assert_eq!(lower.name, upper.name);
        assert_eq!(lower.components.len(), upper.components.len());
    }

    #[test]
    fn from_name_unknown_returns_none() {
        assert!(ExpressionPreset::from_name("xyzzy").is_none());
    }

    #[test]
    fn preset_components_have_valid_weights() {
        for preset in ExpressionPreset::all() {
            for comp in &preset.components {
                assert!(
                    (0.0..=1.0).contains(&comp.weight),
                    "preset '{}' component '{}' has weight {} out of [0,1]",
                    preset.name,
                    comp.target_name,
                    comp.weight
                );
            }
        }
    }

    #[test]
    fn apply_expression_skips_missing_targets() {
        use oxihuman_core::parser::obj::ObjMesh;
        use oxihuman_core::policy::{Policy, PolicyProfile};
        // Minimal valid base mesh (one triangle)
        let base = ObjMesh {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
        };
        let policy = Policy::new(PolicyProfile::Standard);
        let mut engine = HumanEngine::new(base, policy);
        let preset = ExpressionPreset::from_name("smile").expect("smile must exist");
        // Non-existent directory — must return 0 without panicking
        let count = apply_expression_to_engine(
            &mut engine,
            &preset,
            std::path::Path::new("/tmp/nonexistent_expression_dir_oxihuman"),
        );
        assert_eq!(
            count, 0,
            "should return 0 when expression dir does not exist"
        );
    }

    #[test]
    fn all_preset_names_resolve() {
        for name in ExpressionPreset::all_names() {
            assert!(
                ExpressionPreset::from_name(name).is_some(),
                "preset name '{}' must resolve via from_name",
                name
            );
        }
    }
}
