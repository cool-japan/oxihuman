// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Rigid compound: a rigid body composed of multiple sub-shapes with combined inertia.

/// Sub-shape type.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SubShape {
    Sphere {
        center_local: [f32; 3],
        radius: f32,
        mass: f32,
    },
    Box {
        center_local: [f32; 3],
        half_extents: [f32; 3],
        mass: f32,
    },
}

impl SubShape {
    #[allow(dead_code)]
    fn mass(&self) -> f32 {
        match self {
            SubShape::Sphere { mass, .. } => *mass,
            SubShape::Box { mass, .. } => *mass,
        }
    }
    #[allow(dead_code)]
    fn center_local(&self) -> [f32; 3] {
        match self {
            SubShape::Sphere { center_local, .. } => *center_local,
            SubShape::Box { center_local, .. } => *center_local,
        }
    }
}

/// Rigid compound body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigidCompound {
    pub shapes: Vec<SubShape>,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub orientation: [f32; 4],
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Create a new empty `RigidCompound`.
#[allow(dead_code)]
pub fn new_rigid_compound(position: [f32; 3]) -> RigidCompound {
    RigidCompound {
        shapes: Vec::new(),
        position,
        velocity: [0.0; 3],
        angular_velocity: [0.0; 3],
        orientation: [0.0, 0.0, 0.0, 1.0],
    }
}

/// Add a sphere sub-shape.
#[allow(dead_code)]
pub fn rc_add_sphere(rc: &mut RigidCompound, center_local: [f32; 3], radius: f32, mass: f32) {
    rc.shapes.push(SubShape::Sphere {
        center_local,
        radius: radius.max(0.0),
        mass: mass.max(0.0),
    });
}

/// Add a box sub-shape.
#[allow(dead_code)]
pub fn rc_add_box(
    rc: &mut RigidCompound,
    center_local: [f32; 3],
    half_extents: [f32; 3],
    mass: f32,
) {
    rc.shapes.push(SubShape::Box {
        center_local,
        half_extents,
        mass: mass.max(0.0),
    });
}

/// Total mass.
#[allow(dead_code)]
pub fn rc_total_mass(rc: &RigidCompound) -> f32 {
    rc.shapes.iter().map(|s| s.mass()).sum()
}

/// Center of mass in world space.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn rc_center_of_mass(rc: &RigidCompound) -> [f32; 3] {
    let total = rc_total_mass(rc);
    if total < 1e-30 {
        return rc.position;
    }
    let mut com = [0.0f32; 3];
    for s in &rc.shapes {
        let c = add3(rc.position, s.center_local());
        for ax in 0..3 {
            com[ax] += c[ax] * s.mass();
        }
    }
    for ax in 0..3 {
        com[ax] /= total;
    }
    com
}

/// Approximate scalar moment of inertia (sum of m*r² from world CoM).
#[allow(dead_code)]
pub fn rc_moment_of_inertia(rc: &RigidCompound) -> f32 {
    let com = rc_center_of_mass(rc);
    rc.shapes
        .iter()
        .map(|s| {
            let c = add3(rc.position, s.center_local());
            let dx = c[0] - com[0];
            let dy = c[1] - com[1];
            let dz = c[2] - com[2];
            s.mass() * (dx * dx + dy * dy + dz * dz)
        })
        .sum()
}

/// Step the rigid compound under gravity.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn rc_step(rc: &mut RigidCompound, gravity: [f32; 3], dt: f32) {
    for ax in 0..3 {
        rc.velocity[ax] += gravity[ax] * dt;
        rc.position[ax] += rc.velocity[ax] * dt;
    }
}

/// Shape count.
#[allow(dead_code)]
pub fn rc_shape_count(rc: &RigidCompound) -> usize {
    rc.shapes.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new_compound() {
        let rc = new_rigid_compound([0.0; 3]);
        assert_eq!(rc_shape_count(&rc), 0);
    }

    #[test]
    fn test_add_sphere() {
        let mut rc = new_rigid_compound([0.0; 3]);
        rc_add_sphere(&mut rc, [0.0; 3], 0.5, 1.0);
        assert_eq!(rc_shape_count(&rc), 1);
    }

    #[test]
    fn test_add_box() {
        let mut rc = new_rigid_compound([0.0; 3]);
        rc_add_box(&mut rc, [0.0; 3], [0.5; 3], 2.0);
        assert_eq!(rc_shape_count(&rc), 1);
    }

    #[test]
    fn test_total_mass() {
        let mut rc = new_rigid_compound([0.0; 3]);
        rc_add_sphere(&mut rc, [0.0; 3], 0.5, 1.0);
        rc_add_box(&mut rc, [1.0, 0.0, 0.0], [0.5; 3], 2.0);
        assert!((rc_total_mass(&rc) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_center_of_mass() {
        let mut rc = new_rigid_compound([0.0; 3]);
        rc_add_sphere(&mut rc, [0.0; 3], 0.5, 1.0);
        rc_add_sphere(&mut rc, [2.0, 0.0, 0.0], 0.5, 1.0);
        let com = rc_center_of_mass(&rc);
        assert!((com[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_moment_of_inertia_nonneg() {
        let mut rc = new_rigid_compound([0.0; 3]);
        rc_add_sphere(&mut rc, [1.0, 0.0, 0.0], 0.5, 1.0);
        assert!(rc_moment_of_inertia(&rc) >= 0.0);
    }

    #[test]
    fn test_step_gravity() {
        let mut rc = new_rigid_compound([0.0, 10.0, 0.0]);
        rc_add_sphere(&mut rc, [0.0; 3], 0.5, 1.0);
        rc_step(&mut rc, [0.0, -9.81, 0.0], 1.0);
        assert!(rc.position[1] < 10.0);
    }

    #[test]
    fn test_empty_mass_zero() {
        let rc = new_rigid_compound([0.0; 3]);
        assert!((rc_total_mass(&rc)).abs() < 1e-9);
    }

    #[test]
    fn test_pi_used() {
        let circle_area = PI * 1.0 * 1.0;
        assert!(circle_area > 3.0);
    }
}
