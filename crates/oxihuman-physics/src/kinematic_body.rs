// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Kinematic body for animation-driven collision.

#[allow(dead_code)]
pub struct KinematicBody {
    pub id: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub collision_shape: KinematicShape,
    pub layer_mask: u32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub enum KinematicShape {
    Sphere {
        radius: f32,
    },
    Capsule {
        radius: f32,
        height: f32,
    },
    Box {
        half_extents: [f32; 3],
    },
    Mesh {
        vertex_count: usize,
        triangle_count: usize,
    },
}

#[allow(dead_code)]
pub struct KinematicWorld {
    pub bodies: Vec<KinematicBody>,
    pub next_id: u32,
}

#[allow(dead_code)]
pub struct KinematicContact {
    pub body_a: u32,
    pub body_b: u32,
    pub contact_point: [f32; 3],
    pub normal: [f32; 3],
    pub depth: f32,
}

/// Create an empty kinematic world.
#[allow(dead_code)]
pub fn new_kinematic_world() -> KinematicWorld {
    KinematicWorld {
        bodies: Vec::new(),
        next_id: 0,
    }
}

/// Add a new body to the world and return its ID.
#[allow(dead_code)]
pub fn add_kinematic_body(world: &mut KinematicWorld, pos: [f32; 3], shape: KinematicShape) -> u32 {
    let id = world.next_id;
    world.next_id += 1;
    world.bodies.push(KinematicBody {
        id,
        position: pos,
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
        velocity: [0.0, 0.0, 0.0],
        angular_velocity: [0.0, 0.0, 0.0],
        collision_shape: shape,
        layer_mask: 0xFFFF_FFFF,
        enabled: true,
    });
    id
}

/// Move a body to a new position and compute velocity from delta.
#[allow(dead_code)]
pub fn move_body(world: &mut KinematicWorld, id: u32, new_pos: [f32; 3], dt: f32) {
    if let Some(body) = world.bodies.iter_mut().find(|b| b.id == id) {
        let inv_dt = if dt > 1e-12 { 1.0 / dt } else { 0.0 };
        body.velocity = [
            (new_pos[0] - body.position[0]) * inv_dt,
            (new_pos[1] - body.position[1]) * inv_dt,
            (new_pos[2] - body.position[2]) * inv_dt,
        ];
        body.position = new_pos;
    }
}

/// Set the rotation quaternion of a body.
#[allow(dead_code)]
pub fn set_body_rotation(world: &mut KinematicWorld, id: u32, rot: [f32; 4]) {
    if let Some(body) = world.bodies.iter_mut().find(|b| b.id == id) {
        body.rotation = rot;
    }
}

/// Get an immutable reference to a body by ID.
#[allow(dead_code)]
pub fn get_body(world: &KinematicWorld, id: u32) -> Option<&KinematicBody> {
    world.bodies.iter().find(|b| b.id == id)
}

/// Remove a body by ID. Returns true if found and removed.
#[allow(dead_code)]
pub fn remove_body(world: &mut KinematicWorld, id: u32) -> bool {
    if let Some(pos) = world.bodies.iter().position(|b| b.id == id) {
        world.bodies.remove(pos);
        true
    } else {
        false
    }
}

/// Compute contact between two sphere bodies.
#[allow(dead_code)]
pub fn sphere_sphere_contact(a: &KinematicBody, b: &KinematicBody) -> Option<KinematicContact> {
    let (ra, rb) = match (&a.collision_shape, &b.collision_shape) {
        (KinematicShape::Sphere { radius: ra }, KinematicShape::Sphere { radius: rb }) => {
            (*ra, *rb)
        }
        _ => return None,
    };

    let dx = b.position[0] - a.position[0];
    let dy = b.position[1] - a.position[1];
    let dz = b.position[2] - a.position[2];
    let dist2 = dx * dx + dy * dy + dz * dz;
    let sum_r = ra + rb;

    if dist2 >= sum_r * sum_r {
        return None;
    }

    let dist = dist2.sqrt();
    let (nx, ny, nz) = if dist < 1e-12 {
        (0.0, 1.0, 0.0)
    } else {
        (dx / dist, dy / dist, dz / dist)
    };

    let depth = sum_r - dist;
    let contact_point = [
        a.position[0] + nx * ra,
        a.position[1] + ny * ra,
        a.position[2] + nz * ra,
    ];

    Some(KinematicContact {
        body_a: a.id,
        body_b: b.id,
        contact_point,
        normal: [nx, ny, nz],
        depth,
    })
}

/// Compute the axis-aligned bounding box of a body.
#[allow(dead_code)]
pub fn aabb_of_body(body: &KinematicBody) -> ([f32; 3], [f32; 3]) {
    let p = body.position;
    let half = match &body.collision_shape {
        KinematicShape::Sphere { radius } => [*radius, *radius, *radius],
        KinematicShape::Capsule { radius, height } => [*radius, height / 2.0 + radius, *radius],
        KinematicShape::Box { half_extents } => *half_extents,
        KinematicShape::Mesh { .. } => [1.0, 1.0, 1.0],
    };
    let min = [p[0] - half[0], p[1] - half[1], p[2] - half[2]];
    let max = [p[0] + half[0], p[1] + half[1], p[2] + half[2]];
    (min, max)
}

/// AABB overlap test.
#[allow(dead_code)]
pub fn bodies_overlap(a: &KinematicBody, b: &KinematicBody) -> bool {
    let (a_min, a_max) = aabb_of_body(a);
    let (b_min, b_max) = aabb_of_body(b);
    a_min[0] <= b_max[0]
        && a_max[0] >= b_min[0]
        && a_min[1] <= b_max[1]
        && a_max[1] >= b_min[1]
        && a_min[2] <= b_max[2]
        && a_max[2] >= b_min[2]
}

/// Count enabled bodies.
#[allow(dead_code)]
pub fn enabled_body_count(world: &KinematicWorld) -> usize {
    world.bodies.iter().filter(|b| b.enabled).count()
}

/// Total body count.
#[allow(dead_code)]
pub fn kinematic_body_count(world: &KinematicWorld) -> usize {
    world.bodies.len()
}

/// Set layer mask of a body.
#[allow(dead_code)]
pub fn set_layer_mask(world: &mut KinematicWorld, id: u32, mask: u32) {
    if let Some(body) = world.bodies.iter_mut().find(|b| b.id == id) {
        body.layer_mask = mask;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere_body(id_offset: u32, pos: [f32; 3], radius: f32) -> KinematicBody {
        KinematicBody {
            id: id_offset,
            position: pos,
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
            velocity: [0.0, 0.0, 0.0],
            angular_velocity: [0.0, 0.0, 0.0],
            collision_shape: KinematicShape::Sphere { radius },
            layer_mask: 0xFFFF_FFFF,
            enabled: true,
        }
    }

    #[test]
    fn add_body_increments_count() {
        let mut world = new_kinematic_world();
        add_kinematic_body(
            &mut world,
            [0.0, 0.0, 0.0],
            KinematicShape::Sphere { radius: 1.0 },
        );
        add_kinematic_body(
            &mut world,
            [5.0, 0.0, 0.0],
            KinematicShape::Sphere { radius: 1.0 },
        );
        assert_eq!(kinematic_body_count(&world), 2);
    }

    #[test]
    fn add_body_returns_unique_ids() {
        let mut world = new_kinematic_world();
        let id0 = add_kinematic_body(
            &mut world,
            [0.0, 0.0, 0.0],
            KinematicShape::Sphere { radius: 1.0 },
        );
        let id1 = add_kinematic_body(
            &mut world,
            [5.0, 0.0, 0.0],
            KinematicShape::Sphere { radius: 1.0 },
        );
        assert_ne!(id0, id1);
    }

    #[test]
    fn move_body_updates_velocity() {
        let mut world = new_kinematic_world();
        let id = add_kinematic_body(
            &mut world,
            [0.0, 0.0, 0.0],
            KinematicShape::Sphere { radius: 1.0 },
        );
        move_body(&mut world, id, [2.0, 0.0, 0.0], 1.0);
        let body = get_body(&world, id).unwrap();
        assert!((body.velocity[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn move_body_updates_position() {
        let mut world = new_kinematic_world();
        let id = add_kinematic_body(
            &mut world,
            [0.0, 0.0, 0.0],
            KinematicShape::Sphere { radius: 1.0 },
        );
        move_body(&mut world, id, [3.0, 4.0, 0.0], 0.1);
        let body = get_body(&world, id).unwrap();
        assert!((body.position[0] - 3.0).abs() < 1e-5);
        assert!((body.position[1] - 4.0).abs() < 1e-5);
    }

    #[test]
    fn get_body_returns_none_for_invalid_id() {
        let world = new_kinematic_world();
        assert!(get_body(&world, 999).is_none());
    }

    #[test]
    fn remove_body_returns_true() {
        let mut world = new_kinematic_world();
        let id = add_kinematic_body(
            &mut world,
            [0.0, 0.0, 0.0],
            KinematicShape::Sphere { radius: 1.0 },
        );
        assert!(remove_body(&mut world, id));
        assert_eq!(kinematic_body_count(&world), 0);
    }

    #[test]
    fn remove_nonexistent_body_returns_false() {
        let mut world = new_kinematic_world();
        assert!(!remove_body(&mut world, 42));
    }

    #[test]
    fn sphere_contact_overlapping() {
        let a = sphere_body(0, [0.0, 0.0, 0.0], 1.0);
        let b = sphere_body(1, [1.5, 0.0, 0.0], 1.0);
        let contact = sphere_sphere_contact(&a, &b);
        assert!(contact.is_some());
        let c = contact.unwrap();
        assert!(c.depth > 0.0);
    }

    #[test]
    fn sphere_contact_not_overlapping() {
        let a = sphere_body(0, [0.0, 0.0, 0.0], 1.0);
        let b = sphere_body(1, [5.0, 0.0, 0.0], 1.0);
        assert!(sphere_sphere_contact(&a, &b).is_none());
    }

    #[test]
    fn aabb_of_sphere_body() {
        let a = sphere_body(0, [1.0, 2.0, 3.0], 0.5);
        let (min, max) = aabb_of_body(&a);
        assert!((min[0] - 0.5).abs() < 1e-6);
        assert!((max[0] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn bodies_overlap_true() {
        let a = sphere_body(0, [0.0, 0.0, 0.0], 1.0);
        let b = sphere_body(1, [1.0, 0.0, 0.0], 1.0);
        assert!(bodies_overlap(&a, &b));
    }

    #[test]
    fn bodies_overlap_false() {
        let a = sphere_body(0, [0.0, 0.0, 0.0], 1.0);
        let b = sphere_body(1, [10.0, 0.0, 0.0], 1.0);
        assert!(!bodies_overlap(&a, &b));
    }

    #[test]
    fn enabled_count() {
        let mut world = new_kinematic_world();
        let id = add_kinematic_body(
            &mut world,
            [0.0, 0.0, 0.0],
            KinematicShape::Sphere { radius: 1.0 },
        );
        add_kinematic_body(
            &mut world,
            [5.0, 0.0, 0.0],
            KinematicShape::Sphere { radius: 1.0 },
        );
        // Disable one
        world
            .bodies
            .iter_mut()
            .find(|b| b.id == id)
            .unwrap()
            .enabled = false;
        assert_eq!(enabled_body_count(&world), 1);
    }

    #[test]
    fn set_layer_mask_updates() {
        let mut world = new_kinematic_world();
        let id = add_kinematic_body(
            &mut world,
            [0.0, 0.0, 0.0],
            KinematicShape::Sphere { radius: 1.0 },
        );
        set_layer_mask(&mut world, id, 0x0F);
        assert_eq!(get_body(&world, id).unwrap().layer_mask, 0x0F);
    }
}
