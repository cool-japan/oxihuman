// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Trigger volume / sensor for overlap detection.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerVolumeShape {
    Sphere { radius: f32 },
    Box { half_extents: [f32; 3] },
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TriggerVolume {
    pub position: [f32; 3],
    pub shape: TriggerVolumeShape,
    pub id: u32,
    pub active: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TriggerVolumeEvent {
    pub trigger_id: u32,
    pub body_id: u32,
    pub entered: bool,
}

#[allow(dead_code)]
pub fn new_trigger_sphere(pos: [f32; 3], radius: f32, id: u32) -> TriggerVolume {
    TriggerVolume {
        position: pos,
        shape: TriggerVolumeShape::Sphere { radius },
        id,
        active: true,
    }
}

#[allow(dead_code)]
pub fn new_trigger_box(pos: [f32; 3], half_extents: [f32; 3], id: u32) -> TriggerVolume {
    TriggerVolume {
        position: pos,
        shape: TriggerVolumeShape::Box { half_extents },
        id,
        active: true,
    }
}

#[allow(dead_code)]
pub fn trigger_test_point(vol: &TriggerVolume, point: [f32; 3]) -> bool {
    if !vol.active {
        return false;
    }
    let dx = point[0] - vol.position[0];
    let dy = point[1] - vol.position[1];
    let dz = point[2] - vol.position[2];
    match vol.shape {
        TriggerVolumeShape::Sphere { radius } => {
            dx * dx + dy * dy + dz * dz <= radius * radius
        }
        TriggerVolumeShape::Box { half_extents } => {
            dx.abs() <= half_extents[0]
                && dy.abs() <= half_extents[1]
                && dz.abs() <= half_extents[2]
        }
    }
}

#[allow(dead_code)]
pub fn trigger_activate(vol: &mut TriggerVolume) {
    vol.active = true;
}

#[allow(dead_code)]
pub fn trigger_deactivate(vol: &mut TriggerVolume) {
    vol.active = false;
}

#[allow(dead_code)]
pub fn trigger_is_active(vol: &TriggerVolume) -> bool {
    vol.active
}

#[allow(dead_code)]
pub fn trigger_volume_to_json(vol: &TriggerVolume) -> String {
    let p = &vol.position;
    let shape_str = match &vol.shape {
        TriggerVolumeShape::Sphere { radius } => format!("\"sphere\",\"radius\":{}", radius),
        TriggerVolumeShape::Box { half_extents } => format!(
            "\"box\",\"half_extents\":[{},{},{}]",
            half_extents[0], half_extents[1], half_extents[2]
        ),
    };
    format!(
        "{{\"id\":{},\"active\":{},\"position\":[{},{},{}],\"shape\":{}}}",
        vol.id, vol.active, p[0], p[1], p[2], shape_str
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_trigger_sphere() {
        let v = new_trigger_sphere([0.0, 0.0, 0.0], 1.0, 1);
        assert!(trigger_is_active(&v));
        assert_eq!(v.id, 1);
    }

    #[test]
    fn test_sphere_contains_center() {
        let v = new_trigger_sphere([0.0, 0.0, 0.0], 2.0, 1);
        assert!(trigger_test_point(&v, [0.0, 0.0, 0.0]));
    }

    #[test]
    fn test_sphere_excludes_outside() {
        let v = new_trigger_sphere([0.0, 0.0, 0.0], 1.0, 1);
        assert!(!trigger_test_point(&v, [2.0, 0.0, 0.0]));
    }

    #[test]
    fn test_box_contains_center() {
        let v = new_trigger_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 2);
        assert!(trigger_test_point(&v, [0.0, 0.0, 0.0]));
    }

    #[test]
    fn test_box_excludes_outside() {
        let v = new_trigger_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 2);
        assert!(!trigger_test_point(&v, [2.0, 0.0, 0.0]));
    }

    #[test]
    fn test_activate_deactivate() {
        let mut v = new_trigger_sphere([0.0, 0.0, 0.0], 1.0, 3);
        trigger_deactivate(&mut v);
        assert!(!trigger_is_active(&v));
        // Point inside but inactive → false
        assert!(!trigger_test_point(&v, [0.0, 0.0, 0.0]));
        trigger_activate(&mut v);
        assert!(trigger_is_active(&v));
    }

    #[test]
    fn test_to_json() {
        let v = new_trigger_sphere([1.0, 2.0, 3.0], 0.5, 10);
        let j = trigger_volume_to_json(&v);
        assert!(j.contains("sphere"));
        assert!(j.contains("\"id\":10"));
    }
}
