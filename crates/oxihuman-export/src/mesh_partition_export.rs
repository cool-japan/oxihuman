#![allow(dead_code)]

//! Mesh partition export for sub-mesh segmentation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshPartitionExport {
    pub partitions: Vec<PartitionEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PartitionEntry {
    pub face_start: usize,
    pub face_end: usize,
    pub material: String,
}

#[allow(dead_code)]
pub fn export_mesh_partitions(partitions: Vec<PartitionEntry>) -> MeshPartitionExport {
    MeshPartitionExport { partitions }
}

#[allow(dead_code)]
pub fn partition_count_mpe(exp: &MeshPartitionExport) -> usize { exp.partitions.len() }

#[allow(dead_code)]
pub fn partition_face_range(exp: &MeshPartitionExport, idx: usize) -> Option<(usize, usize)> {
    exp.partitions.get(idx).map(|p| (p.face_start, p.face_end))
}

#[allow(dead_code)]
pub fn partition_to_json(exp: &MeshPartitionExport) -> String {
    let items: Vec<String> = exp.partitions.iter().map(|p|
        format!("{{\"start\":{},\"end\":{},\"material\":\"{}\"}}", p.face_start, p.face_end, p.material)
    ).collect();
    format!("{{\"count\":{},\"partitions\":[{}]}}", exp.partitions.len(), items.join(","))
}

#[allow(dead_code)]
pub fn partition_material(exp: &MeshPartitionExport, idx: usize) -> Option<&str> {
    exp.partitions.get(idx).map(|p| p.material.as_str())
}

#[allow(dead_code)]
pub fn partition_export_size(exp: &MeshPartitionExport) -> usize {
    exp.partitions.iter().map(|p| (p.face_end - p.face_start) * 12).sum()
}

#[allow(dead_code)]
pub fn validate_partitions(exp: &MeshPartitionExport) -> bool {
    !exp.partitions.is_empty() && exp.partitions.iter().all(|p| p.face_start <= p.face_end)
}

#[allow(dead_code)]
pub fn partition_bounds(exp: &MeshPartitionExport) -> (usize, usize) {
    let start = exp.partitions.iter().map(|p| p.face_start).min().unwrap_or(0);
    let end = exp.partitions.iter().map(|p| p.face_end).max().unwrap_or(0);
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pe(s: usize, e: usize) -> PartitionEntry { PartitionEntry { face_start: s, face_end: e, material: "mat".into() } }

    #[test]
    fn test_export() { let e = export_mesh_partitions(vec![pe(0, 10)]); assert_eq!(partition_count_mpe(&e), 1); }
    #[test]
    fn test_face_range() { let e = export_mesh_partitions(vec![pe(0, 10)]); assert_eq!(partition_face_range(&e, 0), Some((0, 10))); }
    #[test]
    fn test_face_range_none() { let e = export_mesh_partitions(vec![]); assert_eq!(partition_face_range(&e, 0), None); }
    #[test]
    fn test_to_json() { let e = export_mesh_partitions(vec![pe(0, 5)]); assert!(partition_to_json(&e).contains("\"count\":1")); }
    #[test]
    fn test_material() { let e = export_mesh_partitions(vec![pe(0, 5)]); assert_eq!(partition_material(&e, 0), Some("mat")); }
    #[test]
    fn test_export_size() { let e = export_mesh_partitions(vec![pe(0, 10)]); assert_eq!(partition_export_size(&e), 120); }
    #[test]
    fn test_validate() { assert!(validate_partitions(&export_mesh_partitions(vec![pe(0, 10)]))); }
    #[test]
    fn test_validate_empty() { assert!(!validate_partitions(&export_mesh_partitions(vec![]))); }
    #[test]
    fn test_bounds() { let e = export_mesh_partitions(vec![pe(5, 15), pe(0, 10)]); assert_eq!(partition_bounds(&e), (0, 15)); }
    #[test]
    fn test_multiple() { let e = export_mesh_partitions(vec![pe(0,5),pe(5,10)]); assert_eq!(partition_count_mpe(&e), 2); }
}
