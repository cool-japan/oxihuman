// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Trigger volumes (AABB, sphere, capsule) for event detection.

use std::f32::consts::PI;

#[allow(dead_code)]
pub enum TriggerShape {
    Sphere {
        center: [f32; 3],
        radius: f32,
    },
    Aabb {
        min: [f32; 3],
        max: [f32; 3],
    },
    Capsule {
        base: [f32; 3],
        tip: [f32; 3],
        radius: f32,
    },
    Cylinder {
        center: [f32; 3],
        radius: f32,
        half_height: f32,
    },
}

#[allow(dead_code)]
pub struct TriggerZone {
    pub id: u32,
    pub name: String,
    pub shape: TriggerShape,
    pub enabled: bool,
    pub enter_count: u64,
    pub exit_count: u64,
}

#[allow(dead_code)]
pub struct TriggerWorld {
    pub zones: Vec<TriggerZone>,
    pub next_id: u32,
}

#[allow(dead_code)]
pub struct TriggerEvent {
    pub zone_id: u32,
    pub point: [f32; 3],
    pub entered: bool,
}

/// Create an empty trigger world.
#[allow(dead_code)]
pub fn new_trigger_world() -> TriggerWorld {
    TriggerWorld {
        zones: Vec::new(),
        next_id: 0,
    }
}

fn alloc_zone(world: &mut TriggerWorld, name: &str, shape: TriggerShape) -> u32 {
    let id = world.next_id;
    world.next_id += 1;
    world.zones.push(TriggerZone {
        id,
        name: name.to_string(),
        shape,
        enabled: true,
        enter_count: 0,
        exit_count: 0,
    });
    id
}

/// Add a sphere trigger zone.
#[allow(dead_code)]
pub fn add_sphere_trigger(
    world: &mut TriggerWorld,
    name: &str,
    center: [f32; 3],
    radius: f32,
) -> u32 {
    alloc_zone(world, name, TriggerShape::Sphere { center, radius })
}

/// Add an AABB trigger zone.
#[allow(dead_code)]
pub fn add_aabb_trigger(world: &mut TriggerWorld, name: &str, min: [f32; 3], max: [f32; 3]) -> u32 {
    alloc_zone(world, name, TriggerShape::Aabb { min, max })
}

/// Add a capsule trigger zone.
#[allow(dead_code)]
pub fn add_capsule_trigger(
    world: &mut TriggerWorld,
    name: &str,
    base: [f32; 3],
    tip: [f32; 3],
    radius: f32,
) -> u32 {
    alloc_zone(world, name, TriggerShape::Capsule { base, tip, radius })
}

/// Check if a point is inside a sphere.
#[allow(dead_code)]
pub fn point_in_sphere(center: [f32; 3], radius: f32, point: [f32; 3]) -> bool {
    let dx = point[0] - center[0];
    let dy = point[1] - center[1];
    let dz = point[2] - center[2];
    dx * dx + dy * dy + dz * dz <= radius * radius
}

/// Check if a point is inside an AABB.
#[allow(dead_code)]
pub fn point_in_aabb(min: [f32; 3], max: [f32; 3], point: [f32; 3]) -> bool {
    point[0] >= min[0]
        && point[0] <= max[0]
        && point[1] >= min[1]
        && point[1] <= max[1]
        && point[2] >= min[2]
        && point[2] <= max[2]
}

/// Check if a point is inside a capsule (infinite cylinder with hemispherical caps).
#[allow(dead_code)]
pub fn point_in_capsule(base: [f32; 3], tip: [f32; 3], radius: f32, point: [f32; 3]) -> bool {
    let ab = [tip[0] - base[0], tip[1] - base[1], tip[2] - base[2]];
    let ap = [point[0] - base[0], point[1] - base[1], point[2] - base[2]];
    let ab_len2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];

    let t = if ab_len2 < 1e-12 {
        0.0
    } else {
        let dot = ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2];
        (dot / ab_len2).clamp(0.0, 1.0)
    };

    let closest = [
        base[0] + t * ab[0],
        base[1] + t * ab[1],
        base[2] + t * ab[2],
    ];
    let dx = point[0] - closest[0];
    let dy = point[1] - closest[1];
    let dz = point[2] - closest[2];
    dx * dx + dy * dy + dz * dz <= radius * radius
}

