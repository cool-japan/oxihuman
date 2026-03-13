// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 3DXML (CATIA) format export stub.
//! Note: module name is `threedxml_export` because Rust identifiers
//! cannot start with a digit.

/// A 3DXML reference occurrence.
#[derive(Debug, Clone)]
pub struct ThreeDXmlOccurrence {
    pub id: u32,
    pub name: String,
    pub matrix: [[f32; 4]; 4],
}

/// A 3DXML representation stub.
#[derive(Debug, Clone)]
pub struct ThreeDXmlRep {
    pub id: u32,
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
}

/// 3DXML export container.
#[derive(Debug, Clone, Default)]
pub struct ThreeDXmlExport {
    pub schema_version: String,
    pub occurrences: Vec<ThreeDXmlOccurrence>,
    pub reps: Vec<ThreeDXmlRep>,
}

/// Create a new 3DXML export.
pub fn new_threedxml_export(schema_version: &str) -> ThreeDXmlExport {
    ThreeDXmlExport {
        schema_version: schema_version.to_string(),
        occurrences: Vec::new(),
        reps: Vec::new(),
    }
}

/// Add an occurrence; returns its id.
pub fn add_threedxml_occurrence(export: &mut ThreeDXmlExport, name: &str) -> u32 {
    let id = export.occurrences.len() as u32 + 1;
    let matrix = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    export.occurrences.push(ThreeDXmlOccurrence {
        id,
        name: name.to_string(),
        matrix,
    });
    id
}

/// Add a representation; returns its id.
pub fn add_threedxml_rep(
    export: &mut ThreeDXmlExport,
    verts: Vec<[f32; 3]>,
    tris: Vec<[u32; 3]>,
) -> u32 {
    let id = export.reps.len() as u32 + 1;
    export.reps.push(ThreeDXmlRep { id, verts, tris });
    id
}

/// Return occurrence count.
pub fn threedxml_occurrence_count(export: &ThreeDXmlExport) -> usize {
    export.occurrences.len()
}

/// Return representation count.
pub fn threedxml_rep_count(export: &ThreeDXmlExport) -> usize {
    export.reps.len()
}

/// Render a stub XML header.
pub fn threedxml_xml_header(export: &ThreeDXmlExport) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<XPDMRoot schemaVersion=\"{}\">",
        export.schema_version
    )
}

/// Validate triangle indices within each representation.
pub fn validate_threedxml(export: &ThreeDXmlExport) -> bool {
    export.reps.iter().all(|rep| {
        let n = rep.verts.len() as u32;
        rep.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_export_empty() {
        let exp = new_threedxml_export("4.0");
        assert_eq!(threedxml_occurrence_count(&exp), 0);
        assert_eq!(threedxml_rep_count(&exp), 0);
    }

    #[test]
    fn test_add_occurrence() {
        let mut exp = new_threedxml_export("4.0");
        let id = add_threedxml_occurrence(&mut exp, "Part1");
        assert_eq!(id, 1);
        assert_eq!(threedxml_occurrence_count(&exp), 1);
    }

    #[test]
    fn test_add_rep() {
        let mut exp = new_threedxml_export("4.0");
        let v = vec![[0.0f32; 3]; 3];
        let t = vec![[0u32, 1, 2]];
        let id = add_threedxml_rep(&mut exp, v, t);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_xml_header_contains_schema() {
        let exp = new_threedxml_export("4.0");
        assert!(threedxml_xml_header(&exp).contains("4.0"));
    }

    #[test]
    fn test_validate_valid() {
        let mut exp = new_threedxml_export("4.0");
        let v = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let t = vec![[0u32, 1, 2]];
        add_threedxml_rep(&mut exp, v, t);
        assert!(validate_threedxml(&exp));
    }

    #[test]
    fn test_occurrence_name_stored() {
        let mut exp = new_threedxml_export("4.0");
        add_threedxml_occurrence(&mut exp, "MyPart");
        assert_eq!(exp.occurrences[0].name, "MyPart");
    }

    #[test]
    fn test_identity_matrix() {
        let mut exp = new_threedxml_export("4.0");
        add_threedxml_occurrence(&mut exp, "X");
        let m = exp.occurrences[0].matrix;
        #[allow(clippy::needless_range_loop)]
        for i in 0..4 {
            assert!((m[i][i] - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_schema_version_stored() {
        let exp = new_threedxml_export("5.0");
        assert_eq!(exp.schema_version, "5.0");
    }

    #[test]
    fn test_validate_empty() {
        assert!(validate_threedxml(&new_threedxml_export("4.0")));
    }

    #[test]
    fn test_multiple_occurrences() {
        let mut exp = new_threedxml_export("4.0");
        for name in ["A", "B", "C"] {
            add_threedxml_occurrence(&mut exp, name);
        }
        assert_eq!(threedxml_occurrence_count(&exp), 3);
    }
}
