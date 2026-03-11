#![allow(dead_code)]

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceRigController {
    params: HashMap<String, f32>,
}

#[allow(dead_code)]
pub fn new_face_rig_controller() -> FaceRigController {
    FaceRigController { params: HashMap::new() }
}

#[allow(dead_code)]
pub fn set_rig_param(rig: &mut FaceRigController, name: &str, value: f32) {
    rig.params.insert(name.to_string(), value.clamp(-1.0, 1.0));
}

#[allow(dead_code)]
pub fn get_rig_param(rig: &FaceRigController, name: &str) -> f32 {
    rig.params.get(name).copied().unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn rig_param_count(rig: &FaceRigController) -> usize { rig.params.len() }

#[allow(dead_code)]
pub fn rig_evaluate(rig: &FaceRigController) -> Vec<(String, f32)> {
    rig.params.iter().map(|(k, &v)| (k.clone(), v)).collect()
}

#[allow(dead_code)]
pub fn rig_to_json(rig: &FaceRigController) -> String {
    format!("{{\"param_count\":{}}}", rig.params.len())
}

#[allow(dead_code)]
pub fn rig_reset(rig: &mut FaceRigController) { rig.params.clear(); }

#[allow(dead_code)]
pub fn rig_param_names(rig: &FaceRigController) -> Vec<String> {
    rig.params.keys().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let r = new_face_rig_controller(); assert_eq!(rig_param_count(&r), 0); }
    #[test] fn test_set_get() { let mut r = new_face_rig_controller(); set_rig_param(&mut r, "jaw", 0.5); assert!((get_rig_param(&r, "jaw") - 0.5).abs() < 1e-6); }
    #[test] fn test_clamp() { let mut r = new_face_rig_controller(); set_rig_param(&mut r, "x", 2.0); assert!((get_rig_param(&r, "x") - 1.0).abs() < 1e-6); }
    #[test] fn test_missing() { let r = new_face_rig_controller(); assert!((get_rig_param(&r, "x")).abs() < 1e-6); }
    #[test] fn test_count() { let mut r = new_face_rig_controller(); set_rig_param(&mut r, "a", 0.1); set_rig_param(&mut r, "b", 0.2); assert_eq!(rig_param_count(&r), 2); }
    #[test] fn test_evaluate() { let mut r = new_face_rig_controller(); set_rig_param(&mut r, "a", 0.3); let e = rig_evaluate(&r); assert_eq!(e.len(), 1); }
    #[test] fn test_json() { let r = new_face_rig_controller(); assert!(rig_to_json(&r).contains("param_count")); }
    #[test] fn test_reset() { let mut r = new_face_rig_controller(); set_rig_param(&mut r, "a", 0.5); rig_reset(&mut r); assert_eq!(rig_param_count(&r), 0); }
    #[test] fn test_names() { let mut r = new_face_rig_controller(); set_rig_param(&mut r, "x", 0.1); assert_eq!(rig_param_names(&r).len(), 1); }
    #[test] fn test_overwrite() { let mut r = new_face_rig_controller(); set_rig_param(&mut r, "a", 0.1); set_rig_param(&mut r, "a", 0.9); assert!((get_rig_param(&r, "a") - 0.9).abs() < 1e-6); }
}
