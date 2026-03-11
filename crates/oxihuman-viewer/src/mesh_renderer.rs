// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Mesh rendering state management.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshRendererConfig {
    pub wireframe: bool,
    pub backface_cull: bool,
    pub cast_shadows: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshRendererState {
    pub mesh_id: u32,
    pub material_id: u32,
    pub visible: bool,
    pub config: MeshRendererConfig,
}

#[allow(dead_code)]
pub fn default_mesh_renderer_config() -> MeshRendererConfig {
    MeshRendererConfig {
        wireframe: false,
        backface_cull: true,
        cast_shadows: true,
    }
}

#[allow(dead_code)]
pub fn new_mesh_renderer_state(mesh_id: u32, material_id: u32) -> MeshRendererState {
    MeshRendererState {
        mesh_id,
        material_id,
        visible: true,
        config: default_mesh_renderer_config(),
    }
}

#[allow(dead_code)]
pub fn mr_set_visible(state: &mut MeshRendererState, visible: bool) {
    state.visible = visible;
}

#[allow(dead_code)]
pub fn mr_toggle_wireframe(state: &mut MeshRendererState) {
    state.config.wireframe = !state.config.wireframe;
}

#[allow(dead_code)]
pub fn mr_is_visible(state: &MeshRendererState) -> bool {
    state.visible
}

#[allow(dead_code)]
pub fn mr_to_json(state: &MeshRendererState) -> String {
    format!(
        r#"{{"mesh_id":{},"material_id":{},"visible":{},"wireframe":{},"backface_cull":{},"cast_shadows":{}}}"#,
        state.mesh_id,
        state.material_id,
        state.visible,
        state.config.wireframe,
        state.config.backface_cull,
        state.config.cast_shadows
    )
}

#[allow(dead_code)]
pub fn mr_reset(state: &mut MeshRendererState) {
    state.visible = true;
    state.config = default_mesh_renderer_config();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_mesh_renderer_config();
        assert!(!cfg.wireframe);
        assert!(cfg.backface_cull);
        assert!(cfg.cast_shadows);
    }

    #[test]
    fn test_new_state_visible() {
        let s = new_mesh_renderer_state(1, 2);
        assert!(mr_is_visible(&s));
        assert_eq!(s.mesh_id, 1);
        assert_eq!(s.material_id, 2);
    }

    #[test]
    fn test_set_visible() {
        let mut s = new_mesh_renderer_state(0, 0);
        mr_set_visible(&mut s, false);
        assert!(!mr_is_visible(&s));
    }

    #[test]
    fn test_toggle_wireframe() {
        let mut s = new_mesh_renderer_state(0, 0);
        assert!(!s.config.wireframe);
        mr_toggle_wireframe(&mut s);
        assert!(s.config.wireframe);
        mr_toggle_wireframe(&mut s);
        assert!(!s.config.wireframe);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_mesh_renderer_state(3, 7);
        let j = mr_to_json(&s);
        assert!(j.contains("mesh_id"));
        assert!(j.contains("visible"));
        assert!(j.contains("wireframe"));
    }

    #[test]
    fn test_reset_restores_defaults() {
        let mut s = new_mesh_renderer_state(1, 1);
        mr_set_visible(&mut s, false);
        mr_toggle_wireframe(&mut s);
        mr_reset(&mut s);
        assert!(mr_is_visible(&s));
        assert!(!s.config.wireframe);
    }

    #[test]
    fn test_debug_clone() {
        let s = new_mesh_renderer_state(0, 0);
        let _ = format!("{:?}", s.clone());
    }

    #[test]
    fn test_json_visible_false() {
        let mut s = new_mesh_renderer_state(0, 0);
        mr_set_visible(&mut s, false);
        let j = mr_to_json(&s);
        assert!(j.contains("false"));
    }

    #[test]
    fn test_mesh_id_preserved() {
        let s = new_mesh_renderer_state(999, 0);
        assert_eq!(s.mesh_id, 999);
    }
}
