// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! WGSL (WebGPU Shading Language) shader export stub.

/// A WGSL entry point stage.
#[derive(Clone, Copy, PartialEq)]
pub enum WgslStage {
    Vertex,
    Fragment,
    Compute,
}

impl WgslStage {
    pub fn attribute(&self) -> &'static str {
        match self {
            WgslStage::Vertex => "@vertex",
            WgslStage::Fragment => "@fragment",
            WgslStage::Compute => "@compute",
        }
    }
}

/// A WGSL entry point.
pub struct WgslEntryPoint {
    pub stage: WgslStage,
    pub name: String,
    pub body: String,
}

/// A WGSL shader module export.
pub struct WgslExport {
    pub entry_points: Vec<WgslEntryPoint>,
    pub structs: Vec<String>,
    pub global_vars: Vec<String>,
}

/// Create a new WGSL export.
pub fn new_wgsl_export() -> WgslExport {
    WgslExport {
        entry_points: Vec::new(),
        structs: Vec::new(),
        global_vars: Vec::new(),
    }
}

/// Add an entry point.
pub fn add_wgsl_entry_point(exp: &mut WgslExport, stage: WgslStage, name: &str, body: &str) {
    exp.entry_points.push(WgslEntryPoint {
        stage,
        name: name.to_string(),
        body: body.to_string(),
    });
}

/// Add a struct declaration.
pub fn add_wgsl_struct(exp: &mut WgslExport, decl: &str) {
    exp.structs.push(decl.to_string());
}

/// Add a global variable declaration.
pub fn add_wgsl_global(exp: &mut WgslExport, decl: &str) {
    exp.global_vars.push(decl.to_string());
}

/// Entry point count.
pub fn wgsl_entry_point_count(exp: &WgslExport) -> usize {
    exp.entry_points.len()
}

/// Find an entry point by stage.
pub fn find_wgsl_entry(exp: &WgslExport, stage: WgslStage) -> Option<&WgslEntryPoint> {
    exp.entry_points.iter().find(|e| e.stage == stage)
}

/// Render WGSL source.
pub fn render_wgsl_source(exp: &WgslExport) -> String {
    let mut s = String::new();
    for st in &exp.structs {
        s.push_str(st);
        s.push('\n');
    }
    for g in &exp.global_vars {
        s.push_str(g);
        s.push('\n');
    }
    for ep in &exp.entry_points {
        s.push_str(&format!(
            "{} fn {}() {{\n{}\n}}\n",
            ep.stage.attribute(),
            ep.name,
            ep.body
        ));
    }
    s
}

/// Validate (at least one entry point).
pub fn validate_wgsl_export(exp: &WgslExport) -> bool {
    !exp.entry_points.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_wgsl_export();
        assert_eq!(wgsl_entry_point_count(&exp), 0 /* empty */);
    }

    #[test]
    fn add_entry_point_increments() {
        let mut exp = new_wgsl_export();
        add_wgsl_entry_point(&mut exp, WgslStage::Vertex, "vs_main", "");
        assert_eq!(wgsl_entry_point_count(&exp), 1 /* one entry point */);
    }

    #[test]
    fn stage_attribute_correct() {
        assert_eq!(
            WgslStage::Compute.attribute(),
            "@compute" /* compute */
        );
    }

    #[test]
    fn find_entry_by_stage() {
        let mut exp = new_wgsl_export();
        add_wgsl_entry_point(&mut exp, WgslStage::Fragment, "fs_main", "");
        assert!(find_wgsl_entry(&exp, WgslStage::Fragment).is_some() /* found */);
    }

    #[test]
    fn find_missing_stage_none() {
        let exp = new_wgsl_export();
        assert!(find_wgsl_entry(&exp, WgslStage::Compute).is_none() /* not found */);
    }

    #[test]
    fn render_contains_entry_name() {
        let mut exp = new_wgsl_export();
        add_wgsl_entry_point(&mut exp, WgslStage::Vertex, "my_vertex", "");
        let src = render_wgsl_source(&exp);
        assert!(src.contains("my_vertex") /* entry name */);
    }

    #[test]
    fn render_contains_struct() {
        let mut exp = new_wgsl_export();
        add_wgsl_struct(&mut exp, "struct VertexOut { pos: vec4<f32> }");
        let src = render_wgsl_source(&exp);
        assert!(src.contains("VertexOut") /* struct */);
    }

    #[test]
    fn validate_empty_fails() {
        let exp = new_wgsl_export();
        assert!(!validate_wgsl_export(&exp) /* empty */);
    }

    #[test]
    fn validate_with_entry_passes() {
        let mut exp = new_wgsl_export();
        add_wgsl_entry_point(&mut exp, WgslStage::Compute, "cs_main", "");
        assert!(validate_wgsl_export(&exp) /* valid */);
    }
}
