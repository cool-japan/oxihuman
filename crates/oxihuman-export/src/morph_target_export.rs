#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A single morph target with vertex deltas.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphTarget {
    pub name: String,
    /// Flat delta positions [dx,dy,dz, ...].
    pub deltas: Vec<f32>,
}

/// Export container for morph targets.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphTargetExport {
    pub targets: Vec<MorphTarget>,
}

/// Create a morph target export.
#[allow(dead_code)]
pub fn export_morph_targets(names: &[&str], deltas: &[Vec<f32>]) -> MorphTargetExport {
    let mut targets = Vec::new();
    for (i, &name) in names.iter().enumerate() {
        targets.push(MorphTarget {
            name: name.to_string(),
            deltas: deltas.get(i).cloned().unwrap_or_default(),
        });
    }
    MorphTargetExport { targets }
}

/// Return the number of morph targets.
#[allow(dead_code)]
pub fn morph_target_count_export(exp: &MorphTargetExport) -> usize {
    exp.targets.len()
}

/// Return the name of a morph target.
#[allow(dead_code)]
pub fn morph_target_name(exp: &MorphTargetExport, index: usize) -> Option<&str> {
    exp.targets.get(index).map(|t| t.name.as_str())
}

/// Return the number of delta components for a target (len of deltas / 3).
#[allow(dead_code)]
pub fn morph_target_delta_count(exp: &MorphTargetExport, index: usize) -> usize {
    exp.targets.get(index).map_or(0, |t| t.deltas.len() / 3)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn morph_target_to_json(exp: &MorphTargetExport) -> String {
    let mut s = String::from("{\"targets\":[");
    for (i, t) in exp.targets.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"name\":\"{}\",\"delta_verts\":{}}}",
            t.name,
            t.deltas.len() / 3
        ));
    }
    s.push_str("]}");
    s
}

/// Serialize to bytes (target count u32 LE, then per-target: name_len u32, name bytes, delta_count u32, float data).
#[allow(dead_code)]
pub fn morph_target_to_bytes(exp: &MorphTargetExport) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(exp.targets.len() as u32).to_le_bytes());
    for t in &exp.targets {
        let name_bytes = t.name.as_bytes();
        buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(name_bytes);
        buf.extend_from_slice(&(t.deltas.len() as u32).to_le_bytes());
        for &d in &t.deltas {
            buf.extend_from_slice(&d.to_le_bytes());
        }
    }
    buf
}

/// Return the total byte size.
#[allow(dead_code)]
pub fn morph_target_export_size(exp: &MorphTargetExport) -> usize {
    morph_target_to_bytes(exp).len()
}

/// Validate that all targets have the same delta count.
#[allow(dead_code)]
pub fn validate_morph_target_export(exp: &MorphTargetExport) -> bool {
    if exp.targets.is_empty() {
        return true;
    }
    let first_len = exp.targets[0].deltas.len();
    exp.targets.iter().all(|t| t.deltas.len() == first_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> MorphTargetExport {
        export_morph_targets(
            &["smile", "blink"],
            &[
                vec![0.1, 0.0, 0.0, 0.0, 0.1, 0.0],
                vec![0.0, -0.1, 0.0, 0.0, 0.0, 0.1],
            ],
        )
    }

    #[test]
    fn test_count() {
        assert_eq!(morph_target_count_export(&sample()), 2);
    }

    #[test]
    fn test_name() {
        assert_eq!(morph_target_name(&sample(), 0), Some("smile"));
        assert_eq!(morph_target_name(&sample(), 1), Some("blink"));
        assert_eq!(morph_target_name(&sample(), 5), None);
    }

    #[test]
    fn test_delta_count() {
        assert_eq!(morph_target_delta_count(&sample(), 0), 2);
    }

    #[test]
    fn test_to_json() {
        let j = morph_target_to_json(&sample());
        assert!(j.contains("\"smile\""));
        assert!(j.contains("\"blink\""));
    }

    #[test]
    fn test_to_bytes() {
        let b = morph_target_to_bytes(&sample());
        let tc = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        assert_eq!(tc, 2);
    }

    #[test]
    fn test_export_size() {
        assert!(morph_target_export_size(&sample()) > 0);
    }

    #[test]
    fn test_validate_ok() {
        assert!(validate_morph_target_export(&sample()));
    }

    #[test]
    fn test_validate_mismatch() {
        let e = export_morph_targets(&["a", "b"], &[vec![1.0, 2.0, 3.0], vec![1.0]]);
        assert!(!validate_morph_target_export(&e));
    }

    #[test]
    fn test_empty() {
        let e = export_morph_targets(&[], &[]);
        assert_eq!(morph_target_count_export(&e), 0);
        assert!(validate_morph_target_export(&e));
    }

    #[test]
    fn test_delta_count_oob() {
        assert_eq!(morph_target_delta_count(&sample(), 99), 0);
    }
}
