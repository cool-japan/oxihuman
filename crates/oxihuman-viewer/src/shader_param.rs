#![allow(dead_code)]

/// Shader parameter type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamType { Float, Vec3, Int }

/// A shader parameter.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShaderParam {
    name: String,
    ptype: ParamType,
    value_f32: f32,
    value_vec3: [f32; 3],
}

#[allow(dead_code)]
pub fn new_shader_param_sp(name: &str, ptype: ParamType) -> ShaderParam {
    ShaderParam { name: name.to_string(), ptype, value_f32: 0.0, value_vec3: [0.0; 3] }
}

#[allow(dead_code)]
pub fn param_name_sp(p: &ShaderParam) -> &str { &p.name }

#[allow(dead_code)]
pub fn param_type_sp(p: &ShaderParam) -> ParamType { p.ptype }

#[allow(dead_code)]
pub fn param_value_f32(p: &ShaderParam) -> f32 { p.value_f32 }

#[allow(dead_code)]
pub fn param_value_vec3(p: &ShaderParam) -> [f32; 3] { p.value_vec3 }

#[allow(dead_code)]
pub fn param_to_json_sp(p: &ShaderParam) -> String {
    let t = match p.ptype { ParamType::Float => "float", ParamType::Vec3 => "vec3", ParamType::Int => "int" };
    format!("{{\"name\":\"{}\",\"type\":\"{}\",\"value\":{:.4}}}", p.name, t, p.value_f32)
}

#[allow(dead_code)]
pub fn param_set_f32(p: &mut ShaderParam, val: f32) { p.value_f32 = val; }

#[allow(dead_code)]
pub fn param_count_sp(params: &[ShaderParam]) -> usize { params.len() }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let p = new_shader_param_sp("roughness", ParamType::Float); assert_eq!(param_name_sp(&p), "roughness"); }
    #[test] fn test_type() { assert_eq!(param_type_sp(&new_shader_param_sp("x", ParamType::Vec3)), ParamType::Vec3); }
    #[test] fn test_value_f32() { assert!((param_value_f32(&new_shader_param_sp("x", ParamType::Float))).abs() < 1e-6); }
    #[test] fn test_value_vec3() { assert!((param_value_vec3(&new_shader_param_sp("x", ParamType::Vec3))[0]).abs() < 1e-6); }
    #[test] fn test_set_f32() {
        let mut p = new_shader_param_sp("x", ParamType::Float);
        param_set_f32(&mut p, 0.5);
        assert!((param_value_f32(&p) - 0.5).abs() < 1e-6);
    }
    #[test] fn test_to_json() { assert!(param_to_json_sp(&new_shader_param_sp("x", ParamType::Float)).contains("float")); }
    #[test] fn test_count() {
        let params = vec![new_shader_param_sp("a", ParamType::Float), new_shader_param_sp("b", ParamType::Vec3)];
        assert_eq!(param_count_sp(&params), 2);
    }
    #[test] fn test_int_type() { assert_eq!(param_type_sp(&new_shader_param_sp("x", ParamType::Int)), ParamType::Int); }
    #[test] fn test_json_name() { assert!(param_to_json_sp(&new_shader_param_sp("metallic", ParamType::Float)).contains("metallic")); }
    #[test] fn test_count_empty() { let params: Vec<ShaderParam> = Vec::new(); assert_eq!(param_count_sp(&params), 0); }
}
