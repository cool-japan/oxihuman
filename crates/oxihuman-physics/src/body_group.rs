#![allow(dead_code)]

/// A group of physics bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyGroup {
    pub body_ids: Vec<u32>,
    pub masses: Vec<f32>,
    pub positions: Vec<[f32; 3]>,
}

/// Creates a new empty body group.
#[allow(dead_code)]
pub fn new_body_group() -> BodyGroup {
    BodyGroup {
        body_ids: Vec::new(),
        masses: Vec::new(),
        positions: Vec::new(),
    }
}

/// Adds a body to the group.
#[allow(dead_code)]
pub fn add_to_group(group: &mut BodyGroup, id: u32, mass: f32, position: [f32; 3]) {
    group.body_ids.push(id);
    group.masses.push(mass);
    group.positions.push(position);
}

/// Removes a body from the group.
#[allow(dead_code)]
pub fn remove_from_group(group: &mut BodyGroup, id: u32) -> bool {
    if let Some(pos) = group.body_ids.iter().position(|&i| i == id) {
        group.body_ids.remove(pos);
        group.masses.remove(pos);
        group.positions.remove(pos);
        true
    } else {
        false
    }
}

/// Returns the number of bodies in the group.
#[allow(dead_code)]
pub fn group_size(group: &BodyGroup) -> usize {
    group.body_ids.len()
}

/// Returns true if the group contains the given id.
#[allow(dead_code)]
pub fn group_contains(group: &BodyGroup, id: u32) -> bool {
    group.body_ids.contains(&id)
}

/// Computes the center of mass of the group.
#[allow(dead_code)]
pub fn group_center_of_mass(group: &BodyGroup) -> [f32; 3] {
    let total_mass: f32 = group.masses.iter().sum();
    if total_mass < f32::EPSILON {
        return [0.0; 3];
    }
    let mut com = [0.0f32; 3];
    for (i, pos) in group.positions.iter().enumerate() {
        let m = group.masses[i];
        com[0] += pos[0] * m;
        com[1] += pos[1] * m;
        com[2] += pos[2] * m;
    }
    com[0] /= total_mass;
    com[1] /= total_mass;
    com[2] /= total_mass;
    com
}

/// Returns the total mass of the group.
#[allow(dead_code)]
pub fn group_total_mass(group: &BodyGroup) -> f32 {
    group.masses.iter().sum()
}

/// Clears the group.
#[allow(dead_code)]
pub fn clear_group(group: &mut BodyGroup) {
    group.body_ids.clear();
    group.masses.clear();
    group.positions.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_group() {
        let g = new_body_group();
        assert_eq!(group_size(&g), 0);
    }

    #[test]
    fn test_add() {
        let mut g = new_body_group();
        add_to_group(&mut g, 1, 1.0, [0.0; 3]);
        assert_eq!(group_size(&g), 1);
    }

    #[test]
    fn test_remove() {
        let mut g = new_body_group();
        add_to_group(&mut g, 1, 1.0, [0.0; 3]);
        assert!(remove_from_group(&mut g, 1));
        assert_eq!(group_size(&g), 0);
    }

    #[test]
    fn test_contains() {
        let mut g = new_body_group();
        add_to_group(&mut g, 5, 1.0, [0.0; 3]);
        assert!(group_contains(&g, 5));
        assert!(!group_contains(&g, 6));
    }

    #[test]
    fn test_center_of_mass() {
        let mut g = new_body_group();
        add_to_group(&mut g, 1, 1.0, [0.0, 0.0, 0.0]);
        add_to_group(&mut g, 2, 1.0, [2.0, 0.0, 0.0]);
        let com = group_center_of_mass(&g);
        assert!((com[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_total_mass() {
        let mut g = new_body_group();
        add_to_group(&mut g, 1, 2.0, [0.0; 3]);
        add_to_group(&mut g, 2, 3.0, [0.0; 3]);
        assert!((group_total_mass(&g) - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clear() {
        let mut g = new_body_group();
        add_to_group(&mut g, 1, 1.0, [0.0; 3]);
        clear_group(&mut g);
        assert_eq!(group_size(&g), 0);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut g = new_body_group();
        assert!(!remove_from_group(&mut g, 99));
    }

    #[test]
    fn test_center_of_mass_empty() {
        let g = new_body_group();
        let com = group_center_of_mass(&g);
        assert!((com[0]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_weighted_com() {
        let mut g = new_body_group();
        add_to_group(&mut g, 1, 1.0, [0.0, 0.0, 0.0]);
        add_to_group(&mut g, 2, 3.0, [4.0, 0.0, 0.0]);
        let com = group_center_of_mass(&g);
        assert!((com[0] - 3.0).abs() < f32::EPSILON);
    }
}
