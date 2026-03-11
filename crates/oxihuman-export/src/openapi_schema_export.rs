// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! OpenAPI 3.0 spec export stub.

use std::collections::BTreeMap;

/// HTTP method.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl HttpMethod {
    /// Lowercase method name string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Put => "put",
            Self::Delete => "delete",
            Self::Patch => "patch",
        }
    }
}

/// An OpenAPI path operation.
#[derive(Debug, Clone)]
pub struct ApiOperation {
    pub method: HttpMethod,
    pub summary: String,
    pub operation_id: String,
    pub tags: Vec<String>,
    pub response_codes: Vec<u16>,
}

/// An OpenAPI path item.
#[derive(Debug, Clone, Default)]
pub struct ApiPath {
    pub operations: BTreeMap<String, ApiOperation>,
}

impl ApiPath {
    /// Add an operation.
    pub fn add_operation(&mut self, method: HttpMethod, op: ApiOperation) {
        self.operations.insert(method.as_str().to_string(), op);
    }

    /// Number of operations.
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }
}

/// An OpenAPI info block.
#[derive(Debug, Clone)]
pub struct ApiInfo {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

/// An OpenAPI 3.0 document.
#[derive(Debug, Clone, Default)]
pub struct OpenApiSpec {
    pub info: Option<ApiInfo>,
    pub paths: BTreeMap<String, ApiPath>,
    pub servers: Vec<String>,
}

impl OpenApiSpec {
    /// Add a path.
    pub fn add_path(&mut self, path: impl Into<String>, item: ApiPath) {
        self.paths.insert(path.into(), item);
    }

    /// Number of paths.
    pub fn path_count(&self) -> usize {
        self.paths.len()
    }

    /// Find a path item.
    pub fn find_path(&self, path: &str) -> Option<&ApiPath> {
        self.paths.get(path)
    }
}

/// Render the spec as a minimal JSON string.
pub fn render_openapi_json(spec: &OpenApiSpec) -> String {
    let title = spec
        .info
        .as_ref()
        .map(|i| i.title.as_str())
        .unwrap_or("API");
    let version = spec
        .info
        .as_ref()
        .map(|i| i.version.as_str())
        .unwrap_or("1.0.0");
    let paths_json: Vec<String> = spec
        .paths
        .iter()
        .map(|(path, item)| {
            let ops: Vec<String> = item
                .operations
                .iter()
                .map(|(method, op)| {
                    format!(
                        r#""{method}":{{"summary":"{}","operationId":"{}"}}"#,
                        op.summary, op.operation_id
                    )
                })
                .collect();
            format!(r#""{path}":{{{}}}  "#, ops.join(","))
        })
        .collect();
    format!(
        r#"{{"openapi":"3.0.0","info":{{"title":"{title}","version":"{version}"}},"paths":{{{}}}}}"#,
        paths_json.join(",")
    )
}

/// Validate spec (must have info and at least one path).
pub fn validate_spec(spec: &OpenApiSpec) -> bool {
    spec.info.is_some() && !spec.paths.is_empty()
}

/// Count total operations across all paths.
pub fn total_operation_count(spec: &OpenApiSpec) -> usize {
    spec.paths.values().map(|p| p.operation_count()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_spec() -> OpenApiSpec {
        let mut spec = OpenApiSpec {
            info: Some(ApiInfo {
                title: "Test API".into(),
                version: "1.0.0".into(),
                description: None,
            }),
            ..Default::default()
        };
        let mut path = ApiPath::default();
        path.add_operation(
            HttpMethod::Get,
            ApiOperation {
                method: HttpMethod::Get,
                summary: "List things".into(),
                operation_id: "listThings".into(),
                tags: vec!["things".into()],
                response_codes: vec![200],
            },
        );
        spec.add_path("/things", path);
        spec
    }

    #[test]
    fn path_count() {
        assert_eq!(sample_spec().path_count(), 1);
    }

    #[test]
    fn find_path_found() {
        assert!(sample_spec().find_path("/things").is_some());
    }

    #[test]
    fn operation_count() {
        let spec = sample_spec();
        let p = spec.find_path("/things").unwrap();
        assert_eq!(p.operation_count(), 1);
    }

    #[test]
    fn render_contains_openapi_version() {
        assert!(render_openapi_json(&sample_spec()).contains("3.0.0"));
    }

    #[test]
    fn render_contains_title() {
        assert!(render_openapi_json(&sample_spec()).contains("Test API"));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_spec(&sample_spec()));
    }

    #[test]
    fn validate_no_info() {
        let spec = OpenApiSpec::default();
        assert!(!validate_spec(&spec));
    }

    #[test]
    fn total_operations() {
        assert_eq!(total_operation_count(&sample_spec()), 1);
    }

    #[test]
    fn method_as_str() {
        assert_eq!(HttpMethod::Post.as_str(), "post");
    }
}
