// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export joint parent-child hierarchy data.

/// A single joint with optional parent index.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointRecord {
    pub name: String,
    pub parent_index: Option<usize>,
    pub local_position: [f32; 3],
}

/// Joint hierarchy export.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct JointParentExport {
    pub joints: Vec<JointRecord>,
}

/// Create a new export.
#[allow(dead_code)]
pub fn new_joint_parent_export() -> JointParentExport {
    JointParentExport::default()
}

/// Add a joint.
#[allow(dead_code)]
pub fn add_joint(
    export: &mut JointParentExport,
    name: &str,
    parent_index: Option<usize>,
    local_position: [f32; 3],
) {
    export.joints.push(JointRecord {
        name: name.to_string(),
        parent_index,
        local_position,
    });
}

/// Get children indices for a joint.
#[allow(dead_code)]
pub fn children_of(export: &JointParentExport, joint_idx: usize) -> Vec<usize> {
    export
        .joints
        .iter()
        .enumerate()
        .filter_map(|(i, j)| {
            if j.parent_index == Some(joint_idx) {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

/// Get root joints (those with no parent).
#[allow(dead_code)]
pub fn root_joints(export: &JointParentExport) -> Vec<usize> {
    export
        .joints
        .iter()
        .enumerate()
        .filter_map(|(i, j)| {
            if j.parent_index.is_none() {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

/// Compute the world position of a joint (sum of local positions up the chain).
#[allow(dead_code)]
pub fn world_position(export: &JointParentExport, joint_idx: usize) -> [f32; 3] {
    let mut pos = [0.0_f32; 3];
    let mut current = joint_idx;
    loop {
        let j = &export.joints[current];
        pos[0] += j.local_position[0];
        pos[1] += j.local_position[1];
        pos[2] += j.local_position[2];
        if let Some(p) = j.parent_index {
            current = p;
        } else {
            break;
        }
    }
    pos
}

/// Find a joint by name.
#[allow(dead_code)]
pub fn find_joint(export: &JointParentExport, name: &str) -> Option<usize> {
    export.joints.iter().position(|j| j.name == name)
}

/// Depth (number of ancestors) of a joint.
#[allow(dead_code)]
pub fn joint_depth(export: &JointParentExport, joint_idx: usize) -> usize {
    let mut depth = 0;
    let mut current = joint_idx;
    while let Some(p) = export.joints[current].parent_index {
        depth += 1;
        current = p;
    }
    depth
}

/// Serialise hierarchy to flat i32 buffer (-1 for no parent).
#[allow(dead_code)]
pub fn serialise_parents(export: &JointParentExport) -> Vec<i32> {
    export
        .joints
        .iter()
        .map(|j| j.parent_index.map(|p| p as i32).unwrap_or(-1))
        .collect()
}

/// Count joints.
#[allow(dead_code)]
pub fn joint_count(export: &JointParentExport) -> usize {
    export.joints.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three_joint_chain() -> JointParentExport {
        let mut e = new_joint_parent_export();
        add_joint(&mut e, "root", None, [0.0, 0.0, 0.0]);
        add_joint(&mut e, "spine", Some(0), [0.0, 1.0, 0.0]);
        add_joint(&mut e, "chest", Some(1), [0.0, 1.0, 0.0]);
        e
    }

    #[test]
    fn test_root_joints() {
        let e = three_joint_chain();
        assert_eq!(root_joints(&e), vec![0]);
    }

    #[test]
    fn test_children_of() {
        let e = three_joint_chain();
        assert_eq!(children_of(&e, 0), vec![1]);
        assert_eq!(children_of(&e, 1), vec![2]);
    }

    #[test]
    fn test_world_position_root() {
        let e = three_joint_chain();
        let p = world_position(&e, 0);
        assert_eq!(p, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_world_position_leaf() {
        let e = three_joint_chain();
        let p = world_position(&e, 2);
        assert!((p[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_find_joint() {
        let e = three_joint_chain();
        assert_eq!(find_joint(&e, "chest"), Some(2));
        assert!(find_joint(&e, "head").is_none());
    }

    #[test]
    fn test_joint_depth_root() {
        let e = three_joint_chain();
        assert_eq!(joint_depth(&e, 0), 0);
    }

    #[test]
    fn test_joint_depth_leaf() {
        let e = three_joint_chain();
        assert_eq!(joint_depth(&e, 2), 2);
    }

    #[test]
    fn test_serialise_parents() {
        let e = three_joint_chain();
        let p = serialise_parents(&e);
        assert_eq!(p, vec![-1, 0, 1]);
    }

    #[test]
    fn test_joint_count() {
        let e = three_joint_chain();
        assert_eq!(joint_count(&e), 3);
    }

    #[test]
    fn test_empty_export() {
        let e = new_joint_parent_export();
        assert!(root_joints(&e).is_empty());
    }
}
