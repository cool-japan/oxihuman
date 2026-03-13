// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mesh quality metrics (scaled Jacobian, condition number, etc.)

#[allow(dead_code)]
pub struct QualityMetric {
    pub min_quality: f32,
    pub max_quality: f32,
    pub avg_quality: f32,
    pub face_count: usize,
}

fn vec_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn cross_len(ab: [f32; 3], ac: [f32; 3]) -> f32 {
    let cx = ab[1] * ac[2] - ab[2] * ac[1];
    let cy = ab[2] * ac[0] - ab[0] * ac[2];
    let cz = ab[0] * ac[1] - ab[1] * ac[0];
    (cx * cx + cy * cy + cz * cz).sqrt()
}

#[allow(dead_code)]
pub fn qm_scaled_jacobian(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let area2 = cross_len(ab, ac);
    let lab = vec_len(ab);
    let lac = vec_len(ac);
    if lab < 1e-10 || lac < 1e-10 { return 0.0; }
    area2 / (lab * lac)
}

#[allow(dead_code)]
pub fn qm_compute_mesh(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> QualityMetric {
    if indices.is_empty() {
        return QualityMetric { min_quality: 0.0, max_quality: 0.0, avg_quality: 0.0, face_count: 0 };
    }
    let mut min_q = f32::MAX;
    let mut max_q = 0.0f32;
    let mut sum = 0.0f32;
    let mut count = 0usize;
    for tri in indices {
        let a = tri[0] as usize;
        let b = tri[1] as usize;
        let c = tri[2] as usize;
        if a < positions.len() && b < positions.len() && c < positions.len() {
            let q = qm_scaled_jacobian(positions[a], positions[b], positions[c]);
            if q < min_q { min_q = q; }
            if q > max_q { max_q = q; }
            sum += q;
            count += 1;
        }
    }
    QualityMetric {
        min_quality: if count > 0 { min_q } else { 0.0 },
        max_quality: max_q,
        avg_quality: if count > 0 { sum / count as f32 } else { 0.0 },
        face_count: count,
    }
}

#[allow(dead_code)]
pub fn qm_bad_element_count(metrics: &[f32], threshold: f32) -> usize {
    metrics.iter().filter(|&&q| q < threshold).count()
}

#[allow(dead_code)]
pub fn qm_histogram(metrics: &[f32], bins: usize) -> Vec<usize> {
    if bins == 0 { return vec![]; }
    let mut hist = vec![0usize; bins];
    for &q in metrics {
        let q_clamped = q.clamp(0.0, 1.0);
        let bin = ((q_clamped * bins as f32) as usize).min(bins - 1);
        hist[bin] += 1;
    }
    hist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaled_jacobian_nonneg() {
        let q = qm_scaled_jacobian([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(q >= 0.0);
    }

    #[test]
    fn test_scaled_jacobian_equilateral_close_to_one() {
        let h = (3.0f32).sqrt() / 2.0;
        let q = qm_scaled_jacobian([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, h, 0.0]);
        assert!(q > 0.8, "Expected near 1, got {q}");
    }

    #[test]
    fn test_scaled_jacobian_degenerate_zero() {
        let q = qm_scaled_jacobian([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert!(q < 1e-5);
    }

    #[test]
    fn test_compute_mesh_empty() {
        let m = qm_compute_mesh(&[], &[]);
        assert_eq!(m.face_count, 0);
    }

    #[test]
    fn test_compute_mesh_single_face() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![[0u32, 1, 2]];
        let m = qm_compute_mesh(&positions, &indices);
        assert_eq!(m.face_count, 1);
        assert!(m.avg_quality >= 0.0);
    }

    #[test]
    fn test_bad_element_count() {
        let metrics = vec![0.9, 0.1, 0.5, 0.05, 0.8];
        assert_eq!(qm_bad_element_count(&metrics, 0.2), 2);
    }

    #[test]
    fn test_histogram_bins() {
        let metrics = vec![0.0, 0.5, 1.0];
        let hist = qm_histogram(&metrics, 4);
        assert_eq!(hist.len(), 4);
        assert_eq!(hist.iter().sum::<usize>(), 3);
    }

    #[test]
    fn test_histogram_empty_bins() {
        let hist = qm_histogram(&[0.5], 0);
        assert!(hist.is_empty());
    }
}
