// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! STEP solid body export stub.

/// STEP entity stub types used in AP214/AP203.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepEntityKind {
    ManifoldSolidBrep,
    ClosedShell,
    AdvancedFaceSurface,
}

/// A STEP entity record stub.
#[derive(Debug, Clone)]
pub struct StepEntity {
    pub id: u32,
    pub kind: StepEntityKind,
    pub attributes: Vec<String>,
}

/// STEP solid export container.
#[derive(Debug, Clone, Default)]
pub struct StepSolidExport {
    pub entities: Vec<StepEntity>,
    pub schema: String,
    pub author: String,
}

/// Create a new STEP solid export.
pub fn new_step_solid_export(schema: &str, author: &str) -> StepSolidExport {
    StepSolidExport {
        entities: Vec::new(),
        schema: schema.to_string(),
        author: author.to_string(),
    }
}

/// Add a STEP entity and return its ID.
pub fn add_step_entity(
    export: &mut StepSolidExport,
    kind: StepEntityKind,
    attributes: Vec<String>,
) -> u32 {
    let id = export.entities.len() as u32 + 1;
    export.entities.push(StepEntity {
        id,
        kind,
        attributes,
    });
    id
}

/// Return the entity count.
pub fn step_entity_count(export: &StepSolidExport) -> usize {
    export.entities.len()
}

/// Render the STEP FILE_DESCRIPTION header (stub).
pub fn step_file_header(export: &StepSolidExport) -> String {
    format!(
        "ISO-10303-21;\nHEADER;\nFILE_SCHEMA(('{}'));\nFILE_AUTHOR('{}');\nENDSEC;",
        export.schema, export.author
    )
}

/// Render a STEP data section entity line (stub).
pub fn step_entity_line(entity: &StepEntity) -> String {
    let kind_str = match entity.kind {
        StepEntityKind::ManifoldSolidBrep => "MANIFOLD_SOLID_BREP",
        StepEntityKind::ClosedShell => "CLOSED_SHELL",
        StepEntityKind::AdvancedFaceSurface => "ADVANCED_FACE",
    };
    format!(
        "#{} = {}({});",
        entity.id,
        kind_str,
        entity.attributes.join(",")
    )
}

/// Validate that the export has at least one entity.
pub fn validate_step_export(export: &StepSolidExport) -> bool {
    !export.entities.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_export_empty() {
        let exp = new_step_solid_export("AP214", "Alice");
        assert_eq!(step_entity_count(&exp), 0);
    }

    #[test]
    fn test_add_entity_returns_id() {
        let mut exp = new_step_solid_export("AP214", "Alice");
        let id = add_step_entity(
            &mut exp,
            StepEntityKind::ManifoldSolidBrep,
            vec!["''".into()],
        );
        assert_eq!(id, 1);
    }

    #[test]
    fn test_entity_count_increments() {
        let mut exp = new_step_solid_export("AP214", "Alice");
        add_step_entity(&mut exp, StepEntityKind::ClosedShell, vec![]);
        add_step_entity(&mut exp, StepEntityKind::AdvancedFaceSurface, vec![]);
        assert_eq!(step_entity_count(&exp), 2);
    }

    #[test]
    fn test_file_header_contains_schema() {
        let exp = new_step_solid_export("AP214", "Bob");
        assert!(step_file_header(&exp).contains("AP214"));
    }

    #[test]
    fn test_file_header_contains_author() {
        let exp = new_step_solid_export("AP203", "Charlie");
        assert!(step_file_header(&exp).contains("Charlie"));
    }

    #[test]
    fn test_entity_line_manifold() {
        let e = StepEntity {
            id: 1,
            kind: StepEntityKind::ManifoldSolidBrep,
            attributes: vec![],
        };
        assert!(step_entity_line(&e).contains("MANIFOLD_SOLID_BREP"));
    }

    #[test]
    fn test_entity_line_closed_shell() {
        let e = StepEntity {
            id: 2,
            kind: StepEntityKind::ClosedShell,
            attributes: vec![],
        };
        assert!(step_entity_line(&e).contains("CLOSED_SHELL"));
    }

    #[test]
    fn test_validate_non_empty() {
        let mut exp = new_step_solid_export("AP214", "X");
        add_step_entity(&mut exp, StepEntityKind::ManifoldSolidBrep, vec![]);
        assert!(validate_step_export(&exp));
    }

    #[test]
    fn test_validate_empty_fails() {
        let exp = new_step_solid_export("AP214", "X");
        assert!(!validate_step_export(&exp));
    }

    #[test]
    fn test_schema_stored() {
        let exp = new_step_solid_export("AP203", "Y");
        assert_eq!(exp.schema, "AP203");
    }
}
