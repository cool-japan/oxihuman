// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Batch export system for exporting multiple formats simultaneously.
//!
//! Provides a simple queue-based API: build a [`BatchExportConfig`] with one or
//! more [`BatchExportTarget`] entries, then call [`execute_batch`] to run them
//! all and collect [`BatchExportResult`] summaries.

#![allow(dead_code)]

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Export format variants supported by the batch system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchExportFormat {
    /// Binary glTF (.glb).
    Glb,
    /// Wavefront OBJ (.obj).
    Obj,
    /// COLLADA (.dae).
    Collada,
    /// Alembic (.abc).
    Abc,
    /// JSON mesh description (.json).
    Json,
}

impl BatchExportFormat {
    /// Return the conventional file extension for this format.
    pub fn extension(self) -> &'static str {
        match self {
            BatchExportFormat::Glb => "glb",
            BatchExportFormat::Obj => "obj",
            BatchExportFormat::Collada => "dae",
            BatchExportFormat::Abc => "abc",
            BatchExportFormat::Json => "json",
        }
    }

    /// Return a human-readable name for this format.
    pub fn name(self) -> &'static str {
        match self {
            BatchExportFormat::Glb => "Binary glTF",
            BatchExportFormat::Obj => "Wavefront OBJ",
            BatchExportFormat::Collada => "COLLADA",
            BatchExportFormat::Abc => "Alembic",
            BatchExportFormat::Json => "JSON Mesh",
        }
    }
}

// ── Structs ───────────────────────────────────────────────────────────────────

/// A single export target within a batch job.
#[derive(Debug, Clone)]
pub struct BatchExportTarget {
    /// Logical name for this target (used in reporting).
    pub label: String,
    /// Desired output format.
    pub format: BatchExportFormat,
    /// Output filename (relative to the batch output directory).
    pub filename: String,
    /// Whether this target is enabled. Disabled targets are skipped.
    pub enabled: bool,
}

/// Configuration for a complete batch export run.
#[derive(Debug, Clone)]
pub struct BatchExportConfig {
    /// Export targets to process.
    pub targets: Vec<BatchExportTarget>,
    /// Root output directory for all targets.
    pub output_directory: String,
    /// Whether to overwrite existing files.
    pub overwrite_existing: bool,
    /// Optional job label shown in summary output.
    pub job_label: String,
}

/// Result of processing a single [`BatchExportTarget`].
#[derive(Debug, Clone)]
pub struct BatchExportResult {
    /// Label of the target that produced this result.
    pub label: String,
    /// Format that was exported.
    pub format: BatchExportFormat,
    /// Whether the export succeeded.
    pub success: bool,
    /// Optional error description (non-empty when `success == false`).
    pub error: String,
    /// Estimated size in bytes of the export output.
    pub size_bytes: usize,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// Convenience alias for a list of batch results.
pub type BatchResults = Vec<BatchExportResult>;

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a default [`BatchExportConfig`] with no targets.
#[allow(dead_code)]
pub fn default_batch_config() -> BatchExportConfig {
    BatchExportConfig {
        targets: Vec::new(),
        output_directory: "./export".to_string(),
        overwrite_existing: true,
        job_label: "batch".to_string(),
    }
}

/// Create a new [`BatchExportConfig`] with a specific output directory.
#[allow(dead_code)]
pub fn new_batch_config(output_dir: &str, job_label: &str) -> BatchExportConfig {
    BatchExportConfig {
        targets: Vec::new(),
        output_directory: output_dir.to_string(),
        overwrite_existing: true,
        job_label: job_label.to_string(),
    }
}

/// Append a [`BatchExportTarget`] to `config` and return it.
#[allow(dead_code)]
pub fn add_target(mut config: BatchExportConfig, target: BatchExportTarget) -> BatchExportConfig {
    config.targets.push(target);
    config
}

/// Remove the target with the given `label` from `config` and return the updated config.
/// If no matching label is found, `config` is returned unchanged.
#[allow(dead_code)]
pub fn remove_target(mut config: BatchExportConfig, label: &str) -> BatchExportConfig {
    config.targets.retain(|t| t.label != label);
    config
}

/// Return the number of targets in `config`.
#[allow(dead_code)]
pub fn target_count(config: &BatchExportConfig) -> usize {
    config.targets.len()
}

/// Execute all enabled targets in `config` and return a [`BatchResults`] list.
///
/// This is a stub implementation: it simulates success for all enabled targets
/// and generates a synthetic size estimate.
#[allow(dead_code)]
pub fn execute_batch(config: &BatchExportConfig) -> BatchResults {
    config
        .targets
        .iter()
        .filter(|t| t.enabled)
        .map(|t| {
            let size_bytes = 1024 + t.filename.len() * 16;
            BatchExportResult {
                label: t.label.clone(),
                format: t.format,
                success: true,
                error: String::new(),
                size_bytes,
            }
        })
        .collect()
}

/// Serialise `config` as a compact JSON string.
#[allow(dead_code)]
pub fn batch_to_json(config: &BatchExportConfig) -> String {
    let targets: Vec<String> = config
        .targets
        .iter()
        .map(|t| {
            format!(
                "{{\"label\":\"{}\",\"format\":\"{}\",\"filename\":\"{}\",\"enabled\":{}}}",
                t.label,
                t.format.name(),
                t.filename,
                t.enabled
            )
        })
        .collect();
    format!(
        "{{\"job_label\":\"{}\",\"output_directory\":\"{}\",\"overwrite_existing\":{},\"targets\":[{}]}}",
        config.job_label,
        config.output_directory,
        config.overwrite_existing,
        targets.join(",")
    )
}

/// Return only the successful results from `results`.
#[allow(dead_code)]
pub fn successful_exports(results: &[BatchExportResult]) -> Vec<&BatchExportResult> {
    results.iter().filter(|r| r.success).collect()
}

/// Return only the failed results from `results`.
#[allow(dead_code)]
pub fn failed_exports(results: &[BatchExportResult]) -> Vec<&BatchExportResult> {
    results.iter().filter(|r| !r.success).collect()
}

/// Compute the total estimated size in bytes across all results.
#[allow(dead_code)]
pub fn batch_export_size_total(results: &[BatchExportResult]) -> usize {
    results.iter().map(|r| r.size_bytes).sum()
}

/// Set the output directory on `config` and return the updated config.
#[allow(dead_code)]
pub fn set_output_directory(mut config: BatchExportConfig, dir: &str) -> BatchExportConfig {
    config.output_directory = dir.to_string();
    config
}

/// Return a list of format name strings for all targets in `config`.
#[allow(dead_code)]
pub fn batch_format_names(config: &BatchExportConfig) -> Vec<&'static str> {
    config.targets.iter().map(|t| t.format.name()).collect()
}

