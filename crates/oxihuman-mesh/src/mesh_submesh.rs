#![allow(dead_code)]
//! Submesh definitions within a mesh.

/// A single submesh.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Submesh {
    pub material_name: String,
    pub vertex_start: u32,
    pub vertex_count: u32,
    pub index_start: u32,
    pub index_count: u32,
}

/// A collection of submeshes.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SubmeshSet {
    pub submeshes: Vec<Submesh>,
}

/// Create a new submesh set.
#[allow(dead_code)]
pub fn new_submesh_set() -> SubmeshSet {
    SubmeshSet {
        submeshes: Vec::new(),
    }
}

/// Add a submesh.
#[allow(dead_code)]
pub fn add_submesh(
    set: &mut SubmeshSet,
    material_name: &str,
    vertex_start: u32,
    vertex_count: u32,
    index_start: u32,
    index_count: u32,
) {
    set.submeshes.push(Submesh {
        material_name: material_name.to_string(),
        vertex_start,
        vertex_count,
        index_start,
        index_count,
    });
}

/// Return submesh count.
#[allow(dead_code)]
pub fn submesh_count_sm(set: &SubmeshSet) -> usize {
    set.submeshes.len()
}

/// Get vertex range for a submesh.
#[allow(dead_code)]
pub fn submesh_vertex_range(set: &SubmeshSet, index: usize) -> (u32, u32) {
    if index < set.submeshes.len() {
        let s = &set.submeshes[index];
        (s.vertex_start, s.vertex_start + s.vertex_count)
    } else {
        (0, 0)
    }
}

/// Get index range for a submesh.
#[allow(dead_code)]
pub fn submesh_index_range(set: &SubmeshSet, index: usize) -> (u32, u32) {
    if index < set.submeshes.len() {
        let s = &set.submeshes[index];
        (s.index_start, s.index_start + s.index_count)
    } else {
        (0, 0)
    }
}

/// Get material name for a submesh.
#[allow(dead_code)]
pub fn submesh_material(set: &SubmeshSet, index: usize) -> &str {
    if index < set.submeshes.len() {
        &set.submeshes[index].material_name
    } else {
        ""
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn submesh_to_json(set: &SubmeshSet) -> String {
    let items: Vec<String> = set
        .submeshes
        .iter()
        .map(|s| {
            format!(
                "{{\"material\":\"{}\",\"vertex_start\":{},\"vertex_count\":{},\"index_start\":{},\"index_count\":{}}}",
                s.material_name, s.vertex_start, s.vertex_count, s.index_start, s.index_count
            )
        })
        .collect();
    format!("{{\"submeshes\":[{}]}}", items.join(","))
}

/// Get face count (index_count / 3) for a submesh.
#[allow(dead_code)]
pub fn submesh_face_count(set: &SubmeshSet, index: usize) -> u32 {
    if index < set.submeshes.len() {
        set.submeshes[index].index_count / 3
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_submesh_set() {
        let s = new_submesh_set();
        assert_eq!(submesh_count_sm(&s), 0);
    }

    #[test]
    fn test_add_submesh() {
        let mut s = new_submesh_set();
        add_submesh(&mut s, "mat0", 0, 100, 0, 300);
        assert_eq!(submesh_count_sm(&s), 1);
    }

    #[test]
    fn test_submesh_vertex_range() {
        let mut s = new_submesh_set();
        add_submesh(&mut s, "mat0", 10, 50, 0, 150);
        assert_eq!(submesh_vertex_range(&s, 0), (10, 60));
    }

    #[test]
    fn test_submesh_index_range() {
        let mut s = new_submesh_set();
        add_submesh(&mut s, "mat0", 0, 10, 20, 30);
        assert_eq!(submesh_index_range(&s, 0), (20, 50));
    }

    #[test]
    fn test_submesh_material() {
        let mut s = new_submesh_set();
        add_submesh(&mut s, "skin", 0, 10, 0, 30);
        assert_eq!(submesh_material(&s, 0), "skin");
    }

    #[test]
    fn test_submesh_material_oob() {
        let s = new_submesh_set();
        assert_eq!(submesh_material(&s, 0), "");
    }

    #[test]
    fn test_submesh_to_json() {
        let mut s = new_submesh_set();
        add_submesh(&mut s, "mat0", 0, 10, 0, 30);
        let j = submesh_to_json(&s);
        assert!(j.contains("\"submeshes\""));
    }

    #[test]
    fn test_submesh_face_count() {
        let mut s = new_submesh_set();
        add_submesh(&mut s, "mat0", 0, 10, 0, 30);
        assert_eq!(submesh_face_count(&s, 0), 10);
    }

    #[test]
    fn test_submesh_face_count_oob() {
        let s = new_submesh_set();
        assert_eq!(submesh_face_count(&s, 0), 0);
    }

    #[test]
    fn test_multiple_submeshes() {
        let mut s = new_submesh_set();
        add_submesh(&mut s, "a", 0, 10, 0, 30);
        add_submesh(&mut s, "b", 10, 20, 30, 60);
        assert_eq!(submesh_count_sm(&s), 2);
        assert_eq!(submesh_material(&s, 1), "b");
    }
}
