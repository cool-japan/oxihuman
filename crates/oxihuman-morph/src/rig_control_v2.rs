#![allow(dead_code)]

//! Rig control parameter set with range clamping.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigParamV2 {
    pub name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigControlV2 {
    pub params: Vec<RigParamV2>,
}

#[allow(dead_code)]
pub fn new_rig_control_v2() -> RigControlV2 {
    RigControlV2 { params: Vec::new() }
}

#[allow(dead_code)]
pub fn rcv2_add_param(rig: &mut RigControlV2, name: &str, default: f32, min: f32, max: f32) {
    rig.params.push(RigParamV2 {
        name: name.to_string(),
        value: default.clamp(min, max),
        min,
        max,
        default,
    });
}

#[allow(dead_code)]
pub fn rcv2_set(rig: &mut RigControlV2, name: &str, value: f32) {
    if let Some(p) = rig.params.iter_mut().find(|p| p.name == name) {
        p.value = value.clamp(p.min, p.max);
    }
}

#[allow(dead_code)]
pub fn rcv2_get(rig: &RigControlV2, name: &str) -> Option<f32> {
    rig.params.iter().find(|p| p.name == name).map(|p| p.value)
}

#[allow(dead_code)]
pub fn rcv2_reset_all(rig: &mut RigControlV2) {
    for p in &mut rig.params {
        p.value = p.default.clamp(p.min, p.max);
    }
}

#[allow(dead_code)]
pub fn rcv2_reset_param(rig: &mut RigControlV2, name: &str) {
    if let Some(p) = rig.params.iter_mut().find(|p| p.name == name) {
        p.value = p.default.clamp(p.min, p.max);
    }
}

#[allow(dead_code)]
pub fn rcv2_param_count(rig: &RigControlV2) -> usize {
    rig.params.len()
}

#[allow(dead_code)]
pub fn rcv2_normalized(rig: &RigControlV2, name: &str) -> f32 {
    if let Some(p) = rig.params.iter().find(|p| p.name == name) {
        let range = p.max - p.min;
        if range.abs() < 1e-9 {
            return 0.0;
        }
        (p.value - p.min) / range
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn rcv2_to_json(rig: &RigControlV2) -> String {
    format!("{{\"param_count\":{}}}", rig.params.len())
}

#[allow(dead_code)]
pub fn rcv2_remove_param(rig: &mut RigControlV2, name: &str) {
    rig.params.retain(|p| p.name != name);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_rig() {
        let r = new_rig_control_v2();
        assert_eq!(rcv2_param_count(&r), 0);
    }

    #[test]
    fn test_add_param() {
        let mut r = new_rig_control_v2();
        rcv2_add_param(&mut r, "jaw", 0.0, 0.0, 1.0);
        assert_eq!(rcv2_param_count(&r), 1);
    }

    #[test]
    fn test_get_default() {
        let mut r = new_rig_control_v2();
        rcv2_add_param(&mut r, "jaw", 0.5, 0.0, 1.0);
        assert!((rcv2_get(&r, "jaw").unwrap() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_clamps() {
        let mut r = new_rig_control_v2();
        rcv2_add_param(&mut r, "jaw", 0.0, 0.0, 1.0);
        rcv2_set(&mut r, "jaw", 2.0);
        assert!((rcv2_get(&r, "jaw").unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset_all() {
        let mut r = new_rig_control_v2();
        rcv2_add_param(&mut r, "blink", 0.0, 0.0, 1.0);
        rcv2_set(&mut r, "blink", 0.8);
        rcv2_reset_all(&mut r);
        assert!((rcv2_get(&r, "blink").unwrap() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset_param() {
        let mut r = new_rig_control_v2();
        rcv2_add_param(&mut r, "blink", 0.0, 0.0, 1.0);
        rcv2_set(&mut r, "blink", 0.8);
        rcv2_reset_param(&mut r, "blink");
        assert!((rcv2_get(&r, "blink").unwrap() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalized() {
        let mut r = new_rig_control_v2();
        rcv2_add_param(&mut r, "x", 0.5, 0.0, 1.0);
        assert!((rcv2_normalized(&r, "x") - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_get_nonexistent() {
        let r = new_rig_control_v2();
        assert!(rcv2_get(&r, "missing").is_none());
    }

    #[test]
    fn test_remove_param() {
        let mut r = new_rig_control_v2();
        rcv2_add_param(&mut r, "jaw", 0.0, 0.0, 1.0);
        rcv2_remove_param(&mut r, "jaw");
        assert_eq!(rcv2_param_count(&r), 0);
    }

    #[test]
    fn test_to_json() {
        let r = new_rig_control_v2();
        let json = rcv2_to_json(&r);
        assert!(json.contains("param_count"));
    }

    #[test]
    fn test_normalized_zero_range() {
        let mut r = new_rig_control_v2();
        rcv2_add_param(&mut r, "flat", 0.5, 0.5, 0.5);
        assert!((rcv2_normalized(&r, "flat") - 0.0).abs() < 1e-6);
    }
}
