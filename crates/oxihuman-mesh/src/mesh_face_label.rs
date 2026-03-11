// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Per-face label assignment and querying utilities.

/// Per-face labels (arbitrary u32 tags).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceLabelSet {
    pub labels: Vec<u32>,
    pub face_count: usize,
}

/// Create a label set with all faces initialised to `default_label`.
#[allow(dead_code)]
pub fn new_face_labels(face_count: usize, default_label: u32) -> FaceLabelSet {
    FaceLabelSet {
        labels: vec![default_label; face_count],
        face_count,
    }
}

/// Set a label on a specific face.
#[allow(dead_code)]
pub fn set_face_label(set: &mut FaceLabelSet, face: usize, label: u32) {
    if face < set.labels.len() {
        set.labels[face] = label;
    }
}

/// Get the label of a face.
#[allow(dead_code)]
pub fn get_face_label(set: &FaceLabelSet, face: usize) -> Option<u32> {
    set.labels.get(face).copied()
}

/// Return all face indices with a given label.
#[allow(dead_code)]
pub fn faces_with_label(set: &FaceLabelSet, label: u32) -> Vec<usize> {
    set.labels
        .iter()
        .enumerate()
        .filter(|(_, &l)| l == label)
        .map(|(i, _)| i)
        .collect()
}

/// Number of distinct labels.
#[allow(dead_code)]
pub fn distinct_label_count(set: &FaceLabelSet) -> usize {
    let mut seen: Vec<u32> = set.labels.clone();
    seen.sort_unstable();
    seen.dedup();
    seen.len()
}

/// Flood-fill label assignment: spread label from seed faces using adjacency.
#[allow(dead_code)]
pub fn flood_fill_face_label(
    set: &mut FaceLabelSet,
    adjacency: &[Vec<usize>],
    seed_faces: &[usize],
    label: u32,
) {
    if adjacency.len() != set.face_count {
        return;
    }
    let mut queue: std::collections::VecDeque<usize> = seed_faces.iter().copied().collect();
    for &f in seed_faces {
        if f < set.labels.len() {
            set.labels[f] = label;
        }
    }
    while let Some(f) = queue.pop_front() {
        if f >= adjacency.len() {
            continue;
        }
        for &nb in &adjacency[f] {
            if nb < set.labels.len() && set.labels[nb] != label {
                set.labels[nb] = label;
                queue.push_back(nb);
            }
        }
    }
}

/// Largest label group face count.
#[allow(dead_code)]
pub fn largest_label_group(set: &FaceLabelSet) -> usize {
    let mut counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &l in &set.labels {
        *counts.entry(l).or_insert(0) += 1;
    }
    counts.values().copied().max().unwrap_or(0)
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn face_labels_to_json(set: &FaceLabelSet) -> String {
    format!(
        "{{\"face_count\":{},\"distinct_labels\":{}}}",
        set.face_count,
        distinct_label_count(set)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_label_all_same() {
        let set = new_face_labels(5, 99);
        assert!(set.labels.iter().all(|&l| l == 99));
    }

    #[test]
    fn set_and_get() {
        let mut set = new_face_labels(3, 0);
        set_face_label(&mut set, 1, 42);
        assert_eq!(get_face_label(&set, 1), Some(42));
    }

    #[test]
    fn get_out_of_range_none() {
        let set = new_face_labels(3, 0);
        assert!(get_face_label(&set, 100).is_none());
    }

    #[test]
    fn faces_with_label_query() {
        let mut set = new_face_labels(4, 0);
        set_face_label(&mut set, 1, 7);
        set_face_label(&mut set, 3, 7);
        let v = faces_with_label(&set, 7);
        assert_eq!(v, vec![1, 3]);
    }

    #[test]
    fn distinct_label_count_initial() {
        let set = new_face_labels(5, 0);
        assert_eq!(distinct_label_count(&set), 1);
    }

    #[test]
    fn distinct_label_count_after_set() {
        let mut set = new_face_labels(4, 0);
        set_face_label(&mut set, 2, 1);
        assert_eq!(distinct_label_count(&set), 2);
    }

    #[test]
    fn flood_fill_spreads_label() {
        let mut set = new_face_labels(4, 0);
        let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
        flood_fill_face_label(&mut set, &adj, &[0], 5);
        assert!(set.labels.iter().all(|&l| l == 5));
    }

    #[test]
    fn largest_group_all_same() {
        let set = new_face_labels(6, 3);
        assert_eq!(largest_label_group(&set), 6);
    }

    #[test]
    fn json_contains_face_count() {
        let set = new_face_labels(8, 0);
        let j = face_labels_to_json(&set);
        assert!(j.contains("face_count"));
    }

    #[test]
    fn contains_range() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
