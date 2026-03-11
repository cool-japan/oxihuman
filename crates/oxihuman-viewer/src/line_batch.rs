// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Line batch renderer for efficient line drawing.

/// A line vertex with position and color.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct LineVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

/// Line batch state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LineBatch {
    pub vertices: Vec<LineVertex>,
    pub line_width: f32,
    pub depth_test: bool,
}

#[allow(dead_code)]
pub fn new_line_batch() -> LineBatch {
    LineBatch { vertices: Vec::new(), line_width: 1.0, depth_test: true }
}

#[allow(dead_code)]
pub fn new_line_vertex(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32, a: f32) -> LineVertex {
    LineVertex { position: [x, y, z], color: [r, g, b, a] }
}

#[allow(dead_code)]
pub fn add_line(batch: &mut LineBatch, start: LineVertex, end: LineVertex) {
    batch.vertices.push(start);
    batch.vertices.push(end);
}

#[allow(dead_code)]
pub fn set_batch_line_width(batch: &mut LineBatch, width: f32) {
    batch.line_width = width.max(0.1);
}

#[allow(dead_code)]
pub fn set_batch_depth_test(batch: &mut LineBatch, enabled: bool) {
    batch.depth_test = enabled;
}

#[allow(dead_code)]
pub fn line_count(batch: &LineBatch) -> usize {
    batch.vertices.len() / 2
}

#[allow(dead_code)]
pub fn vertex_count(batch: &LineBatch) -> usize {
    batch.vertices.len()
}

#[allow(dead_code)]
pub fn clear_batch(batch: &mut LineBatch) {
    batch.vertices.clear();
}

#[allow(dead_code)]
pub fn add_axis_lines(batch: &mut LineBatch, length: f32) {
    let o = [0.0, 0.0, 0.0];
    add_line(batch,
        LineVertex { position: o, color: [1.0, 0.0, 0.0, 1.0] },
        LineVertex { position: [length, 0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
    );
    add_line(batch,
        LineVertex { position: o, color: [0.0, 1.0, 0.0, 1.0] },
        LineVertex { position: [0.0, length, 0.0], color: [0.0, 1.0, 0.0, 1.0] },
    );
    add_line(batch,
        LineVertex { position: o, color: [0.0, 0.0, 1.0, 1.0] },
        LineVertex { position: [0.0, 0.0, length], color: [0.0, 0.0, 1.0, 1.0] },
    );
}

#[allow(dead_code)]
pub fn is_batch_empty(batch: &LineBatch) -> bool {
    batch.vertices.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_batch() {
        let b = new_line_batch();
        assert!(is_batch_empty(&b));
    }

    #[test]
    fn test_add_line() {
        let mut b = new_line_batch();
        let s = new_line_vertex(0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0);
        let e = new_line_vertex(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
        add_line(&mut b, s, e);
        assert_eq!(line_count(&b), 1);
        assert_eq!(vertex_count(&b), 2);
    }

    #[test]
    fn test_set_line_width() {
        let mut b = new_line_batch();
        set_batch_line_width(&mut b, 3.0);
        assert!((b.line_width - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_test() {
        let mut b = new_line_batch();
        set_batch_depth_test(&mut b, false);
        assert!(!b.depth_test);
    }

    #[test]
    fn test_clear_batch() {
        let mut b = new_line_batch();
        let v = new_line_vertex(0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0);
        add_line(&mut b, v, v);
        clear_batch(&mut b);
        assert!(is_batch_empty(&b));
    }

    #[test]
    fn test_add_axis_lines() {
        let mut b = new_line_batch();
        add_axis_lines(&mut b, 1.0);
        assert_eq!(line_count(&b), 3);
    }

    #[test]
    fn test_multiple_lines() {
        let mut b = new_line_batch();
        let v = new_line_vertex(0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0);
        add_line(&mut b, v, v);
        add_line(&mut b, v, v);
        assert_eq!(line_count(&b), 2);
    }

    #[test]
    fn test_vertex_position() {
        let v = new_line_vertex(1.0, 2.0, 3.0, 0.5, 0.5, 0.5, 1.0);
        assert!((v.position[0] - 1.0).abs() < 1e-6);
        assert!((v.color[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_line_width_min() {
        let mut b = new_line_batch();
        set_batch_line_width(&mut b, -1.0);
        assert!((b.line_width - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_axis_lines_vertices() {
        let mut b = new_line_batch();
        add_axis_lines(&mut b, 5.0);
        assert_eq!(vertex_count(&b), 6);
    }
}
