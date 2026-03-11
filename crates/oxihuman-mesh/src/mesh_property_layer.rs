// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Named float property layer per vertex.

/// A named float property layer.
pub struct PropertyLayer {
    pub name: String,
    pub values: Vec<f32>,
    pub default_value: f32,
}

/// A set of named float property layers for a mesh.
pub struct PropertyLayerSet {
    pub layers: Vec<PropertyLayer>,
}

/// Create a new empty property layer set.
pub fn new_property_layer_set() -> PropertyLayerSet {
    PropertyLayerSet { layers: Vec::new() }
}

/// Add a new property layer with name, vertex count, and default value.
pub fn add_property_layer(
    set: &mut PropertyLayerSet,
    name: &str,
    vertex_count: usize,
    default_value: f32,
) {
    set.layers.push(PropertyLayer {
        name: name.to_string(),
        values: vec![default_value; vertex_count],
        default_value,
    });
}

/// Get the float value for a vertex in the named layer.
pub fn get_property(set: &PropertyLayerSet, name: &str, vertex: usize) -> Option<f32> {
    let layer = set.layers.iter().find(|l| l.name == name)?;
    layer.values.get(vertex).copied()
}

/// Set the float value for a vertex in the named layer.
pub fn set_property(set: &mut PropertyLayerSet, name: &str, vertex: usize, value: f32) -> bool {
    if let Some(layer) = set.layers.iter_mut().find(|l| l.name == name) {
        if vertex < layer.values.len() {
            layer.values[vertex] = value;
            return true;
        }
    }
    false
}

/// Remove a property layer by name.
pub fn remove_property_layer(set: &mut PropertyLayerSet, name: &str) -> bool {
    if let Some(pos) = set.layers.iter().position(|l| l.name == name) {
        set.layers.remove(pos);
        true
    } else {
        false
    }
}

/// Number of property layers.
pub fn property_layer_count(set: &PropertyLayerSet) -> usize {
    set.layers.len()
}

/// Reset all values in a layer to its default.
pub fn reset_property_layer(set: &mut PropertyLayerSet, name: &str) -> bool {
    if let Some(layer) = set.layers.iter_mut().find(|l| l.name == name) {
        let dv = layer.default_value;
        for v in layer.values.iter_mut() {
            *v = dv;
        }
        true
    } else {
        false
    }
}

/// Compute minimum and maximum values in a layer.
pub fn property_min_max(set: &PropertyLayerSet, name: &str) -> Option<(f32, f32)> {
    let layer = set.layers.iter().find(|l| l.name == name)?;
    if layer.values.is_empty() {
        return None;
    }
    let mut mn = layer.values[0];
    let mut mx = layer.values[0];
    for &v in layer.values.iter().skip(1) {
        mn = mn.min(v);
        mx = mx.max(v);
    }
    Some((mn, mx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_empty() {
        let s = new_property_layer_set();
        assert_eq!(property_layer_count(&s), 0 /* empty */);
    }

    #[test]
    fn add_layer_and_count() {
        let mut s = new_property_layer_set();
        add_property_layer(&mut s, "thickness", 5, 0.0);
        assert_eq!(property_layer_count(&s), 1 /* one layer */);
    }

    #[test]
    fn get_default_value() {
        let mut s = new_property_layer_set();
        add_property_layer(&mut s, "heat", 3, 0.5);
        let v = get_property(&s, "heat", 1).unwrap();
        assert!((v - 0.5).abs() < 1e-6 /* default value */);
    }

    #[test]
    fn set_and_get_property() {
        let mut s = new_property_layer_set();
        add_property_layer(&mut s, "mass", 4, 1.0);
        set_property(&mut s, "mass", 2, 2.71);
        let v = get_property(&s, "mass", 2).unwrap();
        assert!((v - 2.71).abs() < 1e-5 /* updated value */);
    }

    #[test]
    fn set_out_of_bounds_returns_false() {
        let mut s = new_property_layer_set();
        add_property_layer(&mut s, "x", 2, 0.0);
        assert!(!set_property(&mut s, "x", 99, 1.0) /* out of bounds */);
    }

    #[test]
    fn remove_layer_works() {
        let mut s = new_property_layer_set();
        add_property_layer(&mut s, "del", 3, 0.0);
        assert!(remove_property_layer(&mut s, "del") /* removed */);
        assert_eq!(property_layer_count(&s), 0 /* empty */);
    }

    #[test]
    fn reset_layer_to_default() {
        let mut s = new_property_layer_set();
        add_property_layer(&mut s, "r", 2, 7.0);
        set_property(&mut s, "r", 0, 99.0);
        reset_property_layer(&mut s, "r");
        let v = get_property(&s, "r", 0).unwrap();
        assert!((v - 7.0).abs() < 1e-6 /* reset to default */);
    }

    #[test]
    fn min_max_correct() {
        let mut s = new_property_layer_set();
        add_property_layer(&mut s, "mm", 3, 0.0);
        set_property(&mut s, "mm", 0, -1.0);
        set_property(&mut s, "mm", 1, 5.0);
        set_property(&mut s, "mm", 2, 2.0);
        let (mn, mx) = property_min_max(&s, "mm").unwrap();
        assert!((mn + 1.0).abs() < 1e-6 /* min = -1 */);
        assert!((mx - 5.0).abs() < 1e-6 /* max = 5 */);
    }

    #[test]
    fn get_property_missing_layer_none() {
        let s = new_property_layer_set();
        assert!(get_property(&s, "none", 0).is_none() /* not found */);
    }
}
