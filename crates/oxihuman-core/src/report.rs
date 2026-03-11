// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::{Deserialize, Serialize};

/// Severity of a report event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl Severity {
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Info => "INFO",
            Severity::Warning => "WARN",
            Severity::Error => "ERROR",
        }
    }
}

/// A single report event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportEvent {
    pub severity: Severity,
    /// e.g. "parser", "policy", "morph", "export"
    pub category: String,
    pub message: String,
    pub detail: Option<String>,
}

impl ReportEvent {
    pub fn info(category: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Info,
            category: category.into(),
            message: message.into(),
            detail: None,
        }
    }

    pub fn warning(category: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            category: category.into(),
            message: message.into(),
            detail: None,
        }
    }

    pub fn error(category: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            category: category.into(),
            message: message.into(),
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Complete pipeline audit report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineReport {
    pub events: Vec<ReportEvent>,
    pub targets_loaded: usize,
    pub targets_blocked: usize,
    pub targets_failed: usize,
    pub base_mesh_verts: usize,
    pub base_mesh_faces: usize,
    pub export_paths: Vec<String>,
    /// Timestamp string (ISO 8601 UTC, computed at build time).
    pub generated_at: String,
}

impl PipelineReport {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            targets_loaded: 0,
            targets_blocked: 0,
            targets_failed: 0,
            base_mesh_verts: 0,
            base_mesh_faces: 0,
            export_paths: Vec::new(),
            generated_at: current_timestamp(),
        }
    }

    pub fn add_event(&mut self, event: ReportEvent) {
        self.events.push(event);
    }

    pub fn info(&mut self, category: &str, msg: &str) {
        self.add_event(ReportEvent::info(category, msg));
    }

    pub fn warning(&mut self, category: &str, msg: &str) {
        self.add_event(ReportEvent::warning(category, msg));
    }

    pub fn error(&mut self, category: &str, msg: &str) {
        self.add_event(ReportEvent::error(category, msg));
    }

    /// Count events by severity.
    pub fn count_severity(&self, sev: Severity) -> usize {
        self.events.iter().filter(|e| e.severity == sev).count()
    }

    /// True if there are no Error-severity events.
    pub fn is_healthy(&self) -> bool {
        self.count_severity(Severity::Error) == 0
    }

    /// True if there are warnings.
    pub fn has_warnings(&self) -> bool {
        self.count_severity(Severity::Warning) > 0
    }

    /// Render as a human-readable text report.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "OxiHuman Pipeline Report — {}\n",
            self.generated_at
        ));
        out.push_str(&format!(
            "  Targets: {} loaded, {} blocked, {} failed\n",
            self.targets_loaded, self.targets_blocked, self.targets_failed
        ));
        out.push_str(&format!(
            "  Base mesh: {} verts, {} faces\n",
            self.base_mesh_verts, self.base_mesh_faces
        ));
        if !self.export_paths.is_empty() {
            out.push_str(&format!("  Exports: {}\n", self.export_paths.join(", ")));
        }
        out.push_str(&format!(
            "  Health: {} | Warnings: {} | Errors: {}\n",
            if self.is_healthy() { "OK" } else { "FAIL" },
            self.count_severity(Severity::Warning),
            self.count_severity(Severity::Error)
        ));
        for e in &self.events {
            out.push_str(&format!(
                "  [{}] {}: {}\n",
                e.severity.label(),
                e.category,
                e.message
            ));
        }
        out
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }

    /// Save as JSON to a file.
    pub fn save_json(&self, path: &std::path::Path) -> anyhow::Result<()> {
        std::fs::write(path, serde_json::to_string_pretty(&self.to_json())?).map_err(Into::into)
    }
}

impl Default for PipelineReport {
    fn default() -> Self {
        Self::new()
    }
}

/// A builder for constructing a report incrementally during pipeline execution.
pub struct ReportBuilder {
    report: PipelineReport,
}

impl ReportBuilder {
    pub fn new() -> Self {
        Self {
            report: PipelineReport::new(),
        }
    }

