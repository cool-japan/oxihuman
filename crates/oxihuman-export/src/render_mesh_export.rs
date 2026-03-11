#![allow(dead_code)]
//! Export render mesh data.

/// Render mesh export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct RenderMeshExport {
    pub vertex_count: usize,
    pub submeshes: Vec<Submesh>,
}

/// A submesh within a render mesh.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Submesh {
    pub material_name: String,
    pub index_start: u32,
    pub index_count: u32,
}

/// Export a render mesh.
#[allow(dead_code)]
pub fn export_render_mesh(vertex_count: usize, submeshes: &[Submesh]) -> RenderMeshExport {
    RenderMeshExport {
        vertex_count,
        submeshes: submeshes.to_vec(),
    }
}

/// Get vertex count.
#[allow(dead_code)]
pub fn render_mesh_vertex_count(export: &RenderMeshExport) -> usize {
    export.vertex_count
}

/// Get submesh count.
#[allow(dead_code)]
pub fn render_mesh_submesh_count(export: &RenderMeshExport) -> usize {
    export.submeshes.len()
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn render_mesh_to_json(export: &RenderMeshExport) -> String {
    let sub_str: Vec<String> = export.submeshes.iter().map(|s| {
        format!(
            "{{\"material\":\"{}\",\"index_start\":{},\"index_count\":{}}}",
            s.material_name, s.index_start, s.index_count
        )
    }).collect();
    format!(
        "{{\"vertex_count\":{},\"submesh_count\":{},\"submeshes\":[{}]}}",
        export.vertex_count, export.submeshes.len(), sub_str.join(",")
    )
}

/// Get material name for a submesh.
#[allow(dead_code)]
pub fn submesh_material(export: &RenderMeshExport, index: usize) -> Option<&str> {
    export.submeshes.get(index).map(|s| s.material_name.as_str())
}

/// Get index range for a submesh.
#[allow(dead_code)]
pub fn submesh_index_range(export: &RenderMeshExport, index: usize) -> Option<(u32, u32)> {
    export.submeshes.get(index).map(|s| (s.index_start, s.index_start + s.index_count))
}

/// Estimated export size in bytes.
#[allow(dead_code)]
pub fn render_mesh_export_size(export: &RenderMeshExport) -> usize {
    export.vertex_count * 32 + export.submeshes.iter().map(|s| s.index_count as usize * 4).sum::<usize>()
}

/// Validate render mesh data.
#[allow(dead_code)]
pub fn validate_render_mesh(export: &RenderMeshExport) -> bool {
    export.submeshes.iter().all(|s| !s.material_name.is_empty() && s.index_count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RenderMeshExport {
        export_render_mesh(100, &[
            Submesh { material_name: "skin".to_string(), index_start: 0, index_count: 300 },
            Submesh { material_name: "cloth".to_string(), index_start: 300, index_count: 150 },
        ])
    }

    #[test]
    fn test_export_render_mesh() {
        let rm = sample();
        assert_eq!(render_mesh_vertex_count(&rm), 100);
    }

    #[test]
    fn test_submesh_count() {
        let rm = sample();
        assert_eq!(render_mesh_submesh_count(&rm), 2);
    }

    #[test]
    fn test_render_mesh_to_json() {
        let rm = sample();
        let j = render_mesh_to_json(&rm);
        assert!(j.contains("vertex_count"));
        assert!(j.contains("skin"));
    }

    #[test]
    fn test_submesh_material() {
        let rm = sample();
        assert_eq!(submesh_material(&rm, 0), Some("skin"));
    }

    #[test]
    fn test_submesh_material_oob() {
        let rm = sample();
        assert_eq!(submesh_material(&rm, 10), None);
    }

    #[test]
    fn test_submesh_index_range() {
        let rm = sample();
        assert_eq!(submesh_index_range(&rm, 1), Some((300, 450)));
    }

    #[test]
    fn test_render_mesh_export_size() {
        let rm = sample();
        assert!(render_mesh_export_size(&rm) > 0);
    }

    #[test]
    fn test_validate_render_mesh() {
        let rm = sample();
        assert!(validate_render_mesh(&rm));
    }

    #[test]
    fn test_validate_render_mesh_bad() {
        let rm = export_render_mesh(10, &[
            Submesh { material_name: "".to_string(), index_start: 0, index_count: 30 },
        ]);
        assert!(!validate_render_mesh(&rm));
    }

    #[test]
    fn test_empty_submeshes() {
        let rm = export_render_mesh(0, &[]);
        assert_eq!(render_mesh_submesh_count(&rm), 0);
    }
}
