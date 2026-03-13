// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Face island export (connected components of mesh faces).

/// A face island (connected component).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceIsland {
    pub face_indices: Vec<u32>,
}

/// Face island export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceIslandExport {
    pub islands: Vec<FaceIsland>,
}

/// Create new export.
#[allow(dead_code)]
pub fn new_face_island_export() -> FaceIslandExport {
    FaceIslandExport { islands: vec![] }
}

/// Add an island.
#[allow(dead_code)]
pub fn add_island(e: &mut FaceIslandExport, faces: &[u32]) {
    e.islands.push(FaceIsland {
        face_indices: faces.to_vec(),
    });
}

/// Island count.
#[allow(dead_code)]
pub fn fi_island_count(e: &FaceIslandExport) -> usize {
    e.islands.len()
}

/// Total face count.
#[allow(dead_code)]
pub fn fi_total_faces(e: &FaceIslandExport) -> usize {
    e.islands.iter().map(|i| i.face_indices.len()).sum()
}

/// Largest island size.
#[allow(dead_code)]
pub fn fi_largest(e: &FaceIslandExport) -> usize {
    e.islands
        .iter()
        .map(|i| i.face_indices.len())
        .max()
        .unwrap_or(0)
}

/// Smallest island size.
#[allow(dead_code)]
pub fn fi_smallest(e: &FaceIslandExport) -> usize {
    e.islands
        .iter()
        .map(|i| i.face_indices.len())
        .min()
        .unwrap_or(0)
}

/// Get island by index.
#[allow(dead_code)]
pub fn get_island(e: &FaceIslandExport, idx: usize) -> Option<&FaceIsland> {
    e.islands.get(idx)
}

/// Validate (no empty islands).
#[allow(dead_code)]
pub fn fi_validate(e: &FaceIslandExport) -> bool {
    e.islands.iter().all(|i| !i.face_indices.is_empty())
}

/// Export to JSON.
#[allow(dead_code)]
pub fn face_island_to_json(e: &FaceIslandExport) -> String {
    format!(
        "{{\"islands\":{},\"total_faces\":{}}}",
        fi_island_count(e),
        fi_total_faces(e)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let e = new_face_island_export();
        assert_eq!(fi_island_count(&e), 0);
    }
    #[test]
    fn test_add() {
        let mut e = new_face_island_export();
        add_island(&mut e, &[0, 1, 2]);
        assert_eq!(fi_island_count(&e), 1);
    }
    #[test]
    fn test_total() {
        let mut e = new_face_island_export();
        add_island(&mut e, &[0, 1]);
        add_island(&mut e, &[2]);
        assert_eq!(fi_total_faces(&e), 3);
    }
    #[test]
    fn test_largest() {
        let mut e = new_face_island_export();
        add_island(&mut e, &[0, 1]);
        add_island(&mut e, &[2, 3, 4]);
        assert_eq!(fi_largest(&e), 3);
    }
    #[test]
    fn test_smallest() {
        let mut e = new_face_island_export();
        add_island(&mut e, &[0, 1]);
        add_island(&mut e, &[2]);
        assert_eq!(fi_smallest(&e), 1);
    }
    #[test]
    fn test_get() {
        let mut e = new_face_island_export();
        add_island(&mut e, &[0]);
        assert!(get_island(&e, 0).is_some());
    }
    #[test]
    fn test_get_oob() {
        let e = new_face_island_export();
        assert!(get_island(&e, 0).is_none());
    }
    #[test]
    fn test_validate() {
        let mut e = new_face_island_export();
        add_island(&mut e, &[0]);
        assert!(fi_validate(&e));
    }
    #[test]
    fn test_to_json() {
        let e = new_face_island_export();
        assert!(face_island_to_json(&e).contains("\"islands\":0"));
    }
    #[test]
    fn test_empty_largest() {
        let e = new_face_island_export();
        assert_eq!(fi_largest(&e), 0);
    }
}
