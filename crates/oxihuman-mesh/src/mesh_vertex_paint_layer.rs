#![allow(dead_code)]
//! Vertex paint layer management.

/// A paint layer storing per-vertex colors.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct PaintLayer {
    pub name: String,
    pub colors: Vec<[f32; 4]>,
}

/// Create a new paint layer.
#[allow(dead_code)]
pub fn new_paint_layer(name: &str, vertex_count: usize) -> PaintLayer {
    PaintLayer {
        name: name.to_string(),
        colors: vec![[1.0, 1.0, 1.0, 1.0]; vertex_count],
    }
}

/// Paint a vertex with a color.
#[allow(dead_code)]
pub fn paint_vertex(layer: &mut PaintLayer, index: usize, color: [f32; 4]) {
    if index < layer.colors.len() {
        layer.colors[index] = color;
    }
}

/// Get color at a vertex.
#[allow(dead_code)]
pub fn paint_color_at(layer: &PaintLayer, index: usize) -> [f32; 4] {
    if index < layer.colors.len() {
        layer.colors[index]
    } else {
        [0.0, 0.0, 0.0, 0.0]
    }
}

/// Clear all colors to white.
#[allow(dead_code)]
pub fn paint_clear(layer: &mut PaintLayer) {
    for c in &mut layer.colors {
        *c = [1.0, 1.0, 1.0, 1.0];
    }
}

/// Get layer name.
#[allow(dead_code)]
pub fn paint_layer_name(layer: &PaintLayer) -> &str {
    &layer.name
}

/// Get vertex count.
#[allow(dead_code)]
pub fn paint_vertex_count(layer: &PaintLayer) -> usize {
    layer.colors.len()
}

/// Blend two paint layers (linear interpolation).
#[allow(dead_code)]
pub fn paint_blend_layers(a: &PaintLayer, b: &PaintLayer, factor: f32) -> PaintLayer {
    let len = a.colors.len().min(b.colors.len());
    let t = factor.clamp(0.0, 1.0);
    let mut colors = Vec::with_capacity(len);
    for i in 0..len {
        colors.push([
            a.colors[i][0] + (b.colors[i][0] - a.colors[i][0]) * t,
            a.colors[i][1] + (b.colors[i][1] - a.colors[i][1]) * t,
            a.colors[i][2] + (b.colors[i][2] - a.colors[i][2]) * t,
            a.colors[i][3] + (b.colors[i][3] - a.colors[i][3]) * t,
        ]);
    }
    PaintLayer {
        name: format!("{}_blend_{}", a.name, b.name),
        colors,
    }
}

/// Serialize paint layer to bytes.
#[allow(dead_code)]
pub fn paint_to_bytes(layer: &PaintLayer) -> Vec<u8> {
    let mut bytes = Vec::new();
    for c in &layer.colors {
        for &ch in c {
            bytes.extend_from_slice(&ch.to_le_bytes());
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_paint_layer() {
        let pl = new_paint_layer("test", 5);
        assert_eq!(pl.colors.len(), 5);
        assert_eq!(pl.name, "test");
    }

    #[test]
    fn test_paint_vertex() {
        let mut pl = new_paint_layer("t", 3);
        paint_vertex(&mut pl, 1, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(pl.colors[1], [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_paint_vertex_out_of_bounds() {
        let mut pl = new_paint_layer("t", 1);
        paint_vertex(&mut pl, 5, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(pl.colors[0], [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_paint_color_at() {
        let pl = new_paint_layer("t", 2);
        assert_eq!(paint_color_at(&pl, 0), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(paint_color_at(&pl, 10), [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_paint_clear() {
        let mut pl = new_paint_layer("t", 2);
        paint_vertex(&mut pl, 0, [1.0, 0.0, 0.0, 1.0]);
        paint_clear(&mut pl);
        assert_eq!(pl.colors[0], [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_paint_layer_name() {
        let pl = new_paint_layer("my_layer", 0);
        assert_eq!(paint_layer_name(&pl), "my_layer");
    }

    #[test]
    fn test_paint_vertex_count() {
        let pl = new_paint_layer("t", 7);
        assert_eq!(paint_vertex_count(&pl), 7);
    }

    #[test]
    fn test_paint_blend_layers() {
        let a = new_paint_layer("a", 2);
        let mut b = new_paint_layer("b", 2);
        paint_vertex(&mut b, 0, [0.0, 0.0, 0.0, 0.0]);
        let blended = paint_blend_layers(&a, &b, 0.5);
        assert!((blended.colors[0][0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_paint_to_bytes() {
        let pl = new_paint_layer("t", 1);
        let bytes = paint_to_bytes(&pl);
        assert_eq!(bytes.len(), 16); // 4 floats * 4 bytes
    }

    #[test]
    fn test_paint_blend_empty() {
        let a = new_paint_layer("a", 0);
        let b = new_paint_layer("b", 0);
        let blended = paint_blend_layers(&a, &b, 0.5);
        assert!(blended.colors.is_empty());
    }
}
