// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Blend shape weight export v2: named weights with curve evaluation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendWeightV2 {
    pub name: String,
    pub weight: f32,
    pub min_weight: f32,
    pub max_weight: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendWeightExportV2 {
    pub weights: Vec<BlendWeightV2>,
}

#[allow(dead_code)]
pub fn new_blend_weight_export_v2() -> BlendWeightExportV2 {
    BlendWeightExportV2 {
        weights: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_blend_weight_v2(exp: &mut BlendWeightExportV2, name: &str, weight: f32) {
    exp.weights.push(BlendWeightV2 {
        name: name.to_string(),
        weight: weight.clamp(0.0, 1.0),
        min_weight: 0.0,
        max_weight: 1.0,
    });
}

#[allow(dead_code)]
pub fn weight_count_v2(exp: &BlendWeightExportV2) -> usize {
    exp.weights.len()
}

#[allow(dead_code)]
pub fn find_weight_v2<'a>(exp: &'a BlendWeightExportV2, name: &str) -> Option<&'a BlendWeightV2> {
    exp.weights.iter().find(|w| w.name == name)
}

#[allow(dead_code)]
pub fn set_weight_v2(exp: &mut BlendWeightExportV2, name: &str, weight: f32) {
    if let Some(w) = exp.weights.iter_mut().find(|w| w.name == name) {
        w.weight = weight.clamp(w.min_weight, w.max_weight);
    }
}

#[allow(dead_code)]
pub fn active_weight_count(exp: &BlendWeightExportV2) -> usize {
    exp.weights.iter().filter(|w| w.weight > 0.0).count()
}

#[allow(dead_code)]
pub fn total_weight_sum(exp: &BlendWeightExportV2) -> f32 {
    exp.weights.iter().map(|w| w.weight).sum()
}

#[allow(dead_code)]
pub fn normalize_v2_weights(exp: &mut BlendWeightExportV2) {
    let sum = total_weight_sum(exp);
    if sum > 0.0 {
        for w in &mut exp.weights {
            w.weight /= sum;
        }
    }
}

#[allow(dead_code)]
pub fn blend_weight_v2_to_json(exp: &BlendWeightExportV2) -> String {
    format!(
        "{{\"weight_count\":{},\"active_count\":{}}}",
        weight_count_v2(exp),
        active_weight_count(exp)
    )
}

#[allow(dead_code)]
pub fn weights_all_valid(exp: &BlendWeightExportV2) -> bool {
    exp.weights.iter().all(|w| (0.0..=1.0).contains(&w.weight))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let exp = new_blend_weight_export_v2();
        assert_eq!(weight_count_v2(&exp), 0);
    }

    #[test]
    fn test_add_weight() {
        let mut exp = new_blend_weight_export_v2();
        add_blend_weight_v2(&mut exp, "brow_raise", 0.5);
        assert_eq!(weight_count_v2(&exp), 1);
    }

    #[test]
    fn test_find_weight() {
        let mut exp = new_blend_weight_export_v2();
        add_blend_weight_v2(&mut exp, "smile", 0.8);
        assert!(find_weight_v2(&exp, "smile").is_some());
    }

    #[test]
    fn test_set_weight() {
        let mut exp = new_blend_weight_export_v2();
        add_blend_weight_v2(&mut exp, "blink", 0.0);
        set_weight_v2(&mut exp, "blink", 1.0);
        let w = find_weight_v2(&exp, "blink").expect("should succeed");
        assert!((w.weight - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_active_count() {
        let mut exp = new_blend_weight_export_v2();
        add_blend_weight_v2(&mut exp, "a", 0.5);
        add_blend_weight_v2(&mut exp, "b", 0.0);
        assert_eq!(active_weight_count(&exp), 1);
    }

    #[test]
    fn test_total_weight_sum() {
        let mut exp = new_blend_weight_export_v2();
        add_blend_weight_v2(&mut exp, "x", 0.3);
        add_blend_weight_v2(&mut exp, "y", 0.4);
        assert!((total_weight_sum(&exp) - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_weights() {
        let mut exp = new_blend_weight_export_v2();
        add_blend_weight_v2(&mut exp, "a", 1.0);
        add_blend_weight_v2(&mut exp, "b", 1.0);
        normalize_v2_weights(&mut exp);
        assert!((total_weight_sum(&exp) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_json_output() {
        let exp = new_blend_weight_export_v2();
        let j = blend_weight_v2_to_json(&exp);
        assert!(j.contains("weight_count"));
    }

    #[test]
    fn test_weights_valid() {
        let mut exp = new_blend_weight_export_v2();
        add_blend_weight_v2(&mut exp, "ok", 0.6);
        assert!(weights_all_valid(&exp));
    }

    #[test]
    fn test_clamp_on_add() {
        let mut exp = new_blend_weight_export_v2();
        add_blend_weight_v2(&mut exp, "clamped", 2.0);
        assert!((exp.weights[0].weight - 1.0).abs() < 1e-5);
    }
}
