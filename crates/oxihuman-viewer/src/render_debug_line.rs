#![allow(dead_code)]
//! Render debug line: a single debug line for visualization.

/// A debug line between two points.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderDebugLine {
    start: [f32; 3],
    end: [f32; 3],
    color: [f32; 4],
    visible: bool,
}

/// Create a new debug line.
#[allow(dead_code)]
pub fn new_render_debug_line() -> RenderDebugLine {
    RenderDebugLine {
        start: [0.0; 3],
        end: [0.0; 3],
        color: [1.0, 1.0, 1.0, 1.0],
        visible: true,
    }
}

/// Set the start point.
#[allow(dead_code)]
pub fn set_start(line: &mut RenderDebugLine, start: [f32; 3]) {
    line.start = start;
}

/// Set the end point.
#[allow(dead_code)]
pub fn set_end(line: &mut RenderDebugLine, end: [f32; 3]) {
    line.end = end;
}

/// Return the length of the line.
#[allow(dead_code)]
pub fn line_length(line: &RenderDebugLine) -> f32 {
    let dx = line.end[0] - line.start[0];
    let dy = line.end[1] - line.start[1];
    let dz = line.end[2] - line.start[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Return the line color.
#[allow(dead_code)]
pub fn line_color(line: &RenderDebugLine) -> [f32; 4] {
    line.color
}

/// Convert the line to vertex data: [start_x, start_y, start_z, end_x, end_y, end_z].
#[allow(dead_code)]
pub fn line_to_vertices(line: &RenderDebugLine) -> [f32; 6] {
    [
        line.start[0], line.start[1], line.start[2],
        line.end[0], line.end[1], line.end[2],
    ]
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn line_to_json(line: &RenderDebugLine) -> String {
    format!(
        "{{\"start\":[{},{},{}],\"end\":[{},{},{}],\"visible\":{}}}",
        line.start[0], line.start[1], line.start[2],
        line.end[0], line.end[1], line.end[2],
        line.visible
    )
}

/// Check if the line is visible.
#[allow(dead_code)]
pub fn line_is_visible(line: &RenderDebugLine) -> bool {
    line.visible
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_line() {
        let l = new_render_debug_line();
        assert!(line_is_visible(&l));
    }

    #[test]
    fn test_set_start() {
        let mut l = new_render_debug_line();
        set_start(&mut l, [1.0, 2.0, 3.0]);
        assert!((l.start[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_end() {
        let mut l = new_render_debug_line();
        set_end(&mut l, [4.0, 5.0, 6.0]);
        assert!((l.end[2] - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_length_zero() {
        let l = new_render_debug_line();
        assert!((line_length(&l) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_length() {
        let mut l = new_render_debug_line();
        set_start(&mut l, [0.0, 0.0, 0.0]);
        set_end(&mut l, [3.0, 4.0, 0.0]);
        assert!((line_length(&l) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_color() {
        let l = new_render_debug_line();
        let c = line_color(&l);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_vertices() {
        let mut l = new_render_debug_line();
        set_start(&mut l, [1.0, 2.0, 3.0]);
        set_end(&mut l, [4.0, 5.0, 6.0]);
        let v = line_to_vertices(&l);
        assert!((v[0] - 1.0).abs() < 1e-6);
        assert!((v[3] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let l = new_render_debug_line();
        let json = line_to_json(&l);
        assert!(json.contains("\"visible\":true"));
    }

    #[test]
    fn test_visibility() {
        let mut l = new_render_debug_line();
        l.visible = false;
        assert!(!line_is_visible(&l));
    }

    #[test]
    fn test_unit_length() {
        let mut l = new_render_debug_line();
        set_end(&mut l, [1.0, 0.0, 0.0]);
        assert!((line_length(&l) - 1.0).abs() < 1e-6);
    }
}
