#![allow(dead_code)]

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShaderPreprocessor {
    macros: HashMap<String, String>,
}

#[allow(dead_code)]
pub fn new_shader_preprocessor() -> ShaderPreprocessor {
    ShaderPreprocessor { macros: HashMap::new() }
}

#[allow(dead_code)]
pub fn add_macro(pp: &mut ShaderPreprocessor, name: &str, value: &str) {
    pp.macros.insert(name.to_string(), value.to_string());
}

#[allow(dead_code)]
pub fn expand_macros(pp: &ShaderPreprocessor, source: &str) -> String {
    let mut result = source.to_string();
    for (k, v) in &pp.macros {
        result = result.replace(k.as_str(), v.as_str());
    }
    result
}

#[allow(dead_code)]
pub fn macro_count(pp: &ShaderPreprocessor) -> usize { pp.macros.len() }

#[allow(dead_code)]
pub fn preprocessor_to_json(pp: &ShaderPreprocessor) -> String {
    format!("{{\"macro_count\":{}}}", pp.macros.len())
}

#[allow(dead_code)]
pub fn remove_macro(pp: &mut ShaderPreprocessor, name: &str) { pp.macros.remove(name); }

#[allow(dead_code)]
pub fn clear_macros(pp: &mut ShaderPreprocessor) { pp.macros.clear(); }

#[allow(dead_code)]
pub fn has_macro(pp: &ShaderPreprocessor, name: &str) -> bool { pp.macros.contains_key(name) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let p = new_shader_preprocessor(); assert_eq!(macro_count(&p), 0); }
    #[test] fn test_add() { let mut p = new_shader_preprocessor(); add_macro(&mut p, "MAX", "16"); assert_eq!(macro_count(&p), 1); }
    #[test] fn test_expand() { let mut p = new_shader_preprocessor(); add_macro(&mut p, "MAX", "16"); assert_eq!(expand_macros(&p, "limit=MAX"), "limit=16"); }
    #[test] fn test_has() { let mut p = new_shader_preprocessor(); add_macro(&mut p, "X", "1"); assert!(has_macro(&p, "X")); }
    #[test] fn test_not_has() { let p = new_shader_preprocessor(); assert!(!has_macro(&p, "X")); }
    #[test] fn test_remove() { let mut p = new_shader_preprocessor(); add_macro(&mut p, "X", "1"); remove_macro(&mut p, "X"); assert!(!has_macro(&p, "X")); }
    #[test] fn test_clear() { let mut p = new_shader_preprocessor(); add_macro(&mut p, "X", "1"); clear_macros(&mut p); assert_eq!(macro_count(&p), 0); }
    #[test] fn test_json() { let p = new_shader_preprocessor(); assert!(preprocessor_to_json(&p).contains("macro_count")); }
    #[test] fn test_no_expand() { let p = new_shader_preprocessor(); assert_eq!(expand_macros(&p, "hello"), "hello"); }
    #[test] fn test_multiple_macros() { let mut p = new_shader_preprocessor(); add_macro(&mut p, "A", "1"); add_macro(&mut p, "B", "2"); assert_eq!(macro_count(&p), 2); }
}
