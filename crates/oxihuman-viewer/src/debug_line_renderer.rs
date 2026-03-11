#![allow(dead_code)]
//! Debug line rendering for visualization.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct DebugLine {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 4],
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct DebugLineRenderer {
    lines: Vec<DebugLine>,
}

#[allow(dead_code)]
pub fn new_debug_line_renderer() -> DebugLineRenderer {
    DebugLineRenderer { lines: Vec::new() }
}

#[allow(dead_code)]
pub fn add_debug_line(r: &mut DebugLineRenderer, start: [f32; 3], end: [f32; 3], color: [f32; 4]) {
    r.lines.push(DebugLine { start, end, color });
}

#[allow(dead_code)]
pub fn add_debug_box(r: &mut DebugLineRenderer, min: [f32; 3], max: [f32; 3], color: [f32; 4]) {
    let corners = [
        [min[0], min[1], min[2]], [max[0], min[1], min[2]],
        [max[0], max[1], min[2]], [min[0], max[1], min[2]],
        [min[0], min[1], max[2]], [max[0], min[1], max[2]],
        [max[0], max[1], max[2]], [min[0], max[1], max[2]],
    ];
    let edges = [
        (0, 1), (1, 2), (2, 3), (3, 0),
        (4, 5), (5, 6), (6, 7), (7, 4),
        (0, 4), (1, 5), (2, 6), (3, 7),
    ];
    for (a, b) in edges {
        add_debug_line(r, corners[a], corners[b], color);
    }
}

#[allow(dead_code)]
pub fn add_debug_sphere_lines(r: &mut DebugLineRenderer, center: [f32; 3], radius: f32, color: [f32; 4]) {
    use std::f32::consts::TAU;
    let segments = 16;
    for i in 0..segments {
        let a0 = TAU * (i as f32) / (segments as f32);
        let a1 = TAU * ((i + 1) as f32) / (segments as f32);
        // XY circle
        add_debug_line(
            r,
            [center[0] + radius * a0.cos(), center[1] + radius * a0.sin(), center[2]],
            [center[0] + radius * a1.cos(), center[1] + radius * a1.sin(), center[2]],
            color,
        );
        // XZ circle
        add_debug_line(
            r,
            [center[0] + radius * a0.cos(), center[1], center[2] + radius * a0.sin()],
            [center[0] + radius * a1.cos(), center[1], center[2] + radius * a1.sin()],
            color,
        );
    }
}

#[allow(dead_code)]
pub fn debug_line_count(r: &DebugLineRenderer) -> usize {
    r.lines.len()
}

#[allow(dead_code)]
pub fn clear_debug_lines(r: &mut DebugLineRenderer) {
    r.lines.clear();
}

#[allow(dead_code)]
pub fn debug_lines_to_vertices(r: &DebugLineRenderer) -> Vec<[f32; 3]> {
    let mut verts = Vec::with_capacity(r.lines.len() * 2);
    for l in &r.lines {
        verts.push(l.start);
        verts.push(l.end);
    }
    verts
}

#[allow(dead_code)]
pub fn debug_renderer_reset(r: &mut DebugLineRenderer) {
    r.lines.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_debug_line_renderer() {
        let r = new_debug_line_renderer();
        assert_eq!(debug_line_count(&r), 0);
    }

    #[test]
    fn test_add_debug_line() {
        let mut r = new_debug_line_renderer();
        add_debug_line(&mut r, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(debug_line_count(&r), 1);
    }

    #[test]
    fn test_add_debug_box() {
        let mut r = new_debug_line_renderer();
        add_debug_box(&mut r, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(debug_line_count(&r), 12);
    }

    #[test]
    fn test_add_debug_sphere_lines() {
        let mut r = new_debug_line_renderer();
        add_debug_sphere_lines(&mut r, [0.0, 0.0, 0.0], 1.0, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(debug_line_count(&r), 32);
    }

    #[test]
    fn test_clear_debug_lines() {
        let mut r = new_debug_line_renderer();
        add_debug_line(&mut r, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
        clear_debug_lines(&mut r);
        assert_eq!(debug_line_count(&r), 0);
    }

    #[test]
    fn test_debug_lines_to_vertices() {
        let mut r = new_debug_line_renderer();
        add_debug_line(&mut r, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
        let v = debug_lines_to_vertices(&r);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_debug_renderer_reset() {
        let mut r = new_debug_line_renderer();
        add_debug_line(&mut r, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
        debug_renderer_reset(&mut r);
        assert_eq!(debug_line_count(&r), 0);
    }

    #[test]
    fn test_empty_vertices() {
        let r = new_debug_line_renderer();
        assert!(debug_lines_to_vertices(&r).is_empty());
    }

    #[test]
    fn test_multiple_lines() {
        let mut r = new_debug_line_renderer();
        for _ in 0..10 {
            add_debug_line(&mut r, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
        }
        assert_eq!(debug_line_count(&r), 10);
    }

    #[test]
    fn test_vertices_count() {
        let mut r = new_debug_line_renderer();
        add_debug_line(&mut r, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
        add_debug_line(&mut r, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0, 1.0]);
        let v = debug_lines_to_vertices(&r);
        assert_eq!(v.len(), 4);
    }
}
