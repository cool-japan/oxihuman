#![allow(dead_code)]
//! Primitive topology export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PrimitiveTopologyExport { mode: String, index_count: u32, vertex_count: u32, restart: bool }

#[allow(dead_code)]
pub fn export_topology(mode: &str, idx_count: u32, vert_count: u32) -> PrimitiveTopologyExport {
    PrimitiveTopologyExport { mode: mode.to_string(), index_count: idx_count, vertex_count: vert_count, restart: false }
}
#[allow(dead_code)]
pub fn topology_mode(m: &PrimitiveTopologyExport) -> &str { &m.mode }
#[allow(dead_code)]
pub fn topology_to_json_pte(m: &PrimitiveTopologyExport) -> String {
    format!("{{\"mode\":\"{}\",\"indices\":{},\"vertices\":{},\"restart\":{}}}", m.mode, m.index_count, m.vertex_count, m.restart)
}
#[allow(dead_code)]
pub fn topology_index_count_pte(m: &PrimitiveTopologyExport) -> u32 { m.index_count }
#[allow(dead_code)]
pub fn topology_restart_enabled(m: &PrimitiveTopologyExport) -> bool { m.restart }
#[allow(dead_code)]
pub fn topology_vertex_count_pte(m: &PrimitiveTopologyExport) -> u32 { m.vertex_count }
#[allow(dead_code)]
pub fn topology_export_size(m: &PrimitiveTopologyExport) -> usize { m.mode.len() + 12 }
#[allow(dead_code)]
pub fn validate_topology_pte(m: &PrimitiveTopologyExport) -> bool {
    !m.mode.is_empty() && ["triangles","lines","points","triangle_strip","triangle_fan"].contains(&m.mode.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> PrimitiveTopologyExport { export_topology("triangles", 36, 24) }
    #[test] fn test_export() { let m = data(); assert_eq!(topology_mode(&m), "triangles"); }
    #[test] fn test_mode() { let m = data(); assert_eq!(topology_mode(&m), "triangles"); }
    #[test] fn test_json() { let m = data(); assert!(topology_to_json_pte(&m).contains("triangles")); }
    #[test] fn test_idx_count() { let m = data(); assert_eq!(topology_index_count_pte(&m), 36); }
    #[test] fn test_restart() { let m = data(); assert!(!topology_restart_enabled(&m)); }
    #[test] fn test_vert_count() { let m = data(); assert_eq!(topology_vertex_count_pte(&m), 24); }
    #[test] fn test_size() { let m = data(); assert!(topology_export_size(&m) > 0); }
    #[test] fn test_validate() { let m = data(); assert!(validate_topology_pte(&m)); }
    #[test] fn test_invalid_mode() { let m = export_topology("quads", 4, 4); assert!(!validate_topology_pte(&m)); }
    #[test] fn test_lines() { let m = export_topology("lines", 10, 5); assert!(validate_topology_pte(&m)); }
}
