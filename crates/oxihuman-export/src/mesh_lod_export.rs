#![allow(dead_code)]

//! Mesh LOD export utilities.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshLodExport {
    pub levels: Vec<LodMeshLevel>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodMeshLevel {
    pub vertex_count: usize,
    pub index_count: usize,
    pub screen_coverage: f32,
}

#[allow(dead_code)]
pub fn export_mesh_lod(levels: Vec<LodMeshLevel>) -> MeshLodExport {
    MeshLodExport { levels }
}

#[allow(dead_code)]
pub fn lod_mesh_count(exp: &MeshLodExport) -> usize { exp.levels.len() }

#[allow(dead_code)]
pub fn lod_at(exp: &MeshLodExport, index: usize) -> Option<&LodMeshLevel> { exp.levels.get(index) }

#[allow(dead_code)]
pub fn lod_screen_coverage(level: &LodMeshLevel) -> f32 { level.screen_coverage }

#[allow(dead_code)]
pub fn lod_to_json_mle(exp: &MeshLodExport) -> String {
    let items: Vec<String> = exp.levels.iter().map(|l|
        format!("{{\"vertices\":{},\"indices\":{},\"coverage\":{:.4}}}", l.vertex_count, l.index_count, l.screen_coverage)
    ).collect();
    format!("{{\"lod_count\":{},\"levels\":[{}]}}", exp.levels.len(), items.join(","))
}

#[allow(dead_code)]
pub fn lod_total_vertices(exp: &MeshLodExport) -> usize {
    exp.levels.iter().map(|l| l.vertex_count).sum()
}

#[allow(dead_code)]
pub fn lod_export_size(exp: &MeshLodExport) -> usize {
    exp.levels.iter().map(|l| l.vertex_count * 12 + l.index_count * 4).sum()
}

#[allow(dead_code)]
pub fn validate_mesh_lod(exp: &MeshLodExport) -> bool {
    !exp.levels.is_empty() && exp.levels.iter().all(|l| l.vertex_count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn lvl(v: usize, i: usize, c: f32) -> LodMeshLevel { LodMeshLevel { vertex_count: v, index_count: i, screen_coverage: c } }

    #[test]
    fn test_export() { let e = export_mesh_lod(vec![lvl(100, 300, 1.0)]); assert_eq!(lod_mesh_count(&e), 1); }
    #[test]
    fn test_lod_at() { let e = export_mesh_lod(vec![lvl(50, 150, 0.5)]); assert!(lod_at(&e, 0).is_some()); assert!(lod_at(&e, 1).is_none()); }
    #[test]
    fn test_coverage() { let l = lvl(10, 30, 0.75); assert!((lod_screen_coverage(&l) - 0.75).abs() < 1e-6); }
    #[test]
    fn test_to_json() { let e = export_mesh_lod(vec![lvl(10, 30, 1.0)]); assert!(lod_to_json_mle(&e).contains("\"lod_count\":1")); }
    #[test]
    fn test_total_verts() { let e = export_mesh_lod(vec![lvl(10, 30, 1.0), lvl(5, 15, 0.5)]); assert_eq!(lod_total_vertices(&e), 15); }
    #[test]
    fn test_export_size() { let e = export_mesh_lod(vec![lvl(1, 3, 1.0)]); assert_eq!(lod_export_size(&e), 12 + 3*4); }
    #[test]
    fn test_validate() { let e = export_mesh_lod(vec![lvl(10, 30, 1.0)]); assert!(validate_mesh_lod(&e)); }
    #[test]
    fn test_validate_empty() { let e = export_mesh_lod(vec![]); assert!(!validate_mesh_lod(&e)); }
    #[test]
    fn test_validate_zero_verts() { let e = export_mesh_lod(vec![lvl(0, 0, 0.0)]); assert!(!validate_mesh_lod(&e)); }
    #[test]
    fn test_multiple_levels() { let e = export_mesh_lod(vec![lvl(100,300,1.0),lvl(50,150,0.5),lvl(25,75,0.25)]); assert_eq!(lod_mesh_count(&e), 3); }
}
