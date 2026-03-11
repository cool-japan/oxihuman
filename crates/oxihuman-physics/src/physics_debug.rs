// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A debug line segment.
#[allow(dead_code)]
pub struct PhysicsDebugLine {
    pub start: [f32; 3],
    pub end_: [f32; 3],
    pub color: [f32; 4],
}

/// A debug sphere.
#[allow(dead_code)]
pub struct PhysicsDebugSphere {
    pub center: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
}

/// Container for debug draw primitives.
#[allow(dead_code)]
pub struct PhysicsDebugDraw {
    pub lines: Vec<PhysicsDebugLine>,
    pub spheres: Vec<PhysicsDebugSphere>,
}

/// Create a new empty `PhysicsDebugDraw`.
#[allow(dead_code)]
pub fn new_physics_debug() -> PhysicsDebugDraw {
    PhysicsDebugDraw {
        lines: Vec::new(),
        spheres: Vec::new(),
    }
}

/// Add a line segment.
#[allow(dead_code)]
pub fn draw_line(dd: &mut PhysicsDebugDraw, start: [f32; 3], end_: [f32; 3], color: [f32; 4]) {
    dd.lines.push(PhysicsDebugLine { start, end_, color });
}

/// Add a sphere.
#[allow(dead_code)]
pub fn draw_sphere(dd: &mut PhysicsDebugDraw, center: [f32; 3], radius: f32, color: [f32; 4]) {
    dd.spheres.push(PhysicsDebugSphere { center, radius, color });
}

/// Add an AABB as 12 edge lines.
#[allow(dead_code)]
pub fn draw_aabb(dd: &mut PhysicsDebugDraw, min: [f32; 3], max: [f32; 3], color: [f32; 4]) {
    let [x0, y0, z0] = min;
    let [x1, y1, z1] = max;
    // Bottom face edges
    draw_line(dd, [x0, y0, z0], [x1, y0, z0], color);
    draw_line(dd, [x1, y0, z0], [x1, y0, z1], color);
    draw_line(dd, [x1, y0, z1], [x0, y0, z1], color);
    draw_line(dd, [x0, y0, z1], [x0, y0, z0], color);
    // Top face edges
    draw_line(dd, [x0, y1, z0], [x1, y1, z0], color);
    draw_line(dd, [x1, y1, z0], [x1, y1, z1], color);
    draw_line(dd, [x1, y1, z1], [x0, y1, z1], color);
    draw_line(dd, [x0, y1, z1], [x0, y1, z0], color);
    // Vertical edges
    draw_line(dd, [x0, y0, z0], [x0, y1, z0], color);
    draw_line(dd, [x1, y0, z0], [x1, y1, z0], color);
    draw_line(dd, [x1, y0, z1], [x1, y1, z1], color);
    draw_line(dd, [x0, y0, z1], [x0, y1, z1], color);
}

/// Clear all debug primitives.
#[allow(dead_code)]
pub fn clear_debug(dd: &mut PhysicsDebugDraw) {
    dd.lines.clear();
    dd.spheres.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_debug_is_empty() {
        let dd = new_physics_debug();
        assert!(dd.lines.is_empty());
        assert!(dd.spheres.is_empty());
    }

    #[test]
    fn draw_line_adds_one_line() {
        let mut dd = new_physics_debug();
        draw_line(&mut dd, [0.0; 3], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(dd.lines.len(), 1);
    }

    #[test]
    fn draw_sphere_adds_one_sphere() {
        let mut dd = new_physics_debug();
        draw_sphere(&mut dd, [0.0; 3], 1.0, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(dd.spheres.len(), 1);
    }

    #[test]
    fn draw_aabb_adds_12_lines() {
        let mut dd = new_physics_debug();
        draw_aabb(&mut dd, [0.0; 3], [1.0; 3], [1.0; 4]);
        assert_eq!(dd.lines.len(), 12);
    }

    #[test]
    fn clear_debug_removes_all() {
        let mut dd = new_physics_debug();
        draw_line(&mut dd, [0.0; 3], [1.0; 3], [1.0; 4]);
        draw_sphere(&mut dd, [0.0; 3], 1.0, [1.0; 4]);
        clear_debug(&mut dd);
        assert!(dd.lines.is_empty());
        assert!(dd.spheres.is_empty());
    }

    #[test]
    fn line_color_stored() {
        let mut dd = new_physics_debug();
        let color = [0.5, 0.25, 0.1, 1.0];
        draw_line(&mut dd, [0.0; 3], [1.0; 3], color);
        assert_eq!(dd.lines[0].color, color);
    }

    #[test]
    fn sphere_radius_stored() {
        let mut dd = new_physics_debug();
        draw_sphere(&mut dd, [0.0; 3], 2.5, [1.0; 4]);
        assert!((dd.spheres[0].radius - 2.5).abs() < 1e-5);
    }

    #[test]
    fn multiple_primitives_accumulated() {
        let mut dd = new_physics_debug();
        for _ in 0..5 {
            draw_line(&mut dd, [0.0; 3], [1.0; 3], [1.0; 4]);
            draw_sphere(&mut dd, [0.0; 3], 1.0, [1.0; 4]);
        }
        assert_eq!(dd.lines.len(), 5);
        assert_eq!(dd.spheres.len(), 5);
    }

    #[test]
    fn draw_aabb_after_clear() {
        let mut dd = new_physics_debug();
        draw_aabb(&mut dd, [0.0; 3], [1.0; 3], [1.0; 4]);
        clear_debug(&mut dd);
        assert_eq!(dd.lines.len(), 0);
    }

    #[test]
    fn line_start_end_stored() {
        let mut dd = new_physics_debug();
        let start = [1.0, 2.0, 3.0];
        let end = [4.0, 5.0, 6.0];
        draw_line(&mut dd, start, end, [1.0; 4]);
        assert_eq!(dd.lines[0].start, start);
        assert_eq!(dd.lines[0].end_, end);
    }
}
