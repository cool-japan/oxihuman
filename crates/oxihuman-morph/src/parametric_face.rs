// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Parametric face model with FACS action units and expression composition.

use std::collections::HashMap;

/// A single named face parameter.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct FaceParam {
    pub name: String,
    /// Normalized value: 0..1, or −1..1 for bilateral parameters.
    pub value: f32,
    /// `true` = both sides move together.
    pub symmetric: bool,
}

/// Container of all face parameters.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct FaceModel {
    pub params: HashMap<String, FaceParam>,
}

/// A FACS action unit that drives a set of morph weights.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct FaceActionUnit {
    /// FACS AU number (e.g. 1 = inner brow raise).
    pub au_id: u8,
    pub name: String,
    pub description: String,
    /// Morph parameter → base weight mapping.
    pub morph_weights: HashMap<String, f32>,
    pub bilateral: bool,
}

// ── standard params ───────────────────────────────────────────────────────────

/// Return at least 15 standard face parameters.
#[allow(dead_code)]
pub fn standard_face_params() -> Vec<FaceParam> {
    fn p(name: &str, sym: bool) -> FaceParam {
        FaceParam {
            name: name.to_owned(),
            value: 0.0,
            symmetric: sym,
        }
    }
    vec![
        p("brow_raise_l", false),
        p("brow_raise_r", false),
        p("brow_furrow", true),
        p("eye_open_l", false),
        p("eye_open_r", false),
        p("eye_squint", true),
        p("cheek_puff", true),
        p("lip_corner_pull", true),
        p("lip_pucker", true),
        p("jaw_open", true),
        p("chin_raise", true),
        p("nose_wrinkle", true),
        p("upper_lid_l", false),
        p("upper_lid_r", false),
        p("smile_l", false),
        p("smile_r", false),
    ]
}

/// Return at least 10 FACS action units mapped to morph weights.
#[allow(dead_code)]
pub fn standard_face_action_units() -> Vec<FaceActionUnit> {
    fn au(
        id: u8,
        name: &str,
        desc: &str,
        weights: &[(&str, f32)],
        bilateral: bool,
    ) -> FaceActionUnit {
        FaceActionUnit {
            au_id: id,
            name: name.to_owned(),
            description: desc.to_owned(),
            morph_weights: weights.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            bilateral,
        }
    }
    vec![
        au(
            1,
            "AU1",
            "Inner brow raise",
            &[("brow_raise_l", 0.5), ("brow_raise_r", 0.5)],
            true,
        ),
        au(
            2,
            "AU2",
            "Outer brow raise",
            &[("brow_raise_l", 1.0), ("brow_raise_r", 1.0)],
            true,
        ),
        au(4, "AU4", "Brow lowerer", &[("brow_furrow", 1.0)], true),
        au(
            5,
            "AU5",
            "Upper lid raiser",
            &[("upper_lid_l", 1.0), ("upper_lid_r", 1.0)],
            true,
        ),
        au(6, "AU6", "Cheek raiser", &[("cheek_puff", 0.8)], true),
        au(7, "AU7", "Lid tightener", &[("eye_squint", 1.0)], true),
        au(
            10,
            "AU10",
            "Upper lip raiser",
            &[("lip_corner_pull", 0.5)],
            true,
        ),
        au(
            12,
            "AU12",
            "Lip corner puller",
            &[("smile_l", 1.0), ("smile_r", 1.0)],
            true,
        ),
        au(17, "AU17", "Chin raiser", &[("chin_raise", 1.0)], true),
        au(
            25,
            "AU25",
            "Lips part",
            &[("jaw_open", 0.4), ("lip_pucker", -0.2)],
            true,
        ),
    ]
}

// ── FaceModel impl ────────────────────────────────────────────────────────────

impl FaceModel {
    /// Create an empty face model.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    /// Set a parameter value (inserts if absent).
    #[allow(dead_code)]
    pub fn set_param(&mut self, name: &str, value: f32) {
        self.params
            .entry(name.to_owned())
            .and_modify(|p| p.value = value)
            .or_insert(FaceParam {
                name: name.to_owned(),
                value,
                symmetric: false,
            });
    }

    /// Get a parameter value (returns 0.0 if absent).
    #[allow(dead_code)]
    pub fn get_param(&self, name: &str) -> f32 {
        self.params.get(name).map(|p| p.value).unwrap_or(0.0)
    }

