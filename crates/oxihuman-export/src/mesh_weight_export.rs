#![allow(dead_code)]
//! Mesh weight export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshWeightExport { weights: Vec<Vec<(u32, f32)>>, format: String, max_per_vert: usize }

#[allow(dead_code)]
pub fn export_mesh_weights(weights: &[Vec<(u32, f32)>], max_per_vert: usize) -> MeshWeightExport {
    MeshWeightExport { weights: weights.to_vec(), format: "float".into(), max_per_vert }
}
#[allow(dead_code)]
pub fn weight_count_mwe(m: &MeshWeightExport) -> usize { m.weights.len() }
#[allow(dead_code)]
pub fn weight_format(m: &MeshWeightExport) -> &str { &m.format }
#[allow(dead_code)]
pub fn weight_to_bytes_mwe(m: &MeshWeightExport) -> Vec<u8> {
    let mut b = Vec::new();
    for w in &m.weights {
        for &(idx, val) in w.iter().take(m.max_per_vert) { b.extend_from_slice(&idx.to_le_bytes()); b.extend_from_slice(&val.to_le_bytes()); }
        for _ in w.len()..m.max_per_vert { b.extend_from_slice(&0u32.to_le_bytes()); b.extend_from_slice(&0.0f32.to_le_bytes()); }
    }
    b
}
#[allow(dead_code)]
pub fn weight_to_json_mwe(m: &MeshWeightExport) -> String {
    let ws: Vec<String> = m.weights.iter().map(|w| {
        let es: Vec<String> = w.iter().map(|(i,v)| format!("[{},{:.4}]",i,v)).collect();
        format!("[{}]", es.join(","))
    }).collect();
    format!("{{\"weights\":[{}]}}", ws.join(","))
}
#[allow(dead_code)]
pub fn max_weights_per_vertex(m: &MeshWeightExport) -> usize { m.max_per_vert }
#[allow(dead_code)]
pub fn weight_export_size_mwe(m: &MeshWeightExport) -> usize { m.weights.len() * m.max_per_vert * 8 }
#[allow(dead_code)]
pub fn validate_weights(m: &MeshWeightExport) -> bool {
    m.weights.iter().all(|w| w.iter().all(|(_,v)| v.is_finite() && *v >= 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> Vec<Vec<(u32,f32)>> { vec![vec![(0,0.5),(1,0.5)], vec![(0,1.0)]] }
    #[test] fn test_export() { let m = export_mesh_weights(&data(), 4); assert_eq!(weight_count_mwe(&m), 2); }
    #[test] fn test_count() { let m = export_mesh_weights(&data(), 4); assert_eq!(weight_count_mwe(&m), 2); }
    #[test] fn test_format() { let m = export_mesh_weights(&data(), 4); assert_eq!(weight_format(&m), "float"); }
    #[test] fn test_bytes() { let m = export_mesh_weights(&data(), 4); assert_eq!(weight_to_bytes_mwe(&m).len(), 64); }
    #[test] fn test_json() { let m = export_mesh_weights(&data(), 4); assert!(weight_to_json_mwe(&m).contains("weights")); }
    #[test] fn test_max() { let m = export_mesh_weights(&data(), 4); assert_eq!(max_weights_per_vertex(&m), 4); }
    #[test] fn test_size() { let m = export_mesh_weights(&data(), 4); assert_eq!(weight_export_size_mwe(&m), 64); }
    #[test] fn test_validate() { let m = export_mesh_weights(&data(), 4); assert!(validate_weights(&m)); }
    #[test] fn test_empty() { let m = export_mesh_weights(&[], 4); assert_eq!(weight_count_mwe(&m), 0); }
    #[test] fn test_single() { let m = export_mesh_weights(&[vec![(0,1.0)]], 1); assert_eq!(weight_count_mwe(&m), 1); }
}
