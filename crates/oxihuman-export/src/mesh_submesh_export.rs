#![allow(dead_code)]
//! Mesh submesh export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshSubmeshExport { start: u32, count: u32, material: String, bounds_min: [f32;3], bounds_max: [f32;3] }

#[allow(dead_code)]
pub fn export_mesh_submesh(start: u32, count: u32, material: &str) -> MeshSubmeshExport {
    MeshSubmeshExport { start, count, material: material.to_string(), bounds_min: [0.0;3], bounds_max: [0.0;3] }
}
#[allow(dead_code)]
pub fn submesh_index_start(m: &MeshSubmeshExport) -> u32 { m.start }
#[allow(dead_code)]
pub fn submesh_index_count_mse(m: &MeshSubmeshExport) -> u32 { m.count }
#[allow(dead_code)]
pub fn submesh_material_mse(m: &MeshSubmeshExport) -> &str { &m.material }
#[allow(dead_code)]
pub fn submesh_to_json_mse(m: &MeshSubmeshExport) -> String {
    format!("{{\"start\":{},\"count\":{},\"material\":\"{}\"}}", m.start, m.count, m.material)
}
#[allow(dead_code)]
pub fn submesh_bounds_mse(m: &MeshSubmeshExport) -> ([f32;3],[f32;3]) { (m.bounds_min, m.bounds_max) }
#[allow(dead_code)]
pub fn submesh_export_size_mse(m: &MeshSubmeshExport) -> usize { 8 + m.material.len() + 24 }
#[allow(dead_code)]
pub fn validate_submesh(m: &MeshSubmeshExport) -> bool { m.count > 0 && !m.material.is_empty() }

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> MeshSubmeshExport { export_mesh_submesh(0, 36, "default") }
    #[test] fn test_export() { let m = data(); assert_eq!(submesh_index_start(&m), 0); }
    #[test] fn test_start() { let m = data(); assert_eq!(submesh_index_start(&m), 0); }
    #[test] fn test_count() { let m = data(); assert_eq!(submesh_index_count_mse(&m), 36); }
    #[test] fn test_material() { let m = data(); assert_eq!(submesh_material_mse(&m), "default"); }
    #[test] fn test_json() { let m = data(); assert!(submesh_to_json_mse(&m).contains("default")); }
    #[test] fn test_bounds() { let m = data(); let (lo,hi) = submesh_bounds_mse(&m); assert!((lo[0]).abs() < 1e-6); assert!((hi[0]).abs() < 1e-6); }
    #[test] fn test_size() { let m = data(); assert!(submesh_export_size_mse(&m) > 0); }
    #[test] fn test_validate() { let m = data(); assert!(validate_submesh(&m)); }
    #[test] fn test_invalid_count() { let m = export_mesh_submesh(0, 0, "mat"); assert!(!validate_submesh(&m)); }
    #[test] fn test_invalid_mat() { let m = export_mesh_submesh(0, 10, ""); assert!(!validate_submesh(&m)); }
}
