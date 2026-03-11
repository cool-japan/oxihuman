#![allow(dead_code)]
//! Render material: represents a material with shader and parameters.

use std::collections::HashMap;

/// A material parameter value.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum MaterialParam {
    Float(f32),
    Color([f32; 4]),
    Texture(String),
}

/// A render material.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderMaterial {
    name: String,
    shader: String,
    params: HashMap<String, MaterialParam>,
    transparent: bool,
}

/// Create a new render material.
#[allow(dead_code)]
pub fn new_render_material(name: &str, shader: &str) -> RenderMaterial {
    RenderMaterial {
        name: name.to_string(),
        shader: shader.to_string(),
        params: HashMap::new(),
        transparent: false,
    }
}

/// Set a parameter.
#[allow(dead_code)]
pub fn set_material_param(mat: &mut RenderMaterial, key: &str, value: MaterialParam) {
    mat.params.insert(key.to_string(), value);
}

/// Get a parameter reference.
#[allow(dead_code)]
pub fn get_material_param<'a>(mat: &'a RenderMaterial, key: &str) -> Option<&'a MaterialParam> {
    mat.params.get(key)
}

/// Return the number of parameters.
#[allow(dead_code)]
pub fn material_param_count(mat: &RenderMaterial) -> usize {
    mat.params.len()
}

/// Return the shader name.
#[allow(dead_code)]
pub fn material_shader(mat: &RenderMaterial) -> &str {
    &mat.shader
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn material_to_json(mat: &RenderMaterial) -> String {
    format!(
        "{{\"name\":\"{}\",\"shader\":\"{}\",\"param_count\":{},\"transparent\":{}}}",
        mat.name, mat.shader, mat.params.len(), mat.transparent
    )
}

/// Check if material is transparent.
#[allow(dead_code)]
pub fn material_is_transparent(mat: &RenderMaterial) -> bool {
    mat.transparent
}

/// Clone the material (with a new name).
#[allow(dead_code)]
pub fn material_clone(mat: &RenderMaterial, new_name: &str) -> RenderMaterial {
    let mut cloned = mat.clone();
    cloned.name = new_name.to_string();
    cloned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_material() {
        let m = new_render_material("skin", "pbr");
        assert_eq!(material_shader(&m), "pbr");
    }

    #[test]
    fn test_set_param() {
        let mut m = new_render_material("mat", "basic");
        set_material_param(&mut m, "roughness", MaterialParam::Float(0.5));
        assert_eq!(material_param_count(&m), 1);
    }

    #[test]
    fn test_get_param() {
        let mut m = new_render_material("mat", "basic");
        set_material_param(&mut m, "rough", MaterialParam::Float(0.3));
        assert!(get_material_param(&m, "rough").is_some());
    }

    #[test]
    fn test_get_missing_param() {
        let m = new_render_material("mat", "basic");
        assert!(get_material_param(&m, "nope").is_none());
    }

    #[test]
    fn test_to_json() {
        let m = new_render_material("test", "pbr");
        let json = material_to_json(&m);
        assert!(json.contains("\"name\":\"test\""));
    }

    #[test]
    fn test_transparent() {
        let m = new_render_material("glass", "transparent");
        assert!(!material_is_transparent(&m));
    }

    #[test]
    fn test_clone_material() {
        let m = new_render_material("orig", "pbr");
        let c = material_clone(&m, "copy");
        assert_eq!(material_shader(&c), "pbr");
    }

    #[test]
    fn test_color_param() {
        let mut m = new_render_material("mat", "basic");
        set_material_param(&mut m, "albedo", MaterialParam::Color([1.0, 0.0, 0.0, 1.0]));
        assert!(get_material_param(&m, "albedo").is_some());
    }

    #[test]
    fn test_texture_param() {
        let mut m = new_render_material("mat", "basic");
        set_material_param(&mut m, "diffuse", MaterialParam::Texture("tex.png".to_string()));
        assert_eq!(material_param_count(&m), 1);
    }

    #[test]
    fn test_param_count_empty() {
        let m = new_render_material("mat", "basic");
        assert_eq!(material_param_count(&m), 0);
    }
}
