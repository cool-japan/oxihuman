// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! JSON Patch (RFC 6902) applier stub.

/// A single JSON Patch operation.
#[derive(Debug, Clone, PartialEq)]
pub enum PatchOp {
    Add { path: String, value: String },
    Remove { path: String },
    Replace { path: String, value: String },
    Move { from: String, path: String },
    Copy { from: String, path: String },
    Test { path: String, value: String },
}

/// Error type for patch operations.
#[derive(Debug, Clone, PartialEq)]
pub enum PatchError {
    InvalidOperation(String),
    PathNotFound(String),
    TestFailed { path: String, expected: String },
    MissingField(String),
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidOperation(s) => write!(f, "invalid operation: {s}"),
            Self::PathNotFound(p) => write!(f, "path not found: {p}"),
            Self::TestFailed { path, expected } => {
                write!(f, "test failed at {path}: expected {expected}")
            }
            Self::MissingField(s) => write!(f, "missing required field: {s}"),
        }
    }
}

/// A JSON Patch document (sequence of operations).
#[derive(Debug, Clone, Default)]
pub struct JsonPatch {
    ops: Vec<PatchOp>,
}

impl JsonPatch {
    /// Create an empty patch.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an operation to the patch.
    pub fn push(&mut self, op: PatchOp) {
        self.ops.push(op);
    }

    /// Return the operations in this patch.
    pub fn ops(&self) -> &[PatchOp] {
        &self.ops
    }

    /// Return the number of operations.
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Return `true` if the patch has no operations.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

/// Parse a JSON Patch operation kind string.
pub fn parse_op_kind(s: &str) -> Result<&'static str, PatchError> {
    match s {
        "add" => Ok("add"),
        "remove" => Ok("remove"),
        "replace" => Ok("replace"),
        "move" => Ok("move"),
        "copy" => Ok("copy"),
        "test" => Ok("test"),
        other => Err(PatchError::InvalidOperation(other.to_string())),
    }
}

/// Validate that a patch path is non-empty.
pub fn validate_path(path: &str) -> Result<(), PatchError> {
    if path.is_empty() {
        return Err(PatchError::PathNotFound(path.to_string()));
    }
    Ok(())
}

/// Count operations of each kind in a patch.
pub fn count_ops(patch: &JsonPatch) -> [usize; 6] {
    /* [add, remove, replace, move, copy, test] */
    let mut counts = [0usize; 6];
    for op in patch.ops() {
        let idx = match op {
            PatchOp::Add { .. } => 0,
            PatchOp::Remove { .. } => 1,
            PatchOp::Replace { .. } => 2,
            PatchOp::Move { .. } => 3,
            PatchOp::Copy { .. } => 4,
            PatchOp::Test { .. } => 5,
        };
        counts[idx] += 1;
    }
    counts
}

/// Return `true` if the patch contains any `Test` operations.
pub fn has_test_ops(patch: &JsonPatch) -> bool {
    patch
        .ops()
        .iter()
        .any(|op| matches!(op, PatchOp::Test { .. }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_patch() {
        /* new patch has no ops */
        let p = JsonPatch::new();
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn test_push_ops() {
        /* push increments len */
        let mut p = JsonPatch::new();
        p.push(PatchOp::Add {
            path: "/a".to_string(),
            value: "1".to_string(),
        });
        p.push(PatchOp::Remove {
            path: "/b".to_string(),
        });
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn test_count_ops() {
        /* count_ops categorises correctly */
        let mut p = JsonPatch::new();
        p.push(PatchOp::Add {
            path: "/x".to_string(),
            value: "v".to_string(),
        });
        p.push(PatchOp::Add {
            path: "/y".to_string(),
            value: "v".to_string(),
        });
        p.push(PatchOp::Remove {
            path: "/z".to_string(),
        });
        let counts = count_ops(&p);
        assert_eq!(counts[0], 2); /* add */
        assert_eq!(counts[1], 1); /* remove */
    }

    #[test]
    fn test_has_test_ops_false() {
        /* no test ops */
        let mut p = JsonPatch::new();
        p.push(PatchOp::Replace {
            path: "/a".to_string(),
            value: "1".to_string(),
        });
        assert!(!has_test_ops(&p));
    }

    #[test]
    fn test_has_test_ops_true() {
        /* patch with test op */
        let mut p = JsonPatch::new();
        p.push(PatchOp::Test {
            path: "/a".to_string(),
            value: "1".to_string(),
        });
        assert!(has_test_ops(&p));
    }

    #[test]
    fn test_validate_path_ok() {
        /* non-empty path is valid */
        assert!(validate_path("/foo").is_ok());
    }

    #[test]
    fn test_validate_path_empty() {
        /* empty path is invalid */
        assert!(validate_path("").is_err());
    }

    #[test]
    fn test_parse_op_kind_valid() {
        /* known operations parse without error */
        assert!(parse_op_kind("add").is_ok());
        assert!(parse_op_kind("remove").is_ok());
    }

    #[test]
    fn test_parse_op_kind_invalid() {
        /* unknown operation returns error */
        assert!(parse_op_kind("upsert").is_err());
    }

    #[test]
    fn test_ops_slice() {
        /* ops() returns correct slice */
        let mut p = JsonPatch::new();
        p.push(PatchOp::Remove {
            path: "/k".to_string(),
        });
        assert_eq!(p.ops().len(), 1);
    }
}
