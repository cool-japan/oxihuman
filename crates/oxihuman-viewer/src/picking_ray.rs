// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Object picking via ray cast from screen coordinates.

#![allow(dead_code)]

/// Configuration for object picking.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PickingConfig {
    pub max_distance: f32,
    pub pick_backfaces: bool,
}

/// A picking ray defined by origin and direction.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PickingRay {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

/// Result of a picking intersection test.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PickResult {
    pub hit: bool,
    pub object_id: u32,
    pub distance: f32,
    pub position: [f32; 3],
}

#[allow(dead_code)]
pub fn default_picking_config() -> PickingConfig {
    PickingConfig {
        max_distance: 1000.0,
        pick_backfaces: false,
    }
}

/// Construct a picking ray from normalized device coordinates.
#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn picking_ray_from_screen(
    screen_x: f32,
    screen_y: f32,
    viewport: [f32; 4],
    inv_proj_view: &[[f32; 4]; 4],
) -> PickingRay {
    // Convert screen to NDC
    let ndc_x = (screen_x - viewport[0]) / viewport[2] * 2.0 - 1.0;
    let ndc_y = 1.0 - (screen_y - viewport[1]) / viewport[3] * 2.0;

    // Near and far points in clip space
    let near_clip = [ndc_x, ndc_y, -1.0, 1.0];
    let far_clip = [ndc_x, ndc_y, 1.0, 1.0];

    let near_world = mat4_transform_point(inv_proj_view, near_clip);
    let far_world = mat4_transform_point(inv_proj_view, far_clip);

    let dir = normalize3(sub3(far_world, near_world));
    PickingRay { origin: near_world, direction: dir }
}

#[allow(dead_code)]
pub fn picking_test_sphere(
    ray: &PickingRay,
    center: [f32; 3],
    radius: f32,
    id: u32,
    config: &PickingConfig,
) -> PickResult {
    let oc = sub3(ray.origin, center);
    let a = dot3(ray.direction, ray.direction);
    let b = 2.0 * dot3(oc, ray.direction);
    let c = dot3(oc, oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return PickResult { hit: false, object_id: id, distance: f32::MAX, position: [0.0; 3] };
    }
    let t = (-b - discriminant.sqrt()) / (2.0 * a);
    if t < 0.0 || t > config.max_distance {
        return PickResult { hit: false, object_id: id, distance: f32::MAX, position: [0.0; 3] };
    }
    let pos = picking_ray_at(ray, t);
    PickResult { hit: true, object_id: id, distance: t, position: pos }
}

#[allow(dead_code)]
pub fn picking_closest(results: &[PickResult]) -> Option<&PickResult> {
    results.iter().filter(|r| r.hit).min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal))
}

#[allow(dead_code)]
pub fn picking_to_json(ray: &PickingRay) -> String {
    format!(
        r#"{{"origin":[{:.4},{:.4},{:.4}],"direction":[{:.4},{:.4},{:.4}]}}"#,
        ray.origin[0], ray.origin[1], ray.origin[2],
        ray.direction[0], ray.direction[1], ray.direction[2]
    )
}

#[allow(dead_code)]
pub fn picking_ray_at(ray: &PickingRay, t: f32) -> [f32; 3] {
    [
        ray.origin[0] + ray.direction[0] * t,
        ray.origin[1] + ray.direction[1] * t,
        ray.origin[2] + ray.direction[2] * t,
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l < 1e-9 {
        [0.0, 0.0, -1.0]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

fn mat4_transform_point(m: &[[f32; 4]; 4], p: [f32; 4]) -> [f32; 3] {
    let x = m[0][0] * p[0] + m[1][0] * p[1] + m[2][0] * p[2] + m[3][0] * p[3];
    let y = m[0][1] * p[0] + m[1][1] * p[1] + m[2][1] * p[2] + m[3][1] * p[3];
    let z = m[0][2] * p[0] + m[1][2] * p[1] + m[2][2] * p[2] + m[3][2] * p[3];
    let w = m[0][3] * p[0] + m[1][3] * p[1] + m[2][3] * p[2] + m[3][3] * p[3];
    let iw = if w.abs() > 1e-9 { 1.0 / w } else { 1.0 };
    [x * iw, y * iw, z * iw]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_inv() -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_picking_config();
        assert!((cfg.max_distance - 1000.0).abs() < 1e-6);
        assert!(!cfg.pick_backfaces);
    }

    #[test]
    fn test_ray_at() {
        let ray = PickingRay {
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, -1.0],
        };
        let p = picking_ray_at(&ray, 5.0);
        assert!((p[2] + 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_sphere_hit() {
        let cfg = default_picking_config();
        let ray = PickingRay {
            origin: [0.0, 0.0, 5.0],
            direction: [0.0, 0.0, -1.0],
        };
        let result = picking_test_sphere(&ray, [0.0, 0.0, 0.0], 0.5, 1, &cfg);
        assert!(result.hit);
    }

    #[test]
    fn test_sphere_miss() {
        let cfg = default_picking_config();
        let ray = PickingRay {
            origin: [5.0, 0.0, 5.0],
            direction: [0.0, 0.0, -1.0],
        };
        let result = picking_test_sphere(&ray, [0.0, 0.0, 0.0], 0.5, 1, &cfg);
        assert!(!result.hit);
    }

    #[test]
    fn test_closest_none_when_empty() {
        let results: Vec<PickResult> = Vec::new();
        assert!(picking_closest(&results).is_none());
    }

    #[test]
    fn test_closest_picks_nearest() {
        let r1 = PickResult { hit: true, object_id: 1, distance: 10.0, position: [0.0; 3] };
        let r2 = PickResult { hit: true, object_id: 2, distance: 5.0, position: [0.0; 3] };
        let results = vec![r1, r2];
        let closest = picking_closest(&results).expect("should succeed");
        assert_eq!(closest.object_id, 2);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let ray = PickingRay { origin: [1.0, 2.0, 3.0], direction: [0.0, 0.0, -1.0] };
        let j = picking_to_json(&ray);
        assert!(j.contains("origin"));
        assert!(j.contains("direction"));
    }

    #[test]
    fn test_ray_from_screen_runs() {
        let inv = identity_inv();
        let ray = picking_ray_from_screen(400.0, 300.0, [0.0, 0.0, 800.0, 600.0], &inv);
        // direction should be normalized (length ~1)
        let l = len3(ray.direction);
        assert!(l > 0.5);
    }
}
