#![allow(dead_code)]

//! Per-face material assignment for multi-material meshes.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceMaterialAssign {
    pub assignments: HashMap<usize, usize>,
    pub face_count: usize,
}

#[allow(dead_code)]
pub fn assign_material(fma: &mut FaceMaterialAssign, face_idx: usize, material_id: usize) {
    if face_idx < fma.face_count {
        fma.assignments.insert(face_idx, material_id);
    }
}

#[allow(dead_code)]
pub fn material_at_face(fma: &FaceMaterialAssign, face_idx: usize) -> Option<usize> {
    fma.assignments.get(&face_idx).copied()
}

#[allow(dead_code)]
pub fn material_face_count(fma: &FaceMaterialAssign, material_id: usize) -> usize {
    fma.assignments.values().filter(|&&m| m == material_id).count()
}

#[allow(dead_code)]
pub fn material_count_fma(fma: &FaceMaterialAssign) -> usize {
    let mut ids: Vec<usize> = fma.assignments.values().copied().collect();
    ids.sort_unstable();
    ids.dedup();
    ids.len()
}

#[allow(dead_code)]
pub fn reassign_material(fma: &mut FaceMaterialAssign, from: usize, to: usize) {
    for v in fma.assignments.values_mut() {
        if *v == from { *v = to; }
    }
}

#[allow(dead_code)]
pub fn clear_assignments(fma: &mut FaceMaterialAssign) {
    fma.assignments.clear();
}

#[allow(dead_code)]
pub fn assignments_to_json(fma: &FaceMaterialAssign) -> String {
    let entries: Vec<String> = fma.assignments.iter()
        .map(|(f, m)| format!("{{\"face\":{},\"material\":{}}}", f, m))
        .collect();
    format!("{{\"assigned\":{},\"total_faces\":{},\"assignments\":[{}]}}", fma.assignments.len(), fma.face_count, entries.join(","))
}

#[allow(dead_code)]
pub fn validate_assignments(fma: &FaceMaterialAssign) -> bool {
    fma.assignments.keys().all(|&f| f < fma.face_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fma() -> FaceMaterialAssign { FaceMaterialAssign { assignments: HashMap::new(), face_count: 10 } }

    #[test]
    fn test_assign() { let mut f = fma(); assign_material(&mut f, 0, 1); assert_eq!(material_at_face(&f, 0), Some(1)); }
    #[test]
    fn test_unassigned() { let f = fma(); assert_eq!(material_at_face(&f, 0), None); }
    #[test]
    fn test_face_count() { let mut f = fma(); assign_material(&mut f, 0, 1); assign_material(&mut f, 1, 1); assert_eq!(material_face_count(&f, 1), 2); }
    #[test]
    fn test_material_count() { let mut f = fma(); assign_material(&mut f, 0, 0); assign_material(&mut f, 1, 1); assert_eq!(material_count_fma(&f), 2); }
    #[test]
    fn test_reassign() { let mut f = fma(); assign_material(&mut f, 0, 0); reassign_material(&mut f, 0, 1); assert_eq!(material_at_face(&f, 0), Some(1)); }
    #[test]
    fn test_clear() { let mut f = fma(); assign_material(&mut f, 0, 0); clear_assignments(&mut f); assert_eq!(material_at_face(&f, 0), None); }
    #[test]
    fn test_to_json() { let f = fma(); assert!(assignments_to_json(&f).contains("\"assigned\":0")); }
    #[test]
    fn test_validate() { let f = fma(); assert!(validate_assignments(&f)); }
    #[test]
    fn test_out_of_bounds() { let mut f = fma(); assign_material(&mut f, 20, 1); assert_eq!(f.assignments.len(), 0); }
    #[test]
    fn test_overwrite() { let mut f = fma(); assign_material(&mut f, 0, 0); assign_material(&mut f, 0, 5); assert_eq!(material_at_face(&f, 0), Some(5)); }
}
