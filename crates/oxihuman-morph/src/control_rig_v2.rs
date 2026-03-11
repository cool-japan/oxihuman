#![allow(dead_code)]

//! Control rig with driven key relationships.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrivenKey {
    pub driver_name: String,
    pub driven_name: String,
    pub driver_value: f32,
    pub driven_value: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ControlRigV2 {
    pub name: String,
    pub driven_keys: Vec<DrivenKey>,
    pub control_values: std::collections::HashMap<String, f32>,
}

#[allow(dead_code)]
pub fn new_control_rig_v2(name: &str) -> ControlRigV2 {
    ControlRigV2 {
        name: name.to_string(),
        driven_keys: Vec::new(),
        control_values: std::collections::HashMap::new(),
    }
}

#[allow(dead_code)]
pub fn crv2_add_driven_key(
    rig: &mut ControlRigV2,
    driver: &str,
    driven: &str,
    driver_val: f32,
    driven_val: f32,
) {
    rig.driven_keys.push(DrivenKey {
        driver_name: driver.to_string(),
        driven_name: driven.to_string(),
        driver_value: driver_val,
        driven_value: driven_val,
    });
}

#[allow(dead_code)]
pub fn crv2_set_control(rig: &mut ControlRigV2, name: &str, value: f32) {
    rig.control_values.insert(name.to_string(), value);
}

#[allow(dead_code)]
pub fn crv2_evaluate(rig: &ControlRigV2) -> std::collections::HashMap<String, f32> {
    let mut result = std::collections::HashMap::new();
    for dk in &rig.driven_keys {
        if let Some(&driver_val) = rig.control_values.get(&dk.driver_name) {
            let t = if dk.driver_value.abs() > 1e-9 {
                (driver_val / dk.driver_value).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let driven = dk.driven_value * t;
            let entry = result.entry(dk.driven_name.clone()).or_insert(0.0);
            *entry += driven;
        }
    }
    result
}

#[allow(dead_code)]
pub fn crv2_driven_key_count(rig: &ControlRigV2) -> usize {
    rig.driven_keys.len()
}

#[allow(dead_code)]
pub fn crv2_clear_driven_keys(rig: &mut ControlRigV2) {
    rig.driven_keys.clear();
}

#[allow(dead_code)]
pub fn crv2_clear_controls(rig: &mut ControlRigV2) {
    rig.control_values.clear();
}

#[allow(dead_code)]
pub fn crv2_control_count(rig: &ControlRigV2) -> usize {
    rig.control_values.len()
}

#[allow(dead_code)]
pub fn crv2_remove_driven_keys_for(rig: &mut ControlRigV2, driver: &str) {
    rig.driven_keys.retain(|dk| dk.driver_name != driver);
}

#[allow(dead_code)]
pub fn crv2_to_json(rig: &ControlRigV2) -> String {
    format!(
        "{{\"name\":\"{}\",\"driven_key_count\":{},\"control_count\":{}}}",
        rig.name,
        rig.driven_keys.len(),
        rig.control_values.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_rig() {
        let r = new_control_rig_v2("face");
        assert_eq!(crv2_driven_key_count(&r), 0);
    }

    #[test]
    fn test_add_driven_key() {
        let mut r = new_control_rig_v2("face");
        crv2_add_driven_key(&mut r, "jaw_ctrl", "jaw_open", 1.0, 0.5);
        assert_eq!(crv2_driven_key_count(&r), 1);
    }

    #[test]
    fn test_set_control() {
        let mut r = new_control_rig_v2("face");
        crv2_set_control(&mut r, "jaw_ctrl", 0.8);
        assert_eq!(crv2_control_count(&r), 1);
    }

    #[test]
    fn test_evaluate() {
        let mut r = new_control_rig_v2("face");
        crv2_add_driven_key(&mut r, "jaw_ctrl", "jaw_open", 1.0, 1.0);
        crv2_set_control(&mut r, "jaw_ctrl", 1.0);
        let result = crv2_evaluate(&r);
        assert!((result["jaw_open"] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_partial_drive() {
        let mut r = new_control_rig_v2("face");
        crv2_add_driven_key(&mut r, "jaw_ctrl", "jaw_open", 1.0, 1.0);
        crv2_set_control(&mut r, "jaw_ctrl", 0.5);
        let result = crv2_evaluate(&r);
        assert!((result["jaw_open"] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_clear_driven_keys() {
        let mut r = new_control_rig_v2("face");
        crv2_add_driven_key(&mut r, "a", "b", 1.0, 1.0);
        crv2_clear_driven_keys(&mut r);
        assert_eq!(crv2_driven_key_count(&r), 0);
    }

    #[test]
    fn test_clear_controls() {
        let mut r = new_control_rig_v2("face");
        crv2_set_control(&mut r, "ctrl", 0.5);
        crv2_clear_controls(&mut r);
        assert_eq!(crv2_control_count(&r), 0);
    }

    #[test]
    fn test_remove_driven_keys_for() {
        let mut r = new_control_rig_v2("face");
        crv2_add_driven_key(&mut r, "jaw", "open", 1.0, 1.0);
        crv2_add_driven_key(&mut r, "brow", "raise", 1.0, 0.5);
        crv2_remove_driven_keys_for(&mut r, "jaw");
        assert_eq!(crv2_driven_key_count(&r), 1);
    }

    #[test]
    fn test_to_json() {
        let r = new_control_rig_v2("body");
        let json = crv2_to_json(&r);
        assert!(json.contains("\"name\":\"body\""));
    }

    #[test]
    fn test_evaluate_no_matching_control() {
        let mut r = new_control_rig_v2("face");
        crv2_add_driven_key(&mut r, "missing_ctrl", "jaw_open", 1.0, 1.0);
        let result = crv2_evaluate(&r);
        assert!(!result.contains_key("jaw_open"));
    }
}
