// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vertex color layer management.

/// A named vertex color layer storing RGBA colors per vertex.
pub struct VertexColorLayer {
    pub name: String,
    pub colors: Vec<[f32; 4]>,
    pub active: bool,
}

/// Collection of named vertex color layers.
pub struct VertexColorLayerSet {
    pub layers: Vec<VertexColorLayer>,
}

/// Create a new empty vertex color layer set.
pub fn new_vertex_color_layer_set() -> VertexColorLayerSet {
    VertexColorLayerSet { layers: Vec::new() }
}

/// Add a new layer with given name and vertex count (filled with default color).
pub fn add_vertex_color_layer(
    set: &mut VertexColorLayerSet,
    name: &str,
    vertex_count: usize,
    default_color: [f32; 4],
) {
    set.layers.push(VertexColorLayer {
        name: name.to_string(),
        colors: vec![default_color; vertex_count],
        active: set.layers.is_empty(),
    });
}

/// Set the active layer by name; returns true if found.
pub fn set_active_layer(set: &mut VertexColorLayerSet, name: &str) -> bool {
    let found = set.layers.iter().any(|l| l.name == name);
    if found {
        for l in set.layers.iter_mut() {
            l.active = l.name == name;
        }
    }
    found
}

/// Get a mutable reference to a layer by name.
pub fn get_layer_mut<'a>(
    set: &'a mut VertexColorLayerSet,
    name: &str,
) -> Option<&'a mut VertexColorLayer> {
    set.layers.iter_mut().find(|l| l.name == name)
}

/// Remove a layer by name; returns true if removed.
pub fn remove_vertex_color_layer(set: &mut VertexColorLayerSet, name: &str) -> bool {
    if let Some(pos) = set.layers.iter().position(|l| l.name == name) {
        set.layers.remove(pos);
        true
    } else {
        false
    }
}

/// Number of layers.
pub fn layer_count(set: &VertexColorLayerSet) -> usize {
    set.layers.len()
}

/// Average color across all vertices in a named layer.
pub fn layer_average_color(set: &VertexColorLayerSet, name: &str) -> Option<[f32; 4]> {
    let layer = set.layers.iter().find(|l| l.name == name)?;
    if layer.colors.is_empty() {
        return Some([0.0; 4]);
    }
    let n = layer.colors.len() as f32;
    let mut sum = [0.0f32; 4];
    for c in &layer.colors {
        sum[0] += c[0];
        sum[1] += c[1];
        sum[2] += c[2];
        sum[3] += c[3];
    }
    Some([sum[0] / n, sum[1] / n, sum[2] / n, sum[3] / n])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_is_empty() {
        let set = new_vertex_color_layer_set();
        assert_eq!(layer_count(&set), 0 /* empty */);
    }

    #[test]
    fn add_layer_increases_count() {
        let mut set = new_vertex_color_layer_set();
        add_vertex_color_layer(&mut set, "Col", 4, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(layer_count(&set), 1 /* one layer */);
    }

    #[test]
    fn first_layer_is_active() {
        let mut set = new_vertex_color_layer_set();
        add_vertex_color_layer(&mut set, "Base", 3, [0.5, 0.5, 0.5, 1.0]);
        assert!(set.layers[0].active /* first is active */);
    }

    #[test]
    fn set_active_layer_works() {
        let mut set = new_vertex_color_layer_set();
        add_vertex_color_layer(&mut set, "A", 2, [0.0; 4]);
        add_vertex_color_layer(&mut set, "B", 2, [0.0; 4]);
        let ok = set_active_layer(&mut set, "B");
        assert!(ok /* found */);
        assert!(set.layers[1].active /* B is active */);
        assert!(!set.layers[0].active /* A is not active */);
    }

    #[test]
    fn set_active_missing_returns_false() {
        let mut set = new_vertex_color_layer_set();
        assert!(!set_active_layer(&mut set, "none") /* not found */);
    }

    #[test]
    fn remove_layer_decrements_count() {
        let mut set = new_vertex_color_layer_set();
        add_vertex_color_layer(&mut set, "X", 1, [0.0; 4]);
        let ok = remove_vertex_color_layer(&mut set, "X");
        assert!(ok /* removed */);
        assert_eq!(layer_count(&set), 0 /* empty */);
    }

    #[test]
    fn remove_missing_returns_false() {
        let mut set = new_vertex_color_layer_set();
        assert!(!remove_vertex_color_layer(&mut set, "none") /* not found */);
    }

    #[test]
    fn average_color_correct() {
        let mut set = new_vertex_color_layer_set();
        add_vertex_color_layer(&mut set, "col", 2, [0.0; 4]);
        if let Some(l) = get_layer_mut(&mut set, "col") {
            l.colors[0] = [1.0, 0.0, 0.0, 1.0];
            l.colors[1] = [0.0, 0.0, 0.0, 1.0];
        }
        let avg = layer_average_color(&set, "col").expect("should succeed");
        assert!((avg[0] - 0.5).abs() < 1e-6 /* average R = 0.5 */);
    }

    #[test]
    fn average_color_missing_layer_none() {
        let set = new_vertex_color_layer_set();
        assert!(layer_average_color(&set, "none").is_none() /* not found */);
    }
}
