// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Metal MSL shader export stub.

/// MSL function type.
#[derive(Clone, Copy, PartialEq)]
pub enum MslFunctionType {
    Vertex,
    Fragment,
    Kernel,
}

impl MslFunctionType {
    pub fn keyword(&self) -> &'static str {
        match self {
            MslFunctionType::Vertex => "vertex",
            MslFunctionType::Fragment => "fragment",
            MslFunctionType::Kernel => "kernel",
        }
    }
}

/// An MSL shader function.
pub struct MslFunction {
    pub function_type: MslFunctionType,
    pub name: String,
    pub source: String,
}

/// An MSL export document.
pub struct MslExport {
    pub functions: Vec<MslFunction>,
    pub includes: Vec<String>,
}

/// Create a new MSL export.
pub fn new_msl_export() -> MslExport {
    MslExport {
        functions: Vec::new(),
        includes: vec!["<metal_stdlib>".to_string()],
    }
}

/// Add an MSL function.
pub fn add_msl_function(exp: &mut MslExport, fn_type: MslFunctionType, name: &str, src: &str) {
    exp.functions.push(MslFunction {
        function_type: fn_type,
        name: name.to_string(),
        source: src.to_string(),
    });
}

/// Add an include.
pub fn add_msl_include(exp: &mut MslExport, include: &str) {
    exp.includes.push(include.to_string());
}

/// Function count.
pub fn msl_function_count(exp: &MslExport) -> usize {
    exp.functions.len()
}

/// Find a function by name.
pub fn find_msl_function<'a>(exp: &'a MslExport, name: &str) -> Option<&'a MslFunction> {
    exp.functions.iter().find(|f| f.name == name)
}

/// Render full MSL source.
pub fn render_msl_source(exp: &MslExport) -> String {
    let mut s = String::new();
    for inc in &exp.includes {
        s.push_str(&format!("#include {inc}\n"));
    }
    s.push_str("using namespace metal;\n");
    for f in &exp.functions {
        s.push_str(&format!(
            "{} {} {{ {} }}\n",
            f.function_type.keyword(),
            f.name,
            f.source
        ));
    }
    s
}

/// Validate (at least one function).
pub fn validate_msl_export(exp: &MslExport) -> bool {
    !exp.functions.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_has_include() {
        let exp = new_msl_export();
        assert!(exp.includes.iter().any(|i| i.contains("metal_stdlib")) /* has metal include */);
    }

    #[test]
    fn add_function_increments() {
        let mut exp = new_msl_export();
        add_msl_function(
            &mut exp,
            MslFunctionType::Vertex,
            "vertex_main",
            "return pos;",
        );
        assert_eq!(msl_function_count(&exp), 1 /* one function */);
    }

    #[test]
    fn keyword_vertex_correct() {
        assert_eq!(
            MslFunctionType::Vertex.keyword(),
            "vertex" /* keyword */
        );
    }

    #[test]
    fn keyword_kernel_correct() {
        assert_eq!(
            MslFunctionType::Kernel.keyword(),
            "kernel" /* keyword */
        );
    }

    #[test]
    fn find_function_by_name() {
        let mut exp = new_msl_export();
        add_msl_function(
            &mut exp,
            MslFunctionType::Fragment,
            "frag_main",
            "return color;",
        );
        assert!(find_msl_function(&exp, "frag_main").is_some() /* found */);
    }

    #[test]
    fn find_missing_none() {
        let exp = new_msl_export();
        assert!(find_msl_function(&exp, "x").is_none() /* not found */);
    }

    #[test]
    fn render_contains_function_name() {
        let mut exp = new_msl_export();
        add_msl_function(&mut exp, MslFunctionType::Vertex, "vs_main", "");
        let src = render_msl_source(&exp);
        assert!(src.contains("vs_main") /* function name */);
    }

    #[test]
    fn render_contains_namespace() {
        let exp = new_msl_export();
        let src = render_msl_source(&exp);
        assert!(src.contains("namespace metal") /* namespace */);
    }

    #[test]
    fn validate_empty_fails() {
        let exp = new_msl_export();
        assert!(!validate_msl_export(&exp) /* no functions */);
    }

    #[test]
    fn validate_with_function_passes() {
        let mut exp = new_msl_export();
        add_msl_function(&mut exp, MslFunctionType::Kernel, "compute", "");
        assert!(validate_msl_export(&exp) /* valid */);
    }
}
