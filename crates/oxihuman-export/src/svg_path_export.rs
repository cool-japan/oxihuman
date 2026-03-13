// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! SVG path data export (M/L/C/Z commands).

/// A single SVG path command.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SvgPathCmd {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    CubicBezier(f32, f32, f32, f32, f32, f32),
    ClosePath,
}

/// An SVG path element (collection of commands + style).
#[allow(dead_code)]
pub struct SvgPathElement {
    pub commands: Vec<SvgPathCmd>,
    pub stroke: String,
    pub stroke_width: f32,
    pub fill: String,
}

/// Create a new empty path element.
#[allow(dead_code)]
pub fn new_svg_path(stroke: &str, stroke_width: f32, fill: &str) -> SvgPathElement {
    SvgPathElement {
        commands: Vec::new(),
        stroke: stroke.to_string(),
        stroke_width,
        fill: fill.to_string(),
    }
}

/// Append a MoveTo command.
#[allow(dead_code)]
pub fn move_to(path: &mut SvgPathElement, x: f32, y: f32) {
    path.commands.push(SvgPathCmd::MoveTo(x, y));
}

/// Append a LineTo command.
#[allow(dead_code)]
pub fn line_to(path: &mut SvgPathElement, x: f32, y: f32) {
    path.commands.push(SvgPathCmd::LineTo(x, y));
}

/// Append a cubic Bezier command.
#[allow(dead_code)]
pub fn cubic_to(path: &mut SvgPathElement, cx1: f32, cy1: f32, cx2: f32, cy2: f32, x: f32, y: f32) {
    path.commands
        .push(SvgPathCmd::CubicBezier(cx1, cy1, cx2, cy2, x, y));
}

/// Append a ClosePath command.
#[allow(dead_code)]
pub fn close_path(path: &mut SvgPathElement) {
    path.commands.push(SvgPathCmd::ClosePath);
}

/// Serialize commands to SVG path `d` attribute string.
#[allow(dead_code)]
pub fn commands_to_d(commands: &[SvgPathCmd]) -> String {
    commands
        .iter()
        .map(|cmd| match cmd {
            SvgPathCmd::MoveTo(x, y) => format!("M {:.4} {:.4}", x, y),
            SvgPathCmd::LineTo(x, y) => format!("L {:.4} {:.4}", x, y),
            SvgPathCmd::CubicBezier(cx1, cy1, cx2, cy2, x, y) => format!(
                "C {:.4} {:.4} {:.4} {:.4} {:.4} {:.4}",
                cx1, cy1, cx2, cy2, x, y
            ),
            SvgPathCmd::ClosePath => "Z".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Serialize an SVG path element to an `<path .../>` tag.
#[allow(dead_code)]
pub fn path_to_svg_tag(path: &SvgPathElement) -> String {
    let d = commands_to_d(&path.commands);
    format!(
        "<path d=\"{}\" stroke=\"{}\" stroke-width=\"{}\" fill=\"{}\"/>",
        d, path.stroke, path.stroke_width, path.fill
    )
}

/// Wrap path tags in a minimal SVG document.
#[allow(dead_code)]
pub fn wrap_svg(width: f32, height: f32, body: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">{}</svg>",
        width, height, body
    )
}

/// Count commands in a path.
#[allow(dead_code)]
pub fn command_count(path: &SvgPathElement) -> usize {
    path.commands.len()
}

/// Check if path starts with a MoveTo.
#[allow(dead_code)]
pub fn starts_with_move(path: &SvgPathElement) -> bool {
    path.commands
        .first()
        .is_some_and(|c| matches!(c, SvgPathCmd::MoveTo(..)))
}

/// Convert a polyline to a path.
#[allow(dead_code)]
pub fn polyline_to_path(points: &[[f32; 2]], stroke: &str, stroke_width: f32) -> SvgPathElement {
    let mut path = new_svg_path(stroke, stroke_width, "none");
    for (i, &[x, y]) in points.iter().enumerate() {
        if i == 0 {
            move_to(&mut path, x, y);
        } else {
            line_to(&mut path, x, y);
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_to_adds_command() {
        let mut p = new_svg_path("black", 1.0, "none");
        move_to(&mut p, 10.0, 20.0);
        assert_eq!(command_count(&p), 1);
    }

    #[test]
    fn commands_to_d_move() {
        let d = commands_to_d(&[SvgPathCmd::MoveTo(1.0, 2.0)]);
        assert!(d.contains('M'));
    }

    #[test]
    fn commands_to_d_line() {
        let d = commands_to_d(&[SvgPathCmd::LineTo(3.0, 4.0)]);
        assert!(d.contains('L'));
    }

    #[test]
    fn commands_to_d_close() {
        let d = commands_to_d(&[SvgPathCmd::ClosePath]);
        assert!(d.contains('Z'));
    }

    #[test]
    fn commands_to_d_cubic() {
        let d = commands_to_d(&[SvgPathCmd::CubicBezier(1.0, 2.0, 3.0, 4.0, 5.0, 6.0)]);
        assert!(d.contains('C'));
    }

    #[test]
    fn path_tag_has_stroke() {
        let mut p = new_svg_path("red", 2.0, "none");
        move_to(&mut p, 0.0, 0.0);
        let tag = path_to_svg_tag(&p);
        assert!(tag.contains("red"));
    }

    #[test]
    fn wrap_svg_has_namespace() {
        let svg = wrap_svg(100.0, 200.0, "");
        assert!(svg.contains("xmlns"));
    }

    #[test]
    fn starts_with_move_true() {
        let mut p = new_svg_path("black", 1.0, "none");
        move_to(&mut p, 0.0, 0.0);
        assert!(starts_with_move(&p));
    }

    #[test]
    fn starts_with_move_false_empty() {
        let p = new_svg_path("black", 1.0, "none");
        assert!(!starts_with_move(&p));
    }

    #[test]
    fn polyline_to_path_count() {
        let pts = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
        let p = polyline_to_path(&pts, "blue", 1.0);
        assert_eq!(command_count(&p), 3);
    }
}
