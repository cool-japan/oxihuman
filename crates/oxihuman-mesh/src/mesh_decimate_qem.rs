// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! QEM (Quadric Error Metric) decimation stub.

#[allow(dead_code)]
pub struct QEMDecimator {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
    pub target_faces: usize,
}

#[allow(dead_code)]
pub fn new_qem_decimator(
    positions: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
    target_faces: usize,
) -> QEMDecimator {
    QEMDecimator { positions, indices, target_faces }
}

#[allow(dead_code)]
pub fn qem_run(dec: &mut QEMDecimator) {
    if dec.indices.len() > dec.target_faces {
        dec.indices.truncate(dec.target_faces);
    }
}

#[allow(dead_code)]
pub fn qem_face_count(dec: &QEMDecimator) -> usize { dec.indices.len() }

#[allow(dead_code)]
pub fn qem_vertex_count(dec: &QEMDecimator) -> usize { dec.positions.len() }

#[allow(dead_code)]
pub fn qem_reduction_ratio(dec: &QEMDecimator, original_faces: usize) -> f32 {
    if original_faces == 0 { return 0.0; }
    dec.indices.len() as f32 / original_faces as f32
}

#[allow(dead_code)]
pub fn qem_is_complete(dec: &QEMDecimator) -> bool {
    dec.indices.len() <= dec.target_faces
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dec() -> QEMDecimator {
        new_qem_decimator(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.5, 0.5, 0.5]],
            vec![[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]],
            2,
        )
    }

    #[test]
    fn test_initial_face_count() { assert_eq!(qem_face_count(&make_dec()), 4); }

    #[test]
    fn test_initial_vertex_count() { assert_eq!(qem_vertex_count(&make_dec()), 4); }

    #[test]
    fn test_run_reduces() {
        let mut dec = make_dec();
        qem_run(&mut dec);
        assert_eq!(qem_face_count(&dec), 2);
    }

    #[test]
    fn test_reduction_ratio() {
        let mut dec = make_dec();
        qem_run(&mut dec);
        let ratio = qem_reduction_ratio(&dec, 4);
        assert!((ratio - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_is_complete_after_run() {
        let mut dec = make_dec();
        qem_run(&mut dec);
        assert!(qem_is_complete(&dec));
    }

    #[test]
    fn test_is_not_complete_before_run() {
        let dec = make_dec();
        assert!(!qem_is_complete(&dec));
    }

    #[test]
    fn test_zero_original_faces_ratio() {
        let dec = make_dec();
        assert_eq!(qem_reduction_ratio(&dec, 0), 0.0);
    }

    #[test]
    fn test_target_already_met() {
        let mut dec = new_qem_decimator(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
            5,
        );
        qem_run(&mut dec);
        assert_eq!(qem_face_count(&dec), 1);
        assert!(qem_is_complete(&dec));
    }
}
