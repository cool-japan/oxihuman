#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A named group of face indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceGroup {
    pub name: String,
    pub face_indices: Vec<u32>,
}

/// A collection of face groups.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceGroupSet {
    pub groups: Vec<FaceGroup>,
}

/// Create an empty `FaceGroupSet`.
#[allow(dead_code)]
pub fn new_face_group_set() -> FaceGroupSet {
    FaceGroupSet { groups: Vec::new() }
}

/// Add a new named face group and return its index.
#[allow(dead_code)]
pub fn add_face_group(set: &mut FaceGroupSet, name: &str, faces: &[u32]) -> usize {
    let idx = set.groups.len();
    set.groups.push(FaceGroup {
        name: name.to_string(),
        face_indices: faces.to_vec(),
    });
    idx
}

/// Return the number of faces in the group at `index`.
#[allow(dead_code)]
pub fn group_face_count(set: &FaceGroupSet, index: usize) -> usize {
    set.groups.get(index).map_or(0, |g| g.face_indices.len())
}

/// Return the total number of groups.
#[allow(dead_code)]
pub fn group_count(set: &FaceGroupSet) -> usize {
    set.groups.len()
}

/// Return a slice of face indices for a group.
#[allow(dead_code)]
pub fn faces_in_group(set: &FaceGroupSet, index: usize) -> &[u32] {
    set.groups.get(index).map_or(&[], |g| g.face_indices.as_slice())
}

/// Merge two groups into the first, appending faces from the second.
#[allow(dead_code)]
pub fn merge_groups(set: &mut FaceGroupSet, dst: usize, src: usize) -> bool {
    if dst == src || dst >= set.groups.len() || src >= set.groups.len() {
        return false;
    }
    let src_faces = set.groups[src].face_indices.clone();
    set.groups[dst].face_indices.extend(src_faces);
    set.groups.remove(src);
    true
}

/// Return the name of a group.
#[allow(dead_code)]
pub fn group_name(set: &FaceGroupSet, index: usize) -> Option<&str> {
    set.groups.get(index).map(|g| g.name.as_str())
}

/// Convert a group to a selection vec of face indices (clone).
#[allow(dead_code)]
pub fn group_to_selection(set: &FaceGroupSet, index: usize) -> Vec<u32> {
    set.groups.get(index).map_or_else(Vec::new, |g| g.face_indices.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_face_group_set() {
        let set = new_face_group_set();
        assert_eq!(group_count(&set), 0);
    }

    #[test]
    fn test_add_face_group() {
        let mut set = new_face_group_set();
        let idx = add_face_group(&mut set, "body", &[0, 1, 2]);
        assert_eq!(idx, 0);
        assert_eq!(group_count(&set), 1);
    }

    #[test]
    fn test_group_face_count() {
        let mut set = new_face_group_set();
        add_face_group(&mut set, "head", &[3, 4]);
        assert_eq!(group_face_count(&set, 0), 2);
        assert_eq!(group_face_count(&set, 99), 0);
    }

    #[test]
    fn test_faces_in_group() {
        let mut set = new_face_group_set();
        add_face_group(&mut set, "arm", &[10, 11, 12]);
        assert_eq!(faces_in_group(&set, 0), &[10, 11, 12]);
        assert!(faces_in_group(&set, 5).is_empty());
    }

    #[test]
    fn test_merge_groups() {
        let mut set = new_face_group_set();
        add_face_group(&mut set, "a", &[0, 1]);
        add_face_group(&mut set, "b", &[2, 3]);
        assert!(merge_groups(&mut set, 0, 1));
        assert_eq!(group_count(&set), 1);
        assert_eq!(group_face_count(&set, 0), 4);
    }

    #[test]
    fn test_merge_same_index() {
        let mut set = new_face_group_set();
        add_face_group(&mut set, "x", &[0]);
        assert!(!merge_groups(&mut set, 0, 0));
    }

    #[test]
    fn test_group_name() {
        let mut set = new_face_group_set();
        add_face_group(&mut set, "leg", &[5]);
        assert_eq!(group_name(&set, 0), Some("leg"));
        assert_eq!(group_name(&set, 10), None);
    }

    #[test]
    fn test_group_to_selection() {
        let mut set = new_face_group_set();
        add_face_group(&mut set, "torso", &[7, 8, 9]);
        let sel = group_to_selection(&set, 0);
        assert_eq!(sel, vec![7, 8, 9]);
        assert!(group_to_selection(&set, 99).is_empty());
    }

    #[test]
    fn test_multiple_groups() {
        let mut set = new_face_group_set();
        add_face_group(&mut set, "a", &[0]);
        add_face_group(&mut set, "b", &[1]);
        add_face_group(&mut set, "c", &[2]);
        assert_eq!(group_count(&set), 3);
    }

    #[test]
    fn test_merge_out_of_bounds() {
        let mut set = new_face_group_set();
        add_face_group(&mut set, "only", &[0]);
        assert!(!merge_groups(&mut set, 0, 5));
        assert!(!merge_groups(&mut set, 5, 0));
    }
}
