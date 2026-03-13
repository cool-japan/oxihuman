// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! RCP (ReCap project) format stub export.

pub const RCP_MAGIC: &[u8; 4] = b"RCP\0";

/// RCP scan reference.
#[allow(dead_code)]
pub struct RcpScanRef {
    pub scan_id: String,
    pub point_count: u64,
}

/// RCP project export stub.
#[allow(dead_code)]
pub struct RcpExport {
    pub project_name: String,
    pub scans: Vec<RcpScanRef>,
    pub total_points: u64,
}

/// Create a new RCP export.
#[allow(dead_code)]
pub fn new_rcp_export(project_name: &str) -> RcpExport {
    RcpExport {
        project_name: project_name.to_string(),
        scans: Vec::new(),
        total_points: 0,
    }
}

/// Add a scan reference.
#[allow(dead_code)]
pub fn add_rcp_scan(export: &mut RcpExport, scan_id: &str, point_count: u64) {
    export.total_points += point_count;
    export.scans.push(RcpScanRef { scan_id: scan_id.to_string(), point_count });
}

/// Scan count.
#[allow(dead_code)]
pub fn rcp_scan_count(export: &RcpExport) -> usize {
    export.scans.len()
}

/// Total point count.
#[allow(dead_code)]
pub fn rcp_total_points(export: &RcpExport) -> u64 {
    export.total_points
}

/// Build RCP XML manifest stub.
#[allow(dead_code)]
pub fn build_rcp_manifest(export: &RcpExport) -> String {
    let scans: String = export.scans.iter().map(|s| {
        format!("<scan id=\"{}\" points=\"{}\"/>", s.scan_id, s.point_count)
    }).collect::<Vec<_>>().join("\n");
    format!("<rcp project=\"{}\">\n{}\n</rcp>", export.project_name, scans)
}

/// Validate RCP export.
#[allow(dead_code)]
pub fn validate_rcp(export: &RcpExport) -> bool {
    !export.project_name.is_empty() && !export.scans.is_empty()
}

/// Export stub bytes.
#[allow(dead_code)]
pub fn export_rcp_stub(export: &RcpExport) -> Vec<u8> {
    let mut buf = RCP_MAGIC.to_vec();
    buf.extend_from_slice(&(export.scans.len() as u32).to_le_bytes());
    buf.extend_from_slice(&export.total_points.to_le_bytes());
    buf
}

/// Average points per scan.
#[allow(dead_code)]
pub fn rcp_avg_points_per_scan(export: &RcpExport) -> f64 {
    if export.scans.is_empty() { return 0.0; }
    export.total_points as f64 / export.scans.len() as f64
}

/// Find scan by ID.
#[allow(dead_code)]
pub fn find_rcp_scan<'a>(export: &'a RcpExport, scan_id: &str) -> Option<&'a RcpScanRef> {
    export.scans.iter().find(|s| s.scan_id == scan_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_correct() {
        assert_eq!(&RCP_MAGIC[..3], b"RCP");
    }

    #[test]
    fn new_empty() {
        let e = new_rcp_export("MyProject");
        assert_eq!(rcp_scan_count(&e), 0);
    }

    #[test]
    fn add_scan() {
        let mut e = new_rcp_export("test");
        add_rcp_scan(&mut e, "scan001", 1000);
        assert_eq!(rcp_scan_count(&e), 1);
        assert_eq!(rcp_total_points(&e), 1000);
    }

    #[test]
    fn manifest_contains_project() {
        let e = new_rcp_export("MyProject");
        let m = build_rcp_manifest(&e);
        assert!(m.contains("MyProject"));
    }

    #[test]
    fn validate_fails_empty() {
        let e = new_rcp_export("test");
        assert!(!validate_rcp(&e));
    }

    #[test]
    fn validate_passes() {
        let mut e = new_rcp_export("test");
        add_rcp_scan(&mut e, "s1", 100);
        assert!(validate_rcp(&e));
    }

    #[test]
    fn export_bytes_start_with_magic() {
        let e = new_rcp_export("test");
        let b = export_rcp_stub(&e);
        assert_eq!(&b[..3], b"RCP");
    }

    #[test]
    fn avg_points_per_scan() {
        let mut e = new_rcp_export("test");
        add_rcp_scan(&mut e, "s1", 100);
        add_rcp_scan(&mut e, "s2", 200);
        let avg = rcp_avg_points_per_scan(&e);
        assert!((avg - 150.0).abs() < 1e-5);
    }

    #[test]
    fn find_scan() {
        let mut e = new_rcp_export("test");
        add_rcp_scan(&mut e, "abc", 50);
        let found = find_rcp_scan(&e, "abc");
        assert!(found.is_some());
        assert_eq!(found.expect("should succeed").point_count, 50);
    }
}
