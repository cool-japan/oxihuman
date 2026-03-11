#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShaderBinding {
    pub name: String,
    pub group: u32,
    pub binding: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShaderReflection {
    bindings: Vec<ShaderBinding>,
}

#[allow(dead_code)]
pub fn new_shader_reflection() -> ShaderReflection {
    ShaderReflection { bindings: Vec::new() }
}

#[allow(dead_code)]
pub fn add_binding(r: &mut ShaderReflection, name: &str, group: u32, binding: u32) {
    r.bindings.push(ShaderBinding { name: name.to_string(), group, binding });
}

#[allow(dead_code)]
pub fn binding_count_sr(r: &ShaderReflection) -> usize { r.bindings.len() }

#[allow(dead_code)]
pub fn binding_at(r: &ShaderReflection, idx: usize) -> Option<&ShaderBinding> {
    r.bindings.get(idx)
}

#[allow(dead_code)]
pub fn binding_name_sr(r: &ShaderReflection, idx: usize) -> &str {
    if idx < r.bindings.len() { &r.bindings[idx].name } else { "" }
}

#[allow(dead_code)]
pub fn reflection_to_json(r: &ShaderReflection) -> String {
    format!("{{\"bindings\":{}}}", r.bindings.len())
}

#[allow(dead_code)]
pub fn clear_reflection(r: &mut ShaderReflection) { r.bindings.clear(); }

#[allow(dead_code)]
pub fn validate_reflection(r: &ShaderReflection) -> bool {
    for (i, a) in r.bindings.iter().enumerate() {
        for b in r.bindings.iter().skip(i + 1) {
            if a.group == b.group && a.binding == b.binding { return false; }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let r = new_shader_reflection(); assert_eq!(binding_count_sr(&r), 0); }
    #[test] fn test_add() { let mut r = new_shader_reflection(); add_binding(&mut r, "ubo", 0, 0); assert_eq!(binding_count_sr(&r), 1); }
    #[test] fn test_at() { let mut r = new_shader_reflection(); add_binding(&mut r, "tex", 0, 1); assert_eq!(binding_at(&r, 0).unwrap().name, "tex"); }
    #[test] fn test_at_none() { let r = new_shader_reflection(); assert!(binding_at(&r, 0).is_none()); }
    #[test] fn test_name() { let mut r = new_shader_reflection(); add_binding(&mut r, "x", 0, 0); assert_eq!(binding_name_sr(&r, 0), "x"); }
    #[test] fn test_name_oob() { let r = new_shader_reflection(); assert_eq!(binding_name_sr(&r, 0), ""); }
    #[test] fn test_json() { let r = new_shader_reflection(); assert!(reflection_to_json(&r).contains("bindings")); }
    #[test] fn test_clear() { let mut r = new_shader_reflection(); add_binding(&mut r, "x", 0, 0); clear_reflection(&mut r); assert_eq!(binding_count_sr(&r), 0); }
    #[test] fn test_validate_ok() { let mut r = new_shader_reflection(); add_binding(&mut r, "a", 0, 0); add_binding(&mut r, "b", 0, 1); assert!(validate_reflection(&r)); }
    #[test] fn test_validate_dup() { let mut r = new_shader_reflection(); add_binding(&mut r, "a", 0, 0); add_binding(&mut r, "b", 0, 0); assert!(!validate_reflection(&r)); }
}
