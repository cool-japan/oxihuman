#![allow(dead_code)]
//! Mesh normal export functionality.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshNormalExport { normals: Vec<[f32;3]>, format: String }

#[allow(dead_code)]
pub fn export_mesh_normals(normals: &[[f32;3]]) -> MeshNormalExport {
    MeshNormalExport { normals: normals.to_vec(), format: "float3".to_string() }
}
#[allow(dead_code)]
pub fn normal_count_mne(mne: &MeshNormalExport) -> usize { mne.normals.len() }
#[allow(dead_code)]
pub fn normal_format_mne(mne: &MeshNormalExport) -> &str { &mne.format }
#[allow(dead_code)]
pub fn normal_to_bytes_mne(mne: &MeshNormalExport) -> Vec<u8> {
    let mut b = Vec::with_capacity(mne.normals.len()*12);
    for n in &mne.normals { for &v in n { b.extend_from_slice(&v.to_le_bytes()); } }
    b
}
#[allow(dead_code)]
pub fn normal_to_json_mne(mne: &MeshNormalExport) -> String {
    let ns: Vec<String> = mne.normals.iter().map(|n| format!("[{:.6},{:.6},{:.6}]",n[0],n[1],n[2])).collect();
    format!("{{\"normals\":[{}]}}", ns.join(","))
}
#[allow(dead_code)]
pub fn normal_is_unit_mne(mne: &MeshNormalExport, idx: usize) -> bool {
    mne.normals.get(idx).is_some_and(|n| {
        let l = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt();
        (l - 1.0).abs() < 0.01
    })
}
#[allow(dead_code)]
pub fn normal_export_size_mne(mne: &MeshNormalExport) -> usize { mne.normals.len() * 12 }
#[allow(dead_code)]
pub fn validate_mesh_normals(mne: &MeshNormalExport) -> bool {
    mne.normals.iter().all(|n| n.iter().all(|v| v.is_finite()))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> Vec<[f32;3]> { vec![[0.0,0.0,1.0],[0.0,1.0,0.0]] }
    #[test] fn test_export() { let mne = export_mesh_normals(&data()); assert_eq!(normal_count_mne(&mne), 2); }
    #[test] fn test_count() { let mne = export_mesh_normals(&data()); assert_eq!(normal_count_mne(&mne), 2); }
    #[test] fn test_format() { let mne = export_mesh_normals(&data()); assert_eq!(normal_format_mne(&mne), "float3"); }
    #[test] fn test_bytes() { let mne = export_mesh_normals(&data()); assert_eq!(normal_to_bytes_mne(&mne).len(), 24); }
    #[test] fn test_json() { let mne = export_mesh_normals(&data()); assert!(normal_to_json_mne(&mne).contains("normals")); }
    #[test] fn test_unit() { let mne = export_mesh_normals(&data()); assert!(normal_is_unit_mne(&mne, 0)); }
    #[test] fn test_size() { let mne = export_mesh_normals(&data()); assert_eq!(normal_export_size_mne(&mne), 24); }
    #[test] fn test_validate() { let mne = export_mesh_normals(&data()); assert!(validate_mesh_normals(&mne)); }
    #[test] fn test_empty() { let mne = export_mesh_normals(&[]); assert_eq!(normal_count_mne(&mne), 0); }
    #[test] fn test_unit_oob() { let mne = export_mesh_normals(&data()); assert!(!normal_is_unit_mne(&mne, 99)); }
}
