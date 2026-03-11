#![allow(dead_code)]
//! Vertex/face selection sets.

/// Type of selection element.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionType {
    Vertex,
    Face,
}

/// A mesh selection set.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MeshSelection {
    pub selection_type: SelectionType,
    pub selected_vertices: Vec<bool>,
    pub selected_faces: Vec<bool>,
}

/// Create a new empty selection.
#[allow(dead_code)]
pub fn new_mesh_selection(vertex_count: usize, face_count: usize) -> MeshSelection {
    MeshSelection {
        selection_type: SelectionType::Vertex,
        selected_vertices: vec![false; vertex_count],
        selected_faces: vec![false; face_count],
    }
}

/// Select a vertex by index.
#[allow(dead_code)]
pub fn select_vertex(sel: &mut MeshSelection, vertex: usize) {
    if vertex < sel.selected_vertices.len() {
        sel.selected_vertices[vertex] = true;
    }
}

/// Select a face by index.
#[allow(dead_code)]
pub fn select_face(sel: &mut MeshSelection, face: usize) {
    if face < sel.selected_faces.len() {
        sel.selected_faces[face] = true;
    }
}

/// Deselect all vertices and faces.
#[allow(dead_code)]
pub fn deselect_all(sel: &mut MeshSelection) {
    for v in &mut sel.selected_vertices {
        *v = false;
    }
    for f in &mut sel.selected_faces {
        *f = false;
    }
}

/// Count selected vertices.
#[allow(dead_code)]
pub fn selected_vertex_count(sel: &MeshSelection) -> usize {
    sel.selected_vertices.iter().filter(|&&v| v).count()
}

/// Count selected faces.
#[allow(dead_code)]
pub fn selected_face_count(sel: &MeshSelection) -> usize {
    sel.selected_faces.iter().filter(|&&f| f).count()
}

/// Grow vertex selection by one ring (include neighbors of selected vertices).
#[allow(dead_code)]
pub fn selection_grow(sel: &mut MeshSelection, indices: &[[u32; 3]]) {
    let current: Vec<usize> = sel
        .selected_vertices
        .iter()
        .enumerate()
        .filter(|(_, &s)| s)
        .map(|(i, _)| i)
        .collect();
    for &vi in &current {
        for tri in indices {
            if tri.contains(&(vi as u32)) {
                for &v in tri {
                    if (v as usize) < sel.selected_vertices.len() {
                        sel.selected_vertices[v as usize] = true;
                    }
                }
            }
        }
    }
}

/// Shrink vertex selection by one ring (deselect boundary vertices of selection).
#[allow(dead_code)]
pub fn selection_shrink(sel: &mut MeshSelection, indices: &[[u32; 3]]) {
    let mut to_deselect = Vec::new();
    for (vi, &selected) in sel.selected_vertices.iter().enumerate() {
        if !selected {
            continue;
        }
        let mut all_neighbors_selected = true;
        for tri in indices {
            if tri.contains(&(vi as u32)) {
                for &v in tri {
                    if (v as usize) < sel.selected_vertices.len()
                        && v as usize != vi
                        && !sel.selected_vertices[v as usize]
                    {
                        all_neighbors_selected = false;
                    }
                }
            }
        }
        if !all_neighbors_selected {
            to_deselect.push(vi);
        }
    }
    for vi in to_deselect {
        sel.selected_vertices[vi] = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mesh_selection() {
        let sel = new_mesh_selection(10, 5);
        assert_eq!(sel.selected_vertices.len(), 10);
        assert_eq!(sel.selected_faces.len(), 5);
    }

    #[test]
    fn test_select_vertex() {
        let mut sel = new_mesh_selection(3, 1);
        select_vertex(&mut sel, 1);
        assert!(sel.selected_vertices[1]);
        assert!(!sel.selected_vertices[0]);
    }

    #[test]
    fn test_select_face() {
        let mut sel = new_mesh_selection(3, 2);
        select_face(&mut sel, 0);
        assert!(sel.selected_faces[0]);
    }

    #[test]
    fn test_deselect_all() {
        let mut sel = new_mesh_selection(3, 1);
        select_vertex(&mut sel, 0);
        select_face(&mut sel, 0);
        deselect_all(&mut sel);
        assert_eq!(selected_vertex_count(&sel), 0);
        assert_eq!(selected_face_count(&sel), 0);
    }

    #[test]
    fn test_selected_vertex_count() {
        let mut sel = new_mesh_selection(5, 1);
        select_vertex(&mut sel, 0);
        select_vertex(&mut sel, 2);
        assert_eq!(selected_vertex_count(&sel), 2);
    }

    #[test]
    fn test_selected_face_count() {
        let mut sel = new_mesh_selection(3, 3);
        select_face(&mut sel, 1);
        assert_eq!(selected_face_count(&sel), 1);
    }

    #[test]
    fn test_selection_grow() {
        let mut sel = new_mesh_selection(4, 2);
        let indices = vec![[0u32, 1, 2], [1, 3, 2]];
        select_vertex(&mut sel, 0);
        selection_grow(&mut sel, &indices);
        assert!(sel.selected_vertices[1]);
        assert!(sel.selected_vertices[2]);
    }

    #[test]
    fn test_selection_shrink() {
        let mut sel = new_mesh_selection(4, 2);
        let indices = vec![[0u32, 1, 2], [1, 3, 2]];
        for i in 0..4 {
            select_vertex(&mut sel, i);
        }
        let before = selected_vertex_count(&sel);
        selection_shrink(&mut sel, &indices);
        // Shrinking should reduce or keep selection count
        assert!(selected_vertex_count(&sel) <= before);
    }

    #[test]
    fn test_select_out_of_bounds() {
        let mut sel = new_mesh_selection(3, 1);
        select_vertex(&mut sel, 10);
        assert_eq!(selected_vertex_count(&sel), 0);
    }

    #[test]
    fn test_selection_type() {
        let sel = new_mesh_selection(1, 1);
        assert_eq!(sel.selection_type, SelectionType::Vertex);
    }
}
