#![allow(dead_code)]
//! Mesh UV export functionality.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshUvExport { uvs: Vec<[f32;2]>, format: String }

#[allow(dead_code)]
pub fn export_mesh_uvs(uvs: &[[f32;2]]) -> MeshUvExport {
    MeshUvExport { uvs: uvs.to_vec(), format: "float2".to_string() }
}
#[allow(dead_code)]
pub fn uv_count_mue(mue: &MeshUvExport) -> usize { mue.uvs.len() }
#[allow(dead_code)]
pub fn uv_format(mue: &MeshUvExport) -> &str { &mue.format }
#[allow(dead_code)]
pub fn uv_to_bytes(mue: &MeshUvExport) -> Vec<u8> {
    let mut b = Vec::with_capacity(mue.uvs.len()*8);
    for uv in &mue.uvs { for &v in uv { b.extend_from_slice(&v.to_le_bytes()); } }
    b
}
#[allow(dead_code)]
pub fn uv_bounds_mue(mue: &MeshUvExport) -> ([f32;2],[f32;2]) {
    let mut lo = [f32::INFINITY;2]; let mut hi = [f32::NEG_INFINITY;2];
    for uv in &mue.uvs { for i in 0..2 { lo[i]=lo[i].min(uv[i]); hi[i]=hi[i].max(uv[i]); } }
    (lo, hi)
}
#[allow(dead_code)]
pub fn uv_to_json_mue(mue: &MeshUvExport) -> String {
    let us: Vec<String> = mue.uvs.iter().map(|uv| format!("[{:.6},{:.6}]",uv[0],uv[1])).collect();
    format!("{{\"uvs\":[{}]}}", us.join(","))
}
#[allow(dead_code)]
pub fn uv_export_size(mue: &MeshUvExport) -> usize { mue.uvs.len()*8 }
#[allow(dead_code)]
pub fn validate_uvs(mue: &MeshUvExport) -> bool {
    mue.uvs.iter().all(|uv| uv.iter().all(|v| v.is_finite()))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> Vec<[f32;2]> { vec![[0.0,0.0],[1.0,0.0],[0.5,1.0]] }
    #[test] fn test_export() { let m = export_mesh_uvs(&data()); assert_eq!(uv_count_mue(&m), 3); }
    #[test] fn test_count() { let m = export_mesh_uvs(&data()); assert_eq!(uv_count_mue(&m), 3); }
    #[test] fn test_format() { let m = export_mesh_uvs(&data()); assert_eq!(uv_format(&m), "float2"); }
    #[test] fn test_bytes() { let m = export_mesh_uvs(&data()); assert_eq!(uv_to_bytes(&m).len(), 24); }
    #[test] fn test_bounds() { let m = export_mesh_uvs(&data()); let (_lo,hi) = uv_bounds_mue(&m); assert!((hi[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_json() { let m = export_mesh_uvs(&data()); assert!(uv_to_json_mue(&m).contains("uvs")); }
    #[test] fn test_size() { let m = export_mesh_uvs(&data()); assert_eq!(uv_export_size(&m), 24); }
    #[test] fn test_validate() { let m = export_mesh_uvs(&data()); assert!(validate_uvs(&m)); }
    #[test] fn test_empty() { let m = export_mesh_uvs(&[]); assert_eq!(uv_count_mue(&m), 0); }
    #[test] fn test_single() { let m = export_mesh_uvs(&[[0.5,0.5]]); assert_eq!(uv_count_mue(&m), 1); }
}
