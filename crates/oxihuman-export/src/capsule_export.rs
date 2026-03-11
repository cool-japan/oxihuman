// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Export capsule collision/proxy shape data.
#[allow(dead_code)]
pub struct CapsuleExport {
    pub name: String,
    pub center: [f32; 3],
    pub axis: [f32; 3],
    pub radius: f32,
    pub half_height: f32,
}

#[allow(dead_code)]
pub struct CapsuleBundle {
    pub capsules: Vec<CapsuleExport>,
}

#[allow(dead_code)]
pub fn new_capsule_bundle() -> CapsuleBundle {
    CapsuleBundle { capsules: vec![] }
}

#[allow(dead_code)]
pub fn add_capsule(bundle: &mut CapsuleBundle, cap: CapsuleExport) {
    bundle.capsules.push(cap);
}

#[allow(dead_code)]
pub fn capsule_count(bundle: &CapsuleBundle) -> usize {
    bundle.capsules.len()
}

#[allow(dead_code)]
pub fn capsule_volume(cap: &CapsuleExport) -> f32 {
    let r = cap.radius;
    let h = cap.half_height * 2.0;
    // Cylinder + sphere
    PI * r * r * h + (4.0 / 3.0) * PI * r * r * r
}

#[allow(dead_code)]
pub fn capsule_surface_area(cap: &CapsuleExport) -> f32 {
    let r = cap.radius;
    let h = cap.half_height * 2.0;
    2.0 * PI * r * h + 4.0 * PI * r * r
}

#[allow(dead_code)]
pub fn capsule_total_length(cap: &CapsuleExport) -> f32 {
    cap.half_height * 2.0 + cap.radius * 2.0
}

#[allow(dead_code)]
pub fn capsule_to_json(cap: &CapsuleExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"radius\":{},\"half_height\":{},\"center\":[{},{},{}]}}",
        cap.name, cap.radius, cap.half_height, cap.center[0], cap.center[1], cap.center[2]
    )
}

#[allow(dead_code)]
pub fn capsule_bundle_to_json(bundle: &CapsuleBundle) -> String {
    format!("{{\"capsule_count\":{}}}", bundle.capsules.len())
}

#[allow(dead_code)]
pub fn validate_capsule(cap: &CapsuleExport) -> bool {
    cap.radius > 0.0 && cap.half_height >= 0.0 && !cap.name.is_empty()
}

#[allow(dead_code)]
pub fn find_capsule_by_name<'a>(
    bundle: &'a CapsuleBundle,
    name: &str,
) -> Option<&'a CapsuleExport> {
    bundle.capsules.iter().find(|c| c.name == name)
}

#[allow(dead_code)]
pub fn default_capsule(name: &str) -> CapsuleExport {
    CapsuleExport {
        name: name.to_string(),
        center: [0.0; 3],
        axis: [0.0, 1.0, 0.0],
        radius: 0.1,
        half_height: 0.5,
    }
}

#[allow(dead_code)]
pub fn total_capsule_volume(bundle: &CapsuleBundle) -> f32 {
    bundle.capsules.iter().map(capsule_volume).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arm_capsule() -> CapsuleExport {
        default_capsule("upper_arm")
    }

    #[test]
    fn test_add_capsule() {
        let mut b = new_capsule_bundle();
        add_capsule(&mut b, arm_capsule());
        assert_eq!(capsule_count(&b), 1);
    }

    #[test]
    fn test_capsule_volume_positive() {
        let cap = arm_capsule();
        assert!(capsule_volume(&cap) > 0.0);
    }

    #[test]
    fn test_capsule_surface_area_positive() {
        let cap = arm_capsule();
        assert!(capsule_surface_area(&cap) > 0.0);
    }

    #[test]
    fn test_capsule_total_length() {
        let cap = arm_capsule();
        let expected = cap.half_height * 2.0 + cap.radius * 2.0;
        assert!((capsule_total_length(&cap) - expected).abs() < 1e-5);
    }

    #[test]
    fn test_validate_capsule() {
        let cap = arm_capsule();
        assert!(validate_capsule(&cap));
    }

    #[test]
    fn test_find_capsule_found() {
        let mut b = new_capsule_bundle();
        add_capsule(&mut b, arm_capsule());
        assert!(find_capsule_by_name(&b, "upper_arm").is_some());
    }

    #[test]
    fn test_find_capsule_missing() {
        let b = new_capsule_bundle();
        assert!(find_capsule_by_name(&b, "arm").is_none());
    }

    #[test]
    fn test_total_volume() {
        let mut b = new_capsule_bundle();
        add_capsule(&mut b, arm_capsule());
        add_capsule(&mut b, arm_capsule());
        let single = capsule_volume(&arm_capsule());
        assert!((total_capsule_volume(&b) - 2.0 * single).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let cap = arm_capsule();
        let j = capsule_to_json(&cap);
        assert!(j.contains("upper_arm"));
        assert!(j.contains("radius"));
    }

    #[test]
    fn test_bundle_to_json() {
        let mut b = new_capsule_bundle();
        add_capsule(&mut b, arm_capsule());
        let j = capsule_bundle_to_json(&b);
        assert!(j.contains("capsule_count"));
    }
}
