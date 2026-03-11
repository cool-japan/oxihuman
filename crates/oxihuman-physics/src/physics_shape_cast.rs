#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shape casting for physics queries.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeCastResult {
    pub hit: bool,
    pub distance: f32,
    pub normal: [f32; 3],
    pub point: [f32; 3],
}

#[allow(dead_code)]
fn no_hit() -> ShapeCastResult {
    ShapeCastResult {
        hit: false,
        distance: f32::INFINITY,
        normal: [0.0, 0.0, 0.0],
        point: [0.0, 0.0, 0.0],
    }
}

#[allow(dead_code)]
pub fn cast_sphere_phys(origin: [f32; 3], dir: [f32; 3], radius: f32, plane_y: f32) -> ShapeCastResult {
    if dir[1].abs() < 1e-10 {
        return no_hit();
    }
    let t = (plane_y - origin[1] + radius) / (-dir[1]);
    if t < 0.0 {
        return no_hit();
    }
    ShapeCastResult {
        hit: true,
        distance: t,
        normal: [0.0, 1.0, 0.0],
        point: [
            origin[0] + dir[0] * t,
            plane_y,
            origin[2] + dir[2] * t,
        ],
    }
}

#[allow(dead_code)]
pub fn cast_box_phys(origin: [f32; 3], dir: [f32; 3], half_ext: [f32; 3], plane_y: f32) -> ShapeCastResult {
    if dir[1].abs() < 1e-10 {
        return no_hit();
    }
    let t = (plane_y - origin[1] + half_ext[1]) / (-dir[1]);
    if t < 0.0 {
        return no_hit();
    }
    ShapeCastResult {
        hit: true,
        distance: t,
        normal: [0.0, 1.0, 0.0],
        point: [
            origin[0] + dir[0] * t,
            plane_y,
            origin[2] + dir[2] * t,
        ],
    }
}

#[allow(dead_code)]
pub fn cast_capsule_phys(origin: [f32; 3], dir: [f32; 3], radius: f32, _half_height: f32, plane_y: f32) -> ShapeCastResult {
    cast_sphere_phys(origin, dir, radius, plane_y)
}

#[allow(dead_code)]
pub fn shape_cast_distance(result: &ShapeCastResult) -> f32 {
    result.distance
}

#[allow(dead_code)]
pub fn shape_cast_normal(result: &ShapeCastResult) -> [f32; 3] {
    result.normal
}

#[allow(dead_code)]
pub fn shape_cast_point(result: &ShapeCastResult) -> [f32; 3] {
    result.point
}

#[allow(dead_code)]
pub fn shape_cast_hit(result: &ShapeCastResult) -> bool {
    result.hit
}

#[allow(dead_code)]
pub fn shape_cast_to_json(result: &ShapeCastResult) -> String {
    format!(
        r#"{{"hit":{},"distance":{}}}"#,
        result.hit, result.distance
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_cast_hit() {
        let r = cast_sphere_phys([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], 0.5, 0.0);
        assert!(shape_cast_hit(&r));
    }

    #[test]
    fn test_sphere_cast_miss() {
        let r = cast_sphere_phys([0.0, 5.0, 0.0], [1.0, 0.0, 0.0], 0.5, 0.0);
        assert!(!shape_cast_hit(&r));
    }

    #[test]
    fn test_box_cast_hit() {
        let r = cast_box_phys([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], [0.5, 0.5, 0.5], 0.0);
        assert!(shape_cast_hit(&r));
    }

    #[test]
    fn test_capsule_cast_hit() {
        let r = cast_capsule_phys([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], 0.5, 1.0, 0.0);
        assert!(shape_cast_hit(&r));
    }

    #[test]
    fn test_distance() {
        let r = cast_sphere_phys([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], 0.5, 0.0);
        assert!(shape_cast_distance(&r) > 0.0);
    }

    #[test]
    fn test_normal() {
        let r = cast_sphere_phys([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], 0.5, 0.0);
        assert_eq!(shape_cast_normal(&r), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_point() {
        let r = cast_sphere_phys([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], 0.5, 0.0);
        assert!((shape_cast_point(&r)[1]).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let r = no_hit();
        let json = shape_cast_to_json(&r);
        assert!(json.contains("\"hit\":false"));
    }

    #[test]
    fn test_no_hit_result() {
        let r = no_hit();
        assert!(!shape_cast_hit(&r));
        assert_eq!(shape_cast_distance(&r), f32::INFINITY);
    }

    #[test]
    fn test_backward_ray() {
        let r = cast_sphere_phys([0.0, 5.0, 0.0], [0.0, 1.0, 0.0], 0.5, 0.0);
        assert!(!shape_cast_hit(&r));
    }
}
