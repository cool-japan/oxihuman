// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export compound collision shapes (collections of primitive shapes).

/// Primitive collision shape type.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum CollisionPrimitive {
    Sphere {
        center: [f32; 3],
        radius: f32,
    },
    Box {
        center: [f32; 3],
        half_extents: [f32; 3],
    },
    Capsule {
        a: [f32; 3],
        b: [f32; 3],
        radius: f32,
    },
}

/// A compound collision shape composed of primitives.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CompoundCollision {
    pub name: String,
    pub primitives: Vec<CollisionPrimitive>,
}

/// Create a new compound collision shape.
#[allow(dead_code)]
pub fn new_compound(name: &str) -> CompoundCollision {
    CompoundCollision {
        name: name.to_string(),
        primitives: vec![],
    }
}

/// Add a sphere primitive.
#[allow(dead_code)]
pub fn add_sphere(compound: &mut CompoundCollision, center: [f32; 3], radius: f32) {
    compound
        .primitives
        .push(CollisionPrimitive::Sphere { center, radius });
}

/// Add a box primitive.
#[allow(dead_code)]
pub fn add_box(compound: &mut CompoundCollision, center: [f32; 3], half_extents: [f32; 3]) {
    compound.primitives.push(CollisionPrimitive::Box {
        center,
        half_extents,
    });
}

/// Add a capsule primitive.
#[allow(dead_code)]
pub fn add_capsule(compound: &mut CompoundCollision, a: [f32; 3], b: [f32; 3], radius: f32) {
    compound
        .primitives
        .push(CollisionPrimitive::Capsule { a, b, radius });
}

/// Count of primitives.
#[allow(dead_code)]
pub fn primitive_count(compound: &CompoundCollision) -> usize {
    compound.primitives.len()
}

/// Approximate total volume of the compound shape.
#[allow(dead_code)]
pub fn compound_volume(compound: &CompoundCollision) -> f32 {
    use std::f32::consts::PI;
    compound
        .primitives
        .iter()
        .map(|p| match p {
            CollisionPrimitive::Sphere { radius, .. } => {
                (4.0 / 3.0) * PI * radius * radius * radius
            }
            CollisionPrimitive::Box { half_extents, .. } => {
                8.0 * half_extents[0] * half_extents[1] * half_extents[2]
            }
            CollisionPrimitive::Capsule { a, b, radius } => {
                let len = {
                    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
                };
                PI * radius * radius * len + (4.0 / 3.0) * PI * radius * radius * radius
            }
        })
        .sum()
}

/// Compute the AABB of all sphere primitives' centers.
#[allow(dead_code)]
pub fn aabb_of_centers(compound: &CompoundCollision) -> Option<([f32; 3], [f32; 3])> {
    let centers: Vec<[f32; 3]> = compound
        .primitives
        .iter()
        .map(|p| match p {
            CollisionPrimitive::Sphere { center, .. } => *center,
            CollisionPrimitive::Box { center, .. } => *center,
            CollisionPrimitive::Capsule { a, b, .. } => [
                (a[0] + b[0]) * 0.5,
                (a[1] + b[1]) * 0.5,
                (a[2] + b[2]) * 0.5,
            ],
        })
        .collect();
    if centers.is_empty() {
        return None;
    }
    let mn = centers
        .iter()
        .cloned()
        .reduce(|a, b| [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])])?;
    let mx = centers
        .iter()
        .cloned()
        .reduce(|a, b| [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])])?;
    Some((mn, mx))
}

/// Serialise compound to flat f32 buffer (type tag + data per primitive).
#[allow(dead_code)]
pub fn serialise_compound(compound: &CompoundCollision) -> Vec<f32> {
    let mut buf = Vec::new();
    for p in &compound.primitives {
        match p {
            CollisionPrimitive::Sphere { center, radius } => {
                buf.extend_from_slice(&[0.0, center[0], center[1], center[2], *radius]);
            }
            CollisionPrimitive::Box {
                center,
                half_extents,
            } => {
                buf.extend_from_slice(&[
                    1.0,
                    center[0],
                    center[1],
                    center[2],
                    half_extents[0],
                    half_extents[1],
                    half_extents[2],
                ]);
            }
            CollisionPrimitive::Capsule { a, b, radius } => {
                buf.extend_from_slice(&[2.0, a[0], a[1], a[2], b[0], b[1], b[2], *radius]);
            }
        }
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_compound() {
        let c = new_compound("body");
        assert_eq!(primitive_count(&c), 0);
    }

    #[test]
    fn test_add_sphere() {
        let mut c = new_compound("c");
        add_sphere(&mut c, [0.0; 3], 1.0);
        assert_eq!(primitive_count(&c), 1);
    }

    #[test]
    fn test_add_box() {
        let mut c = new_compound("c");
        add_box(&mut c, [0.0; 3], [1.0; 3]);
        assert_eq!(primitive_count(&c), 1);
    }

    #[test]
    fn test_add_capsule() {
        let mut c = new_compound("c");
        add_capsule(&mut c, [0.0; 3], [0.0, 1.0, 0.0], 0.1);
        assert_eq!(primitive_count(&c), 1);
    }

    #[test]
    fn test_compound_volume_positive() {
        let mut c = new_compound("c");
        add_sphere(&mut c, [0.0; 3], 1.0);
        assert!(compound_volume(&c) > 0.0);
    }

    #[test]
    fn test_aabb_of_centers_none_for_empty() {
        let c = new_compound("c");
        assert!(aabb_of_centers(&c).is_none());
    }

    #[test]
    fn test_aabb_of_centers_some() {
        let mut c = new_compound("c");
        add_sphere(&mut c, [-1.0, 0.0, 0.0], 0.5);
        add_sphere(&mut c, [1.0, 0.0, 0.0], 0.5);
        let (mn, mx) = aabb_of_centers(&c).unwrap();
        assert!((mn[0] - (-1.0)).abs() < 1e-6);
        assert!((mx[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_serialise_sphere() {
        let mut c = new_compound("c");
        add_sphere(&mut c, [0.0; 3], 1.0);
        let buf = serialise_compound(&c);
        assert_eq!(buf.len(), 5);
        assert!((buf[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_serialise_empty() {
        let c = new_compound("c");
        assert!(serialise_compound(&c).is_empty());
    }

    #[test]
    fn test_name_stored() {
        let c = new_compound("spine");
        assert_eq!(c.name, "spine".to_string());
    }
}
