#![allow(dead_code)]
//! Debug information for mesh.

/// Debug info for a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MeshDebugInfo {
    pub vertex_count: usize,
    pub face_count: usize,
    pub edge_count: usize,
    pub has_normals: bool,
    pub has_uvs: bool,
}

/// Produce a debug summary string for a mesh.
#[allow(dead_code)]
pub fn mesh_debug_summary(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[[u32; 3]],
) -> String {
    format!(
        "vertices={} faces={} normals={} uvs={}",
        positions.len(),
        indices.len(),
        normals.len(),
        uvs.len()
    )
}

/// Get info for a specific vertex.
#[allow(dead_code)]
pub fn vertex_info_at(positions: &[[f32; 3]], index: usize) -> Option<String> {
    positions.get(index).map(|p| format!("v[{}] = ({:.4}, {:.4}, {:.4})", index, p[0], p[1], p[2]))
}

/// Get info for a specific face.
#[allow(dead_code)]
pub fn face_info_at(indices: &[[u32; 3]], face: usize) -> Option<String> {
    indices.get(face).map(|tri| format!("f[{}] = ({}, {}, {})", face, tri[0], tri[1], tri[2]))
}

/// Get info for a specific edge (by face index and edge within face).
#[allow(dead_code)]
pub fn edge_info_at(positions: &[[f32; 3]], indices: &[[u32; 3]], face: usize, edge: usize) -> Option<String> {
    if face >= indices.len() || edge >= 3 {
        return None;
    }
    let tri = indices[face];
    let a = tri[edge];
    let b = tri[(edge + 1) % 3];
    let pa = positions[a as usize];
    let pb = positions[b as usize];
    let dx = pb[0] - pa[0];
    let dy = pb[1] - pa[1];
    let dz = pb[2] - pa[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    Some(format!("e[{}-{}] len={:.4}", a, b, len))
}

/// Get topology info (vertices, edges, faces, Euler characteristic estimate).
#[allow(dead_code)]
pub fn mesh_topology_info(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> String {
    use std::collections::HashSet;
    let v = positions.len();
    let f = indices.len();
    let mut edges = HashSet::new();
    for tri in indices {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edges.insert(key);
        }
    }
    let e = edges.len();
    let euler = v as i64 - e as i64 + f as i64;
    format!("V={} E={} F={} euler={}", v, e, f, euler)
}

/// Get bounding box info.
#[allow(dead_code)]
pub fn mesh_bounds_info(positions: &[[f32; 3]]) -> String {
    if positions.is_empty() {
        return "empty mesh".to_string();
    }
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for p in positions {
        for i in 0..3 {
            if p[i] < min[i] { min[i] = p[i]; }
            if p[i] > max[i] { max[i] = p[i]; }
        }
    }
    format!(
        "min=({:.4},{:.4},{:.4}) max=({:.4},{:.4},{:.4})",
        min[0], min[1], min[2], max[0], max[1], max[2]
    )
}

/// Get UV info.
#[allow(dead_code)]
pub fn mesh_uv_info(uvs: &[[f32; 2]]) -> String {
    if uvs.is_empty() {
        return "no UVs".to_string();
    }
    let mut min_u = f32::MAX;
    let mut max_u = f32::MIN;
    let mut min_v = f32::MAX;
    let mut max_v = f32::MIN;
    for uv in uvs {
        if uv[0] < min_u { min_u = uv[0]; }
        if uv[0] > max_u { max_u = uv[0]; }
        if uv[1] < min_v { min_v = uv[1]; }
        if uv[1] > max_v { max_v = uv[1]; }
    }
    format!("uv_count={} u=[{:.4},{:.4}] v=[{:.4},{:.4}]", uvs.len(), min_u, max_u, min_v, max_v)
}

/// Convert debug info to a string.
#[allow(dead_code)]
pub fn debug_to_string(info: &MeshDebugInfo) -> String {
    format!(
        "MeshDebugInfo {{ vertices: {}, faces: {}, edges: {}, normals: {}, uvs: {} }}",
        info.vertex_count, info.face_count, info.edge_count, info.has_normals, info.has_uvs
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_debug_summary() {
        let p = vec![[0.0; 3]; 3];
        let n = vec![[0.0, 0.0, 1.0]; 3];
        let uv = vec![[0.0, 0.0]; 3];
        let i = vec![[0u32, 1, 2]];
        let s = mesh_debug_summary(&p, &n, &uv, &i);
        assert!(s.contains("vertices=3"));
    }

    #[test]
    fn test_vertex_info_at() {
        let p = vec![[1.0, 2.0, 3.0]];
        let info = vertex_info_at(&p, 0);
        assert!(info.is_some());
        assert!(info.unwrap().contains("1.0000"));
    }

    #[test]
    fn test_vertex_info_at_oob() {
        let p = vec![[0.0; 3]];
        assert!(vertex_info_at(&p, 5).is_none());
    }

    #[test]
    fn test_face_info_at() {
        let i = vec![[0u32, 1, 2]];
        let info = face_info_at(&i, 0);
        assert!(info.is_some());
    }

    #[test]
    fn test_edge_info_at() {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let i = vec![[0u32, 1, 2]];
        let info = edge_info_at(&p, &i, 0, 0);
        assert!(info.is_some());
        assert!(info.unwrap().contains("len="));
    }

    #[test]
    fn test_mesh_topology_info() {
        let p = vec![[0.0; 3]; 3];
        let i = vec![[0u32, 1, 2]];
        let info = mesh_topology_info(&p, &i);
        assert!(info.contains("V=3"));
    }

    #[test]
    fn test_mesh_bounds_info() {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let info = mesh_bounds_info(&p);
        assert!(info.contains("min="));
    }

    #[test]
    fn test_mesh_bounds_info_empty() {
        let info = mesh_bounds_info(&[]);
        assert_eq!(info, "empty mesh");
    }

    #[test]
    fn test_mesh_uv_info() {
        let uvs = vec![[0.0, 0.0], [1.0, 1.0]];
        let info = mesh_uv_info(&uvs);
        assert!(info.contains("uv_count=2"));
    }

    #[test]
    fn test_debug_to_string() {
        let info = MeshDebugInfo {
            vertex_count: 10,
            face_count: 5,
            edge_count: 15,
            has_normals: true,
            has_uvs: false,
        };
        let s = debug_to_string(&info);
        assert!(s.contains("vertices: 10"));
    }
}
