// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Object mask — per-object bitmask for selection, culling, and layer filtering.

/// Object mask layer bits (up to 32 layers).
pub type MaskBits = u32;

/// Predefined mask layers.
#[allow(dead_code)]
pub mod layers {
    pub const DEFAULT: u32 = 1 << 0;
    pub const UI: u32 = 1 << 1;
    pub const PHYSICS: u32 = 1 << 2;
    pub const SHADOW: u32 = 1 << 3;
    pub const SELECTION: u32 = 1 << 4;
    pub const HIDDEN: u32 = 1 << 5;
    pub const ALL: u32 = u32::MAX;
    pub const NONE: u32 = 0;
}

/// Object mask entry.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct ObjectMaskEntry {
    pub object_id: u32,
    pub mask: MaskBits,
    pub name: String,
}

impl ObjectMaskEntry {
    #[allow(dead_code)]
    pub fn is_on_layer(&self, layer: u32) -> bool {
        self.mask & layer != 0
    }
    #[allow(dead_code)]
    pub fn is_visible(&self) -> bool {
        self.mask & layers::HIDDEN == 0
    }
}

/// Object mask registry.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ObjectMaskRegistry {
    pub entries: Vec<ObjectMaskEntry>,
    pub camera_mask: MaskBits,
}

/// Create new registry.
#[allow(dead_code)]
pub fn new_object_mask_registry() -> ObjectMaskRegistry {
    ObjectMaskRegistry {
        entries: Vec::new(),
        camera_mask: layers::ALL,
    }
}

/// Register an object.
#[allow(dead_code)]
pub fn register_object_mask(
    r: &mut ObjectMaskRegistry,
    id: u32,
    name: &str,
    mask: MaskBits,
) -> usize {
    let idx = r.entries.len();
    r.entries.push(ObjectMaskEntry {
        object_id: id,
        mask,
        name: name.to_string(),
    });
    idx
}

/// Remove object by ID.
#[allow(dead_code)]
pub fn remove_object_mask(r: &mut ObjectMaskRegistry, id: u32) {
    r.entries.retain(|e| e.object_id != id);
}

/// Set mask for an object.
#[allow(dead_code)]
pub fn set_object_mask(r: &mut ObjectMaskRegistry, id: u32, mask: MaskBits) {
    if let Some(e) = r.entries.iter_mut().find(|e| e.object_id == id) {
        e.mask = mask;
    }
}

/// Add a layer to an object mask.
#[allow(dead_code)]
pub fn add_layer(r: &mut ObjectMaskRegistry, id: u32, layer: u32) {
    if let Some(e) = r.entries.iter_mut().find(|e| e.object_id == id) {
        e.mask |= layer;
    }
}

/// Remove a layer from an object mask.
#[allow(dead_code)]
pub fn remove_layer(r: &mut ObjectMaskRegistry, id: u32, layer: u32) {
    if let Some(e) = r.entries.iter_mut().find(|e| e.object_id == id) {
        e.mask &= !layer;
    }
}

/// Count of registered objects.
#[allow(dead_code)]
pub fn object_count_mask(r: &ObjectMaskRegistry) -> usize {
    r.entries.len()
}

/// Count of visible objects (not on HIDDEN layer).
#[allow(dead_code)]
pub fn visible_object_count(r: &ObjectMaskRegistry) -> usize {
    r.entries.iter().filter(|e| e.is_visible()).count()
}

/// Get entries visible to camera.
#[allow(dead_code)]
pub fn camera_visible(r: &ObjectMaskRegistry) -> Vec<&ObjectMaskEntry> {
    r.entries
        .iter()
        .filter(|e| e.is_visible() && e.mask & r.camera_mask != 0)
        .collect()
}

/// Export to JSON-like string.
#[allow(dead_code)]
pub fn object_mask_to_json(r: &ObjectMaskRegistry) -> String {
    format!(
        r#"{{"object_count":{},"visible":{}}}"#,
        r.entries.len(),
        visible_object_count(r)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_registry_empty() {
        let r = new_object_mask_registry();
        assert_eq!(object_count_mask(&r), 0);
    }

    #[test]
    fn register_object() {
        let mut r = new_object_mask_registry();
        register_object_mask(&mut r, 1, "mesh", layers::DEFAULT);
        assert_eq!(object_count_mask(&r), 1);
    }

    #[test]
    fn is_on_layer() {
        let e = ObjectMaskEntry {
            object_id: 0,
            mask: layers::DEFAULT | layers::SHADOW,
            name: "x".to_string(),
        };
        assert!(e.is_on_layer(layers::SHADOW));
        assert!(!e.is_on_layer(layers::UI));
    }

    #[test]
    fn is_visible_hidden() {
        let e = ObjectMaskEntry {
            object_id: 0,
            mask: layers::HIDDEN,
            name: "h".to_string(),
        };
        assert!(!e.is_visible());
    }

    #[test]
    fn add_remove_layer() {
        let mut r = new_object_mask_registry();
        register_object_mask(&mut r, 1, "m", layers::DEFAULT);
        add_layer(&mut r, 1, layers::SHADOW);
        assert!(r.entries[0].is_on_layer(layers::SHADOW));
        remove_layer(&mut r, 1, layers::SHADOW);
        assert!(!r.entries[0].is_on_layer(layers::SHADOW));
    }

    #[test]
    fn remove_object() {
        let mut r = new_object_mask_registry();
        register_object_mask(&mut r, 5, "obj", layers::DEFAULT);
        remove_object_mask(&mut r, 5);
        assert_eq!(object_count_mask(&r), 0);
    }

    #[test]
    fn visible_count() {
        let mut r = new_object_mask_registry();
        register_object_mask(&mut r, 1, "vis", layers::DEFAULT);
        register_object_mask(&mut r, 2, "hid", layers::HIDDEN);
        assert_eq!(visible_object_count(&r), 1);
    }

    #[test]
    fn camera_visible_filters() {
        let mut r = new_object_mask_registry();
        r.camera_mask = layers::DEFAULT;
        register_object_mask(&mut r, 1, "d", layers::DEFAULT);
        register_object_mask(&mut r, 2, "p", layers::PHYSICS);
        assert_eq!(camera_visible(&r).len(), 1);
    }

    #[test]
    fn json_contains_object_count() {
        let r = new_object_mask_registry();
        assert!(object_mask_to_json(&r).contains("object_count"));
    }

    #[test]
    fn set_mask() {
        let mut r = new_object_mask_registry();
        register_object_mask(&mut r, 1, "m", layers::DEFAULT);
        set_object_mask(&mut r, 1, layers::PHYSICS);
        assert!(r.entries[0].is_on_layer(layers::PHYSICS));
        assert!(!r.entries[0].is_on_layer(layers::DEFAULT));
    }
}
