// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export corrective blend-shape data.

/// A single corrective blend shape target.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CorrectiveBlend {
    pub name: String,
    pub driver_bone: String,
    pub driver_angle_deg: f32,
    pub deltas: Vec<[f32; 3]>,
}

/// A bundle of corrective blends.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CorrectiveBlendBundle {
    pub shapes: Vec<CorrectiveBlend>,
}

/// Create a new corrective blend bundle.
#[allow(dead_code)]
pub fn new_corrective_blend_bundle() -> CorrectiveBlendBundle {
    CorrectiveBlendBundle { shapes: Vec::new() }
}

/// Add a corrective blend shape.
#[allow(dead_code)]
pub fn add_corrective_blend(bundle: &mut CorrectiveBlendBundle, shape: CorrectiveBlend) {
    bundle.shapes.push(shape);
}

/// Count shapes in the bundle.
#[allow(dead_code)]
pub fn corrective_blend_count(bundle: &CorrectiveBlendBundle) -> usize {
    bundle.shapes.len()
}

/// Find a shape by name.
#[allow(dead_code)]
pub fn find_corrective_blend<'a>(
    bundle: &'a CorrectiveBlendBundle,
    name: &str,
) -> Option<&'a CorrectiveBlend> {
    bundle.shapes.iter().find(|s| s.name == name)
}

/// Compute the maximum delta magnitude in a shape.
#[allow(dead_code)]
pub fn max_corrective_delta(shape: &CorrectiveBlend) -> f32 {
    shape
        .deltas
        .iter()
        .map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
        .fold(0.0f32, f32::max)
}

/// Count shapes with a given driver bone.
#[allow(dead_code)]
pub fn shapes_for_bone(bundle: &CorrectiveBlendBundle, bone: &str) -> usize {
    bundle
        .shapes
        .iter()
        .filter(|s| s.driver_bone == bone)
        .count()
}

/// Validate all shapes have non-empty deltas.
#[allow(dead_code)]
pub fn validate_corrective_bundle(bundle: &CorrectiveBlendBundle) -> bool {
    bundle.shapes.iter().all(|s| !s.deltas.is_empty())
}

/// Serialize bundle to JSON.
#[allow(dead_code)]
pub fn corrective_blend_to_json(bundle: &CorrectiveBlendBundle) -> String {
    format!("{{\"shape_count\":{}}}", bundle.shapes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_shape(name: &str) -> CorrectiveBlend {
        CorrectiveBlend {
            name: name.to_string(),
            driver_bone: "arm".to_string(),
            driver_angle_deg: 45.0,
            deltas: vec![[0.1, 0.0, 0.0], [0.0, 0.2, 0.0]],
        }
    }

    #[test]
    fn test_add_and_count() {
        let mut b = new_corrective_blend_bundle();
        add_corrective_blend(&mut b, sample_shape("test"));
        assert_eq!(corrective_blend_count(&b), 1);
    }

    #[test]
    fn test_find_existing() {
        let mut b = new_corrective_blend_bundle();
        add_corrective_blend(&mut b, sample_shape("elbow"));
        assert!(find_corrective_blend(&b, "elbow").is_some());
    }

    #[test]
    fn test_find_missing() {
        let b = new_corrective_blend_bundle();
        assert!(find_corrective_blend(&b, "nope").is_none());
    }

    #[test]
    fn test_max_delta_positive() {
        let s = sample_shape("x");
        assert!(max_corrective_delta(&s) > 0.0);
    }

    #[test]
    fn test_shapes_for_bone() {
        let mut b = new_corrective_blend_bundle();
        add_corrective_blend(&mut b, sample_shape("a"));
        add_corrective_blend(&mut b, sample_shape("b"));
        assert_eq!(shapes_for_bone(&b, "arm"), 2);
    }

    #[test]
    fn test_validate_valid() {
        let mut b = new_corrective_blend_bundle();
        add_corrective_blend(&mut b, sample_shape("a"));
        assert!(validate_corrective_bundle(&b));
    }

    #[test]
    fn test_validate_empty_deltas() {
        let mut b = new_corrective_blend_bundle();
        b.shapes.push(CorrectiveBlend {
            name: "x".to_string(),
            driver_bone: "b".to_string(),
            driver_angle_deg: 0.0,
            deltas: vec![],
        });
        assert!(!validate_corrective_bundle(&b));
    }

    #[test]
    fn test_corrective_blend_to_json() {
        let b = new_corrective_blend_bundle();
        let j = corrective_blend_to_json(&b);
        assert!(j.contains("shape_count"));
    }

    #[test]
    fn test_empty_bundle() {
        let b = new_corrective_blend_bundle();
        assert_eq!(corrective_blend_count(&b), 0);
    }

    #[test]
    fn test_shapes_for_bone_zero() {
        let b = new_corrective_blend_bundle();
        assert_eq!(shapes_for_bone(&b, "leg"), 0);
    }
}
