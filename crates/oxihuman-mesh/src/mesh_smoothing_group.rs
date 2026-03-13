// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Smoothing group assignment for hard/soft edges.

use std::collections::HashSet;

#[allow(dead_code)]
pub struct SmoothingGroups {
    pub face_groups: Vec<u32>,
}

#[allow(dead_code)]
pub fn new_smoothing_groups(n_faces: usize) -> SmoothingGroups {
    SmoothingGroups { face_groups: vec![0u32; n_faces] }
}

#[allow(dead_code)]
pub fn sg_set(sg: &mut SmoothingGroups, face_idx: usize, group: u32) {
    if face_idx < sg.face_groups.len() {
        sg.face_groups[face_idx] = group;
    }
}

#[allow(dead_code)]
pub fn sg_get(sg: &SmoothingGroups, face_idx: usize) -> u32 {
    if face_idx < sg.face_groups.len() { sg.face_groups[face_idx] } else { 0 }
}

#[allow(dead_code)]
pub fn sg_group_count(sg: &SmoothingGroups) -> usize {
    let set: HashSet<u32> = sg.face_groups.iter().copied().collect();
    set.len()
}

#[allow(dead_code)]
pub fn sg_faces_in_group(sg: &SmoothingGroups, group: u32) -> Vec<usize> {
    sg.face_groups
        .iter()
        .enumerate()
        .filter(|&(_, &g)| g == group)
        .map(|(i, _)| i)
        .collect()
}

#[allow(dead_code)]
pub fn sg_is_smooth_edge(sg: &SmoothingGroups, face_a: usize, face_b: usize) -> bool {
    sg_get(sg, face_a) == sg_get(sg, face_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_all_zero() {
        let sg = new_smoothing_groups(5);
        assert!(sg.face_groups.iter().all(|&g| g == 0));
    }

    #[test]
    fn test_set_get() {
        let mut sg = new_smoothing_groups(3);
        sg_set(&mut sg, 1, 2);
        assert_eq!(sg_get(&sg, 1), 2);
    }

    #[test]
    fn test_group_count_all_same() {
        let sg = new_smoothing_groups(4);
        assert_eq!(sg_group_count(&sg), 1);
    }

    #[test]
    fn test_group_count_multiple() {
        let mut sg = new_smoothing_groups(4);
        sg_set(&mut sg, 0, 1);
        sg_set(&mut sg, 1, 2);
        sg_set(&mut sg, 2, 1);
        sg_set(&mut sg, 3, 3);
        assert_eq!(sg_group_count(&sg), 3);
    }

    #[test]
    fn test_faces_in_group() {
        let mut sg = new_smoothing_groups(4);
        sg_set(&mut sg, 0, 1);
        sg_set(&mut sg, 2, 1);
        let faces = sg_faces_in_group(&sg, 1);
        assert_eq!(faces, vec![0, 2]);
    }

    #[test]
    fn test_is_smooth_edge_same_group() {
        let sg = new_smoothing_groups(3);
        assert!(sg_is_smooth_edge(&sg, 0, 1));
    }

    #[test]
    fn test_is_smooth_edge_diff_group() {
        let mut sg = new_smoothing_groups(3);
        sg_set(&mut sg, 0, 1);
        sg_set(&mut sg, 1, 2);
        assert!(!sg_is_smooth_edge(&sg, 0, 1));
    }

    #[test]
    fn test_faces_in_group_empty() {
        let sg = new_smoothing_groups(3);
        let faces = sg_faces_in_group(&sg, 99);
        assert!(faces.is_empty());
    }
}
