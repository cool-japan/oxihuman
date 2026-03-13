// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fast LBS (linear blend skinning) approximation stub.

/// A compact skinning record for one vertex: up to 4 bone weights.
#[derive(Debug, Clone, Copy)]
pub struct FastLbsRecord {
    pub bones: [u8; 4],
    pub weights: [f32; 4],
}

impl Default for FastLbsRecord {
    fn default() -> Self {
        FastLbsRecord {
            bones: [0; 4],
            weights: [0.0; 4],
        }
    }
}

/// Fast LBS mesh.
#[derive(Debug, Clone)]
pub struct FastLbs {
    pub records: Vec<FastLbsRecord>,
    pub bone_count: usize,
}

impl FastLbs {
    pub fn new(vertex_count: usize, bone_count: usize) -> Self {
        FastLbs {
            records: vec![FastLbsRecord::default(); vertex_count],
            bone_count,
        }
    }
}

/// Create a new fast LBS mesh.
pub fn new_fast_lbs(vertex_count: usize, bone_count: usize) -> FastLbs {
    FastLbs::new(vertex_count, bone_count)
}

/// Set the skinning record for a vertex.
pub fn fast_lbs_set(lbs: &mut FastLbs, vertex: usize, record: FastLbsRecord) {
    if vertex < lbs.records.len() {
        lbs.records[vertex] = record;
    }
}

/// Normalize weights for a vertex record so they sum to 1.0.
pub fn fast_lbs_normalize(record: &mut FastLbsRecord) {
    let sum: f32 = record.weights.iter().sum();
    if sum > 1e-9 {
        for w in &mut record.weights {
            *w /= sum;
        }
    }
}

/// Compute the blended position for a vertex (stub: just returns source position).
pub fn fast_lbs_transform(
    lbs: &FastLbs,
    vertex: usize,
    source: [f32; 3],
    _bone_matrices: &[[[f32; 4]; 4]],
) -> [f32; 3] {
    /* Stub: returns source position unchanged for now */
    if vertex < lbs.records.len() {
        source
    } else {
        [0.0; 3]
    }
}

/// Return vertex count.
pub fn fast_lbs_vertex_count(lbs: &FastLbs) -> usize {
    lbs.records.len()
}

/// Return a JSON-like string.
pub fn fast_lbs_to_json(lbs: &FastLbs) -> String {
    format!(
        r#"{{"vertices":{},"bones":{}}}"#,
        lbs.records.len(),
        lbs.bone_count
    )
}

/// Check all records have weights that sum to ~1 (ignoring zero-weight vertices).
pub fn fast_lbs_is_valid(lbs: &FastLbs) -> bool {
    lbs.records.iter().all(|r| {
        let s: f32 = r.weights.iter().sum();
        s < 1e-9 || (s - 1.0).abs() < 1e-4
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fast_lbs_vertex_count() {
        let lbs = new_fast_lbs(16, 4);
        assert_eq!(
            fast_lbs_vertex_count(&lbs),
            16, /* vertex count must match */
        );
    }

    #[test]
    fn test_initial_records_zero_weights() {
        let lbs = new_fast_lbs(3, 2);
        for r in &lbs.records {
            let s: f32 = r.weights.iter().sum();
            assert!((s).abs() < 1e-6 /* initial weights should be zero */,);
        }
    }

    #[test]
    fn test_initial_valid() {
        let lbs = new_fast_lbs(4, 2);
        assert!(fast_lbs_is_valid(&lbs), /* zero-weight records are trivially valid */);
    }

    #[test]
    fn test_set_record_updates() {
        let mut lbs = new_fast_lbs(4, 2);
        let r = FastLbsRecord {
            bones: [1, 2, 0, 0],
            weights: [0.5, 0.5, 0.0, 0.0],
        };
        fast_lbs_set(&mut lbs, 0, r);
        assert!((lbs.records[0].weights[0] - 0.5).abs() < 1e-5, /* weight must match */);
    }

    #[test]
    fn test_set_out_of_bounds_ignored() {
        let mut lbs = new_fast_lbs(2, 2);
        fast_lbs_set(&mut lbs, 99, FastLbsRecord::default());
        assert_eq!(
            fast_lbs_vertex_count(&lbs),
            2, /* vertex count unchanged */
        );
    }

    #[test]
    fn test_normalize_record() {
        let mut r = FastLbsRecord {
            bones: [0; 4],
            weights: [2.0, 2.0, 0.0, 0.0],
        };
        fast_lbs_normalize(&mut r);
        let s: f32 = r.weights.iter().sum();
        assert!((s - 1.0).abs() < 1e-5, /* normalized weights should sum to 1 */);
    }

    #[test]
    fn test_normalize_zero_weights_unchanged() {
        let mut r = FastLbsRecord::default();
        fast_lbs_normalize(&mut r);
        let s: f32 = r.weights.iter().sum();
        assert!((s).abs() < 1e-6 /* zero-weight record stays zero */,);
    }

    #[test]
    fn test_transform_returns_source() {
        let lbs = new_fast_lbs(2, 2);
        let pos = fast_lbs_transform(&lbs, 0, [1.0, 2.0, 3.0], &[]);
        assert!((pos[0] - 1.0).abs() < 1e-5, /* stub returns source position */);
    }

    #[test]
    fn test_to_json_contains_vertices() {
        let lbs = new_fast_lbs(5, 3);
        let j = fast_lbs_to_json(&lbs);
        assert!(j.contains("vertices") /* JSON must contain vertices */,);
    }

    #[test]
    fn test_bone_count_stored() {
        let lbs = new_fast_lbs(4, 7);
        assert_eq!(lbs.bone_count, 7 /* bone count must match */,);
    }
}
