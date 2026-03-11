#![allow(dead_code)]
//! Muscle group definitions for morph-driven muscle simulation.

/// Describes a single muscle action.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MuscleAction {
    /// Muscle name.
    pub name: String,
    /// Strength factor in [0, 1].
    pub strength: f32,
    /// Whether the muscle is currently activated.
    pub active: bool,
}

/// A group of related muscles.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MuscleGroup {
    /// Group label.
    pub label: String,
    /// Muscles in this group.
    pub muscles: Vec<MuscleAction>,
}

/// Create a new empty [`MuscleGroup`].
#[allow(dead_code)]
pub fn new_muscle_group(label: &str) -> MuscleGroup {
    MuscleGroup {
        label: label.to_string(),
        muscles: Vec::new(),
    }
}

/// Add a muscle to the group.
#[allow(dead_code)]
pub fn add_muscle(group: &mut MuscleGroup, name: &str, strength: f32) {
    group.muscles.push(MuscleAction {
        name: name.to_string(),
        strength: strength.clamp(0.0, 1.0),
        active: false,
    });
}

/// Activate a muscle by index.
#[allow(dead_code)]
pub fn activate_muscle(group: &mut MuscleGroup, index: usize) {
    if let Some(m) = group.muscles.get_mut(index) {
        m.active = true;
    }
}

/// Deactivate a muscle by index.
#[allow(dead_code)]
pub fn deactivate_muscle(group: &mut MuscleGroup, index: usize) {
    if let Some(m) = group.muscles.get_mut(index) {
        m.active = false;
    }
}

/// Return the number of muscles in the group.
#[allow(dead_code)]
pub fn muscle_count(group: &MuscleGroup) -> usize {
    group.muscles.len()
}

/// Return the strength of a muscle by index.
#[allow(dead_code)]
pub fn muscle_strength(group: &MuscleGroup, index: usize) -> f32 {
    group.muscles.get(index).map_or(0.0, |m| m.strength)
}

/// Compute the combined force of all active muscles.
#[allow(dead_code)]
pub fn muscle_group_force(group: &MuscleGroup) -> f32 {
    group
        .muscles
        .iter()
        .filter(|m| m.active)
        .map(|m| m.strength)
        .sum()
}

/// Serialize the muscle group to a JSON-like string.
#[allow(dead_code)]
pub fn muscle_group_to_json(group: &MuscleGroup) -> String {
    let mut s = format!("{{\"label\":\"{}\",\"muscles\":[", group.label);
    for (i, m) in group.muscles.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"name\":\"{}\",\"strength\":{:.4},\"active\":{}}}",
            m.name, m.strength, m.active
        ));
    }
    s.push_str("]}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_muscle_group() {
        let g = new_muscle_group("arm");
        assert_eq!(g.label, "arm");
        assert_eq!(muscle_count(&g), 0);
    }

    #[test]
    fn test_add_muscle() {
        let mut g = new_muscle_group("arm");
        add_muscle(&mut g, "bicep", 0.8);
        assert_eq!(muscle_count(&g), 1);
    }

    #[test]
    fn test_activate_deactivate() {
        let mut g = new_muscle_group("arm");
        add_muscle(&mut g, "bicep", 0.9);
        activate_muscle(&mut g, 0);
        assert!(g.muscles[0].active);
        deactivate_muscle(&mut g, 0);
        assert!(!g.muscles[0].active);
    }

    #[test]
    fn test_muscle_strength() {
        let mut g = new_muscle_group("arm");
        add_muscle(&mut g, "bicep", 0.7);
        assert!((muscle_strength(&g, 0) - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_muscle_strength_missing() {
        let g = new_muscle_group("arm");
        assert!((muscle_strength(&g, 0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_muscle_group_force_none_active() {
        let mut g = new_muscle_group("arm");
        add_muscle(&mut g, "a", 0.5);
        assert!((muscle_group_force(&g) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_muscle_group_force_active() {
        let mut g = new_muscle_group("arm");
        add_muscle(&mut g, "a", 0.3);
        add_muscle(&mut g, "b", 0.5);
        activate_muscle(&mut g, 0);
        activate_muscle(&mut g, 1);
        assert!((muscle_group_force(&g) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_muscle_group_to_json() {
        let mut g = new_muscle_group("leg");
        add_muscle(&mut g, "quad", 1.0);
        let json = muscle_group_to_json(&g);
        assert!(json.contains("quad"));
    }

    #[test]
    fn test_strength_clamped() {
        let mut g = new_muscle_group("arm");
        add_muscle(&mut g, "bicep", 1.5);
        assert!((muscle_strength(&g, 0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_activate_out_of_bounds() {
        let mut g = new_muscle_group("arm");
        activate_muscle(&mut g, 99); // should not panic
        assert_eq!(muscle_count(&g), 0);
    }
}
