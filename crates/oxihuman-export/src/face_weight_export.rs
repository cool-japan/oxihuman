// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-face weight export.

/// Face weight export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceWeightExport {
    pub weights: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_face_weight_export(face_count: usize) -> FaceWeightExport {
    FaceWeightExport {
        weights: vec![1.0; face_count],
    }
}

#[allow(dead_code)]
pub fn fw_set(e: &mut FaceWeightExport, face: usize, w: f32) {
    if face < e.weights.len() {
        e.weights[face] = w;
    }
}

#[allow(dead_code)]
pub fn fw_get(e: &FaceWeightExport, face: usize) -> f32 {
    e.weights.get(face).copied().unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn fw_face_count(e: &FaceWeightExport) -> usize {
    e.weights.len()
}

#[allow(dead_code)]
pub fn fw_average(e: &FaceWeightExport) -> f32 {
    if e.weights.is_empty() {
        return 0.0;
    }
    e.weights.iter().sum::<f32>() / e.weights.len() as f32
}

#[allow(dead_code)]
pub fn fw_min(e: &FaceWeightExport) -> f32 {
    e.weights.iter().copied().fold(f32::MAX, f32::min)
}

#[allow(dead_code)]
pub fn fw_max(e: &FaceWeightExport) -> f32 {
    e.weights.iter().copied().fold(f32::MIN, f32::max)
}

#[allow(dead_code)]
pub fn fw_normalize(e: &mut FaceWeightExport) {
    let mx = fw_max(e);
    if mx > 1e-12 {
        for w in &mut e.weights {
            *w /= mx;
        }
    }
}

#[allow(dead_code)]
pub fn fw_to_json(e: &FaceWeightExport) -> String {
    format!(
        "{{\"face_count\":{},\"avg\":{:.6}}}",
        e.weights.len(),
        fw_average(e)
    )
}

#[allow(dead_code)]
pub fn fw_validate(e: &FaceWeightExport) -> bool {
    e.weights.iter().all(|w| (0.0..=1.0).contains(w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(fw_face_count(&new_face_weight_export(5)), 5);
    }

    #[test]
    fn test_default_weight() {
        assert!((fw_get(&new_face_weight_export(3), 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_get() {
        let mut e = new_face_weight_export(2);
        fw_set(&mut e, 0, 0.5);
        assert!((fw_get(&e, 0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_average() {
        let mut e = new_face_weight_export(2);
        fw_set(&mut e, 0, 0.0);
        assert!((fw_average(&e) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_min_max() {
        let mut e = new_face_weight_export(3);
        fw_set(&mut e, 0, 0.2);
        fw_set(&mut e, 2, 0.8);
        assert!((fw_min(&e) - 0.2).abs() < 1e-6);
        assert!((fw_max(&e) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize() {
        let mut e = FaceWeightExport {
            weights: vec![2.0, 4.0],
        };
        fw_normalize(&mut e);
        assert!((e.weights[1] - 1.0).abs() < 1e-6);
        assert!((e.weights[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        assert!(fw_to_json(&new_face_weight_export(1)).contains("\"face_count\":1"));
    }

    #[test]
    fn test_validate_ok() {
        assert!(fw_validate(&new_face_weight_export(3)));
    }

    #[test]
    fn test_validate_bad() {
        assert!(!fw_validate(&FaceWeightExport {
            weights: vec![-0.1]
        }));
    }

    #[test]
    fn test_empty_average() {
        assert!((fw_average(&new_face_weight_export(0))).abs() < 1e-6);
    }

    #[test]
    fn test_get_oob() {
        assert!((fw_get(&new_face_weight_export(0), 0)).abs() < 1e-6);
    }
}
