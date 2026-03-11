// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Checker deselect: alternates the selection of faces/vertices in a checkerboard pattern.

/// Result of a checker-deselect operation.
#[derive(Debug, Clone, Default)]
pub struct CheckerDeselResult {
    pub selected_before: usize,
    pub selected_after: usize,
    pub deselected_count: usize,
}

/// Applies checker deselect to a face selection.
/// `selected` is a vec of booleans, one per face.
/// Every other selected face is deselected.
pub fn checker_deselect_faces(selected: &mut [bool]) -> CheckerDeselResult {
    let before = selected.iter().filter(|&&s| s).count();
    let mut toggle = false;
    for s in selected.iter_mut() {
        if *s {
            if toggle {
                *s = false;
            }
            toggle = !toggle;
        }
    }
    let after = selected.iter().filter(|&&s| s).count();
    CheckerDeselResult {
        selected_before: before,
        selected_after: after,
        deselected_count: before.saturating_sub(after),
    }
}

/// Applies checker deselect to a list of selected indices.
/// Returns two sets: the kept indices and the deselected indices.
pub fn checker_deselect_indices(selected: &[usize]) -> (Vec<usize>, Vec<usize>) {
    let mut keep = Vec::new();
    let mut remove = Vec::new();
    for (i, &idx) in selected.iter().enumerate() {
        if i.is_multiple_of(2) {
            keep.push(idx);
        } else {
            remove.push(idx);
        }
    }
    (keep, remove)
}

/// Checker deselect on vertex selection flags.
pub fn checker_deselect_vertices(selected: &mut [bool]) -> usize {
    let mut toggle = false;
    let mut count = 0usize;
    for s in selected.iter_mut() {
        if *s {
            if toggle {
                *s = false;
                count += 1;
            }
            toggle = !toggle;
        }
    }
    count
}

/// Checker deselect with an offset: start deselecting from the (offset mod 2) selected item.
pub fn checker_deselect_offset(selected: &[usize], offset: usize) -> (Vec<usize>, Vec<usize>) {
    let mut keep = Vec::new();
    let mut remove = Vec::new();
    for (i, &idx) in selected.iter().enumerate() {
        if (i + offset).is_multiple_of(2) {
            keep.push(idx);
        } else {
            remove.push(idx);
        }
    }
    (keep, remove)
}

/// Returns a bitmask where bit `i` is set if face `i` should remain selected
/// after checker deselect.
pub fn checker_deselect_mask(face_count: usize) -> Vec<bool> {
    (0..face_count).map(|i| i.is_multiple_of(2)).collect()
}

/// Applies checker deselect and returns the count of alternating face selections.
pub fn checker_select_count(face_count: usize) -> usize {
    face_count.div_ceil(2)
}

/// Inverts a boolean selection mask.
pub fn invert_selection(selected: &[bool]) -> Vec<bool> {
    selected.iter().map(|&s| !s).collect()
}

/// Returns the selected indices from a bool mask.
pub fn selected_from_mask(mask: &[bool]) -> Vec<usize> {
    mask.iter()
        .enumerate()
        .filter(|(_, &s)| s)
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checker_deselect_faces_alternates() {
        let mut sel = vec![true, true, true, true];
        let res = checker_deselect_faces(&mut sel);
        /* every other one deselected */
        assert_eq!(res.selected_after, 2);
        assert_eq!(res.deselected_count, 2);
    }

    #[test]
    fn checker_deselect_indices_split() {
        let sel = vec![0usize, 1, 2, 3, 4];
        let (keep, remove) = checker_deselect_indices(&sel);
        assert_eq!(keep.len(), 3);
        assert_eq!(remove.len(), 2);
    }

    #[test]
    fn checker_deselect_vertices_count() {
        let mut sel = vec![true; 6];
        let deselected = checker_deselect_vertices(&mut sel);
        assert_eq!(deselected, 3);
    }

    #[test]
    fn checker_deselect_offset_inverts() {
        let sel = vec![0usize, 1, 2, 3];
        let (keep0, _) = checker_deselect_offset(&sel, 0);
        let (keep1, _) = checker_deselect_offset(&sel, 1);
        /* opposite selections */
        assert_ne!(keep0, keep1);
    }

    #[test]
    fn checker_mask_correct_length() {
        let mask = checker_deselect_mask(6);
        assert_eq!(mask.len(), 6);
    }

    #[test]
    fn checker_mask_alternating() {
        let mask = checker_deselect_mask(4);
        assert_eq!(mask, vec![true, false, true, false]);
    }

    #[test]
    fn checker_select_count_even() {
        assert_eq!(checker_select_count(6), 3);
    }

    #[test]
    fn checker_select_count_odd() {
        assert_eq!(checker_select_count(5), 3);
    }

    #[test]
    fn invert_selection_flips() {
        let sel = vec![true, false, true];
        let inv = invert_selection(&sel);
        assert_eq!(inv, vec![false, true, false]);
    }

    #[test]
    fn selected_from_mask_correct() {
        let mask = vec![true, false, true, false];
        let idx = selected_from_mask(&mask);
        assert_eq!(idx, vec![0, 2]);
    }
}
