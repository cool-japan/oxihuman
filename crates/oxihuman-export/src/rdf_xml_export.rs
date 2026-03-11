// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! RDF/XML serialization stub.

/// An RDF/XML namespace binding.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RdfNs {
    pub prefix: String,
    pub uri: String,
}

/// An RDF description element.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RdfDescription {
    pub about: String,
    pub properties: Vec<(String, String)>,
}

/// An RDF/XML document.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct RdfXmlDoc {
    pub namespaces: Vec<RdfNs>,
    pub descriptions: Vec<RdfDescription>,
}

impl RdfXmlDoc {
    #[allow(dead_code)]
    pub fn new() -> Self {
        RdfXmlDoc::default()
    }

    #[allow(dead_code)]
    pub fn add_namespace(&mut self, prefix: &str, uri: &str) {
        self.namespaces.push(RdfNs {
            prefix: prefix.to_string(),
            uri: uri.to_string(),
        });
    }

    #[allow(dead_code)]
    pub fn add_description(&mut self, about: &str) -> usize {
        self.descriptions.push(RdfDescription {
            about: about.to_string(),
            properties: Vec::new(),
        });
        self.descriptions.len() - 1
    }

    #[allow(dead_code)]
    pub fn add_property(&mut self, desc_idx: usize, predicate: &str, value: &str) {
        if desc_idx < self.descriptions.len() {
            self.descriptions[desc_idx]
                .properties
                .push((predicate.to_string(), value.to_string()));
        }
    }
}

/// Export to RDF/XML text.
#[allow(dead_code)]
pub fn export_rdf_xml(doc: &RdfXmlDoc) -> String {
    let ns_attrs: Vec<String> = doc
        .namespaces
        .iter()
        .map(|ns| format!(r#"    xmlns:{}="{}""#, ns.prefix, ns.uri))
        .collect();
    let ns_str = if ns_attrs.is_empty() {
        String::new()
    } else {
        format!("\n{}", ns_attrs.join("\n"))
    };

    let mut descriptions = String::new();
    for desc in &doc.descriptions {
        descriptions.push_str(&format!(
            "  <rdf:Description rdf:about=\"{}\">\n",
            desc.about
        ));
        for (pred, val) in &desc.properties {
            descriptions.push_str(&format!("    <{}>{}</{}>\n", pred, val, pred));
        }
        descriptions.push_str("  </rdf:Description>\n");
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<rdf:RDF\n    xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"{}>\n{}</rdf:RDF>",
        ns_str, descriptions
    )
}

/// Description count.
#[allow(dead_code)]
pub fn description_count(doc: &RdfXmlDoc) -> usize {
    doc.descriptions.len()
}

/// Check if the export starts with XML declaration.
#[allow(dead_code)]
pub fn has_xml_declaration(s: &str) -> bool {
    s.starts_with("<?xml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_doc_empty() {
        let doc = RdfXmlDoc::new();
        assert_eq!(description_count(&doc), 0);
    }

    #[test]
    fn add_description_count() {
        let mut doc = RdfXmlDoc::new();
        doc.add_description("http://example.com/Alice");
        assert_eq!(description_count(&doc), 1);
    }

    #[test]
    fn export_has_xml_declaration() {
        let doc = RdfXmlDoc::new();
        let s = export_rdf_xml(&doc);
        assert!(has_xml_declaration(&s));
    }

    #[test]
    fn export_contains_rdf_root() {
        let doc = RdfXmlDoc::new();
        let s = export_rdf_xml(&doc);
        assert!(s.contains("rdf:RDF"));
    }

    #[test]
    fn export_contains_about() {
        let mut doc = RdfXmlDoc::new();
        doc.add_description("http://example.com/Alice");
        let s = export_rdf_xml(&doc);
        assert!(s.contains("http://example.com/Alice"));
    }

    #[test]
    fn add_property_in_output() {
        let mut doc = RdfXmlDoc::new();
        let idx = doc.add_description("http://example.com/Alice");
        doc.add_property(idx, "schema:name", "Alice");
        let s = export_rdf_xml(&doc);
        assert!(s.contains("Alice"));
    }

    #[test]
    fn add_namespace_in_output() {
        let mut doc = RdfXmlDoc::new();
        doc.add_namespace("schema", "http://schema.org/");
        let s = export_rdf_xml(&doc);
        assert!(s.contains("schema"));
    }

    #[test]
    fn invalid_desc_idx_safe() {
        let mut doc = RdfXmlDoc::new();
        doc.add_property(99, "p", "v");
        assert_eq!(description_count(&doc), 0);
    }

    #[test]
    fn rdf_description_tag_in_output() {
        let mut doc = RdfXmlDoc::new();
        doc.add_description("http://x.com/1");
        let s = export_rdf_xml(&doc);
        assert!(s.contains("rdf:Description"));
    }

    #[test]
    fn closing_rdf_tag() {
        let doc = RdfXmlDoc::new();
        let s = export_rdf_xml(&doc);
        assert!(s.ends_with("</rdf:RDF>"));
    }
}
