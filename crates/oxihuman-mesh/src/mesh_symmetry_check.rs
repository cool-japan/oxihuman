#![allow(dead_code)]
//! Mesh symmetry checking and scoring.

/// Result of a symmetry check.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SymmetryCheck {
    pub score: f32,
    pub axis: u8,
    pub plane_origin: [f32; 3],
    pub plane_normal: [f32; 3],
}

/// Check mesh symmetry along X axis, returning a SymmetryCheck.
#[allow(dead_code)]
pub fn check_mesh_symmetry(positions: &[[f32; 3]]) -> SymmetryCheck {
    let score = symmetry_score_mesh(positions, 0);
    SymmetryCheck {
        score,
        axis: 0,
        plane_origin: [0.0; 3],
        plane_normal: [1.0, 0.0, 0.0],
    }
}

/// Compute a symmetry score along the given axis (0=X,1=Y,2=Z).
#[allow(dead_code)]
pub fn symmetry_score_mesh(positions: &[[f32; 3]], axis: u8) -> f32 {
    if positions.is_empty() {
        return 1.0;
    }
    let ax = axis as usize;
    let mut total_error = 0.0f32;
    let mut count = 0u32;
    for (i, p) in positions.iter().enumerate() {
        let mut best = f32::MAX;
        for (j, q) in positions.iter().enumerate() {
            if i == j {
                continue;
            }
            let mut mirrored = *q;
            mirrored[ax] = -mirrored[ax];
            let dx = p[0] - mirrored[0];
            let dy = p[1] - mirrored[1];
            let dz = p[2] - mirrored[2];
            let d = dx * dx + dy * dy + dz * dz;
            if d < best {
                best = d;
            }
        }
        if best < f32::MAX {
            total_error += best.sqrt();
            count += 1;
        }
    }
    if count == 0 {
        return 1.0;
    }
    let avg = total_error / count as f32;
    (1.0 - avg.min(1.0)).max(0.0)
}

/// Find the best symmetry plane among X/Y/Z.
#[allow(dead_code)]
pub fn find_symmetry_plane(positions: &[[f32; 3]]) -> [f32; 3] {
    let mut best_axis = 0u8;
    let mut best_score = -1.0f32;
    for ax in 0..3u8 {
        let s = symmetry_score_mesh(positions, ax);
        if s > best_score {
            best_score = s;
            best_axis = ax;
        }
    }
    let mut normal = [0.0f32; 3];
    normal[best_axis as usize] = 1.0;
    normal
}

/// Build a mirror vertex map along an axis.
#[allow(dead_code)]
pub fn mirror_vertex_map(positions: &[[f32; 3]], axis: u8) -> Vec<Option<usize>> {
    let ax = axis as usize;
    let threshold = 0.001f32;
    positions
        .iter()
        .map(|p| {
            let mut mirrored = *p;
            mirrored[ax] = -mirrored[ax];
            positions.iter().position(|q| {
                let dx = q[0] - mirrored[0];
                let dy = q[1] - mirrored[1];
                let dz = q[2] - mirrored[2];
                (dx * dx + dy * dy + dz * dz).sqrt() < threshold
            })
        })
        .collect()
}

/// Get symmetry error at a specific vertex.
#[allow(dead_code)]
pub fn symmetry_error_at(positions: &[[f32; 3]], vertex: usize, axis: u8) -> f32 {
    if vertex >= positions.len() {
        return 0.0;
    }
    let ax = axis as usize;
    let p = positions[vertex];
    let mut mirrored = p;
    mirrored[ax] = -mirrored[ax];
    let mut best = f32::MAX;
    for (i, q) in positions.iter().enumerate() {
        if i == vertex {
            continue;
        }
        let dx = q[0] - mirrored[0];
        let dy = q[1] - mirrored[1];
        let dz = q[2] - mirrored[2];
        let d = (dx * dx + dy * dy + dz * dz).sqrt();
        if d < best {
            best = d;
        }
    }
    if best == f32::MAX { 0.0 } else { best }
}

/// Serialize symmetry check to JSON.
#[allow(dead_code)]
pub fn symmetry_to_json(check: &SymmetryCheck) -> String {
    format!(
        "{{\"score\":{:.4},\"axis\":{}}}",
        check.score, check.axis
    )
}

/// Check if the mesh is considered symmetric (score >= threshold).
#[allow(dead_code)]
pub fn is_symmetric(check: &SymmetryCheck, threshold: f32) -> bool {
    check.score >= threshold
}

/// Return which axis (0,1,2) the symmetry check uses.
#[allow(dead_code)]
pub fn symmetry_axis_mesh(check: &SymmetryCheck) -> u8 {
    check.axis
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_mesh_symmetry_empty() {
        let c = check_mesh_symmetry(&[]);
        assert_eq!(c.score, 1.0);
    }

    #[test]
    fn test_symmetric_points() {
        let pts = [[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let c = check_mesh_symmetry(&pts);
        assert!(c.score > 0.9);
    }

    #[test]
    fn test_symmetry_score_single() {
        let pts = [[0.0, 0.0, 0.0]];
        let s = symmetry_score_mesh(&pts, 0);
        assert_eq!(s, 1.0);
    }

    #[test]
    fn test_find_symmetry_plane() {
        let pts = [[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let normal = find_symmetry_plane(&pts);
        assert_eq!(normal[0], 1.0);
    }

    #[test]
    fn test_mirror_vertex_map() {
        let pts = [[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let map = mirror_vertex_map(&pts, 0);
        assert_eq!(map[0], Some(1));
        assert_eq!(map[1], Some(0));
    }

    #[test]
    fn test_symmetry_error_at_oob() {
        let pts = [[0.0; 3]];
        assert_eq!(symmetry_error_at(&pts, 5, 0), 0.0);
    }

    #[test]
    fn test_symmetry_to_json() {
        let c = check_mesh_symmetry(&[]);
        let j = symmetry_to_json(&c);
        assert!(j.contains("\"score\""));
    }

    #[test]
    fn test_is_symmetric() {
        let c = SymmetryCheck {
            score: 0.95,
            axis: 0,
            plane_origin: [0.0; 3],
            plane_normal: [1.0, 0.0, 0.0],
        };
        assert!(is_symmetric(&c, 0.9));
        assert!(!is_symmetric(&c, 0.99));
    }

    #[test]
    fn test_symmetry_axis() {
        let c = SymmetryCheck {
            score: 1.0,
            axis: 2,
            plane_origin: [0.0; 3],
            plane_normal: [0.0, 0.0, 1.0],
        };
        assert_eq!(symmetry_axis_mesh(&c), 2);
    }

    #[test]
    fn test_symmetry_error_at_symmetric() {
        let pts = [[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let err = symmetry_error_at(&pts, 0, 0);
        assert!(err < 0.001);
    }
}
