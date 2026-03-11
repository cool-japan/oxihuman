// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Static (immovable) body used as collision geometry.

/// Shape type for a static body.
#[derive(Debug, Clone, PartialEq)]
pub enum StaticShape {
    Plane {
        normal: [f32; 3],
        offset: f32,
    },
    Box {
        center: [f32; 3],
        half_extents: [f32; 3],
    },
    Sphere {
        center: [f32; 3],
        radius: f32,
    },
}

/// Immovable body used as world geometry.
#[derive(Debug, Clone)]
pub struct StaticBody {
    pub id: u64,
    pub shape: StaticShape,
    pub friction: f32,
    pub restitution: f32,
    pub enabled: bool,
    pub label: String,
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
impl StaticBody {
    pub fn new_plane(id: u64, normal: [f32; 3], offset: f32) -> Self {
        StaticBody {
            id,
            shape: StaticShape::Plane { normal, offset },
            friction: 0.5,
            restitution: 0.3,
            enabled: true,
            label: "plane".to_string(),
        }
    }

    pub fn new_sphere(id: u64, center: [f32; 3], radius: f32) -> Self {
        StaticBody {
            id,
            shape: StaticShape::Sphere { center, radius },
            friction: 0.5,
            restitution: 0.3,
            enabled: true,
            label: "sphere".to_string(),
        }
    }

    pub fn new_box(id: u64, center: [f32; 3], half_extents: [f32; 3]) -> Self {
        StaticBody {
            id,
            shape: StaticShape::Box {
                center,
                half_extents,
            },
            friction: 0.5,
            restitution: 0.3,
            enabled: true,
            label: "box".to_string(),
        }
    }

    /// Signed distance from point to surface (negative = inside/below).
    pub fn signed_distance(&self, point: [f32; 3]) -> f32 {
        match &self.shape {
            StaticShape::Plane { normal, offset } => dot3(*normal, point) - offset,
            StaticShape::Sphere { center, radius } => len3(sub3(point, *center)) - radius,
            StaticShape::Box {
                center,
                half_extents,
            } => {
                let d = [
                    (point[0] - center[0]).abs() - half_extents[0],
                    (point[1] - center[1]).abs() - half_extents[1],
                    (point[2] - center[2]).abs() - half_extents[2],
                ];
                let ext_len = [d[0].max(0.0), d[1].max(0.0), d[2].max(0.0)];
                len3(ext_len) + d[0].max(d[1]).max(d[2]).min(0.0)
            }
        }
    }

    pub fn is_point_inside(&self, point: [f32; 3]) -> bool {
        self.signed_distance(point) < 0.0
    }

    pub fn set_label(&mut self, label: &str) {
        self.label = label.to_string();
    }

    pub fn shape_name(&self) -> &str {
        match &self.shape {
            StaticShape::Plane { .. } => "plane",
            StaticShape::Sphere { .. } => "sphere",
            StaticShape::Box { .. } => "box",
        }
    }
}

pub fn new_static_plane(id: u64, normal: [f32; 3], offset: f32) -> StaticBody {
    StaticBody::new_plane(id, normal, offset)
}

pub fn new_static_sphere(id: u64, center: [f32; 3], radius: f32) -> StaticBody {
    StaticBody::new_sphere(id, center, radius)
}

pub fn new_static_box(id: u64, center: [f32; 3], half_extents: [f32; 3]) -> StaticBody {
    StaticBody::new_box(id, center, half_extents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plane_above_surface() {
        let p = new_static_plane(1, [0.0, 1.0, 0.0], 0.0);
        assert!(p.signed_distance([0.0, 1.0, 0.0]) > 0.0);
    }

    #[test]
    fn plane_on_surface() {
        let p = new_static_plane(1, [0.0, 1.0, 0.0], 0.0);
        assert!((p.signed_distance([0.0, 0.0, 0.0])).abs() < 1e-6);
    }

    #[test]
    fn sphere_outside() {
        let s = new_static_sphere(1, [0.0; 3], 1.0);
        assert!(s.signed_distance([2.0, 0.0, 0.0]) > 0.0);
    }

    #[test]
    fn sphere_inside() {
        let s = new_static_sphere(1, [0.0; 3], 2.0);
        assert!(s.is_point_inside([0.5, 0.0, 0.0]));
    }

    #[test]
    fn box_inside() {
        let b = new_static_box(1, [0.0; 3], [1.0; 3]);
        assert!(b.is_point_inside([0.5, 0.5, 0.5]));
    }

    #[test]
    fn box_outside() {
        let b = new_static_box(1, [0.0; 3], [1.0; 3]);
        assert!(!b.is_point_inside([2.0, 0.0, 0.0]));
    }

    #[test]
    fn shape_name() {
        let p = new_static_plane(1, [0.0, 1.0, 0.0], 0.0);
        assert_eq!(p.shape_name(), "plane");
    }

    #[test]
    fn label() {
        let mut p = new_static_plane(1, [0.0, 1.0, 0.0], 0.0);
        p.set_label("floor");
        assert_eq!(p.label, "floor");
    }

    #[test]
    fn disabled_body_still_has_shape() {
        let mut s = new_static_sphere(1, [0.0; 3], 1.0);
        s.enabled = false;
        assert_eq!(s.shape_name(), "sphere");
    }
}
