//! JSON-RPC 2.0 stub for remote procedure call handling.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum RpcErrorCode {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
    Custom(i32),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub id: Option<u64>,
    pub params: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub result: Option<String>,
    pub error: Option<RpcError>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RpcServer {
    pub methods: Vec<String>,
    pub call_count: u64,
}

#[allow(dead_code)]
pub fn new_rpc_request(method: &str, id: Option<u64>) -> RpcRequest {
    RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        id,
        params: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn new_rpc_server() -> RpcServer {
    RpcServer {
        methods: Vec::new(),
        call_count: 0,
    }
}

#[allow(dead_code)]
pub fn register_method(server: &mut RpcServer, method: &str) {
    if !server.methods.iter().any(|m| m == method) {
        server.methods.push(method.to_string());
    }
}

#[allow(dead_code)]
pub fn handle_request(server: &mut RpcServer, req: &RpcRequest) -> RpcResponse {
    server.call_count += 1;
    if server.methods.iter().any(|m| m == &req.method) {
        rpc_success(req.id, &format!("ok:{}", req.method))
    } else {
        rpc_error_response(req.id, -32601, "Method not found")
    }
}

#[allow(dead_code)]
pub fn rpc_success(id: Option<u64>, result: &str) -> RpcResponse {
    RpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result.to_string()),
        error: None,
    }
}

#[allow(dead_code)]
pub fn rpc_error_response(id: Option<u64>, code: i32, msg: &str) -> RpcResponse {
    RpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(RpcError {
            code,
            message: msg.to_string(),
        }),
    }
}

#[allow(dead_code)]
pub fn rpc_request_to_json(req: &RpcRequest) -> String {
    let id_str = match req.id {
        Some(v) => format!("{}", v),
        None => "null".to_string(),
    };
    let params_str: Vec<String> = req.params.iter().map(|p| format!("\"{}\"", p)).collect();
    format!(
        r#"{{"jsonrpc":"{}","method":"{}","id":{},"params":[{}]}}"#,
        req.jsonrpc,
        req.method,
        id_str,
        params_str.join(",")
    )
}

#[allow(dead_code)]
pub fn rpc_response_to_json(resp: &RpcResponse) -> String {
    let id_str = match resp.id {
        Some(v) => format!("{}", v),
        None => "null".to_string(),
    };
    let result_str = match &resp.result {
        Some(r) => format!("\"{}\"", r),
        None => "null".to_string(),
    };
    let error_str = match &resp.error {
        Some(e) => format!(r#"{{"code":{},"message":"{}"}}"#, e.code, e.message),
        None => "null".to_string(),
    };
    format!(
        r#"{{"jsonrpc":"{}","id":{},"result":{},"error":{}}}"#,
        resp.jsonrpc, id_str, result_str, error_str
    )
}

#[allow(dead_code)]
pub fn method_registered(server: &RpcServer, method: &str) -> bool {
    server.methods.iter().any(|m| m == method)
}

#[allow(dead_code)]
pub fn rpc_server_call_count(server: &RpcServer) -> u64 {
    server.call_count
}

#[allow(dead_code)]
pub fn error_code_value(code: &RpcErrorCode) -> i32 {
    match code {
        RpcErrorCode::ParseError => -32700,
        RpcErrorCode::InvalidRequest => -32600,
        RpcErrorCode::MethodNotFound => -32601,
        RpcErrorCode::InvalidParams => -32602,
        RpcErrorCode::InternalError => -32603,
        RpcErrorCode::Custom(v) => *v,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_server() {
        let srv = new_rpc_server();
        assert!(srv.methods.is_empty());
        assert_eq!(srv.call_count, 0);
    }

    #[test]
    fn test_register_and_check() {
        let mut srv = new_rpc_server();
        register_method(&mut srv, "add");
        assert!(method_registered(&srv, "add"));
        assert!(!method_registered(&srv, "sub"));
    }

    #[test]
    fn test_handle_known_method() {
        let mut srv = new_rpc_server();
        register_method(&mut srv, "ping");
        let req = new_rpc_request("ping", Some(1));
        let resp = handle_request(&mut srv, &req);
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
        assert_eq!(rpc_server_call_count(&srv), 1);
    }

    #[test]
    fn test_handle_unknown_method() {
        let mut srv = new_rpc_server();
        let req = new_rpc_request("unknown", Some(2));
        let resp = handle_request(&mut srv, &req);
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
    }

    #[test]
    fn test_error_code_values() {
        assert_eq!(error_code_value(&RpcErrorCode::ParseError), -32700);
        assert_eq!(error_code_value(&RpcErrorCode::MethodNotFound), -32601);
        assert_eq!(error_code_value(&RpcErrorCode::Custom(42)), 42);
    }

    #[test]
    fn test_request_to_json() {
        let req = new_rpc_request("foo", Some(5));
        let j = rpc_request_to_json(&req);
        assert!(j.contains("\"method\":\"foo\""));
        assert!(j.contains("\"id\":5"));
    }

    #[test]
    fn test_response_to_json() {
        let resp = rpc_success(Some(3), "done");
        let j = rpc_response_to_json(&resp);
        assert!(j.contains("\"result\":\"done\""));
        assert!(j.contains("\"error\":null"));
    }
}
