#![allow(dead_code)]
//! Smooth group management for faces.

/// A smooth group containing face indices.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SmoothGroup {
    pub id: u32,
    pub face_indices: Vec<usize>,
}

/// Smooth group collection.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SmoothGroupSet {
    pub groups: Vec<SmoothGroup>,
    pub face_to_group: Vec<Option<u32>>,
}

/// Create a new smooth group.
#[allow(dead_code)]
pub fn new_smooth_group(id: u32) -> SmoothGroup {
    SmoothGroup {
        id,
        face_indices: Vec::new(),
    }
}

/// Add a face to a group.
#[allow(dead_code)]
pub fn add_face_to_group(group: &mut SmoothGroup, face_index: usize) {
    if !group.face_indices.contains(&face_index) {
        group.face_indices.push(face_index);
    }
}

/// Get face count in a group.
#[allow(dead_code)]
pub fn group_face_count_sg(group: &SmoothGroup) -> usize {
    group.face_indices.len()
}

/// Auto-assign smooth groups based on angle threshold.
#[allow(dead_code)]
pub fn auto_smooth_groups(
    face_normals: &[[f32; 3]],
    angle_threshold: f32,
) -> SmoothGroupSet {
    let mut groups: Vec<SmoothGroup> = Vec::new();
    let mut face_to_group = vec![None; face_normals.len()];
    for i in 0..face_normals.len() {
        if face_to_group[i].is_some() {
            continue;
        }
        let gid = groups.len() as u32;
        let mut group = new_smooth_group(gid);
        add_face_to_group(&mut group, i);
        face_to_group[i] = Some(gid);
        for j in (i + 1)..face_normals.len() {
            if face_to_group[j].is_some() {
                continue;
            }
            let n1 = face_normals[i];
            let n2 = face_normals[j];
            let dot = n1[0] * n2[0] + n1[1] * n2[1] + n1[2] * n2[2];
            let angle = dot.clamp(-1.0, 1.0).acos();
            if angle < angle_threshold {
                add_face_to_group(&mut group, j);
                face_to_group[j] = Some(gid);
            }
        }
        groups.push(group);
    }
    SmoothGroupSet { groups, face_to_group }
}

/// Count smooth groups.
#[allow(dead_code)]
pub fn smooth_group_count(sgs: &SmoothGroupSet) -> usize {
    sgs.groups.len()
}

/// Serialize smooth groups to JSON.
#[allow(dead_code)]
pub fn smooth_groups_to_json(sgs: &SmoothGroupSet) -> String {
    format!(
        "{{\"group_count\":{},\"face_count\":{}}}",
        sgs.groups.len(),
        sgs.face_to_group.len()
    )
}

/// Get the smooth group for a face.
#[allow(dead_code)]
pub fn face_smooth_group(sgs: &SmoothGroupSet, face_index: usize) -> Option<u32> {
    if face_index < sgs.face_to_group.len() {
        sgs.face_to_group[face_index]
    } else {
        None
    }
}

/// Clear all smooth groups.
#[allow(dead_code)]
pub fn clear_smooth_groups(sgs: &mut SmoothGroupSet) {
    sgs.groups.clear();
    for g in &mut sgs.face_to_group {
        *g = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_smooth_group() {
        let sg = new_smooth_group(0);
        assert_eq!(sg.id, 0);
        assert!(sg.face_indices.is_empty());
    }

    #[test]
    fn test_add_face_to_group() {
        let mut sg = new_smooth_group(0);
        add_face_to_group(&mut sg, 5);
        assert_eq!(sg.face_indices, vec![5]);
    }

    #[test]
    fn test_add_face_duplicate() {
        let mut sg = new_smooth_group(0);
        add_face_to_group(&mut sg, 3);
        add_face_to_group(&mut sg, 3);
        assert_eq!(sg.face_indices.len(), 1);
    }

    #[test]
    fn test_group_face_count() {
        let mut sg = new_smooth_group(0);
        add_face_to_group(&mut sg, 0);
        add_face_to_group(&mut sg, 1);
        assert_eq!(group_face_count_sg(&sg), 2);
    }

    #[test]
    fn test_auto_smooth_groups_same() {
        let normals = vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let sgs = auto_smooth_groups(&normals, 0.5);
        assert_eq!(smooth_group_count(&sgs), 1);
    }

    #[test]
    fn test_auto_smooth_groups_different() {
        let normals = vec![[0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];
        let sgs = auto_smooth_groups(&normals, 0.1);
        assert!(smooth_group_count(&sgs) >= 2);
    }

    #[test]
    fn test_smooth_groups_to_json() {
        let sgs = SmoothGroupSet { groups: vec![], face_to_group: vec![] };
        let j = smooth_groups_to_json(&sgs);
        assert!(j.contains("group_count"));
    }

    #[test]
    fn test_face_smooth_group() {
        let sgs = SmoothGroupSet {
            groups: vec![],
            face_to_group: vec![Some(0), None],
        };
        assert_eq!(face_smooth_group(&sgs, 0), Some(0));
        assert_eq!(face_smooth_group(&sgs, 1), None);
        assert_eq!(face_smooth_group(&sgs, 5), None);
    }

    #[test]
    fn test_clear_smooth_groups() {
        let mut sgs = auto_smooth_groups(&[[0.0, 1.0, 0.0]], 0.5);
        clear_smooth_groups(&mut sgs);
        assert!(sgs.groups.is_empty());
    }

    #[test]
    fn test_auto_smooth_empty() {
        let sgs = auto_smooth_groups(&[], 0.5);
        assert_eq!(smooth_group_count(&sgs), 0);
    }
}
