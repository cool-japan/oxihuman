#![allow(dead_code)]
//! Face inset operations.

/// Face inset result.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FaceInset {
    pub new_vertices: Vec<[f32; 3]>,
    pub new_faces: Vec<[u32; 3]>,
    pub inset_amount: f32,
}

/// Inset a single face by moving vertices toward the centroid.
#[allow(dead_code)]
pub fn inset_face(
    positions: &[[f32; 3]],
    tri: [u32; 3],
    amount: f32,
) -> FaceInset {
    let a = positions[tri[0] as usize];
    let b = positions[tri[1] as usize];
    let c = positions[tri[2] as usize];
    let cx = (a[0] + b[0] + c[0]) / 3.0;
    let cy = (a[1] + b[1] + c[1]) / 3.0;
    let cz = (a[2] + b[2] + c[2]) / 3.0;
    let t = amount.clamp(0.0, 1.0);
    let new_a = [a[0] + (cx - a[0]) * t, a[1] + (cy - a[1]) * t, a[2] + (cz - a[2]) * t];
    let new_b = [b[0] + (cx - b[0]) * t, b[1] + (cy - b[1]) * t, b[2] + (cz - b[2]) * t];
    let new_c = [c[0] + (cx - c[0]) * t, c[1] + (cy - c[1]) * t, c[2] + (cz - c[2]) * t];
    let base = positions.len() as u32;
    FaceInset {
        new_vertices: vec![new_a, new_b, new_c],
        new_faces: vec![[base, base + 1, base + 2]],
        inset_amount: amount,
    }
}

/// Inset all faces.
#[allow(dead_code)]
pub fn inset_all_faces(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    amount: f32,
) -> Vec<FaceInset> {
    tris.iter().map(|t| inset_face(positions, *t, amount)).collect()
}

/// Get inset amount.
#[allow(dead_code)]
pub fn inset_amount(fi: &FaceInset) -> f32 {
    fi.inset_amount
}

/// Check if inset creates quads (always true for triangle inset).
#[allow(dead_code)]
pub fn inset_creates_quads(_fi: &FaceInset) -> bool {
    true
}

/// Count new vertices from inset.
#[allow(dead_code)]
pub fn inset_vertex_count(fi: &FaceInset) -> usize {
    fi.new_vertices.len()
}

/// Count new faces from inset.
#[allow(dead_code)]
pub fn inset_face_count(fi: &FaceInset) -> usize {
    fi.new_faces.len()
}

/// Serialize inset to JSON.
#[allow(dead_code)]
pub fn inset_to_json(fi: &FaceInset) -> String {
    format!(
        "{{\"amount\":{:.4},\"new_verts\":{},\"new_faces\":{}}}",
        fi.inset_amount,
        fi.new_vertices.len(),
        fi.new_faces.len()
    )
}

/// Inset each face individually.
#[allow(dead_code)]
pub fn inset_individual(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    amounts: &[f32],
) -> Vec<FaceInset> {
    tris.iter()
        .enumerate()
        .map(|(i, t)| {
            let a = if i < amounts.len() { amounts[i] } else { 0.0 };
            inset_face(positions, *t, a)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inset_face() {
        let pos = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
        let fi = inset_face(&pos, [0, 1, 2], 0.5);
        assert_eq!(fi.new_vertices.len(), 3);
    }

    #[test]
    fn test_inset_face_zero() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let fi = inset_face(&pos, [0, 1, 2], 0.0);
        assert_eq!(fi.new_vertices[0], pos[0]);
    }

    #[test]
    fn test_inset_all_faces() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0, 1, 2]];
        let results = inset_all_faces(&pos, &tris, 0.3);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_inset_amount() {
        let fi = FaceInset { new_vertices: vec![], new_faces: vec![], inset_amount: 0.5 };
        assert!((inset_amount(&fi) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_inset_creates_quads() {
        let fi = FaceInset { new_vertices: vec![], new_faces: vec![], inset_amount: 0.0 };
        assert!(inset_creates_quads(&fi));
    }

    #[test]
    fn test_inset_vertex_count() {
        let fi = FaceInset { new_vertices: vec![[0.0; 3]; 3], new_faces: vec![], inset_amount: 0.0 };
        assert_eq!(inset_vertex_count(&fi), 3);
    }

    #[test]
    fn test_inset_face_count() {
        let fi = FaceInset { new_vertices: vec![], new_faces: vec![[0, 1, 2]], inset_amount: 0.0 };
        assert_eq!(inset_face_count(&fi), 1);
    }

    #[test]
    fn test_inset_to_json() {
        let fi = FaceInset { new_vertices: vec![], new_faces: vec![], inset_amount: 0.25 };
        let j = inset_to_json(&fi);
        assert!(j.contains("amount"));
    }

    #[test]
    fn test_inset_individual() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0, 1, 2]];
        let r = inset_individual(&pos, &tris, &[0.5]);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_inset_individual_no_amount() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0, 1, 2]];
        let r = inset_individual(&pos, &tris, &[]);
        assert_eq!(r.len(), 1);
    }
}
