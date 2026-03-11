// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ACIS SAT format export stub.

/// A single ACIS SAT entity record.
#[derive(Debug, Clone)]
pub struct SatEntity {
    pub index: u32,
    pub entity_type: String,
    pub fields: Vec<String>,
}

/// ACIS SAT export container.
#[derive(Debug, Clone, Default)]
pub struct SatExport {
    pub header_version: u32,
    pub entities: Vec<SatEntity>,
    pub unit_scale: f64,
}

/// Create a new SAT export.
pub fn new_sat_export(version: u32, unit_scale: f64) -> SatExport {
    SatExport {
        header_version: version,
        entities: Vec::new(),
        unit_scale,
    }
}

/// Add an ACIS entity; returns its index.
pub fn add_sat_entity(export: &mut SatExport, entity_type: &str, fields: Vec<String>) -> u32 {
    let idx = export.entities.len() as u32;
    export.entities.push(SatEntity {
        index: idx,
        entity_type: entity_type.to_string(),
        fields,
    });
    idx
}

/// Return the entity count.
pub fn sat_entity_count(export: &SatExport) -> usize {
    export.entities.len()
}

/// Render the SAT header section (stub).
pub fn sat_header(export: &SatExport) -> String {
    format!(
        "{} 0 0 0\n400 0 1 0\n{}",
        export.header_version, export.unit_scale
    )
}

/// Render a SAT entity line.
pub fn sat_entity_line(entity: &SatEntity) -> String {
    format!(
        "-{}  {} {}  #",
        entity.index,
        entity.entity_type,
        entity.fields.join(" ")
    )
}

/// Find the first entity of the given type.
pub fn find_sat_entity<'a>(export: &'a SatExport, entity_type: &str) -> Option<&'a SatEntity> {
    export
        .entities
        .iter()
        .find(|e| e.entity_type == entity_type)
}

/// Validate that all entity types are non-empty.
pub fn validate_sat(export: &SatExport) -> bool {
    export.entities.iter().all(|e| !e.entity_type.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sat_export_empty() {
        let exp = new_sat_export(700, 1.0);
        assert_eq!(sat_entity_count(&exp), 0);
    }

    #[test]
    fn test_add_entity_returns_index() {
        let mut exp = new_sat_export(700, 1.0);
        let idx = add_sat_entity(&mut exp, "body", vec![]);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_entity_count_increments() {
        let mut exp = new_sat_export(700, 1.0);
        add_sat_entity(&mut exp, "body", vec![]);
        add_sat_entity(&mut exp, "lump", vec![]);
        assert_eq!(sat_entity_count(&exp), 2);
    }

    #[test]
    fn test_sat_header_contains_version() {
        let exp = new_sat_export(700, 25.4);
        assert!(sat_header(&exp).contains("700"));
    }

    #[test]
    fn test_sat_entity_line_contains_type() {
        let e = SatEntity {
            index: 0,
            entity_type: "face".into(),
            fields: vec![],
        };
        assert!(sat_entity_line(&e).contains("face"));
    }

    #[test]
    fn test_find_entity_existing() {
        let mut exp = new_sat_export(700, 1.0);
        add_sat_entity(&mut exp, "body", vec![]);
        assert!(find_sat_entity(&exp, "body").is_some());
    }

    #[test]
    fn test_find_entity_missing() {
        let exp = new_sat_export(700, 1.0);
        assert!(find_sat_entity(&exp, "body").is_none());
    }

    #[test]
    fn test_validate_valid() {
        let mut exp = new_sat_export(700, 1.0);
        add_sat_entity(&mut exp, "body", vec![]);
        assert!(validate_sat(&exp));
    }

    #[test]
    fn test_unit_scale_stored() {
        let exp = new_sat_export(700, 25.4);
        assert!((exp.unit_scale - 25.4).abs() < 1e-9);
    }

    #[test]
    fn test_version_stored() {
        let exp = new_sat_export(700, 1.0);
        assert_eq!(exp.header_version, 700);
    }
}
