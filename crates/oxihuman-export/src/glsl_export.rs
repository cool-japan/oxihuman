// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! GLSL shader source export.

/// GLSL shader stage.
#[derive(Clone, Copy, PartialEq)]
pub enum GlslStage {
    Vertex,
    Fragment,
    Geometry,
    TessControl,
    TessEvaluation,
    Compute,
}

/// A GLSL shader source.
pub struct GlslShader {
    pub stage: GlslStage,
    pub source: String,
    pub version: u32,
    pub defines: Vec<String>,
}

/// A GLSL export containing multiple shader stages.
pub struct GlslExport {
    pub shaders: Vec<GlslShader>,
}

/// Create a new GLSL export.
pub fn new_glsl_export() -> GlslExport {
    GlslExport {
        shaders: Vec::new(),
    }
}

/// Add a shader stage.
pub fn add_glsl_shader(exp: &mut GlslExport, stage: GlslStage, source: &str, version: u32) {
    exp.shaders.push(GlslShader {
        stage,
        source: source.to_string(),
        version,
        defines: Vec::new(),
    });
}

/// Add a preprocessor define to the last added shader.
pub fn add_glsl_define(exp: &mut GlslExport, define: &str) -> bool {
    if let Some(shader) = exp.shaders.last_mut() {
        shader.defines.push(define.to_string());
        true
    } else {
        false
    }
}

/// Shader count.
pub fn glsl_shader_count(exp: &GlslExport) -> usize {
    exp.shaders.len()
}

/// Find a shader by stage.
pub fn find_glsl_shader(exp: &GlslExport, stage: GlslStage) -> Option<&GlslShader> {
    exp.shaders.iter().find(|s| s.stage == stage)
}

/// Render a shader to string with version and defines prepended.
pub fn render_glsl_shader(shader: &GlslShader) -> String {
    let mut s = format!("#version {}\n", shader.version);
    for d in &shader.defines {
        s.push_str(&format!("#define {d}\n"));
    }
    s.push_str(&shader.source);
    s
}

/// Validate (has at least vertex and fragment shader).
pub fn validate_glsl_export(exp: &GlslExport) -> bool {
    find_glsl_shader(exp, GlslStage::Vertex).is_some()
        && find_glsl_shader(exp, GlslStage::Fragment).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_glsl_export();
        assert_eq!(glsl_shader_count(&exp), 0 /* empty */);
    }

    #[test]
    fn add_shader_increments() {
        let mut exp = new_glsl_export();
        add_glsl_shader(&mut exp, GlslStage::Vertex, "void main(){}", 450);
        assert_eq!(glsl_shader_count(&exp), 1 /* one shader */);
    }

    #[test]
    fn find_shader_by_stage() {
        let mut exp = new_glsl_export();
        add_glsl_shader(&mut exp, GlslStage::Fragment, "void main(){}", 330);
        let s = find_glsl_shader(&exp, GlslStage::Fragment);
        assert!(s.is_some() /* found */);
    }

    #[test]
    fn find_missing_stage_none() {
        let exp = new_glsl_export();
        assert!(find_glsl_shader(&exp, GlslStage::Geometry).is_none() /* not found */);
    }

    #[test]
    fn add_define_works() {
        let mut exp = new_glsl_export();
        add_glsl_shader(&mut exp, GlslStage::Vertex, "void main(){}", 450);
        let ok = add_glsl_define(&mut exp, "USE_NORMALS 1");
        assert!(ok /* added */);
    }

    #[test]
    fn render_shader_contains_version() {
        let mut exp = new_glsl_export();
        add_glsl_shader(&mut exp, GlslStage::Vertex, "void main(){}", 450);
        let s = find_glsl_shader(&exp, GlslStage::Vertex).expect("should succeed");
        let src = render_glsl_shader(s);
        assert!(src.contains("450") /* version in output */);
    }

    #[test]
    fn render_shader_contains_define() {
        let mut exp = new_glsl_export();
        add_glsl_shader(&mut exp, GlslStage::Fragment, "void main(){}", 330);
        add_glsl_define(&mut exp, "USE_IBL");
        let s = find_glsl_shader(&exp, GlslStage::Fragment).expect("should succeed");
        let src = render_glsl_shader(s);
        assert!(src.contains("USE_IBL") /* define */);
    }

    #[test]
    fn validate_requires_both_stages() {
        let mut exp = new_glsl_export();
        add_glsl_shader(&mut exp, GlslStage::Vertex, "void main(){}", 450);
        assert!(!validate_glsl_export(&exp) /* missing fragment */);
        add_glsl_shader(&mut exp, GlslStage::Fragment, "void main(){}", 450);
        assert!(validate_glsl_export(&exp) /* now valid */);
    }

    #[test]
    fn add_define_without_shader_returns_false() {
        let mut exp = new_glsl_export();
        assert!(!add_glsl_define(&mut exp, "X") /* no shader */);
    }
}
