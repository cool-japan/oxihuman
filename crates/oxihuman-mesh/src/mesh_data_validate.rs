#![allow(dead_code)]
//! Validate mesh data integrity.

/// A validation report.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validate all mesh data.
#[allow(dead_code)]
pub fn validate_mesh_data(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[[u32; 3]],
) -> ValidationReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if let Some(e) = mv_check_index_bounds(positions, indices) {
        errors.push(e);
    }
    if let Some(w) = check_normal_lengths(normals) {
        warnings.push(w);
    }
    if let Some(w) = check_uv_range(uvs) {
        warnings.push(w);
    }
    if let Some(w) = check_degenerate_faces(positions, indices) {
        warnings.push(w);
    }

    ValidationReport { errors, warnings }
}

/// Check that all indices are within bounds.
#[allow(dead_code)]
pub fn mv_check_index_bounds(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Option<String> {
    let n = positions.len() as u32;
    for (fi, tri) in indices.iter().enumerate() {
        for &vi in tri {
            if vi >= n {
                return Some(format!("face {} has out-of-bounds index {}", fi, vi));
            }
        }
    }
    None
}

/// Check that all normals have approximately unit length.
#[allow(dead_code)]
pub fn check_normal_lengths(normals: &[[f32; 3]]) -> Option<String> {
    let mut bad_count = 0usize;
    for n in normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if (len - 1.0).abs() > 0.01 {
            bad_count += 1;
        }
    }
    if bad_count > 0 {
        Some(format!("{} normals have non-unit length", bad_count))
    } else {
        None
    }
}

/// Check that UVs are in [0,1] range.
#[allow(dead_code)]
pub fn check_uv_range(uvs: &[[f32; 2]]) -> Option<String> {
    let mut out_count = 0usize;
    for uv in uvs {
        if !(0.0..=1.0).contains(&uv[0]) || !(0.0..=1.0).contains(&uv[1]) {
            out_count += 1;
        }
    }
    if out_count > 0 {
        Some(format!("{} UVs outside [0,1] range", out_count))
    } else {
        None
    }
}

/// Check for degenerate (zero-area) faces.
#[allow(dead_code)]
pub fn check_degenerate_faces(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Option<String> {
    let mut deg_count = 0usize;
    for tri in indices {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        let area = 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        if area < 1e-10 {
            deg_count += 1;
        }
    }
    if deg_count > 0 {
        Some(format!("{} degenerate faces found", deg_count))
    } else {
        None
    }
}

/// Check if all validations passed.
#[allow(dead_code)]
pub fn validation_passed(report: &ValidationReport) -> bool {
    report.errors.is_empty()
}

/// Count validation errors.
#[allow(dead_code)]
pub fn validation_error_count(report: &ValidationReport) -> usize {
    report.errors.len()
}

/// Convert validation report to a string.
#[allow(dead_code)]
pub fn validation_to_string(report: &ValidationReport) -> String {
    let mut s = String::new();
    for e in &report.errors {
        s.push_str(&format!("ERROR: {}\n", e));
    }
    for w in &report.warnings {
        s.push_str(&format!("WARN: {}\n", w));
    }
    if report.errors.is_empty() && report.warnings.is_empty() {
        s.push_str("OK\n");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_good_mesh() {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let n = vec![[0.0, 0.0, 1.0]; 3];
        let uv = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let i = vec![[0u32, 1, 2]];
        let report = validate_mesh_data(&p, &n, &uv, &i);
        assert!(validation_passed(&report));
    }

    #[test]
    fn test_check_index_bounds_bad() {
        let p = vec![[0.0; 3]; 3];
        let i = vec![[0u32, 1, 10]];
        let err = mv_check_index_bounds(&p, &i);
        assert!(err.is_some());
    }

    #[test]
    fn test_check_normal_lengths_bad() {
        let normals = vec![[0.0, 0.0, 2.0]];
        let warn = check_normal_lengths(&normals);
        assert!(warn.is_some());
    }

    #[test]
    fn test_check_uv_range_bad() {
        let uvs = vec![[-0.1, 0.5]];
        let warn = check_uv_range(&uvs);
        assert!(warn.is_some());
    }

    #[test]
    fn test_check_degenerate_faces() {
        let p = vec![[0.0; 3], [0.0; 3], [0.0; 3]];
        let i = vec![[0u32, 1, 2]];
        let warn = check_degenerate_faces(&p, &i);
        assert!(warn.is_some());
    }

    #[test]
    fn test_validation_passed() {
        let report = ValidationReport { errors: vec![], warnings: vec![] };
        assert!(validation_passed(&report));
    }

    #[test]
    fn test_validation_error_count() {
        let report = ValidationReport {
            errors: vec!["err1".to_string(), "err2".to_string()],
            warnings: vec![],
        };
        assert_eq!(validation_error_count(&report), 2);
    }

    #[test]
    fn test_validation_to_string() {
        let report = ValidationReport { errors: vec![], warnings: vec![] };
        let s = validation_to_string(&report);
        assert!(s.contains("OK"));
    }

    #[test]
    fn test_validation_to_string_with_errors() {
        let report = ValidationReport {
            errors: vec!["bad index".to_string()],
            warnings: vec!["non-unit normal".to_string()],
        };
        let s = validation_to_string(&report);
        assert!(s.contains("ERROR"));
        assert!(s.contains("WARN"));
    }

    #[test]
    fn test_check_uv_range_good() {
        let uvs = vec![[0.5, 0.5]];
        assert!(check_uv_range(&uvs).is_none());
    }
}
