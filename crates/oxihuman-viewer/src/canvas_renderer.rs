// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! 2D canvas renderer for overlay drawing.

/// Canvas draw command.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum CanvasCommand {
    Line { x0: f32, y0: f32, x1: f32, y1: f32 },
    Rect { x: f32, y: f32, w: f32, h: f32 },
    Circle { cx: f32, cy: f32, radius: f32 },
    Text { x: f32, y: f32, content: String },
}

/// Canvas state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CanvasState {
    pub commands: Vec<CanvasCommand>,
    pub color: [f32; 4],
    pub line_width: f32,
}

#[allow(dead_code)]
pub fn new_canvas() -> CanvasState {
    CanvasState { commands: Vec::new(), color: [1.0, 1.0, 1.0, 1.0], line_width: 1.0 }
}

#[allow(dead_code)]
pub fn set_canvas_color(canvas: &mut CanvasState, r: f32, g: f32, b: f32, a: f32) {
    canvas.color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), a.clamp(0.0, 1.0)];
}

#[allow(dead_code)]
pub fn set_line_width(canvas: &mut CanvasState, width: f32) {
    canvas.line_width = width.max(0.1);
}

#[allow(dead_code)]
pub fn draw_line(canvas: &mut CanvasState, x0: f32, y0: f32, x1: f32, y1: f32) {
    canvas.commands.push(CanvasCommand::Line { x0, y0, x1, y1 });
}

#[allow(dead_code)]
pub fn draw_rect(canvas: &mut CanvasState, x: f32, y: f32, w: f32, h: f32) {
    canvas.commands.push(CanvasCommand::Rect { x, y, w, h });
}

#[allow(dead_code)]
pub fn draw_circle(canvas: &mut CanvasState, cx: f32, cy: f32, radius: f32) {
    canvas.commands.push(CanvasCommand::Circle { cx, cy, radius });
}

#[allow(dead_code)]
pub fn draw_text(canvas: &mut CanvasState, x: f32, y: f32, content: &str) {
    canvas.commands.push(CanvasCommand::Text { x, y, content: content.to_string() });
}

#[allow(dead_code)]
pub fn clear_canvas(canvas: &mut CanvasState) {
    canvas.commands.clear();
}

#[allow(dead_code)]
pub fn canvas_command_count(canvas: &CanvasState) -> usize {
    canvas.commands.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_canvas() {
        let c = new_canvas();
        assert!(c.commands.is_empty());
    }

    #[test]
    fn test_set_color() {
        let mut c = new_canvas();
        set_canvas_color(&mut c, 0.5, 0.5, 0.5, 1.0);
        assert!((c.color[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_line_width() {
        let mut c = new_canvas();
        set_line_width(&mut c, 3.0);
        assert!((c.line_width - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_draw_line() {
        let mut c = new_canvas();
        draw_line(&mut c, 0.0, 0.0, 100.0, 100.0);
        assert_eq!(canvas_command_count(&c), 1);
    }

    #[test]
    fn test_draw_rect() {
        let mut c = new_canvas();
        draw_rect(&mut c, 10.0, 10.0, 50.0, 50.0);
        assert_eq!(canvas_command_count(&c), 1);
    }

    #[test]
    fn test_draw_circle() {
        let mut c = new_canvas();
        draw_circle(&mut c, 50.0, 50.0, 25.0);
        assert_eq!(canvas_command_count(&c), 1);
    }

    #[test]
    fn test_draw_text() {
        let mut c = new_canvas();
        draw_text(&mut c, 10.0, 10.0, "hello");
        assert_eq!(canvas_command_count(&c), 1);
    }

    #[test]
    fn test_clear_canvas() {
        let mut c = new_canvas();
        draw_line(&mut c, 0.0, 0.0, 1.0, 1.0);
        clear_canvas(&mut c);
        assert!(c.commands.is_empty());
    }

    #[test]
    fn test_multiple_commands() {
        let mut c = new_canvas();
        draw_line(&mut c, 0.0, 0.0, 1.0, 1.0);
        draw_rect(&mut c, 0.0, 0.0, 10.0, 10.0);
        draw_circle(&mut c, 5.0, 5.0, 3.0);
        assert_eq!(canvas_command_count(&c), 3);
    }

    #[test]
    fn test_color_clamp() {
        let mut c = new_canvas();
        set_canvas_color(&mut c, 2.0, -1.0, 0.5, 0.5);
        assert!((c.color[0] - 1.0).abs() < 1e-6);
        assert!(c.color[1].abs() < 1e-6);
    }
}
