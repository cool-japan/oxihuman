// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! OData feed export stub — generates OData v4 feed documents for mesh/animation data.

/// OData entity property kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OdataPropertyKind {
    Primitive,
    Navigation,
    Complex,
}

/// An OData entity property descriptor.
#[derive(Debug, Clone)]
pub struct OdataProperty {
    pub name: String,
    pub edm_type: String,
    pub nullable: bool,
    pub kind: OdataPropertyKind,
}

/// An OData entity type descriptor.
#[derive(Debug, Clone)]
pub struct OdataEntityType {
    pub name: String,
    pub key: String,
    pub properties: Vec<OdataProperty>,
}

/// An OData entity set (collection endpoint).
#[derive(Debug, Clone)]
pub struct OdataEntitySet {
    pub name: String,
    pub entity_type: String,
}

/// An OData feed export session.
#[derive(Debug, Default)]
pub struct OdataExport {
    pub entity_types: Vec<OdataEntityType>,
    pub entity_sets: Vec<OdataEntitySet>,
    pub service_root: String,
    pub namespace: String,
}

/// Create a new OData export session.
pub fn new_odata_export(service_root: &str, namespace: &str) -> OdataExport {
    OdataExport {
        entity_types: Vec::new(),
        entity_sets: Vec::new(),
        service_root: service_root.to_owned(),
        namespace: namespace.to_owned(),
    }
}

/// Add an entity type with a key property.
pub fn add_odata_entity_type(export: &mut OdataExport, name: &str, key: &str) {
    export.entity_types.push(OdataEntityType {
        name: name.to_owned(),
        key: key.to_owned(),
        properties: Vec::new(),
    });
}

/// Add a primitive property to the last entity type.
pub fn add_odata_property(
    export: &mut OdataExport,
    prop_name: &str,
    edm_type: &str,
    nullable: bool,
) {
    if let Some(et) = export.entity_types.last_mut() {
        et.properties.push(OdataProperty {
            name: prop_name.to_owned(),
            edm_type: edm_type.to_owned(),
            nullable,
            kind: OdataPropertyKind::Primitive,
        });
    }
}

/// Add a navigation property to the last entity type.
pub fn add_odata_nav_property(export: &mut OdataExport, prop_name: &str, edm_type: &str) {
    if let Some(et) = export.entity_types.last_mut() {
        et.properties.push(OdataProperty {
            name: prop_name.to_owned(),
            edm_type: edm_type.to_owned(),
            nullable: true,
            kind: OdataPropertyKind::Navigation,
        });
    }
}

/// Register an entity set.
pub fn add_odata_entity_set(export: &mut OdataExport, set_name: &str, entity_type: &str) {
    export.entity_sets.push(OdataEntitySet {
        name: set_name.to_owned(),
        entity_type: entity_type.to_owned(),
    });
}

/// Number of entity types.
pub fn odata_entity_type_count(export: &OdataExport) -> usize {
    export.entity_types.len()
}

/// Number of entity sets.
pub fn odata_entity_set_count(export: &OdataExport) -> usize {
    export.entity_sets.len()
}

/// Total number of properties across all entity types.
pub fn total_odata_properties(export: &OdataExport) -> usize {
    export
        .entity_types
        .iter()
        .map(|et| et.properties.len())
        .sum()
}

/// Find an entity type by name.
pub fn find_odata_entity_type<'a>(
    export: &'a OdataExport,
    name: &str,
) -> Option<&'a OdataEntityType> {
    export.entity_types.iter().find(|et| et.name == name)
}

/// Find an entity set by name.
pub fn find_odata_entity_set<'a>(
    export: &'a OdataExport,
    name: &str,
) -> Option<&'a OdataEntitySet> {
    export.entity_sets.iter().find(|es| es.name == name)
}

/// Count navigation properties across all entity types.
pub fn navigation_property_count(export: &OdataExport) -> usize {
    export
        .entity_types
        .iter()
        .flat_map(|et| et.properties.iter())
        .filter(|p| p.kind == OdataPropertyKind::Navigation)
        .count()
}

