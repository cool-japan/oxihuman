#![allow(dead_code)]
//! Mesh color export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshColorExport { colors: Vec<[f32;4]>, format: String }

#[allow(dead_code)]
pub fn export_mesh_colors(colors: &[[f32;4]]) -> MeshColorExport { MeshColorExport { colors: colors.to_vec(), format: "float4".into() } }
#[allow(dead_code)]
pub fn color_count_mce(m: &MeshColorExport) -> usize { m.colors.len() }
#[allow(dead_code)]
pub fn color_format_mce(m: &MeshColorExport) -> &str { &m.format }
#[allow(dead_code)]
pub fn color_to_bytes_mce(m: &MeshColorExport) -> Vec<u8> { let mut b=Vec::with_capacity(m.colors.len()*16); for c in &m.colors { for &v in c { b.extend_from_slice(&v.to_le_bytes()); } } b }
#[allow(dead_code)]
pub fn color_to_json_mce(m: &MeshColorExport) -> String { let cs: Vec<String> = m.colors.iter().map(|c| format!("[{:.4},{:.4},{:.4},{:.4}]",c[0],c[1],c[2],c[3])).collect(); format!("{{\"colors\":[{}]}}", cs.join(",")) }
#[allow(dead_code)]
pub fn color_channel_count_mce(_m: &MeshColorExport) -> usize { 4 }
#[allow(dead_code)]
pub fn color_export_size_mce(m: &MeshColorExport) -> usize { m.colors.len()*16 }
#[allow(dead_code)]
pub fn validate_mesh_colors(m: &MeshColorExport) -> bool { m.colors.iter().all(|c| c.iter().all(|&v| (0.0..=1.0).contains(&v))) }

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> Vec<[f32;4]> { vec![[1.0,0.0,0.0,1.0],[0.0,1.0,0.0,1.0]] }
    #[test] fn test_export() { let m = export_mesh_colors(&data()); assert_eq!(color_count_mce(&m), 2); }
    #[test] fn test_count() { let m = export_mesh_colors(&data()); assert_eq!(color_count_mce(&m), 2); }
    #[test] fn test_format() { let m = export_mesh_colors(&data()); assert_eq!(color_format_mce(&m), "float4"); }
    #[test] fn test_bytes() { let m = export_mesh_colors(&data()); assert_eq!(color_to_bytes_mce(&m).len(), 32); }
    #[test] fn test_json() { let m = export_mesh_colors(&data()); assert!(color_to_json_mce(&m).contains("colors")); }
    #[test] fn test_channels() { let m = export_mesh_colors(&data()); assert_eq!(color_channel_count_mce(&m), 4); }
    #[test] fn test_size() { let m = export_mesh_colors(&data()); assert_eq!(color_export_size_mce(&m), 32); }
    #[test] fn test_validate() { let m = export_mesh_colors(&data()); assert!(validate_mesh_colors(&m)); }
    #[test] fn test_empty() { let m = export_mesh_colors(&[]); assert_eq!(color_count_mce(&m), 0); }
    #[test] fn test_invalid() { let m = export_mesh_colors(&[[2.0,0.0,0.0,1.0]]); assert!(!validate_mesh_colors(&m)); }
}
