#![allow(dead_code)]
//! Mesh position export functionality.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshPositionExport { positions: Vec<[f32;3]>, format: String }

#[allow(dead_code)]
pub fn export_mesh_positions(positions: &[[f32;3]]) -> MeshPositionExport {
    MeshPositionExport { positions: positions.to_vec(), format: "float3".to_string() }
}
#[allow(dead_code)]
pub fn position_count_mpe2(mpe: &MeshPositionExport) -> usize { mpe.positions.len() }
#[allow(dead_code)]
pub fn position_format(mpe: &MeshPositionExport) -> &str { &mpe.format }
#[allow(dead_code)]
pub fn position_to_bytes(mpe: &MeshPositionExport) -> Vec<u8> {
    let mut b = Vec::with_capacity(mpe.positions.len()*12);
    for p in &mpe.positions { for &v in p { b.extend_from_slice(&v.to_le_bytes()); } }
    b
}
#[allow(dead_code)]
pub fn position_bounds_mpe(mpe: &MeshPositionExport) -> ([f32;3],[f32;3]) {
    let mut lo = [f32::INFINITY;3]; let mut hi = [f32::NEG_INFINITY;3];
    for p in &mpe.positions { for i in 0..3 { lo[i]=lo[i].min(p[i]); hi[i]=hi[i].max(p[i]); } }
    (lo, hi)
}
#[allow(dead_code)]
pub fn position_to_json_mpe(mpe: &MeshPositionExport) -> String {
    let ps: Vec<String> = mpe.positions.iter().map(|p| format!("[{:.6},{:.6},{:.6}]",p[0],p[1],p[2])).collect();
    format!("{{\"positions\":[{}]}}", ps.join(","))
}
#[allow(dead_code)]
pub fn position_export_size_mpe(mpe: &MeshPositionExport) -> usize { mpe.positions.len()*12 }
#[allow(dead_code)]
pub fn validate_positions(mpe: &MeshPositionExport) -> bool {
    mpe.positions.iter().all(|p| p.iter().all(|v| v.is_finite()))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> Vec<[f32;3]> { vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]] }
    #[test] fn test_export() { let m = export_mesh_positions(&data()); assert_eq!(position_count_mpe2(&m), 3); }
    #[test] fn test_count() { let m = export_mesh_positions(&data()); assert_eq!(position_count_mpe2(&m), 3); }
    #[test] fn test_format() { let m = export_mesh_positions(&data()); assert_eq!(position_format(&m), "float3"); }
    #[test] fn test_bytes() { let m = export_mesh_positions(&data()); assert_eq!(position_to_bytes(&m).len(), 36); }
    #[test] fn test_bounds() { let m = export_mesh_positions(&data()); let (lo,hi) = position_bounds_mpe(&m); assert!((hi[0] - 1.0).abs() < 1e-6); assert!((lo[0]).abs() < 1e-6); }
    #[test] fn test_json() { let m = export_mesh_positions(&data()); assert!(position_to_json_mpe(&m).contains("positions")); }
    #[test] fn test_size() { let m = export_mesh_positions(&data()); assert_eq!(position_export_size_mpe(&m), 36); }
    #[test] fn test_validate() { let m = export_mesh_positions(&data()); assert!(validate_positions(&m)); }
    #[test] fn test_empty() { let m = export_mesh_positions(&[]); assert_eq!(position_count_mpe2(&m), 0); }
    #[test] fn test_single() { let m = export_mesh_positions(&[[1.0,2.0,3.0]]); assert_eq!(position_count_mpe2(&m), 1); }
}