/// Check if a point is inside a trigger zone.
#[allow(dead_code)]
pub fn point_in_trigger(zone: &TriggerZone, point: [f32; 3]) -> bool {
    if !zone.enabled {
        return false;
    }
    match &zone.shape {
        TriggerShape::Sphere { center, radius } => point_in_sphere(*center, *radius, point),
        TriggerShape::Aabb { min, max } => point_in_aabb(*min, *max, point),
        TriggerShape::Capsule { base, tip, radius } => {
            point_in_capsule(*base, *tip, *radius, point)
        }
        TriggerShape::Cylinder {
            center,
            radius,
            half_height,
        } => {
            let dx = point[0] - center[0];
            let dz = point[2] - center[2];
            let in_circle = dx * dx + dz * dz <= radius * radius;
            let dy = (point[1] - center[1]).abs();
            in_circle && dy <= *half_height
        }
    }
}

/// Test a point against all enabled zones and return trigger events.
#[allow(dead_code)]
pub fn query_triggers(world: &mut TriggerWorld, point: [f32; 3]) -> Vec<TriggerEvent> {
    let mut events = Vec::new();
    for zone in world.zones.iter_mut() {
        if !zone.enabled {
            continue;
        }
        let inside = point_in_trigger(zone, point);
        if inside {
            zone.enter_count += 1;
            events.push(TriggerEvent {
                zone_id: zone.id,
                point,
                entered: true,
            });
        } else {
            zone.exit_count += 1;
            events.push(TriggerEvent {
                zone_id: zone.id,
                point,
                entered: false,
            });
        }
    }
    events
}

/// Remove a trigger by ID. Returns true if found and removed.
#[allow(dead_code)]
pub fn remove_trigger(world: &mut TriggerWorld, id: u32) -> bool {
    if let Some(pos) = world.zones.iter().position(|z| z.id == id) {
        world.zones.remove(pos);
        true
    } else {
        false
    }
}

/// Get an immutable reference to a trigger zone by ID.
#[allow(dead_code)]
pub fn get_trigger(world: &TriggerWorld, id: u32) -> Option<&TriggerZone> {
    world.zones.iter().find(|z| z.id == id)
}

/// Count enabled trigger zones.
#[allow(dead_code)]
pub fn enabled_trigger_count(world: &TriggerWorld) -> usize {
    world.zones.iter().filter(|z| z.enabled).count()
}

