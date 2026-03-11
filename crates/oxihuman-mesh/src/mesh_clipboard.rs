#![allow(dead_code)]
//! Copy/paste mesh regions.

/// A clipboard for mesh data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MeshClipboard {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
}

/// Create a new empty clipboard.
#[allow(dead_code)]
pub fn new_mesh_clipboard() -> MeshClipboard {
    MeshClipboard {
        positions: Vec::new(),
        normals: Vec::new(),
        indices: Vec::new(),
    }
}

/// Copy faces from a mesh into the clipboard.
#[allow(dead_code)]
pub fn copy_faces(
    clipboard: &mut MeshClipboard,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    indices: &[[u32; 3]],
    face_selection: &[usize],
) {
    use std::collections::HashMap;
    clipboard.positions.clear();
    clipboard.normals.clear();
    clipboard.indices.clear();
    let mut vertex_map: HashMap<u32, u32> = HashMap::new();
    for &fi in face_selection {
        if fi >= indices.len() {
            continue;
        }
        let tri = indices[fi];
        let mut new_tri = [0u32; 3];
        for (i, &vi) in tri.iter().enumerate() {
            let new_vi = vertex_map.entry(vi).or_insert_with(|| {
                let idx = clipboard.positions.len() as u32;
                clipboard.positions.push(positions[vi as usize]);
                if (vi as usize) < normals.len() {
                    clipboard.normals.push(normals[vi as usize]);
                } else {
                    clipboard.normals.push([0.0, 0.0, 1.0]);
                }
                idx
            });
            new_tri[i] = *new_vi;
        }
        clipboard.indices.push(new_tri);
    }
}

/// Paste faces from clipboard into a mesh, returning combined data.
#[allow(dead_code, clippy::type_complexity)]
pub fn paste_faces(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    indices: &[[u32; 3]],
    clipboard: &MeshClipboard,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let offset = positions.len() as u32;
    let mut new_positions = positions.to_vec();
    let mut new_normals = normals.to_vec();
    let mut new_indices = indices.to_vec();
    new_positions.extend_from_slice(&clipboard.positions);
    new_normals.extend_from_slice(&clipboard.normals);
    for tri in &clipboard.indices {
        new_indices.push([tri[0] + offset, tri[1] + offset, tri[2] + offset]);
    }
    (new_positions, new_normals, new_indices)
}

/// Check if the clipboard is empty.
#[allow(dead_code)]
pub fn clipboard_is_empty(clipboard: &MeshClipboard) -> bool {
    clipboard.positions.is_empty()
}

/// Get the number of faces in the clipboard.
#[allow(dead_code)]
pub fn clipboard_face_count(clipboard: &MeshClipboard) -> usize {
    clipboard.indices.len()
}

/// Get the number of vertices in the clipboard.
#[allow(dead_code)]
pub fn clipboard_vertex_count(clipboard: &MeshClipboard) -> usize {
    clipboard.positions.len()
}

/// Clear the clipboard.
#[allow(dead_code)]
pub fn clipboard_clear(clipboard: &mut MeshClipboard) {
    clipboard.positions.clear();
    clipboard.normals.clear();
    clipboard.indices.clear();
}

/// Convert clipboard contents to a standalone mesh (positions, normals, indices).
#[allow(dead_code, clippy::type_complexity)]
pub fn clipboard_to_mesh(clipboard: &MeshClipboard) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[u32; 3]>) {
    (
        clipboard.positions.clone(),
        clipboard.normals.clone(),
        clipboard.indices.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::type_complexity)]
    fn sample_mesh() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        let n = vec![[0.0, 0.0, 1.0]; 4];
        let i = vec![[0u32, 1, 2], [1, 3, 2]];
        (p, n, i)
    }

    #[test]
    fn test_new_clipboard() {
        let cb = new_mesh_clipboard();
        assert!(clipboard_is_empty(&cb));
    }

    #[test]
    fn test_copy_faces() {
        let (p, n, i) = sample_mesh();
        let mut cb = new_mesh_clipboard();
        copy_faces(&mut cb, &p, &n, &i, &[0]);
        assert_eq!(clipboard_face_count(&cb), 1);
        assert_eq!(clipboard_vertex_count(&cb), 3);
    }

    #[test]
    fn test_paste_faces() {
        let (p, n, i) = sample_mesh();
        let mut cb = new_mesh_clipboard();
        copy_faces(&mut cb, &p, &n, &i, &[0]);
        let (np, nn, ni) = paste_faces(&p, &n, &i, &cb);
        assert_eq!(np.len(), 7);
        assert_eq!(nn.len(), 7);
        assert_eq!(ni.len(), 3);
    }

    #[test]
    fn test_clipboard_is_empty() {
        let cb = new_mesh_clipboard();
        assert!(clipboard_is_empty(&cb));
    }

    #[test]
    fn test_clipboard_face_count() {
        let (p, n, i) = sample_mesh();
        let mut cb = new_mesh_clipboard();
        copy_faces(&mut cb, &p, &n, &i, &[0, 1]);
        assert_eq!(clipboard_face_count(&cb), 2);
    }

    #[test]
    fn test_clipboard_vertex_count() {
        let (p, n, i) = sample_mesh();
        let mut cb = new_mesh_clipboard();
        copy_faces(&mut cb, &p, &n, &i, &[0, 1]);
        assert_eq!(clipboard_vertex_count(&cb), 4);
    }

    #[test]
    fn test_clipboard_clear() {
        let (p, n, i) = sample_mesh();
        let mut cb = new_mesh_clipboard();
        copy_faces(&mut cb, &p, &n, &i, &[0]);
        clipboard_clear(&mut cb);
        assert!(clipboard_is_empty(&cb));
    }

    #[test]
    fn test_clipboard_to_mesh() {
        let (p, n, i) = sample_mesh();
        let mut cb = new_mesh_clipboard();
        copy_faces(&mut cb, &p, &n, &i, &[0]);
        let (mp, mn, mi) = clipboard_to_mesh(&cb);
        assert_eq!(mp.len(), 3);
        assert_eq!(mn.len(), 3);
        assert_eq!(mi.len(), 1);
    }

    #[test]
    fn test_copy_out_of_bounds() {
        let (p, n, i) = sample_mesh();
        let mut cb = new_mesh_clipboard();
        copy_faces(&mut cb, &p, &n, &i, &[99]);
        assert!(clipboard_is_empty(&cb));
    }

    #[test]
    fn test_paste_empty_clipboard() {
        let (p, n, i) = sample_mesh();
        let cb = new_mesh_clipboard();
        let (np, _, ni) = paste_faces(&p, &n, &i, &cb);
        assert_eq!(np.len(), p.len());
        assert_eq!(ni.len(), i.len());
    }
}
