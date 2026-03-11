#![allow(dead_code)]
//! Export shader parameter data.

/// Shader parameter export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderParamExport {
    pub params: Vec<ShaderParam>,
}

/// A single shader parameter.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderParam {
    pub name: String,
    pub value_type: String,
    pub default_value: String,
}

/// Export shader parameters.
#[allow(dead_code)]
pub fn export_shader_params(params: &[ShaderParam]) -> ShaderParamExport {
    ShaderParamExport { params: params.to_vec() }
}

/// Get the parameter count.
#[allow(dead_code)]
pub fn param_count_export(export: &ShaderParamExport) -> usize {
    export.params.len()
}

/// Get the name at an index.
#[allow(dead_code)]
pub fn param_name_at(export: &ShaderParamExport, index: usize) -> Option<&str> {
    export.params.get(index).map(|p| p.name.as_str())
}

/// Get the value type at an index.
#[allow(dead_code)]
pub fn param_value_type(export: &ShaderParamExport, index: usize) -> Option<&str> {
    export.params.get(index).map(|p| p.value_type.as_str())
}

/// Convert params to JSON.
#[allow(dead_code)]
pub fn param_to_json(export: &ShaderParamExport) -> String {
    let params_str: Vec<String> = export.params.iter().map(|p| {
        format!(
            "{{\"name\":\"{}\",\"type\":\"{}\",\"default\":\"{}\"}}",
            p.name, p.value_type, p.default_value
        )
    }).collect();
    format!("{{\"param_count\":{},\"params\":[{}]}}", export.params.len(), params_str.join(","))
}

/// Get the default value at an index.
#[allow(dead_code)]
pub fn param_default_value(export: &ShaderParamExport, index: usize) -> Option<&str> {
    export.params.get(index).map(|p| p.default_value.as_str())
}

/// Estimated export size.
#[allow(dead_code)]
pub fn param_export_size(export: &ShaderParamExport) -> usize {
    export.params.iter().map(|p| p.name.len() + p.value_type.len() + p.default_value.len() + 16).sum()
}

/// Validate shader parameters (names must be non-empty).
#[allow(dead_code)]
pub fn validate_shader_params(export: &ShaderParamExport) -> bool {
    export.params.iter().all(|p| !p.name.is_empty() && !p.value_type.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ShaderParamExport {
        export_shader_params(&[
            ShaderParam { name: "albedo".to_string(), value_type: "float4".to_string(), default_value: "1,1,1,1".to_string() },
            ShaderParam { name: "roughness".to_string(), value_type: "float".to_string(), default_value: "0.5".to_string() },
        ])
    }

    #[test]
    fn test_export_shader_params() {
        let sp = sample();
        assert_eq!(param_count_export(&sp), 2);
    }

    #[test]
    fn test_param_name_at() {
        let sp = sample();
        assert_eq!(param_name_at(&sp, 0), Some("albedo"));
    }

    #[test]
    fn test_param_name_at_oob() {
        let sp = sample();
        assert_eq!(param_name_at(&sp, 10), None);
    }

    #[test]
    fn test_param_value_type() {
        let sp = sample();
        assert_eq!(param_value_type(&sp, 1), Some("float"));
    }

    #[test]
    fn test_param_to_json() {
        let sp = sample();
        let j = param_to_json(&sp);
        assert!(j.contains("param_count"));
        assert!(j.contains("albedo"));
    }

    #[test]
    fn test_param_default_value() {
        let sp = sample();
        assert_eq!(param_default_value(&sp, 0), Some("1,1,1,1"));
    }

    #[test]
    fn test_param_export_size() {
        let sp = sample();
        assert!(param_export_size(&sp) > 0);
    }

    #[test]
    fn test_validate_shader_params() {
        let sp = sample();
        assert!(validate_shader_params(&sp));
    }

    #[test]
    fn test_validate_shader_params_bad() {
        let sp = export_shader_params(&[
            ShaderParam { name: "".to_string(), value_type: "float".to_string(), default_value: "0".to_string() },
        ]);
        assert!(!validate_shader_params(&sp));
    }

    #[test]
    fn test_empty_params() {
        let sp = export_shader_params(&[]);
        assert_eq!(param_count_export(&sp), 0);
        assert!(validate_shader_params(&sp));
    }
}
