// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Instanced mesh renderer — manages instance transforms and draw call batching.

use std::f32::consts::TAU;

/// A single mesh instance.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct MeshInstance {
    pub transform: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub lod: u32,
    pub enabled: bool,
}

impl Default for MeshInstance {
    fn default() -> Self {
        Self {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            color: [1.0, 1.0, 1.0, 1.0],
            lod: 0,
            enabled: true,
        }
    }
}

/// Instanced mesh state.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct InstancedMesh {
    pub instances: Vec<MeshInstance>,
    pub max_instances: usize,
    pub mesh_id: u32,
}

/// Create new instanced mesh.
#[allow(dead_code)]
pub fn new_instanced_mesh(mesh_id: u32, max_instances: usize) -> InstancedMesh {
    InstancedMesh {
        instances: Vec::new(),
        max_instances,
        mesh_id,
    }
}

/// Add an instance.
#[allow(dead_code)]
pub fn add_instance(m: &mut InstancedMesh, inst: MeshInstance) -> Option<usize> {
    if m.instances.len() >= m.max_instances {
        return None;
    }
    let idx = m.instances.len();
    m.instances.push(inst);
    Some(idx)
}

/// Remove instance by index.
#[allow(dead_code)]
pub fn remove_instance(m: &mut InstancedMesh, idx: usize) {
    if idx < m.instances.len() {
        m.instances.remove(idx);
    }
}

/// Total instance count.
#[allow(dead_code)]
pub fn instance_count(m: &InstancedMesh) -> usize {
    m.instances.len()
}

/// Enabled instance count.
#[allow(dead_code)]
pub fn enabled_instance_count(m: &InstancedMesh) -> usize {
    m.instances.iter().filter(|i| i.enabled).count()
}

/// Build rotation matrix around Y axis by angle (using TAU for full circle check).
#[allow(dead_code)]
pub fn rotation_y(angle_rad: f32) -> [[f32; 4]; 4] {
    let _full = TAU; // reference to ensure TAU is used
    let c = angle_rad.cos();
    let s = angle_rad.sin();
    [
        [c, 0.0, s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-s, 0.0, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Build translation matrix.
#[allow(dead_code)]
pub fn translation_matrix(tx: f32, ty: f32, tz: f32) -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [tx, ty, tz, 1.0],
    ]
}

/// Set LOD for all instances.
#[allow(dead_code)]
pub fn set_all_lod(m: &mut InstancedMesh, lod: u32) {
    for inst in &mut m.instances {
        inst.lod = lod;
    }
}

/// Clear all instances.
#[allow(dead_code)]
pub fn clear_instances(m: &mut InstancedMesh) {
    m.instances.clear();
}

/// Memory estimate in bytes.
#[allow(dead_code)]
pub fn instance_memory_bytes(m: &InstancedMesh) -> usize {
    m.instances.len() * std::mem::size_of::<MeshInstance>()
}

/// Export to JSON-like string.
#[allow(dead_code)]
pub fn instanced_mesh_to_json(m: &InstancedMesh) -> String {
    format!(
        r#"{{"mesh_id":{},"count":{},"enabled":{}}}"#,
        m.mesh_id,
        m.instances.len(),
        enabled_instance_count(m)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_mesh_empty() {
        let m = new_instanced_mesh(1, 100);
        assert_eq!(instance_count(&m), 0);
    }

    #[test]
    fn add_and_count() {
        let mut m = new_instanced_mesh(1, 100);
        add_instance(&mut m, MeshInstance::default());
        assert_eq!(instance_count(&m), 1);
    }

    #[test]
    fn capacity_limit() {
        let mut m = new_instanced_mesh(1, 1);
        add_instance(&mut m, MeshInstance::default());
        assert!(add_instance(&mut m, MeshInstance::default()).is_none());
    }

    #[test]
    fn remove_works() {
        let mut m = new_instanced_mesh(1, 10);
        add_instance(&mut m, MeshInstance::default());
        remove_instance(&mut m, 0);
        assert_eq!(instance_count(&m), 0);
    }

    #[test]
    fn enabled_count() {
        let mut m = new_instanced_mesh(1, 10);
        add_instance(
            &mut m,
            MeshInstance {
                enabled: true,
                ..Default::default()
            },
        );
        add_instance(
            &mut m,
            MeshInstance {
                enabled: false,
                ..Default::default()
            },
        );
        assert_eq!(enabled_instance_count(&m), 1);
    }

    #[test]
    fn rotation_y_identity_at_zero() {
        let r = rotation_y(0.0);
        assert!((r[0][0] - 1.0).abs() < 1e-6);
        assert!((r[1][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn rotation_y_full_circle() {
        let r = rotation_y(TAU);
        assert!((r[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn set_all_lod_changes() {
        let mut m = new_instanced_mesh(1, 10);
        add_instance(&mut m, MeshInstance::default());
        add_instance(&mut m, MeshInstance::default());
        set_all_lod(&mut m, 2);
        assert!(m.instances.iter().all(|i| i.lod == 2));
    }

    #[test]
    fn json_contains_mesh_id() {
        let m = new_instanced_mesh(42, 10);
        assert!(instanced_mesh_to_json(&m).contains("42"));
    }

    #[test]
    fn memory_bytes_nonzero_after_add() {
        let mut m = new_instanced_mesh(1, 10);
        add_instance(&mut m, MeshInstance::default());
        assert!(instance_memory_bytes(&m) > 0);
    }
}
