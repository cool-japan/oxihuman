#![allow(dead_code)]
//! Manages scene lights — add, remove, update.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ManagedLight {
    pub name: String,
    pub color: [f32; 3],
    pub intensity: f32,
    pub position: [f32; 3],
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct LightManager {
    lights: Vec<ManagedLight>,
}

#[allow(dead_code)]
pub fn new_light_manager() -> LightManager {
    LightManager {
        lights: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_managed_light(lm: &mut LightManager, name: &str, color: [f32; 3], intensity: f32, position: [f32; 3]) {
    lm.lights.push(ManagedLight {
        name: name.to_string(),
        color,
        intensity,
        position,
    });
}

#[allow(dead_code)]
pub fn remove_light(lm: &mut LightManager, name: &str) -> bool {
    let before = lm.lights.len();
    lm.lights.retain(|l| l.name != name);
    lm.lights.len() < before
}

#[allow(dead_code)]
pub fn light_count_managed(lm: &LightManager) -> usize {
    lm.lights.len()
}

#[allow(dead_code)]
pub fn light_at(lm: &LightManager, index: usize) -> Option<&ManagedLight> {
    lm.lights.get(index)
}

#[allow(dead_code)]
pub fn update_light(lm: &mut LightManager, name: &str, intensity: f32) -> bool {
    if let Some(l) = lm.lights.iter_mut().find(|l| l.name == name) {
        l.intensity = intensity;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn lights_to_json(lm: &LightManager) -> String {
    let entries: Vec<String> = lm
        .lights
        .iter()
        .map(|l| {
            format!(
                "{{\"name\":\"{}\",\"intensity\":{}}}",
                l.name, l.intensity
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

#[allow(dead_code)]
pub fn clear_lights(lm: &mut LightManager) {
    lm.lights.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_light_manager() {
        let lm = new_light_manager();
        assert_eq!(light_count_managed(&lm), 0);
    }

    #[test]
    fn test_add_managed_light() {
        let mut lm = new_light_manager();
        add_managed_light(&mut lm, "sun", [1.0, 1.0, 1.0], 1.0, [0.0, 10.0, 0.0]);
        assert_eq!(light_count_managed(&lm), 1);
    }

    #[test]
    fn test_remove_light() {
        let mut lm = new_light_manager();
        add_managed_light(&mut lm, "sun", [1.0, 1.0, 1.0], 1.0, [0.0, 0.0, 0.0]);
        assert!(remove_light(&mut lm, "sun"));
        assert_eq!(light_count_managed(&lm), 0);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut lm = new_light_manager();
        assert!(!remove_light(&mut lm, "nope"));
    }

    #[test]
    fn test_light_at() {
        let mut lm = new_light_manager();
        add_managed_light(&mut lm, "key", [1.0, 0.9, 0.8], 2.0, [1.0, 2.0, 3.0]);
        let l = light_at(&lm, 0).expect("should succeed");
        assert_eq!(l.name, "key");
    }

    #[test]
    fn test_update_light() {
        let mut lm = new_light_manager();
        add_managed_light(&mut lm, "fill", [1.0, 1.0, 1.0], 0.5, [0.0, 0.0, 0.0]);
        assert!(update_light(&mut lm, "fill", 2.0));
        assert!((light_at(&lm, 0).expect("should succeed").intensity - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_update_missing() {
        let mut lm = new_light_manager();
        assert!(!update_light(&mut lm, "nope", 1.0));
    }

    #[test]
    fn test_lights_to_json() {
        let mut lm = new_light_manager();
        add_managed_light(&mut lm, "a", [1.0, 1.0, 1.0], 1.0, [0.0, 0.0, 0.0]);
        let json = lights_to_json(&lm);
        assert!(json.contains("\"name\":\"a\""));
    }

    #[test]
    fn test_clear_lights() {
        let mut lm = new_light_manager();
        add_managed_light(&mut lm, "a", [1.0, 1.0, 1.0], 1.0, [0.0, 0.0, 0.0]);
        clear_lights(&mut lm);
        assert_eq!(light_count_managed(&lm), 0);
    }

    #[test]
    fn test_multiple_lights() {
        let mut lm = new_light_manager();
        for i in 0..5 {
            add_managed_light(&mut lm, &format!("l{i}"), [1.0, 1.0, 1.0], 1.0, [0.0, 0.0, 0.0]);
        }
        assert_eq!(light_count_managed(&lm), 5);
    }
}
