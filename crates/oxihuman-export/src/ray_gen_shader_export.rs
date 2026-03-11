// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ray generation shader stub export.

/// Ray shader type.
#[derive(Clone, Copy, PartialEq)]
pub enum RayShaderType {
    RayGen,
    Miss,
    ClosestHit,
    AnyHit,
    Intersection,
    Callable,
}

impl RayShaderType {
    pub fn name(&self) -> &'static str {
        match self {
            RayShaderType::RayGen => "raygen",
            RayShaderType::Miss => "miss",
            RayShaderType::ClosestHit => "closesthit",
            RayShaderType::AnyHit => "anyhit",
            RayShaderType::Intersection => "intersection",
            RayShaderType::Callable => "callable",
        }
    }
}

/// A ray tracing shader.
pub struct RayShader {
    pub shader_type: RayShaderType,
    pub entry_point: String,
    pub source: String,
}

/// A ray tracing pipeline export.
pub struct RayGenShaderExport {
    pub shaders: Vec<RayShader>,
    pub max_recursion_depth: u32,
    pub miss_shader_index: u32,
}

/// Create a new ray generation shader export.
pub fn new_ray_gen_shader_export(max_recursion: u32) -> RayGenShaderExport {
    RayGenShaderExport {
        shaders: Vec::new(),
        max_recursion_depth: max_recursion,
        miss_shader_index: 0,
    }
}

/// Add a ray shader.
pub fn add_ray_shader(
    exp: &mut RayGenShaderExport,
    shader_type: RayShaderType,
    entry: &str,
    src: &str,
) {
    exp.shaders.push(RayShader {
        shader_type,
        entry_point: entry.to_string(),
        source: src.to_string(),
    });
}

/// Shader count.
pub fn ray_shader_count(exp: &RayGenShaderExport) -> usize {
    exp.shaders.len()
}

/// Find a shader by type.
pub fn find_ray_shader(exp: &RayGenShaderExport, shader_type: RayShaderType) -> Option<&RayShader> {
    exp.shaders.iter().find(|s| s.shader_type == shader_type)
}

/// Validate (has at least a raygen and a miss shader).
pub fn validate_ray_gen_export(exp: &RayGenShaderExport) -> bool {
    find_ray_shader(exp, RayShaderType::RayGen).is_some()
        && find_ray_shader(exp, RayShaderType::Miss).is_some()
        && exp.max_recursion_depth > 0
}

/// Render a summary.
pub fn render_ray_gen_summary(exp: &RayGenShaderExport) -> String {
    format!(
        "Shaders:{} MaxRecursion:{} MissIdx:{}",
        exp.shaders.len(),
        exp.max_recursion_depth,
        exp.miss_shader_index
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_ray_gen_shader_export(4);
        assert_eq!(ray_shader_count(&exp), 0 /* empty */);
    }

    #[test]
    fn add_shader_increments() {
        let mut exp = new_ray_gen_shader_export(4);
        add_ray_shader(
            &mut exp,
            RayShaderType::RayGen,
            "rgen",
            "[shader(\"raygeneration\")] void rgen(){}",
        );
        assert_eq!(ray_shader_count(&exp), 1 /* one shader */);
    }

    #[test]
    fn shader_type_name_correct() {
        assert_eq!(
            RayShaderType::ClosestHit.name(),
            "closesthit" /* name */
        );
    }

    #[test]
    fn find_shader_by_type() {
        let mut exp = new_ray_gen_shader_export(4);
        add_ray_shader(&mut exp, RayShaderType::Miss, "miss_main", "");
        assert!(find_ray_shader(&exp, RayShaderType::Miss).is_some() /* found */);
    }

    #[test]
    fn find_missing_type_none() {
        let exp = new_ray_gen_shader_export(4);
        assert!(find_ray_shader(&exp, RayShaderType::Callable).is_none() /* not found */);
    }

    #[test]
    fn validate_requires_rgen_and_miss() {
        let mut exp = new_ray_gen_shader_export(4);
        add_ray_shader(&mut exp, RayShaderType::RayGen, "rgen", "");
        assert!(!validate_ray_gen_export(&exp) /* missing miss */);
        add_ray_shader(&mut exp, RayShaderType::Miss, "miss", "");
        assert!(validate_ray_gen_export(&exp) /* now valid */);
    }

    #[test]
    fn validate_zero_recursion_fails() {
        let mut exp = new_ray_gen_shader_export(0);
        add_ray_shader(&mut exp, RayShaderType::RayGen, "rgen", "");
        add_ray_shader(&mut exp, RayShaderType::Miss, "miss", "");
        assert!(!validate_ray_gen_export(&exp) /* zero recursion */);
    }

    #[test]
    fn render_summary_contains_recursion() {
        let exp = new_ray_gen_shader_export(8);
        let s = render_ray_gen_summary(&exp);
        assert!(s.contains("8") /* recursion depth */);
    }

    #[test]
    fn all_shader_types_have_names() {
        let types = [
            RayShaderType::RayGen,
            RayShaderType::Miss,
            RayShaderType::ClosestHit,
            RayShaderType::AnyHit,
            RayShaderType::Intersection,
            RayShaderType::Callable,
        ];
        for t in &types {
            assert!(!t.name().is_empty() /* non-empty name */);
        }
    }
}
