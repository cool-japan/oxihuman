// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! GPU instanced rendering state.

#![allow(dead_code)]

/// Configuration for instanced rendering.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstancedConfig {
    pub max_instances: usize,
    pub dynamic: bool,
}

/// Per-instance data for the GPU.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstanceData {
    pub transform: [f32; 16],
    pub color: [f32; 4],
}

/// Instanced renderer state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstancedRenderer {
    pub mesh_id: u32,
    pub instances: Vec<InstanceData>,
    pub config: InstancedConfig,
}

#[allow(dead_code)]
pub fn default_instanced_config() -> InstancedConfig {
    InstancedConfig {
        max_instances: 1024,
        dynamic: true,
    }
}

#[allow(dead_code)]
pub fn new_instanced_renderer(mesh_id: u32) -> InstancedRenderer {
    InstancedRenderer {
        mesh_id,
        instances: Vec::new(),
        config: default_instanced_config(),
    }
}

#[allow(dead_code)]
pub fn ir_add_instance(renderer: &mut InstancedRenderer, data: InstanceData) -> bool {
    if renderer.instances.len() >= renderer.config.max_instances {
        return false;
    }
    renderer.instances.push(data);
    true
}

#[allow(dead_code)]
pub fn ir_remove_instance(renderer: &mut InstancedRenderer, index: usize) -> bool {
    if index >= renderer.instances.len() {
        return false;
    }
    renderer.instances.remove(index);
    true
}

#[allow(dead_code)]
pub fn ir_instance_count(renderer: &InstancedRenderer) -> usize {
    renderer.instances.len()
}

#[allow(dead_code)]
pub fn ir_clear(renderer: &mut InstancedRenderer) {
    renderer.instances.clear();
}

#[allow(dead_code)]
pub fn ir_get_instance(renderer: &InstancedRenderer, index: usize) -> Option<&InstanceData> {
    renderer.instances.get(index)
}

#[allow(dead_code)]
pub fn ir_to_json(renderer: &InstancedRenderer) -> String {
    format!(
        r#"{{"mesh_id":{},"instance_count":{}}}"#,
        renderer.mesh_id,
        renderer.instances.len()
    )
}

#[allow(dead_code)]
pub fn ir_is_full(renderer: &InstancedRenderer) -> bool {
    renderer.instances.len() >= renderer.config.max_instances
}

fn identity_transform() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_instanced_config();
        assert_eq!(cfg.max_instances, 1024);
        assert!(cfg.dynamic);
    }

    #[test]
    fn test_new_renderer_empty() {
        let r = new_instanced_renderer(1);
        assert_eq!(r.mesh_id, 1);
        assert_eq!(ir_instance_count(&r), 0);
    }

    #[test]
    fn test_add_instance() {
        let mut r = new_instanced_renderer(1);
        let d = InstanceData { transform: identity_transform(), color: [1.0; 4] };
        assert!(ir_add_instance(&mut r, d));
        assert_eq!(ir_instance_count(&r), 1);
    }

    #[test]
    fn test_remove_instance() {
        let mut r = new_instanced_renderer(1);
        let d = InstanceData { transform: identity_transform(), color: [1.0; 4] };
        ir_add_instance(&mut r, d);
        assert!(ir_remove_instance(&mut r, 0));
        assert_eq!(ir_instance_count(&r), 0);
    }

    #[test]
    fn test_remove_out_of_bounds() {
        let mut r = new_instanced_renderer(1);
        assert!(!ir_remove_instance(&mut r, 5));
    }

    #[test]
    fn test_clear() {
        let mut r = new_instanced_renderer(1);
        let d = InstanceData { transform: identity_transform(), color: [1.0; 4] };
        ir_add_instance(&mut r, d.clone());
        ir_add_instance(&mut r, d);
        ir_clear(&mut r);
        assert_eq!(ir_instance_count(&r), 0);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let r = new_instanced_renderer(42);
        let j = ir_to_json(&r);
        assert!(j.contains("mesh_id"));
        assert!(j.contains("instance_count"));
    }

    #[test]
    fn test_is_full() {
        let mut r = new_instanced_renderer(1);
        r.config.max_instances = 1;
        let d = InstanceData { transform: identity_transform(), color: [1.0; 4] };
        ir_add_instance(&mut r, d.clone());
        assert!(ir_is_full(&r));
        assert!(!ir_add_instance(&mut r, d));
    }
}
