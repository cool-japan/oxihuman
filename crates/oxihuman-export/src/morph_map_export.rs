#![allow(dead_code)]
//! Export morph maps.

/// Morph map export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MorphMapExport {
    pub name: String,
    pub deltas: Vec<[f32; 3]>,
}

/// Export a morph map.
#[allow(dead_code)]
pub fn export_morph_map(name: &str, deltas: &[[f32; 3]]) -> MorphMapExport {
    MorphMapExport {
        name: name.to_string(),
        deltas: deltas.to_vec(),
    }
}

/// Return the morph map count (number of deltas).
#[allow(dead_code)]
pub fn morph_map_count(exp: &MorphMapExport) -> usize {
    exp.deltas.len()
}

/// Return the morph map name.
#[allow(dead_code)]
pub fn morph_map_name(exp: &MorphMapExport) -> &str {
    &exp.name
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn morph_map_to_json(exp: &MorphMapExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"delta_count\":{}}}",
        exp.name,
        exp.deltas.len()
    )
}

/// Serialize to bytes.
#[allow(dead_code)]
pub fn morph_map_to_bytes(exp: &MorphMapExport) -> Vec<u8> {
    let mut buf = Vec::new();
    for d in &exp.deltas {
        for &f in d {
            buf.extend_from_slice(&f.to_le_bytes());
        }
    }
    buf
}

/// Count non-zero deltas.
#[allow(dead_code)]
pub fn morph_map_delta_count(exp: &MorphMapExport) -> usize {
    exp.deltas
        .iter()
        .filter(|d| d[0].abs() > 1e-7 || d[1].abs() > 1e-7 || d[2].abs() > 1e-7)
        .count()
}

/// Compute export size.
#[allow(dead_code)]
pub fn morph_map_export_size(exp: &MorphMapExport) -> usize {
    exp.deltas.len() * 12
}

/// Validate morph map (no NaN/Inf).
#[allow(dead_code)]
pub fn validate_morph_map(exp: &MorphMapExport) -> bool {
    exp.deltas
        .iter()
        .all(|d| d.iter().all(|f| f.is_finite()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_morph_map() {
        let e = export_morph_map("smile", &[[0.1, 0.0, 0.0]]);
        assert_eq!(morph_map_count(&e), 1);
    }

    #[test]
    fn test_morph_map_name() {
        let e = export_morph_map("blink", &[]);
        assert_eq!(morph_map_name(&e), "blink");
    }

    #[test]
    fn test_morph_map_to_json() {
        let e = export_morph_map("t", &[[0.0; 3]]);
        let j = morph_map_to_json(&e);
        assert!(j.contains("\"name\":\"t\""));
    }

    #[test]
    fn test_morph_map_to_bytes() {
        let e = export_morph_map("t", &[[1.0, 2.0, 3.0]]);
        assert_eq!(morph_map_to_bytes(&e).len(), 12);
    }

    #[test]
    fn test_morph_map_delta_count() {
        let e = export_morph_map("t", &[[0.0; 3], [1.0, 0.0, 0.0]]);
        assert_eq!(morph_map_delta_count(&e), 1);
    }

    #[test]
    fn test_morph_map_export_size() {
        let e = export_morph_map("t", &[[0.0; 3]; 10]);
        assert_eq!(morph_map_export_size(&e), 120);
    }

    #[test]
    fn test_validate_morph_map() {
        let e = export_morph_map("t", &[[1.0, 2.0, 3.0]]);
        assert!(validate_morph_map(&e));
    }

    #[test]
    fn test_validate_nan() {
        let e = export_morph_map("t", &[[f32::NAN, 0.0, 0.0]]);
        assert!(!validate_morph_map(&e));
    }

    #[test]
    fn test_empty_morph_map() {
        let e = export_morph_map("e", &[]);
        assert_eq!(morph_map_count(&e), 0);
        assert_eq!(morph_map_export_size(&e), 0);
    }

    #[test]
    fn test_morph_map_count_all_zero() {
        let e = export_morph_map("t", &[[0.0; 3]; 5]);
        assert_eq!(morph_map_delta_count(&e), 0);
    }
}
