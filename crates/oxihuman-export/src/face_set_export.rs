// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export face sets (named groups of faces for material assignment).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceSet {
    pub name: String,
    pub face_indices: Vec<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceSetExport {
    pub sets: Vec<FaceSet>,
}

#[allow(dead_code)]
pub fn new_face_set_export() -> FaceSetExport {
    FaceSetExport { sets: Vec::new() }
}

#[allow(dead_code)]
pub fn fse_add(fse: &mut FaceSetExport, name: &str, indices: Vec<u32>) {
    fse.sets.push(FaceSet { name: name.to_string(), face_indices: indices });
}

#[allow(dead_code)]
pub fn fse_count(fse: &FaceSetExport) -> usize { fse.sets.len() }

#[allow(dead_code)]
pub fn fse_total_faces(fse: &FaceSetExport) -> usize {
    fse.sets.iter().map(|s| s.face_indices.len()).sum()
}

#[allow(dead_code)]
pub fn fse_find<'a>(fse: &'a FaceSetExport, name: &str) -> Option<&'a FaceSet> {
    fse.sets.iter().find(|s| s.name == name)
}

#[allow(dead_code)]
pub fn fse_largest_set(fse: &FaceSetExport) -> Option<&FaceSet> {
    fse.sets.iter().max_by_key(|s| s.face_indices.len())
}

#[allow(dead_code)]
pub fn fse_validate(fse: &FaceSetExport) -> bool {
    fse.sets.iter().all(|s| !s.name.is_empty() && !s.face_indices.is_empty())
}

#[allow(dead_code)]
pub fn fse_names(fse: &FaceSetExport) -> Vec<&str> {
    fse.sets.iter().map(|s| s.name.as_str()).collect()
}

#[allow(dead_code)]
pub fn fse_to_json(fse: &FaceSetExport) -> String {
    let items: Vec<String> = fse.sets.iter().map(|s| format!("{{\"name\":\"{}\",\"faces\":{}}}", s.name, s.face_indices.len())).collect();
    format!("{{\"face_sets\":[{}]}}", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> FaceSetExport {
        let mut f = new_face_set_export();
        fse_add(&mut f, "body", vec![0, 1, 2, 3]);
        fse_add(&mut f, "head", vec![4, 5]);
        f
    }

    #[test] fn test_new() { assert_eq!(fse_count(&new_face_set_export()), 0); }
    #[test] fn test_add() { assert_eq!(fse_count(&sample()), 2); }
    #[test] fn test_total() { assert_eq!(fse_total_faces(&sample()), 6); }
    #[test] fn test_find() { assert!(fse_find(&sample(), "body").is_some()); }
    #[test] fn test_find_missing() { assert!(fse_find(&sample(), "nope").is_none()); }
    #[test] fn test_largest() { let s = sample(); let l = fse_largest_set(&s).unwrap(); assert_eq!(l.name, "body"); }
    #[test] fn test_validate() { assert!(fse_validate(&sample())); }
    #[test] fn test_names() { let s = sample(); let n = fse_names(&s); assert_eq!(n.len(), 2); }
    #[test] fn test_to_json() { assert!(fse_to_json(&sample()).contains("body")); }
    #[test] fn test_empty_invalid() { let mut f = new_face_set_export(); fse_add(&mut f, "", vec![0]); assert!(!fse_validate(&f)); }
}
