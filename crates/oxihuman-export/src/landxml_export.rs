// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! LandXML civil engineering export stub.

/// A LandXML surface TIN (Triangulated Irregular Network).
#[derive(Debug, Clone)]
pub struct LandXmlSurface {
    pub name: String,
    pub verts: Vec<[f64; 3]>,
    pub tris: Vec<[u32; 3]>,
}

/// A LandXML alignment stub.
#[derive(Debug, Clone)]
pub struct LandXmlAlignment {
    pub name: String,
    pub sta_start: f64,
    pub sta_end: f64,
    pub points: Vec<[f64; 3]>,
}

/// LandXML export container.
#[derive(Debug, Clone, Default)]
pub struct LandXmlExport {
    pub version: String,
    pub surfaces: Vec<LandXmlSurface>,
    pub alignments: Vec<LandXmlAlignment>,
}

/// Create a new LandXML export.
pub fn new_landxml_export(version: &str) -> LandXmlExport {
    LandXmlExport {
        version: version.to_string(),
        surfaces: Vec::new(),
        alignments: Vec::new(),
    }
}

/// Add a TIN surface.
pub fn add_landxml_surface(
    export: &mut LandXmlExport,
    name: &str,
    verts: Vec<[f64; 3]>,
    tris: Vec<[u32; 3]>,
) {
    export.surfaces.push(LandXmlSurface {
        name: name.to_string(),
        verts,
        tris,
    });
}

/// Add an alignment.
pub fn add_landxml_alignment(
    export: &mut LandXmlExport,
    name: &str,
    sta_start: f64,
    sta_end: f64,
    points: Vec<[f64; 3]>,
) {
    export.alignments.push(LandXmlAlignment {
        name: name.to_string(),
        sta_start,
        sta_end,
        points,
    });
}

/// Return surface count.
pub fn landxml_surface_count(export: &LandXmlExport) -> usize {
    export.surfaces.len()
}

/// Return alignment count.
pub fn landxml_alignment_count(export: &LandXmlExport) -> usize {
    export.alignments.len()
}

/// Render a stub LandXML header.
pub fn landxml_xml_header(export: &LandXmlExport) -> String {
    format!(
        "<LandXML version=\"{}\" xmlns=\"http://www.landxml.org/schema/LandXML-{}\">",
        export.version, export.version
    )
}

/// Validate all TIN triangle indices.
pub fn validate_landxml(export: &LandXmlExport) -> bool {
    export.surfaces.iter().all(|s| {
        let n = s.verts.len() as u32;
        s.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
    })
}

/// Return total triangle count across all surfaces.
pub fn landxml_total_tris(export: &LandXmlExport) -> usize {
    export.surfaces.iter().map(|s| s.tris.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_tin() -> (Vec<[f64; 3]>, Vec<[u32; 3]>) {
        let v = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 0.5],
            [1.0, 1.0, 1.5],
        ];
        let t = vec![[0, 1, 2], [1, 3, 2]];
        (v, t)
    }

    #[test]
    fn test_new_export_empty() {
        let exp = new_landxml_export("1.2");
        assert_eq!(landxml_surface_count(&exp), 0);
    }

    #[test]
    fn test_add_surface() {
        let mut exp = new_landxml_export("1.2");
        let (v, t) = simple_tin();
        add_landxml_surface(&mut exp, "Ground", v, t);
        assert_eq!(landxml_surface_count(&exp), 1);
    }

    #[test]
    fn test_add_alignment() {
        let mut exp = new_landxml_export("1.2");
        add_landxml_alignment(
            &mut exp,
            "Road1",
            0.0,
            100.0,
            vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0]],
        );
        assert_eq!(landxml_alignment_count(&exp), 1);
    }

    #[test]
    fn test_header_contains_version() {
        let exp = new_landxml_export("1.2");
        assert!(landxml_xml_header(&exp).contains("1.2"));
    }

    #[test]
    fn test_validate_valid() {
        let mut exp = new_landxml_export("1.2");
        let (v, t) = simple_tin();
        add_landxml_surface(&mut exp, "G", v, t);
        assert!(validate_landxml(&exp));
    }

    #[test]
    fn test_total_tris() {
        let mut exp = new_landxml_export("1.2");
        let (v, t) = simple_tin();
        add_landxml_surface(&mut exp, "G", v, t);
        assert_eq!(landxml_total_tris(&exp), 2);
    }

    #[test]
    fn test_validate_empty() {
        assert!(validate_landxml(&new_landxml_export("1.2")));
    }

    #[test]
    fn test_version_stored() {
        let exp = new_landxml_export("2.0");
        assert_eq!(exp.version, "2.0");
    }

    #[test]
    fn test_alignment_stations() {
        let mut exp = new_landxml_export("1.2");
        add_landxml_alignment(&mut exp, "R", 100.0, 500.0, vec![]);
        assert!((exp.alignments[0].sta_start - 100.0).abs() < 1e-9);
        assert!((exp.alignments[0].sta_end - 500.0).abs() < 1e-9);
    }
}
