#![allow(dead_code)]

//! Geometry data export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeometryExport {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

#[allow(dead_code)]
pub fn export_geometry(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> GeometryExport {
    GeometryExport { positions, indices }
}

#[allow(dead_code)]
pub fn geometry_vertex_count_ge(exp: &GeometryExport) -> usize { exp.positions.len() }

#[allow(dead_code)]
pub fn geometry_index_count_ge(exp: &GeometryExport) -> usize { exp.indices.len() }

#[allow(dead_code)]
pub fn geometry_to_json(exp: &GeometryExport) -> String {
    format!("{{\"vertices\":{},\"indices\":{}}}", exp.positions.len(), exp.indices.len())
}

#[allow(dead_code)]
pub fn geometry_bounds(exp: &GeometryExport) -> ([f32; 3], [f32; 3]) {
    if exp.positions.is_empty() { return ([0.0;3],[0.0;3]); }
    let mut lo = exp.positions[0];
    let mut hi = exp.positions[0];
    for p in &exp.positions {
        for i in 0..3 { if p[i]<lo[i]{lo[i]=p[i];} else if p[i]>hi[i]{hi[i]=p[i];} }
    }
    (lo, hi)
}

#[allow(dead_code)]
pub fn geometry_to_bytes(exp: &GeometryExport) -> Vec<u8> {
    let mut bytes = Vec::new();
    for p in &exp.positions { for &v in p { bytes.extend_from_slice(&v.to_le_bytes()); } }
    for &i in &exp.indices { bytes.extend_from_slice(&i.to_le_bytes()); }
    bytes
}

#[allow(dead_code)]
pub fn geometry_export_size(exp: &GeometryExport) -> usize {
    exp.positions.len() * 12 + exp.indices.len() * 4
}

#[allow(dead_code)]
pub fn validate_geometry(exp: &GeometryExport) -> bool {
    !exp.positions.is_empty() && exp.indices.len().is_multiple_of(3)
        && exp.indices.iter().all(|&i| (i as usize) < exp.positions.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export() { let e = export_geometry(vec![[0.0;3];3], vec![0,1,2]); assert_eq!(geometry_vertex_count_ge(&e), 3); }
    #[test]
    fn test_index_count() { let e = export_geometry(vec![[0.0;3];3], vec![0,1,2]); assert_eq!(geometry_index_count_ge(&e), 3); }
    #[test]
    fn test_to_json() { let e = export_geometry(vec![[0.0;3]], vec![]); assert!(geometry_to_json(&e).contains("\"vertices\":1")); }
    #[test]
    fn test_bounds() { let e = export_geometry(vec![[0.0,0.0,0.0],[1.0,2.0,3.0]], vec![]); let (_,hi)=geometry_bounds(&e); assert!((hi[2]-3.0).abs()<1e-6); }
    #[test]
    fn test_bounds_empty() { let e = export_geometry(vec![], vec![]); let (lo,_)=geometry_bounds(&e); assert!((lo[0]).abs()<1e-6); }
    #[test]
    fn test_to_bytes() { let e = export_geometry(vec![[1.0,0.0,0.0]], vec![0]); assert_eq!(geometry_to_bytes(&e).len(), 16); }
    #[test]
    fn test_export_size() { let e = export_geometry(vec![[0.0;3]], vec![0,1,2]); assert_eq!(geometry_export_size(&e), 24); }
    #[test]
    fn test_validate() { assert!(validate_geometry(&export_geometry(vec![[0.0;3];3], vec![0,1,2]))); }
    #[test]
    fn test_validate_bad_idx() { assert!(!validate_geometry(&export_geometry(vec![[0.0;3]], vec![0,1,2]))); }
    #[test]
    fn test_validate_empty() { assert!(!validate_geometry(&export_geometry(vec![], vec![]))); }
}
