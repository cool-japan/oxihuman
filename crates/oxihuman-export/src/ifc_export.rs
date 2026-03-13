// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! IFC BIM building model export stub.

/// IFC entity class stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IfcClass {
    IfcProject,
    IfcSite,
    IfcBuilding,
    IfcBuildingStorey,
    IfcWall,
    IfcSlab,
    IfcColumn,
}

/// A single IFC entity stub.
#[derive(Debug, Clone)]
pub struct IfcEntity {
    pub id: u32,
    pub class: IfcClass,
    pub name: String,
    pub attributes: Vec<String>,
}

/// IFC export container.
#[derive(Debug, Clone, Default)]
pub struct IfcExport {
    pub schema: String,
    pub entities: Vec<IfcEntity>,
}

/// Create a new IFC export.
pub fn new_ifc_export(schema: &str) -> IfcExport {
    IfcExport {
        schema: schema.to_string(),
        entities: Vec::new(),
    }
}

/// Add an IFC entity; returns its id.
pub fn add_ifc_entity(export: &mut IfcExport, class: IfcClass, name: &str) -> u32 {
    let id = export.entities.len() as u32 + 1;
    export.entities.push(IfcEntity {
        id,
        class,
        name: name.to_string(),
        attributes: Vec::new(),
    });
    id
}

/// Return entity count.
pub fn ifc_entity_count(export: &IfcExport) -> usize {
    export.entities.len()
}

/// Count entities of a given class.
pub fn ifc_count_class(export: &IfcExport, class: IfcClass) -> usize {
    export.entities.iter().filter(|e| e.class == class).count()
}

/// Render a stub STEP-based IFC header.
pub fn ifc_header(export: &IfcExport) -> String {
    format!(
        "ISO-10303-21;\nHEADER;\nFILE_SCHEMA(('{}'));\nENDSEC;\nDATA;",
        export.schema
    )
}

/// Render a single IFC entity line.
pub fn ifc_entity_line(entity: &IfcEntity) -> String {
    let class_name = match entity.class {
        IfcClass::IfcProject => "IFCPROJECT",
        IfcClass::IfcSite => "IFCSITE",
        IfcClass::IfcBuilding => "IFCBUILDING",
        IfcClass::IfcBuildingStorey => "IFCBUILDINGSTOREY",
        IfcClass::IfcWall => "IFCWALL",
        IfcClass::IfcSlab => "IFCSLAB",
        IfcClass::IfcColumn => "IFCCOLUMN",
    };
    format!("#{} = {}('{}');", entity.id, class_name, entity.name)
}

/// Validate that a project entity exists.
pub fn validate_ifc(export: &IfcExport) -> bool {
    export
        .entities
        .iter()
        .any(|e| e.class == IfcClass::IfcProject)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_export_empty() {
        let exp = new_ifc_export("IFC4");
        assert_eq!(ifc_entity_count(&exp), 0);
    }

    #[test]
    fn test_add_entity_returns_id() {
        let mut exp = new_ifc_export("IFC4");
        let id = add_ifc_entity(&mut exp, IfcClass::IfcProject, "MyProject");
        assert_eq!(id, 1);
    }

    #[test]
    fn test_entity_count() {
        let mut exp = new_ifc_export("IFC4");
        add_ifc_entity(&mut exp, IfcClass::IfcProject, "P");
        add_ifc_entity(&mut exp, IfcClass::IfcBuilding, "B");
        assert_eq!(ifc_entity_count(&exp), 2);
    }

    #[test]
    fn test_count_class() {
        let mut exp = new_ifc_export("IFC4");
        add_ifc_entity(&mut exp, IfcClass::IfcWall, "W1");
        add_ifc_entity(&mut exp, IfcClass::IfcWall, "W2");
        add_ifc_entity(&mut exp, IfcClass::IfcSlab, "S1");
        assert_eq!(ifc_count_class(&exp, IfcClass::IfcWall), 2);
    }

    #[test]
    fn test_header_contains_schema() {
        let exp = new_ifc_export("IFC4");
        assert!(ifc_header(&exp).contains("IFC4"));
    }

    #[test]
    fn test_entity_line_wall() {
        let mut exp = new_ifc_export("IFC4");
        add_ifc_entity(&mut exp, IfcClass::IfcWall, "MyWall");
        let line = ifc_entity_line(&exp.entities[0]);
        assert!(line.contains("IFCWALL"));
        assert!(line.contains("MyWall"));
    }

    #[test]
    fn test_validate_with_project() {
        let mut exp = new_ifc_export("IFC4");
        add_ifc_entity(&mut exp, IfcClass::IfcProject, "Proj");
        assert!(validate_ifc(&exp));
    }

    #[test]
    fn test_validate_without_project() {
        let mut exp = new_ifc_export("IFC4");
        add_ifc_entity(&mut exp, IfcClass::IfcWall, "W");
        assert!(!validate_ifc(&exp));
    }

    #[test]
    fn test_schema_stored() {
        let exp = new_ifc_export("IFC2X3");
        assert_eq!(exp.schema, "IFC2X3");
    }
}
