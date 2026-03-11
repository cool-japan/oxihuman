#![allow(dead_code)]
//! Export submesh data.

/// Submesh export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SubmeshExport {
    pub submeshes: Vec<SubmeshEntry>,
}

/// A single submesh entry.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SubmeshEntry {
    pub material_index: u32,
    pub vertex_start: u32,
    pub vertex_count: u32,
    pub index_start: u32,
    pub index_count: u32,
}

/// Export submeshes.
#[allow(dead_code)]
pub fn export_submeshes(entries: &[(u32, u32, u32, u32, u32)]) -> SubmeshExport {
    SubmeshExport {
        submeshes: entries
            .iter()
            .map(|&(mi, vs, vc, is, ic)| SubmeshEntry {
                material_index: mi,
                vertex_start: vs,
                vertex_count: vc,
                index_start: is,
                index_count: ic,
            })
            .collect(),
    }
}

/// Return submesh count.
#[allow(dead_code)]
pub fn submesh_count_export(exp: &SubmeshExport) -> usize {
    exp.submeshes.len()
}

/// Return material index for a submesh.
#[allow(dead_code)]
pub fn submesh_material_index(exp: &SubmeshExport, index: usize) -> u32 {
    if index < exp.submeshes.len() {
        exp.submeshes[index].material_index
    } else {
        0
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn submesh_to_json_export(exp: &SubmeshExport) -> String {
    let items: Vec<String> = exp
        .submeshes
        .iter()
        .map(|s| {
            format!(
                "{{\"material\":{},\"vertices\":{},\"indices\":{}}}",
                s.material_index, s.vertex_count, s.index_count
            )
        })
        .collect();
    format!("{{\"submeshes\":[{}]}}", items.join(","))
}

/// Get index range for a submesh.
#[allow(dead_code)]
pub fn submesh_index_range_export(exp: &SubmeshExport, index: usize) -> (u32, u32) {
    if index < exp.submeshes.len() {
        let s = &exp.submeshes[index];
        (s.index_start, s.index_start + s.index_count)
    } else {
        (0, 0)
    }
}

/// Get vertex range for a submesh.
#[allow(dead_code)]
pub fn submesh_vertex_range_export(exp: &SubmeshExport, index: usize) -> (u32, u32) {
    if index < exp.submeshes.len() {
        let s = &exp.submeshes[index];
        (s.vertex_start, s.vertex_start + s.vertex_count)
    } else {
        (0, 0)
    }
}

/// Compute export size.
#[allow(dead_code)]
pub fn submesh_export_size(exp: &SubmeshExport) -> usize {
    exp.submeshes.len() * 20 // 5 u32s * 4 bytes
}

/// Validate submeshes.
#[allow(dead_code)]
pub fn validate_submeshes(exp: &SubmeshExport) -> bool {
    !exp.submeshes.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_submeshes() {
        let e = export_submeshes(&[(0, 0, 100, 0, 300)]);
        assert_eq!(submesh_count_export(&e), 1);
    }

    #[test]
    fn test_submesh_material_index() {
        let e = export_submeshes(&[(5, 0, 10, 0, 30)]);
        assert_eq!(submesh_material_index(&e, 0), 5);
    }

    #[test]
    fn test_submesh_material_index_oob() {
        let e = export_submeshes(&[]);
        assert_eq!(submesh_material_index(&e, 0), 0);
    }

    #[test]
    fn test_submesh_to_json() {
        let e = export_submeshes(&[(0, 0, 10, 0, 30)]);
        let j = submesh_to_json_export(&e);
        assert!(j.contains("\"submeshes\""));
    }

    #[test]
    fn test_submesh_index_range() {
        let e = export_submeshes(&[(0, 0, 10, 20, 30)]);
        assert_eq!(submesh_index_range_export(&e, 0), (20, 50));
    }

    #[test]
    fn test_submesh_vertex_range() {
        let e = export_submeshes(&[(0, 10, 50, 0, 150)]);
        assert_eq!(submesh_vertex_range_export(&e, 0), (10, 60));
    }

    #[test]
    fn test_submesh_export_size() {
        let e = export_submeshes(&[(0, 0, 10, 0, 30), (1, 10, 20, 30, 60)]);
        assert_eq!(submesh_export_size(&e), 40);
    }

    #[test]
    fn test_validate_submeshes() {
        let e = export_submeshes(&[(0, 0, 10, 0, 30)]);
        assert!(validate_submeshes(&e));
    }

    #[test]
    fn test_validate_empty() {
        let e = export_submeshes(&[]);
        assert!(!validate_submeshes(&e));
    }

    #[test]
    fn test_multiple_submeshes() {
        let e = export_submeshes(&[(0, 0, 10, 0, 30), (1, 10, 20, 30, 60)]);
        assert_eq!(submesh_count_export(&e), 2);
    }
}
