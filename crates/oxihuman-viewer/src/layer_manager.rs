// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Scene layer visibility/lock manager (similar to Photoshop layers).

#![allow(dead_code)]

/// Configuration for the layer manager.
#[derive(Debug, Clone)]
pub struct LayerManagerConfig {
    /// Maximum number of layers allowed (0 = unlimited).
    pub max_layers: usize,
    /// Whether newly added layers start as visible.
    pub default_visible: bool,
}

/// A single scene layer.
#[derive(Debug, Clone)]
pub struct SceneLayer {
    /// Unique layer id.
    pub id: u32,
    /// Human-readable name.
    pub name: String,
    /// Whether the layer is visible.
    pub visible: bool,
    /// Whether the layer is locked (prevents editing).
    pub locked: bool,
    /// Render order (lower = rendered first).
    pub order: i32,
}

/// The layer manager holding all layers.
#[derive(Debug, Clone)]
pub struct LayerManager {
    /// All layers in the scene.
    pub layers: Vec<SceneLayer>,
    /// Configuration.
    pub config: LayerManagerConfig,
    /// Next id to assign.
    next_id: u32,
}

/// Returns the default [`LayerManagerConfig`].
pub fn default_layer_manager_config() -> LayerManagerConfig {
    LayerManagerConfig {
        max_layers: 0,
        default_visible: true,
    }
}

/// Creates a new, empty [`LayerManager`].
pub fn new_layer_manager(config: LayerManagerConfig) -> LayerManager {
    LayerManager {
        layers: Vec::new(),
        config,
        next_id: 1,
    }
}

/// Adds a new layer and returns its assigned id.
pub fn layer_add(manager: &mut LayerManager, name: &str) -> u32 {
    let id = manager.next_id;
    manager.next_id += 1;
    let order = manager.layers.len() as i32;
    manager.layers.push(SceneLayer {
        id,
        name: name.to_string(),
        visible: manager.config.default_visible,
        locked: false,
        order,
    });
    id
}

/// Removes a layer by id. Returns true if found and removed.
pub fn layer_remove(manager: &mut LayerManager, id: u32) -> bool {
    let before = manager.layers.len();
    manager.layers.retain(|l| l.id != id);
    manager.layers.len() < before
}

/// Sets visibility for a layer. Returns false if id not found.
pub fn layer_set_visible(manager: &mut LayerManager, id: u32, visible: bool) -> bool {
    if let Some(layer) = manager.layers.iter_mut().find(|l| l.id == id) {
        layer.visible = visible;
        true
    } else {
        false
    }
}

/// Sets the locked flag for a layer. Returns false if id not found.
pub fn layer_set_locked(manager: &mut LayerManager, id: u32, locked: bool) -> bool {
    if let Some(layer) = manager.layers.iter_mut().find(|l| l.id == id) {
        layer.locked = locked;
        true
    } else {
        false
    }
}

/// Returns the visibility of a layer by id (false if not found).
pub fn layer_is_visible(manager: &LayerManager, id: u32) -> bool {
    manager
        .layers
        .iter()
        .find(|l| l.id == id)
        .map(|l| l.visible)
        .unwrap_or(false)
}

/// Returns the number of layers.
pub fn layer_count(manager: &LayerManager) -> usize {
    manager.layers.len()
}

/// Removes all layers and resets id counter.
pub fn layer_clear(manager: &mut LayerManager) {
    manager.layers.clear();
    manager.next_id = 1;
}

/// Serialises the layer manager state as JSON.
pub fn layer_manager_to_json(manager: &LayerManager) -> String {
    let mut out = String::from("{\"layers\":[\n");
    for (i, layer) in manager.layers.iter().enumerate() {
        let comma = if i + 1 < manager.layers.len() { "," } else { "" };
        out.push_str(&format!(
            "  {{\"id\":{},\"name\":\"{}\",\"visible\":{},\"locked\":{},\"order\":{}}}{}\n",
            layer.id, layer.name, layer.visible, layer.locked, layer.order, comma
        ));
    }
    out.push_str("]}");
    out
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_manager() -> LayerManager {
        new_layer_manager(default_layer_manager_config())
    }

    #[test]
    fn default_config_values() {
        let cfg = default_layer_manager_config();
        assert_eq!(cfg.max_layers, 0);
        assert!(cfg.default_visible);
    }

    #[test]
    fn new_manager_is_empty() {
        let m = default_manager();
        assert_eq!(layer_count(&m), 0);
    }

    #[test]
    fn add_layer_assigns_unique_ids() {
        let mut m = default_manager();
        let id1 = layer_add(&mut m, "Background");
        let id2 = layer_add(&mut m, "Foreground");
        assert_ne!(id1, id2);
        assert_eq!(layer_count(&m), 2);
    }

    #[test]
    fn new_layer_is_visible() {
        let mut m = default_manager();
        let id = layer_add(&mut m, "Test");
        assert!(layer_is_visible(&m, id));
    }

    #[test]
    fn set_visible_false() {
        let mut m = default_manager();
        let id = layer_add(&mut m, "Test");
        let ok = layer_set_visible(&mut m, id, false);
        assert!(ok);
        assert!(!layer_is_visible(&m, id));
    }

    #[test]
    fn set_locked_on_layer() {
        let mut m = default_manager();
        let id = layer_add(&mut m, "Test");
        let ok = layer_set_locked(&mut m, id, true);
        assert!(ok);
        assert!(m.layers.iter().find(|l| l.id == id).unwrap().locked);
    }

    #[test]
    fn remove_layer_by_id() {
        let mut m = default_manager();
        let id = layer_add(&mut m, "Temp");
        let removed = layer_remove(&mut m, id);
        assert!(removed);
        assert_eq!(layer_count(&m), 0);
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let mut m = default_manager();
        assert!(!layer_remove(&mut m, 999));
    }

    #[test]
    fn clear_resets_all() {
        let mut m = default_manager();
        layer_add(&mut m, "A");
        layer_add(&mut m, "B");
        layer_clear(&mut m);
        assert_eq!(layer_count(&m), 0);
        // After clear, new ids should restart from 1
        let id = layer_add(&mut m, "Fresh");
        assert_eq!(id, 1);
    }

    #[test]
    fn to_json_contains_layers_key() {
        let mut m = default_manager();
        layer_add(&mut m, "Base");
        let json = layer_manager_to_json(&m);
        assert!(json.contains("\"layers\""));
        assert!(json.contains("\"visible\""));
        assert!(json.contains("Base"));
    }
}
