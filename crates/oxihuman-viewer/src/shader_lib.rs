#![allow(dead_code)]

use std::collections::HashMap;

/// An entry in the shader library.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShaderEntry {
    pub name: String,
    pub source: String,
}

/// Library of named shaders.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShaderLib {
    shaders: HashMap<String, ShaderEntry>,
}

#[allow(dead_code)]
pub fn new_shader_lib() -> ShaderLib { ShaderLib { shaders: HashMap::new() } }

#[allow(dead_code)]
pub fn add_shader_sl(lib: &mut ShaderLib, name: &str, source: &str) {
    lib.shaders.insert(name.to_string(), ShaderEntry { name: name.to_string(), source: source.to_string() });
}

#[allow(dead_code)]
pub fn get_shader_sl<'a>(lib: &'a ShaderLib, name: &str) -> Option<&'a ShaderEntry> { lib.shaders.get(name) }

#[allow(dead_code)]
pub fn shader_count_sl(lib: &ShaderLib) -> usize { lib.shaders.len() }

#[allow(dead_code)]
pub fn shader_names_sl(lib: &ShaderLib) -> Vec<String> {
    let mut n: Vec<String> = lib.shaders.keys().cloned().collect();
    n.sort();
    n
}

#[allow(dead_code)]
pub fn shader_lib_to_json(lib: &ShaderLib) -> String {
    let names = shader_names_sl(lib);
    let e: Vec<String> = names.iter().map(|n| format!("\"{}\":true", n)).collect();
    format!("{{\"count\":{},{}}}", lib.shaders.len(), e.join(","))
}

#[allow(dead_code)]
pub fn remove_shader_sl(lib: &mut ShaderLib, name: &str) -> bool { lib.shaders.remove(name).is_some() }

#[allow(dead_code)]
pub fn shader_lib_clear(lib: &mut ShaderLib) { lib.shaders.clear(); }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert_eq!(shader_count_sl(&new_shader_lib()), 0); }
    #[test] fn test_add_get() {
        let mut l = new_shader_lib();
        add_shader_sl(&mut l, "vert", "void main(){}");
        assert!(get_shader_sl(&l, "vert").is_some());
    }
    #[test] fn test_get_missing() { assert!(get_shader_sl(&new_shader_lib(), "x").is_none()); }
    #[test] fn test_count() {
        let mut l = new_shader_lib();
        add_shader_sl(&mut l, "a", ""); add_shader_sl(&mut l, "b", "");
        assert_eq!(shader_count_sl(&l), 2);
    }
    #[test] fn test_names() {
        let mut l = new_shader_lib();
        add_shader_sl(&mut l, "b", ""); add_shader_sl(&mut l, "a", "");
        assert_eq!(shader_names_sl(&l)[0], "a");
    }
    #[test] fn test_remove() {
        let mut l = new_shader_lib();
        add_shader_sl(&mut l, "x", "");
        assert!(remove_shader_sl(&mut l, "x"));
        assert!(!remove_shader_sl(&mut l, "x"));
    }
    #[test] fn test_clear() {
        let mut l = new_shader_lib();
        add_shader_sl(&mut l, "x", "");
        shader_lib_clear(&mut l);
        assert_eq!(shader_count_sl(&l), 0);
    }
    #[test] fn test_to_json() {
        let mut l = new_shader_lib();
        add_shader_sl(&mut l, "s", "");
        assert!(shader_lib_to_json(&l).contains("count"));
    }
    #[test] fn test_overwrite() {
        let mut l = new_shader_lib();
        add_shader_sl(&mut l, "x", "old");
        add_shader_sl(&mut l, "x", "new");
        assert_eq!(get_shader_sl(&l, "x").unwrap().source, "new");
    }
    #[test] fn test_empty_names() { assert!(shader_names_sl(&new_shader_lib()).is_empty()); }
}
