// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! TorchScript model stub export.

/// TorchScript method descriptor.
#[derive(Debug, Clone)]
pub struct TsMethod {
    pub name: String,
    pub docstring: String,
    pub input_count: usize,
    pub output_count: usize,
}

/// TorchScript module stub.
#[derive(Debug, Clone, Default)]
pub struct TorchScriptExport {
    pub class_name: String,
    pub methods: Vec<TsMethod>,
    pub constants: Vec<(String, f64)>,
    pub extra_files: Vec<(String, String)>,
    pub torch_version: String,
}

/// Creates a new TorchScript export stub.
pub fn new_torch_script_export(class_name: &str, torch_version: &str) -> TorchScriptExport {
    TorchScriptExport {
        class_name: class_name.to_string(),
        torch_version: torch_version.to_string(),
        ..Default::default()
    }
}

/// Adds a method to the module.
pub fn add_ts_method(export: &mut TorchScriptExport, method: TsMethod) {
    export.methods.push(method);
}

/// Adds a constant to the module.
pub fn add_ts_constant(export: &mut TorchScriptExport, name: &str, value: f64) {
    export.constants.push((name.to_string(), value));
}

/// Adds an extra file (e.g. `extra/config.json`).
pub fn add_ts_extra_file(export: &mut TorchScriptExport, path: &str, content: &str) {
    export
        .extra_files
        .push((path.to_string(), content.to_string()));
}

/// Returns the method count.
pub fn ts_method_count(export: &TorchScriptExport) -> usize {
    export.methods.len()
}

/// Returns `true` if the module has a `forward` method.
pub fn has_forward_method(export: &TorchScriptExport) -> bool {
    export.methods.iter().any(|m| m.name == "forward")
}

/// Estimates the serialised size in bytes.
pub fn ts_size_estimate(export: &TorchScriptExport) -> usize {
    let method_bytes: usize = export
        .methods
        .iter()
        .map(|m| m.name.len() + m.docstring.len() + 64)
        .sum();
    let extra_bytes: usize = export
        .extra_files
        .iter()
        .map(|(k, v)| k.len() + v.len())
        .sum();
    method_bytes + extra_bytes + export.class_name.len() + 256
}

/// Validates that the module has at least one method.
pub fn validate_ts_export(export: &TorchScriptExport) -> bool {
    !export.methods.is_empty() && !export.class_name.is_empty()
}

/// Returns a JSON-like summary.
pub fn ts_summary_json(export: &TorchScriptExport) -> String {
    format!(
        "{{\"class\":\"{}\",\"torch_version\":\"{}\",\"methods\":{},\"constants\":{}}}",
        export.class_name,
        export.torch_version,
        export.methods.len(),
        export.constants.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> TorchScriptExport {
        let mut e = new_torch_script_export("MyModel", "2.1");
        add_ts_method(
            &mut e,
            TsMethod {
                name: "forward".into(),
                docstring: "Main forward pass".into(),
                input_count: 1,
                output_count: 1,
            },
        );
        e
    }

    #[test]
    fn new_export_class_name() {
        let e = new_torch_script_export("Net", "2.0");
        assert_eq!(e.class_name, "Net");
    }

    #[test]
    fn add_method_increments() {
        let mut e = new_torch_script_export("Net", "2.0");
        add_ts_method(
            &mut e,
            TsMethod {
                name: "encode".into(),
                docstring: "".into(),
                input_count: 1,
                output_count: 1,
            },
        );
        assert_eq!(ts_method_count(&e), 1);
    }

    #[test]
    fn has_forward_method_true() {
        let e = sample_export();
        assert!(has_forward_method(&e));
    }

    #[test]
    fn has_forward_method_false() {
        let e = new_torch_script_export("Net", "2.0");
        assert!(!has_forward_method(&e));
    }

    #[test]
    fn validate_with_method() {
        let e = sample_export();
        assert!(validate_ts_export(&e));
    }

    #[test]
    fn validate_empty_false() {
        let e = new_torch_script_export("Net", "2.0");
        assert!(!validate_ts_export(&e));
    }

    #[test]
    fn size_estimate_positive() {
        let e = sample_export();
        assert!(ts_size_estimate(&e) > 0);
    }

    #[test]
    fn constants_stored() {
        let mut e = sample_export();
        add_ts_constant(&mut e, "scale", 0.5);
        assert_eq!(e.constants.len(), 1);
    }

    #[test]
    fn summary_json_contains_class() {
        let e = sample_export();
        let json = ts_summary_json(&e);
        assert!(json.contains("MyModel"));
    }
}
