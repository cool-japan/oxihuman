#![allow(dead_code)]
//! Material slot assignment for mesh faces.

/// A material slot.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialSlot {
    pub name: String,
    pub face_indices: Vec<u32>,
}

/// Assignment of faces to material slots.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialAssignment {
    pub slots: Vec<MaterialSlot>,
}

/// Create a new material slot.
#[allow(dead_code)]
pub fn new_material_slot(name: &str) -> MaterialAssignment {
    MaterialAssignment {
        slots: vec![MaterialSlot {
            name: name.to_string(),
            face_indices: Vec::new(),
        }],
    }
}

/// Assign faces to a slot by index.
#[allow(dead_code)]
pub fn assign_faces_to_slot(assignment: &mut MaterialAssignment, slot_index: usize, faces: &[u32]) {
    if slot_index < assignment.slots.len() {
        assignment.slots[slot_index].face_indices.extend_from_slice(faces);
    }
}

/// Return the face count for a slot.
#[allow(dead_code)]
pub fn slot_face_count(assignment: &MaterialAssignment, slot_index: usize) -> usize {
    if slot_index < assignment.slots.len() {
        assignment.slots[slot_index].face_indices.len()
    } else {
        0
    }
}

/// Return the number of slots.
#[allow(dead_code)]
pub fn slot_count(assignment: &MaterialAssignment) -> usize {
    assignment.slots.len()
}

/// Return the name of a slot.
#[allow(dead_code)]
pub fn slot_name(assignment: &MaterialAssignment, slot_index: usize) -> &str {
    if slot_index < assignment.slots.len() {
        &assignment.slots[slot_index].name
    } else {
        ""
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn slot_to_json(assignment: &MaterialAssignment) -> String {
    let slots: Vec<String> = assignment
        .slots
        .iter()
        .map(|s| format!("{{\"name\":\"{}\",\"faces\":{}}}", s.name, s.face_indices.len()))
        .collect();
    format!("{{\"slots\":[{}]}}", slots.join(","))
}

/// Return all assignments as (slot_index, face_indices).
#[allow(dead_code)]
pub fn material_assignments(assignment: &MaterialAssignment) -> Vec<(usize, Vec<u32>)> {
    assignment
        .slots
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.face_indices.clone()))
        .collect()
}

/// Clear all face assignments from a slot.
#[allow(dead_code)]
pub fn clear_slot(assignment: &mut MaterialAssignment, slot_index: usize) {
    if slot_index < assignment.slots.len() {
        assignment.slots[slot_index].face_indices.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_material_slot() {
        let a = new_material_slot("default");
        assert_eq!(slot_count(&a), 1);
    }

    #[test]
    fn test_slot_name() {
        let a = new_material_slot("metal");
        assert_eq!(slot_name(&a, 0), "metal");
    }

    #[test]
    fn test_assign_faces() {
        let mut a = new_material_slot("mat");
        assign_faces_to_slot(&mut a, 0, &[0, 1, 2]);
        assert_eq!(slot_face_count(&a, 0), 3);
    }

    #[test]
    fn test_slot_face_count_oob() {
        let a = new_material_slot("mat");
        assert_eq!(slot_face_count(&a, 10), 0);
    }

    #[test]
    fn test_slot_name_oob() {
        let a = new_material_slot("mat");
        assert_eq!(slot_name(&a, 10), "");
    }

    #[test]
    fn test_slot_to_json() {
        let a = new_material_slot("mat");
        let j = slot_to_json(&a);
        assert!(j.contains("\"slots\""));
    }

    #[test]
    fn test_material_assignments() {
        let mut a = new_material_slot("mat");
        assign_faces_to_slot(&mut a, 0, &[5, 6]);
        let assigns = material_assignments(&a);
        assert_eq!(assigns.len(), 1);
        assert_eq!(assigns[0].1, vec![5, 6]);
    }

    #[test]
    fn test_clear_slot() {
        let mut a = new_material_slot("mat");
        assign_faces_to_slot(&mut a, 0, &[0, 1]);
        clear_slot(&mut a, 0);
        assert_eq!(slot_face_count(&a, 0), 0);
    }

    #[test]
    fn test_clear_slot_oob() {
        let mut a = new_material_slot("mat");
        clear_slot(&mut a, 10); // should not panic
        assert_eq!(slot_count(&a), 1);
    }

    #[test]
    fn test_multiple_assigns() {
        let mut a = new_material_slot("mat");
        assign_faces_to_slot(&mut a, 0, &[0]);
        assign_faces_to_slot(&mut a, 0, &[1, 2]);
        assert_eq!(slot_face_count(&a, 0), 3);
    }
}