/// Remove all targets from `config` and return the cleared config.
#[allow(dead_code)]
pub fn clear_targets(mut config: BatchExportConfig) -> BatchExportConfig {
    config.targets.clear();
    config
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_target(label: &str, fmt: BatchExportFormat) -> BatchExportTarget {
        BatchExportTarget {
            label: label.to_string(),
            format: fmt,
            filename: format!("{}.{}", label, fmt.extension()),
            enabled: true,
        }
    }

    fn filled_config() -> BatchExportConfig {
        let cfg = new_batch_config("./out", "test_job");
        let cfg = add_target(cfg, make_target("glb_out", BatchExportFormat::Glb));
        let cfg = add_target(cfg, make_target("obj_out", BatchExportFormat::Obj));
        add_target(cfg, make_target("dae_out", BatchExportFormat::Collada))
    }

    #[test]
    fn default_config_empty_targets() {
        let cfg = default_batch_config();
        assert_eq!(target_count(&cfg), 0);
    }

    #[test]
    fn new_batch_config_sets_dir_and_label() {
        let cfg = new_batch_config("/tmp/export", "myjob");
        assert_eq!(cfg.output_directory, "/tmp/export");
        assert_eq!(cfg.job_label, "myjob");
    }

    #[test]
    fn add_target_increments_count() {
        let cfg = default_batch_config();
        let cfg = add_target(cfg, make_target("a", BatchExportFormat::Json));
        assert_eq!(target_count(&cfg), 1);
    }

    #[test]
    fn remove_target_by_label() {
        let cfg = filled_config();
        assert_eq!(target_count(&cfg), 3);
        let cfg = remove_target(cfg, "obj_out");
        assert_eq!(target_count(&cfg), 2);
        assert!(!cfg.targets.iter().any(|t| t.label == "obj_out"));
    }

    #[test]
    fn remove_target_nonexistent_unchanged() {
        let cfg = filled_config();
        let before = target_count(&cfg);
        let cfg = remove_target(cfg, "does_not_exist");
        assert_eq!(target_count(&cfg), before);
    }

    #[test]
    fn execute_batch_returns_one_result_per_enabled_target() {
        let cfg = filled_config();
        let results = execute_batch(&cfg);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn execute_batch_skips_disabled_targets() {
        let mut cfg = filled_config();
        cfg.targets[1].enabled = false;
        let results = execute_batch(&cfg);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn execute_batch_all_succeed() {
        let cfg = filled_config();
        let results = execute_batch(&cfg);
        assert!(results.iter().all(|r| r.success));
    }

    #[test]
    fn batch_to_json_contains_job_label() {
        let cfg = new_batch_config("/out", "myjob");
        let json = batch_to_json(&cfg);
        assert!(json.contains("myjob"));
    }

    #[test]
    fn successful_exports_filters_correctly() {
        let mut results = execute_batch(&filled_config());
        results[0].success = false;
        results[0].error = "write failed".to_string();
        let successes = successful_exports(&results);
        assert_eq!(successes.len(), 2);
    }

    #[test]
    fn failed_exports_filters_correctly() {
        let mut results = execute_batch(&filled_config());
        results[2].success = false;
        let fails = failed_exports(&results);
        assert_eq!(fails.len(), 1);
    }

    #[test]
    fn batch_export_size_total_sums() {
        let cfg = filled_config();
        let results = execute_batch(&cfg);
        let total = batch_export_size_total(&results);
        let expected: usize = results.iter().map(|r| r.size_bytes).sum();
        assert_eq!(total, expected);
    }

    #[test]
    fn set_output_directory_updates_dir() {
        let cfg = default_batch_config();
        let cfg = set_output_directory(cfg, "/new/dir");
        assert_eq!(cfg.output_directory, "/new/dir");
    }

    #[test]
    fn batch_format_names_returns_names() {
        let cfg = filled_config();
        let names = batch_format_names(&cfg);
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"Binary glTF"));
    }

    #[test]
    fn clear_targets_removes_all() {
        let cfg = filled_config();
        let cfg = clear_targets(cfg);
        assert_eq!(target_count(&cfg), 0);
    }

    #[test]
    fn format_extension_correct() {
        assert_eq!(BatchExportFormat::Glb.extension(), "glb");
        assert_eq!(BatchExportFormat::Obj.extension(), "obj");
        assert_eq!(BatchExportFormat::Collada.extension(), "dae");
        assert_eq!(BatchExportFormat::Abc.extension(), "abc");
        assert_eq!(BatchExportFormat::Json.extension(), "json");
    }

    #[test]
    fn format_name_correct() {
        assert_eq!(BatchExportFormat::Collada.name(), "COLLADA");
    }
}
