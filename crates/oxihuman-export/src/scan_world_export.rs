// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ScanWorld project file stub export (Leica Cyclone format).

/// A scan registration entry.
#[allow(dead_code)]
pub struct ScanRegistration {
    pub scan_id: String,
    pub transform: [f32; 16],
    pub rms_error: f32,
}

/// ScanWorld project export stub.
#[allow(dead_code)]
pub struct ScanWorldExport {
    pub project_name: String,
    pub registrations: Vec<ScanRegistration>,
}

/// Create a new ScanWorld export.
#[allow(dead_code)]
pub fn new_scan_world_export(name: &str) -> ScanWorldExport {
    ScanWorldExport { project_name: name.to_string(), registrations: Vec::new() }
}

/// Identity transform (4x4 column-major).
#[allow(dead_code)]
pub fn identity_transform() -> [f32; 16] {
    [1.,0.,0.,0., 0.,1.,0.,0., 0.,0.,1.,0., 0.,0.,0.,1.]
}

/// Add a scan registration.
#[allow(dead_code)]
pub fn add_registration(export: &mut ScanWorldExport, scan_id: &str, transform: [f32; 16], rms: f32) {
    export.registrations.push(ScanRegistration { scan_id: scan_id.to_string(), transform, rms_error: rms });
}

/// Count.
#[allow(dead_code)]
pub fn registration_count(export: &ScanWorldExport) -> usize {
    export.registrations.len()
}

/// Average RMS error.
#[allow(dead_code)]
pub fn avg_rms_error(export: &ScanWorldExport) -> f32 {
    if export.registrations.is_empty() { return 0.0; }
    export.registrations.iter().map(|r| r.rms_error).sum::<f32>() / export.registrations.len() as f32
}

/// Find registration by scan ID.
#[allow(dead_code)]
pub fn find_registration<'a>(export: &'a ScanWorldExport, id: &str) -> Option<&'a ScanRegistration> {
    export.registrations.iter().find(|r| r.scan_id == id)
}

/// Validate ScanWorld export.
#[allow(dead_code)]
pub fn validate_scan_world(export: &ScanWorldExport) -> bool {
    !export.project_name.is_empty() && !export.registrations.is_empty()
}

/// Build XML project stub.
#[allow(dead_code)]
pub fn build_scan_world_xml(export: &ScanWorldExport) -> String {
    let regs: String = export.registrations.iter().map(|r| {
        format!("<scan id=\"{}\" rms=\"{:.6}\"/>", r.scan_id, r.rms_error)
    }).collect::<Vec<_>>().join("\n");
    format!("<ScanWorld project=\"{}\">\n{}\n</ScanWorld>", export.project_name, regs)
}

/// Max RMS error.
#[allow(dead_code)]
pub fn max_rms_error(export: &ScanWorldExport) -> f32 {
    export.registrations.iter().map(|r| r.rms_error).fold(0.0f32, f32::max)
}

/// Estimate export size.
#[allow(dead_code)]
pub fn scan_world_size_estimate(reg_count: usize) -> usize {
    512 + reg_count * 256
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let e = new_scan_world_export("test");
        assert_eq!(registration_count(&e), 0);
    }

    #[test]
    fn add_registration_count() {
        let mut e = new_scan_world_export("test");
        add_registration(&mut e, "scan001", identity_transform(), 0.002);
        assert_eq!(registration_count(&e), 1);
    }

    #[test]
    fn identity_diagonal() {
        let id = identity_transform();
        assert!((id[0] - 1.0).abs() < 1e-6);
        assert!((id[5] - 1.0).abs() < 1e-6);
        assert!((id[10] - 1.0).abs() < 1e-6);
        assert!((id[15] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn avg_rms_correct() {
        let mut e = new_scan_world_export("test");
        add_registration(&mut e, "s1", identity_transform(), 0.002);
        add_registration(&mut e, "s2", identity_transform(), 0.004);
        let avg = avg_rms_error(&e);
        assert!((avg - 0.003).abs() < 1e-5);
    }

    #[test]
    fn find_registration_by_name() {
        let mut e = new_scan_world_export("test");
        add_registration(&mut e, "abc", identity_transform(), 0.001);
        let r = find_registration(&e, "abc");
        assert!(r.is_some());
    }

    #[test]
    fn validate_fails_empty() {
        let e = new_scan_world_export("test");
        assert!(!validate_scan_world(&e));
    }

    #[test]
    fn validate_passes() {
        let mut e = new_scan_world_export("test");
        add_registration(&mut e, "s1", identity_transform(), 0.0);
        assert!(validate_scan_world(&e));
    }

    #[test]
    fn xml_contains_project() {
        let mut e = new_scan_world_export("MyProject");
        add_registration(&mut e, "s1", identity_transform(), 0.0);
        let xml = build_scan_world_xml(&e);
        assert!(xml.contains("MyProject"));
    }

    #[test]
    fn max_rms_correct() {
        let mut e = new_scan_world_export("test");
        add_registration(&mut e, "s1", identity_transform(), 0.001);
        add_registration(&mut e, "s2", identity_transform(), 0.01);
        let mx = max_rms_error(&e);
        assert!((mx - 0.01).abs() < 1e-6);
    }
}
