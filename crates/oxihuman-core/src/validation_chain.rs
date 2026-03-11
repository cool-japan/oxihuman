#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Validation chain: compose multiple validators and collect errors.

/// A validation error with a message.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    #[allow(dead_code)]
    pub fn new(msg: &str) -> Self {
        ValidationError { message: msg.to_string() }
    }
}

/// Result of running the validation chain.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    #[allow(dead_code)]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Validator check function type alias.
#[allow(dead_code)]
pub type ValidatorFn = Box<dyn Fn(&str) -> Option<String> + Send + Sync>;

/// A validator: a named function that returns an optional error message.
#[allow(clippy::type_complexity)]
#[allow(dead_code)]
pub struct Validator {
    pub name: String,
    pub check: ValidatorFn,
}

impl std::fmt::Debug for Validator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Validator").field("name", &self.name).finish()
    }
}

/// A chain of validators.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct ValidationChain {
    validators: Vec<Validator>,
}

/// Create a new empty `ValidationChain`.
#[allow(dead_code)]
pub fn new_validation_chain() -> ValidationChain {
    ValidationChain::default()
}

/// Add a validator to the chain.
#[allow(dead_code)]
pub fn add_validator(
    chain: &mut ValidationChain,
    name: &str,
    check: impl Fn(&str) -> Option<String> + Send + Sync + 'static,
) {
    chain.validators.push(Validator { name: name.to_string(), check: Box::new(check) });
}

/// Run all validators against a value and collect errors.
#[allow(dead_code)]
pub fn validate_value(chain: &ValidationChain, value: &str) -> ValidationResult {
    let errors = chain
        .validators
        .iter()
        .filter_map(|v| (v.check)(value).map(|msg| ValidationError { message: msg }))
        .collect();
    ValidationResult { errors }
}

/// Return true if the value passes all validators.
#[allow(dead_code)]
pub fn validation_passed(chain: &ValidationChain, value: &str) -> bool {
    validate_value(chain, value).is_ok()
}

/// Return the number of errors in a `ValidationResult`.
#[allow(dead_code)]
pub fn validation_error_count(result: &ValidationResult) -> usize {
    result.errors.len()
}

/// Return the first error message, if any.
#[allow(dead_code)]
pub fn first_error(result: &ValidationResult) -> Option<&str> {
    result.errors.first().map(|e| e.message.as_str())
}

/// Remove all validators from the chain.
#[allow(dead_code)]
pub fn clear_validators(chain: &mut ValidationChain) {
    chain.validators.clear();
}

/// Return the number of validators in the chain.
#[allow(dead_code)]
pub fn validator_count(chain: &ValidationChain) -> usize {
    chain.validators.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_validation_chain() {
        let chain = new_validation_chain();
        assert_eq!(validator_count(&chain), 0);
    }

    #[test]
    fn test_add_validator() {
        let mut chain = new_validation_chain();
        add_validator(&mut chain, "non_empty", |v| {
            if v.is_empty() { Some("must not be empty".to_string()) } else { None }
        });
        assert_eq!(validator_count(&chain), 1);
    }

    #[test]
    fn test_validation_passed() {
        let mut chain = new_validation_chain();
        add_validator(&mut chain, "non_empty", |v| {
            if v.is_empty() { Some("empty".to_string()) } else { None }
        });
        assert!(validation_passed(&chain, "hello"));
        assert!(!validation_passed(&chain, ""));
    }

    #[test]
    fn test_validate_value_errors() {
        let mut chain = new_validation_chain();
        add_validator(&mut chain, "min_len", |v| {
            if v.len() < 5 { Some("too short".to_string()) } else { None }
        });
        add_validator(&mut chain, "no_spaces", |v| {
            if v.contains(' ') { Some("no spaces".to_string()) } else { None }
        });
        // "a b" length is 3 < 5 AND contains space -> 2 errors
        let result = validate_value(&chain, "a b");
        assert_eq!(validation_error_count(&result), 2);
    }

    #[test]
    fn test_first_error() {
        let mut chain = new_validation_chain();
        add_validator(&mut chain, "v", |_| Some("err".to_string()));
        let result = validate_value(&chain, "x");
        assert_eq!(first_error(&result), Some("err"));
    }

    #[test]
    fn test_no_errors() {
        let chain = new_validation_chain();
        let result = validate_value(&chain, "anything");
        assert_eq!(validation_error_count(&result), 0);
        assert!(first_error(&result).is_none());
    }

    #[test]
    fn test_clear_validators() {
        let mut chain = new_validation_chain();
        add_validator(&mut chain, "v", |_| None);
        clear_validators(&mut chain);
        assert_eq!(validator_count(&chain), 0);
    }

    #[test]
    fn test_validation_error_new() {
        let e = ValidationError::new("test error");
        assert_eq!(e.message, "test error");
    }

    #[test]
    fn test_validation_result_is_ok() {
        let result = ValidationResult { errors: vec![] };
        assert!(result.is_ok());
        let result2 = ValidationResult { errors: vec![ValidationError::new("e")] };
        assert!(!result2.is_ok());
    }
}