    pub fn target_loaded(mut self, name: &str) -> Self {
        self.report.targets_loaded += 1;
        self.report.info("morph", &format!("loaded target: {name}"));
        self
    }

    pub fn target_blocked(mut self, name: &str, reason: &str) -> Self {
        self.report.targets_blocked += 1;
        self.report
            .warning("policy", &format!("blocked: {name} \u{2014} {reason}"));
        self
    }

    pub fn target_failed(mut self, name: &str, err: &str) -> Self {
        self.report.targets_failed += 1;
        self.report
            .error("parser", &format!("failed: {name} \u{2014} {err}"));
        self
    }

    pub fn base_mesh(mut self, verts: usize, faces: usize) -> Self {
        self.report.base_mesh_verts = verts;
        self.report.base_mesh_faces = faces;
        self.report
            .info("mesh", &format!("base mesh: {verts} verts, {faces} faces"));
        self
    }

    pub fn export(mut self, path: &str) -> Self {
        self.report.export_paths.push(path.to_string());
        self.report.info("export", &format!("exported: {path}"));
        self
    }

    pub fn build(self) -> PipelineReport {
        self.report
    }
}

impl Default for ReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, mo, d, h, mi, s) = unix_to_datetime(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

fn unix_to_datetime(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let sec = (secs % 60) as u32;
    let min = ((secs / 60) % 60) as u32;
    let hour = ((secs / 3600) % 24) as u32;
    let mut days = secs / 86400;
    let mut year = 1970u64;
    loop {
        let dy: u64 =
            if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) {
                366
            } else {
                365
            };
        if days < dy {
            break;
        }
        days -= dy;
        year += 1;
    }
    let is_leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    let months: [u64; 12] = [
        31,
        if is_leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u64;
    for &ml in &months {
        if days < ml {
            break;
        }
        days -= ml;
        month += 1;
    }
    (year as u32, month as u32, (days + 1) as u32, hour, min, sec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_report_is_healthy() {
        assert!(PipelineReport::new().is_healthy());
    }

    #[test]
    fn add_error_makes_unhealthy() {
        let mut r = PipelineReport::new();
        r.error("test", "something broke");
        assert!(!r.is_healthy());
    }

    #[test]
    fn count_severity_correct() {
        let mut r = PipelineReport::new();
        r.warning("a", "w1");
        r.warning("b", "w2");
        r.info("c", "i1");
        assert_eq!(r.count_severity(Severity::Warning), 2);
        assert_eq!(r.count_severity(Severity::Info), 1);
        assert_eq!(r.count_severity(Severity::Error), 0);
    }

    #[test]
    fn builder_target_loaded_increments() {
        let report = ReportBuilder::new().target_loaded("x").build();
        assert_eq!(report.targets_loaded, 1);
    }

    #[test]
    fn builder_target_blocked() {
        let report = ReportBuilder::new().target_blocked("y", "nsfw").build();
        assert_eq!(report.targets_blocked, 1);
        assert!(report.has_warnings());
    }

    #[test]
    fn builder_build_is_healthy_after_loaded() {
        let report = ReportBuilder::new().target_loaded("z").build();
        assert!(report.is_healthy());
    }

    #[test]
    fn to_text_contains_report() {
        let report = PipelineReport::new();
        assert!(report.to_text().contains("Pipeline Report"));
    }

    #[test]
    fn to_json_has_targets_loaded() {
        let report = PipelineReport::new();
        let json = report.to_json();
        assert!(json["targets_loaded"].as_u64().is_some());
    }

    #[test]
    fn save_json_creates_file() {
        let report = PipelineReport::new();
        let path = std::path::Path::new("/tmp/oxihuman_report_test.json");
        report.save_json(path).expect("save_json failed");
        assert!(path.exists());
    }

    #[test]
    fn timestamp_nonempty() {
        let report = PipelineReport::new();
        assert!(!report.generated_at.is_empty());
    }
}
