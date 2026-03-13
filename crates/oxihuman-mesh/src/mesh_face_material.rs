// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-face material assignment and multi-material mesh utilities.

/// A material slot descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialSlot {
    pub id: u32,
    pub name: String,
    pub base_color: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
}

/// Per-face material binding.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceMaterialSet {
    pub assignments: Vec<u32>, // per-face material id
    pub slots: Vec<MaterialSlot>,
}

/// Create a new material slot.
#[allow(dead_code)]
pub fn new_material_slot(id: u32, name: &str) -> MaterialSlot {
    MaterialSlot {
        id,
        name: name.to_string(),
        base_color: [1.0, 1.0, 1.0, 1.0],
        roughness: 0.5,
        metallic: 0.0,
    }
}

/// Create a face material set with all faces assigned to slot 0.
#[allow(dead_code)]
pub fn new_face_material_set(face_count: usize) -> FaceMaterialSet {
    FaceMaterialSet {
        assignments: vec![0; face_count],
        slots: Vec::new(),
    }
}

/// Add a material slot.
#[allow(dead_code)]
pub fn add_slot(set: &mut FaceMaterialSet, slot: MaterialSlot) {
    set.slots.push(slot);
}

/// Assign a material id to a face.
#[allow(dead_code)]
pub fn assign_face_material(set: &mut FaceMaterialSet, face: usize, material_id: u32) {
    if face < set.assignments.len() {
        set.assignments[face] = material_id;
    }
}

/// Get material id for a face.
#[allow(dead_code)]
pub fn get_face_material(set: &FaceMaterialSet, face: usize) -> Option<u32> {
    set.assignments.get(face).copied()
}

/// Return all faces assigned to a material id.
#[allow(dead_code)]
pub fn faces_for_material(set: &FaceMaterialSet, material_id: u32) -> Vec<usize> {
    set.assignments
        .iter()
        .enumerate()
        .filter(|(_, &m)| m == material_id)
        .map(|(i, _)| i)
        .collect()
}

/// Number of slots.
#[allow(dead_code)]
pub fn slot_count(set: &FaceMaterialSet) -> usize {
    set.slots.len()
}

/// Find slot by id.
#[allow(dead_code)]
pub fn find_slot_by_id(set: &FaceMaterialSet, id: u32) -> Option<&MaterialSlot> {
    set.slots.iter().find(|s| s.id == id)
}

/// Number of distinct materials in use.
#[allow(dead_code)]
pub fn distinct_material_count(set: &FaceMaterialSet) -> usize {
    let mut seen: Vec<u32> = set.assignments.clone();
    seen.sort_unstable();
    seen.dedup();
    seen.len()
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn face_material_to_json(set: &FaceMaterialSet) -> String {
    format!(
        "{{\"face_count\":{},\"slot_count\":{},\"distinct_materials\":{}}}",
        set.assignments.len(),
        slot_count(set),
        distinct_material_count(set)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_all_zero() {
        let set = new_face_material_set(4);
        assert!(set.assignments.iter().all(|&m| m == 0));
    }

    #[test]
    fn assign_and_get() {
        let mut set = new_face_material_set(3);
        assign_face_material(&mut set, 1, 5);
        assert_eq!(get_face_material(&set, 1), Some(5));
    }

    #[test]
    fn get_oob_none() {
        let set = new_face_material_set(2);
        assert!(get_face_material(&set, 999).is_none());
    }

    #[test]
    fn faces_for_material_query() {
        let mut set = new_face_material_set(4);
        assign_face_material(&mut set, 0, 3);
        assign_face_material(&mut set, 2, 3);
        let v = faces_for_material(&set, 3);
        assert_eq!(v, vec![0, 2]);
    }

    #[test]
    fn add_slot_increments() {
        let mut set = new_face_material_set(2);
        add_slot(&mut set, new_material_slot(0, "default"));
        assert_eq!(slot_count(&set), 1);
    }

    #[test]
    fn find_slot_by_id_some() {
        let mut set = new_face_material_set(1);
        add_slot(&mut set, new_material_slot(7, "skin"));
        assert!(find_slot_by_id(&set, 7).is_some_and(|s| s.name == "skin"));
    }

    #[test]
    fn distinct_material_count_initial() {
        let set = new_face_material_set(5);
        assert_eq!(distinct_material_count(&set), 1);
    }

    #[test]
    fn json_contains_face_count() {
        let set = new_face_material_set(6);
        let j = face_material_to_json(&set);
        assert!(j.contains("face_count"));
    }

    #[test]
    fn roughness_in_range() {
        let slot = new_material_slot(0, "mat");
        assert!((0.0..=1.0).contains(&slot.roughness));
    }

    #[test]
    fn base_color_alpha_one() {
        let slot = new_material_slot(0, "mat");
        assert!((slot.base_color[3] - 1.0).abs() < 1e-5);
    }
}