    /// Apply a FACS action unit at the given intensity (0..1).
    #[allow(dead_code)]
    pub fn apply_action_unit(&mut self, au: &FaceActionUnit, intensity: f32) {
        for (param, &weight) in &au.morph_weights {
            let v = weight * intensity;
            self.set_param(param, v);
        }
    }

    /// Flatten all parameters to a morph weight map.
    #[allow(dead_code)]
    pub fn compose_expression(&self) -> HashMap<String, f32> {
        self.params
            .iter()
            .map(|(k, p)| (k.clone(), p.value))
            .collect()
    }

    /// Reset all parameters to 0.0.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        for p in self.params.values_mut() {
            p.value = 0.0;
        }
    }
}

// ── free functions ────────────────────────────────────────────────────────────

/// Linearly interpolate all parameters between models `a` and `b` at position `t`.
#[allow(dead_code)]
pub fn blend_face_params(a: &FaceModel, b: &FaceModel, t: f32) -> FaceModel {
    let t = t.clamp(0.0, 1.0);
    let mut result = FaceModel::new();
    // collect all param names
    let names: std::collections::HashSet<&String> =
        a.params.keys().chain(b.params.keys()).collect();
    for name in names {
        let va = a.get_param(name);
        let vb = b.get_param(name);
        let sym = a
            .params
            .get(name)
            .or_else(|| b.params.get(name))
            .map(|p| p.symmetric)
            .unwrap_or(false);
        result.params.insert(
            name.clone(),
            FaceParam {
                name: name.clone(),
                value: va + (vb - va) * t,
                symmetric: sym,
            },
        );
    }
    result
}

