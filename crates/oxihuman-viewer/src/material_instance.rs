#![allow(dead_code)]

//! Material instance with per-object overrides.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ParamValue {
    Float(f32),
    Vec4([f32; 4]),
    Int(i32),
    Bool(bool),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialInstance {
    pub base_material: String,
    pub overrides: HashMap<String, ParamValue>,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn new_material_instance(base_material: &str) -> MaterialInstance {
    MaterialInstance {
        base_material: base_material.to_string(),
        overrides: HashMap::new(),
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn mi_set_float(inst: &mut MaterialInstance, name: &str, value: f32) {
    inst.overrides.insert(name.to_string(), ParamValue::Float(value));
}

#[allow(dead_code)]
pub fn mi_set_vec4(inst: &mut MaterialInstance, name: &str, value: [f32; 4]) {
    inst.overrides.insert(name.to_string(), ParamValue::Vec4(value));
}

#[allow(dead_code)]
pub fn mi_set_int(inst: &mut MaterialInstance, name: &str, value: i32) {
    inst.overrides.insert(name.to_string(), ParamValue::Int(value));
}

#[allow(dead_code)]
pub fn mi_set_bool(inst: &mut MaterialInstance, name: &str, value: bool) {
    inst.overrides.insert(name.to_string(), ParamValue::Bool(value));
}

#[allow(dead_code)]
pub fn mi_get_float(inst: &MaterialInstance, name: &str) -> Option<f32> {
    inst.overrides.get(name).and_then(|v| match v {
        ParamValue::Float(f) => Some(*f),
        _ => None,
    })
}

#[allow(dead_code)]
pub fn mi_remove_override(inst: &mut MaterialInstance, name: &str) {
    inst.overrides.remove(name);
}

#[allow(dead_code)]
pub fn mi_override_count(inst: &MaterialInstance) -> usize {
    inst.overrides.len()
}

#[allow(dead_code)]
pub fn mi_clear_overrides(inst: &mut MaterialInstance) {
    inst.overrides.clear();
}

#[allow(dead_code)]
pub fn mi_has_override(inst: &MaterialInstance, name: &str) -> bool {
    inst.overrides.contains_key(name)
}

#[allow(dead_code)]
pub fn mi_set_enabled(inst: &mut MaterialInstance, enabled: bool) {
    inst.enabled = enabled;
}

#[allow(dead_code)]
pub fn mi_to_json(inst: &MaterialInstance) -> String {
    format!(
        "{{\"base\":\"{}\",\"override_count\":{},\"enabled\":{}}}",
        inst.base_material,
        inst.overrides.len(),
        inst.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_instance() {
        let inst = new_material_instance("PBR");
        assert_eq!(inst.base_material, "PBR");
        assert_eq!(mi_override_count(&inst), 0);
    }

    #[test]
    fn test_set_float() {
        let mut inst = new_material_instance("PBR");
        mi_set_float(&mut inst, "roughness", 0.5);
        assert!((mi_get_float(&inst, "roughness").unwrap() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_vec4() {
        let mut inst = new_material_instance("PBR");
        mi_set_vec4(&mut inst, "albedo", [1.0, 0.0, 0.0, 1.0]);
        assert!(mi_has_override(&inst, "albedo"));
    }

    #[test]
    fn test_set_bool() {
        let mut inst = new_material_instance("PBR");
        mi_set_bool(&mut inst, "use_normal_map", true);
        assert!(mi_has_override(&inst, "use_normal_map"));
    }

    #[test]
    fn test_remove_override() {
        let mut inst = new_material_instance("PBR");
        mi_set_float(&mut inst, "metallic", 0.0);
        mi_remove_override(&mut inst, "metallic");
        assert!(!mi_has_override(&inst, "metallic"));
    }

    #[test]
    fn test_clear_overrides() {
        let mut inst = new_material_instance("PBR");
        mi_set_float(&mut inst, "a", 1.0);
        mi_set_float(&mut inst, "b", 2.0);
        mi_clear_overrides(&mut inst);
        assert_eq!(mi_override_count(&inst), 0);
    }

    #[test]
    fn test_enabled() {
        let mut inst = new_material_instance("PBR");
        mi_set_enabled(&mut inst, false);
        assert!(!inst.enabled);
    }

    #[test]
    fn test_get_float_wrong_type() {
        let mut inst = new_material_instance("PBR");
        mi_set_bool(&mut inst, "flag", true);
        assert!(mi_get_float(&inst, "flag").is_none());
    }

    #[test]
    fn test_to_json() {
        let inst = new_material_instance("Toon");
        let json = mi_to_json(&inst);
        assert!(json.contains("\"base\":\"Toon\""));
    }

    #[test]
    fn test_set_int() {
        let mut inst = new_material_instance("PBR");
        mi_set_int(&mut inst, "uv_set", 1);
        assert!(mi_has_override(&inst, "uv_set"));
    }
}
