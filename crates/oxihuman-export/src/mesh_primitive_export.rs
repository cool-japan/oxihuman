#![allow(dead_code)]
//! Mesh primitive export.

/// Mesh primitive export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MeshPrimitiveExport {
    pub mode: u32,
    pub vertex_count: u32,
    pub index_count: u32,
    pub material_index: Option<u32>,
}

/// Export a mesh primitive.
#[allow(dead_code)]
pub fn export_mesh_primitive(mode: u32, vertex_count: u32, index_count: u32, material_index: Option<u32>) -> MeshPrimitiveExport {
    MeshPrimitiveExport { mode, vertex_count, index_count, material_index }
}

/// Get primitive mode (4 = triangles, 0 = points, etc).
#[allow(dead_code)]
pub fn primitive_mode(e: &MeshPrimitiveExport) -> u32 {
    e.mode
}

/// Get vertex count.
#[allow(dead_code)]
pub fn primitive_vertex_count_mpe(e: &MeshPrimitiveExport) -> u32 {
    e.vertex_count
}

/// Get index count.
#[allow(dead_code)]
pub fn primitive_index_count_mpe(e: &MeshPrimitiveExport) -> u32 {
    e.index_count
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn primitive_to_json(e: &MeshPrimitiveExport) -> String {
    format!(
        "{{\"mode\":{},\"vertices\":{},\"indices\":{}}}",
        e.mode, e.vertex_count, e.index_count
    )
}

/// Get material index.
#[allow(dead_code)]
pub fn primitive_material_index(e: &MeshPrimitiveExport) -> Option<u32> {
    e.material_index
}

/// Get export size estimate.
#[allow(dead_code)]
pub fn primitive_export_size(e: &MeshPrimitiveExport) -> usize {
    (e.vertex_count as usize * 12) + (e.index_count as usize * 4)
}

/// Validate primitive.
#[allow(dead_code)]
pub fn validate_primitive(e: &MeshPrimitiveExport) -> bool {
    e.vertex_count > 0 && (0..=6).contains(&e.mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_mesh_primitive() {
        let e = export_mesh_primitive(4, 100, 300, Some(0));
        assert_eq!(e.mode, 4);
    }

    #[test]
    fn test_primitive_mode() {
        let e = export_mesh_primitive(4, 10, 30, None);
        assert_eq!(primitive_mode(&e), 4);
    }

    #[test]
    fn test_primitive_vertex_count() {
        let e = export_mesh_primitive(4, 50, 150, None);
        assert_eq!(primitive_vertex_count_mpe(&e), 50);
    }

    #[test]
    fn test_primitive_index_count() {
        let e = export_mesh_primitive(4, 50, 150, None);
        assert_eq!(primitive_index_count_mpe(&e), 150);
    }

    #[test]
    fn test_primitive_to_json() {
        let e = export_mesh_primitive(4, 10, 30, None);
        let j = primitive_to_json(&e);
        assert!(j.contains("\"mode\":4"));
    }

    #[test]
    fn test_primitive_material_index() {
        let e = export_mesh_primitive(4, 10, 30, Some(2));
        assert_eq!(primitive_material_index(&e), Some(2));
    }

    #[test]
    fn test_primitive_material_index_none() {
        let e = export_mesh_primitive(4, 10, 30, None);
        assert_eq!(primitive_material_index(&e), None);
    }

    #[test]
    fn test_primitive_export_size() {
        let e = export_mesh_primitive(4, 10, 30, None);
        assert_eq!(primitive_export_size(&e), 10 * 12 + 30 * 4);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_mesh_primitive(4, 10, 30, None);
        assert!(validate_primitive(&e));
    }

    #[test]
    fn test_validate_bad_mode() {
        let e = export_mesh_primitive(10, 10, 30, None);
        assert!(!validate_primitive(&e));
    }
}
