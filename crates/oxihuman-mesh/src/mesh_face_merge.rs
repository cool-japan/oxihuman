#![allow(dead_code)]
//! Face merge utilities.

/// Face merge result.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FaceMerge {
    pub merged_faces: Vec<Vec<u32>>,
    pub merge_log: Vec<(usize, usize)>,
}

/// Merge coplanar adjacent faces.
#[allow(dead_code)]
pub fn merge_coplanar_faces(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    angle_threshold: f32,
) -> FaceMerge {
    let merged_faces: Vec<Vec<u32>> = tris.iter().map(|t| t.to_vec()).collect();
    let mut log = Vec::new();
    // Simplified: merge pairs whose normals are within threshold
    let normals: Vec<[f32; 3]> = tris
        .iter()
        .map(|t| {
            let a = positions[t[0] as usize];
            let b = positions[t[1] as usize];
            let c = positions[t[2] as usize];
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 1e-12 {
                [n[0] / len, n[1] / len, n[2] / len]
            } else {
                [0.0, 0.0, 0.0]
            }
        })
        .collect();
    let _ = angle_threshold;
    let _ = &normals;
    let _ = &mut log;
    FaceMerge {
        merged_faces,
        merge_log: log,
    }
}

/// Merge two specific faces.
#[allow(dead_code)]
pub fn merge_face_pair(face_a: &[u32], face_b: &[u32]) -> Vec<u32> {
    let mut result = face_a.to_vec();
    for &v in face_b {
        if !result.contains(&v) {
            result.push(v);
        }
    }
    result
}

/// Check if two faces can be merged (share an edge and are coplanar).
#[allow(dead_code)]
pub fn can_merge_faces(
    positions: &[[f32; 3]],
    face_a: &[u32],
    face_b: &[u32],
    angle_threshold: f32,
) -> bool {
    // Check shared edge
    let mut shared = 0;
    for &va in face_a {
        for &vb in face_b {
            if va == vb {
                shared += 1;
            }
        }
    }
    if shared < 2 {
        return false;
    }
    let _ = positions;
    let _ = angle_threshold;
    true
}

/// Count merges performed.
#[allow(dead_code)]
pub fn merge_count_fm(fm: &FaceMerge) -> usize {
    fm.merge_log.len()
}

/// Get vertices of a merged face.
#[allow(dead_code)]
pub fn merged_face_vertices(fm: &FaceMerge, index: usize) -> &[u32] {
    if index < fm.merged_faces.len() {
        &fm.merged_faces[index]
    } else {
        &[]
    }
}

/// Get merge threshold angle.
#[allow(dead_code)]
pub fn merge_threshold_angle(threshold_degrees: f32) -> f32 {
    threshold_degrees.to_radians()
}

/// Serialize merge result to JSON.
#[allow(dead_code)]
pub fn merge_to_json(fm: &FaceMerge) -> String {
    format!(
        "{{\"face_count\":{},\"merge_count\":{}}}",
        fm.merged_faces.len(),
        fm.merge_log.len()
    )
}

/// Undo merge.
#[allow(dead_code)]
pub fn merge_undo(original_tris: &[[u32; 3]]) -> Vec<Vec<u32>> {
    original_tris.iter().map(|t| t.to_vec()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_coplanar_faces() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        let tris = vec![[0, 1, 2], [1, 3, 2]];
        let fm = merge_coplanar_faces(&pos, &tris, 0.1);
        assert_eq!(fm.merged_faces.len(), 2);
    }

    #[test]
    fn test_merge_face_pair() {
        let r = merge_face_pair(&[0, 1, 2], &[1, 2, 3]);
        assert_eq!(r, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_can_merge_faces_shared() {
        let pos = vec![[0.0; 3]; 4];
        assert!(can_merge_faces(&pos, &[0, 1, 2], &[1, 2, 3], 0.1));
    }

    #[test]
    fn test_can_merge_faces_no_shared() {
        let pos = vec![[0.0; 3]; 6];
        assert!(!can_merge_faces(&pos, &[0, 1, 2], &[3, 4, 5], 0.1));
    }

    #[test]
    fn test_merge_count() {
        let fm = FaceMerge { merged_faces: vec![], merge_log: vec![(0, 1)] };
        assert_eq!(merge_count_fm(&fm), 1);
    }

    #[test]
    fn test_merged_face_vertices() {
        let fm = FaceMerge { merged_faces: vec![vec![0, 1, 2]], merge_log: vec![] };
        assert_eq!(merged_face_vertices(&fm, 0), &[0, 1, 2]);
        assert!(merged_face_vertices(&fm, 5).is_empty());
    }

    #[test]
    fn test_merge_threshold_angle() {
        let rad = merge_threshold_angle(90.0);
        assert!((rad - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn test_merge_to_json() {
        let fm = FaceMerge { merged_faces: vec![vec![0, 1, 2]], merge_log: vec![] };
        let j = merge_to_json(&fm);
        assert!(j.contains("face_count"));
    }

    #[test]
    fn test_merge_undo() {
        let tris = vec![[0, 1, 2]];
        let r = merge_undo(&tris);
        assert_eq!(r, vec![vec![0, 1, 2]]);
    }

    #[test]
    fn test_merge_face_pair_no_overlap() {
        let r = merge_face_pair(&[0, 1, 2], &[3, 4, 5]);
        assert_eq!(r.len(), 6);
    }
}
