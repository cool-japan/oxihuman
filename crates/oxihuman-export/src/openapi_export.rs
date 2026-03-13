// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! OpenAPI 3.0 schema stub export (JSON).

/// An OpenAPI 3.0 info block.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
    pub description: String,
}

/// An OpenAPI parameter.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OpenApiParam {
    pub name: String,
    pub r#in: String,
    pub required: bool,
    pub schema_type: String,
}

/// An OpenAPI path item.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OpenApiPath {
    pub path: String,
    pub method: String,
    pub summary: String,
    pub parameters: Vec<OpenApiParam>,
    pub response_schema: String,
}

/// An OpenAPI 3.0 document.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OpenApiDoc {
    pub info: OpenApiInfo,
    pub paths: Vec<OpenApiPath>,
}

/// Create a new OpenAPI document.
#[allow(dead_code)]
pub fn new_openapi_doc(title: &str, version: &str) -> OpenApiDoc {
    OpenApiDoc {
        info: OpenApiInfo {
            title: title.to_string(),
            version: version.to_string(),
            description: String::new(),
        },
        paths: Vec::new(),
    }
}

/// Add a path to the document.
#[allow(dead_code)]
pub fn add_path(doc: &mut OpenApiDoc, path: &str, method: &str, summary: &str) -> usize {
    doc.paths.push(OpenApiPath {
        path: path.to_string(),
        method: method.to_string(),
        summary: summary.to_string(),
        parameters: Vec::new(),
        response_schema: "object".to_string(),
    });
    doc.paths.len() - 1
}

/// Add a parameter to a path.
#[allow(dead_code)]
pub fn add_param(
    doc: &mut OpenApiDoc,
    path_idx: usize,
    name: &str,
    location: &str,
    required: bool,
    schema_type: &str,
) {
    if path_idx < doc.paths.len() {
        doc.paths[path_idx].parameters.push(OpenApiParam {
            name: name.to_string(),
            r#in: location.to_string(),
            required,
            schema_type: schema_type.to_string(),
        });
    }
}

/// Export to JSON text.
#[allow(dead_code)]
pub fn export_openapi_json(doc: &OpenApiDoc) -> String {
    let mut paths_json = String::new();
    for (i, p) in doc.paths.iter().enumerate() {
        let params: Vec<String> = p
            .parameters
            .iter()
            .map(|param| {
                format!(
                    r#"{{"name":"{}","in":"{}","required":{},"schema":{{"type":"{}"}}}}"#,
                    param.name, param.r#in, param.required, param.schema_type
                )
            })
            .collect();
        let params_arr = format!("[{}]", params.join(","));
        let path_entry = format!(
            r#""{}":{{"{}": {{"summary":"{}","parameters":{},"responses":{{"200":{{"description":"OK"}}}}}}}}"#,
            p.path, p.method, p.summary, params_arr
        );
        if i > 0 {
            paths_json.push(',');
        }
        paths_json.push_str(&path_entry);
    }
    format!(
        r#"{{"openapi":"3.0.0","info":{{"title":"{}","version":"{}"}},"paths":{{{}}}}}"#,
        doc.info.title, doc.info.version, paths_json
    )
}

/// Path count.
#[allow(dead_code)]
pub fn path_count(doc: &OpenApiDoc) -> usize {
    doc.paths.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_doc_title() {
        let doc = new_openapi_doc("My API", "1.0");
        assert_eq!(doc.info.title, "My API");
    }

    #[test]
    fn add_path_increases_count() {
        let mut doc = new_openapi_doc("API", "1.0");
        add_path(&mut doc, "/users", "get", "List users");
        assert_eq!(path_count(&doc), 1);
    }

    #[test]
    fn export_contains_openapi_version() {
        let doc = new_openapi_doc("API", "1.0");
        let s = export_openapi_json(&doc);
        assert!(s.contains("3.0.0"));
    }

    #[test]
    fn export_contains_title() {
        let doc = new_openapi_doc("My API", "1.0");
        let s = export_openapi_json(&doc);
        assert!(s.contains("My API"));
    }

    #[test]
    fn export_contains_path() {
        let mut doc = new_openapi_doc("API", "1.0");
        add_path(&mut doc, "/users", "get", "Get users");
        let s = export_openapi_json(&doc);
        assert!(s.contains("/users"));
    }

    #[test]
    fn add_param_to_path() {
        let mut doc = new_openapi_doc("API", "1.0");
        let idx = add_path(&mut doc, "/user", "get", "Get user");
        add_param(&mut doc, idx, "id", "query", true, "integer");
        assert_eq!(doc.paths[0].parameters.len(), 1);
    }

    #[test]
    fn param_in_export() {
        let mut doc = new_openapi_doc("API", "1.0");
        let idx = add_path(&mut doc, "/user", "get", "Get user");
        add_param(&mut doc, idx, "id", "query", true, "integer");
        let s = export_openapi_json(&doc);
        assert!(s.contains("\"id\""));
    }

    #[test]
    fn invalid_path_idx_ignored() {
        let mut doc = new_openapi_doc("API", "1.0");
        add_param(&mut doc, 99, "x", "query", false, "string");
        assert_eq!(path_count(&doc), 0);
    }

    #[test]
    fn empty_paths_json() {
        let doc = new_openapi_doc("API", "1.0");
        let s = export_openapi_json(&doc);
        assert!(s.contains("paths"));
    }

    #[test]
    fn version_in_export() {
        let doc = new_openapi_doc("API", "2.5.0");
        let s = export_openapi_json(&doc);
        assert!(s.contains("2.5.0"));
    }
}