/// Render a minimal OData $metadata CSDL skeleton.
pub fn render_odata_metadata(export: &OdataExport) -> String {
    let mut out = format!(
        "<edmx:Edmx Version=\"4.0\">\n<edmx:DataServices>\n<Schema Namespace=\"{}\">\n",
        export.namespace
    );
    for et in &export.entity_types {
        out.push_str(&format!("  <EntityType Name=\"{}\">\n", et.name));
        out.push_str(&format!(
            "    <Key><PropertyRef Name=\"{}\"/></Key>\n",
            et.key
        ));
        for p in &et.properties {
            out.push_str(&format!(
                "    <Property Name=\"{}\" Type=\"{}\"/>\n",
                p.name, p.edm_type
            ));
        }
        out.push_str("  </EntityType>\n");
    }
    out.push_str("</Schema>\n</edmx:DataServices>\n</edmx:Edmx>");
    out
}

/// Serialize metadata summary to JSON-style string.
pub fn odata_export_to_json(export: &OdataExport) -> String {
    format!(
        r#"{{"service_root":"{}", "namespace":"{}", "entity_type_count":{}, "entity_set_count":{}}}"#,
        export.service_root,
        export.namespace,
        odata_entity_type_count(export),
        odata_entity_set_count(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        /* fresh export has no entity types */
        let e = new_odata_export("https://api.example.com/odata", "Example.Model");
        assert_eq!(odata_entity_type_count(&e), 0);
    }

    #[test]
    fn add_entity_type_increments_count() {
        /* adding entity type increases count */
        let mut e = new_odata_export("https://api.example.com/odata", "Example.Model");
        add_odata_entity_type(&mut e, "Mesh", "Id");
        assert_eq!(odata_entity_type_count(&e), 1);
    }

    #[test]
    fn add_entity_set_increments_count() {
        /* adding entity set increases count */
        let mut e = new_odata_export("https://api.example.com/odata", "Example.Model");
        add_odata_entity_type(&mut e, "Mesh", "Id");
        add_odata_entity_set(&mut e, "Meshes", "Mesh");
        assert_eq!(odata_entity_set_count(&e), 1);
    }

    #[test]
    fn add_property_stored() {
        /* property is stored on entity type */
        let mut e = new_odata_export("https://api.example.com/odata", "Example.Model");
        add_odata_entity_type(&mut e, "Mesh", "Id");
        add_odata_property(&mut e, "Id", "Edm.Int32", false);
        assert_eq!(e.entity_types[0].properties.len(), 1);
    }

    #[test]
    fn total_properties_correct() {
        /* total properties across all entity types */
        let mut e = new_odata_export("https://api.example.com/odata", "Example.Model");
        add_odata_entity_type(&mut e, "Mesh", "Id");
        add_odata_property(&mut e, "Id", "Edm.Int32", false);
        add_odata_property(&mut e, "Name", "Edm.String", true);
        assert_eq!(total_odata_properties(&e), 2);
    }

    #[test]
    fn find_entity_type_success() {
        /* find returns matching entity type */
        let mut e = new_odata_export("https://api.example.com/odata", "Example.Model");
        add_odata_entity_type(&mut e, "Avatar", "Id");
        assert!(find_odata_entity_type(&e, "Avatar").is_some());
    }

    #[test]
    fn find_entity_type_missing_none() {
        /* missing entity type returns None */
        let e = new_odata_export("https://api.example.com/odata", "Example.Model");
        assert!(find_odata_entity_type(&e, "Ghost").is_none());
    }

    #[test]
    fn navigation_property_counted() {
        /* navigation properties counted separately */
        let mut e = new_odata_export("https://api.example.com/odata", "Example.Model");
        add_odata_entity_type(&mut e, "Mesh", "Id");
        add_odata_property(&mut e, "Id", "Edm.Int32", false);
        add_odata_nav_property(&mut e, "Bones", "Collection(Example.Model.Bone)");
        assert_eq!(navigation_property_count(&e), 1);
    }

    #[test]
    fn render_metadata_contains_schema() {
        /* rendered metadata contains Schema tag */
        let mut e = new_odata_export("https://api.example.com/odata", "Example.Model");
        add_odata_entity_type(&mut e, "Mesh", "Id");
        assert!(render_odata_metadata(&e).contains("Schema"));
    }

    #[test]
    fn json_contains_service_root() {
        /* JSON includes service root */
        let e = new_odata_export("https://mesh.api.com/odata", "Mesh.Model");
        assert!(odata_export_to_json(&e).contains("mesh.api.com"));
    }
}
