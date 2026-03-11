// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Duplicate face detection and removal.
//! Two triangles are considered duplicates if they share the same three vertex
//! indices (in any winding order).

use std::collections::HashMap;

/// Statistics from the deduplication pass.
#[derive(Debug, Clone, Default)]
pub struct DupeFaceResult {
    pub total_input: usize,
    pub duplicates_removed: usize,
    pub total_output: usize,
}

/// Returns a canonical key for a triangle (sorted indices).
pub fn face_canonical_key(a: u32, b: u32, c: u32) -> (u32, u32, u32) {
    let mut v = [a, b, c];
    v.sort_unstable();
    (v[0], v[1], v[2])
}

/// Detects duplicate face indices.
/// Returns indices (into the face list) of all duplicate occurrences.
pub fn find_duplicate_faces(indices: &[u32]) -> Vec<usize> {
    let n = indices.len() / 3;
    let mut seen: HashMap<(u32, u32, u32), usize> = HashMap::new();
    let mut dupes = Vec::new();
    for i in 0..n {
        let key = face_canonical_key(indices[i * 3], indices[i * 3 + 1], indices[i * 3 + 2]);
        if let Some(_prev) = seen.get(&key) {
            dupes.push(i);
        } else {
            seen.insert(key, i);
        }
    }
    dupes
}

/// Removes duplicate faces from an index buffer.
pub fn remove_duplicate_faces_ex(indices: &[u32]) -> (Vec<u32>, DupeFaceResult) {
    let n = indices.len() / 3;
    let mut seen: HashMap<(u32, u32, u32), usize> = HashMap::new();
    let mut out = Vec::with_capacity(indices.len());
    let mut removed = 0usize;
    for i in 0..n {
        let key = face_canonical_key(indices[i * 3], indices[i * 3 + 1], indices[i * 3 + 2]);
        use std::collections::hash_map::Entry;
        match seen.entry(key) {
            Entry::Occupied(_) => {
                removed += 1;
            }
            Entry::Vacant(e) => {
                e.insert(i);
                out.push(indices[i * 3]);
                out.push(indices[i * 3 + 1]);
                out.push(indices[i * 3 + 2]);
            }
        }
    }
    let result = DupeFaceResult {
        total_input: n,
        duplicates_removed: removed,
        total_output: out.len() / 3,
    };
    (out, result)
}

/// Counts how many unique faces an index buffer has.
pub fn unique_face_count(indices: &[u32]) -> usize {
    let n = indices.len() / 3;
    let mut seen: std::collections::HashSet<(u32, u32, u32)> = std::collections::HashSet::new();
    for i in 0..n {
        let key = face_canonical_key(indices[i * 3], indices[i * 3 + 1], indices[i * 3 + 2]);
        seen.insert(key);
    }
    seen.len()
}

/// Returns `true` if the index buffer contains any duplicate faces.
pub fn has_duplicate_faces(indices: &[u32]) -> bool {
    let n = indices.len() / 3;
    let mut seen: std::collections::HashSet<(u32, u32, u32)> = std::collections::HashSet::new();
    for i in 0..n {
        let key = face_canonical_key(indices[i * 3], indices[i * 3 + 1], indices[i * 3 + 2]);
        if !seen.insert(key) {
            return true;
        }
    }
    false
}

/// Counts duplicate occurrences for each canonical face key.
pub fn duplicate_face_histogram(indices: &[u32]) -> HashMap<(u32, u32, u32), usize> {
    let n = indices.len() / 3;
    let mut counts: HashMap<(u32, u32, u32), usize> = HashMap::new();
    for i in 0..n {
        let key = face_canonical_key(indices[i * 3], indices[i * 3 + 1], indices[i * 3 + 2]);
        *counts.entry(key).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tris_one_dupe() -> Vec<u32> {
        /* triangle (0,1,2) appears twice */
        vec![0, 1, 2, 0, 2, 1, 3, 4, 5]
    }

    #[test]
    fn canonical_key_sorted() {
        /* any permutation should give the same key */
        let k1 = face_canonical_key(3, 1, 2);
        let k2 = face_canonical_key(2, 3, 1);
        assert_eq!(k1, k2);
    }

    #[test]
    fn find_duplicates_detects_one() {
        let idx = two_tris_one_dupe();
        let dupes = find_duplicate_faces(&idx);
        assert_eq!(dupes.len(), 1);
    }

    #[test]
    fn remove_duplicates_correct_count() {
        let idx = two_tris_one_dupe();
        let (out, stats) = remove_duplicate_faces_ex(&idx);
        assert_eq!(stats.duplicates_removed, 1);
        assert_eq!(out.len(), 6); /* 2 unique triangles × 3 */
    }

    #[test]
    fn unique_face_count_no_dupes() {
        let idx = vec![0u32, 1, 2, 3, 4, 5];
        assert_eq!(unique_face_count(&idx), 2);
    }

    #[test]
    fn has_duplicate_faces_true() {
        let idx = two_tris_one_dupe();
        assert!(has_duplicate_faces(&idx));
    }

    #[test]
    fn has_duplicate_faces_false() {
        let idx = vec![0u32, 1, 2, 3, 4, 5];
        assert!(!has_duplicate_faces(&idx));
    }

    #[test]
    fn histogram_counts_correct() {
        let idx = two_tris_one_dupe();
        let hist = duplicate_face_histogram(&idx);
        /* (0,1,2) should have count 2 */
        let k = face_canonical_key(0, 1, 2);
        assert_eq!(hist[&k], 2);
    }

    #[test]
    fn empty_indices_no_dupes() {
        let idx: Vec<u32> = vec![];
        assert!(!has_duplicate_faces(&idx));
        assert_eq!(unique_face_count(&idx), 0);
    }

    #[test]
    fn result_stats_sum() {
        let idx = two_tris_one_dupe();
        let (_, stats) = remove_duplicate_faces_ex(&idx);
        assert_eq!(
            stats.total_input,
            stats.total_output + stats.duplicates_removed
        );
    }
}
