// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Generic per-vertex attribute storage (float arrays of arbitrary width).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexAttrLayer {
    pub name: String,
    pub width: usize,
    pub data: Vec<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexAttrSet {
    pub layers: Vec<VertexAttrLayer>,
    pub vertex_count: usize,
}

#[allow(dead_code)]
pub fn new_vertex_attr_set(vertex_count: usize) -> VertexAttrSet {
    VertexAttrSet {
        layers: Vec::new(),
        vertex_count,
    }
}

#[allow(dead_code)]
pub fn add_attr_layer(set: &mut VertexAttrSet, name: &str, width: usize) {
    set.layers.push(VertexAttrLayer {
        name: name.to_string(),
        width,
        data: vec![0.0; set.vertex_count * width],
    });
}

#[allow(dead_code)]
pub fn set_attr(set: &mut VertexAttrSet, layer: usize, vertex: usize, values: &[f32]) {
    if layer >= set.layers.len() {
        return;
    }
    let w = set.layers[layer].width;
    let start = vertex * w;
    let end = start + w.min(values.len());
    if end <= set.layers[layer].data.len() {
        set.layers[layer].data[start..end].copy_from_slice(&values[..w.min(values.len())]);
    }
}

#[allow(dead_code)]
pub fn get_attr(set: &VertexAttrSet, layer: usize, vertex: usize) -> &[f32] {
    if layer >= set.layers.len() {
        return &[];
    }
    let w = set.layers[layer].width;
    let start = vertex * w;
    let end = start + w;
    if end <= set.layers[layer].data.len() {
        &set.layers[layer].data[start..end]
    } else {
        &[]
    }
}

#[allow(dead_code)]
pub fn layer_count(set: &VertexAttrSet) -> usize {
    set.layers.len()
}

#[allow(dead_code)]
pub fn find_layer_by_name<'a>(set: &'a VertexAttrSet, name: &str) -> Option<&'a VertexAttrLayer> {
    set.layers.iter().find(|l| l.name == name)
}

#[allow(dead_code)]
pub fn attr_average(set: &VertexAttrSet, layer: usize, component: usize) -> f32 {
    if layer >= set.layers.len() || set.vertex_count == 0 {
        return 0.0;
    }
    let w = set.layers[layer].width;
    if component >= w {
        return 0.0;
    }
    let sum: f32 = (0..set.vertex_count)
        .map(|v| set.layers[layer].data[v * w + component])
        .sum();
    sum / set.vertex_count as f32
}

#[allow(dead_code)]
pub fn attr_set_to_json(set: &VertexAttrSet) -> String {
    format!(
        "{{\"vertex_count\":{},\"layer_count\":{}}}",
        set.vertex_count,
        layer_count(set)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty_layers() {
        let set = new_vertex_attr_set(10);
        assert_eq!(layer_count(&set), 0);
    }

    #[test]
    fn test_add_layer() {
        let mut set = new_vertex_attr_set(5);
        add_attr_layer(&mut set, "color", 4);
        assert_eq!(layer_count(&set), 1);
    }

    #[test]
    fn test_set_get_attr() {
        let mut set = new_vertex_attr_set(3);
        add_attr_layer(&mut set, "weight", 1);
        set_attr(&mut set, 0, 1, &[0.75]);
        let val = get_attr(&set, 0, 1);
        assert!((val[0] - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_find_layer() {
        let mut set = new_vertex_attr_set(4);
        add_attr_layer(&mut set, "uv2", 2);
        assert!(find_layer_by_name(&set, "uv2").is_some());
    }

    #[test]
    fn test_find_missing_layer() {
        let set = new_vertex_attr_set(4);
        assert!(find_layer_by_name(&set, "nonexistent").is_none());
    }

    #[test]
    fn test_attr_average() {
        let mut set = new_vertex_attr_set(4);
        add_attr_layer(&mut set, "ao", 1);
        for v in 0..4 {
            set_attr(&mut set, 0, v, &[1.0]);
        }
        let avg = attr_average(&set, 0, 0);
        assert!((avg - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_initial_data_zero() {
        let mut set = new_vertex_attr_set(3);
        add_attr_layer(&mut set, "x", 1);
        let val = get_attr(&set, 0, 0);
        assert!(!val.is_empty());
        assert!((val[0]).abs() < 1e-6);
    }

    #[test]
    fn test_json_output() {
        let set = new_vertex_attr_set(8);
        let j = attr_set_to_json(&set);
        assert!(j.contains("vertex_count"));
    }

    #[test]
    fn test_out_of_bounds_layer() {
        let set = new_vertex_attr_set(3);
        let v = get_attr(&set, 99, 0);
        assert!(v.is_empty());
    }

    #[test]
    fn test_width_stored() {
        let mut set = new_vertex_attr_set(2);
        add_attr_layer(&mut set, "tangent", 3);
        assert_eq!(set.layers[0].width, 3);
    }
}
