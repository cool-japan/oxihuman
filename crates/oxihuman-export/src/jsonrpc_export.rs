// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! JSON-RPC 2.0 request/response serialization.

/// A JSON-RPC 2.0 request.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: String,
    pub id: Option<i64>,
}

/// A JSON-RPC 2.0 response.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<String>,
    pub error: Option<JsonRpcError>,
    pub id: Option<i64>,
}

/// A JSON-RPC 2.0 error object.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

/// Build a JSON-RPC request.
#[allow(dead_code)]
pub fn new_jsonrpc_request(method: &str, params_json: &str, id: Option<i64>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params: params_json.to_string(),
        id,
    }
}

/// Build a successful JSON-RPC response.
#[allow(dead_code)]
pub fn new_jsonrpc_result(result_json: &str, id: Option<i64>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result_json.to_string()),
        error: None,
        id,
    }
}

/// Build an error JSON-RPC response.
#[allow(dead_code)]
pub fn new_jsonrpc_error(code: i32, message: &str, id: Option<i64>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.to_string(),
        }),
        id,
    }
}

/// Serialize a request to JSON text.
#[allow(dead_code)]
pub fn serialize_request(req: &JsonRpcRequest) -> String {
    let id_part = match req.id {
        Some(id) => id.to_string(),
        None => "null".to_string(),
    };
    format!(
        r#"{{"jsonrpc":"{}","method":"{}","params":{},"id":{}}}"#,
        req.jsonrpc, req.method, req.params, id_part
    )
}

/// Serialize a response to JSON text.
#[allow(dead_code)]
pub fn serialize_response(resp: &JsonRpcResponse) -> String {
    let id_part = match resp.id {
        Some(id) => id.to_string(),
        None => "null".to_string(),
    };
    if let Some(ref result) = resp.result {
        format!(
            r#"{{"jsonrpc":"{}","result":{},"id":{}}}"#,
            resp.jsonrpc, result, id_part
        )
    } else if let Some(ref err) = resp.error {
        format!(
            r#"{{"jsonrpc":"{}","error":{{"code":{},"message":"{}"}},"id":{}}}"#,
            resp.jsonrpc, err.code, err.message, id_part
        )
    } else {
        format!(r#"{{"jsonrpc":"{}","id":{}}}"#, resp.jsonrpc, id_part)
    }
}

/// Check if a response is successful.
#[allow(dead_code)]
pub fn is_success(resp: &JsonRpcResponse) -> bool {
    resp.result.is_some() && resp.error.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_version_is_2_0() {
        let req = new_jsonrpc_request("add", "[1,2]", Some(1));
        assert_eq!(req.jsonrpc, "2.0");
    }

    #[test]
    fn request_method_stored() {
        let req = new_jsonrpc_request("ping", "{}", None);
        assert_eq!(req.method, "ping");
    }

    #[test]
    fn serialize_request_contains_method() {
        let req = new_jsonrpc_request("ping", "null", Some(1));
        let s = serialize_request(&req);
        assert!(s.contains("ping"));
    }

    #[test]
    fn serialize_request_null_id() {
        let req = new_jsonrpc_request("ping", "null", None);
        let s = serialize_request(&req);
        assert!(s.contains("\"id\":null"));
    }

    #[test]
    fn success_response_is_success() {
        let resp = new_jsonrpc_result("42", Some(1));
        assert!(is_success(&resp));
    }

    #[test]
    fn error_response_not_success() {
        let resp = new_jsonrpc_error(-32600, "Invalid Request", Some(1));
        assert!(!is_success(&resp));
    }

    #[test]
    fn serialize_response_contains_result() {
        let resp = new_jsonrpc_result("99", Some(1));
        let s = serialize_response(&resp);
        assert!(s.contains("result"));
    }

    #[test]
    fn serialize_error_contains_code() {
        let resp = new_jsonrpc_error(-32600, "Bad", Some(1));
        let s = serialize_response(&resp);
        assert!(s.contains("-32600"));
    }

    #[test]
    fn error_code_stored() {
        let resp = new_jsonrpc_error(-32700, "Parse error", None);
        assert_eq!(resp.error.as_ref().expect("should succeed").code, -32700);
    }

    #[test]
    fn id_none_null_in_output() {
        let resp = new_jsonrpc_result("true", None);
        let s = serialize_response(&resp);
        assert!(s.contains("null"));
    }
}
