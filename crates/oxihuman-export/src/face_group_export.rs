// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export face groups (material assignments, selection sets) for mesh assets.

/// A face group: a named set of face indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceGroup {
    pub name: String,
    pub face_indices: Vec<u32>,
    pub material_index: Option<u32>,
}

/// Face group collection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceGroupExport {
    pub groups: Vec<FaceGroup>,
    pub total_faces: u32,
}

#[allow(dead_code)]
pub fn new_face_group_export(total_faces: u32) -> FaceGroupExport {
    FaceGroupExport { groups: Vec::new(), total_faces }
}

#[allow(dead_code)]
pub fn fg_add_group(export: &mut FaceGroupExport, name: &str, faces: &[u32]) -> usize {
    let idx = export.groups.len();
    export.groups.push(FaceGroup {
        name: name.to_string(), face_indices: faces.to_vec(), material_index: None,
    });
    idx
}

#[allow(dead_code)]
pub fn fg_set_material(export: &mut FaceGroupExport, group_idx: usize, mat: u32) {
    if group_idx < export.groups.len() {
        export.groups[group_idx].material_index = Some(mat);
    }
}

#[allow(dead_code)]
pub fn fg_group_count(export: &FaceGroupExport) -> usize { export.groups.len() }

#[allow(dead_code)]
pub fn fg_faces_in_group(export: &FaceGroupExport, idx: usize) -> usize {
    if idx < export.groups.len() { export.groups[idx].face_indices.len() } else { 0 }
}

#[allow(dead_code)]
pub fn fg_unassigned_faces(export: &FaceGroupExport) -> Vec<u32> {
    let mut assigned = std::collections::HashSet::new();
    for g in &export.groups {
        for &fi in &g.face_indices { assigned.insert(fi); }
    }
    (0..export.total_faces).filter(|f| !assigned.contains(f)).collect()
}

#[allow(dead_code)]
pub fn fg_to_json(export: &FaceGroupExport) -> String {
    let groups: Vec<String> = export.groups.iter().map(|g| {
        let mat = g.material_index.map_or("null".to_string(), |m| m.to_string());
        format!(r#"{{"name":"{}","faces":{},"material":{}}}"#, g.name, g.face_indices.len(), mat)
    }).collect();
    format!(r#"{{"total_faces":{},"groups":[{}]}}"#, export.total_faces, groups.join(","))
}

#[allow(dead_code)]
pub fn fg_validate(export: &FaceGroupExport) -> bool {
    export.groups.iter().all(|g| g.face_indices.iter().all(|&f| f < export.total_faces))
}

#[allow(dead_code)]
pub fn fg_find_group(export: &FaceGroupExport, name: &str) -> Option<usize> {
    export.groups.iter().position(|g| g.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_export() {
        let e = new_face_group_export(100);
        assert_eq!(fg_group_count(&e), 0);
    }

    #[test]
    fn test_add_group() {
        let mut e = new_face_group_export(10);
        fg_add_group(&mut e, "body", &[0, 1, 2]);
        assert_eq!(fg_group_count(&e), 1);
        assert_eq!(fg_faces_in_group(&e, 0), 3);
    }

    #[test]
    fn test_set_material() {
        let mut e = new_face_group_export(10);
        fg_add_group(&mut e, "head", &[0]);
        fg_set_material(&mut e, 0, 5);
        assert_eq!(e.groups[0].material_index, Some(5));
    }

    #[test]
    fn test_unassigned() {
        let mut e = new_face_group_export(5);
        fg_add_group(&mut e, "g", &[0, 2, 4]);
        let unassigned = fg_unassigned_faces(&e);
        assert_eq!(unassigned, vec![1, 3]);
    }

    #[test]
    fn test_to_json() {
        let mut e = new_face_group_export(3);
        fg_add_group(&mut e, "test", &[0, 1]);
        let json = fg_to_json(&e);
        assert!(json.contains("test"));
    }

    #[test]
    fn test_validate_ok() {
        let mut e = new_face_group_export(5);
        fg_add_group(&mut e, "g", &[0, 1, 4]);
        assert!(fg_validate(&e));
    }

    #[test]
    fn test_validate_fail() {
        let mut e = new_face_group_export(3);
        fg_add_group(&mut e, "g", &[0, 1, 10]);
        assert!(!fg_validate(&e));
    }

    #[test]
    fn test_find_group() {
        let mut e = new_face_group_export(5);
        fg_add_group(&mut e, "alpha", &[0]);
        fg_add_group(&mut e, "beta", &[1]);
        assert_eq!(fg_find_group(&e, "beta"), Some(1));
    }

    #[test]
    fn test_find_missing() {
        let e = new_face_group_export(5);
        assert_eq!(fg_find_group(&e, "nope"), None);
    }

    #[test]
    fn test_faces_out_of_range() {
        let e = new_face_group_export(5);
        assert_eq!(fg_faces_in_group(&e, 99), 0);
    }

}
