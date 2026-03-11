#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Export container for bone weights.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneWeightExport {
    pub bone_names: Vec<String>,
    /// Per-vertex: list of (bone_index, weight) pairs.
    pub weights: Vec<Vec<(usize, f32)>>,
}

/// Create a `BoneWeightExport` from raw data.
#[allow(dead_code)]
pub fn export_bone_weights(
    bone_names: &[&str],
    weights: &[Vec<(usize, f32)>],
) -> BoneWeightExport {
    BoneWeightExport {
        bone_names: bone_names.iter().map(|s| s.to_string()).collect(),
        weights: weights.to_vec(),
    }
}

/// Return the number of vertices with weights.
#[allow(dead_code)]
pub fn weight_vertex_count(exp: &BoneWeightExport) -> usize {
    exp.weights.len()
}

/// Return the number of bones.
#[allow(dead_code)]
pub fn weight_bone_count(exp: &BoneWeightExport) -> usize {
    exp.bone_names.len()
}

/// Serialize to a JSON-like string.
#[allow(dead_code)]
pub fn weight_to_json(exp: &BoneWeightExport) -> String {
    let mut s = String::from("{\"bones\":[");
    for (i, name) in exp.bone_names.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('"');
        s.push_str(name);
        s.push('"');
    }
    s.push_str("],\"vertex_count\":");
    s.push_str(&exp.weights.len().to_string());
    s.push('}');
    s
}

/// Serialize to a compact byte representation (vertex count as u32 LE, then per-vertex influence count + data).
#[allow(dead_code)]
pub fn weight_to_bytes(exp: &BoneWeightExport) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(exp.weights.len() as u32).to_le_bytes());
    for w in &exp.weights {
        buf.extend_from_slice(&(w.len() as u32).to_le_bytes());
        for &(bi, bw) in w {
            buf.extend_from_slice(&(bi as u32).to_le_bytes());
            buf.extend_from_slice(&bw.to_le_bytes());
        }
    }
    buf
}

/// Return the maximum number of influences any single vertex has.
#[allow(dead_code)]
pub fn max_influences_per_vertex(exp: &BoneWeightExport) -> usize {
    exp.weights.iter().map(|w| w.len()).max().unwrap_or(0)
}

/// Normalize bone weights so they sum to 1.0 per vertex.
#[allow(dead_code)]
pub fn normalize_bone_weights_export(exp: &mut BoneWeightExport) {
    for w in &mut exp.weights {
        let sum: f32 = w.iter().map(|&(_, v)| v).sum();
        if sum > 0.0 {
            for pair in w.iter_mut() {
                pair.1 /= sum;
            }
        }
    }
}

/// Return the total byte size of the export.
#[allow(dead_code)]
pub fn bone_weight_export_size(exp: &BoneWeightExport) -> usize {
    weight_to_bytes(exp).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BoneWeightExport {
        export_bone_weights(
            &["hip", "spine", "head"],
            &[
                vec![(0, 0.5), (1, 0.3), (2, 0.2)],
                vec![(1, 1.0)],
            ],
        )
    }

    #[test]
    fn test_export_bone_weights() {
        let e = sample();
        assert_eq!(e.bone_names.len(), 3);
        assert_eq!(e.weights.len(), 2);
    }

    #[test]
    fn test_weight_vertex_count() {
        assert_eq!(weight_vertex_count(&sample()), 2);
    }

    #[test]
    fn test_weight_bone_count() {
        assert_eq!(weight_bone_count(&sample()), 3);
    }

    #[test]
    fn test_weight_to_json() {
        let j = weight_to_json(&sample());
        assert!(j.contains("\"bones\""));
        assert!(j.contains("\"hip\""));
        assert!(j.contains("\"vertex_count\":2"));
    }

    #[test]
    fn test_weight_to_bytes() {
        let b = weight_to_bytes(&sample());
        assert!(!b.is_empty());
        let vc = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        assert_eq!(vc, 2);
    }

    #[test]
    fn test_max_influences() {
        assert_eq!(max_influences_per_vertex(&sample()), 3);
    }

    #[test]
    fn test_normalize() {
        let mut e = export_bone_weights(&["a", "b"], &[vec![(0, 2.0), (1, 2.0)]]);
        normalize_bone_weights_export(&mut e);
        let sum: f32 = e.weights[0].iter().map(|p| p.1).sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_bone_weight_export_size() {
        let sz = bone_weight_export_size(&sample());
        assert!(sz > 4);
    }

    #[test]
    fn test_empty_export() {
        let e = export_bone_weights(&[], &[]);
        assert_eq!(weight_vertex_count(&e), 0);
        assert_eq!(max_influences_per_vertex(&e), 0);
    }

    #[test]
    fn test_normalize_zero_sum() {
        let mut e = export_bone_weights(&["a"], &[vec![(0, 0.0)]]);
        normalize_bone_weights_export(&mut e);
        assert!((e.weights[0][0].1).abs() < 1e-5);
    }
}
