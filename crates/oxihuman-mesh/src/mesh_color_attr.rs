// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Vertex color attribute manipulation for meshes.

/// Vertex color buffer (RGBA float per vertex).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorAttr {
    pub colors: Vec<[f32; 4]>,
}

/// Create a new color attribute buffer filled with a uniform color.
#[allow(dead_code)]
pub fn new_color_attr(vertex_count: usize, rgba: [f32; 4]) -> ColorAttr {
    ColorAttr {
        colors: vec![rgba; vertex_count],
    }
}

/// Set color at a specific vertex.
#[allow(dead_code)]
pub fn set_color(attr: &mut ColorAttr, idx: usize, rgba: [f32; 4]) {
    if idx < attr.colors.len() {
        attr.colors[idx] = rgba;
    }
}

/// Get color at a specific vertex.
#[allow(dead_code)]
pub fn get_color(attr: &ColorAttr, idx: usize) -> Option<[f32; 4]> {
    attr.colors.get(idx).copied()
}

/// Vertex count.
#[allow(dead_code)]
pub fn color_vertex_count(attr: &ColorAttr) -> usize {
    attr.colors.len()
}

/// Lerp between two colors.
#[allow(dead_code)]
pub fn lerp_color(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

/// Apply gamma correction to all colors.
#[allow(dead_code)]
pub fn apply_gamma(attr: &mut ColorAttr, gamma: f32) {
    for c in &mut attr.colors {
        c[0] = c[0].powf(gamma);
        c[1] = c[1].powf(gamma);
        c[2] = c[2].powf(gamma);
    }
}

/// Clamp all color channels to [0, 1].
#[allow(dead_code)]
pub fn clamp_colors(attr: &mut ColorAttr) {
    for c in &mut attr.colors {
        c[0] = c[0].clamp(0.0, 1.0);
        c[1] = c[1].clamp(0.0, 1.0);
        c[2] = c[2].clamp(0.0, 1.0);
        c[3] = c[3].clamp(0.0, 1.0);
    }
}

/// Compute average color.
#[allow(dead_code)]
pub fn average_color(attr: &ColorAttr) -> [f32; 4] {
    if attr.colors.is_empty() {
        return [0.0; 4];
    }
    let mut sum = [0.0f32; 4];
    for c in &attr.colors {
        sum[0] += c[0];
        sum[1] += c[1];
        sum[2] += c[2];
        sum[3] += c[3];
    }
    let n = attr.colors.len() as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n, sum[3] / n]
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn color_attr_to_json(attr: &ColorAttr) -> String {
    let avg = average_color(attr);
    format!(
        "{{\"vertex_count\":{},\"avg_color\":[{:.4},{:.4},{:.4},{:.4}]}}",
        attr.colors.len(),
        avg[0],
        avg[1],
        avg[2],
        avg[3]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_color_attr() {
        let attr = new_color_attr(5, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(color_vertex_count(&attr), 5);
    }

    #[test]
    fn test_set_get_color() {
        let mut attr = new_color_attr(3, [0.0; 4]);
        set_color(&mut attr, 1, [0.5, 0.5, 0.5, 1.0]);
        let c = get_color(&attr, 1).unwrap();
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_get_oob() {
        let attr = new_color_attr(1, [0.0; 4]);
        assert!(get_color(&attr, 5).is_none());
    }

    #[test]
    fn test_lerp_color() {
        let c = lerp_color([0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 0.5);
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_gamma() {
        let mut attr = new_color_attr(1, [0.25, 0.25, 0.25, 1.0]);
        apply_gamma(&mut attr, 2.0);
        assert!((attr.colors[0][0] - 0.0625).abs() < 1e-5);
    }

    #[test]
    fn test_clamp_colors() {
        let mut attr = ColorAttr {
            colors: vec![[-0.5, 1.5, 0.5, 2.0]],
        };
        clamp_colors(&mut attr);
        assert!((attr.colors[0][0]).abs() < 1e-9);
        assert!((attr.colors[0][1] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_average_color() {
        let attr = ColorAttr {
            colors: vec![[0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]],
        };
        let avg = average_color(&attr);
        assert!((avg[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_average_empty() {
        let attr = ColorAttr { colors: vec![] };
        let avg = average_color(&attr);
        assert!((avg[0]).abs() < 1e-9);
    }

    #[test]
    fn test_to_json() {
        let attr = new_color_attr(2, [1.0, 0.0, 0.0, 1.0]);
        let j = color_attr_to_json(&attr);
        assert!(j.contains("\"vertex_count\":2"));
    }

    #[test]
    fn test_set_oob_no_panic() {
        let mut attr = new_color_attr(1, [0.0; 4]);
        set_color(&mut attr, 10, [1.0; 4]); // should not panic
    }
}
