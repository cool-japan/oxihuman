// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Decal layer management (ordered decal rendering stacks).

/// Blend mode for a decal layer.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecalBlendMode {
    Opaque,
    AlphaBlend,
    Multiply,
    Screen,
}

/// A single decal layer.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct DecalLayerEntry {
    pub id: u32,
    pub name: String,
    pub blend_mode: DecalBlendMode,
    pub opacity: f32,
    pub visible: bool,
    pub sort_order: i32,
}

/// Layer stack.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct DecalLayerStack {
    pub layers: Vec<DecalLayerEntry>,
}

#[allow(dead_code)]
pub fn new_decal_layer_stack() -> DecalLayerStack {
    DecalLayerStack::default()
}

#[allow(dead_code)]
pub fn dl_add_layer(stack: &mut DecalLayerStack, id: u32, name: &str, mode: DecalBlendMode) {
    let order = stack.layers.len() as i32;
    stack.layers.push(DecalLayerEntry {
        id,
        name: name.to_string(),
        blend_mode: mode,
        opacity: 1.0,
        visible: true,
        sort_order: order,
    });
}

#[allow(dead_code)]
pub fn dl_remove_layer(stack: &mut DecalLayerStack, id: u32) {
    stack.layers.retain(|l| l.id != id);
}

#[allow(dead_code)]
pub fn dl_set_opacity(stack: &mut DecalLayerStack, id: u32, op: f32) {
    if let Some(l) = stack.layers.iter_mut().find(|l| l.id == id) {
        l.opacity = op.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn dl_set_visible(stack: &mut DecalLayerStack, id: u32, v: bool) {
    if let Some(l) = stack.layers.iter_mut().find(|l| l.id == id) {
        l.visible = v;
    }
}

#[allow(dead_code)]
pub fn dl_visible_count(stack: &DecalLayerStack) -> usize {
    stack.layers.iter().filter(|l| l.visible).count()
}

#[allow(dead_code)]
pub fn dl_layer_count(stack: &DecalLayerStack) -> usize {
    stack.layers.len()
}

#[allow(dead_code)]
pub fn dl_blend_mode_name(m: DecalBlendMode) -> &'static str {
    match m {
        DecalBlendMode::Opaque => "opaque",
        DecalBlendMode::AlphaBlend => "alpha_blend",
        DecalBlendMode::Multiply => "multiply",
        DecalBlendMode::Screen => "screen",
    }
}

#[allow(dead_code)]
pub fn dl_sort_layers(stack: &mut DecalLayerStack) {
    stack.layers.sort_by_key(|l| l.sort_order);
}

#[allow(dead_code)]
pub fn dl_to_json(stack: &DecalLayerStack) -> String {
    let e: Vec<String> = stack
        .layers
        .iter()
        .map(|l| {
            format!(
                "{{\"id\":{},\"name\":\"{}\",\"mode\":\"{}\",\"opacity\":{:.4}}}",
                l.id,
                l.name,
                dl_blend_mode_name(l.blend_mode),
                l.opacity
            )
        })
        .collect();
    format!("[{}]", e.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        assert_eq!(dl_layer_count(&new_decal_layer_stack()), 0);
    }

    #[test]
    fn add_layer() {
        let mut s = new_decal_layer_stack();
        dl_add_layer(&mut s, 1, "base", DecalBlendMode::AlphaBlend);
        assert_eq!(dl_layer_count(&s), 1);
    }

    #[test]
    fn remove_layer() {
        let mut s = new_decal_layer_stack();
        dl_add_layer(&mut s, 1, "base", DecalBlendMode::Opaque);
        dl_remove_layer(&mut s, 1);
        assert_eq!(dl_layer_count(&s), 0);
    }

    #[test]
    fn set_opacity_clamps() {
        let mut s = new_decal_layer_stack();
        dl_add_layer(&mut s, 1, "x", DecalBlendMode::Opaque);
        dl_set_opacity(&mut s, 1, 5.0);
        assert!(s.layers[0].opacity <= 1.0);
    }

    #[test]
    fn set_not_visible() {
        let mut s = new_decal_layer_stack();
        dl_add_layer(&mut s, 1, "x", DecalBlendMode::Multiply);
        dl_set_visible(&mut s, 1, false);
        assert_eq!(dl_visible_count(&s), 0);
    }

    #[test]
    fn visible_count_default_all() {
        let mut s = new_decal_layer_stack();
        dl_add_layer(&mut s, 1, "a", DecalBlendMode::Screen);
        dl_add_layer(&mut s, 2, "b", DecalBlendMode::Screen);
        assert_eq!(dl_visible_count(&s), 2);
    }

    #[test]
    fn blend_mode_names() {
        assert_eq!(dl_blend_mode_name(DecalBlendMode::Screen), "screen");
    }

    #[test]
    fn sort_layers() {
        let mut s = new_decal_layer_stack();
        dl_add_layer(&mut s, 1, "a", DecalBlendMode::Opaque);
        dl_add_layer(&mut s, 2, "b", DecalBlendMode::Opaque);
        s.layers[0].sort_order = 10;
        s.layers[1].sort_order = 0;
        dl_sort_layers(&mut s);
        assert_eq!(s.layers[0].id, 2);
    }

    #[test]
    fn json_not_empty() {
        let s = new_decal_layer_stack();
        assert!(!dl_to_json(&s).is_empty());
    }

    #[test]
    fn json_has_id_after_add() {
        let mut s = new_decal_layer_stack();
        dl_add_layer(&mut s, 42, "test", DecalBlendMode::Multiply);
        assert!(dl_to_json(&s).contains("42"));
    }
}
