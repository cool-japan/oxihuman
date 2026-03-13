// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-joint non-uniform scale export for skeleton rigs.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointScaleEntry {
    pub joint_name: String,
    pub scale: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointScaleExport {
    pub entries: Vec<JointScaleEntry>,
}

#[allow(dead_code)]
pub fn new_joint_scale_export() -> JointScaleExport {
    JointScaleExport {
        entries: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_joint_scale(exp: &mut JointScaleExport, joint: &str, scale: [f32; 3]) {
    exp.entries.push(JointScaleEntry {
        joint_name: joint.to_string(),
        scale,
    });
}

#[allow(dead_code)]
pub fn joint_scale_count(exp: &JointScaleExport) -> usize {
    exp.entries.len()
}

#[allow(dead_code)]
pub fn find_joint_scale<'a>(exp: &'a JointScaleExport, joint: &str) -> Option<&'a JointScaleEntry> {
    exp.entries.iter().find(|e| e.joint_name == joint)
}

#[allow(dead_code)]
pub fn is_uniform_scale(entry: &JointScaleEntry) -> bool {
    let [x, y, z] = entry.scale;
    (x - y).abs() < 1e-5 && (y - z).abs() < 1e-5
}

#[allow(dead_code)]
pub fn uniform_scale_count(exp: &JointScaleExport) -> usize {
    exp.entries.iter().filter(|e| is_uniform_scale(e)).count()
}

#[allow(dead_code)]
pub fn avg_scale_magnitude(exp: &JointScaleExport) -> f32 {
    if exp.entries.is_empty() {
        return 0.0;
    }
    let sum: f32 = exp
        .entries
        .iter()
        .map(|e| {
            let [x, y, z] = e.scale;
            (x * x + y * y + z * z).sqrt() / 3.0_f32.sqrt()
        })
        .sum();
    sum / exp.entries.len() as f32
}

#[allow(dead_code)]
pub fn set_scale(exp: &mut JointScaleExport, joint: &str, scale: [f32; 3]) {
    if let Some(e) = exp.entries.iter_mut().find(|e| e.joint_name == joint) {
        e.scale = scale;
    }
}

#[allow(dead_code)]
pub fn joint_scale_to_json(exp: &JointScaleExport) -> String {
    format!(
        "{{\"entry_count\":{},\"uniform_count\":{}}}",
        joint_scale_count(exp),
        uniform_scale_count(exp)
    )
}

#[allow(dead_code)]
pub fn scales_positive(exp: &JointScaleExport) -> bool {
    exp.entries.iter().all(|e| e.scale.iter().all(|&s| s > 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let exp = new_joint_scale_export();
        assert_eq!(joint_scale_count(&exp), 0);
    }

    #[test]
    fn test_add_entry() {
        let mut exp = new_joint_scale_export();
        add_joint_scale(&mut exp, "spine", [1.0, 1.2, 1.0]);
        assert_eq!(joint_scale_count(&exp), 1);
    }

    #[test]
    fn test_find_entry() {
        let mut exp = new_joint_scale_export();
        add_joint_scale(&mut exp, "hip", [1.0; 3]);
        assert!(find_joint_scale(&exp, "hip").is_some());
    }

    #[test]
    fn test_is_uniform() {
        let e = JointScaleEntry {
            joint_name: "x".to_string(),
            scale: [2.0, 2.0, 2.0],
        };
        assert!(is_uniform_scale(&e));
    }

    #[test]
    fn test_not_uniform() {
        let e = JointScaleEntry {
            joint_name: "x".to_string(),
            scale: [1.0, 2.0, 1.0],
        };
        assert!(!is_uniform_scale(&e));
    }

    #[test]
    fn test_uniform_count() {
        let mut exp = new_joint_scale_export();
        add_joint_scale(&mut exp, "a", [1.0; 3]);
        add_joint_scale(&mut exp, "b", [1.0, 2.0, 1.0]);
        assert_eq!(uniform_scale_count(&exp), 1);
    }

    #[test]
    fn test_set_scale() {
        let mut exp = new_joint_scale_export();
        add_joint_scale(&mut exp, "knee", [1.0; 3]);
        set_scale(&mut exp, "knee", [2.0, 2.0, 2.0]);
        let e = find_joint_scale(&exp, "knee").expect("should succeed");
        assert!((e.scale[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_json_output() {
        let exp = new_joint_scale_export();
        let j = joint_scale_to_json(&exp);
        assert!(j.contains("entry_count"));
    }

    #[test]
    fn test_scales_positive() {
        let mut exp = new_joint_scale_export();
        add_joint_scale(&mut exp, "ok", [1.0, 0.5, 2.0]);
        assert!(scales_positive(&exp));
    }

    #[test]
    fn test_avg_scale_magnitude_one() {
        let mut exp = new_joint_scale_export();
        add_joint_scale(&mut exp, "unit", [1.0; 3]);
        assert!((avg_scale_magnitude(&exp) - 1.0).abs() < 1e-4);
    }
}
