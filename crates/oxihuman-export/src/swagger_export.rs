// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Swagger 2.0 spec stub export.

/// Swagger 2.0 document info.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SwaggerInfo {
    pub title: String,
    pub version: String,
    pub description: String,
}

/// A Swagger path operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SwaggerOperation {
    pub path: String,
    pub method: String,
    pub summary: String,
    pub tags: Vec<String>,
}

/// A Swagger 2.0 document.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SwaggerDoc {
    pub info: SwaggerInfo,
    pub host: String,
    pub base_path: String,
    pub operations: Vec<SwaggerOperation>,
}

/// Create a new Swagger 2.0 document.
#[allow(dead_code)]
pub fn new_swagger_doc(title: &str, version: &str, host: &str) -> SwaggerDoc {
    SwaggerDoc {
        info: SwaggerInfo {
            title: title.to_string(),
            version: version.to_string(),
            description: String::new(),
        },
        host: host.to_string(),
        base_path: "/".to_string(),
        operations: Vec::new(),
    }
}

/// Add an operation to the swagger doc.
#[allow(dead_code)]
pub fn add_operation(doc: &mut SwaggerDoc, path: &str, method: &str, summary: &str) {
    doc.operations.push(SwaggerOperation {
        path: path.to_string(),
        method: method.to_string(),
        summary: summary.to_string(),
        tags: Vec::new(),
    });
}

/// Add a tag to the last operation.
#[allow(dead_code)]
pub fn add_tag(doc: &mut SwaggerDoc, tag: &str) {
    if let Some(op) = doc.operations.last_mut() {
        op.tags.push(tag.to_string());
    }
}

/// Export to JSON text (Swagger 2.0 format).
#[allow(dead_code)]
pub fn export_swagger_json(doc: &SwaggerDoc) -> String {
    let mut paths = String::new();
    for (i, op) in doc.operations.iter().enumerate() {
        let tags: Vec<String> = op.tags.iter().map(|t| format!("\"{}\"", t)).collect();
        let entry = format!(
            r#""{}":{{"{}": {{"summary":"{}","tags":[{}]}}}}"#,
            op.path,
            op.method,
            op.summary,
            tags.join(",")
        );
        if i > 0 {
            paths.push(',');
        }
        paths.push_str(&entry);
    }
    format!(
        r#"{{"swagger":"2.0","info":{{"title":"{}","version":"{}"}},"host":"{}","basePath":"{}","paths":{{{}}}}}"#,
        doc.info.title, doc.info.version, doc.host, doc.base_path, paths
    )
}

/// Operation count.
#[allow(dead_code)]
pub fn operation_count(doc: &SwaggerDoc) -> usize {
    doc.operations.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_doc_title() {
        let doc = new_swagger_doc("My API", "1.0", "api.example.com");
        assert_eq!(doc.info.title, "My API");
    }

    #[test]
    fn new_doc_host() {
        let doc = new_swagger_doc("API", "1.0", "localhost:8080");
        assert_eq!(doc.host, "localhost:8080");
    }

    #[test]
    fn add_operation_count() {
        let mut doc = new_swagger_doc("API", "1.0", "host");
        add_operation(&mut doc, "/pets", "get", "List pets");
        assert_eq!(operation_count(&doc), 1);
    }

    #[test]
    fn export_contains_swagger_version() {
        let doc = new_swagger_doc("API", "1.0", "host");
        let s = export_swagger_json(&doc);
        assert!(s.contains("\"2.0\""));
    }

    #[test]
    fn export_contains_title() {
        let doc = new_swagger_doc("PetStore", "1.0", "host");
        let s = export_swagger_json(&doc);
        assert!(s.contains("PetStore"));
    }

    #[test]
    fn export_contains_path() {
        let mut doc = new_swagger_doc("API", "1.0", "host");
        add_operation(&mut doc, "/pets", "get", "List");
        let s = export_swagger_json(&doc);
        assert!(s.contains("/pets"));
    }

    #[test]
    fn add_tag_stored() {
        let mut doc = new_swagger_doc("API", "1.0", "host");
        add_operation(&mut doc, "/pets", "get", "List");
        add_tag(&mut doc, "pets");
        assert_eq!(doc.operations[0].tags.len(), 1);
    }

    #[test]
    fn tag_in_export() {
        let mut doc = new_swagger_doc("API", "1.0", "host");
        add_operation(&mut doc, "/pets", "get", "List");
        add_tag(&mut doc, "pets");
        let s = export_swagger_json(&doc);
        assert!(s.contains("pets"));
    }

    #[test]
    fn add_tag_no_operations_safe() {
        let mut doc = new_swagger_doc("API", "1.0", "host");
        add_tag(&mut doc, "safe");
        assert_eq!(operation_count(&doc), 0);
    }

    #[test]
    fn base_path_default() {
        let doc = new_swagger_doc("API", "1.0", "host");
        assert_eq!(doc.base_path, "/");
    }
}
