// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Generic model checkpoint stub export.

/// Training phase associated with the checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub enum CheckpointPhase {
    Pretrain,
    Finetune,
    Validation,
    Final,
}

/// A single scalar metric stored in the checkpoint.
#[derive(Debug, Clone)]
pub struct CheckpointMetric {
    pub name: String,
    pub value: f64,
}

/// Generic model checkpoint stub.
#[derive(Debug, Clone, Default)]
pub struct CheckpointExport {
    pub model_name: String,
    pub epoch: u32,
    pub global_step: u64,
    pub phase: Option<CheckpointPhase>,
    pub metrics: Vec<CheckpointMetric>,
    pub file_paths: Vec<String>,
    pub extra_info: Vec<(String, String)>,
}

/// Creates a new checkpoint export stub.
pub fn new_checkpoint_export(model_name: &str, epoch: u32) -> CheckpointExport {
    CheckpointExport {
        model_name: model_name.to_string(),
        epoch,
        ..Default::default()
    }
}

/// Sets the training phase.
pub fn set_checkpoint_phase(export: &mut CheckpointExport, phase: CheckpointPhase) {
    export.phase = Some(phase);
}

/// Adds a scalar metric.
pub fn add_checkpoint_metric(export: &mut CheckpointExport, name: &str, value: f64) {
    export.metrics.push(CheckpointMetric {
        name: name.to_string(),
        value,
    });
}

/// Adds a file path (e.g. weights shard).
pub fn add_checkpoint_file(export: &mut CheckpointExport, path: &str) {
    export.file_paths.push(path.to_string());
}

/// Finds a metric by name.
pub fn find_checkpoint_metric(export: &CheckpointExport, name: &str) -> Option<f64> {
    export
        .metrics
        .iter()
        .find(|m| m.name == name)
        .map(|m| m.value)
}

/// Validates the checkpoint (non-empty model name and at least one file path).
pub fn validate_checkpoint(export: &CheckpointExport) -> bool {
    !export.model_name.is_empty() && !export.file_paths.is_empty()
}

/// Estimates the metadata size in bytes.
pub fn checkpoint_metadata_size(export: &CheckpointExport) -> usize {
    let metric_bytes: usize = export.metrics.iter().map(|m| m.name.len() + 8).sum();
    let file_bytes: usize = export.file_paths.iter().map(|p| p.len() + 4).sum();
    let info_bytes: usize = export
        .extra_info
        .iter()
        .map(|(k, v)| k.len() + v.len() + 4)
        .sum();
    metric_bytes + file_bytes + info_bytes + 64 /* fixed header */
}

/// Returns a JSON-like summary.
pub fn checkpoint_summary_json(export: &CheckpointExport) -> String {
    format!(
        "{{\"model\":\"{}\",\"epoch\":{},\"global_step\":{},\"metrics\":{},\"files\":{}}}",
        export.model_name,
        export.epoch,
        export.global_step,
        export.metrics.len(),
        export.file_paths.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export() -> CheckpointExport {
        let mut e = new_checkpoint_export("GPT-2", 10);
        add_checkpoint_metric(&mut e, "loss", 1.23);
        add_checkpoint_metric(&mut e, "perplexity", 3.42);
        add_checkpoint_file(&mut e, "weights-00001-of-00002.safetensors");
        add_checkpoint_file(&mut e, "weights-00002-of-00002.safetensors");
        set_checkpoint_phase(&mut e, CheckpointPhase::Finetune);
        e.global_step = 50_000;
        e
    }

    #[test]
    fn model_name_stored() {
        let e = new_checkpoint_export("BERT", 5);
        assert_eq!(e.model_name, "BERT");
    }

    #[test]
    fn epoch_stored() {
        let e = sample_export();
        assert_eq!(e.epoch, 10);
    }

    #[test]
    fn validate_complete() {
        let e = sample_export();
        assert!(validate_checkpoint(&e));
    }

    #[test]
    fn validate_no_files_false() {
        let e = new_checkpoint_export("BERT", 1);
        assert!(!validate_checkpoint(&e));
    }

    #[test]
    fn find_metric_found() {
        let e = sample_export();
        let loss = find_checkpoint_metric(&e, "loss");
        assert!(loss.is_some());
        assert!((loss.expect("should succeed") - 1.23).abs() < 1e-9);
    }

    #[test]
    fn find_metric_missing() {
        let e = sample_export();
        assert!(find_checkpoint_metric(&e, "accuracy").is_none());
    }

    #[test]
    fn file_count() {
        let e = sample_export();
        assert_eq!(e.file_paths.len(), 2);
    }

    #[test]
    fn phase_stored() {
        let e = sample_export();
        assert_eq!(e.phase, Some(CheckpointPhase::Finetune));
    }

    #[test]
    fn metadata_size_positive() {
        let e = sample_export();
        assert!(checkpoint_metadata_size(&e) > 0);
    }

    #[test]
    fn summary_json_has_model() {
        let e = sample_export();
        let json = checkpoint_summary_json(&e);
        assert!(json.contains("GPT-2"));
    }
}
