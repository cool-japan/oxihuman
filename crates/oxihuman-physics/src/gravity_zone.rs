// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Gravity zone: localised gravity override regions.

use std::f32::consts::PI;

/// Shape of a gravity zone.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ZoneShape {
    Box {
        min: [f32; 3],
        max: [f32; 3],
    },
    Sphere {
        center: [f32; 3],
        radius: f32,
    },
    Cylinder {
        center: [f32; 3],
        radius: f32,
        half_height: f32,
    },
}

/// A gravity zone.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GravityZone {
    pub gravity: [f32; 3],
    pub shape: ZoneShape,
    pub priority: i32,
    pub enabled: bool,
}

/// Create a box gravity zone.
#[allow(dead_code)]
pub fn new_box_zone(gravity: [f32; 3], min: [f32; 3], max: [f32; 3], priority: i32) -> GravityZone {
    GravityZone {
        gravity,
        shape: ZoneShape::Box { min, max },
        priority,
        enabled: true,
    }
}

/// Create a sphere gravity zone.
#[allow(dead_code)]
pub fn new_sphere_zone(
    gravity: [f32; 3],
    center: [f32; 3],
    radius: f32,
    priority: i32,
) -> GravityZone {
    GravityZone {
        gravity,
        shape: ZoneShape::Sphere { center, radius },
        priority,
        enabled: true,
    }
}

/// Whether position is inside zone.
#[allow(dead_code)]
pub fn zone_contains(zone: &GravityZone, pos: [f32; 3]) -> bool {
    if !zone.enabled {
        return false;
    }
    match &zone.shape {
        ZoneShape::Box { min, max } => {
            (min[0]..=max[0]).contains(&pos[0])
                && (min[1]..=max[1]).contains(&pos[1])
                && (min[2]..=max[2]).contains(&pos[2])
        }
        ZoneShape::Sphere { center, radius } => {
            let d = [pos[0] - center[0], pos[1] - center[1], pos[2] - center[2]];
            d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= radius * radius
        }
        ZoneShape::Cylinder {
            center,
            radius,
            half_height,
        } => {
            let dx = pos[0] - center[0];
            let dz = pos[2] - center[2];
            let dy = (pos[1] - center[1]).abs();
            dx * dx + dz * dz <= radius * radius && dy <= *half_height
        }
    }
}

/// Get the active gravity for a position from a list of zones (highest priority wins).
#[allow(dead_code)]
pub fn gravity_at(zones: &[GravityZone], pos: [f32; 3], default_gravity: [f32; 3]) -> [f32; 3] {
    let mut best: Option<&GravityZone> = None;
    for zone in zones {
        if zone_contains(zone, pos) {
            match best {
                None => best = Some(zone),
                Some(b) if zone.priority > b.priority => best = Some(zone),
                _ => {}
            }
        }
    }
    best.map(|z| z.gravity).unwrap_or(default_gravity)
}

/// Volume of a zone (approximate for cylinder).
#[allow(dead_code)]
pub fn zone_volume(zone: &GravityZone) -> f32 {
    match &zone.shape {
        ZoneShape::Box { min, max } => {
            (max[0] - min[0]).max(0.0) * (max[1] - min[1]).max(0.0) * (max[2] - min[2]).max(0.0)
        }
        ZoneShape::Sphere { radius, .. } => 4.0 / 3.0 * PI * radius * radius * radius,
        ZoneShape::Cylinder {
            radius,
            half_height,
            ..
        } => PI * radius * radius * 2.0 * half_height,
    }
}

/// Enable or disable a zone.
#[allow(dead_code)]
pub fn zone_set_enabled(zone: &mut GravityZone, enabled: bool) {
    zone.enabled = enabled;
}

/// Update gravity direction.
#[allow(dead_code)]
pub fn zone_set_gravity(zone: &mut GravityZone, gravity: [f32; 3]) {
    zone.gravity = gravity;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_contains() {
        let z = new_box_zone([0.0, -9.81, 0.0], [0.0; 3], [10.0; 3], 0);
        assert!(zone_contains(&z, [5.0, 5.0, 5.0]));
        assert!(!zone_contains(&z, [11.0, 5.0, 5.0]));
    }

    #[test]
    fn test_sphere_contains() {
        let z = new_sphere_zone([0.0, 5.0, 0.0], [0.0; 3], 3.0, 0);
        assert!(zone_contains(&z, [0.0, 0.0, 0.0]));
        assert!(!zone_contains(&z, [4.0, 0.0, 0.0]));
    }

    #[test]
    fn test_disabled_zone() {
        let mut z = new_box_zone([1.0, 0.0, 0.0], [0.0; 3], [10.0; 3], 0);
        zone_set_enabled(&mut z, false);
        assert!(!zone_contains(&z, [5.0, 5.0, 5.0]));
    }

    #[test]
    fn test_gravity_at_default() {
        let default = [0.0, -9.81, 0.0];
        let g = gravity_at(&[], [0.0; 3], default);
        assert_eq!(g, default);
    }

    #[test]
    fn test_gravity_at_zone() {
        let z = new_sphere_zone([0.0, 5.0, 0.0], [0.0; 3], 10.0, 0);
        let g = gravity_at(&[z], [0.0; 3], [0.0, -9.81, 0.0]);
        assert!((g[1] - 5.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_priority() {
        let z1 = new_box_zone([0.0, -5.0, 0.0], [0.0; 3], [10.0; 3], 1);
        let z2 = new_box_zone([0.0, -2.0, 0.0], [0.0; 3], [10.0; 3], 2);
        let g = gravity_at(&[z1, z2], [5.0, 5.0, 5.0], [0.0, -9.81, 0.0]);
        assert!((g[1] - (-2.0_f32)).abs() < 1e-5);
    }

    #[test]
    fn test_box_volume() {
        let z = new_box_zone([0.0; 3], [0.0; 3], [2.0, 3.0, 4.0], 0);
        assert!((zone_volume(&z) - 24.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_sphere_volume() {
        let z = new_sphere_zone([0.0; 3], [0.0; 3], 1.0, 0);
        let expected = 4.0 / 3.0 * PI;
        assert!((zone_volume(&z) - expected).abs() < 1e-4);
    }

    #[test]
    fn test_set_gravity() {
        let mut z = new_box_zone([0.0, -9.81, 0.0], [0.0; 3], [1.0; 3], 0);
        zone_set_gravity(&mut z, [0.0, 0.0, 0.0]);
        assert_eq!(z.gravity, [0.0; 3]);
    }
}
