// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Face smooth group export: per-face smooth/flat shading group data.

/// Smooth group flags per face (bit field).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceSmoothExport {
    pub groups: Vec<u32>, // per-face smooth group bitmask
}

/// Create a face smooth export with all faces in the same group.
#[allow(dead_code)]
pub fn new_face_smooth_export(face_count: usize, default_group: u32) -> FaceSmoothExport {
    FaceSmoothExport {
        groups: vec![default_group; face_count],
    }
}

/// Set smooth group for a face.
#[allow(dead_code)]
pub fn set_smooth_group(exp: &mut FaceSmoothExport, face: usize, group: u32) {
    if face < exp.groups.len() {
        exp.groups[face] = group;
    }
}

/// Get smooth group of a face.
#[allow(dead_code)]
pub fn get_smooth_group(exp: &FaceSmoothExport, face: usize) -> Option<u32> {
    exp.groups.get(face).copied()
}

/// Set flat shading on a face (group 0).
#[allow(dead_code)]
pub fn set_flat(exp: &mut FaceSmoothExport, face: usize) {
    set_smooth_group(exp, face, 0);
}

/// Count faces in a given smooth group.
#[allow(dead_code)]
pub fn count_in_group(exp: &FaceSmoothExport, group: u32) -> usize {
    exp.groups.iter().filter(|&&g| g == group).count()
}

/// Flat face count (group 0).
#[allow(dead_code)]
pub fn flat_face_count(exp: &FaceSmoothExport) -> usize {
    count_in_group(exp, 0)
}

/// Smooth face count (group != 0).
#[allow(dead_code)]
pub fn smooth_face_count(exp: &FaceSmoothExport) -> usize {
    exp.groups.iter().filter(|&&g| g != 0).count()
}

/// Number of distinct smooth groups.
#[allow(dead_code)]
pub fn distinct_group_count(exp: &FaceSmoothExport) -> usize {
    let mut v: Vec<u32> = exp.groups.clone();
    v.sort_unstable();
    v.dedup();
    v.len()
}

/// Face count.
#[allow(dead_code)]
pub fn face_smooth_face_count(exp: &FaceSmoothExport) -> usize {
    exp.groups.len()
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn face_smooth_to_json(exp: &FaceSmoothExport) -> String {
    format!(
        "{{\"face_count\":{},\"distinct_groups\":{}}}",
        face_smooth_face_count(exp),
        distinct_group_count(exp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_all_same() {
        let exp = new_face_smooth_export(4, 1);
        assert!(exp.groups.iter().all(|&g| g == 1));
    }

    #[test]
    fn set_and_get() {
        let mut exp = new_face_smooth_export(4, 1);
        set_smooth_group(&mut exp, 2, 3);
        assert_eq!(get_smooth_group(&exp, 2), Some(3));
    }

    #[test]
    fn set_flat_works() {
        let mut exp = new_face_smooth_export(3, 1);
        set_flat(&mut exp, 1);
        assert_eq!(flat_face_count(&exp), 1);
    }

    #[test]
    fn count_in_group_test() {
        let exp = new_face_smooth_export(5, 2);
        assert_eq!(count_in_group(&exp, 2), 5);
    }

    #[test]
    fn smooth_count_initial() {
        let exp = new_face_smooth_export(4, 1);
        assert_eq!(smooth_face_count(&exp), 4);
    }

    #[test]
    fn distinct_groups_initial() {
        let exp = new_face_smooth_export(5, 1);
        assert_eq!(distinct_group_count(&exp), 1);
    }

    #[test]
    fn distinct_groups_after_mix() {
        let mut exp = new_face_smooth_export(4, 1);
        set_smooth_group(&mut exp, 0, 2);
        assert_eq!(distinct_group_count(&exp), 2);
    }

    #[test]
    fn json_contains_face_count() {
        let exp = new_face_smooth_export(6, 1);
        let j = face_smooth_to_json(&exp);
        assert!(j.contains("face_count"));
    }

    #[test]
    fn oob_get_none() {
        let exp = new_face_smooth_export(2, 1);
        assert!(get_smooth_group(&exp, 99).is_none());
    }

    #[test]
    fn contains_range() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
