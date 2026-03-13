// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Render mesh bounding box as an ASCII diagram.

#![allow(dead_code)]

/// Axis-aligned bounding box for ASCII rendering.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct AsciiAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl AsciiAabb {
    /// Create an AABB from min and max corners.
    #[allow(dead_code)]
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    /// Width along X.
    #[allow(dead_code)]
    pub fn width(&self) -> f32 {
        (self.max[0] - self.min[0]).max(0.0)
    }

    /// Height along Y.
    #[allow(dead_code)]
    pub fn height(&self) -> f32 {
        (self.max[1] - self.min[1]).max(0.0)
    }

    /// Depth along Z.
    #[allow(dead_code)]
    pub fn depth(&self) -> f32 {
        (self.max[2] - self.min[2]).max(0.0)
    }

    /// Volume of the bounding box.
    #[allow(dead_code)]
    pub fn volume(&self) -> f32 {
        self.width() * self.height() * self.depth()
    }
}

/// Compute AABB from a list of points.
#[allow(dead_code)]
pub fn compute_aabb(points: &[[f32; 3]]) -> Option<AsciiAabb> {
    if points.is_empty() {
        return None;
    }
    let mut mn = points[0];
    let mut mx = points[0];
    for p in points {
        for i in 0..3 {
            mn[i] = mn[i].min(p[i]);
            mx[i] = mx[i].max(p[i]);
        }
    }
    Some(AsciiAabb::new(mn, mx))
}

/// Render an ASCII top-down (XZ) projection of the bounding box.
/// `width_chars` and `height_chars` control diagram dimensions.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn render_aabb_ascii(aabb: &AsciiAabb, width_chars: usize, height_chars: usize) -> String {
    let w = width_chars.max(3);
    let h = height_chars.max(3);
    let mut grid: Vec<Vec<char>> = vec![vec![' '; w]; h];

    // Draw border rectangle
    for x in 0..w {
        grid[0][x] = '-';
        grid[h - 1][x] = '-';
    }
    for y in 0..h {
        grid[y][0] = '|';
        grid[y][w - 1] = '|';
    }
    // Corners
    grid[0][0] = '+';
    grid[0][w - 1] = '+';
    grid[h - 1][0] = '+';
    grid[h - 1][w - 1] = '+';

    // Label in centre
    let label = format!("{:.1}x{:.1}", aabb.width(), aabb.depth());
    let mid_y = h / 2;
    let start_x = if w > label.len() + 2 {
        (w - label.len()) / 2
    } else {
        1
    };
    for (i, ch) in label.chars().enumerate() {
        if start_x + i < w - 1 {
            grid[mid_y][start_x + i] = ch;
        }
    }

    let mut out = String::new();
    for row in &grid {
        for &ch in row {
            out.push(ch);
        }
        out.push('\n');
    }
    out
}

/// Render a simple side-view (XY) ASCII diagram.
#[allow(dead_code)]
pub fn render_side_view_ascii(aabb: &AsciiAabb, width_chars: usize, height_chars: usize) -> String {
    let w = width_chars.max(3);
    let h = height_chars.max(3);
    let mut out = String::new();
    for y in 0..h {
        if y == 0 || y == h - 1 {
            out.push('+');
            for _ in 0..w - 2 {
                out.push('-');
            }
            out.push('+');
        } else {
            out.push('|');
            if y == h / 2 {
                let label = format!("H={:.1}", aabb.height());
                let pad = (w - 2).saturating_sub(label.len()) / 2;
                for _ in 0..pad {
                    out.push(' ');
                }
                for ch in label.chars().take(w - 2) {
                    out.push(ch);
                }
                let used = pad + label.len().min(w - 2);
                for _ in used..w - 2 {
                    out.push(' ');
                }
            } else {
                for _ in 0..w - 2 {
                    out.push(' ');
                }
            }
            out.push('|');
        }
        out.push('\n');
    }
    out
}

/// Return a one-line summary of the bounding box.
#[allow(dead_code)]
pub fn aabb_summary(aabb: &AsciiAabb) -> String {
    format!(
        "AABB: [{:.2},{:.2},{:.2}] -> [{:.2},{:.2},{:.2}] ({}x{}x{})",
        aabb.min[0],
        aabb.min[1],
        aabb.min[2],
        aabb.max[0],
        aabb.max[1],
        aabb.max[2],
        aabb.width(),
        aabb.height(),
        aabb.depth()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_aabb() -> AsciiAabb {
        AsciiAabb::new([0.0, 0.0, 0.0], [2.0, 3.0, 1.5])
    }

    #[test]
    fn test_width() {
        let a = sample_aabb();
        assert!((a.width() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_height() {
        let a = sample_aabb();
        assert!((a.height() - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_depth() {
        let a = sample_aabb();
        assert!((a.depth() - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_volume() {
        let a = sample_aabb();
        assert!((a.volume() - 9.0).abs() < 1e-4);
    }

    #[test]
    fn test_compute_aabb_some() {
        let pts = vec![[0.0, 0.0, 0.0f32], [1.0, 2.0, 3.0]];
        assert!(compute_aabb(&pts).is_some());
    }

    #[test]
    fn test_compute_aabb_none_on_empty() {
        let pts: Vec<[f32; 3]> = vec![];
        assert!(compute_aabb(&pts).is_none());
    }

    #[test]
    fn test_render_aabb_ascii_has_corners() {
        let a = sample_aabb();
        let s = render_aabb_ascii(&a, 20, 8);
        assert!(s.contains('+'));
    }

    #[test]
    fn test_render_side_view_has_borders() {
        let a = sample_aabb();
        let s = render_side_view_ascii(&a, 20, 8);
        assert!(s.contains('|'));
    }

    #[test]
    fn test_aabb_summary_contains_aabb() {
        let a = sample_aabb();
        let s = aabb_summary(&a);
        assert!(s.contains("AABB"));
    }

    #[test]
    fn test_aabb_summary_contains_dimensions() {
        let a = sample_aabb();
        let s = aabb_summary(&a);
        assert!(s.contains("2") && s.contains("3"));
    }
}
