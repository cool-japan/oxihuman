// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A named group of morph targets with a shared group weight.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphGroup {
    pub name: String,
    pub members: Vec<String>,
    pub weight: f32,
}

/// Create a new empty morph group.
#[allow(dead_code)]
pub fn new_morph_group(name: &str) -> MorphGroup {
    MorphGroup {
        name: name.to_string(),
        members: Vec::new(),
        weight: 0.0,
    }
}

/// Add a morph target name to the group.
#[allow(dead_code)]
pub fn add_to_group(group: &mut MorphGroup, target: &str) {
    if !group.members.contains(&target.to_string()) {
        group.members.push(target.to_string());
    }
}

/// Return the number of members in the group.
#[allow(dead_code)]
pub fn group_member_count(group: &MorphGroup) -> usize {
    group.members.len()
}

/// Set the group weight (clamped to 0..=1).
#[allow(dead_code)]
pub fn group_set_weight(group: &mut MorphGroup, w: f32) {
    group.weight = w.clamp(0.0, 1.0);
}

/// Get the current group weight.
#[allow(dead_code)]
pub fn group_weight(group: &MorphGroup) -> f32 {
    group.weight
}

/// Evaluate the group, returning each member paired with the group weight.
#[allow(dead_code)]
pub fn evaluate_group(group: &MorphGroup) -> Vec<(String, f32)> {
    group
        .members
        .iter()
        .map(|m| (m.clone(), group.weight))
        .collect()
}

/// Return the group name.
#[allow(dead_code)]
pub fn group_name_mg(group: &MorphGroup) -> &str {
    &group.name
}

/// Serialize the group to a JSON string.
#[allow(dead_code)]
pub fn group_to_json(group: &MorphGroup) -> String {
    let members: Vec<String> = group.members.iter().map(|m| format!("\"{}\"", m)).collect();
    format!(
        "{{\"name\":\"{}\",\"weight\":{:.4},\"members\":[{}]}}",
        group.name,
        group.weight,
        members.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_group_is_empty() {
        let g = new_morph_group("test");
        assert_eq!(group_member_count(&g), 0);
    }

    #[test]
    fn add_member() {
        let mut g = new_morph_group("g");
        add_to_group(&mut g, "smile");
        assert_eq!(group_member_count(&g), 1);
    }

    #[test]
    fn no_duplicate_members() {
        let mut g = new_morph_group("g");
        add_to_group(&mut g, "smile");
        add_to_group(&mut g, "smile");
        assert_eq!(group_member_count(&g), 1);
    }

    #[test]
    fn set_and_get_weight() {
        let mut g = new_morph_group("g");
        group_set_weight(&mut g, 0.75);
        assert!((group_weight(&g) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn weight_clamped_high() {
        let mut g = new_morph_group("g");
        group_set_weight(&mut g, 1.5);
        assert!((group_weight(&g) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn weight_clamped_low() {
        let mut g = new_morph_group("g");
        group_set_weight(&mut g, -0.5);
        assert!(group_weight(&g).abs() < 1e-6);
    }

    #[test]
    fn evaluate_returns_pairs() {
        let mut g = new_morph_group("g");
        add_to_group(&mut g, "a");
        add_to_group(&mut g, "b");
        group_set_weight(&mut g, 0.5);
        let pairs = evaluate_group(&g);
        assert_eq!(pairs.len(), 2);
        assert!((pairs[0].1 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn name_accessor() {
        let g = new_morph_group("face");
        assert_eq!(group_name_mg(&g), "face");
    }

    #[test]
    fn to_json_not_empty() {
        let g = new_morph_group("test");
        let j = group_to_json(&g);
        assert!(j.contains("\"name\":\"test\""));
    }

    #[test]
    fn evaluate_empty_group() {
        let g = new_morph_group("empty");
        let pairs = evaluate_group(&g);
        assert!(pairs.is_empty());
    }
}
