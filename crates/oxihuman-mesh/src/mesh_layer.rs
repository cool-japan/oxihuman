#![allow(dead_code)]
//! Mesh layer for organizing vertex/face subsets.

/// Layer type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerType {
    Geometry,
    Selection,
    Paint,
    Custom,
}

/// A mesh layer.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MeshLayer {
    pub name: String,
    pub layer_type: LayerType,
    pub vertex_indices: Vec<u32>,
    pub face_indices: Vec<u32>,
}

/// Create a new mesh layer.
#[allow(dead_code)]
pub fn new_mesh_layer(name: &str, layer_type: LayerType) -> MeshLayer {
    MeshLayer {
        name: name.to_string(),
        layer_type,
        vertex_indices: Vec::new(),
        face_indices: Vec::new(),
    }
}

/// Return vertex count in the layer.
#[allow(dead_code)]
pub fn layer_vertex_count(layer: &MeshLayer) -> usize {
    layer.vertex_indices.len()
}

/// Return face count in the layer.
#[allow(dead_code)]
pub fn layer_face_count(layer: &MeshLayer) -> usize {
    layer.face_indices.len()
}

/// Return layer name.
#[allow(dead_code)]
pub fn layer_name_ml(layer: &MeshLayer) -> &str {
    &layer.name
}

/// Return layer type.
#[allow(dead_code)]
pub fn layer_type(layer: &MeshLayer) -> LayerType {
    layer.layer_type
}

/// Merge another layer into this one.
#[allow(dead_code)]
pub fn merge_layers(dst: &mut MeshLayer, src: &MeshLayer) {
    dst.vertex_indices.extend_from_slice(&src.vertex_indices);
    dst.face_indices.extend_from_slice(&src.face_indices);
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn layer_to_json(layer: &MeshLayer) -> String {
    format!(
        "{{\"name\":\"{}\",\"type\":\"{:?}\",\"vertices\":{},\"faces\":{}}}",
        layer.name,
        layer.layer_type,
        layer.vertex_indices.len(),
        layer.face_indices.len()
    )
}

/// Check if the layer is empty.
#[allow(dead_code)]
pub fn layer_is_empty(layer: &MeshLayer) -> bool {
    layer.vertex_indices.is_empty() && layer.face_indices.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mesh_layer() {
        let l = new_mesh_layer("base", LayerType::Geometry);
        assert!(layer_is_empty(&l));
    }

    #[test]
    fn test_layer_name() {
        let l = new_mesh_layer("detail", LayerType::Custom);
        assert_eq!(layer_name_ml(&l), "detail");
    }

    #[test]
    fn test_layer_type() {
        let l = new_mesh_layer("sel", LayerType::Selection);
        assert_eq!(layer_type(&l), LayerType::Selection);
    }

    #[test]
    fn test_layer_vertex_count() {
        let mut l = new_mesh_layer("l", LayerType::Geometry);
        l.vertex_indices = vec![0, 1, 2];
        assert_eq!(layer_vertex_count(&l), 3);
    }

    #[test]
    fn test_layer_face_count() {
        let mut l = new_mesh_layer("l", LayerType::Geometry);
        l.face_indices = vec![0, 1];
        assert_eq!(layer_face_count(&l), 2);
    }

    #[test]
    fn test_merge_layers() {
        let mut a = new_mesh_layer("a", LayerType::Geometry);
        a.vertex_indices = vec![0, 1];
        let mut b = new_mesh_layer("b", LayerType::Geometry);
        b.vertex_indices = vec![2, 3];
        merge_layers(&mut a, &b);
        assert_eq!(layer_vertex_count(&a), 4);
    }

    #[test]
    fn test_layer_to_json() {
        let l = new_mesh_layer("l", LayerType::Paint);
        let j = layer_to_json(&l);
        assert!(j.contains("\"name\":\"l\""));
    }

    #[test]
    fn test_layer_is_empty() {
        let l = new_mesh_layer("l", LayerType::Custom);
        assert!(layer_is_empty(&l));
    }

    #[test]
    fn test_layer_not_empty() {
        let mut l = new_mesh_layer("l", LayerType::Geometry);
        l.vertex_indices.push(0);
        assert!(!layer_is_empty(&l));
    }

    #[test]
    fn test_merge_empty() {
        let mut a = new_mesh_layer("a", LayerType::Geometry);
        let b = new_mesh_layer("b", LayerType::Geometry);
        merge_layers(&mut a, &b);
        assert!(layer_is_empty(&a));
    }
}
