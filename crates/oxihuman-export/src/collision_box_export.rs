// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Export axis-aligned and oriented bounding box collision shapes.
#[allow(dead_code)]
pub struct CollisionBox {
    pub name: String,
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
    pub rotation: [f32; 4], // quaternion xyzw
}

#[allow(dead_code)]
pub struct CollisionBoxBundle {
    pub boxes: Vec<CollisionBox>,
}

#[allow(dead_code)]
pub fn new_collision_box_bundle() -> CollisionBoxBundle {
    CollisionBoxBundle { boxes: vec![] }
}

#[allow(dead_code)]
pub fn add_collision_box(bundle: &mut CollisionBoxBundle, b: CollisionBox) {
    bundle.boxes.push(b);
}

#[allow(dead_code)]
pub fn collision_box_count(bundle: &CollisionBoxBundle) -> usize {
    bundle.boxes.len()
}

#[allow(dead_code)]
pub fn box_volume(b: &CollisionBox) -> f32 {
    b.half_extents[0] * b.half_extents[1] * b.half_extents[2] * 8.0
}

#[allow(dead_code)]
pub fn box_surface_area(b: &CollisionBox) -> f32 {
    let [ex, ey, ez] = b.half_extents;
    8.0 * (ex * ey + ey * ez + ez * ex)
}

#[allow(dead_code)]
pub fn default_collision_box(name: &str) -> CollisionBox {
    CollisionBox {
        name: name.to_string(),
        center: [0.0; 3],
        half_extents: [0.5, 0.5, 0.5],
        rotation: [0.0, 0.0, 0.0, 1.0],
    }
}

#[allow(dead_code)]
pub fn validate_collision_box(b: &CollisionBox) -> bool {
    b.half_extents.iter().all(|&e| e > 0.0) && !b.name.is_empty()
}

#[allow(dead_code)]
pub fn collision_box_to_json(b: &CollisionBox) -> String {
    format!(
        "{{\"name\":\"{}\",\"center\":[{},{},{}],\"half_extents\":[{},{},{}]}}",
        b.name,
        b.center[0],
        b.center[1],
        b.center[2],
        b.half_extents[0],
        b.half_extents[1],
        b.half_extents[2]
    )
}

#[allow(dead_code)]
pub fn collision_box_bundle_to_json(bundle: &CollisionBoxBundle) -> String {
    format!("{{\"box_count\":{}}}", bundle.boxes.len())
}

#[allow(dead_code)]
pub fn find_box_by_name<'a>(
    bundle: &'a CollisionBoxBundle,
    name: &str,
) -> Option<&'a CollisionBox> {
    bundle.boxes.iter().find(|b| b.name == name)
}

#[allow(dead_code)]
pub fn total_box_volume(bundle: &CollisionBoxBundle) -> f32 {
    bundle.boxes.iter().map(box_volume).sum()
}

#[allow(dead_code)]
pub fn point_in_box(b: &CollisionBox, p: [f32; 3]) -> bool {
    // AABB test (ignoring rotation)
    (p[0] - b.center[0]).abs() <= b.half_extents[0]
        && (p[1] - b.center[1]).abs() <= b.half_extents[1]
        && (p[2] - b.center[2]).abs() <= b.half_extents[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_box() -> CollisionBox {
        default_collision_box("torso")
    }

    #[test]
    fn test_add_box() {
        let mut b = new_collision_box_bundle();
        add_collision_box(&mut b, unit_box());
        assert_eq!(collision_box_count(&b), 1);
    }

    #[test]
    fn test_box_volume() {
        let b = unit_box();
        assert!((box_volume(&b) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_box_surface_area() {
        let b = unit_box();
        assert!((box_surface_area(&b) - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_validate_box() {
        let b = unit_box();
        assert!(validate_collision_box(&b));
    }

    #[test]
    fn test_validate_zero_extent_fails() {
        let mut b = unit_box();
        b.half_extents[0] = 0.0;
        assert!(!validate_collision_box(&b));
    }

    #[test]
    fn test_find_box_found() {
        let mut bundle = new_collision_box_bundle();
        add_collision_box(&mut bundle, unit_box());
        assert!(find_box_by_name(&bundle, "torso").is_some());
    }

    #[test]
    fn test_point_in_box() {
        let b = unit_box();
        assert!(point_in_box(&b, [0.0, 0.0, 0.0]));
        assert!(!point_in_box(&b, [1.0, 0.0, 0.0]));
    }

    #[test]
    fn test_total_volume() {
        let mut bundle = new_collision_box_bundle();
        add_collision_box(&mut bundle, unit_box());
        add_collision_box(&mut bundle, unit_box());
        assert!((total_box_volume(&bundle) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let b = unit_box();
        let j = collision_box_to_json(&b);
        assert!(j.contains("torso"));
    }

    #[test]
    fn test_bundle_to_json() {
        let mut bundle = new_collision_box_bundle();
        add_collision_box(&mut bundle, unit_box());
        let j = collision_box_bundle_to_json(&bundle);
        assert!(j.contains("box_count"));
    }
}
