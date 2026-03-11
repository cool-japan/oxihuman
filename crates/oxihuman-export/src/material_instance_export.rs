#![allow(dead_code)]
//! Export material instances.

/// Material instance export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialInstanceExport {
    pub shader: String,
    pub params: Vec<(String, f32)>,
    pub textures: Vec<String>,
}

/// Export a material instance.
#[allow(dead_code)]
pub fn export_material_instance(
    shader: &str,
    params: &[(&str, f32)],
    textures: &[&str],
) -> MaterialInstanceExport {
    MaterialInstanceExport {
        shader: shader.to_string(),
        params: params.iter().map(|&(k, v)| (k.to_string(), v)).collect(),
        textures: textures.iter().map(|s| s.to_string()).collect(),
    }
}

/// Return shader name.
#[allow(dead_code)]
pub fn instance_shader(exp: &MaterialInstanceExport) -> &str {
    &exp.shader
}

/// Return parameter count.
#[allow(dead_code)]
pub fn instance_param_count(exp: &MaterialInstanceExport) -> usize {
    exp.params.len()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn instance_to_json(exp: &MaterialInstanceExport) -> String {
    let params: Vec<String> = exp
        .params
        .iter()
        .map(|(k, v)| format!("\"{}\":{:.4}", k, v))
        .collect();
    format!(
        "{{\"shader\":\"{}\",\"params\":{{{}}},\"textures\":{}}}",
        exp.shader,
        params.join(","),
        exp.textures.len()
    )
}

/// Return texture count.
#[allow(dead_code)]
pub fn instance_texture_count(exp: &MaterialInstanceExport) -> usize {
    exp.textures.len()
}

/// Serialize to bytes.
#[allow(dead_code)]
pub fn instance_to_bytes(exp: &MaterialInstanceExport) -> Vec<u8> {
    instance_to_json(exp).into_bytes()
}

/// Compute export size.
#[allow(dead_code)]
pub fn instance_export_size(exp: &MaterialInstanceExport) -> usize {
    instance_to_bytes(exp).len()
}

/// Validate material instance.
#[allow(dead_code)]
pub fn validate_material_instance(exp: &MaterialInstanceExport) -> bool {
    !exp.shader.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_material_instance() {
        let e = export_material_instance("pbr", &[("roughness", 0.5)], &["diffuse.png"]);
        assert_eq!(instance_shader(&e), "pbr");
    }

    #[test]
    fn test_instance_param_count() {
        let e = export_material_instance("pbr", &[("a", 1.0), ("b", 2.0)], &[]);
        assert_eq!(instance_param_count(&e), 2);
    }

    #[test]
    fn test_instance_to_json() {
        let e = export_material_instance("basic", &[], &[]);
        let j = instance_to_json(&e);
        assert!(j.contains("\"shader\":\"basic\""));
    }

    #[test]
    fn test_instance_texture_count() {
        let e = export_material_instance("pbr", &[], &["a.png", "b.png"]);
        assert_eq!(instance_texture_count(&e), 2);
    }

    #[test]
    fn test_instance_to_bytes() {
        let e = export_material_instance("pbr", &[], &[]);
        assert!(!instance_to_bytes(&e).is_empty());
    }

    #[test]
    fn test_instance_export_size() {
        let e = export_material_instance("pbr", &[], &[]);
        assert!(instance_export_size(&e) > 0);
    }

    #[test]
    fn test_validate_material_instance() {
        let e = export_material_instance("pbr", &[], &[]);
        assert!(validate_material_instance(&e));
    }

    #[test]
    fn test_validate_empty_shader() {
        let e = export_material_instance("", &[], &[]);
        assert!(!validate_material_instance(&e));
    }

    #[test]
    fn test_empty_instance() {
        let e = export_material_instance("s", &[], &[]);
        assert_eq!(instance_param_count(&e), 0);
        assert_eq!(instance_texture_count(&e), 0);
    }

    #[test]
    fn test_instance_shader_name() {
        let e = export_material_instance("toon_shader", &[], &[]);
        assert_eq!(instance_shader(&e), "toon_shader");
    }
}
