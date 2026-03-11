#![allow(dead_code)]

/// Wireframe representation of a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshWireframe {
    lines: Vec<([f32; 3], [f32; 3])>,
    color: [f32; 3],
    width: f32,
}

#[allow(dead_code)]
pub fn new_mesh_wireframe() -> MeshWireframe {
    MeshWireframe { lines: Vec::new(), color: [1.0, 1.0, 1.0], width: 1.0 }
}

#[allow(dead_code)]
pub fn wireframe_from_mesh(positions: &[[f32; 3]], indices: &[u32]) -> MeshWireframe {
    let mut lines = Vec::new();
    for tri in indices.chunks(3) {
        if tri.len() == 3 {
            let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
            if a < positions.len() && b < positions.len() && c < positions.len() {
                lines.push((positions[a], positions[b]));
                lines.push((positions[b], positions[c]));
                lines.push((positions[c], positions[a]));
            }
        }
    }
    MeshWireframe { lines, color: [1.0, 1.0, 1.0], width: 1.0 }
}

#[allow(dead_code)]
pub fn wireframe_line_count(w: &MeshWireframe) -> usize { w.lines.len() }

#[allow(dead_code)]
pub fn wireframe_vertex_count_mw(w: &MeshWireframe) -> usize { w.lines.len() * 2 }

#[allow(dead_code)]
pub fn wireframe_to_json(w: &MeshWireframe) -> String {
    format!("{{\"lines\":{},\"width\":{:.2}}}", w.lines.len(), w.width)
}

#[allow(dead_code)]
pub fn wireframe_color(w: &MeshWireframe) -> [f32; 3] { w.color }

#[allow(dead_code)]
pub fn wireframe_width(w: &MeshWireframe) -> f32 { w.width }

#[allow(dead_code)]
pub fn wireframe_clear(w: &mut MeshWireframe) { w.lines.clear(); }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert_eq!(wireframe_line_count(&new_mesh_wireframe()), 0); }
    #[test] fn test_from_mesh() {
        let pos = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = [0u32, 1, 2];
        let w = wireframe_from_mesh(&pos, &idx);
        assert_eq!(wireframe_line_count(&w), 3);
    }
    #[test] fn test_vertex_count() {
        let pos = [[0.0; 3]; 3];
        let idx = [0u32, 1, 2];
        let w = wireframe_from_mesh(&pos, &idx);
        assert_eq!(wireframe_vertex_count_mw(&w), 6);
    }
    #[test] fn test_color() { assert!((wireframe_color(&new_mesh_wireframe())[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_width() { assert!((wireframe_width(&new_mesh_wireframe()) - 1.0).abs() < 1e-6); }
    #[test] fn test_to_json() { assert!(wireframe_to_json(&new_mesh_wireframe()).contains("lines")); }
    #[test] fn test_clear() {
        let pos = [[0.0; 3]; 3]; let idx = [0u32, 1, 2];
        let mut w = wireframe_from_mesh(&pos, &idx);
        wireframe_clear(&mut w);
        assert_eq!(wireframe_line_count(&w), 0);
    }
    #[test] fn test_empty_indices() { let w = wireframe_from_mesh(&[], &[]); assert_eq!(wireframe_line_count(&w), 0); }
    #[test] fn test_partial_tri() { let w = wireframe_from_mesh(&[[0.0; 3]], &[0, 1]); assert_eq!(wireframe_line_count(&w), 0); }
    #[test] fn test_oob_indices() { let w = wireframe_from_mesh(&[[0.0; 3]], &[0, 1, 2]); assert_eq!(wireframe_line_count(&w), 0); }
}
