#![allow(dead_code)]

//! Mesh instance export for GPU instancing.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshInstanceExport {
    pub instances: Vec<MeshInstanceEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshInstanceEntry {
    pub mesh_ref: String,
    pub transform: [f32; 16],
    pub material: String,
}

#[allow(dead_code)]
pub fn export_mesh_instances(instances: Vec<MeshInstanceEntry>) -> MeshInstanceExport {
    MeshInstanceExport { instances }
}

#[allow(dead_code)]
pub fn instance_count_mie(exp: &MeshInstanceExport) -> usize { exp.instances.len() }

#[allow(dead_code)]
pub fn instance_mesh_ref(exp: &MeshInstanceExport, idx: usize) -> Option<&str> {
    exp.instances.get(idx).map(|i| i.mesh_ref.as_str())
}

#[allow(dead_code)]
pub fn instance_transform_mie(exp: &MeshInstanceExport, idx: usize) -> Option<&[f32; 16]> {
    exp.instances.get(idx).map(|i| &i.transform)
}

#[allow(dead_code)]
pub fn instance_to_json(exp: &MeshInstanceExport) -> String {
    let items: Vec<String> = exp.instances.iter().map(|i|
        format!("{{\"mesh\":\"{}\",\"material\":\"{}\"}}", i.mesh_ref, i.material)
    ).collect();
    format!("{{\"instance_count\":{},\"instances\":[{}]}}", exp.instances.len(), items.join(","))
}

#[allow(dead_code)]
pub fn instance_material_mie(exp: &MeshInstanceExport, idx: usize) -> Option<&str> {
    exp.instances.get(idx).map(|i| i.material.as_str())
}

#[allow(dead_code)]
pub fn instance_export_size(exp: &MeshInstanceExport) -> usize {
    exp.instances.len() * 64
}

#[allow(dead_code)]
pub fn validate_mesh_instances(exp: &MeshInstanceExport) -> bool {
    !exp.instances.is_empty() && exp.instances.iter().all(|i| !i.mesh_ref.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn inst(m: &str) -> MeshInstanceEntry {
        let mut t = [0.0f32;16]; t[0]=1.0; t[5]=1.0; t[10]=1.0; t[15]=1.0;
        MeshInstanceEntry { mesh_ref: m.into(), transform: t, material: "default".into() }
    }

    #[test]
    fn test_export() { let e = export_mesh_instances(vec![inst("tree")]); assert_eq!(instance_count_mie(&e), 1); }
    #[test]
    fn test_mesh_ref() { let e = export_mesh_instances(vec![inst("rock")]); assert_eq!(instance_mesh_ref(&e, 0), Some("rock")); }
    #[test]
    fn test_mesh_ref_none() { let e = export_mesh_instances(vec![]); assert!(instance_mesh_ref(&e, 0).is_none()); }
    #[test]
    fn test_transform() { let e = export_mesh_instances(vec![inst("a")]); assert!(instance_transform_mie(&e, 0).is_some()); }
    #[test]
    fn test_to_json() { let e = export_mesh_instances(vec![inst("a")]); assert!(instance_to_json(&e).contains("\"instance_count\":1")); }
    #[test]
    fn test_material() { let e = export_mesh_instances(vec![inst("a")]); assert_eq!(instance_material_mie(&e, 0), Some("default")); }
    #[test]
    fn test_export_size() { let e = export_mesh_instances(vec![inst("a")]); assert_eq!(instance_export_size(&e), 64); }
    #[test]
    fn test_validate() { assert!(validate_mesh_instances(&export_mesh_instances(vec![inst("a")]))); }
    #[test]
    fn test_validate_empty() { assert!(!validate_mesh_instances(&export_mesh_instances(vec![]))); }
    #[test]
    fn test_multi() { let e = export_mesh_instances(vec![inst("a"),inst("b"),inst("c")]); assert_eq!(instance_count_mie(&e), 3); }
}
