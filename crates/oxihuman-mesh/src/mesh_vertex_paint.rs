// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VertexColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexPaintLayer {
    pub name: String,
    pub colors: Vec<VertexColor>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexPaintMap {
    pub layers: Vec<VertexPaintLayer>,
}

#[allow(dead_code)]
pub fn default_vertex_color() -> VertexColor {
    VertexColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
}

#[allow(dead_code)]
pub fn new_vertex_paint_map() -> VertexPaintMap {
    VertexPaintMap { layers: Vec::new() }
}

#[allow(dead_code)]
pub fn vp_add_layer(map: &mut VertexPaintMap, name: &str, n_verts: usize) -> usize {
    let idx = map.layers.len();
    map.layers.push(VertexPaintLayer {
        name: name.to_string(),
        colors: vec![default_vertex_color(); n_verts],
    });
    idx
}

#[allow(dead_code)]
pub fn vp_set_color(map: &mut VertexPaintMap, layer: usize, vert: usize, color: VertexColor) {
    if let Some(l) = map.layers.get_mut(layer) {
        if let Some(c) = l.colors.get_mut(vert) {
            *c = color;
        }
    }
}

#[allow(dead_code)]
pub fn vp_get_color(map: &VertexPaintMap, layer: usize, vert: usize) -> Option<&VertexColor> {
    map.layers.get(layer)?.colors.get(vert)
}

#[allow(dead_code)]
pub fn vp_layer_count(map: &VertexPaintMap) -> usize {
    map.layers.len()
}

#[allow(dead_code)]
pub fn vp_to_json(map: &VertexPaintMap) -> String {
    format!(r#"{{"layer_count":{}}}"#, map.layers.len())
}

#[allow(dead_code)]
pub fn vp_fill_layer(map: &mut VertexPaintMap, layer: usize, color: VertexColor) {
    if let Some(l) = map.layers.get_mut(layer) {
        for c in &mut l.colors {
            *c = color.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_color_white() {
        let c = default_vertex_color();
        assert!((c.r - 1.0).abs() < 1e-6);
        assert!((c.a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_map_empty() {
        let m = new_vertex_paint_map();
        assert_eq!(vp_layer_count(&m), 0);
    }

    #[test]
    fn test_add_layer() {
        let mut m = new_vertex_paint_map();
        let idx = vp_add_layer(&mut m, "Col", 5);
        assert_eq!(idx, 0);
        assert_eq!(vp_layer_count(&m), 1);
    }

    #[test]
    fn test_set_and_get_color() {
        let mut m = new_vertex_paint_map();
        vp_add_layer(&mut m, "Col", 3);
        let red = VertexColor { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
        vp_set_color(&mut m, 0, 1, red.clone());
        let c = vp_get_color(&m, 0, 1).expect("should succeed");
        assert!((c.r - 1.0).abs() < 1e-6);
        assert!(c.g.abs() < 1e-6);
    }

    #[test]
    fn test_get_out_of_bounds() {
        let m = new_vertex_paint_map();
        assert!(vp_get_color(&m, 0, 0).is_none());
    }

    #[test]
    fn test_fill_layer() {
        let mut m = new_vertex_paint_map();
        vp_add_layer(&mut m, "Col", 4);
        let blue = VertexColor { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
        vp_fill_layer(&mut m, 0, blue);
        for i in 0..4 {
            let c = vp_get_color(&m, 0, i).expect("should succeed");
            assert!((c.b - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_to_json() {
        let mut m = new_vertex_paint_map();
        vp_add_layer(&mut m, "A", 2);
        let j = vp_to_json(&m);
        assert!(j.contains("layer_count"));
    }

    #[test]
    fn test_multiple_layers() {
        let mut m = new_vertex_paint_map();
        vp_add_layer(&mut m, "Layer1", 3);
        vp_add_layer(&mut m, "Layer2", 3);
        assert_eq!(vp_layer_count(&m), 2);
    }

    #[test]
    fn test_layer_name() {
        let mut m = new_vertex_paint_map();
        vp_add_layer(&mut m, "MyLayer", 2);
        assert_eq!(m.layers[0].name, "MyLayer");
    }
}
