// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! REST API schema export — generates a minimal REST endpoint schema for mesh data.

/// HTTP method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// A REST endpoint descriptor.
#[derive(Debug, Clone)]
pub struct RestEndpoint {
    pub path: String,
    pub method: RestMethod,
    pub summary: String,
    pub request_schema: Option<String>,
    pub response_schema: Option<String>,
    pub tags: Vec<String>,
}

/// A REST schema export session.
#[derive(Debug, Default)]
pub struct RestSchemaExport {
    pub endpoints: Vec<RestEndpoint>,
    pub base_url: String,
    pub api_version: String,
}

/// Create a new REST schema export.
pub fn new_rest_schema_export(base_url: &str, api_version: &str) -> RestSchemaExport {
    RestSchemaExport {
        endpoints: Vec::new(),
        base_url: base_url.to_owned(),
        api_version: api_version.to_owned(),
    }
}

/// Add an endpoint.
pub fn add_rest_endpoint(
    export: &mut RestSchemaExport,
    path: &str,
    method: RestMethod,
    summary: &str,
) {
    export.endpoints.push(RestEndpoint {
        path: path.to_owned(),
        method,
        summary: summary.to_owned(),
        request_schema: None,
        response_schema: None,
        tags: Vec::new(),
    });
}

/// Set the request schema on the last endpoint.
pub fn set_request_schema(export: &mut RestSchemaExport, schema: &str) {
    if let Some(ep) = export.endpoints.last_mut() {
        ep.request_schema = Some(schema.to_owned());
    }
}

/// Set the response schema on the last endpoint.
pub fn set_response_schema(export: &mut RestSchemaExport, schema: &str) {
    if let Some(ep) = export.endpoints.last_mut() {
        ep.response_schema = Some(schema.to_owned());
    }
}

/// Add a tag to the last endpoint.
pub fn add_endpoint_tag(export: &mut RestSchemaExport, tag: &str) {
    if let Some(ep) = export.endpoints.last_mut() {
        ep.tags.push(tag.to_owned());
    }
}

/// Number of endpoints.
pub fn rest_endpoint_count(export: &RestSchemaExport) -> usize {
    export.endpoints.len()
}

/// Count endpoints matching a given method.
pub fn endpoints_of_method(export: &RestSchemaExport, method: RestMethod) -> usize {
    export
        .endpoints
        .iter()
        .filter(|e| e.method == method)
        .count()
}

/// Find an endpoint by path and method.
pub fn find_rest_endpoint<'a>(
    export: &'a RestSchemaExport,
    path: &str,
    method: RestMethod,
) -> Option<&'a RestEndpoint> {
    export
        .endpoints
        .iter()
        .find(|e| e.path == path && e.method == method)
}

/// HTTP method name.
pub fn method_name(m: RestMethod) -> &'static str {
    match m {
        RestMethod::Get => "GET",
        RestMethod::Post => "POST",
        RestMethod::Put => "PUT",
        RestMethod::Patch => "PATCH",
        RestMethod::Delete => "DELETE",
    }
}

/// Serialize metadata to JSON-style string.
pub fn rest_schema_to_json(export: &RestSchemaExport) -> String {
    format!(
        r#"{{"base_url":"{}", "version":"{}", "endpoint_count":{}}}"#,
        export.base_url,
        export.api_version,
        rest_endpoint_count(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        /* fresh export has no endpoints */
        let e = new_rest_schema_export("https://api.example.com", "v1");
        assert_eq!(rest_endpoint_count(&e), 0);
    }

    #[test]
    fn add_endpoint_increments_count() {
        /* adding an endpoint increases count */
        let mut e = new_rest_schema_export("https://api.example.com", "v1");
        add_rest_endpoint(&mut e, "/mesh", RestMethod::Get, "Get mesh");
        assert_eq!(rest_endpoint_count(&e), 1);
    }

    #[test]
    fn endpoints_of_method_get() {
        /* GET endpoints counted separately */
        let mut e = new_rest_schema_export("https://api.example.com", "v1");
        add_rest_endpoint(&mut e, "/mesh", RestMethod::Get, "Get");
        add_rest_endpoint(&mut e, "/mesh", RestMethod::Post, "Create");
        assert_eq!(endpoints_of_method(&e, RestMethod::Get), 1);
    }

    #[test]
    fn find_endpoint_success() {
        /* find returns the matching endpoint */
        let mut e = new_rest_schema_export("https://api.example.com", "v1");
        add_rest_endpoint(&mut e, "/avatar", RestMethod::Post, "Create avatar");
        assert!(find_rest_endpoint(&e, "/avatar", RestMethod::Post).is_some());
    }

    #[test]
    fn find_endpoint_missing_returns_none() {
        /* wrong path returns None */
        let e = new_rest_schema_export("https://api.example.com", "v1");
        assert!(find_rest_endpoint(&e, "/nope", RestMethod::Get).is_none());
    }

    #[test]
    fn method_name_get() {
        /* GET method name is "GET" */
        assert_eq!(method_name(RestMethod::Get), "GET");
    }

    #[test]
    fn method_name_delete() {
        /* DELETE method name is "DELETE" */
        assert_eq!(method_name(RestMethod::Delete), "DELETE");
    }

    #[test]
    fn request_schema_stored() {
        /* request schema should be stored on last endpoint */
        let mut e = new_rest_schema_export("https://api.example.com", "v1");
        add_rest_endpoint(&mut e, "/mesh", RestMethod::Post, "Create");
        set_request_schema(&mut e, r#"{"type":"object"}"#);
        assert!(e.endpoints[0].request_schema.is_some());
    }

    #[test]
    fn json_contains_base_url() {
        /* JSON includes base_url */
        let e = new_rest_schema_export("https://mesh.api.com", "v2");
        assert!(rest_schema_to_json(&e).contains("mesh.api.com"));
    }
}
