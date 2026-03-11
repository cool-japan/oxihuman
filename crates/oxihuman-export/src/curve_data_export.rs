// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Curve data export (Bezier, NURBS control points).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurveDataExport {
    pub curves: Vec<ExportCurve>,
}

/// A single curve.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExportCurve {
    pub name: String,
    pub control_points: Vec<[f32; 3]>,
    pub degree: u32,
    pub is_closed: bool,
}

#[allow(dead_code)]
impl CurveDataExport {
    pub fn new() -> Self { Self { curves: Vec::new() } }

    pub fn add_curve(&mut self, name: &str, points: Vec<[f32; 3]>, degree: u32, closed: bool) {
        self.curves.push(ExportCurve { name: name.to_string(), control_points: points, degree, is_closed: closed });
    }

    pub fn count(&self) -> usize { self.curves.len() }

    pub fn total_control_points(&self) -> usize {
        self.curves.iter().map(|c| c.control_points.len()).sum()
    }

    pub fn find(&self, name: &str) -> Option<&ExportCurve> {
        self.curves.iter().find(|c| c.name == name)
    }

    pub fn to_json(&self) -> String {
        let mut s = String::from("[");
        for (i, c) in self.curves.iter().enumerate() {
            if i > 0 { s.push(','); }
            s.push_str(&format!(
                "{{\"name\":\"{}\",\"points\":{},\"degree\":{},\"closed\":{}}}",
                c.name, c.control_points.len(), c.degree, c.is_closed
            ));
        }
        s.push(']');
        s
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.curves.len() as u32).to_le_bytes());
        for c in &self.curves {
            bytes.extend_from_slice(&c.degree.to_le_bytes());
            bytes.push(if c.is_closed { 1 } else { 0 });
            bytes.extend_from_slice(&(c.control_points.len() as u32).to_le_bytes());
            for p in &c.control_points {
                for &f in p { bytes.extend_from_slice(&f.to_le_bytes()); }
            }
        }
        bytes
    }
}

impl Default for CurveDataExport {
    fn default() -> Self { Self::new() }
}

/// Validate curve data.
#[allow(dead_code)]
pub fn validate_curve_data(cd: &CurveDataExport) -> bool {
    cd.curves.iter().all(|c| c.control_points.len() > c.degree as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CurveDataExport {
        let mut cd = CurveDataExport::new();
        cd.add_curve("path", vec![[0.0;3],[1.0,0.0,0.0],[2.0,1.0,0.0],[3.0,0.0,0.0]], 3, false);
        cd
    }

    #[test]
    fn test_count() { assert_eq!(sample().count(), 1); }

    #[test]
    fn test_total_points() { assert_eq!(sample().total_control_points(), 4); }

    #[test]
    fn test_find() { assert!(sample().find("path").is_some()); }

    #[test]
    fn test_find_missing() { assert!(sample().find("nope").is_none()); }

    #[test]
    fn test_validate() { assert!(validate_curve_data(&sample())); }

    #[test]
    fn test_to_json() { assert!(sample().to_json().contains("path")); }

    #[test]
    fn test_to_bytes() { assert!(!sample().to_bytes().is_empty()); }

    #[test]
    fn test_empty() { assert_eq!(CurveDataExport::new().count(), 0); }

    #[test]
    fn test_default() { assert_eq!(CurveDataExport::default().count(), 0); }

    #[test]
    fn test_invalid_degree() {
        let mut cd = CurveDataExport::new();
        cd.add_curve("bad", vec![[0.0;3]], 3, false);
        assert!(!validate_curve_data(&cd));
    }
}
