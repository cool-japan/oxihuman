#![allow(dead_code)]

/// Debug sphere for visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderDebugSphere {
    center: [f32; 3],
    radius: f32,
    color: [f32; 3],
    segments: u32,
    visible: bool,
}

#[allow(dead_code)]
pub fn new_debug_sphere(center: [f32; 3], radius: f32) -> RenderDebugSphere {
    RenderDebugSphere { center, radius: radius.max(0.0), color: [1.0, 0.0, 0.0], segments: 16, visible: true }
}

#[allow(dead_code)]
pub fn sphere_center(s: &RenderDebugSphere) -> [f32; 3] { s.center }

#[allow(dead_code)]
pub fn sphere_radius_rds(s: &RenderDebugSphere) -> f32 { s.radius }

#[allow(dead_code)]
pub fn sphere_color(s: &RenderDebugSphere) -> [f32; 3] { s.color }

#[allow(dead_code)]
pub fn sphere_segments(s: &RenderDebugSphere) -> u32 { s.segments }

#[allow(dead_code)]
pub fn sphere_to_vertices(s: &RenderDebugSphere) -> Vec<[f32; 3]> {
    let mut verts = Vec::new();
    let seg = s.segments as usize;
    for i in 0..=seg {
        let theta = std::f32::consts::PI * i as f32 / seg as f32;
        for j in 0..=seg {
            let phi = std::f32::consts::TAU * j as f32 / seg as f32;
            verts.push([
                s.center[0] + s.radius * theta.sin() * phi.cos(),
                s.center[1] + s.radius * theta.cos(),
                s.center[2] + s.radius * theta.sin() * phi.sin(),
            ]);
        }
    }
    verts
}

#[allow(dead_code)]
pub fn sphere_to_json(s: &RenderDebugSphere) -> String {
    format!("{{\"center\":[{:.4},{:.4},{:.4}],\"radius\":{:.4},\"segments\":{}}}", s.center[0], s.center[1], s.center[2], s.radius, s.segments)
}

#[allow(dead_code)]
pub fn sphere_is_visible(s: &RenderDebugSphere) -> bool { s.visible }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let s = new_debug_sphere([0.0; 3], 1.0); assert!((sphere_radius_rds(&s) - 1.0).abs() < 1e-6); }
    #[test] fn test_center() { assert_eq!(sphere_center(&new_debug_sphere([1.0, 2.0, 3.0], 1.0)), [1.0, 2.0, 3.0]); }
    #[test] fn test_color() { assert!((sphere_color(&new_debug_sphere([0.0; 3], 1.0))[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_segments() { assert_eq!(sphere_segments(&new_debug_sphere([0.0; 3], 1.0)), 16); }
    #[test] fn test_visible() { assert!(sphere_is_visible(&new_debug_sphere([0.0; 3], 1.0))); }
    #[test] fn test_to_vertices() { assert!(!sphere_to_vertices(&new_debug_sphere([0.0; 3], 1.0)).is_empty()); }
    #[test] fn test_to_json() { assert!(sphere_to_json(&new_debug_sphere([0.0; 3], 1.0)).contains("radius")); }
    #[test] fn test_negative_radius() { assert!((sphere_radius_rds(&new_debug_sphere([0.0; 3], -1.0))).abs() < 1e-6); }
    #[test] fn test_zero_radius() {
        let s = new_debug_sphere([0.0; 3], 0.0);
        let v = sphere_to_vertices(&s);
        for p in &v { assert!((p[0]).abs() < 1e-6); }
    }
    #[test] fn test_vertex_count() {
        let s = new_debug_sphere([0.0; 3], 1.0);
        let v = sphere_to_vertices(&s);
        assert!(v.len() > 10);
    }
}
