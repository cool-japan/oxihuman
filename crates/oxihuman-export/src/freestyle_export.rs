// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Freestyle stroke export (SVG-based line rendering).

/// A single Freestyle stroke.
#[derive(Debug, Clone)]
pub struct FreestyleStroke {
    pub points: Vec<[f32; 2]>,
    pub color: [f32; 3],
    pub line_width: f32,
}

/// Create a new blank `FreestyleStroke`.
pub fn new_freestyle_stroke(color: [f32; 3], line_width: f32) -> FreestyleStroke {
    FreestyleStroke {
        points: Vec::new(),
        color,
        line_width,
    }
}

/// Add a 2D screen-space point.
pub fn freestyle_push_point(s: &mut FreestyleStroke, point: [f32; 2]) {
    s.points.push(point);
}

/// Polyline length of the stroke.
pub fn freestyle_stroke_length(s: &FreestyleStroke) -> f32 {
    if s.points.len() < 2 {
        return 0.0;
    }
    s.points
        .windows(2)
        .map(|w| {
            let dx = w[1][0] - w[0][0];
            let dy = w[1][1] - w[0][1];
            (dx * dx + dy * dy).sqrt()
        })
        .sum()
}

/// Serialize strokes to an SVG string.
pub fn freestyle_strokes_to_svg(strokes: &[FreestyleStroke], width: u32, height: u32) -> String {
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">",
        width, height
    );
    for stroke in strokes {
        if stroke.points.len() < 2 {
            continue;
        }
        let d: String = stroke
            .points
            .iter()
            .enumerate()
            .map(|(i, &p)| {
                if i == 0 {
                    format!("M {:.3} {:.3}", p[0], p[1])
                } else {
                    format!(" L {:.3} {:.3}", p[0], p[1])
                }
            })
            .collect();
        s.push_str(&format!(
            "<path d=\"{}\" stroke=\"#{:02x}{:02x}{:02x}\" stroke-width=\"{}\" fill=\"none\"/>",
            d,
            (stroke.color[0] * 255.0) as u8,
            (stroke.color[1] * 255.0) as u8,
            (stroke.color[2] * 255.0) as u8,
            stroke.line_width
        ));
    }
    s.push_str("</svg>");
    s
}

/// Number of strokes.
pub fn freestyle_stroke_count(strokes: &[FreestyleStroke]) -> usize {
    strokes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_freestyle_stroke() {
        let s = new_freestyle_stroke([0.0, 0.0, 0.0], 1.0);
        assert_eq!(s.points.len(), 0);
    }

    #[test]
    fn test_push_point() {
        let mut s = new_freestyle_stroke([1.0, 0.0, 0.0], 1.0);
        freestyle_push_point(&mut s, [0.0, 0.0]);
        freestyle_push_point(&mut s, [1.0, 0.0]);
        assert_eq!(s.points.len(), 2);
    }

    #[test]
    fn test_stroke_length() {
        let mut s = new_freestyle_stroke([1.0, 0.0, 0.0], 1.0);
        freestyle_push_point(&mut s, [0.0, 0.0]);
        freestyle_push_point(&mut s, [3.0, 4.0]);
        assert!((freestyle_stroke_length(&s) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_strokes_to_svg() {
        let mut s = new_freestyle_stroke([1.0, 0.0, 0.0], 2.0);
        freestyle_push_point(&mut s, [0.0, 0.0]);
        freestyle_push_point(&mut s, [100.0, 100.0]);
        let svg = freestyle_strokes_to_svg(&[s], 200, 200);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_freestyle_stroke_count() {
        let s = new_freestyle_stroke([0.0, 0.0, 0.0], 1.0);
        assert_eq!(freestyle_stroke_count(&[s]), 1);
    }
}
