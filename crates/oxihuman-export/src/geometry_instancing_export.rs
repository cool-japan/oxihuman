// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Geometry instancing export for repeated mesh placement.

/// A geometry instance with transform.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeoInstance {
    pub mesh_id: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

/// Instancing export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeoInstancingExport {
    pub instances: Vec<GeoInstance>,
}

/// Create new export.
#[allow(dead_code)]
pub fn new_geo_instancing_export() -> GeoInstancingExport {
    GeoInstancingExport { instances: vec![] }
}

/// Add instance with identity transform.
#[allow(dead_code)]
pub fn add_instance(e: &mut GeoInstancingExport, mesh_id: u32, pos: [f32; 3]) {
    e.instances.push(GeoInstance {
        mesh_id,
        position: pos,
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    });
}

/// Add instance with full transform.
#[allow(dead_code)]
pub fn add_instance_full(
    e: &mut GeoInstancingExport,
    mesh_id: u32,
    pos: [f32; 3],
    rot: [f32; 4],
    scale: [f32; 3],
) {
    e.instances.push(GeoInstance {
        mesh_id,
        position: pos,
        rotation: rot,
        scale,
    });
}

/// Instance count.
#[allow(dead_code)]
pub fn gi_count(e: &GeoInstancingExport) -> usize {
    e.instances.len()
}

/// Count instances of a specific mesh.
#[allow(dead_code)]
pub fn instances_of_mesh(e: &GeoInstancingExport, mesh_id: u32) -> usize {
    e.instances.iter().filter(|i| i.mesh_id == mesh_id).count()
}

/// Unique mesh count.
#[allow(dead_code)]
pub fn unique_mesh_count(e: &GeoInstancingExport) -> usize {
    let mut ids: Vec<u32> = e.instances.iter().map(|i| i.mesh_id).collect();
    ids.sort_unstable();
    ids.dedup();
    ids.len()
}

/// Validate (quaternions roughly unit length).
#[allow(dead_code)]
pub fn gi_validate(e: &GeoInstancingExport) -> bool {
    e.instances.iter().all(|i| {
        let q = i.rotation;
        let len_sq = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
        (len_sq - 1.0).abs() < 0.1
    })
}

/// Export to JSON.
#[allow(dead_code)]
pub fn geo_instancing_to_json(e: &GeoInstancingExport) -> String {
    format!(
        "{{\"instances\":{},\"unique_meshes\":{}}}",
        gi_count(e),
        unique_mesh_count(e)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let e = new_geo_instancing_export();
        assert_eq!(gi_count(&e), 0);
    }
    #[test]
    fn test_add() {
        let mut e = new_geo_instancing_export();
        add_instance(&mut e, 0, [1.0, 0.0, 0.0]);
        assert_eq!(gi_count(&e), 1);
    }
    #[test]
    fn test_add_full() {
        let mut e = new_geo_instancing_export();
        add_instance_full(&mut e, 0, [0.0; 3], [0.0, 0.0, 0.0, 1.0], [2.0; 3]);
        assert_eq!(gi_count(&e), 1);
    }
    #[test]
    fn test_instances_of() {
        let mut e = new_geo_instancing_export();
        add_instance(&mut e, 0, [0.0; 3]);
        add_instance(&mut e, 0, [1.0; 3]);
        add_instance(&mut e, 1, [0.0; 3]);
        assert_eq!(instances_of_mesh(&e, 0), 2);
    }
    #[test]
    fn test_unique() {
        let mut e = new_geo_instancing_export();
        add_instance(&mut e, 0, [0.0; 3]);
        add_instance(&mut e, 1, [0.0; 3]);
        assert_eq!(unique_mesh_count(&e), 2);
    }
    #[test]
    fn test_validate() {
        let mut e = new_geo_instancing_export();
        add_instance(&mut e, 0, [0.0; 3]);
        assert!(gi_validate(&e));
    }
    #[test]
    fn test_to_json() {
        let e = new_geo_instancing_export();
        assert!(geo_instancing_to_json(&e).contains("\"instances\":0"));
    }
    #[test]
    fn test_empty_unique() {
        let e = new_geo_instancing_export();
        assert_eq!(unique_mesh_count(&e), 0);
    }
    #[test]
    fn test_position() {
        let mut e = new_geo_instancing_export();
        add_instance(&mut e, 0, [5.0, 3.0, 1.0]);
        assert!((e.instances[0].position[0] - 5.0).abs() < 1e-6);
    }
    #[test]
    fn test_default_scale() {
        let mut e = new_geo_instancing_export();
        add_instance(&mut e, 0, [0.0; 3]);
        assert!((e.instances[0].scale[0] - 1.0).abs() < 1e-6);
    }
}
