// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Geometry instance placement export (point-scatter instancing).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeoInstanceEntry {
    pub mesh_name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeoInstanceSetExport {
    pub instances: Vec<GeoInstanceEntry>,
}

#[allow(dead_code)]
pub fn new_geo_instance_set() -> GeoInstanceSetExport {
    GeoInstanceSetExport {
        instances: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_instance_entry(exp: &mut GeoInstanceSetExport, mesh: &str, pos: [f32; 3]) {
    exp.instances.push(GeoInstanceEntry {
        mesh_name: mesh.to_string(),
        position: pos,
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    });
}

#[allow(dead_code)]
pub fn instance_count(exp: &GeoInstanceSetExport) -> usize {
    exp.instances.len()
}

#[allow(dead_code)]
pub fn instances_of_mesh<'a>(
    exp: &'a GeoInstanceSetExport,
    mesh: &str,
) -> Vec<&'a GeoInstanceEntry> {
    exp.instances
        .iter()
        .filter(|i| i.mesh_name == mesh)
        .collect()
}

#[allow(dead_code)]
pub fn unique_mesh_names(exp: &GeoInstanceSetExport) -> Vec<String> {
    let mut names: Vec<String> = exp.instances.iter().map(|i| i.mesh_name.clone()).collect();
    names.sort_unstable();
    names.dedup();
    names
}

#[allow(dead_code)]
pub fn instance_bounds(exp: &GeoInstanceSetExport) -> ([f32; 3], [f32; 3]) {
    if exp.instances.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for inst in &exp.instances {
        for j in 0..3 {
            mn[j] = mn[j].min(inst.position[j]);
            mx[j] = mx[j].max(inst.position[j]);
        }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn geo_instance_to_json(exp: &GeoInstanceSetExport) -> String {
    format!(
        "{{\"instance_count\":{},\"mesh_types\":{}}}",
        instance_count(exp),
        unique_mesh_names(exp).len()
    )
}

#[allow(dead_code)]
pub fn validate_instances(exp: &GeoInstanceSetExport) -> bool {
    exp.instances.iter().all(|i| !i.mesh_name.is_empty())
}

#[allow(dead_code)]
pub fn clear_instances(exp: &mut GeoInstanceSetExport) {
    exp.instances.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let exp = new_geo_instance_set();
        assert_eq!(instance_count(&exp), 0);
    }

    #[test]
    fn test_add_instance() {
        let mut exp = new_geo_instance_set();
        add_instance_entry(&mut exp, "tree", [1.0, 0.0, 0.0]);
        assert_eq!(instance_count(&exp), 1);
    }

    #[test]
    fn test_instances_of_mesh() {
        let mut exp = new_geo_instance_set();
        add_instance_entry(&mut exp, "rock", [0.0; 3]);
        add_instance_entry(&mut exp, "tree", [1.0, 0.0, 0.0]);
        add_instance_entry(&mut exp, "rock", [2.0, 0.0, 0.0]);
        assert_eq!(instances_of_mesh(&exp, "rock").len(), 2);
    }

    #[test]
    fn test_unique_mesh_names() {
        let mut exp = new_geo_instance_set();
        add_instance_entry(&mut exp, "tree", [0.0; 3]);
        add_instance_entry(&mut exp, "rock", [0.0; 3]);
        add_instance_entry(&mut exp, "tree", [1.0, 0.0, 0.0]);
        assert_eq!(unique_mesh_names(&exp).len(), 2);
    }

    #[test]
    fn test_bounds() {
        let mut exp = new_geo_instance_set();
        add_instance_entry(&mut exp, "x", [-1.0, 0.0, 0.0]);
        add_instance_entry(&mut exp, "x", [2.0, 0.0, 0.0]);
        let (mn, mx) = instance_bounds(&exp);
        assert!((mn[0] - -1.0).abs() < 1e-5);
        assert!((mx[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_json_output() {
        let exp = new_geo_instance_set();
        let j = geo_instance_to_json(&exp);
        assert!(j.contains("instance_count"));
    }

    #[test]
    fn test_validate_ok() {
        let mut exp = new_geo_instance_set();
        add_instance_entry(&mut exp, "mesh", [0.0; 3]);
        assert!(validate_instances(&exp));
    }

    #[test]
    fn test_clear() {
        let mut exp = new_geo_instance_set();
        add_instance_entry(&mut exp, "mesh", [0.0; 3]);
        clear_instances(&mut exp);
        assert_eq!(instance_count(&exp), 0);
    }

    #[test]
    fn test_default_scale_one() {
        let mut exp = new_geo_instance_set();
        add_instance_entry(&mut exp, "mesh", [0.0; 3]);
        assert_eq!(exp.instances[0].scale, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_default_rotation_identity() {
        let mut exp = new_geo_instance_set();
        add_instance_entry(&mut exp, "mesh", [0.0; 3]);
        assert!((exp.instances[0].rotation[3] - 1.0).abs() < 1e-6);
    }
}
