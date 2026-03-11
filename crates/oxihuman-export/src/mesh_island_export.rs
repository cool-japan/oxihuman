#![allow(dead_code)]
//! Export mesh island data.

/// Mesh island export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MeshIslandExport {
    pub islands: Vec<IslandData>,
}

/// Data for a single island.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct IslandData {
    pub vertex_indices: Vec<u32>,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
}

/// Export mesh islands.
#[allow(dead_code)]
pub fn export_mesh_islands(islands: Vec<IslandData>) -> MeshIslandExport {
    MeshIslandExport { islands }
}

/// Return island count.
#[allow(dead_code)]
pub fn island_count_export(exp: &MeshIslandExport) -> usize {
    exp.islands.len()
}

/// Return vertex count for an island.
#[allow(dead_code)]
pub fn island_vertex_count(exp: &MeshIslandExport, index: usize) -> usize {
    if index < exp.islands.len() {
        exp.islands[index].vertex_indices.len()
    } else {
        0
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn island_to_json(exp: &MeshIslandExport) -> String {
    let items: Vec<String> = exp
        .islands
        .iter()
        .map(|i| format!("{{\"vertices\":{}}}", i.vertex_indices.len()))
        .collect();
    format!("{{\"islands\":[{}]}}", items.join(","))
}

/// Return bounds for an island.
#[allow(dead_code)]
pub fn island_bounds(exp: &MeshIslandExport, index: usize) -> ([f32; 3], [f32; 3]) {
    if index < exp.islands.len() {
        (exp.islands[index].bounds_min, exp.islands[index].bounds_max)
    } else {
        ([0.0; 3], [0.0; 3])
    }
}

/// Compute center of an island.
#[allow(dead_code)]
pub fn island_center(exp: &MeshIslandExport, index: usize) -> [f32; 3] {
    if index < exp.islands.len() {
        let mn = exp.islands[index].bounds_min;
        let mx = exp.islands[index].bounds_max;
        [
            (mn[0] + mx[0]) * 0.5,
            (mn[1] + mx[1]) * 0.5,
            (mn[2] + mx[2]) * 0.5,
        ]
    } else {
        [0.0; 3]
    }
}

/// Compute export size.
#[allow(dead_code)]
pub fn island_export_size(exp: &MeshIslandExport) -> usize {
    exp.islands.iter().map(|i| i.vertex_indices.len() * 4 + 24).sum()
}

/// Validate islands.
#[allow(dead_code)]
pub fn validate_islands(exp: &MeshIslandExport) -> bool {
    !exp.islands.is_empty() && exp.islands.iter().all(|i| !i.vertex_indices.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_island() -> IslandData {
        IslandData {
            vertex_indices: vec![0, 1, 2],
            bounds_min: [0.0; 3],
            bounds_max: [1.0; 3],
        }
    }

    #[test]
    fn test_export_mesh_islands() {
        let e = export_mesh_islands(vec![sample_island()]);
        assert_eq!(island_count_export(&e), 1);
    }

    #[test]
    fn test_island_vertex_count() {
        let e = export_mesh_islands(vec![sample_island()]);
        assert_eq!(island_vertex_count(&e, 0), 3);
    }

    #[test]
    fn test_island_vertex_count_oob() {
        let e = export_mesh_islands(vec![]);
        assert_eq!(island_vertex_count(&e, 0), 0);
    }

    #[test]
    fn test_island_to_json() {
        let e = export_mesh_islands(vec![sample_island()]);
        let j = island_to_json(&e);
        assert!(j.contains("\"islands\""));
    }

    #[test]
    fn test_island_bounds() {
        let e = export_mesh_islands(vec![sample_island()]);
        let (mn, mx) = island_bounds(&e, 0);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [1.0; 3]);
    }

    #[test]
    fn test_island_center() {
        let e = export_mesh_islands(vec![sample_island()]);
        let c = island_center(&e, 0);
        assert!((c[0] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_island_export_size() {
        let e = export_mesh_islands(vec![sample_island()]);
        assert!(island_export_size(&e) > 0);
    }

    #[test]
    fn test_validate_islands() {
        let e = export_mesh_islands(vec![sample_island()]);
        assert!(validate_islands(&e));
    }

    #[test]
    fn test_validate_empty() {
        let e = export_mesh_islands(vec![]);
        assert!(!validate_islands(&e));
    }

    #[test]
    fn test_island_bounds_oob() {
        let e = export_mesh_islands(vec![]);
        let (mn, mx) = island_bounds(&e, 0);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
    }
}
