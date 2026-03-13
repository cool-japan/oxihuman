#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// Display state for a single bone in the armature viewport.
#[derive(Debug, Clone)]
pub struct BoneDisplayEntry {
    pub name: String,
    pub selected: bool,
    pub visible: bool,
    pub in_front: bool,
}

/// Armature/skeleton viewport display state.
#[derive(Debug, Clone)]
pub struct ArmatureView {
    pub bones: Vec<BoneDisplayEntry>,
    pub show_names: bool,
    pub show_axes: bool,
    pub draw_type: u8,
}

#[allow(dead_code)]
pub fn new_armature_view() -> ArmatureView {
    ArmatureView {
        bones: Vec::new(),
        show_names: true,
        show_axes: false,
        draw_type: 0,
    }
}

#[allow(dead_code)]
pub fn add_bone_entry(view: &mut ArmatureView, name: &str) {
    view.bones.push(BoneDisplayEntry {
        name: name.to_string(),
        selected: false,
        visible: true,
        in_front: false,
    });
}

#[allow(dead_code)]
pub fn select_bone(view: &mut ArmatureView, name: &str, sel: bool) {
    if let Some(b) = view.bones.iter_mut().find(|b| b.name == name) {
        b.selected = sel;
    }
}

#[allow(dead_code)]
pub fn visible_bones(view: &ArmatureView) -> Vec<&str> {
    view.bones
        .iter()
        .filter(|b| b.visible)
        .map(|b| b.name.as_str())
        .collect()
}

#[allow(dead_code)]
pub fn armature_bone_count(view: &ArmatureView) -> usize {
    view.bones.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_armature_view_empty() {
        let v = new_armature_view();
        assert_eq!(armature_bone_count(&v), 0);
        assert!(v.show_names);
    }

    #[test]
    fn test_add_bone_entry() {
        let mut v = new_armature_view();
        add_bone_entry(&mut v, "Hips");
        assert_eq!(armature_bone_count(&v), 1);
    }

    #[test]
    fn test_bone_visible_by_default() {
        let mut v = new_armature_view();
        add_bone_entry(&mut v, "Spine");
        assert!(v.bones[0].visible);
    }

    #[test]
    fn test_bone_not_selected_by_default() {
        let mut v = new_armature_view();
        add_bone_entry(&mut v, "Head");
        assert!(!v.bones[0].selected);
    }

    #[test]
    fn test_select_bone_true() {
        let mut v = new_armature_view();
        add_bone_entry(&mut v, "LeftArm");
        select_bone(&mut v, "LeftArm", true);
        assert!(v.bones[0].selected);
    }

    #[test]
    fn test_select_bone_false() {
        let mut v = new_armature_view();
        add_bone_entry(&mut v, "RightLeg");
        select_bone(&mut v, "RightLeg", true);
        select_bone(&mut v, "RightLeg", false);
        assert!(!v.bones[0].selected);
    }

    #[test]
    fn test_visible_bones_all() {
        let mut v = new_armature_view();
        add_bone_entry(&mut v, "Hips");
        add_bone_entry(&mut v, "Spine");
        let vis = visible_bones(&v);
        assert_eq!(vis.len(), 2);
    }

    #[test]
    fn test_visible_bones_some_hidden() {
        let mut v = new_armature_view();
        add_bone_entry(&mut v, "Hips");
        add_bone_entry(&mut v, "Hidden");
        v.bones[1].visible = false;
        let vis = visible_bones(&v);
        assert_eq!(vis.len(), 1);
        assert_eq!(vis[0], "Hips");
    }

    #[test]
    fn test_multiple_bones() {
        let mut v = new_armature_view();
        for name in &["A", "B", "C", "D"] {
            add_bone_entry(&mut v, name);
        }
        assert_eq!(armature_bone_count(&v), 4);
    }
}
