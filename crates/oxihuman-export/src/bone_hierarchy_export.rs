// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bone hierarchy export: tree structure of joints/bones.

/// A single bone in the hierarchy.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HierarchyBone {
    pub name: String,
    pub parent: Option<u32>,
    pub head: [f32; 3],
    pub tail: [f32; 3],
}

/// Bone hierarchy export structure.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneHierarchyExport {
    pub bones: Vec<HierarchyBone>,
}

/// Create a new empty bone hierarchy.
#[allow(dead_code)]
pub fn new_bone_hierarchy() -> BoneHierarchyExport {
    BoneHierarchyExport { bones: Vec::new() }
}

/// Add a bone to the hierarchy.
#[allow(dead_code)]
pub fn add_hierarchy_bone(
    bh: &mut BoneHierarchyExport,
    name: &str,
    parent: Option<u32>,
    head: [f32; 3],
    tail: [f32; 3],
) {
    bh.bones.push(HierarchyBone {
        name: name.to_string(),
        parent,
        head,
        tail,
    });
}

/// Total bone count.
#[allow(dead_code)]
pub fn bone_hierarchy_count(bh: &BoneHierarchyExport) -> usize {
    bh.bones.len()
}

/// Root bones (no parent).
#[allow(dead_code)]
pub fn root_hierarchy_bones(bh: &BoneHierarchyExport) -> Vec<u32> {
    bh.bones
        .iter()
        .enumerate()
        .filter(|(_, b)| b.parent.is_none())
        .map(|(i, _)| i as u32)
        .collect()
}

/// Children of a bone.
#[allow(dead_code)]
pub fn children_of(bh: &BoneHierarchyExport, parent_idx: u32) -> Vec<u32> {
    bh.bones
        .iter()
        .enumerate()
        .filter(|(_, b)| b.parent.is_some_and(|p| p == parent_idx))
        .map(|(i, _)| i as u32)
        .collect()
}

/// Bone length.
#[allow(dead_code)]
pub fn bone_length(b: &HierarchyBone) -> f32 {
    let d = [
        b.tail[0] - b.head[0],
        b.tail[1] - b.head[1],
        b.tail[2] - b.head[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Hierarchy depth (max chain length from root).
#[allow(dead_code)]
pub fn hierarchy_depth(bh: &BoneHierarchyExport) -> usize {
    fn depth_of(bh: &BoneHierarchyExport, idx: u32, memo: &mut Vec<Option<usize>>) -> usize {
        if let Some(d) = memo[idx as usize] {
            return d;
        }
        let d = match bh.bones[idx as usize].parent {
            None => 0,
            Some(p) => 1 + depth_of(bh, p, memo),
        };
        memo[idx as usize] = Some(d);
        d
    }
    if bh.bones.is_empty() {
        return 0;
    }
    let mut memo = vec![None; bh.bones.len()];
    (0..bh.bones.len())
        .map(|i| depth_of(bh, i as u32, &mut memo))
        .max()
        .unwrap_or(0)
}

/// Validate: all parent indices are valid.
#[allow(dead_code)]
pub fn validate_bone_hierarchy(bh: &BoneHierarchyExport) -> bool {
    bh.bones
        .iter()
        .all(|b| b.parent.is_none_or(|p| (p as usize) < bh.bones.len()))
}

/// Export to JSON.
#[allow(dead_code)]
pub fn bone_hierarchy_to_json(bh: &BoneHierarchyExport) -> String {
    format!(
        "{{\"bone_count\":{},\"depth\":{}}}",
        bone_hierarchy_count(bh),
        hierarchy_depth(bh)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_hierarchy() -> BoneHierarchyExport {
        let mut bh = new_bone_hierarchy();
        add_hierarchy_bone(&mut bh, "root", None, [0.0; 3], [0.0, 1.0, 0.0]);
        add_hierarchy_bone(&mut bh, "child", Some(0), [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]);
        bh
    }

    #[test]
    fn test_new_bone_hierarchy() {
        let bh = new_bone_hierarchy();
        assert_eq!(bone_hierarchy_count(&bh), 0);
    }

    #[test]
    fn test_add_bones() {
        let bh = simple_hierarchy();
        assert_eq!(bone_hierarchy_count(&bh), 2);
    }

    #[test]
    fn test_root_bones() {
        let bh = simple_hierarchy();
        let roots = root_hierarchy_bones(&bh);
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0], 0);
    }

    #[test]
    fn test_children_of() {
        let bh = simple_hierarchy();
        let children = children_of(&bh, 0);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], 1);
    }

    #[test]
    fn test_bone_length() {
        let bh = simple_hierarchy();
        let l = bone_length(&bh.bones[0]);
        assert!((l - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_hierarchy_depth() {
        let bh = simple_hierarchy();
        assert_eq!(hierarchy_depth(&bh), 1);
    }

    #[test]
    fn test_validate() {
        let bh = simple_hierarchy();
        assert!(validate_bone_hierarchy(&bh));
    }

    #[test]
    fn test_validate_invalid_parent() {
        let mut bh = new_bone_hierarchy();
        add_hierarchy_bone(&mut bh, "bad", Some(99), [0.0; 3], [0.0; 3]);
        assert!(!validate_bone_hierarchy(&bh));
    }

    #[test]
    fn test_bone_hierarchy_to_json() {
        let bh = simple_hierarchy();
        let j = bone_hierarchy_to_json(&bh);
        assert!(j.contains("\"bone_count\":2"));
    }

    #[test]
    fn test_empty_depth() {
        let bh = new_bone_hierarchy();
        assert_eq!(hierarchy_depth(&bh), 0);
    }
}
