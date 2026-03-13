// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export profiler timing data (span names, durations) as JSON or CSV.

#![allow(dead_code)]

/// Configuration for profile-data export.
#[derive(Debug, Clone)]
pub struct ProfileDataExportConfig {
    /// Pretty-print JSON output.
    pub pretty: bool,
    /// Include zero-duration spans in the output.
    pub include_zero_duration: bool,
    /// CSV field delimiter character.
    pub csv_delimiter: char,
}

/// A single profiler span record.
#[derive(Debug, Clone)]
pub struct ProfileSpanRecord {
    /// Name of the profiler span.
    pub name: String,
    /// Duration of the span in milliseconds.
    pub duration_ms: f64,
    /// Optional category / thread label.
    pub category: String,
    /// Nesting depth (0 = top level).
    pub depth: u32,
}

/// Result container produced by a profile-data export session.
#[derive(Debug, Clone)]
pub struct ProfileDataExportResult {
    /// All recorded spans.
    pub spans: Vec<ProfileSpanRecord>,
    /// Total byte size of the last serialised output.
    pub total_bytes: usize,
}

/// Returns the default [`ProfileDataExportConfig`].
pub fn default_profile_data_config() -> ProfileDataExportConfig {
    ProfileDataExportConfig {
        pretty: true,
        include_zero_duration: false,
        csv_delimiter: ',',
    }
}

/// Creates a new, empty [`ProfileDataExportResult`].
pub fn new_profile_data_export() -> ProfileDataExportResult {
    ProfileDataExportResult {
        spans: Vec::new(),
        total_bytes: 0,
    }
}

/// Appends a span record to the export result.
pub fn pde_add_span(result: &mut ProfileDataExportResult, span: ProfileSpanRecord) {
    result.spans.push(span);
}

/// Serialises the span list as JSON.
pub fn pde_export_json(
    result: &mut ProfileDataExportResult,
    cfg: &ProfileDataExportConfig,
) -> String {
    let spans: Vec<&ProfileSpanRecord> = result
        .spans
        .iter()
        .filter(|s| cfg.include_zero_duration || s.duration_ms > 0.0)
        .collect();

    let sep = if cfg.pretty { "\n  " } else { "" };
    let indent = if cfg.pretty { "  " } else { "" };
    let mut out = String::from("{\"spans\":[");
    if cfg.pretty {
        out.push('\n');
    }
    for (i, s) in spans.iter().enumerate() {
        let comma = if i + 1 < spans.len() { "," } else { "" };
        out.push_str(indent);
        out.push_str(&format!(
            "{{\"name\":\"{}\",\"duration_ms\":{:.4},\"category\":\"{}\",\"depth\":{}}}{}{}",
            s.name, s.duration_ms, s.category, s.depth, comma, sep
        ));
    }
    out.push_str("]}");
    result.total_bytes = out.len();
    out
}

/// Serialises the span list as CSV.
pub fn pde_export_csv(
    result: &mut ProfileDataExportResult,
    cfg: &ProfileDataExportConfig,
) -> String {
    let d = cfg.csv_delimiter;
    let mut out = format!("name{d}duration_ms{d}category{d}depth\n");
    for s in &result.spans {
        if !cfg.include_zero_duration && s.duration_ms <= 0.0 {
            continue;
        }
        out.push_str(&format!(
            "{}{d}{:.4}{d}{}{d}{}\n",
            s.name, s.duration_ms, s.category, s.depth
        ));
    }
    result.total_bytes = out.len();
    out
}

/// Returns the number of spans currently recorded.
pub fn pde_span_count(result: &ProfileDataExportResult) -> usize {
    result.spans.len()
}

/// Returns the sum of all span durations in milliseconds.
pub fn pde_total_duration_ms(result: &ProfileDataExportResult) -> f64 {
    result.spans.iter().map(|s| s.duration_ms).sum()
}

/// Writes JSON output to a file path (stub — returns byte count).
pub fn pde_write_to_file(
    result: &mut ProfileDataExportResult,
    cfg: &ProfileDataExportConfig,
    _path: &str,
) -> usize {
    let json = pde_export_json(result, cfg);
    result.total_bytes = json.len();
    result.total_bytes
}

