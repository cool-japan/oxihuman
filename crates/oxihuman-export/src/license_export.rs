// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! SPDX license expression export helper.

/// A single license entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LicenseEntry {
    pub spdx_id: String,
    pub name: String,
    pub url: String,
    pub osi_approved: bool,
}

/// A collection of license entries for a project.
#[allow(dead_code)]
pub struct LicenseManifest {
    pub project: String,
    pub entries: Vec<LicenseEntry>,
}

impl LicenseManifest {
    #[allow(dead_code)]
    pub fn new(project: &str) -> Self {
        Self {
            project: project.to_string(),
            entries: Vec::new(),
        }
    }
}

/// Add a license entry.
#[allow(dead_code)]
pub fn add_license(
    manifest: &mut LicenseManifest,
    spdx_id: &str,
    name: &str,
    url: &str,
    osi: bool,
) {
    manifest.entries.push(LicenseEntry {
        spdx_id: spdx_id.to_string(),
        name: name.to_string(),
        url: url.to_string(),
        osi_approved: osi,
    });
}

/// Export as SPDX expression string (OR-joined).
#[allow(dead_code)]
pub fn export_spdx_expression(manifest: &LicenseManifest) -> String {
    manifest
        .entries
        .iter()
        .map(|e| e.spdx_id.as_str())
        .collect::<Vec<_>>()
        .join(" OR ")
}

/// Export as plain-text license list.
#[allow(dead_code)]
pub fn export_license_txt(manifest: &LicenseManifest) -> String {
    let mut out = format!("Project: {}\n\nLicenses:\n", manifest.project);
    for e in &manifest.entries {
        out.push_str(&format!(
            "  {} ({})\n  URL: {}\n  OSI Approved: {}\n\n",
            e.spdx_id, e.name, e.url, e.osi_approved
        ));
    }
    out
}

/// Export as JSON-like string.
#[allow(dead_code)]
pub fn export_license_json(manifest: &LicenseManifest) -> String {
    let mut out = format!("{{\"project\":\"{}\",\"licenses\":[", manifest.project);
    for (i, e) in manifest.entries.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"spdx_id\":\"{}\",\"name\":\"{}\",\"url\":\"{}\",\"osi_approved\":{}}}",
            e.spdx_id, e.name, e.url, e.osi_approved
        ));
    }
    out.push_str("]}");
    out
}

/// Count of OSI-approved licenses.
#[allow(dead_code)]
pub fn osi_approved_count(manifest: &LicenseManifest) -> usize {
    manifest.entries.iter().filter(|e| e.osi_approved).count()
}

/// Total license count.
#[allow(dead_code)]
pub fn license_count(manifest: &LicenseManifest) -> usize {
    manifest.entries.len()
}

/// Find a license by SPDX ID.
#[allow(dead_code)]
pub fn find_by_spdx_id<'a>(
    manifest: &'a LicenseManifest,
    spdx_id: &str,
) -> Option<&'a LicenseEntry> {
    manifest.entries.iter().find(|e| e.spdx_id == spdx_id)
}

/// Check if manifest contains a given SPDX ID.
#[allow(dead_code)]
pub fn has_license(manifest: &LicenseManifest, spdx_id: &str) -> bool {
    manifest.entries.iter().any(|e| e.spdx_id == spdx_id)
}

/// Remove a license by SPDX ID. Returns true if removed.
#[allow(dead_code)]
pub fn remove_license(manifest: &mut LicenseManifest, spdx_id: &str) -> bool {
    let before = manifest.entries.len();
    manifest.entries.retain(|e| e.spdx_id != spdx_id);
    manifest.entries.len() < before
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> LicenseManifest {
        let mut m = LicenseManifest::new("oxihuman");
        add_license(
            &mut m,
            "MIT",
            "MIT License",
            "https://opensource.org/licenses/MIT",
            true,
        );
        add_license(
            &mut m,
            "Apache-2.0",
            "Apache License 2.0",
            "https://www.apache.org/licenses/LICENSE-2.0",
            true,
        );
        add_license(
            &mut m,
            "GPL-3.0-only",
            "GNU General Public License v3.0 only",
            "https://www.gnu.org/licenses/gpl-3.0.html",
            true,
        );
        m
    }

    #[test]
    fn license_count_correct() {
        let m = sample_manifest();
        assert_eq!(license_count(&m), 3);
    }

    #[test]
    fn osi_approved_count_correct() {
        let m = sample_manifest();
        assert_eq!(osi_approved_count(&m), 3);
    }

    #[test]
    fn spdx_expression_contains_or() {
        let m = sample_manifest();
        let expr = export_spdx_expression(&m);
        assert!(expr.contains(" OR "));
    }

    #[test]
    fn spdx_expression_contains_mit() {
        let m = sample_manifest();
        let expr = export_spdx_expression(&m);
        assert!(expr.contains("MIT"));
    }

    #[test]
    fn has_license_true() {
        let m = sample_manifest();
        assert!(has_license(&m, "MIT"));
    }

    #[test]
    fn has_license_false() {
        let m = sample_manifest();
        assert!(!has_license(&m, "BSD-3-Clause"));
    }

    #[test]
    fn find_by_spdx_id_some() {
        let m = sample_manifest();
        let e = find_by_spdx_id(&m, "Apache-2.0");
        assert!(e.is_some());
    }

    #[test]
    fn find_by_spdx_id_none() {
        let m = sample_manifest();
        assert!(find_by_spdx_id(&m, "LGPL-2.1").is_none());
    }

    #[test]
    fn remove_license_success() {
        let mut m = sample_manifest();
        let removed = remove_license(&mut m, "GPL-3.0-only");
        assert!(removed);
        assert_eq!(license_count(&m), 2);
    }

    #[test]
    fn txt_contains_project_name() {
        let m = sample_manifest();
        let txt = export_license_txt(&m);
        assert!(txt.contains("oxihuman"));
    }
}
