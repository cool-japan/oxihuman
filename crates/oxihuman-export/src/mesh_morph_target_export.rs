#![allow(dead_code)]
//! Mesh morph target export v2.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshMorphTargetExportV2 { name: String, deltas: Vec<[f32;3]> }

#[allow(dead_code)]
pub fn export_morph_target_v2(name: &str, deltas: &[[f32;3]]) -> MeshMorphTargetExportV2 { MeshMorphTargetExportV2 { name: name.to_string(), deltas: deltas.to_vec() } }
#[allow(dead_code)]
pub fn target_count_mmte(m: &MeshMorphTargetExportV2) -> usize { m.deltas.len() }
#[allow(dead_code)]
pub fn target_name_mmte(m: &MeshMorphTargetExportV2) -> &str { &m.name }
#[allow(dead_code)]
pub fn target_deltas_mmte(m: &MeshMorphTargetExportV2) -> &[[f32;3]] { &m.deltas }
#[allow(dead_code)]
pub fn target_to_json_mmte(m: &MeshMorphTargetExportV2) -> String {
    let ds: Vec<String> = m.deltas.iter().map(|d| format!("[{:.6},{:.6},{:.6}]",d[0],d[1],d[2])).collect();
    format!("{{\"name\":\"{}\",\"deltas\":[{}]}}", m.name, ds.join(","))
}
#[allow(dead_code)]
pub fn target_to_bytes_mmte(m: &MeshMorphTargetExportV2) -> Vec<u8> { let mut b=Vec::with_capacity(m.deltas.len()*12); for d in &m.deltas { for &v in d { b.extend_from_slice(&v.to_le_bytes()); } } b }
#[allow(dead_code)]
pub fn target_export_size_mmte(m: &MeshMorphTargetExportV2) -> usize { m.deltas.len()*12 }
#[allow(dead_code)]
pub fn validate_morph_target_v2(m: &MeshMorphTargetExportV2) -> bool { !m.name.is_empty() && m.deltas.iter().all(|d| d.iter().all(|v| v.is_finite())) }

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> MeshMorphTargetExportV2 { export_morph_target_v2("smile", &[[0.1,0.0,0.0],[0.0,0.1,0.0]]) }
    #[test] fn test_export() { let m = data(); assert_eq!(target_count_mmte(&m), 2); }
    #[test] fn test_count() { let m = data(); assert_eq!(target_count_mmte(&m), 2); }
    #[test] fn test_name() { let m = data(); assert_eq!(target_name_mmte(&m), "smile"); }
    #[test] fn test_deltas() { let m = data(); assert_eq!(target_deltas_mmte(&m).len(), 2); }
    #[test] fn test_json() { let m = data(); assert!(target_to_json_mmte(&m).contains("smile")); }
    #[test] fn test_bytes() { let m = data(); assert_eq!(target_to_bytes_mmte(&m).len(), 24); }
    #[test] fn test_size() { let m = data(); assert_eq!(target_export_size_mmte(&m), 24); }
    #[test] fn test_validate() { let m = data(); assert!(validate_morph_target_v2(&m)); }
    #[test] fn test_empty_name() { let m = export_morph_target_v2("", &[[0.0;3]]); assert!(!validate_morph_target_v2(&m)); }
    #[test] fn test_empty_deltas() { let m = export_morph_target_v2("x", &[]); assert_eq!(target_count_mmte(&m), 0); }
}
