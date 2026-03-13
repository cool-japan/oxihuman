// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Face pair detection: adjacent triangles sharing an edge.

/// A pair of adjacent faces sharing an edge.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FacePair {
    pub face_a: usize,
    pub face_b: usize,
    pub shared_v0: u32,
    pub shared_v1: u32,
}

/// Result of face pair analysis.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FacePairResult {
    pub pairs: Vec<FacePair>,
    pub unpaired_faces: usize,
}

/// Build a canonical edge key (smaller index first).
#[allow(dead_code)]
pub fn edge_key(a: u32, b: u32) -> (u32, u32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Detect all face pairs sharing exactly one edge.
#[allow(dead_code)]
pub fn detect_face_pairs(indices: &[u32]) -> FacePairResult {
    use std::collections::HashMap;
    let face_count = indices.len() / 3;
    type FaceEdgeVec = Vec<(usize, u32, u32)>;
    let mut edge_map: HashMap<(u32, u32), FaceEdgeVec> = HashMap::new();
    for f in 0..face_count {
        let i0 = indices[f * 3];
        let i1 = indices[f * 3 + 1];
        let i2 = indices[f * 3 + 2];
        for (a, b) in [(i0, i1), (i1, i2), (i2, i0)] {
            edge_map.entry(edge_key(a, b)).or_default().push((f, a, b));
        }
    }
    let mut pairs = Vec::new();
    let mut paired = vec![false; face_count];
    for faces in edge_map.values() {
        if faces.len() == 2 {
            let (fa, va0, va1) = faces[0];
            let (fb, _vb0, _vb1) = faces[1];
            pairs.push(FacePair {
                face_a: fa,
                face_b: fb,
                shared_v0: va0,
                shared_v1: va1,
            });
            paired[fa] = true;
            paired[fb] = true;
        }
    }
    let unpaired_faces = paired.iter().filter(|&&p| !p).count();
    FacePairResult {
        pairs,
        unpaired_faces,
    }
}

/// Count of face pairs.
#[allow(dead_code)]
pub fn face_pair_count(r: &FacePairResult) -> usize {
    r.pairs.len()
}

/// Check if two faces are paired.
#[allow(dead_code)]
pub fn faces_are_paired(r: &FacePairResult, fa: usize, fb: usize) -> bool {
    r.pairs
        .iter()
        .any(|p| (p.face_a == fa && p.face_b == fb) || (p.face_a == fb && p.face_b == fa))
}

/// Maximum face index in any pair.
#[allow(dead_code)]
pub fn max_paired_face(r: &FacePairResult) -> Option<usize> {
    r.pairs.iter().flat_map(|p| [p.face_a, p.face_b]).max()
}

/// Pairing ratio: paired faces / total faces.
#[allow(dead_code)]
pub fn pairing_ratio(r: &FacePairResult, face_count: usize) -> f32 {
    if face_count == 0 {
        return 0.0;
    }
    let paired = face_count.saturating_sub(r.unpaired_faces);
    paired as f32 / face_count as f32
}

/// Export to JSON.
#[allow(dead_code)]
pub fn face_pair_to_json(r: &FacePairResult) -> String {
    format!(
        "{{\"pair_count\":{},\"unpaired_faces\":{}}}",
        r.pairs.len(),
        r.unpaired_faces
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_sharing_tris() -> Vec<u32> {
        vec![0, 1, 2, 1, 3, 2]
    }

    #[test]
    fn test_edge_key_canonical() {
        assert_eq!(edge_key(3, 1), (1, 3));
        assert_eq!(edge_key(1, 3), (1, 3));
    }

    #[test]
    fn test_detect_face_pairs_two_tris() {
        let idx = two_sharing_tris();
        let r = detect_face_pairs(&idx);
        assert_eq!(face_pair_count(&r), 1);
    }

    #[test]
    fn test_detect_face_pairs_empty() {
        let r = detect_face_pairs(&[]);
        assert_eq!(face_pair_count(&r), 0);
        assert_eq!(r.unpaired_faces, 0);
    }

    #[test]
    fn test_faces_are_paired() {
        let idx = two_sharing_tris();
        let r = detect_face_pairs(&idx);
        assert!(faces_are_paired(&r, 0, 1));
        assert!(!faces_are_paired(&r, 0, 2));
    }

    #[test]
    fn test_max_paired_face() {
        let idx = two_sharing_tris();
        let r = detect_face_pairs(&idx);
        assert!(max_paired_face(&r).is_some_and(|m| m == 1));
    }

    #[test]
    fn test_pairing_ratio_full() {
        let idx = two_sharing_tris();
        let r = detect_face_pairs(&idx);
        let ratio = pairing_ratio(&r, 2);
        assert!((0.0..=1.0).contains(&ratio));
    }

    #[test]
    fn test_pairing_ratio_zero_faces() {
        let r = detect_face_pairs(&[]);
        assert!((pairing_ratio(&r, 0)).abs() < 1e-9);
    }

    #[test]
    fn test_face_pair_to_json() {
        let idx = two_sharing_tris();
        let r = detect_face_pairs(&idx);
        let j = face_pair_to_json(&r);
        assert!(j.contains("\"pair_count\":1"));
    }

    #[test]
    fn test_isolated_triangle_unpaired() {
        let idx = vec![0u32, 1, 2];
        let r = detect_face_pairs(&idx);
        assert_eq!(r.unpaired_faces, 1);
    }

    #[test]
    fn test_four_tris_two_pairs() {
        // Two disconnected pairs
        let idx = vec![0u32, 1, 2, 1, 3, 2, 4, 5, 6, 5, 7, 6];
        let r = detect_face_pairs(&idx);
        assert!(face_pair_count(&r) >= 2);
    }
}
