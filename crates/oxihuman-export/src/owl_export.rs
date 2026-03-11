// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! OWL ontology stub export.

use std::collections::BTreeSet;

/// An OWL class definition.
#[derive(Debug, Clone)]
pub struct OwlClass {
    pub iri: String,
    pub label: String,
    pub superclasses: Vec<String>,
    pub comment: Option<String>,
}

impl OwlClass {
    /// Create a class with IRI and label.
    pub fn new(iri: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            iri: iri.into(),
            label: label.into(),
            superclasses: Vec::new(),
            comment: None,
        }
    }
}

/// An OWL object property.
#[derive(Debug, Clone)]
pub struct OwlObjectProperty {
    pub iri: String,
    pub label: String,
    pub domain: Option<String>,
    pub range: Option<String>,
}

/// An OWL ontology.
#[derive(Debug, Clone, Default)]
pub struct OwlOntology {
    pub iri: String,
    pub version: String,
    pub classes: Vec<OwlClass>,
    pub object_properties: Vec<OwlObjectProperty>,
}

impl OwlOntology {
    /// Add a class.
    pub fn add_class(&mut self, cls: OwlClass) {
        self.classes.push(cls);
    }

    /// Add an object property.
    pub fn add_object_property(&mut self, prop: OwlObjectProperty) {
        self.object_properties.push(prop);
    }

    /// Number of classes.
    pub fn class_count(&self) -> usize {
        self.classes.len()
    }

    /// Find class by IRI.
    pub fn find_class(&self, iri: &str) -> Option<&OwlClass> {
        self.classes.iter().find(|c| c.iri == iri)
    }
}

/// Render the ontology as Turtle-like stub.
pub fn render_owl_turtle(onto: &OwlOntology) -> String {
    let mut out = format!(
        "@prefix owl: <http://www.w3.org/2002/07/owl#> .\n@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\n<{}> a owl:Ontology .\n\n",
        onto.iri
    );
    for cls in &onto.classes {
        out.push_str(&format!(
            "<{}> a owl:Class ;\n  rdfs:label \"{}\" .\n\n",
            cls.iri, cls.label
        ));
    }
    for prop in &onto.object_properties {
        out.push_str(&format!(
            "<{}> a owl:ObjectProperty ;\n  rdfs:label \"{}\" .\n\n",
            prop.iri, prop.label
        ));
    }
    out
}

/// Validate the ontology (no duplicate IRIs, non-empty ontology IRI).
pub fn validate_ontology(onto: &OwlOntology) -> bool {
    if onto.iri.is_empty() {
        return false;
    }
    let iris: BTreeSet<&str> = onto.classes.iter().map(|c| c.iri.as_str()).collect();
    iris.len() == onto.classes.len()
}

/// Collect all superclass IRIs referenced.
pub fn all_superclass_iris(onto: &OwlOntology) -> Vec<&str> {
    onto.classes
        .iter()
        .flat_map(|c| c.superclasses.iter().map(String::as_str))
        .collect()
}

/// Count classes with no superclass (root classes).
pub fn root_class_count(onto: &OwlOntology) -> usize {
    onto.classes
        .iter()
        .filter(|c| c.superclasses.is_empty())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ontology() -> OwlOntology {
        let mut onto = OwlOntology {
            iri: "https://example.com/onto".into(),
            version: "1.0".into(),
            ..Default::default()
        };
        let mut animal = OwlClass::new("https://example.com/onto#Animal", "Animal");
        animal.comment = Some("A living creature".into());
        let dog = OwlClass::new("https://example.com/onto#Dog", "Dog");
        onto.add_class(animal);
        onto.add_class(dog);
        onto
    }

    #[test]
    fn class_count() {
        assert_eq!(sample_ontology().class_count(), 2);
    }

    #[test]
    fn find_class_found() {
        let onto = sample_ontology();
        assert!(onto.find_class("https://example.com/onto#Animal").is_some());
    }

    #[test]
    fn find_class_missing() {
        assert!(sample_ontology().find_class("nope").is_none());
    }

    #[test]
    fn render_contains_owl_class() {
        assert!(render_owl_turtle(&sample_ontology()).contains("owl:Class"));
    }

    #[test]
    fn render_contains_ontology_iri() {
        assert!(render_owl_turtle(&sample_ontology()).contains("https://example.com/onto"));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_ontology(&sample_ontology()));
    }

    #[test]
    fn validate_empty_iri() {
        let onto = OwlOntology::default();
        assert!(!validate_ontology(&onto));
    }

    #[test]
    fn root_class_count_correct() {
        /* both classes have no superclasses → 2 roots */
        assert_eq!(root_class_count(&sample_ontology()), 2);
    }

    #[test]
    fn all_superclass_iris_empty() {
        assert!(all_superclass_iris(&sample_ontology()).is_empty());
    }
}
