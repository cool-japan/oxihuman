// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! HTML5 Canvas 2D drawing commands export.

/// A 2D canvas drawing command.
#[derive(Clone)]
pub enum Canvas2dCmd {
    BeginPath,
    MoveTo(f32, f32),
    LineTo(f32, f32),
    ClosePath,
    Stroke,
    Fill,
    SetStrokeStyle(String),
    SetFillStyle(String),
    SetLineWidth(f32),
    Arc(f32, f32, f32, f32, f32),
}

/// A canvas 2D export document.
pub struct Canvas2dExport {
    pub commands: Vec<Canvas2dCmd>,
    pub canvas_width: u32,
    pub canvas_height: u32,
}

/// Create a new Canvas 2D export.
pub fn new_canvas_2d_export(width: u32, height: u32) -> Canvas2dExport {
    Canvas2dExport {
        commands: Vec::new(),
        canvas_width: width,
        canvas_height: height,
    }
}

/// Append a command.
pub fn push_cmd(exp: &mut Canvas2dExport, cmd: Canvas2dCmd) {
    exp.commands.push(cmd);
}

/// Draw a line from (x0,y0) to (x1,y1).
pub fn draw_line(exp: &mut Canvas2dExport, x0: f32, y0: f32, x1: f32, y1: f32) {
    exp.commands.push(Canvas2dCmd::BeginPath);
    exp.commands.push(Canvas2dCmd::MoveTo(x0, y0));
    exp.commands.push(Canvas2dCmd::LineTo(x1, y1));
    exp.commands.push(Canvas2dCmd::Stroke);
}

/// Total command count.
pub fn command_count(exp: &Canvas2dExport) -> usize {
    exp.commands.len()
}

/// Render commands as a JavaScript snippet.
pub fn render_canvas_js(exp: &Canvas2dExport) -> String {
    let mut s = format!(
        "const canvas = document.createElement('canvas');\ncanvas.width={};\ncanvas.height={};\n",
        exp.canvas_width, exp.canvas_height
    );
    s.push_str("const ctx = canvas.getContext('2d');\n");
    for cmd in &exp.commands {
        let line = match cmd {
            Canvas2dCmd::BeginPath => "ctx.beginPath();".to_string(),
            Canvas2dCmd::MoveTo(x, y) => format!("ctx.moveTo({x},{y});"),
            Canvas2dCmd::LineTo(x, y) => format!("ctx.lineTo({x},{y});"),
            Canvas2dCmd::ClosePath => "ctx.closePath();".to_string(),
            Canvas2dCmd::Stroke => "ctx.stroke();".to_string(),
            Canvas2dCmd::Fill => "ctx.fill();".to_string(),
            Canvas2dCmd::SetStrokeStyle(c) => format!("ctx.strokeStyle='{c}';"),
            Canvas2dCmd::SetFillStyle(c) => format!("ctx.fillStyle='{c}';"),
            Canvas2dCmd::SetLineWidth(w) => format!("ctx.lineWidth={w};"),
            Canvas2dCmd::Arc(x, y, r, s, e) => format!("ctx.arc({x},{y},{r},{s},{e});"),
        };
        s.push_str(&line);
        s.push('\n');
    }
    s
}

/// Validate (non-zero canvas size).
pub fn validate_canvas_export(exp: &Canvas2dExport) -> bool {
    exp.canvas_width > 0 && exp.canvas_height > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_canvas_2d_export(800, 600);
        assert_eq!(command_count(&exp), 0 /* empty */);
    }

    #[test]
    fn draw_line_adds_four_cmds() {
        let mut exp = new_canvas_2d_export(100, 100);
        draw_line(&mut exp, 0.0, 0.0, 100.0, 100.0);
        assert_eq!(command_count(&exp), 4 /* begin, move, line, stroke */);
    }

    #[test]
    fn push_cmd_increments_count() {
        let mut exp = new_canvas_2d_export(100, 100);
        push_cmd(&mut exp, Canvas2dCmd::Fill);
        assert_eq!(command_count(&exp), 1 /* one command */);
    }

    #[test]
    fn render_contains_canvas_create() {
        let exp = new_canvas_2d_export(400, 300);
        let js = render_canvas_js(&exp);
        assert!(js.contains("getContext") /* canvas context */);
    }

    #[test]
    fn render_contains_width() {
        let exp = new_canvas_2d_export(1024, 768);
        let js = render_canvas_js(&exp);
        assert!(js.contains("1024") /* width in output */);
    }

    #[test]
    fn validate_valid_canvas() {
        let exp = new_canvas_2d_export(100, 100);
        assert!(validate_canvas_export(&exp) /* valid */);
    }

    #[test]
    fn validate_zero_size_fails() {
        let exp = new_canvas_2d_export(0, 100);
        assert!(!validate_canvas_export(&exp) /* invalid */);
    }

    #[test]
    fn stroke_style_cmd_in_output() {
        let mut exp = new_canvas_2d_export(100, 100);
        push_cmd(&mut exp, Canvas2dCmd::SetStrokeStyle("red".to_string()));
        let js = render_canvas_js(&exp);
        assert!(js.contains("red") /* stroke style */);
    }

    #[test]
    fn arc_cmd_in_output() {
        let mut exp = new_canvas_2d_export(100, 100);
        push_cmd(
            &mut exp,
            Canvas2dCmd::Arc(50.0, 50.0, 25.0, 0.0, std::f32::consts::TAU),
        );
        let js = render_canvas_js(&exp);
        assert!(js.contains("arc") /* arc command */);
    }
}
