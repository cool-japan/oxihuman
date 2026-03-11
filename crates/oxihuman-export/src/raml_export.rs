// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! RAML API spec stub export (YAML-based).

/// A RAML resource (endpoint).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RamlResource {
    pub path: String,
    pub description: String,
    pub methods: Vec<RamlMethod>,
}

/// A RAML HTTP method.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RamlMethod {
    pub method: String,
    pub description: String,
    pub responses: Vec<(u16, String)>,
}

/// A RAML document.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RamlDoc {
    pub title: String,
    pub version: String,
    pub base_uri: String,
    pub resources: Vec<RamlResource>,
}

/// Create a new RAML document.
#[allow(dead_code)]
pub fn new_raml_doc(title: &str, version: &str, base_uri: &str) -> RamlDoc {
    RamlDoc {
        title: title.to_string(),
        version: version.to_string(),
        base_uri: base_uri.to_string(),
        resources: Vec::new(),
    }
}

/// Add a resource.
#[allow(dead_code)]
pub fn add_resource(doc: &mut RamlDoc, path: &str, description: &str) -> usize {
    doc.resources.push(RamlResource {
        path: path.to_string(),
        description: description.to_string(),
        methods: Vec::new(),
    });
    doc.resources.len() - 1
}

/// Add a method to a resource.
#[allow(dead_code)]
pub fn add_method(doc: &mut RamlDoc, res_idx: usize, method: &str, desc: &str) {
    if res_idx < doc.resources.len() {
        doc.resources[res_idx].methods.push(RamlMethod {
            method: method.to_string(),
            description: desc.to_string(),
            responses: vec![(200, "application/json".to_string())],
        });
    }
}

/// Export to RAML YAML text.
#[allow(dead_code)]
pub fn export_raml(doc: &RamlDoc) -> String {
    let mut out = format!(
        "#%RAML 1.0\ntitle: {}\nversion: {}\nbaseUri: {}\n",
        doc.title, doc.version, doc.base_uri
    );
    for res in &doc.resources {
        out.push_str(&format!(
            "{}:\n  description: {}\n",
            res.path, res.description
        ));
        for m in &res.methods {
            out.push_str(&format!(
                "  {}:\n    description: {}\n    responses:\n",
                m.method, m.description
            ));
            for (code, mime) in &m.responses {
                out.push_str(&format!(
                    "      {}:\n        body:\n          {}: {{}}\n",
                    code, mime
                ));
            }
        }
    }
    out
}

/// Resource count.
#[allow(dead_code)]
pub fn resource_count(doc: &RamlDoc) -> usize {
    doc.resources.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_doc_title() {
        let doc = new_raml_doc("My API", "v1", "https://example.com/api");
        assert_eq!(doc.title, "My API");
    }

    #[test]
    fn add_resource_count() {
        let mut doc = new_raml_doc("API", "v1", "https://x.com");
        add_resource(&mut doc, "/users", "User resource");
        assert_eq!(resource_count(&doc), 1);
    }

    #[test]
    fn export_starts_with_raml_header() {
        let doc = new_raml_doc("API", "v1", "https://x.com");
        let s = export_raml(&doc);
        assert!(s.starts_with("#%RAML 1.0"));
    }

    #[test]
    fn export_contains_title() {
        let doc = new_raml_doc("My API", "v1", "https://x.com");
        let s = export_raml(&doc);
        assert!(s.contains("My API"));
    }

    #[test]
    fn export_contains_base_uri() {
        let doc = new_raml_doc("API", "v1", "https://api.example.com");
        let s = export_raml(&doc);
        assert!(s.contains("https://api.example.com"));
    }

    #[test]
    fn export_contains_resource_path() {
        let mut doc = new_raml_doc("API", "v1", "https://x.com");
        add_resource(&mut doc, "/users", "Users");
        let s = export_raml(&doc);
        assert!(s.contains("/users"));
    }

    #[test]
    fn add_method_stored() {
        let mut doc = new_raml_doc("API", "v1", "https://x.com");
        let idx = add_resource(&mut doc, "/users", "Users");
        add_method(&mut doc, idx, "get", "Get users");
        assert_eq!(doc.resources[0].methods.len(), 1);
    }

    #[test]
    fn method_in_export() {
        let mut doc = new_raml_doc("API", "v1", "https://x.com");
        let idx = add_resource(&mut doc, "/users", "Users");
        add_method(&mut doc, idx, "get", "Get users");
        let s = export_raml(&doc);
        assert!(s.contains("get:"));
    }

    #[test]
    fn invalid_res_idx_safe() {
        let mut doc = new_raml_doc("API", "v1", "https://x.com");
        add_method(&mut doc, 99, "get", "Safe");
        assert_eq!(resource_count(&doc), 0);
    }

    #[test]
    fn export_contains_version() {
        let doc = new_raml_doc("API", "v2", "https://x.com");
        let s = export_raml(&doc);
        assert!(s.contains("v2"));
    }
}
