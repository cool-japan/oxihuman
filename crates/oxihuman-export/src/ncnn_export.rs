// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! NCNN model stub export (.param / .bin).

/// NCNN operator descriptor.
#[derive(Debug, Clone)]
pub struct NcnnOp {
    pub op_type: String,
    pub name: String,
    pub num_inputs: usize,
    pub num_outputs: usize,
    pub params: Vec<(i32, String)>, /* param_id → value string */
}

/// NCNN model stub.
#[derive(Debug, Clone, Default)]
pub struct NcnnExport {
    pub magic: u32,
    pub layers: Vec<NcnnOp>,
    pub input_names: Vec<String>,
    pub output_names: Vec<String>,
}

/// Creates a new NCNN export stub.
pub fn new_ncnn_export() -> NcnnExport {
    NcnnExport {
        magic: 7767517,
        ..Default::default()
    }
}

/// Adds a layer to the model.
pub fn add_ncnn_layer(export: &mut NcnnExport, op: NcnnOp) {
    export.layers.push(op);
}

/// Returns total layer count.
pub fn ncnn_layer_count(export: &NcnnExport) -> usize {
    export.layers.len()
}

/// Registers an input blob name.
pub fn add_ncnn_input(export: &mut NcnnExport, name: &str) {
    export.input_names.push(name.to_string());
}

/// Registers an output blob name.
pub fn add_ncnn_output(export: &mut NcnnExport, name: &str) {
    export.output_names.push(name.to_string());
}

/// Finds a layer by name.
pub fn find_ncnn_layer<'a>(export: &'a NcnnExport, name: &str) -> Option<&'a NcnnOp> {
    export.layers.iter().find(|l| l.name == name)
}

/// Serialises a minimal `.param`-style text.
pub fn ncnn_param_text(export: &NcnnExport) -> String {
    let mut lines = vec![
        format!("{}", export.magic),
        format!(
            "{} {}",
            export.layers.len(),
            export.input_names.len() + export.output_names.len()
        ),
    ];
    for layer in &export.layers {
        lines.push(format!(
            "{} {} {} {}",
            layer.op_type, layer.name, layer.num_inputs, layer.num_outputs
        ));
    }
    lines.join("\n")
}

/// Validates that the model has layers and I/O.
pub fn validate_ncnn(export: &NcnnExport) -> bool {
    !export.layers.is_empty() && !export.input_names.is_empty()
}

/// Estimates the binary `.bin` size in bytes.
pub fn ncnn_bin_size_estimate(export: &NcnnExport) -> usize {
    export.layers.len() * 256 + 64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> NcnnExport {
        let mut e = new_ncnn_export();
        add_ncnn_input(&mut e, "data");
        add_ncnn_output(&mut e, "output");
        add_ncnn_layer(
            &mut e,
            NcnnOp {
                op_type: "Convolution".into(),
                name: "conv0".into(),
                num_inputs: 1,
                num_outputs: 1,
                params: vec![(0, "64".into())],
            },
        );
        e
    }

    #[test]
    fn magic_value() {
        let e = new_ncnn_export();
        assert_eq!(e.magic, 7767517);
    }

    #[test]
    fn add_layer_increments() {
        let e = sample_export();
        assert_eq!(ncnn_layer_count(&e), 1);
    }

    #[test]
    fn find_layer_found() {
        let e = sample_export();
        assert!(find_ncnn_layer(&e, "conv0").is_some());
    }

    #[test]
    fn find_layer_not_found() {
        let e = sample_export();
        assert!(find_ncnn_layer(&e, "missing").is_none());
    }

    #[test]
    fn validate_with_io() {
        let e = sample_export();
        assert!(validate_ncnn(&e));
    }

    #[test]
    fn validate_empty_false() {
        let e = new_ncnn_export();
        assert!(!validate_ncnn(&e));
    }

    #[test]
    fn param_text_has_magic() {
        let e = sample_export();
        let text = ncnn_param_text(&e);
        assert!(text.contains("7767517"));
    }

    #[test]
    fn bin_size_positive() {
        let e = sample_export();
        assert!(ncnn_bin_size_estimate(&e) > 0);
    }

    #[test]
    fn input_names_stored() {
        let e = sample_export();
        assert!(e.input_names.contains(&"data".to_string()));
    }
}
