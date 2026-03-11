#![allow(dead_code)]
//! Mesh draw call: represents a single draw call for a mesh.

/// A single mesh draw call.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshDrawCall {
    vertex_count: u32,
    index_count: u32,
    material_name: String,
    transform: [f32; 16],
    transparent: bool,
}

/// Create a new draw call.
#[allow(dead_code)]
pub fn new_mesh_draw_call(
    vertex_count: u32,
    index_count: u32,
    material_name: &str,
    transparent: bool,
) -> MeshDrawCall {
    MeshDrawCall {
        vertex_count,
        index_count,
        material_name: material_name.to_string(),
        transform: [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ],
        transparent,
    }
}

/// Return the vertex count.
#[allow(dead_code)]
pub fn draw_vertex_count(dc: &MeshDrawCall) -> u32 {
    dc.vertex_count
}

/// Return the index count.
#[allow(dead_code)]
pub fn draw_index_count(dc: &MeshDrawCall) -> u32 {
    dc.index_count
}

/// Return the material name.
#[allow(dead_code)]
pub fn draw_material(dc: &MeshDrawCall) -> &str {
    &dc.material_name
}

/// Return the transform matrix.
#[allow(dead_code)]
pub fn draw_transform(dc: &MeshDrawCall) -> &[f32; 16] {
    &dc.transform
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn draw_to_json(dc: &MeshDrawCall) -> String {
    format!(
        "{{\"vertex_count\":{},\"index_count\":{},\"material\":\"{}\",\"transparent\":{}}}",
        dc.vertex_count, dc.index_count, dc.material_name, dc.transparent
    )
}

/// Check if the draw call is for a transparent material.
#[allow(dead_code)]
pub fn draw_is_transparent(dc: &MeshDrawCall) -> bool {
    dc.transparent
}

/// Return a sort key (transparent items sort after opaque).
#[allow(dead_code)]
pub fn draw_sort_key(dc: &MeshDrawCall) -> u32 {
    if dc.transparent { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_draw_call() {
        let dc = new_mesh_draw_call(100, 300, "default", false);
        assert_eq!(draw_vertex_count(&dc), 100);
    }

    #[test]
    fn test_index_count() {
        let dc = new_mesh_draw_call(100, 300, "default", false);
        assert_eq!(draw_index_count(&dc), 300);
    }

    #[test]
    fn test_material() {
        let dc = new_mesh_draw_call(100, 300, "skin", false);
        assert_eq!(draw_material(&dc), "skin");
    }

    #[test]
    fn test_transform_identity() {
        let dc = new_mesh_draw_call(100, 300, "default", false);
        let t = draw_transform(&dc);
        assert!((t[0] - 1.0).abs() < 1e-6);
        assert!((t[5] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let dc = new_mesh_draw_call(10, 30, "mat", false);
        let json = draw_to_json(&dc);
        assert!(json.contains("\"vertex_count\":10"));
    }

    #[test]
    fn test_transparent() {
        let dc = new_mesh_draw_call(10, 30, "glass", true);
        assert!(draw_is_transparent(&dc));
    }

    #[test]
    fn test_opaque() {
        let dc = new_mesh_draw_call(10, 30, "metal", false);
        assert!(!draw_is_transparent(&dc));
    }

    #[test]
    fn test_sort_key_opaque() {
        let dc = new_mesh_draw_call(10, 30, "mat", false);
        assert_eq!(draw_sort_key(&dc), 0);
    }

    #[test]
    fn test_sort_key_transparent() {
        let dc = new_mesh_draw_call(10, 30, "mat", true);
        assert_eq!(draw_sort_key(&dc), 1);
    }

    #[test]
    fn test_zero_counts() {
        let dc = new_mesh_draw_call(0, 0, "empty", false);
        assert_eq!(draw_vertex_count(&dc), 0);
        assert_eq!(draw_index_count(&dc), 0);
    }
}
