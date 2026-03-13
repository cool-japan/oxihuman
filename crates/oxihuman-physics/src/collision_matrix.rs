// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Layer-based collision filtering matrix.
//!
//! Each layer is identified by a `usize` index in `[0, layer_count)`.  The
//! matrix is a symmetric bit-matrix that tracks which pairs of layers should
//! generate collision contacts.  Layer `i` always collides with itself unless
//! explicitly disabled.

#![allow(dead_code)]

/// Configuration for a collision matrix.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionMatrixConfig {
    /// Number of collision layers.
    pub layer_count: usize,
    /// If `true`, all pairs are enabled by default at construction time.
    pub default_enabled: bool,
}

/// Returns a sensible default [`CollisionMatrixConfig`].
#[allow(dead_code)]
pub fn default_collision_matrix_config() -> CollisionMatrixConfig {
    CollisionMatrixConfig {
        layer_count: 16,
        default_enabled: true,
    }
}

/// A symmetric collision filter matrix.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionMatrix {
    /// Flat upper-triangular (including diagonal) bit storage.
    /// Entry `(i, j)` with `i <= j` is at index `j*(j+1)/2 + i`.
    bits: Vec<bool>,
    /// Number of layers.
    pub layer_count: usize,
    /// Configuration.
    pub config: CollisionMatrixConfig,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn mat_idx(i: usize, j: usize) -> usize {
    let (lo, hi) = if i <= j { (i, j) } else { (j, i) };
    hi * (hi + 1) / 2 + lo
}

fn storage_size(n: usize) -> usize {
    n * (n + 1) / 2
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create a new collision matrix.
#[allow(dead_code)]
pub fn new_collision_matrix(config: CollisionMatrixConfig) -> CollisionMatrix {
    let n = config.layer_count;
    let bits = vec![config.default_enabled; storage_size(n)];
    CollisionMatrix { bits, layer_count: n, config }
}

/// Set whether layers `i` and `j` collide with each other.
#[allow(dead_code)]
pub fn matrix_set_collides(mat: &mut CollisionMatrix, i: usize, j: usize, collides: bool) {
    if i >= mat.layer_count || j >= mat.layer_count { return; }
    let idx = mat_idx(i, j);
    mat.bits[idx] = collides;
}

/// Returns `true` if layers `i` and `j` collide.
#[allow(dead_code)]
pub fn matrix_collides(mat: &CollisionMatrix, i: usize, j: usize) -> bool {
    if i >= mat.layer_count || j >= mat.layer_count { return false; }
    mat.bits[mat_idx(i, j)]
}

/// Number of layers in the matrix.
#[allow(dead_code)]
pub fn matrix_layer_count(mat: &CollisionMatrix) -> usize {
    mat.layer_count
}

/// Returns all enabled collision pairs `(i, j)` with `i <= j`.
#[allow(dead_code)]
pub fn matrix_enabled_pairs(mat: &CollisionMatrix) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    let n = mat.layer_count;
    for j in 0..n {
        for i in 0..=j {
            if mat.bits[mat_idx(i, j)] {
                pairs.push((i, j));
            }
        }
    }
    pairs
}

/// Serialize the matrix to a simple JSON string.
#[allow(dead_code)]
pub fn matrix_to_json(mat: &CollisionMatrix) -> String {
    let pairs = matrix_enabled_pairs(mat);
    let pair_strs: Vec<String> = pairs.iter()
        .map(|(a, b)| format!("[{a},{b}]"))
        .collect();
    format!(
        "{{\"layer_count\":{},\"enabled_pairs\":[{}]}}",
        mat.layer_count,
        pair_strs.join(",")
    )
}

/// Disable all collision pairs (including self-collisions).
#[allow(dead_code)]
pub fn matrix_disable_all(mat: &mut CollisionMatrix) {
    for b in &mut mat.bits { *b = false; }
}

