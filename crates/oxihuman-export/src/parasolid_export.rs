// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Parasolid XT format export stub.

/// Parasolid XT entity tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PsEntityTag {
    Body,
    Face,
    Edge,
    Vertex,
}

/// A Parasolid XT entity stub.
#[derive(Debug, Clone)]
pub struct PsEntity {
    pub tag: PsEntityTag,
    pub id: u64,
    pub attributes: Vec<f64>,
}

/// Parasolid XT export container.
#[derive(Debug, Clone, Default)]
pub struct ParasolidExport {
    pub schema_version: String,
    pub entities: Vec<PsEntity>,
}

/// Create a new Parasolid export.
pub fn new_parasolid_export(schema_version: &str) -> ParasolidExport {
    ParasolidExport {
        schema_version: schema_version.to_string(),
        entities: Vec::new(),
    }
}

/// Add an entity; returns its id.
pub fn add_ps_entity(export: &mut ParasolidExport, tag: PsEntityTag, attributes: Vec<f64>) -> u64 {
    let id = export.entities.len() as u64 + 1;
    export.entities.push(PsEntity {
        tag,
        id,
        attributes,
    });
    id
}

/// Return total entity count.
pub fn ps_entity_count(export: &ParasolidExport) -> usize {
    export.entities.len()
}

/// Count entities of a given tag.
pub fn ps_count_by_tag(export: &ParasolidExport, tag: PsEntityTag) -> usize {
    export.entities.iter().filter(|e| e.tag == tag).count()
}

/// Render the XT file magic header (stub).
pub fn ps_xt_header(export: &ParasolidExport) -> String {
    format!(
        "PS_XT_BEGIN\nSCHEMA_VERSION={}\nBEGIN_TOPOLOGY",
        export.schema_version
    )
}

/// Validate that no entity has a duplicate id.
pub fn validate_parasolid(export: &ParasolidExport) -> bool {
    let mut ids: Vec<u64> = export.entities.iter().map(|e| e.id).collect();
    ids.sort_unstable();
    ids.windows(2).all(|w| w[0] != w[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_export_empty() {
        let exp = new_parasolid_export("35.0");
        assert_eq!(ps_entity_count(&exp), 0);
    }

    #[test]
    fn test_add_entity_increments_count() {
        let mut exp = new_parasolid_export("35.0");
        add_ps_entity(&mut exp, PsEntityTag::Body, vec![]);
        assert_eq!(ps_entity_count(&exp), 1);
    }

    #[test]
    fn test_add_entity_returns_id() {
        let mut exp = new_parasolid_export("35.0");
        let id = add_ps_entity(&mut exp, PsEntityTag::Face, vec![]);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_count_by_tag() {
        let mut exp = new_parasolid_export("35.0");
        add_ps_entity(&mut exp, PsEntityTag::Face, vec![]);
        add_ps_entity(&mut exp, PsEntityTag::Face, vec![]);
        add_ps_entity(&mut exp, PsEntityTag::Edge, vec![]);
        assert_eq!(ps_count_by_tag(&exp, PsEntityTag::Face), 2);
        assert_eq!(ps_count_by_tag(&exp, PsEntityTag::Edge), 1);
    }

    #[test]
    fn test_header_contains_schema() {
        let exp = new_parasolid_export("35.0");
        assert!(ps_xt_header(&exp).contains("35.0"));
    }

    #[test]
    fn test_validate_no_duplicates() {
        let mut exp = new_parasolid_export("35.0");
        add_ps_entity(&mut exp, PsEntityTag::Body, vec![]);
        add_ps_entity(&mut exp, PsEntityTag::Face, vec![]);
        assert!(validate_parasolid(&exp));
    }

    #[test]
    fn test_validate_empty() {
        assert!(validate_parasolid(&new_parasolid_export("35.0")));
    }

    #[test]
    fn test_schema_version_stored() {
        let exp = new_parasolid_export("36.1");
        assert_eq!(exp.schema_version, "36.1");
    }

    #[test]
    fn test_vertex_count_by_tag() {
        let mut exp = new_parasolid_export("35.0");
        for _ in 0..5 {
            add_ps_entity(&mut exp, PsEntityTag::Vertex, vec![]);
        }
        assert_eq!(ps_count_by_tag(&exp, PsEntityTag::Vertex), 5);
    }
}
