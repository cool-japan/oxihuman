#![allow(dead_code)]

//! Mesh topology data export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshTopologyExport {
    pub vertex_count: usize,
    pub face_count: usize,
    pub edge_count: usize,
    pub is_manifold: bool,
}

#[allow(dead_code)]
pub fn export_mesh_topology(
    vertex_count: usize,
    face_count: usize,
    edge_count: usize,
    is_manifold: bool,
) -> MeshTopologyExport {
    MeshTopologyExport {
        vertex_count,
        face_count,
        edge_count,
        is_manifold,
    }
}

#[allow(dead_code)]
pub fn topology_vertex_count(exp: &MeshTopologyExport) -> usize {
    exp.vertex_count
}

#[allow(dead_code)]
pub fn topology_face_count(exp: &MeshTopologyExport) -> usize {
    exp.face_count
}

#[allow(dead_code)]
pub fn topology_edge_count(exp: &MeshTopologyExport) -> usize {
    exp.edge_count
}

#[allow(dead_code)]
pub fn topology_to_json(exp: &MeshTopologyExport) -> String {
    format!(
        "{{\"vertices\":{},\"faces\":{},\"edges\":{},\"manifold\":{}}}",
        exp.vertex_count, exp.face_count, exp.edge_count, exp.is_manifold
    )
}

#[allow(dead_code)]
pub fn topology_is_manifold(exp: &MeshTopologyExport) -> bool {
    exp.is_manifold
}

#[allow(dead_code)]
pub fn topology_export_size(exp: &MeshTopologyExport) -> usize {
    exp.vertex_count * 12 + exp.face_count * 12
}

#[allow(dead_code)]
pub fn validate_topology(exp: &MeshTopologyExport) -> bool {
    exp.vertex_count > 0 && exp.face_count > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export() {
        let e = export_mesh_topology(100, 200, 300, true);
        assert_eq!(topology_vertex_count(&e), 100);
    }
    #[test]
    fn test_face_count() {
        let e = export_mesh_topology(10, 20, 30, true);
        assert_eq!(topology_face_count(&e), 20);
    }
    #[test]
    fn test_edge_count() {
        let e = export_mesh_topology(10, 20, 30, true);
        assert_eq!(topology_edge_count(&e), 30);
    }
    #[test]
    fn test_manifold() {
        let e = export_mesh_topology(10, 20, 30, false);
        assert!(!topology_is_manifold(&e));
    }
    #[test]
    fn test_to_json() {
        let e = export_mesh_topology(10, 20, 30, true);
        assert!(topology_to_json(&e).contains("\"vertices\":10"));
    }
    #[test]
    fn test_export_size() {
        let e = export_mesh_topology(1, 1, 1, true);
        assert_eq!(topology_export_size(&e), 24);
    }
    #[test]
    fn test_validate() {
        assert!(validate_topology(&export_mesh_topology(10, 20, 30, true)));
    }
    #[test]
    fn test_validate_zero() {
        assert!(!validate_topology(&export_mesh_topology(0, 0, 0, true)));
    }
    #[test]
    fn test_manifold_true() {
        assert!(topology_is_manifold(&export_mesh_topology(
            10, 20, 30, true
        )));
    }
    #[test]
    fn test_large() {
        let e = export_mesh_topology(100000, 200000, 300000, true);
        assert_eq!(topology_vertex_count(&e), 100000);
    }
}
