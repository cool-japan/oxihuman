#![allow(dead_code)]
//! Mesh shape key (blend shape) storage.

/// A shape key with per-vertex deltas.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MeshShapeKey {
    pub name: String,
    pub deltas: Vec<[f32; 3]>,
    pub weight: f32,
}

/// Create a new shape key.
#[allow(dead_code)]
pub fn new_mesh_shape_key(name: &str, vertex_count: usize) -> MeshShapeKey {
    MeshShapeKey {
        name: name.to_string(),
        deltas: vec![[0.0; 3]; vertex_count],
        weight: 0.0,
    }
}

/// Get delta at a vertex.
#[allow(dead_code)]
pub fn shape_key_delta(key: &MeshShapeKey, index: usize) -> [f32; 3] {
    if index < key.deltas.len() {
        key.deltas[index]
    } else {
        [0.0; 3]
    }
}

/// Return shape key name.
#[allow(dead_code)]
pub fn shape_key_name(key: &MeshShapeKey) -> &str {
    &key.name
}

/// Set the weight.
#[allow(dead_code)]
pub fn set_shape_weight(key: &mut MeshShapeKey, weight: f32) {
    key.weight = weight.clamp(0.0, 1.0);
}

/// Get the weight.
#[allow(dead_code)]
pub fn get_shape_weight(key: &MeshShapeKey) -> f32 {
    key.weight
}

/// Apply the shape key to positions based on current weight.
#[allow(dead_code)]
pub fn apply_shape_key(key: &MeshShapeKey, positions: &mut [[f32; 3]]) {
    let w = key.weight;
    for (pos, d) in positions.iter_mut().zip(key.deltas.iter()) {
        pos[0] += d[0] * w;
        pos[1] += d[1] * w;
        pos[2] += d[2] * w;
    }
}

/// Return vertex count.
#[allow(dead_code)]
pub fn shape_key_vertex_count(key: &MeshShapeKey) -> usize {
    key.deltas.len()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn shape_key_to_json(key: &MeshShapeKey) -> String {
    format!(
        "{{\"name\":\"{}\",\"vertex_count\":{},\"weight\":{:.4}}}",
        key.name,
        key.deltas.len(),
        key.weight
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mesh_shape_key() {
        let k = new_mesh_shape_key("smile", 10);
        assert_eq!(shape_key_vertex_count(&k), 10);
    }

    #[test]
    fn test_shape_key_name() {
        let k = new_mesh_shape_key("blink", 1);
        assert_eq!(shape_key_name(&k), "blink");
    }

    #[test]
    fn test_set_get_weight() {
        let mut k = new_mesh_shape_key("t", 1);
        set_shape_weight(&mut k, 0.5);
        assert!((get_shape_weight(&k) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_weight_clamp() {
        let mut k = new_mesh_shape_key("t", 1);
        set_shape_weight(&mut k, 2.0);
        assert!((get_shape_weight(&k) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_shape_key_delta() {
        let mut k = new_mesh_shape_key("t", 2);
        k.deltas[0] = [1.0, 2.0, 3.0];
        assert_eq!(shape_key_delta(&k, 0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_shape_key_delta_oob() {
        let k = new_mesh_shape_key("t", 1);
        assert_eq!(shape_key_delta(&k, 10), [0.0; 3]);
    }

    #[test]
    fn test_apply_shape_key() {
        let mut k = new_mesh_shape_key("t", 1);
        k.deltas[0] = [10.0, 0.0, 0.0];
        set_shape_weight(&mut k, 0.5);
        let mut pos = [[0.0, 0.0, 0.0]];
        apply_shape_key(&k, &mut pos);
        assert!((pos[0][0] - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_shape_key_to_json() {
        let k = new_mesh_shape_key("smile", 5);
        let j = shape_key_to_json(&k);
        assert!(j.contains("\"name\":\"smile\""));
    }

    #[test]
    fn test_zero_weight_no_effect() {
        let mut k = new_mesh_shape_key("t", 1);
        k.deltas[0] = [10.0, 10.0, 10.0];
        let mut pos = [[0.0; 3]];
        apply_shape_key(&k, &mut pos);
        assert_eq!(pos[0], [0.0; 3]);
    }

    #[test]
    fn test_empty_shape_key() {
        let k = new_mesh_shape_key("e", 0);
        assert_eq!(shape_key_vertex_count(&k), 0);
    }
}
