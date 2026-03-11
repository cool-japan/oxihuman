#![allow(dead_code)]
//! Validates morph data: weights, deltas, names.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MorphValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub passed: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MorphValidator {
    max_weight: f32,
    max_delta: f32,
}

#[allow(dead_code)]
pub fn new_morph_validator(max_weight: f32, max_delta: f32) -> MorphValidator {
    MorphValidator {
        max_weight,
        max_delta,
    }
}

#[allow(dead_code)]
pub fn validate_weights(v: &MorphValidator, weights: &[f32]) -> MorphValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for (i, &w) in weights.iter().enumerate() {
        if w < 0.0 {
            errors.push(format!("weight[{i}] is negative: {w}"));
        } else if w > v.max_weight {
            warnings.push(format!("weight[{i}] exceeds max: {w} > {}", v.max_weight));
        }
    }
    let passed = errors.is_empty();
    MorphValidationResult {
        errors,
        warnings,
        passed,
    }
}

#[allow(dead_code)]
pub fn validate_deltas(v: &MorphValidator, deltas: &[[f32; 3]]) -> MorphValidationResult {
    let mut errors = Vec::new();
    let warnings = Vec::new();
    for (i, d) in deltas.iter().enumerate() {
        let mag = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if mag > v.max_delta {
            errors.push(format!("delta[{i}] magnitude {mag} exceeds max {}", v.max_delta));
        }
    }
    let passed = errors.is_empty();
    MorphValidationResult {
        errors,
        warnings,
        passed,
    }
}

#[allow(dead_code)]
pub fn validate_names(names: &[&str]) -> MorphValidationResult {
    let mut errors = Vec::new();
    let warnings = Vec::new();
    for (i, &name) in names.iter().enumerate() {
        if name.is_empty() {
            errors.push(format!("name[{i}] is empty"));
        }
    }
    let passed = errors.is_empty();
    MorphValidationResult {
        errors,
        warnings,
        passed,
    }
}

#[allow(dead_code)]
pub fn validation_error_count(r: &MorphValidationResult) -> usize {
    r.errors.len()
}

#[allow(dead_code)]
pub fn validation_warnings(r: &MorphValidationResult) -> &[String] {
    &r.warnings
}

#[allow(dead_code)]
pub fn validation_passed(r: &MorphValidationResult) -> bool {
    r.passed
}

#[allow(dead_code)]
pub fn validation_to_json(r: &MorphValidationResult) -> String {
    let errs: Vec<String> = r.errors.iter().map(|e| format!("\"{e}\"")).collect();
    let warns: Vec<String> = r.warnings.iter().map(|w| format!("\"{w}\"")).collect();
    format!(
        "{{\"passed\":{},\"errors\":[{}],\"warnings\":[{}]}}",
        r.passed,
        errs.join(","),
        warns.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_validator() {
        let v = new_morph_validator(1.0, 10.0);
        assert!((v.max_weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_validate_weights_valid() {
        let v = new_morph_validator(1.0, 10.0);
        let r = validate_weights(&v, &[0.0, 0.5, 1.0]);
        assert!(validation_passed(&r));
    }

    #[test]
    fn test_validate_weights_negative() {
        let v = new_morph_validator(1.0, 10.0);
        let r = validate_weights(&v, &[-0.5]);
        assert!(!validation_passed(&r));
        assert_eq!(validation_error_count(&r), 1);
    }

    #[test]
    fn test_validate_weights_warning() {
        let v = new_morph_validator(1.0, 10.0);
        let r = validate_weights(&v, &[2.0]);
        assert!(validation_passed(&r));
        assert_eq!(validation_warnings(&r).len(), 1);
    }

    #[test]
    fn test_validate_deltas_valid() {
        let v = new_morph_validator(1.0, 10.0);
        let r = validate_deltas(&v, &[[1.0, 0.0, 0.0]]);
        assert!(validation_passed(&r));
    }

    #[test]
    fn test_validate_deltas_too_large() {
        let v = new_morph_validator(1.0, 1.0);
        let r = validate_deltas(&v, &[[10.0, 10.0, 10.0]]);
        assert!(!validation_passed(&r));
    }

    #[test]
    fn test_validate_names_valid() {
        let r = validate_names(&["smile", "blink"]);
        assert!(validation_passed(&r));
    }

    #[test]
    fn test_validate_names_empty() {
        let r = validate_names(&[""]);
        assert!(!validation_passed(&r));
    }

    #[test]
    fn test_validation_to_json() {
        let v = new_morph_validator(1.0, 10.0);
        let r = validate_weights(&v, &[0.5]);
        let json = validation_to_json(&r);
        assert!(json.contains("\"passed\":true"));
    }

    #[test]
    fn test_validation_error_count_zero() {
        let v = new_morph_validator(1.0, 10.0);
        let r = validate_weights(&v, &[0.5]);
        assert_eq!(validation_error_count(&r), 0);
    }
}
