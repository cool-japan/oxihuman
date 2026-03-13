// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Joint name export: joint naming conventions and remapping tables.

/// A joint name entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointNameEntry {
    pub index: u32,
    pub name: String,
    pub parent_index: Option<u32>,
}

/// Joint name export bundle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointNameExport {
    pub joints: Vec<JointNameEntry>,
}

/// Create a new joint name export.
#[allow(dead_code)]
pub fn new_joint_name_export() -> JointNameExport {
    JointNameExport { joints: Vec::new() }
}

/// Add a joint.
#[allow(dead_code)]
pub fn add_joint_name(exp: &mut JointNameExport, index: u32, name: &str, parent: Option<u32>) {
    exp.joints.push(JointNameEntry {
        index,
        name: name.to_string(),
        parent_index: parent,
    });
}

/// Joint count.
#[allow(dead_code)]
pub fn joint_name_count(exp: &JointNameExport) -> usize {
    exp.joints.len()
}

/// Find joint by name.
#[allow(dead_code)]
pub fn find_joint_by_name<'a>(exp: &'a JointNameExport, name: &str) -> Option<&'a JointNameEntry> {
    exp.joints.iter().find(|j| j.name == name)
}

/// Find joint by index.
#[allow(dead_code)]
pub fn find_joint_by_index(exp: &JointNameExport, idx: u32) -> Option<&JointNameEntry> {
    exp.joints.iter().find(|j| j.index == idx)
}

/// Root joints (no parent).
#[allow(dead_code)]
pub fn root_joints(exp: &JointNameExport) -> Vec<&JointNameEntry> {
    exp.joints
        .iter()
        .filter(|j| j.parent_index.is_none())
        .collect()
}

/// Children of a joint.
#[allow(dead_code)]
pub fn children_of_joint(exp: &JointNameExport, parent_idx: u32) -> Vec<&JointNameEntry> {
    exp.joints
        .iter()
        .filter(|j| j.parent_index == Some(parent_idx))
        .collect()
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn joint_name_to_json(exp: &JointNameExport) -> String {
    format!(
        "{{\"joint_count\":{},\"root_count\":{}}}",
        joint_name_count(exp),
        root_joints(exp).len()
    )
}

/// Validate: no duplicate names.
#[allow(dead_code)]
pub fn validate_joint_names(exp: &JointNameExport) -> bool {
    let mut names: Vec<&str> = exp.joints.iter().map(|j| j.name.as_str()).collect();
    let orig_len = names.len();
    names.sort_unstable();
    names.dedup();
    names.len() == orig_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_joint_name_export();
        assert_eq!(joint_name_count(&exp), 0);
    }

    #[test]
    fn add_joint_increments() {
        let mut exp = new_joint_name_export();
        add_joint_name(&mut exp, 0, "root", None);
        assert_eq!(joint_name_count(&exp), 1);
    }

    #[test]
    fn find_by_name_some() {
        let mut exp = new_joint_name_export();
        add_joint_name(&mut exp, 0, "hip", None);
        assert!(find_joint_by_name(&exp, "hip").is_some());
    }

    #[test]
    fn find_by_index_some() {
        let mut exp = new_joint_name_export();
        add_joint_name(&mut exp, 5, "spine", None);
        assert!(find_joint_by_index(&exp, 5).is_some());
    }

    #[test]
    fn root_joints_no_parent() {
        let mut exp = new_joint_name_export();
        add_joint_name(&mut exp, 0, "root", None);
        add_joint_name(&mut exp, 1, "child", Some(0));
        assert_eq!(root_joints(&exp).len(), 1);
    }

    #[test]
    fn children_of_joint_correct() {
        let mut exp = new_joint_name_export();
        add_joint_name(&mut exp, 0, "root", None);
        add_joint_name(&mut exp, 1, "c1", Some(0));
        add_joint_name(&mut exp, 2, "c2", Some(0));
        assert_eq!(children_of_joint(&exp, 0).len(), 2);
    }

    #[test]
    fn validate_no_dup_names() {
        let mut exp = new_joint_name_export();
        add_joint_name(&mut exp, 0, "a", None);
        add_joint_name(&mut exp, 1, "b", None);
        assert!(validate_joint_names(&exp));
    }

    #[test]
    fn json_contains_joint_count() {
        let exp = new_joint_name_export();
        let j = joint_name_to_json(&exp);
        assert!(j.contains("joint_count"));
    }

    #[test]
    fn find_missing_none() {
        let exp = new_joint_name_export();
        assert!(find_joint_by_name(&exp, "ghost").is_none());
    }

    #[test]
    fn contains_range() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
