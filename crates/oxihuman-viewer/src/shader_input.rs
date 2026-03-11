#![allow(dead_code)]
//! Shader input: describes a vertex input to a shader program.

/// The semantic meaning of a shader input.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum InputSemantic {
    Position,
    Normal,
    TexCoord,
    Color,
    Tangent,
    Custom(String),
}

/// A shader input binding.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShaderInput {
    name: String,
    semantic: InputSemantic,
    location: u32,
    format: String,
    stride: u32,
}

/// Create a new shader input.
#[allow(dead_code)]
pub fn new_shader_input(
    name: &str,
    semantic: InputSemantic,
    location: u32,
    format: &str,
    stride: u32,
) -> ShaderInput {
    ShaderInput {
        name: name.to_string(),
        semantic,
        location,
        format: format.to_string(),
        stride,
    }
}

/// Return the input name.
#[allow(dead_code)]
pub fn input_name(input: &ShaderInput) -> &str {
    &input.name
}

/// Return the semantic.
#[allow(dead_code)]
pub fn input_semantic(input: &ShaderInput) -> &InputSemantic {
    &input.semantic
}

/// Return the location.
#[allow(dead_code)]
pub fn input_location(input: &ShaderInput) -> u32 {
    input.location
}

/// Return the format string.
#[allow(dead_code)]
pub fn input_format(input: &ShaderInput) -> &str {
    &input.format
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn input_to_json(input: &ShaderInput) -> String {
    let sem_str = match &input.semantic {
        InputSemantic::Position => "position".to_string(),
        InputSemantic::Normal => "normal".to_string(),
        InputSemantic::TexCoord => "texcoord".to_string(),
        InputSemantic::Color => "color".to_string(),
        InputSemantic::Tangent => "tangent".to_string(),
        InputSemantic::Custom(s) => format!("custom:{s}"),
    };
    format!(
        "{{\"name\":\"{}\",\"semantic\":\"{}\",\"location\":{},\"format\":\"{}\",\"stride\":{}}}",
        input.name, sem_str, input.location, input.format, input.stride
    )
}

/// Return the stride.
#[allow(dead_code)]
pub fn input_stride(input: &ShaderInput) -> u32 {
    input.stride
}

/// Return a count of components based on format (heuristic: "float3" -> 3, "float4" -> 4, etc.).
#[allow(dead_code)]
pub fn input_count_si(input: &ShaderInput) -> u32 {
    if input.format.contains('4') {
        4
    } else if input.format.contains('3') {
        3
    } else if input.format.contains('2') {
        2
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_input() {
        let i = new_shader_input("pos", InputSemantic::Position, 0, "float3", 12);
        assert_eq!(input_name(&i), "pos");
    }

    #[test]
    fn test_semantic() {
        let i = new_shader_input("norm", InputSemantic::Normal, 1, "float3", 12);
        assert_eq!(*input_semantic(&i), InputSemantic::Normal);
    }

    #[test]
    fn test_location() {
        let i = new_shader_input("uv", InputSemantic::TexCoord, 2, "float2", 8);
        assert_eq!(input_location(&i), 2);
    }

    #[test]
    fn test_format() {
        let i = new_shader_input("pos", InputSemantic::Position, 0, "float3", 12);
        assert_eq!(input_format(&i), "float3");
    }

    #[test]
    fn test_stride() {
        let i = new_shader_input("pos", InputSemantic::Position, 0, "float3", 12);
        assert_eq!(input_stride(&i), 12);
    }

    #[test]
    fn test_to_json() {
        let i = new_shader_input("pos", InputSemantic::Position, 0, "float3", 12);
        let json = input_to_json(&i);
        assert!(json.contains("\"semantic\":\"position\""));
    }

    #[test]
    fn test_count_float3() {
        let i = new_shader_input("pos", InputSemantic::Position, 0, "float3", 12);
        assert_eq!(input_count_si(&i), 3);
    }

    #[test]
    fn test_count_float4() {
        let i = new_shader_input("col", InputSemantic::Color, 0, "float4", 16);
        assert_eq!(input_count_si(&i), 4);
    }

    #[test]
    fn test_count_float2() {
        let i = new_shader_input("uv", InputSemantic::TexCoord, 0, "float2", 8);
        assert_eq!(input_count_si(&i), 2);
    }

    #[test]
    fn test_custom_semantic() {
        let i = new_shader_input("bone", InputSemantic::Custom("bone_weights".to_string()), 5, "float4", 16);
        let json = input_to_json(&i);
        assert!(json.contains("custom:bone_weights"));
    }
}
