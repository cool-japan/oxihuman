#![allow(dead_code)]
//! Render object: a renderable object in the scene.

/// The type of render object.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    Mesh,
    Light,
    Camera,
    Empty,
}

/// A renderable object.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderObject {
    mesh_name: String,
    material_name: String,
    transform: [f32; 16],
    visible: bool,
    object_type: ObjectType,
    bounding_radius: f32,
}

/// Create a new render object.
#[allow(dead_code)]
pub fn new_render_object(mesh: &str, material: &str, object_type: ObjectType) -> RenderObject {
    RenderObject {
        mesh_name: mesh.to_string(),
        material_name: material.to_string(),
        transform: [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ],
        visible: true,
        object_type,
        bounding_radius: 1.0,
    }
}

/// Return the mesh name.
#[allow(dead_code)]
pub fn object_mesh(obj: &RenderObject) -> &str {
    &obj.mesh_name
}

/// Return the material name.
#[allow(dead_code)]
pub fn object_material(obj: &RenderObject) -> &str {
    &obj.material_name
}

/// Return the transform.
#[allow(dead_code)]
pub fn object_transform(obj: &RenderObject) -> &[f32; 16] {
    &obj.transform
}

/// Check if the object is visible.
#[allow(dead_code)]
pub fn object_is_visible(obj: &RenderObject) -> bool {
    obj.visible
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn object_to_json(obj: &RenderObject) -> String {
    let type_str = match obj.object_type {
        ObjectType::Mesh => "mesh",
        ObjectType::Light => "light",
        ObjectType::Camera => "camera",
        ObjectType::Empty => "empty",
    };
    format!(
        "{{\"mesh\":\"{}\",\"material\":\"{}\",\"type\":\"{}\",\"visible\":{}}}",
        obj.mesh_name, obj.material_name, type_str, obj.visible
    )
}

/// Return the bounding radius.
#[allow(dead_code)]
pub fn object_bounds(obj: &RenderObject) -> f32 {
    obj.bounding_radius
}

/// Return a sort key based on object type.
#[allow(dead_code)]
pub fn object_sort_key(obj: &RenderObject) -> u32 {
    match obj.object_type {
        ObjectType::Mesh => 0,
        ObjectType::Light => 1,
        ObjectType::Camera => 2,
        ObjectType::Empty => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_object() {
        let o = new_render_object("body", "skin", ObjectType::Mesh);
        assert_eq!(object_mesh(&o), "body");
    }

    #[test]
    fn test_material() {
        let o = new_render_object("body", "skin", ObjectType::Mesh);
        assert_eq!(object_material(&o), "skin");
    }

    #[test]
    fn test_visible() {
        let o = new_render_object("body", "skin", ObjectType::Mesh);
        assert!(object_is_visible(&o));
    }

    #[test]
    fn test_transform_identity() {
        let o = new_render_object("body", "skin", ObjectType::Mesh);
        let t = object_transform(&o);
        assert!((t[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let o = new_render_object("body", "skin", ObjectType::Mesh);
        let json = object_to_json(&o);
        assert!(json.contains("\"type\":\"mesh\""));
    }

    #[test]
    fn test_bounds() {
        let o = new_render_object("body", "skin", ObjectType::Mesh);
        assert!((object_bounds(&o) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sort_key_mesh() {
        let o = new_render_object("body", "skin", ObjectType::Mesh);
        assert_eq!(object_sort_key(&o), 0);
    }

    #[test]
    fn test_sort_key_light() {
        let o = new_render_object("", "", ObjectType::Light);
        assert_eq!(object_sort_key(&o), 1);
    }

    #[test]
    fn test_object_type_camera() {
        let o = new_render_object("", "", ObjectType::Camera);
        assert_eq!(object_sort_key(&o), 2);
    }

    #[test]
    fn test_object_type_empty() {
        let o = new_render_object("", "", ObjectType::Empty);
        assert_eq!(object_sort_key(&o), 3);
    }
}
