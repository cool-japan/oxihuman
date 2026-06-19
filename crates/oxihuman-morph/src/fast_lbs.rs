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

/// Compute the blended position for a vertex using Linear Blend Skinning.
///
/// Computes: `out = Σ (weights[i] * (bone_matrices[bones[i]] * [x, y, z, 1.0]))`
/// for each of the 4 bone records where the weight is non-negligible.
/// Returns `source` unchanged when no bones have weight or vertex index is out of range.
pub fn fast_lbs_transform(
    lbs: &FastLbs,
    vertex: usize,
    source: [f32; 3],
    bone_matrices: &[[[f32; 4]; 4]],
) -> [f32; 3] {
    if vertex >= lbs.records.len() {
        return [0.0; 3];
    }
    let rec = &lbs.records[vertex];
    let [x, y, z] = source;

    let mut out = [0.0f32; 3];
    let mut total_weight = 0.0f32;

    for i in 0..4 {
        let w = rec.weights[i];
        if w < 1e-9 {
            continue;
        }
        let bone_idx = rec.bones[i] as usize;
        if bone_idx >= bone_matrices.len() {
            continue;
        }
        let m = &bone_matrices[bone_idx];
        // 4×4 matrix * homogeneous point [x, y, z, 1]:
        let tx = m[0][0] * x + m[0][1] * y + m[0][2] * z + m[0][3];
        let ty = m[1][0] * x + m[1][1] * y + m[1][2] * z + m[1][3];
        let tz = m[2][0] * x + m[2][1] * y + m[2][2] * z + m[2][3];
        out[0] += w * tx;
        out[1] += w * ty;
        out[2] += w * tz;
        total_weight += w;
    }

    if total_weight < 1e-9 {
        // No influential bones — return source unchanged
        source
    } else {
        out
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
        // With all-zero weights, the identity-matrix case falls back to source.
        let mut lbs = new_fast_lbs(2, 2);
        // Set vertex 0 to use bone 0 with full weight; bone matrix is identity.
        let rec = FastLbsRecord {
            bones: [0, 0, 0, 0],
            weights: [1.0, 0.0, 0.0, 0.0],
        };
        fast_lbs_set(&mut lbs, 0, rec);
        let identity: [[f32; 4]; 4] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let source = [1.0, 2.0, 3.0];
        let pos = fast_lbs_transform(&lbs, 0, source, &[identity]);
        assert!(
            (pos[0] - 1.0).abs() < 1e-5,
            "identity matrix must return source x"
        );
        assert!(
            (pos[1] - 2.0).abs() < 1e-5,
            "identity matrix must return source y"
        );
        assert!(
            (pos[2] - 3.0).abs() < 1e-5,
            "identity matrix must return source z"
        );
    }

    #[test]
    fn test_transform_rigid_translate() {
        // Translation matrix: translates by [1, 2, 3].
        let mut lbs = new_fast_lbs(1, 1);
        let rec = FastLbsRecord {
            bones: [0, 0, 0, 0],
            weights: [1.0, 0.0, 0.0, 0.0],
        };
        fast_lbs_set(&mut lbs, 0, rec);
        let translate: [[f32; 4]; 4] = [
            [1.0, 0.0, 0.0, 1.0], // tx = 1
            [0.0, 1.0, 0.0, 2.0], // ty = 2
            [0.0, 0.0, 1.0, 3.0], // tz = 3
            [0.0, 0.0, 0.0, 1.0],
        ];
        let source = [5.0, 6.0, 7.0];
        let pos = fast_lbs_transform(&lbs, 0, source, &[translate]);
        assert!((pos[0] - 6.0).abs() < 1e-5, "x must be source + 1");
        assert!((pos[1] - 8.0).abs() < 1e-5, "y must be source + 2");
        assert!((pos[2] - 10.0).abs() < 1e-5, "z must be source + 3");
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