/// Compute approximate volume of a trigger zone.
#[allow(dead_code)]
pub fn trigger_zone_volume(zone: &TriggerZone) -> f32 {
    match &zone.shape {
        TriggerShape::Sphere { radius: r, .. } => (4.0 / 3.0) * PI * r * r * r,
        TriggerShape::Aabb { min, max } => {
            let dx = (max[0] - min[0]).abs();
            let dy = (max[1] - min[1]).abs();
            let dz = (max[2] - min[2]).abs();
            dx * dy * dz
        }
        TriggerShape::Capsule {
            base,
            tip,
            radius: r,
        } => {
            let dx = tip[0] - base[0];
            let dy = tip[1] - base[1];
            let dz = tip[2] - base[2];
            let h = (dx * dx + dy * dy + dz * dz).sqrt();
            PI * r * r * h + (4.0 / 3.0) * PI * r * r * r
        }
        TriggerShape::Cylinder {
            radius: r,
            half_height: hh,
            ..
        } => PI * r * r * (2.0 * hh),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_in_sphere_inside() {
        assert!(point_in_sphere([0.0, 0.0, 0.0], 1.0, [0.5, 0.0, 0.0]));
    }

    #[test]
    fn point_in_sphere_outside() {
        assert!(!point_in_sphere([0.0, 0.0, 0.0], 1.0, [2.0, 0.0, 0.0]));
    }

    #[test]
    fn point_on_sphere_surface() {
        assert!(point_in_sphere([0.0, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0]));
    }

    #[test]
    fn point_in_aabb_inside() {
        assert!(point_in_aabb(
            [-1.0, -1.0, -1.0],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0]
        ));
    }

    #[test]
    fn point_in_aabb_outside() {
        assert!(!point_in_aabb(
            [-1.0, -1.0, -1.0],
            [1.0, 1.0, 1.0],
            [2.0, 0.0, 0.0]
        ));
    }

    #[test]
    fn point_in_capsule_inside_cylinder() {
        // Vertical capsule from (0,0,0) to (0,2,0) with radius 1
        assert!(point_in_capsule(
            [0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            1.0,
            [0.5, 1.0, 0.0]
        ));
    }

    #[test]
    fn point_in_capsule_outside() {
        assert!(!point_in_capsule(
            [0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            0.5,
            [2.0, 1.0, 0.0]
        ));
    }

    #[test]
    fn point_in_capsule_in_end_cap() {
        // Point above tip but within radius
        assert!(point_in_capsule(
            [0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            1.0,
            [0.0, 2.5, 0.0]
        ));
    }

    #[test]
    fn add_sphere_trigger_and_query() {
        let mut world = new_trigger_world();
        add_sphere_trigger(&mut world, "test", [0.0, 0.0, 0.0], 2.0);
        let events = query_triggers(&mut world, [1.0, 0.0, 0.0]);
        assert!(!events.is_empty());
        assert!(events[0].entered);
    }

    #[test]
    fn query_outside_trigger() {
        let mut world = new_trigger_world();
        add_sphere_trigger(&mut world, "test", [0.0, 0.0, 0.0], 1.0);
        let events = query_triggers(&mut world, [5.0, 0.0, 0.0]);
        assert_eq!(events.len(), 1);
        assert!(!events[0].entered);
    }

    #[test]
    fn remove_trigger_returns_true() {
        let mut world = new_trigger_world();
        let id = add_sphere_trigger(&mut world, "test", [0.0, 0.0, 0.0], 1.0);
        assert!(remove_trigger(&mut world, id));
        assert!(world.zones.is_empty());
    }

    #[test]
    fn remove_nonexistent_trigger() {
        let mut world = new_trigger_world();
        assert!(!remove_trigger(&mut world, 999));
    }

    #[test]
    fn get_trigger_by_id() {
        let mut world = new_trigger_world();
        let id = add_aabb_trigger(&mut world, "aabb", [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        let zone = get_trigger(&world, id);
        assert!(zone.is_some());
        assert_eq!(zone.expect("should succeed").name, "aabb");
    }

    #[test]
    fn enabled_trigger_count_correct() {
        let mut world = new_trigger_world();
        let id = add_sphere_trigger(&mut world, "a", [0.0, 0.0, 0.0], 1.0);
        add_sphere_trigger(&mut world, "b", [5.0, 0.0, 0.0], 1.0);
        world
            .zones
            .iter_mut()
            .find(|z| z.id == id)
            .expect("should succeed")
            .enabled = false;
        assert_eq!(enabled_trigger_count(&world), 1);
    }

    #[test]
    fn sphere_volume_positive() {
        let mut world = new_trigger_world();
        let id = add_sphere_trigger(&mut world, "vol", [0.0, 0.0, 0.0], 2.0);
        let vol = trigger_zone_volume(get_trigger(&world, id).expect("should succeed"));
        assert!(vol > 0.0);
    }

    #[test]
    fn aabb_volume_correct() {
        let mut world = new_trigger_world();
        let id = add_aabb_trigger(&mut world, "box", [0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);
        let vol = trigger_zone_volume(get_trigger(&world, id).expect("should succeed"));
        assert!((vol - 24.0).abs() < 1e-4);
    }

    #[test]
    fn enter_count_increments() {
        let mut world = new_trigger_world();
        let id = add_sphere_trigger(&mut world, "cnt", [0.0, 0.0, 0.0], 2.0);
        query_triggers(&mut world, [0.0, 0.0, 0.0]);
        query_triggers(&mut world, [0.0, 0.0, 0.0]);
        assert_eq!(
            get_trigger(&world, id).expect("should succeed").enter_count,
            2
        );
    }
}
