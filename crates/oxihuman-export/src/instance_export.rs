#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A single instance with transform, mesh, and material references.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Instance {
    pub mesh_id: u32,
    pub material_id: u32,
    /// 4x4 column-major transform.
    pub transform: [f32; 16],
}

/// Instance export container.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstanceExport {
    pub instances: Vec<Instance>,
}

/// Export instances to JSON string.
#[allow(dead_code)]
pub fn export_instances_json(instances: &[Instance]) -> String {
    let mut s = String::from("{\"instances\":[");
    for (i, inst) in instances.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"mesh_id\":{},\"material_id\":{}}}",
            inst.mesh_id, inst.material_id,
        ));
    }
    s.push_str("]}");
    s
}

/// Return the number of instances.
#[allow(dead_code)]
pub fn instance_count_export(exp: &InstanceExport) -> usize {
    exp.instances.len()
}

/// Return the transform of an instance.
#[allow(dead_code)]
pub fn instance_transform_at(exp: &InstanceExport, index: usize) -> Option<[f32; 16]> {
    exp.instances.get(index).map(|i| i.transform)
}

/// Serialize a single instance to JSON.
#[allow(dead_code)]
pub fn instance_to_json(inst: &Instance) -> String {
    format!(
        "{{\"mesh_id\":{},\"material_id\":{},\"transform\":[{}]}}",
        inst.mesh_id,
        inst.material_id,
        inst.transform.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","),
    )
}

/// Return mesh id.
#[allow(dead_code)]
pub fn instance_mesh_id(exp: &InstanceExport, index: usize) -> Option<u32> {
    exp.instances.get(index).map(|i| i.mesh_id)
}

/// Return material id.
#[allow(dead_code)]
pub fn instance_material_id(exp: &InstanceExport, index: usize) -> Option<u32> {
    exp.instances.get(index).map(|i| i.material_id)
}

/// Return total byte size estimate.
#[allow(dead_code)]
pub fn instance_export_size(exp: &InstanceExport) -> usize {
    // per instance: mesh_id(4) + material_id(4) + transform(64) = 72
    exp.instances.len() * 72
}

/// Validate: all transforms have finite values.
#[allow(dead_code)]
pub fn validate_instance_export(exp: &InstanceExport) -> bool {
    exp.instances.iter().all(|i| i.transform.iter().all(|v| v.is_finite()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_transform() -> [f32; 16] {
        [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]
    }

    fn sample() -> InstanceExport {
        InstanceExport {
            instances: vec![
                Instance { mesh_id: 0, material_id: 1, transform: identity_transform() },
                Instance { mesh_id: 2, material_id: 3, transform: identity_transform() },
            ],
        }
    }

    #[test]
    fn test_export_json() {
        let j = export_instances_json(&sample().instances);
        assert!(j.contains("\"mesh_id\":0"));
        assert!(j.contains("\"mesh_id\":2"));
    }

    #[test]
    fn test_count() {
        assert_eq!(instance_count_export(&sample()), 2);
    }

    #[test]
    fn test_transform_at() {
        let t = instance_transform_at(&sample(), 0).expect("should succeed");
        assert!((t[0] - 1.0).abs() < 1e-5);
        assert!(instance_transform_at(&sample(), 99).is_none());
    }

    #[test]
    fn test_instance_to_json() {
        let inst = Instance { mesh_id: 5, material_id: 7, transform: identity_transform() };
        let j = instance_to_json(&inst);
        assert!(j.contains("\"mesh_id\":5"));
    }

    #[test]
    fn test_mesh_id() {
        assert_eq!(instance_mesh_id(&sample(), 0), Some(0));
        assert_eq!(instance_mesh_id(&sample(), 1), Some(2));
    }

    #[test]
    fn test_material_id() {
        assert_eq!(instance_material_id(&sample(), 0), Some(1));
    }

    #[test]
    fn test_export_size() {
        assert_eq!(instance_export_size(&sample()), 144);
    }

    #[test]
    fn test_validate_ok() {
        assert!(validate_instance_export(&sample()));
    }

    #[test]
    fn test_validate_nan() {
        let mut e = sample();
        e.instances[0].transform[0] = f32::NAN;
        assert!(!validate_instance_export(&e));
    }

    #[test]
    fn test_empty() {
        let e = InstanceExport { instances: vec![] };
        assert_eq!(instance_count_export(&e), 0);
        assert!(validate_instance_export(&e));
    }
}
