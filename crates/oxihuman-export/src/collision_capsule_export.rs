// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A collision capsule.
#[allow(dead_code)]
#[derive(Clone)]
pub struct CollisionCapsule {
    pub name: String,
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub radius: f32,
}

/// Export bundle for collision capsules.
#[allow(dead_code)]
#[derive(Default)]
pub struct CollisionCapsuleExport {
    pub capsules: Vec<CollisionCapsule>,
}

/// Create a new collision capsule export.
#[allow(dead_code)]
pub fn new_collision_capsule_export() -> CollisionCapsuleExport {
    CollisionCapsuleExport::default()
}

/// Add a capsule.
#[allow(dead_code)]
pub fn add_collision_capsule(
    export: &mut CollisionCapsuleExport,
    name: &str,
    start: [f32; 3],
    end: [f32; 3],
    radius: f32,
) {
    export.capsules.push(CollisionCapsule {
        name: name.to_string(),
        start,
        end,
        radius,
    });
}

/// Count capsules.
#[allow(dead_code)]
pub fn capsule_count_cc(export: &CollisionCapsuleExport) -> usize {
    export.capsules.len()
}

/// Cylinder height of a capsule.
#[allow(dead_code)]
pub fn capsule_height(cap: &CollisionCapsule) -> f32 {
    let d = [
        cap.end[0] - cap.start[0],
        cap.end[1] - cap.start[1],
        cap.end[2] - cap.start[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Total volume of a capsule.
#[allow(dead_code)]
pub fn capsule_volume_cc(cap: &CollisionCapsule) -> f32 {
    let h = capsule_height(cap);
    let r = cap.radius;
    PI * r * r * h + (4.0 / 3.0) * PI * r * r * r
}

/// Total volume of all capsules.
#[allow(dead_code)]
pub fn total_capsule_volume_cc(export: &CollisionCapsuleExport) -> f32 {
    export.capsules.iter().map(capsule_volume_cc).sum()
}

/// Find capsule by name.
#[allow(dead_code)]
pub fn find_capsule_cc<'a>(
    export: &'a CollisionCapsuleExport,
    name: &str,
) -> Option<&'a CollisionCapsule> {
    export.capsules.iter().find(|c| c.name == name)
}

/// Validate all capsules have positive radius.
#[allow(dead_code)]
pub fn validate_capsules(export: &CollisionCapsuleExport) -> bool {
    export.capsules.iter().all(|c| c.radius > 0.0)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn collision_capsule_to_json(export: &CollisionCapsuleExport) -> String {
    format!(
        r#"{{"capsules":{},"total_volume":{:.4}}}"#,
        export.capsules.len(),
        total_capsule_volume_cc(export)
    )
}

/// Average radius across all capsules.
#[allow(dead_code)]
pub fn avg_radius_cc(export: &CollisionCapsuleExport) -> f32 {
    if export.capsules.is_empty() {
        return 0.0;
    }
    export.capsules.iter().map(|c| c.radius).sum::<f32>() / export.capsules.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_capsule() -> CollisionCapsule {
        CollisionCapsule {
            name: "arm".to_string(),
            start: [0.0, 0.0, 0.0],
            end: [0.0, 1.0, 0.0],
            radius: 0.1,
        }
    }

    #[test]
    fn add_and_count() {
        let mut e = new_collision_capsule_export();
        let c = make_capsule();
        add_collision_capsule(&mut e, &c.name, c.start, c.end, c.radius);
        assert_eq!(capsule_count_cc(&e), 1);
    }

    #[test]
    fn height_correct() {
        let c = make_capsule();
        assert!((capsule_height(&c) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn volume_positive() {
        let c = make_capsule();
        assert!(capsule_volume_cc(&c) > 0.0);
    }

    #[test]
    fn total_volume_sum() {
        let mut e = new_collision_capsule_export();
        let c = make_capsule();
        let v = capsule_volume_cc(&c);
        add_collision_capsule(&mut e, "a", c.start, c.end, c.radius);
        add_collision_capsule(&mut e, "b", c.start, c.end, c.radius);
        assert!((total_capsule_volume_cc(&e) - v * 2.0).abs() < 1e-4);
    }

    #[test]
    fn find_capsule() {
        let mut e = new_collision_capsule_export();
        let c = make_capsule();
        add_collision_capsule(&mut e, &c.name, c.start, c.end, c.radius);
        assert!(find_capsule_cc(&e, "arm").is_some());
    }

    #[test]
    fn validate_positive_radius() {
        let mut e = new_collision_capsule_export();
        let c = make_capsule();
        add_collision_capsule(&mut e, &c.name, c.start, c.end, c.radius);
        assert!(validate_capsules(&e));
    }

    #[test]
    fn validate_zero_radius_fails() {
        let mut e = new_collision_capsule_export();
        add_collision_capsule(&mut e, "x", [0.0; 3], [0.0, 1.0, 0.0], 0.0);
        assert!(!validate_capsules(&e));
    }

    #[test]
    fn json_has_capsules() {
        let e = new_collision_capsule_export();
        let j = collision_capsule_to_json(&e);
        assert!(j.contains("\"capsules\":0"));
    }

    #[test]
    fn avg_radius() {
        let mut e = new_collision_capsule_export();
        add_collision_capsule(&mut e, "a", [0.0; 3], [0.0, 1.0, 0.0], 0.2);
        add_collision_capsule(&mut e, "b", [0.0; 3], [0.0, 1.0, 0.0], 0.4);
        assert!((avg_radius_cc(&e) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn empty_avg_radius() {
        let e = new_collision_capsule_export();
        assert!((avg_radius_cc(&e) - 0.0).abs() < 1e-6);
    }
}
