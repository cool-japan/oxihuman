// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Generic mesh attribute layer for per-element data.

/// The domain of an attribute.
#[derive(Clone, Copy, PartialEq)]
pub enum AttrDomain {
    Vertex,
    Edge,
    Face,
    FaceCorner,
}

/// A generic f32 attribute layer.
pub struct AttributeLayer {
    pub name: String,
    pub domain: AttrDomain,
    pub data: Vec<f32>,
}

/// A collection of attribute layers on a mesh.
pub struct AttributeLayerSet {
    pub layers: Vec<AttributeLayer>,
}

/// Create a new empty attribute layer set.
pub fn new_attribute_layer_set() -> AttributeLayerSet {
    AttributeLayerSet { layers: Vec::new() }
}

/// Add a new attribute layer with a given name, domain, element count and default value.
pub fn add_attribute_layer(
    set: &mut AttributeLayerSet,
    name: &str,
    domain: AttrDomain,
    element_count: usize,
    default_val: f32,
) {
    set.layers.push(AttributeLayer {
        name: name.to_string(),
        domain,
        data: vec![default_val; element_count],
    });
}

/// Get a reference to an attribute layer by name.
pub fn get_attribute_layer<'a>(
    set: &'a AttributeLayerSet,
    name: &str,
) -> Option<&'a AttributeLayer> {
    set.layers.iter().find(|l| l.name == name)
}

/// Get a mutable reference to an attribute layer by name.
pub fn get_attribute_layer_mut<'a>(
    set: &'a mut AttributeLayerSet,
    name: &str,
) -> Option<&'a mut AttributeLayer> {
    set.layers.iter_mut().find(|l| l.name == name)
}

/// Remove an attribute layer; returns true if removed.
pub fn remove_attribute_layer(set: &mut AttributeLayerSet, name: &str) -> bool {
    if let Some(pos) = set.layers.iter().position(|l| l.name == name) {
        set.layers.remove(pos);
        true
    } else {
        false
    }
}

/// Number of attribute layers.
pub fn attribute_layer_count(set: &AttributeLayerSet) -> usize {
    set.layers.len()
}

/// Compute average value of a named attribute layer.
pub fn attribute_average(set: &AttributeLayerSet, name: &str) -> Option<f32> {
    let layer = get_attribute_layer(set, name)?;
    if layer.data.is_empty() {
        return Some(0.0);
    }
    let sum: f32 = layer.data.iter().sum();
    Some(sum / layer.data.len() as f32)
}

/// Validate attribute layer lengths for the given element counts.
pub fn validate_attribute_layers(set: &AttributeLayerSet, vertex_count: usize) -> bool {
    set.layers.iter().all(|l| match l.domain {
        AttrDomain::Vertex => l.data.len() == vertex_count,
        _ => true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_empty() {
        let s = new_attribute_layer_set();
        assert_eq!(attribute_layer_count(&s), 0 /* empty */);
    }

    #[test]
    fn add_layer_increments() {
        let mut s = new_attribute_layer_set();
        add_attribute_layer(&mut s, "weights", AttrDomain::Vertex, 4, 0.0);
        assert_eq!(attribute_layer_count(&s), 1 /* one layer */);
    }

    #[test]
    fn get_layer_by_name() {
        let mut s = new_attribute_layer_set();
        add_attribute_layer(&mut s, "ao", AttrDomain::Vertex, 3, 1.0);
        let l = get_attribute_layer(&s, "ao");
        assert!(l.is_some() /* found */);
        assert_eq!(l.unwrap().data.len(), 3 /* three elements */);
    }

    #[test]
    fn get_missing_none() {
        let s = new_attribute_layer_set();
        assert!(get_attribute_layer(&s, "x").is_none() /* not found */);
    }

    #[test]
    fn remove_layer_works() {
        let mut s = new_attribute_layer_set();
        add_attribute_layer(&mut s, "del", AttrDomain::Face, 2, 0.0);
        let ok = remove_attribute_layer(&mut s, "del");
        assert!(ok /* removed */);
        assert_eq!(attribute_layer_count(&s), 0 /* empty */);
    }

    #[test]
    fn remove_missing_false() {
        let mut s = new_attribute_layer_set();
        assert!(!remove_attribute_layer(&mut s, "none") /* not found */);
    }

    #[test]
    fn attribute_average_correct() {
        let mut s = new_attribute_layer_set();
        add_attribute_layer(&mut s, "val", AttrDomain::Vertex, 4, 0.0);
        if let Some(l) = get_attribute_layer_mut(&mut s, "val") {
            l.data[0] = 2.0;
            l.data[1] = 4.0;
            l.data[2] = 6.0;
            l.data[3] = 8.0;
        }
        let avg = attribute_average(&s, "val").unwrap();
        assert!((avg - 5.0).abs() < 1e-6 /* average = 5.0 */);
    }

    #[test]
    fn attribute_average_missing_none() {
        let s = new_attribute_layer_set();
        assert!(attribute_average(&s, "x").is_none() /* not found */);
    }

    #[test]
    fn validate_vertex_layer_correct_count() {
        let mut s = new_attribute_layer_set();
        add_attribute_layer(&mut s, "v", AttrDomain::Vertex, 5, 0.0);
        assert!(validate_attribute_layers(&s, 5) /* valid */);
        assert!(!validate_attribute_layers(&s, 3) /* wrong count */);
    }
}
