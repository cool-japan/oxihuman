#![allow(dead_code)]

use std::collections::HashMap;

/// A named group of face indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceGroup2 {
    pub name: String,
    pub face_indices: Vec<usize>,
}

/// A collection of face groups.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceGroupSet2 {
    pub groups: Vec<FaceGroup2>,
    name_map: HashMap<String, usize>,
}

/// Create a new empty face group set.
#[allow(dead_code)]
pub fn new_face_group_set2() -> FaceGroupSet2 {
    FaceGroupSet2 {
        groups: Vec::new(),
        name_map: HashMap::new(),
    }
}

/// Add a new named face group and return its index.
#[allow(dead_code)]
pub fn add_face_group2(set: &mut FaceGroupSet2, name: &str, faces: &[usize]) -> usize {
    let idx = set.groups.len();
    set.name_map.insert(name.to_string(), idx);
    set.groups.push(FaceGroup2 {
        name: name.to_string(),
        face_indices: faces.to_vec(),
    });
    idx
}

/// Get the number of faces in a group.
#[allow(dead_code)]
pub fn group_face_count2(set: &FaceGroupSet2, group_idx: usize) -> usize {
    set.groups.get(group_idx).map_or(0, |g| g.face_indices.len())
}

/// Get the number of groups.
#[allow(dead_code)]
pub fn group_count2(set: &FaceGroupSet2) -> usize {
    set.groups.len()
}

/// Get all face indices in a group.
#[allow(dead_code)]
pub fn faces_in_group2(set: &FaceGroupSet2, group_idx: usize) -> Vec<usize> {
    set.groups
        .get(group_idx)
        .map_or_else(Vec::new, |g| g.face_indices.clone())
}

/// Merge two groups into the first, removing the second.
#[allow(dead_code)]
pub fn merge_groups2(set: &mut FaceGroupSet2, a: usize, b: usize) -> bool {
    if a >= set.groups.len() || b >= set.groups.len() || a == b {
        return false;
    }
    let b_faces = set.groups[b].face_indices.clone();
    set.groups[a].face_indices.extend(b_faces);
    let b_name = set.groups[b].name.clone();
    set.name_map.remove(&b_name);
    set.groups.remove(b);
    // Rebuild name map
    set.name_map.clear();
    for (i, g) in set.groups.iter().enumerate() {
        set.name_map.insert(g.name.clone(), i);
    }
    true
}

/// Get the name of a group.
#[allow(dead_code)]
pub fn group_name2(set: &FaceGroupSet2, group_idx: usize) -> String {
    set.groups
        .get(group_idx)
        .map_or_else(String::new, |g| g.name.clone())
}

/// Convert a group to a selection (list of face indices).
#[allow(dead_code)]
pub fn group_to_selection2(set: &FaceGroupSet2, group_idx: usize) -> Vec<usize> {
    faces_in_group2(set, group_idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_set_empty() {
        let s = new_face_group_set2();
        assert_eq!(group_count2(&s), 0);
    }

    #[test]
    fn test_add_group() {
        let mut s = new_face_group_set2();
        add_face_group2(&mut s, "body", &[0, 1, 2]);
        assert_eq!(group_count2(&s), 1);
        assert_eq!(group_face_count2(&s, 0), 3);
    }

    #[test]
    fn test_faces_in_group() {
        let mut s = new_face_group_set2();
        add_face_group2(&mut s, "arm", &[3, 4]);
        let f = faces_in_group2(&s, 0);
        assert_eq!(f, vec![3, 4]);
    }

    #[test]
    fn test_group_name() {
        let mut s = new_face_group_set2();
        add_face_group2(&mut s, "head", &[0]);
        assert_eq!(group_name2(&s, 0), "head");
    }

    #[test]
    fn test_merge_groups() {
        let mut s = new_face_group_set2();
        add_face_group2(&mut s, "a", &[0, 1]);
        add_face_group2(&mut s, "b", &[2, 3]);
        assert!(merge_groups2(&mut s, 0, 1));
        assert_eq!(group_count2(&s), 1);
        assert_eq!(group_face_count2(&s, 0), 4);
    }

    #[test]
    fn test_merge_invalid() {
        let mut s = new_face_group_set2();
        add_face_group2(&mut s, "a", &[0]);
        assert!(!merge_groups2(&mut s, 0, 5));
        assert!(!merge_groups2(&mut s, 0, 0));
    }

    #[test]
    fn test_group_to_selection() {
        let mut s = new_face_group_set2();
        add_face_group2(&mut s, "sel", &[10, 20]);
        let sel = group_to_selection2(&s, 0);
        assert_eq!(sel, vec![10, 20]);
    }

    #[test]
    fn test_empty_group() {
        let mut s = new_face_group_set2();
        add_face_group2(&mut s, "empty", &[]);
        assert_eq!(group_face_count2(&s, 0), 0);
    }

    #[test]
    fn test_multiple_groups() {
        let mut s = new_face_group_set2();
        add_face_group2(&mut s, "a", &[0]);
        add_face_group2(&mut s, "b", &[1]);
        add_face_group2(&mut s, "c", &[2]);
        assert_eq!(group_count2(&s), 3);
    }

    #[test]
    fn test_nonexistent_group() {
        let s = new_face_group_set2();
        assert_eq!(group_face_count2(&s, 0), 0);
        assert_eq!(group_name2(&s, 0), "");
    }
}
