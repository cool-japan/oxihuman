#![allow(dead_code)]
//! Export skin weights.

/// Skin weight export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SkinWeightExport {
    pub weights: Vec<Vec<(u32, f32)>>,
    pub format: String,
}

/// Export skin weights.
#[allow(dead_code)]
pub fn export_skin_weights(weights: &[Vec<(u32, f32)>]) -> SkinWeightExport {
    SkinWeightExport {
        weights: weights.to_vec(),
        format: "binary".to_string(),
    }
}

/// Return format string.
#[allow(dead_code)]
pub fn weight_format(exp: &SkinWeightExport) -> &str {
    &exp.format
}

/// Return total weight entry count.
#[allow(dead_code)]
pub fn weight_count_swe(exp: &SkinWeightExport) -> usize {
    exp.weights.iter().map(|v| v.len()).sum()
}

/// Serialize to bytes.
#[allow(dead_code)]
pub fn weight_to_bytes(exp: &SkinWeightExport) -> Vec<u8> {
    let mut buf = Vec::new();
    for vw in &exp.weights {
        buf.extend_from_slice(&(vw.len() as u32).to_le_bytes());
        for &(joint, w) in vw {
            buf.extend_from_slice(&joint.to_le_bytes());
            buf.extend_from_slice(&w.to_le_bytes());
        }
    }
    buf
}

/// Return max bones per vertex.
#[allow(dead_code)]
pub fn max_bones_per_vertex(exp: &SkinWeightExport) -> usize {
    exp.weights.iter().map(|v| v.len()).max().unwrap_or(0)
}

/// Normalize weights per vertex.
#[allow(dead_code)]
pub fn weight_normalize(exp: &mut SkinWeightExport) {
    for vw in &mut exp.weights {
        let sum: f32 = vw.iter().map(|&(_, w)| w).sum();
        if sum > 0.0 {
            for entry in vw.iter_mut() {
                entry.1 /= sum;
            }
        }
    }
}

/// Compute export size in bytes.
#[allow(dead_code)]
pub fn weight_export_size(exp: &SkinWeightExport) -> usize {
    weight_to_bytes(exp).len()
}

/// Validate skin weights (all non-negative, within [0,1]).
#[allow(dead_code)]
pub fn validate_skin_weights(exp: &SkinWeightExport) -> bool {
    exp.weights
        .iter()
        .all(|vw| vw.iter().all(|&(_, w)| (0.0..=1.0).contains(&w)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_skin_weights() {
        let w = vec![vec![(0, 0.5), (1, 0.5)]];
        let e = export_skin_weights(&w);
        assert_eq!(e.weights.len(), 1);
    }

    #[test]
    fn test_weight_format() {
        let e = export_skin_weights(&[]);
        assert_eq!(weight_format(&e), "binary");
    }

    #[test]
    fn test_weight_count() {
        let w = vec![vec![(0, 0.5), (1, 0.5)], vec![(0, 1.0)]];
        let e = export_skin_weights(&w);
        assert_eq!(weight_count_swe(&e), 3);
    }

    #[test]
    fn test_weight_to_bytes() {
        let w = vec![vec![(0u32, 1.0f32)]];
        let e = export_skin_weights(&w);
        assert!(!weight_to_bytes(&e).is_empty());
    }

    #[test]
    fn test_max_bones() {
        let w = vec![vec![(0, 0.3), (1, 0.3), (2, 0.4)]];
        let e = export_skin_weights(&w);
        assert_eq!(max_bones_per_vertex(&e), 3);
    }

    #[test]
    fn test_weight_normalize() {
        let w = vec![vec![(0, 3.0), (1, 1.0)]];
        let mut e = export_skin_weights(&w);
        weight_normalize(&mut e);
        let sum: f32 = e.weights[0].iter().map(|&(_, w)| w).sum();
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_weight_export_size() {
        let e = export_skin_weights(&[]);
        assert_eq!(weight_export_size(&e), 0);
    }

    #[test]
    fn test_validate_skin_weights() {
        let w = vec![vec![(0, 0.5), (1, 0.5)]];
        let e = export_skin_weights(&w);
        assert!(validate_skin_weights(&e));
    }

    #[test]
    fn test_validate_negative() {
        let w = vec![vec![(0, -0.1)]];
        let e = export_skin_weights(&w);
        assert!(!validate_skin_weights(&e));
    }

    #[test]
    fn test_empty_weights() {
        let e = export_skin_weights(&[]);
        assert_eq!(weight_count_swe(&e), 0);
        assert_eq!(max_bones_per_vertex(&e), 0);
    }
}
