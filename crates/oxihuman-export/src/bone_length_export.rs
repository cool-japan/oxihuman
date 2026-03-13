// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bone length measurement and export utilities.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneLengthEntry {
    pub name: String,
    pub head: [f32; 3],
    pub tail: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneLengthExport {
    pub bones: Vec<BoneLengthEntry>,
}

#[allow(dead_code)]
pub fn new_bone_length_export() -> BoneLengthExport {
    BoneLengthExport { bones: Vec::new() }
}

#[allow(dead_code)]
pub fn add_bone_length(exp: &mut BoneLengthExport, name: &str, head: [f32; 3], tail: [f32; 3]) {
    exp.bones.push(BoneLengthEntry {
        name: name.to_string(),
        head,
        tail,
    });
}

#[allow(dead_code)]
pub fn bone_count_ble(exp: &BoneLengthExport) -> usize {
    exp.bones.len()
}

#[allow(dead_code)]
pub fn bone_length(entry: &BoneLengthEntry) -> f32 {
    let d = [
        entry.tail[0] - entry.head[0],
        entry.tail[1] - entry.head[1],
        entry.tail[2] - entry.head[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

#[allow(dead_code)]
pub fn total_bone_length_ble(exp: &BoneLengthExport) -> f32 {
    exp.bones.iter().map(bone_length).sum()
}

#[allow(dead_code)]
pub fn max_bone_length(exp: &BoneLengthExport) -> f32 {
    exp.bones.iter().map(bone_length).fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn min_bone_length(exp: &BoneLengthExport) -> f32 {
    if exp.bones.is_empty() {
        return 0.0;
    }
    exp.bones.iter().map(bone_length).fold(f32::MAX, f32::min)
}

#[allow(dead_code)]
pub fn find_bone_ble<'a>(exp: &'a BoneLengthExport, name: &str) -> Option<&'a BoneLengthEntry> {
    exp.bones.iter().find(|b| b.name == name)
}

#[allow(dead_code)]
pub fn bone_length_to_json(exp: &BoneLengthExport) -> String {
    format!(
        "{{\"bone_count\":{},\"total_length\":{}}}",
        bone_count_ble(exp),
        total_bone_length_ble(exp)
    )
}

#[allow(dead_code)]
pub fn bone_lengths_to_csv(exp: &BoneLengthExport) -> String {
    let mut s = "name,length\n".to_string();
    for b in &exp.bones {
        s.push_str(&format!("{},{}\n", b.name, bone_length(b)));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BoneLengthExport {
        let mut exp = new_bone_length_export();
        add_bone_length(&mut exp, "upper_arm", [0.0, 0.0, 0.0], [0.3, 0.0, 0.0]);
        add_bone_length(&mut exp, "lower_arm", [0.3, 0.0, 0.0], [0.55, 0.0, 0.0]);
        exp
    }

    #[test]
    fn test_empty() {
        let exp = new_bone_length_export();
        assert_eq!(bone_count_ble(&exp), 0);
    }

    #[test]
    fn test_add_bone() {
        let exp = sample();
        assert_eq!(bone_count_ble(&exp), 2);
    }

    #[test]
    fn test_bone_length() {
        let exp = sample();
        assert!((bone_length(&exp.bones[0]) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_total_length() {
        let exp = sample();
        assert!((total_bone_length_ble(&exp) - 0.55).abs() < 1e-4);
    }

    #[test]
    fn test_max_length() {
        let exp = sample();
        assert!(max_bone_length(&exp) >= 0.3);
    }

    #[test]
    fn test_min_length() {
        let exp = sample();
        assert!(min_bone_length(&exp) > 0.0);
    }

    #[test]
    fn test_find_bone() {
        let exp = sample();
        assert!(find_bone_ble(&exp, "upper_arm").is_some());
    }

    #[test]
    fn test_json_output() {
        let exp = sample();
        let j = bone_length_to_json(&exp);
        assert!(j.contains("bone_count"));
    }

    #[test]
    fn test_csv_header() {
        let exp = sample();
        let csv = bone_lengths_to_csv(&exp);
        assert!(csv.starts_with("name,length"));
    }

    #[test]
    fn test_min_empty() {
        let exp = new_bone_length_export();
        assert!((min_bone_length(&exp)).abs() < 1e-6);
    }
}
