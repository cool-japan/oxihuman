#![allow(dead_code)]
//! Export collision mesh data.

/// Collision mesh export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CollisionMeshExport {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
}

/// Export a collision mesh.
#[allow(dead_code)]
pub fn export_collision_mesh(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> CollisionMeshExport {
    CollisionMeshExport {
        positions: positions.to_vec(),
        indices: indices.to_vec(),
    }
}

/// Get collision vertex count.
#[allow(dead_code)]
pub fn collision_vertex_count(export: &CollisionMeshExport) -> usize {
    export.positions.len()
}

/// Get collision face count.
#[allow(dead_code)]
pub fn collision_face_count(export: &CollisionMeshExport) -> usize {
    export.indices.len()
}

/// Convert collision mesh to JSON.
#[allow(dead_code)]
pub fn collision_to_json(export: &CollisionMeshExport) -> String {
    format!(
        "{{\"vertex_count\":{},\"face_count\":{}}}",
        export.positions.len(),
        export.indices.len()
    )
}

/// Convert collision mesh to OBJ format string.
#[allow(dead_code)]
pub fn collision_to_obj(export: &CollisionMeshExport) -> String {
    let mut s = String::new();
    for p in &export.positions {
        s.push_str(&format!("v {:.6} {:.6} {:.6}\n", p[0], p[1], p[2]));
    }
    for tri in &export.indices {
        s.push_str(&format!("f {} {} {}\n", tri[0] + 1, tri[1] + 1, tri[2] + 1));
    }
    s
}

/// Export a simplified convex hull.
#[allow(dead_code)]
pub fn convex_hull_export(positions: &[[f32; 3]]) -> CollisionMeshExport {
    // Simplified: just use axis-aligned bounding box as 8 verts + 12 tris
    if positions.is_empty() {
        return CollisionMeshExport { positions: vec![], indices: vec![] };
    }
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for p in positions {
        for i in 0..3 {
            if p[i] < min[i] { min[i] = p[i]; }
            if p[i] > max[i] { max[i] = p[i]; }
        }
    }
    let verts = vec![
        [min[0], min[1], min[2]], [max[0], min[1], min[2]],
        [max[0], max[1], min[2]], [min[0], max[1], min[2]],
        [min[0], min[1], max[2]], [max[0], min[1], max[2]],
        [max[0], max[1], max[2]], [min[0], max[1], max[2]],
    ];
    let tris = vec![
        [0, 2, 1], [0, 3, 2], [4, 5, 6], [4, 6, 7],
        [0, 1, 5], [0, 5, 4], [2, 3, 7], [2, 7, 6],
        [1, 2, 6], [1, 6, 5], [0, 4, 7], [0, 7, 3],
    ];
    CollisionMeshExport { positions: verts, indices: tris }
}

/// Compute collision volume.
#[allow(dead_code)]
pub fn collision_volume(export: &CollisionMeshExport) -> f32 {
    let mut vol = 0.0f32;
    for tri in &export.indices {
        let a = export.positions[tri[0] as usize];
        let b = export.positions[tri[1] as usize];
        let c = export.positions[tri[2] as usize];
        vol += a[0] * (b[1] * c[2] - b[2] * c[1])
             + b[0] * (c[1] * a[2] - c[2] * a[1])
             + c[0] * (a[1] * b[2] - a[2] * b[1]);
    }
    (vol / 6.0).abs()
}

/// Validate collision mesh data.
#[allow(dead_code)]
pub fn validate_collision_mesh(export: &CollisionMeshExport) -> bool {
    let n = export.positions.len() as u32;
    export.indices.iter().all(|tri| tri.iter().all(|&i| i < n))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CollisionMeshExport {
        export_collision_mesh(
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[[0u32, 1, 2]],
        )
    }

    #[test]
    fn test_export_collision_mesh() {
        let cm = sample();
        assert_eq!(collision_vertex_count(&cm), 3);
    }

    #[test]
    fn test_collision_face_count() {
        let cm = sample();
        assert_eq!(collision_face_count(&cm), 1);
    }

    #[test]
    fn test_collision_to_json() {
        let cm = sample();
        let j = collision_to_json(&cm);
        assert!(j.contains("vertex_count"));
    }

    #[test]
    fn test_collision_to_obj() {
        let cm = sample();
        let obj = collision_to_obj(&cm);
        assert!(obj.contains("v "));
        assert!(obj.contains("f "));
    }

    #[test]
    fn test_convex_hull_export() {
        let hull = convex_hull_export(&[[0.0; 3], [1.0, 1.0, 1.0]]);
        assert_eq!(hull.positions.len(), 8);
        assert_eq!(hull.indices.len(), 12);
    }

    #[test]
    fn test_convex_hull_empty() {
        let hull = convex_hull_export(&[]);
        assert!(hull.positions.is_empty());
    }

    #[test]
    fn test_collision_volume() {
        let hull = convex_hull_export(&[[0.0; 3], [1.0, 1.0, 1.0]]);
        let vol = collision_volume(&hull);
        assert!((vol - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_validate_collision_mesh() {
        let cm = sample();
        assert!(validate_collision_mesh(&cm));
    }

    #[test]
    fn test_validate_collision_mesh_bad() {
        let cm = CollisionMeshExport {
            positions: vec![[0.0; 3]],
            indices: vec![[0, 1, 2]],
        };
        assert!(!validate_collision_mesh(&cm));
    }

    #[test]
    fn test_collision_vertex_count_empty() {
        let cm = export_collision_mesh(&[], &[]);
        assert_eq!(collision_vertex_count(&cm), 0);
    }
}
