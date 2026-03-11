#![allow(dead_code)]
//! Body muscle map: stores muscle entries with activation levels and force output.

/// A single muscle entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MuscleEntry {
    pub name: String,
    pub activation: f32,
    pub max_force: f32,
}

/// Collection of muscle entries.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyMuscleMap {
    muscles: Vec<MuscleEntry>,
}

/// Create a new empty muscle map.
#[allow(dead_code)]
pub fn new_body_muscle_map() -> BodyMuscleMap {
    BodyMuscleMap {
        muscles: Vec::new(),
    }
}

/// Add a muscle entry.
#[allow(dead_code)]
pub fn add_muscle_entry(map: &mut BodyMuscleMap, name: &str, max_force: f32) {
    map.muscles.push(MuscleEntry {
        name: name.to_string(),
        activation: 0.0,
        max_force,
    });
}

/// Get a reference to the muscle at `index`.
#[allow(dead_code)]
pub fn muscle_at(map: &BodyMuscleMap, index: usize) -> Option<&MuscleEntry> {
    map.muscles.get(index)
}

/// Return the number of muscles.
#[allow(dead_code)]
pub fn muscle_count_bmm(map: &BodyMuscleMap) -> usize {
    map.muscles.len()
}

/// Set the activation level of the muscle at `index`.
#[allow(dead_code)]
pub fn activate_muscle_bmm(map: &mut BodyMuscleMap, index: usize, activation: f32) {
    if let Some(m) = map.muscles.get_mut(index) {
        m.activation = activation.clamp(0.0, 1.0);
    }
}

/// Return the current force output of the muscle at `index`.
#[allow(dead_code)]
pub fn muscle_force_bmm(map: &BodyMuscleMap, index: usize) -> f32 {
    map.muscles
        .get(index)
        .map_or(0.0, |m| m.activation * m.max_force)
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn muscle_map_to_json(map: &BodyMuscleMap) -> String {
    let entries: Vec<String> = map
        .muscles
        .iter()
        .map(|m| {
            format!(
                "{{\"name\":\"{}\",\"activation\":{},\"max_force\":{}}}",
                m.name, m.activation, m.max_force
            )
        })
        .collect();
    format!("{{\"muscles\":[{}]}}", entries.join(","))
}

/// Remove all muscles.
#[allow(dead_code)]
pub fn clear_muscles(map: &mut BodyMuscleMap) {
    map.muscles.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map() {
        let m = new_body_muscle_map();
        assert_eq!(muscle_count_bmm(&m), 0);
    }

    #[test]
    fn test_add_muscle() {
        let mut m = new_body_muscle_map();
        add_muscle_entry(&mut m, "bicep", 100.0);
        assert_eq!(muscle_count_bmm(&m), 1);
    }

    #[test]
    fn test_muscle_at() {
        let mut m = new_body_muscle_map();
        add_muscle_entry(&mut m, "tricep", 80.0);
        let entry = muscle_at(&m, 0).unwrap();
        assert_eq!(entry.name, "tricep");
    }

    #[test]
    fn test_muscle_at_out_of_range() {
        let m = new_body_muscle_map();
        assert!(muscle_at(&m, 0).is_none());
    }

    #[test]
    fn test_activate_muscle() {
        let mut m = new_body_muscle_map();
        add_muscle_entry(&mut m, "quad", 200.0);
        activate_muscle_bmm(&mut m, 0, 0.5);
        assert!((muscle_at(&m, 0).unwrap().activation - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_activate_clamp() {
        let mut m = new_body_muscle_map();
        add_muscle_entry(&mut m, "glute", 150.0);
        activate_muscle_bmm(&mut m, 0, 2.0);
        assert!((muscle_at(&m, 0).unwrap().activation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_muscle_force() {
        let mut m = new_body_muscle_map();
        add_muscle_entry(&mut m, "delt", 100.0);
        activate_muscle_bmm(&mut m, 0, 0.5);
        assert!((muscle_force_bmm(&m, 0) - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_muscle_force_out_of_range() {
        let m = new_body_muscle_map();
        assert!((muscle_force_bmm(&m, 0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let m = new_body_muscle_map();
        let json = muscle_map_to_json(&m);
        assert!(json.contains("\"muscles\":[]"));
    }

    #[test]
    fn test_clear() {
        let mut m = new_body_muscle_map();
        add_muscle_entry(&mut m, "a", 1.0);
        clear_muscles(&mut m);
        assert_eq!(muscle_count_bmm(&m), 0);
    }
}
