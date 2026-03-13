// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Python pickle protocol stub export.

/// Pickle protocol version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PickleProtocol {
    V2 = 2,
    V4 = 4,
    V5 = 5,
}

/// A simplified pickle object descriptor.
#[derive(Debug, Clone)]
pub enum PickleValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Bytes(Vec<u8>),
    Str(String),
    List(Vec<PickleValue>),
    Dict(Vec<(String, PickleValue)>),
    None,
}

impl PickleValue {
    /// Returns a rough byte estimate of the serialised value.
    pub fn size_estimate(&self) -> usize {
        match self {
            PickleValue::Int(_) => 9,
            PickleValue::Float(_) => 9,
            PickleValue::Bool(_) => 2,
            PickleValue::None => 2,
            PickleValue::Bytes(b) => b.len() + 5,
            PickleValue::Str(s) => s.len() + 5,
            PickleValue::List(l) => l.iter().map(|v| v.size_estimate()).sum::<usize>() + 4,
            PickleValue::Dict(d) => {
                d.iter()
                    .map(|(k, v)| k.len() + v.size_estimate() + 4)
                    .sum::<usize>()
                    + 4
            }
        }
    }
}

/// Pickle file stub.
#[derive(Debug, Clone)]
pub struct PickleExport {
    pub protocol: PickleProtocol,
    pub root: PickleValue,
}

impl Default for PickleExport {
    fn default() -> Self {
        Self {
            protocol: PickleProtocol::V4,
            root: PickleValue::None,
        }
    }
}

/// Creates a new pickle export stub.
pub fn new_pickle_export(protocol: PickleProtocol) -> PickleExport {
    PickleExport {
        protocol,
        root: PickleValue::None,
    }
}

/// Sets the root value to serialise.
pub fn set_pickle_root(export: &mut PickleExport, value: PickleValue) {
    export.root = value;
}

/// Returns `true` if the root is not `None`.
pub fn pickle_has_content(export: &PickleExport) -> bool {
    !matches!(export.root, PickleValue::None)
}

/// Estimates the serialised size in bytes (including protocol header).
pub fn pickle_size_estimate(export: &PickleExport) -> usize {
    export.root.size_estimate() + 4 /* PROTO + STOP bytes */
}

/// Returns a minimal textual description.
pub fn pickle_summary(export: &PickleExport) -> String {
    let kind = match &export.root {
        PickleValue::Dict(_) => "dict",
        PickleValue::List(_) => "list",
        PickleValue::Str(_) => "str",
        PickleValue::Int(_) => "int",
        PickleValue::Float(_) => "float",
        PickleValue::Bool(_) => "bool",
        PickleValue::Bytes(_) => "bytes",
        PickleValue::None => "None",
    };
    format!(
        "pickle v{} root={} est_bytes={}",
        export.protocol.clone() as u8,
        kind,
        pickle_size_estimate(export)
    )
}

/// Validates that the protocol is at least V4 (for NEWOBJ_EX support).
pub fn validate_pickle_protocol(export: &PickleExport) -> bool {
    export.protocol >= PickleProtocol::V4
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> PickleExport {
        let mut e = new_pickle_export(PickleProtocol::V4);
        set_pickle_root(
            &mut e,
            PickleValue::Dict(vec![
                ("epoch".into(), PickleValue::Int(10)),
                ("loss".into(), PickleValue::Float(0.123)),
            ]),
        );
        e
    }

    #[test]
    fn new_export_protocol() {
        let e = new_pickle_export(PickleProtocol::V5);
        assert_eq!(e.protocol, PickleProtocol::V5);
    }

    #[test]
    fn has_content_after_set() {
        let e = sample_export();
        assert!(pickle_has_content(&e));
    }

    #[test]
    fn no_content_initially() {
        let e = new_pickle_export(PickleProtocol::V4);
        assert!(!pickle_has_content(&e));
    }

    #[test]
    fn size_estimate_positive() {
        let e = sample_export();
        assert!(pickle_size_estimate(&e) > 0);
    }

    #[test]
    fn summary_contains_dict() {
        let e = sample_export();
        assert!(pickle_summary(&e).contains("dict"));
    }

    #[test]
    fn validate_v4_ok() {
        let e = sample_export();
        assert!(validate_pickle_protocol(&e));
    }

    #[test]
    fn validate_v2_false() {
        let e = new_pickle_export(PickleProtocol::V2);
        assert!(!validate_pickle_protocol(&e));
    }

    #[test]
    fn int_size_estimate() {
        assert_eq!(PickleValue::Int(42).size_estimate(), 9);
    }

    #[test]
    fn protocol_ordering() {
        assert!(PickleProtocol::V5 > PickleProtocol::V4);
    }
}