/// Return expression presets (at least 6).
#[allow(dead_code)]
pub fn expression_presets() -> HashMap<String, HashMap<String, f32>> {
    let mut map: HashMap<String, HashMap<String, f32>> = HashMap::new();

    // neutral — all zero
    map.insert("neutral".to_owned(), HashMap::new());

    // happy
    map.insert(
        "happy".to_owned(),
        [
            ("smile_l", 0.9),
            ("smile_r", 0.9),
            ("cheek_puff", 0.3),
            ("eye_squint", 0.2),
            ("lip_corner_pull", 0.8),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect(),
    );

    // sad
    map.insert(
        "sad".to_owned(),
        [
            ("brow_furrow", 0.6),
            ("brow_raise_l", -0.3),
            ("brow_raise_r", -0.3),
            ("lip_corner_pull", -0.5),
            ("jaw_open", 0.1),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect(),
    );

    // surprised
    map.insert(
        "surprised".to_owned(),
        [
            ("brow_raise_l", 1.0),
            ("brow_raise_r", 1.0),
            ("eye_open_l", 1.0),
            ("eye_open_r", 1.0),
            ("jaw_open", 0.6),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect(),
    );

    // angry
    map.insert(
        "angry".to_owned(),
        [
            ("brow_furrow", 1.0),
            ("eye_squint", 0.5),
            ("nose_wrinkle", 0.6),
            ("lip_corner_pull", -0.4),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect(),
    );

    // disgusted
    map.insert(
        "disgusted".to_owned(),
        [
            ("nose_wrinkle", 1.0),
            ("lip_pucker", 0.4),
            ("lip_corner_pull", -0.5),
            ("chin_raise", 0.3),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect(),
    );

    // fearful (7th preset)
    map.insert(
        "fearful".to_owned(),
        [
            ("brow_raise_l", 0.7),
            ("brow_raise_r", 0.7),
            ("eye_open_l", 0.8),
            ("eye_open_r", 0.8),
            ("jaw_open", 0.3),
            ("lip_corner_pull", -0.2),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect(),
    );

    map
}

/// Apply an expression preset to a face model.
#[allow(dead_code)]
pub fn apply_expression_preset(model: &mut FaceModel, preset: &HashMap<String, f32>) {
    for (name, &value) in preset {
        model.set_param(name, value);
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // 1. standard_face_params >= 15
    #[test]
    fn test_standard_face_params_count() {
        assert!(standard_face_params().len() >= 15);
    }

    // 2. standard_face_action_units >= 10
    #[test]
    fn test_standard_face_action_units_count() {
        assert!(standard_face_action_units().len() >= 10);
    }

    // 3. FaceModel::new empty params
    #[test]
    fn test_face_model_new_empty() {
        let m = FaceModel::new();
        assert!(m.params.is_empty());
    }

    // 4. set_param / get_param round-trip
    #[test]
    fn test_set_get_param() {
        let mut m = FaceModel::new();
        m.set_param("jaw_open", 0.7);
        assert!((m.get_param("jaw_open") - 0.7).abs() < 1e-6);
    }

    // 5. get_param returns 0.0 for absent key
    #[test]
    fn test_get_param_absent() {
        let m = FaceModel::new();
        assert!((m.get_param("nonexistent") - 0.0).abs() < 1e-6);
    }

    // 6. apply_action_unit scales morph weights by intensity
    #[test]
    fn test_apply_action_unit_scales() {
        let aus = standard_face_action_units();
        let au12 = aus
            .iter()
            .find(|au| au.au_id == 12)
            .expect("should succeed");
        let mut model = FaceModel::new();
        model.apply_action_unit(au12, 0.5);
        // AU12 drives smile_l with weight 1.0 × 0.5 = 0.5
        assert!((model.get_param("smile_l") - 0.5).abs() < 1e-5);
    }

    // 7. compose_expression returns map with all params
    #[test]
    fn test_compose_expression_contains_params() {
        let mut m = FaceModel::new();
        m.set_param("brow_furrow", 0.3);
        m.set_param("jaw_open", 0.6);
        let expr = m.compose_expression();
        assert!(expr.contains_key("brow_furrow"));
        assert!(expr.contains_key("jaw_open"));
    }

    // 8. reset zeros all params
    #[test]
    fn test_reset_zeros_all() {
        let mut m = FaceModel::new();
        m.set_param("smile_l", 0.8);
        m.set_param("jaw_open", 0.4);
        m.reset();
        for p in m.params.values() {
            assert!((p.value).abs() < 1e-6, "param {} should be 0", p.name);
        }
    }

    // 9. blend at t=0 returns a
    #[test]
    fn test_blend_t0_is_a() {
        let mut a = FaceModel::new();
        a.set_param("x", 1.0);
        let mut b = FaceModel::new();
        b.set_param("x", 0.0);
        let result = blend_face_params(&a, &b, 0.0);
        assert!((result.get_param("x") - 1.0).abs() < 1e-5);
    }

    // 10. blend at t=1 returns b
    #[test]
    fn test_blend_t1_is_b() {
        let mut a = FaceModel::new();
        a.set_param("x", 0.0);
        let mut b = FaceModel::new();
        b.set_param("x", 1.0);
        let result = blend_face_params(&a, &b, 1.0);
        assert!((result.get_param("x") - 1.0).abs() < 1e-5);
    }

    // 11. expression_presets has >= 6 presets
    #[test]
    fn test_expression_presets_count() {
        let presets = expression_presets();
        assert!(
            presets.len() >= 6,
            "expected >= 6 presets, got {}",
            presets.len()
        );
    }

    // 12. apply_expression_preset applies values
    #[test]
    fn test_apply_expression_preset_applies() {
        let presets = expression_presets();
        let happy = &presets["happy"];
        let mut model = FaceModel::new();
        apply_expression_preset(&mut model, happy);
        // smile_l should be set
        assert!(
            model.get_param("smile_l") > 0.0,
            "happy preset should set smile_l"
        );
    }

    // 13. neutral preset — explicitly all-zero (no entries)
    #[test]
    fn test_neutral_preset_all_zero() {
        let presets = expression_presets();
        let neutral = &presets["neutral"];
        for &v in neutral.values() {
            assert!((v).abs() < 1e-6, "neutral preset should be all-zero");
        }
    }

    // 14. bilateral param flag set correctly for symmetric params
    #[test]
    fn test_bilateral_param_symmetric_flag() {
        let params = standard_face_params();
        let jaw = params
            .iter()
            .find(|p| p.name == "jaw_open")
            .expect("should succeed");
        assert!(jaw.symmetric, "jaw_open should be symmetric");
        let brow_l = params
            .iter()
            .find(|p| p.name == "brow_raise_l")
            .expect("should succeed");
        assert!(!brow_l.symmetric, "brow_raise_l should not be symmetric");
    }

    // 15. FaceActionUnit bilateral flag set for AU12
    #[test]
    fn test_action_unit_bilateral_flag() {
        let aus = standard_face_action_units();
        let au12 = aus
            .iter()
            .find(|au| au.au_id == 12)
            .expect("should succeed");
        assert!(au12.bilateral);
    }

    // 16. blend midpoint
    #[test]
    fn test_blend_midpoint() {
        let mut a = FaceModel::new();
        a.set_param("x", 0.0);
        let mut b = FaceModel::new();
        b.set_param("x", 1.0);
        let result = blend_face_params(&a, &b, 0.5);
        assert!((result.get_param("x") - 0.5).abs() < 1e-5);
    }
}
