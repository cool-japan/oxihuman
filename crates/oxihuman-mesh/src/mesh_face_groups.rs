// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceGroup {
    pub id: u32,
    pub name: String,
    pub face_indices: Vec<u32>,
    pub material_index: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceGroupSet {
    pub groups: Vec<FaceGroup>,
    next_id: u32,
}

#[allow(dead_code)]
pub fn new_face_group_set() -> FaceGroupSet {
    FaceGroupSet { groups: Vec::new(), next_id: 0 }
}

#[allow(dead_code)]
pub fn fg_add_group(set: &mut FaceGroupSet, name: &str, mat_idx: u32) -> u32 {
    let id = set.next_id;
    set.next_id += 1;
    set.groups.push(FaceGroup {
        id,
        name: name.to_string(),
        face_indices: Vec::new(),
        material_index: mat_idx,
    });
    id
}

#[allow(dead_code)]
pub fn fg_assign_face(set: &mut FaceGroupSet, group_id: u32, face: u32) {
    if let Some(g) = set.groups.iter_mut().find(|g| g.id == group_id) {
        g.face_indices.push(face);
    }
}

#[allow(dead_code)]
pub fn fg_get_group(set: &FaceGroupSet, id: u32) -> Option<&FaceGroup> {
    set.groups.iter().find(|g| g.id == id)
}

#[allow(dead_code)]
pub fn fg_group_count(set: &FaceGroupSet) -> usize {
    set.groups.len()
}

#[allow(dead_code)]
pub fn fg_face_count(set: &FaceGroupSet, group_id: u32) -> usize {
    set.groups
        .iter()
        .find(|g| g.id == group_id)
        .map(|g| g.face_indices.len())
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn fg_remove_group(set: &mut FaceGroupSet, group_id: u32) {
    set.groups.retain(|g| g.id != group_id);
}

#[allow(dead_code)]
pub fn fg_to_json(set: &FaceGroupSet) -> String {
    format!(r#"{{"group_count":{}}}"#, set.groups.len())
}

#[allow(dead_code)]
pub fn fg_find_by_name<'a>(set: &'a FaceGroupSet, name: &str) -> Option<&'a FaceGroup> {
    set.groups.iter().find(|g| g.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_set_empty() {
        let s = new_face_group_set();
        assert_eq!(fg_group_count(&s), 0);
    }

    #[test]
    fn test_add_group() {
        let mut s = new_face_group_set();
        let id = fg_add_group(&mut s, "body", 0);
        assert_eq!(fg_group_count(&s), 1);
        assert_eq!(id, 0);
    }

    #[test]
    fn test_assign_face() {
        let mut s = new_face_group_set();
        let id = fg_add_group(&mut s, "body", 0);
        fg_assign_face(&mut s, id, 10);
        fg_assign_face(&mut s, id, 11);
        assert_eq!(fg_face_count(&s, id), 2);
    }

    #[test]
    fn test_get_group() {
        let mut s = new_face_group_set();
        let id = fg_add_group(&mut s, "face", 1);
        let g = fg_get_group(&s, id).unwrap();
        assert_eq!(g.name, "face");
        assert_eq!(g.material_index, 1);
    }

    #[test]
    fn test_get_group_missing() {
        let s = new_face_group_set();
        assert!(fg_get_group(&s, 99).is_none());
    }

    #[test]
    fn test_remove_group() {
        let mut s = new_face_group_set();
        let id = fg_add_group(&mut s, "temp", 0);
        fg_remove_group(&mut s, id);
        assert_eq!(fg_group_count(&s), 0);
    }

    #[test]
    fn test_find_by_name() {
        let mut s = new_face_group_set();
        fg_add_group(&mut s, "skin", 2);
        let g = fg_find_by_name(&s, "skin").unwrap();
        assert_eq!(g.material_index, 2);
    }

    #[test]
    fn test_to_json() {
        let mut s = new_face_group_set();
        fg_add_group(&mut s, "a", 0);
        let j = fg_to_json(&s);
        assert!(j.contains("group_count"));
        assert!(j.contains('1'));
    }
}
