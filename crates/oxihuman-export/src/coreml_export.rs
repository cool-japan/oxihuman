// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Core ML model stub export (.mlmodel / .mlpackage).

/// Core ML layer type tag.
#[derive(Debug, Clone, PartialEq)]
pub enum CoreMlLayerType {
    InnerProduct,
    Convolution,
    BatchNorm,
    Activation,
    Pooling,
    Softmax,
    Custom(String),
}

/// Core ML feature description.
#[derive(Debug, Clone)]
pub struct CoreMlFeature {
    pub name: String,
    pub is_optional: bool,
    pub shape_hint: Vec<i64>,
}

/// Core ML layer stub.
#[derive(Debug, Clone)]
pub struct CoreMlLayer {
    pub name: String,
    pub layer_type: CoreMlLayerType,
    pub input_names: Vec<String>,
    pub output_names: Vec<String>,
}

/// Core ML model stub.
#[derive(Debug, Clone, Default)]
pub struct CoreMlExport {
    pub spec_version: u32,
    pub description: String,
    pub input_features: Vec<CoreMlFeature>,
    pub output_features: Vec<CoreMlFeature>,
    pub layers: Vec<CoreMlLayer>,
    pub minimum_deployment_target: String,
}

/// Creates a new Core ML export stub.
pub fn new_coreml_export(description: &str, target: &str) -> CoreMlExport {
    CoreMlExport {
        spec_version: 6,
        description: description.to_string(),
        minimum_deployment_target: target.to_string(),
        ..Default::default()
    }
}

/// Adds an input feature.
pub fn add_coreml_input(export: &mut CoreMlExport, feat: CoreMlFeature) {
    export.input_features.push(feat);
}

/// Adds an output feature.
pub fn add_coreml_output(export: &mut CoreMlExport, feat: CoreMlFeature) {
    export.output_features.push(feat);
}

/// Adds a layer.
pub fn add_coreml_layer(export: &mut CoreMlExport, layer: CoreMlLayer) {
    export.layers.push(layer);
}

/// Returns total layer count.
pub fn coreml_layer_count(export: &CoreMlExport) -> usize {
    export.layers.len()
}

/// Validates the model (at least one input/output and one layer).
pub fn validate_coreml(export: &CoreMlExport) -> bool {
    !export.input_features.is_empty()
        && !export.output_features.is_empty()
        && !export.layers.is_empty()
}

/// Returns estimated size in bytes.
pub fn coreml_size_estimate(export: &CoreMlExport) -> usize {
    let layer_bytes: usize = export.layers.iter().map(|l| l.name.len() + 64).sum();
    layer_bytes + export.input_features.len() * 64 + export.output_features.len() * 64 + 256
}

/// Returns a JSON-like model header.
pub fn coreml_header_json(export: &CoreMlExport) -> String {
    format!(
        "{{\"spec_version\":{},\"layers\":{},\"inputs\":{},\"outputs\":{},\"target\":\"{}\"}}",
        export.spec_version,
        export.layers.len(),
        export.input_features.len(),
        export.output_features.len(),
        export.minimum_deployment_target
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> CoreMlExport {
        let mut e = new_coreml_export("Vision model", "iOS17");
        add_coreml_input(
            &mut e,
            CoreMlFeature {
                name: "image".into(),
                is_optional: false,
                shape_hint: vec![1, 3, 224, 224],
            },
        );
        add_coreml_output(
            &mut e,
            CoreMlFeature {
                name: "logits".into(),
                is_optional: false,
                shape_hint: vec![1, 1000],
            },
        );
        add_coreml_layer(
            &mut e,
            CoreMlLayer {
                name: "conv0".into(),
                layer_type: CoreMlLayerType::Convolution,
                input_names: vec!["image".into()],
                output_names: vec!["feat".into()],
            },
        );
        e
    }

    #[test]
    fn new_export_spec_version() {
        let e = new_coreml_export("test", "iOS16");
        assert_eq!(e.spec_version, 6);
    }

    #[test]
    fn add_layer_increments() {
        let e = sample_export();
        assert_eq!(coreml_layer_count(&e), 1);
    }

    #[test]
    fn validate_complete_model() {
        let e = sample_export();
        assert!(validate_coreml(&e));
    }

    #[test]
    fn validate_empty_false() {
        let e = new_coreml_export("test", "iOS16");
        assert!(!validate_coreml(&e));
    }

    #[test]
    fn size_estimate_positive() {
        let e = sample_export();
        assert!(coreml_size_estimate(&e) > 0);
    }

    #[test]
    fn header_json_has_target() {
        let e = sample_export();
        let json = coreml_header_json(&e);
        assert!(json.contains("iOS17"));
    }

    #[test]
    fn layer_type_eq() {
        assert_eq!(CoreMlLayerType::Softmax, CoreMlLayerType::Softmax);
    }

    #[test]
    fn input_feature_stored() {
        let e = sample_export();
        assert_eq!(e.input_features[0].name, "image");
    }

    #[test]
    fn deployment_target_stored() {
        let e = new_coreml_export("test", "macOS14");
        assert_eq!(e.minimum_deployment_target, "macOS14");
    }
}