/// Enable all collision pairs (including self-collisions).
#[allow(dead_code)]
pub fn matrix_enable_all(mat: &mut CollisionMatrix) {
    for b in &mut mat.bits { *b = true; }
}

/// Toggle the collision flag for layers `i` and `j`.
#[allow(dead_code)]
pub fn matrix_toggle(mat: &mut CollisionMatrix, i: usize, j: usize) {
    if i >= mat.layer_count || j >= mat.layer_count { return; }
    let idx = mat_idx(i, j);
    mat.bits[idx] = !mat.bits[idx];
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mat(n: usize, default_on: bool) -> CollisionMatrix {
        new_collision_matrix(CollisionMatrixConfig {
            layer_count: n,
            default_enabled: default_on,
        })
    }

    #[test]
    fn test_default_all_enabled() {
        let mat = make_mat(4, true);
        for i in 0..4 {
            for j in 0..4 {
                assert!(matrix_collides(&mat, i, j));
            }
        }
    }

    #[test]
    fn test_default_all_disabled() {
        let mat = make_mat(4, false);
        for i in 0..4 {
            for j in 0..4 {
                assert!(!matrix_collides(&mat, i, j));
            }
        }
    }

    #[test]
    fn test_set_and_get() {
        let mut mat = make_mat(4, true);
        matrix_set_collides(&mut mat, 1, 2, false);
        assert!(!matrix_collides(&mat, 1, 2));
        assert!(!matrix_collides(&mat, 2, 1), "matrix should be symmetric");
    }

    #[test]
    fn test_symmetry() {
        let mut mat = make_mat(8, false);
        matrix_set_collides(&mut mat, 3, 7, true);
        assert_eq!(matrix_collides(&mat, 3, 7), matrix_collides(&mat, 7, 3));
    }

    #[test]
    fn test_out_of_bounds_safe() {
        let mat = make_mat(4, true);
        assert!(!matrix_collides(&mat, 99, 0));
        let mut mat2 = make_mat(4, true);
        matrix_set_collides(&mut mat2, 99, 0, false); // should not panic
    }

    #[test]
    fn test_layer_count() {
        let mat = make_mat(10, true);
        assert_eq!(matrix_layer_count(&mat), 10);
    }

    #[test]
    fn test_enabled_pairs_count() {
        let mat = make_mat(3, true);
        let pairs = matrix_enabled_pairs(&mat);
        // 3 layers: pairs are (0,0),(0,1),(0,2),(1,1),(1,2),(2,2) = 6
        assert_eq!(pairs.len(), 6);
    }

    #[test]
    fn test_disable_all() {
        let mut mat = make_mat(4, true);
        matrix_disable_all(&mut mat);
        assert_eq!(matrix_enabled_pairs(&mat).len(), 0);
    }

    #[test]
    fn test_enable_all() {
        let mut mat = make_mat(4, false);
        matrix_enable_all(&mut mat);
        let n = 4;
        let expected = n * (n + 1) / 2;
        assert_eq!(matrix_enabled_pairs(&mat).len(), expected);
    }

    #[test]
    fn test_toggle() {
        let mut mat = make_mat(4, true);
        assert!(matrix_collides(&mat, 0, 1));
        matrix_toggle(&mut mat, 0, 1);
        assert!(!matrix_collides(&mat, 0, 1));
        matrix_toggle(&mut mat, 0, 1);
        assert!(matrix_collides(&mat, 0, 1));
    }

    #[test]
    fn test_json_output() {
        let mat = make_mat(2, true);
        let json = matrix_to_json(&mat);
        assert!(json.contains("layer_count"));
        assert!(json.contains("enabled_pairs"));
    }

    #[test]
    fn test_zero_layers() {
        let mat = make_mat(0, true);
        assert_eq!(matrix_layer_count(&mat), 0);
        assert_eq!(matrix_enabled_pairs(&mat).len(), 0);
    }
}
