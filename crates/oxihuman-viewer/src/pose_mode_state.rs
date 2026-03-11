#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Pose mode state (IK, selected bones, transforms).
#[derive(Debug, Clone)]
pub struct PoseModeState {
    pub selected_bones: Vec<String>,
    pub auto_ik: bool,
    pub show_ik_lines: bool,
    pub ik_chain_length: u32,
}

#[allow(dead_code)]
pub fn new_pose_mode_state() -> PoseModeState {
    PoseModeState {
        selected_bones: Vec::new(),
        auto_ik: false,
        show_ik_lines: true,
        ik_chain_length: 2,
    }
}

#[allow(dead_code)]
pub fn select_pose_bone(state: &mut PoseModeState, name: &str) {
    if !state.selected_bones.iter().any(|s| s == name) {
        state.selected_bones.push(name.to_string());
    }
}

#[allow(dead_code)]
pub fn deselect_pose_bones(state: &mut PoseModeState) {
    state.selected_bones.clear();
}

#[allow(dead_code)]
pub fn selected_bone_count(state: &PoseModeState) -> usize {
    state.selected_bones.len()
}

#[allow(dead_code)]
pub fn toggle_auto_ik(state: &mut PoseModeState) {
    state.auto_ik = !state.auto_ik;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state_empty() {
        let s = new_pose_mode_state();
        assert_eq!(selected_bone_count(&s), 0);
    }

    #[test]
    fn test_select_pose_bone() {
        let mut s = new_pose_mode_state();
        select_pose_bone(&mut s, "Head");
        assert_eq!(selected_bone_count(&s), 1);
    }

    #[test]
    fn test_no_duplicate_selection() {
        let mut s = new_pose_mode_state();
        select_pose_bone(&mut s, "Spine");
        select_pose_bone(&mut s, "Spine");
        assert_eq!(selected_bone_count(&s), 1);
    }

    #[test]
    fn test_deselect_all() {
        let mut s = new_pose_mode_state();
        select_pose_bone(&mut s, "LeftArm");
        select_pose_bone(&mut s, "RightArm");
        deselect_pose_bones(&mut s);
        assert_eq!(selected_bone_count(&s), 0);
    }

    #[test]
    fn test_toggle_auto_ik_off_to_on() {
        let mut s = new_pose_mode_state();
        assert!(!s.auto_ik);
        toggle_auto_ik(&mut s);
        assert!(s.auto_ik);
    }

    #[test]
    fn test_toggle_auto_ik_on_to_off() {
        let mut s = new_pose_mode_state();
        toggle_auto_ik(&mut s);
        toggle_auto_ik(&mut s);
        assert!(!s.auto_ik);
    }

    #[test]
    fn test_show_ik_lines_default_true() {
        let s = new_pose_mode_state();
        assert!(s.show_ik_lines);
    }

    #[test]
    fn test_ik_chain_length_default() {
        let s = new_pose_mode_state();
        assert_eq!(s.ik_chain_length, 2);
    }

    #[test]
    fn test_multiple_bones_selected() {
        let mut s = new_pose_mode_state();
        for bone in &["A", "B", "C"] {
            select_pose_bone(&mut s, bone);
        }
        assert_eq!(selected_bone_count(&s), 3);
    }
}
