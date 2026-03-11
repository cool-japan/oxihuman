#![allow(dead_code)]
//! Mesh tangent export functionality.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshTangentExport { tangents: Vec<[f32;4]>, format: String }

#[allow(dead_code)]
pub fn export_mesh_tangents(tangents: &[[f32;4]]) -> MeshTangentExport {
    MeshTangentExport { tangents: tangents.to_vec(), format: "float4".to_string() }
}
#[allow(dead_code)]
pub fn tangent_count_mte(mte: &MeshTangentExport) -> usize { mte.tangents.len() }
#[allow(dead_code)]
pub fn tangent_format_mte(mte: &MeshTangentExport) -> &str { &mte.format }
#[allow(dead_code)]
pub fn tangent_to_bytes_mte(mte: &MeshTangentExport) -> Vec<u8> {
    let mut b = Vec::with_capacity(mte.tangents.len()*16);
    for t in &mte.tangents { for &v in t { b.extend_from_slice(&v.to_le_bytes()); } }
    b
}
#[allow(dead_code)]
pub fn tangent_to_json_mte(mte: &MeshTangentExport) -> String {
    let ts: Vec<String> = mte.tangents.iter().map(|t| format!("[{:.4},{:.4},{:.4},{:.4}]",t[0],t[1],t[2],t[3])).collect();
    format!("{{\"tangents\":[{}],\"format\":\"{}\"}}", ts.join(","), mte.format)
}
#[allow(dead_code)]
pub fn tangent_sign_mte(mte: &MeshTangentExport, idx: usize) -> f32 {
    mte.tangents.get(idx).map_or(0.0, |t| t[3])
}
#[allow(dead_code)]
pub fn tangent_export_size_mte(mte: &MeshTangentExport) -> usize { mte.tangents.len() * 16 }
#[allow(dead_code)]
pub fn validate_mesh_tangents(mte: &MeshTangentExport) -> bool {
    mte.tangents.iter().all(|t| t.iter().all(|v| v.is_finite()))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> Vec<[f32;4]> { vec![[1.0,0.0,0.0,1.0],[0.0,1.0,0.0,-1.0]] }
    #[test] fn test_export() { let mte = export_mesh_tangents(&data()); assert_eq!(tangent_count_mte(&mte), 2); }
    #[test] fn test_count() { let mte = export_mesh_tangents(&data()); assert_eq!(tangent_count_mte(&mte), 2); }
    #[test] fn test_format() { let mte = export_mesh_tangents(&data()); assert_eq!(tangent_format_mte(&mte), "float4"); }
    #[test] fn test_bytes() { let mte = export_mesh_tangents(&data()); assert_eq!(tangent_to_bytes_mte(&mte).len(), 32); }
    #[test] fn test_json() { let mte = export_mesh_tangents(&data()); assert!(tangent_to_json_mte(&mte).contains("tangents")); }
    #[test] fn test_sign() { let mte = export_mesh_tangents(&data()); assert!((tangent_sign_mte(&mte, 1) - (-1.0)).abs() < 1e-6); }
    #[test] fn test_size() { let mte = export_mesh_tangents(&data()); assert_eq!(tangent_export_size_mte(&mte), 32); }
    #[test] fn test_validate() { let mte = export_mesh_tangents(&data()); assert!(validate_mesh_tangents(&mte)); }
    #[test] fn test_empty() { let mte = export_mesh_tangents(&[]); assert_eq!(tangent_count_mte(&mte), 0); }
    #[test] fn test_sign_oob() { let mte = export_mesh_tangents(&data()); assert!((tangent_sign_mte(&mte, 99) - 0.0).abs() < 1e-9); }
}