/// Clears all recorded spans and resets state.
pub fn pde_clear(result: &mut ProfileDataExportResult) {
    result.spans.clear();
    result.total_bytes = 0;
}

/// Returns the byte count of the last serialised output.
pub fn pde_total_bytes(result: &ProfileDataExportResult) -> usize {
    result.total_bytes
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_span(name: &str, duration_ms: f64, category: &str, depth: u32) -> ProfileSpanRecord {
    ProfileSpanRecord {
        name: name.to_string(),
        duration_ms,
        category: category.to_string(),
        depth,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_profile_data_config();
        assert!(cfg.pretty);
        assert!(!cfg.include_zero_duration);
        assert_eq!(cfg.csv_delimiter, ',');
    }

    #[test]
    fn new_export_is_empty() {
        let r = new_profile_data_export();
        assert_eq!(pde_span_count(&r), 0);
        assert_eq!(pde_total_bytes(&r), 0);
    }

    #[test]
    fn add_span_increments_count() {
        let mut r = new_profile_data_export();
        pde_add_span(&mut r, make_span("render", 12.5, "gpu", 0));
        assert_eq!(pde_span_count(&r), 1);
    }

    #[test]
    fn total_duration_sums_all_spans() {
        let mut r = new_profile_data_export();
        pde_add_span(&mut r, make_span("a", 10.0, "cpu", 0));
        pde_add_span(&mut r, make_span("b", 5.5, "cpu", 1));
        let total = pde_total_duration_ms(&r);
        assert!((total - 15.5).abs() < 1e-9);
    }

    #[test]
    fn json_export_contains_name() {
        let mut r = new_profile_data_export();
        pde_add_span(&mut r, make_span("physics_step", 3.0, "cpu", 0));
        let cfg = default_profile_data_config();
        let json = pde_export_json(&mut r, &cfg);
        assert!(json.contains("physics_step"));
        assert!(json.contains("spans"));
    }

    #[test]
    fn csv_export_contains_header() {
        let mut r = new_profile_data_export();
        pde_add_span(&mut r, make_span("morph", 1.0, "cpu", 0));
        let cfg = default_profile_data_config();
        let csv = pde_export_csv(&mut r, &cfg);
        assert!(csv.contains("name,duration_ms"));
        assert!(csv.contains("morph"));
    }

    #[test]
    fn zero_duration_excluded_by_default() {
        let mut r = new_profile_data_export();
        pde_add_span(&mut r, make_span("empty", 0.0, "cpu", 0));
        pde_add_span(&mut r, make_span("active", 2.0, "cpu", 0));
        let cfg = default_profile_data_config();
        let json = pde_export_json(&mut r, &cfg);
        assert!(!json.contains("\"empty\""));
        assert!(json.contains("\"active\""));
    }

    #[test]
    fn write_to_file_sets_total_bytes() {
        let mut r = new_profile_data_export();
        pde_add_span(&mut r, make_span("tick", 4.0, "main", 0));
        let cfg = default_profile_data_config();
        let n = pde_write_to_file(&mut r, &cfg, "/tmp/profile.json");
        assert!(n > 0);
        assert_eq!(pde_total_bytes(&r), n);
    }

    #[test]
    fn clear_resets_state() {
        let mut r = new_profile_data_export();
        pde_add_span(&mut r, make_span("tick", 1.0, "main", 0));
        let cfg = default_profile_data_config();
        pde_write_to_file(&mut r, &cfg, "/tmp/profile.json");
        pde_clear(&mut r);
        assert_eq!(pde_span_count(&r), 0);
        assert_eq!(pde_total_bytes(&r), 0);
    }

    #[test]
    fn csv_custom_delimiter() {
        let mut r = new_profile_data_export();
        pde_add_span(&mut r, make_span("draw", 7.0, "gpu", 0));
        let cfg = ProfileDataExportConfig {
            pretty: false,
            include_zero_duration: false,
            csv_delimiter: ';',
        };
        let csv = pde_export_csv(&mut r, &cfg);
        assert!(csv.contains("name;duration_ms"));
        assert!(csv.contains("draw"));
    }
}
