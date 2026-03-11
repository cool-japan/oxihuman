// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! IK influence weight export per joint.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IkWeightEntry {
    pub joint_name: String,
    pub ik_weight: f32,
    pub fk_weight: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IkWeightExport {
    pub entries: Vec<IkWeightEntry>,
}

#[allow(dead_code)]
pub fn new_ik_weight_export() -> IkWeightExport {
    IkWeightExport {
        entries: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_ik_weight(exp: &mut IkWeightExport, joint: &str, ik: f32, fk: f32) {
    exp.entries.push(IkWeightEntry {
        joint_name: joint.to_string(),
        ik_weight: ik.clamp(0.0, 1.0),
        fk_weight: fk.clamp(0.0, 1.0),
    });
}

#[allow(dead_code)]
pub fn entry_count_ikw(exp: &IkWeightExport) -> usize {
    exp.entries.len()
}

#[allow(dead_code)]
pub fn find_ik_entry<'a>(exp: &'a IkWeightExport, joint: &str) -> Option<&'a IkWeightEntry> {
    exp.entries.iter().find(|e| e.joint_name == joint)
}

#[allow(dead_code)]
pub fn avg_ik_weight(exp: &IkWeightExport) -> f32 {
    if exp.entries.is_empty() {
        return 0.0;
    }
    exp.entries.iter().map(|e| e.ik_weight).sum::<f32>() / exp.entries.len() as f32
}

#[allow(dead_code)]
pub fn fully_ik_joints(exp: &IkWeightExport) -> usize {
    exp.entries.iter().filter(|e| e.ik_weight >= 1.0).count()
}

#[allow(dead_code)]
pub fn set_ik_fk_blend(exp: &mut IkWeightExport, joint: &str, ik: f32) {
    if let Some(e) = exp.entries.iter_mut().find(|e| e.joint_name == joint) {
        e.ik_weight = ik.clamp(0.0, 1.0);
        e.fk_weight = (1.0 - ik).clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn ik_weight_to_json(exp: &IkWeightExport) -> String {
    format!(
        "{{\"entry_count\":{},\"avg_ik\":{}}}",
        entry_count_ikw(exp),
        avg_ik_weight(exp)
    )
}

#[allow(dead_code)]
pub fn weights_sum_to_one(exp: &IkWeightExport) -> bool {
    exp.entries
        .iter()
        .all(|e| ((e.ik_weight + e.fk_weight) - 1.0).abs() < 1e-4)
}

#[allow(dead_code)]
pub fn normalize_ik_fk(exp: &mut IkWeightExport) {
    for e in &mut exp.entries {
        let sum = e.ik_weight + e.fk_weight;
        if sum > 0.0 {
            e.ik_weight /= sum;
            e.fk_weight /= sum;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let exp = new_ik_weight_export();
        assert_eq!(entry_count_ikw(&exp), 0);
    }

    #[test]
    fn test_add_entry() {
        let mut exp = new_ik_weight_export();
        add_ik_weight(&mut exp, "knee", 1.0, 0.0);
        assert_eq!(entry_count_ikw(&exp), 1);
    }

    #[test]
    fn test_find_entry() {
        let mut exp = new_ik_weight_export();
        add_ik_weight(&mut exp, "elbow", 0.5, 0.5);
        assert!(find_ik_entry(&exp, "elbow").is_some());
    }

    #[test]
    fn test_avg_ik_weight() {
        let mut exp = new_ik_weight_export();
        add_ik_weight(&mut exp, "a", 1.0, 0.0);
        add_ik_weight(&mut exp, "b", 0.0, 1.0);
        assert!((avg_ik_weight(&exp) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_fully_ik_joints() {
        let mut exp = new_ik_weight_export();
        add_ik_weight(&mut exp, "arm", 1.0, 0.0);
        add_ik_weight(&mut exp, "hip", 0.3, 0.7);
        assert_eq!(fully_ik_joints(&exp), 1);
    }

    #[test]
    fn test_set_blend() {
        let mut exp = new_ik_weight_export();
        add_ik_weight(&mut exp, "wrist", 0.0, 1.0);
        set_ik_fk_blend(&mut exp, "wrist", 0.8);
        let e = find_ik_entry(&exp, "wrist").unwrap();
        assert!((e.ik_weight - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_json_output() {
        let exp = new_ik_weight_export();
        let j = ik_weight_to_json(&exp);
        assert!(j.contains("entry_count"));
    }

    #[test]
    fn test_normalize() {
        let mut exp = new_ik_weight_export();
        add_ik_weight(&mut exp, "x", 2.0, 2.0);
        normalize_ik_fk(&mut exp);
        assert!(weights_sum_to_one(&exp));
    }

    #[test]
    fn test_clamp_on_add() {
        let mut exp = new_ik_weight_export();
        add_ik_weight(&mut exp, "y", 3.0, -1.0);
        assert!((exp.entries[0].ik_weight - 1.0).abs() < 1e-5);
        assert!((exp.entries[0].fk_weight).abs() < 1e-5);
    }

    #[test]
    fn test_avg_empty_zero() {
        let exp = new_ik_weight_export();
        assert!((avg_ik_weight(&exp)).abs() < 1e-6);
    }
}
