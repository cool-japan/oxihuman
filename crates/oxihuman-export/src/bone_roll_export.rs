// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bone roll angle export for skeleton rigs.

use std::f32::consts::PI;

/// Bone roll data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneRollExport {
    pub bone_names: Vec<String>,
    pub rolls: Vec<f32>,
}

/// Create new bone roll export.
#[allow(dead_code)]
pub fn new_bone_roll_export() -> BoneRollExport {
    BoneRollExport {
        bone_names: vec![],
        rolls: vec![],
    }
}

/// Add a bone roll entry (angle in radians).
#[allow(dead_code)]
pub fn add_bone_roll(e: &mut BoneRollExport, name: &str, roll: f32) {
    e.bone_names.push(name.to_string());
    e.rolls.push(roll);
}

/// Bone count.
#[allow(dead_code)]
pub fn br_bone_count(e: &BoneRollExport) -> usize {
    e.bone_names.len()
}

/// Get roll for bone index.
#[allow(dead_code)]
pub fn get_roll(e: &BoneRollExport, idx: usize) -> Option<f32> {
    e.rolls.get(idx).copied()
}

/// Get roll by name.
#[allow(dead_code)]
pub fn get_roll_by_name(e: &BoneRollExport, name: &str) -> Option<f32> {
    e.bone_names
        .iter()
        .position(|n| n == name)
        .and_then(|i| e.rolls.get(i).copied())
}

/// Average roll.
#[allow(dead_code)]
pub fn avg_roll(e: &BoneRollExport) -> f32 {
    if e.rolls.is_empty() {
        return 0.0;
    }
    e.rolls.iter().sum::<f32>() / e.rolls.len() as f32
}

/// Normalize roll to [-PI, PI].
#[allow(dead_code)]
pub fn normalize_roll(roll: f32) -> f32 {
    let mut r = roll % (2.0 * PI);
    if r > PI {
        r -= 2.0 * PI;
    }
    if r < -PI {
        r += 2.0 * PI;
    }
    r
}

/// Validate (all rolls finite).
#[allow(dead_code)]
pub fn br_validate(e: &BoneRollExport) -> bool {
    e.bone_names.len() == e.rolls.len() && e.rolls.iter().all(|r| r.is_finite())
}

/// Export to JSON.
#[allow(dead_code)]
pub fn bone_roll_to_json(e: &BoneRollExport) -> String {
    format!(
        "{{\"bone_count\":{},\"avg_roll\":{:.6}}}",
        br_bone_count(e),
        avg_roll(e)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let e = new_bone_roll_export();
        assert_eq!(br_bone_count(&e), 0);
    }
    #[test]
    fn test_add() {
        let mut e = new_bone_roll_export();
        add_bone_roll(&mut e, "arm", 0.5);
        assert_eq!(br_bone_count(&e), 1);
    }
    #[test]
    fn test_get_roll() {
        let mut e = new_bone_roll_export();
        add_bone_roll(&mut e, "leg", 1.0);
        assert!((get_roll(&e, 0).unwrap() - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_get_by_name() {
        let mut e = new_bone_roll_export();
        add_bone_roll(&mut e, "spine", 0.3);
        assert!(get_roll_by_name(&e, "spine").is_some());
    }
    #[test]
    fn test_get_by_name_missing() {
        let e = new_bone_roll_export();
        assert!(get_roll_by_name(&e, "x").is_none());
    }
    #[test]
    fn test_avg() {
        let mut e = new_bone_roll_export();
        add_bone_roll(&mut e, "a", 1.0);
        add_bone_roll(&mut e, "b", 3.0);
        assert!((avg_roll(&e) - 2.0).abs() < 1e-6);
    }
    #[test]
    fn test_normalize() {
        let r = normalize_roll(3.0 * PI);
        assert!(r.abs() <= PI + 1e-5);
    }
    #[test]
    fn test_validate() {
        let mut e = new_bone_roll_export();
        add_bone_roll(&mut e, "a", 0.0);
        assert!(br_validate(&e));
    }
    #[test]
    fn test_to_json() {
        let e = new_bone_roll_export();
        let j = bone_roll_to_json(&e);
        assert!(j.contains("\"bone_count\":0"));
    }
    #[test]
    fn test_avg_empty() {
        let e = new_bone_roll_export();
        assert!((avg_roll(&e)).abs() < 1e-9);
    }
}
