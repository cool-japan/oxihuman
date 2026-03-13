// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Face map — named groups of face indices.

use std::collections::HashMap;

/// A face map: named groups mapping face indices.
#[derive(Debug, Clone)]
pub struct FaceMap {
    pub groups: HashMap<String, Vec<usize>>,
}

impl FaceMap {
    /// Create an empty face map.
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    /// Add a named group with the given face indices.
    pub fn add_group(&mut self, name: &str, faces: Vec<usize>) {
        self.groups.insert(name.to_string(), faces);
    }

    /// Remove a group by name, returning the face list if it existed.
    pub fn remove_group(&mut self, name: &str) -> Option<Vec<usize>> {
        self.groups.remove(name)
    }

    /// Return the number of groups.
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Return sorted group names.
    pub fn group_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.groups.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get faces for a group.
    pub fn get_group(&self, name: &str) -> Option<&Vec<usize>> {
        self.groups.get(name)
    }
}

impl Default for FaceMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Count total faces across all groups.
pub fn total_face_count(fm: &FaceMap) -> usize {
    fm.groups.values().map(|v| v.len()).sum()
}

/// Check if a face is in any group.
pub fn face_in_any_group(fm: &FaceMap, face_idx: usize) -> bool {
    fm.groups.values().any(|v| v.contains(&face_idx))
}

/// Merge two face maps (second overwrites first on name collision).
pub fn merge_face_maps(a: &FaceMap, b: &FaceMap) -> FaceMap {
    let mut result = a.clone();
    for (name, faces) in &b.groups {
        result.groups.insert(name.clone(), faces.clone());
    }
    result
}

/// Rename a group.
pub fn rename_group(fm: &mut FaceMap, old_name: &str, new_name: &str) -> bool {
    if let Some(faces) = fm.groups.remove(old_name) {
        fm.groups.insert(new_name.to_string(), faces);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_map() -> FaceMap {
        let mut fm = FaceMap::new();
        fm.add_group("left_arm", vec![0, 1, 2]);
        fm.add_group("right_arm", vec![3, 4, 5]);
        fm
    }

    #[test]
    fn test_group_count() {
        /* map starts with correct group count */
        let fm = basic_map();
        assert_eq!(fm.group_count(), 2);
    }

    #[test]
    fn test_get_group() {
        /* get_group returns correct faces */
        let fm = basic_map();
        let g = fm.get_group("left_arm").expect("should succeed");
        assert_eq!(g, &vec![0, 1, 2]);
    }

    #[test]
    fn test_remove_group() {
        /* remove group reduces count */
        let mut fm = basic_map();
        fm.remove_group("left_arm");
        assert_eq!(fm.group_count(), 1);
    }

    #[test]
    fn test_total_face_count() {
        /* total face count sums correctly */
        let fm = basic_map();
        assert_eq!(total_face_count(&fm), 6);
    }

    #[test]
    fn test_face_in_any_group() {
        /* face lookup across groups works */
        let fm = basic_map();
        assert!(face_in_any_group(&fm, 4));
        assert!(!face_in_any_group(&fm, 99));
    }

    #[test]
    fn test_merge_face_maps() {
        /* merge combines both maps */
        let fm1 = basic_map();
        let mut fm2 = FaceMap::new();
        fm2.add_group("head", vec![10, 11]);
        let merged = merge_face_maps(&fm1, &fm2);
        assert_eq!(merged.group_count(), 3);
    }

    #[test]
    fn test_rename_group() {
        /* rename moves group name */
        let mut fm = basic_map();
        assert!(rename_group(&mut fm, "left_arm", "arm_left"));
        assert!(fm.get_group("arm_left").is_some());
        assert!(fm.get_group("left_arm").is_none());
    }

    #[test]
    fn test_group_names_sorted() {
        /* names are returned sorted */
        let fm = basic_map();
        let names = fm.group_names();
        assert_eq!(names[0], "left_arm");
        assert_eq!(names[1], "right_arm");
    }

    #[test]
    fn test_rename_nonexistent_returns_false() {
        /* renaming a nonexistent group returns false */
        let mut fm = basic_map();
        assert!(!rename_group(&mut fm, "no_such_group", "new_name"));
    }
}
