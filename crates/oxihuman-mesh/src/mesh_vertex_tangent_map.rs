#![allow(dead_code)]
//! Per-vertex tangent map storage.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexTangentMap { tangents: Vec<[f32;4]> }

#[allow(dead_code)]
pub fn build_tangent_map(tangents: &[[f32;4]]) -> VertexTangentMap { VertexTangentMap { tangents: tangents.to_vec() } }
#[allow(dead_code)]
pub fn tangent_at_vertex(tm: &VertexTangentMap, idx: usize) -> [f32;4] { tm.tangents.get(idx).copied().unwrap_or([0.0;4]) }
#[allow(dead_code)]
pub fn tangent_map_count(tm: &VertexTangentMap) -> usize { tm.tangents.len() }
#[allow(dead_code)]
pub fn tangent_map_to_bytes(tm: &VertexTangentMap) -> Vec<u8> {
    let mut b = Vec::with_capacity(tm.tangents.len()*16);
    for t in &tm.tangents { for &v in t { b.extend_from_slice(&v.to_le_bytes()); } }
    b
}
#[allow(dead_code)]
pub fn tangent_map_to_json(tm: &VertexTangentMap) -> String {
    let ts: Vec<String> = tm.tangents.iter().map(|t| format!("[{:.6},{:.6},{:.6},{:.6}]",t[0],t[1],t[2],t[3])).collect();
    format!("{{\"tangents\":[{}]}}", ts.join(","))
}
#[allow(dead_code)]
pub fn clear_tangent_map(tm: &mut VertexTangentMap) { tm.tangents.clear(); }
#[allow(dead_code)]
pub fn validate_tangent_map(tm: &VertexTangentMap) -> bool {
    tm.tangents.iter().all(|t| {
        let l = (t[0]*t[0]+t[1]*t[1]+t[2]*t[2]).sqrt();
        (l - 1.0).abs() < 0.01 || l < 1e-10
    })
}
#[allow(dead_code)]
pub fn tangent_sign_at(tm: &VertexTangentMap, idx: usize) -> f32 {
    tm.tangents.get(idx).map_or(0.0, |t| t[3])
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> Vec<[f32;4]> { vec![[1.0,0.0,0.0,1.0],[0.0,1.0,0.0,-1.0],[0.0,0.0,1.0,1.0]] }
    #[test] fn test_build() { let tm = build_tangent_map(&data()); assert_eq!(tangent_map_count(&tm), 3); }
    #[test] fn test_at() { let tm = build_tangent_map(&data()); assert!((tangent_at_vertex(&tm, 0)[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_count() { let tm = build_tangent_map(&data()); assert_eq!(tangent_map_count(&tm), 3); }
    #[test] fn test_bytes() { let tm = build_tangent_map(&data()); assert_eq!(tangent_map_to_bytes(&tm).len(), 48); }
    #[test] fn test_json() { let tm = build_tangent_map(&data()); assert!(tangent_map_to_json(&tm).contains("tangents")); }
    #[test] fn test_clear() { let mut tm = build_tangent_map(&data()); clear_tangent_map(&mut tm); assert_eq!(tangent_map_count(&tm), 0); }
    #[test] fn test_validate() { let tm = build_tangent_map(&data()); assert!(validate_tangent_map(&tm)); }
    #[test] fn test_sign() { let tm = build_tangent_map(&data()); assert!((tangent_sign_at(&tm, 1) - (-1.0)).abs() < 1e-6); }
    #[test] fn test_oob() { let tm = build_tangent_map(&[]); assert!((tangent_at_vertex(&tm, 5)[0]).abs() < 1e-9); }
    #[test] fn test_sign_oob() { let tm = build_tangent_map(&[]); assert!((tangent_sign_at(&tm, 5) - 0.0).abs() < 1e-9); }
}
