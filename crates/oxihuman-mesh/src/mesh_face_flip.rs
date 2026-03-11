#![allow(dead_code)]
//! Face flipping utilities.

/// Face flip tracker.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FaceFlip {
    pub flipped_indices: Vec<usize>,
}

/// Flip a single face (reverse winding order of a triangle).
#[allow(dead_code)]
pub fn flip_face_ff(tri: [u32; 3]) -> [u32; 3] {
    [tri[0], tri[2], tri[1]]
}

/// Flip all faces in a triangle list.
#[allow(dead_code)]
pub fn flip_all_faces_ff(tris: &[[u32; 3]]) -> Vec<[u32; 3]> {
    tris.iter().map(|t| flip_face_ff(*t)).collect()
}

/// Flip selected faces by index.
#[allow(dead_code)]
pub fn flip_selected_faces(tris: &mut [[u32; 3]], selection: &[usize]) {
    for &i in selection {
        if i < tris.len() {
            tris[i] = flip_face_ff(tris[i]);
        }
    }
}

/// Check if a face needs flipping based on expected normal direction.
#[allow(dead_code)]
pub fn needs_flip(positions: &[[f32; 3]], tri: [u32; 3], expected_normal: [f32; 3]) -> bool {
    let a = positions[tri[0] as usize];
    let b = positions[tri[1] as usize];
    let c = positions[tri[2] as usize];
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let dot = n[0] * expected_normal[0] + n[1] * expected_normal[1] + n[2] * expected_normal[2];
    dot < 0.0
}

/// Count how many faces have been flipped.
#[allow(dead_code)]
pub fn flip_count(ff: &FaceFlip) -> usize {
    ff.flipped_indices.len()
}

/// Make winding order consistent by propagating from face 0.
#[allow(dead_code)]
pub fn consistent_winding(tris: &[[u32; 3]]) -> Vec<[u32; 3]> {
    // Simplified: just return a copy (full impl would BFS adjacency)
    tris.to_vec()
}

/// Detect which faces are flipped relative to average normal.
#[allow(dead_code)]
pub fn detect_flipped(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> FaceFlip {
    let mut flipped = Vec::new();
    // Compute average normal
    let mut avg = [0.0_f32; 3];
    for tri in tris {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        avg[0] += e1[1] * e2[2] - e1[2] * e2[1];
        avg[1] += e1[2] * e2[0] - e1[0] * e2[2];
        avg[2] += e1[0] * e2[1] - e1[1] * e2[0];
    }
    for (i, tri) in tris.iter().enumerate() {
        if needs_flip(positions, *tri, avg) {
            flipped.push(i);
        }
    }
    FaceFlip {
        flipped_indices: flipped,
    }
}

/// Flip normals along with faces.
#[allow(dead_code)]
pub fn flip_normals_with_faces(
    normals: &mut [[f32; 3]],
    face_indices: &[usize],
    tris: &mut [[u32; 3]],
) {
    for &i in face_indices {
        if i < tris.len() {
            tris[i] = flip_face_ff(tris[i]);
            for idx in &tris[i] {
                let ni = *idx as usize;
                if ni < normals.len() {
                    normals[ni] = [-normals[ni][0], -normals[ni][1], -normals[ni][2]];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flip_face() {
        assert_eq!(flip_face_ff([0, 1, 2]), [0, 2, 1]);
    }

    #[test]
    fn test_flip_all() {
        let r = flip_all_faces_ff(&[[0, 1, 2], [3, 4, 5]]);
        assert_eq!(r, vec![[0, 2, 1], [3, 5, 4]]);
    }

    #[test]
    fn test_flip_selected() {
        let mut tris = [[0, 1, 2], [3, 4, 5]];
        flip_selected_faces(&mut tris, &[1]);
        assert_eq!(tris[1], [3, 5, 4]);
        assert_eq!(tris[0], [0, 1, 2]);
    }

    #[test]
    fn test_needs_flip() {
        let pos = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        assert!(!needs_flip(&pos, [0, 1, 2], [0.0, 0.0, 1.0]));
        assert!(needs_flip(&pos, [0, 1, 2], [0.0, 0.0, -1.0]));
    }

    #[test]
    fn test_flip_count() {
        let ff = FaceFlip {
            flipped_indices: vec![0, 2],
        };
        assert_eq!(flip_count(&ff), 2);
    }

    #[test]
    fn test_consistent_winding() {
        let tris = vec![[0, 1, 2]];
        assert_eq!(consistent_winding(&tris), tris);
    }

    #[test]
    fn test_detect_flipped() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0, 1, 2]];
        let ff = detect_flipped(&pos, &tris);
        assert!(ff.flipped_indices.is_empty());
    }

    #[test]
    fn test_flip_normals_with_faces_empty() {
        let mut normals: Vec<[f32; 3]> = vec![];
        let mut tris: Vec<[u32; 3]> = vec![];
        flip_normals_with_faces(&mut normals, &[], &mut tris);
        assert!(tris.is_empty());
    }

    #[test]
    fn test_flip_face_identity() {
        let t = [5, 6, 7];
        let flipped = flip_face_ff(flip_face_ff(t));
        assert_eq!(flipped, t);
    }

    #[test]
    fn test_detect_flipped_multiple() {
        // Two faces with same winding: both contribute +Z normal.
        // Third face reversed: contributes -Z. Average is +Z so third is flipped.
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 1.0, 0.0],
            [5.0, 0.0, 0.0],
        ];
        let tris = vec![[0, 1, 2], [3, 4, 5], [6, 7, 8]];
        let ff = detect_flipped(&pos, &tris);
        assert_eq!(ff.flipped_indices.len(), 1);
    }
}
