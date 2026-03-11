#![allow(dead_code)]

use std::collections::HashMap;

/// RGBA vertex color.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertexColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Map from vertex index to color.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexColorMap {
    pub colors: HashMap<usize, VertexColor>,
}

/// Create a new empty vertex color map.
#[allow(dead_code)]
pub fn new_vertex_color_map() -> VertexColorMap {
    VertexColorMap {
        colors: HashMap::new(),
    }
}

/// Set a vertex color.
#[allow(dead_code)]
pub fn set_vertex_color(map: &mut VertexColorMap, index: usize, color: VertexColor) {
    map.colors.insert(index, color);
}

/// Get a vertex color, returning white if not set.
#[allow(dead_code)]
pub fn get_vertex_color(map: &VertexColorMap, index: usize) -> VertexColor {
    map.colors.get(&index).copied().unwrap_or(VertexColor {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    })
}

/// Blend two vertex colors with a factor (0 = a, 1 = b).
#[allow(dead_code)]
pub fn blend_vertex_colors(a: &VertexColor, b: &VertexColor, factor: f32) -> VertexColor {
    let f = factor.clamp(0.0, 1.0);
    VertexColor {
        r: a.r + (b.r - a.r) * f,
        g: a.g + (b.g - a.g) * f,
        b: a.b + (b.b - a.b) * f,
        a: a.a + (b.a - a.a) * f,
    }
}

/// Count how many vertices have colors assigned.
#[allow(dead_code)]
pub fn vertex_color_count(map: &VertexColorMap) -> usize {
    map.colors.len()
}

/// Clear all vertex colors.
#[allow(dead_code)]
pub fn clear_vertex_colors(map: &mut VertexColorMap) {
    map.colors.clear();
}

/// Convert a vertex color to an RGBA u8 array.
#[allow(dead_code)]
pub fn color_to_rgba(c: &VertexColor) -> [u8; 4] {
    [
        (c.r.clamp(0.0, 1.0) * 255.0) as u8,
        (c.g.clamp(0.0, 1.0) * 255.0) as u8,
        (c.b.clamp(0.0, 1.0) * 255.0) as u8,
        (c.a.clamp(0.0, 1.0) * 255.0) as u8,
    ]
}

/// Convert all vertex colors to a byte buffer (sorted by index).
#[allow(dead_code)]
pub fn vertex_colors_to_bytes(map: &VertexColorMap) -> Vec<u8> {
    let mut indices: Vec<usize> = map.colors.keys().copied().collect();
    indices.sort_unstable();
    let mut bytes = Vec::with_capacity(indices.len() * 4);
    for idx in indices {
        let c = map.colors[&idx];
        let rgba = color_to_rgba(&c);
        bytes.extend_from_slice(&rgba);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map_empty() {
        let m = new_vertex_color_map();
        assert_eq!(vertex_color_count(&m), 0);
    }

    #[test]
    fn test_set_get_color() {
        let mut m = new_vertex_color_map();
        let c = VertexColor {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        set_vertex_color(&mut m, 0, c);
        let got = get_vertex_color(&m, 0);
        assert!((got.r - 1.0).abs() < 1e-6);
        assert!((got.g - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_get_default_color() {
        let m = new_vertex_color_map();
        let c = get_vertex_color(&m, 99);
        assert!((c.r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_colors() {
        let a = VertexColor {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let b = VertexColor {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        let mid = blend_vertex_colors(&a, &b, 0.5);
        assert!((mid.r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_color_count() {
        let mut m = new_vertex_color_map();
        set_vertex_color(
            &mut m,
            0,
            VertexColor {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        );
        set_vertex_color(
            &mut m,
            1,
            VertexColor {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
        );
        assert_eq!(vertex_color_count(&m), 2);
    }

    #[test]
    fn test_clear_colors() {
        let mut m = new_vertex_color_map();
        set_vertex_color(
            &mut m,
            0,
            VertexColor {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        );
        clear_vertex_colors(&mut m);
        assert_eq!(vertex_color_count(&m), 0);
    }

    #[test]
    fn test_color_to_rgba() {
        let c = VertexColor {
            r: 1.0,
            g: 0.5,
            b: 0.0,
            a: 1.0,
        };
        let rgba = color_to_rgba(&c);
        assert_eq!(rgba[0], 255);
        assert!((rgba[1] as f32 - 127.5).abs() < 1.5);
        assert_eq!(rgba[2], 0);
        assert_eq!(rgba[3], 255);
    }

    #[test]
    fn test_vertex_colors_to_bytes() {
        let mut m = new_vertex_color_map();
        set_vertex_color(
            &mut m,
            0,
            VertexColor {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        );
        let bytes = vertex_colors_to_bytes(&m);
        assert_eq!(bytes.len(), 4);
        assert_eq!(bytes[0], 255);
    }

    #[test]
    fn test_blend_clamp() {
        let a = VertexColor {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };
        let b = VertexColor {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        let out = blend_vertex_colors(&a, &b, 2.0);
        assert!((out.r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_overwrite_color() {
        let mut m = new_vertex_color_map();
        set_vertex_color(
            &mut m,
            0,
            VertexColor {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        );
        set_vertex_color(
            &mut m,
            0,
            VertexColor {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
        );
        let c = get_vertex_color(&m, 0);
        assert!((c.g - 1.0).abs() < 1e-6);
        assert_eq!(vertex_color_count(&m), 1);
    }
}
