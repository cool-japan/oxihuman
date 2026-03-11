// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! gRPC service stub export — generates proto3 service description stubs for mesh data.

/// A gRPC method descriptor.
#[derive(Debug, Clone)]
pub struct GrpcMethod {
    pub name: String,
    pub request_type: String,
    pub response_type: String,
    pub client_streaming: bool,
    pub server_streaming: bool,
}

/// A gRPC service descriptor.
#[derive(Debug, Clone)]
pub struct GrpcService {
    pub name: String,
    pub package: String,
    pub methods: Vec<GrpcMethod>,
}

/// A gRPC export session.
#[derive(Debug, Default)]
pub struct GrpcServiceExport {
    pub services: Vec<GrpcService>,
}

/// Create a new gRPC service export.
pub fn new_grpc_service_export() -> GrpcServiceExport {
    GrpcServiceExport::default()
}

/// Add a service.
pub fn add_grpc_service(export: &mut GrpcServiceExport, name: &str, package: &str) {
    export.services.push(GrpcService {
        name: name.to_owned(),
        package: package.to_owned(),
        methods: Vec::new(),
    });
}

/// Add a unary method to the last service.
pub fn add_grpc_unary_method(export: &mut GrpcServiceExport, name: &str, req: &str, resp: &str) {
    if let Some(svc) = export.services.last_mut() {
        svc.methods.push(GrpcMethod {
            name: name.to_owned(),
            request_type: req.to_owned(),
            response_type: resp.to_owned(),
            client_streaming: false,
            server_streaming: false,
        });
    }
}

/// Add a server-streaming method to the last service.
pub fn add_grpc_server_stream_method(
    export: &mut GrpcServiceExport,
    name: &str,
    req: &str,
    resp: &str,
) {
    if let Some(svc) = export.services.last_mut() {
        svc.methods.push(GrpcMethod {
            name: name.to_owned(),
            request_type: req.to_owned(),
            response_type: resp.to_owned(),
            client_streaming: false,
            server_streaming: true,
        });
    }
}

/// Number of services.
pub fn grpc_service_count(export: &GrpcServiceExport) -> usize {
    export.services.len()
}

/// Total method count across all services.
pub fn total_grpc_methods(export: &GrpcServiceExport) -> usize {
    export.services.iter().map(|s| s.methods.len()).sum()
}

/// Find a service by name.
pub fn find_grpc_service<'a>(export: &'a GrpcServiceExport, name: &str) -> Option<&'a GrpcService> {
    export.services.iter().find(|s| s.name == name)
}

/// Render a proto3 service block stub.
pub fn render_proto3_service(svc: &GrpcService) -> String {
    let methods: Vec<String> = svc
        .methods
        .iter()
        .map(|m| {
            let req = if m.client_streaming {
                format!("stream {}", m.request_type)
            } else {
                m.request_type.clone()
            };
            let resp = if m.server_streaming {
                format!("stream {}", m.response_type)
            } else {
                m.response_type.clone()
            };
            format!("  rpc {} ({}) returns ({});", m.name, req, resp)
        })
        .collect();
    format!("service {} {{\n{}\n}}", svc.name, methods.join("\n"))
}

/// Serialize to JSON-style string.
pub fn grpc_service_export_to_json(export: &GrpcServiceExport) -> String {
    format!(
        r#"{{"service_count":{}, "total_methods":{}}}"#,
        grpc_service_count(export),
        total_grpc_methods(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_has_no_services() {
        /* fresh export has no services */
        let e = new_grpc_service_export();
        assert_eq!(grpc_service_count(&e), 0);
    }

    #[test]
    fn add_service_increments_count() {
        /* adding a service increases count */
        let mut e = new_grpc_service_export();
        add_grpc_service(&mut e, "MeshService", "oxihuman");
        assert_eq!(grpc_service_count(&e), 1);
    }

    #[test]
    fn add_method_to_service() {
        /* adding a method increases total method count */
        let mut e = new_grpc_service_export();
        add_grpc_service(&mut e, "MeshService", "oxihuman");
        add_grpc_unary_method(&mut e, "GetMesh", "MeshRequest", "MeshResponse");
        assert_eq!(total_grpc_methods(&e), 1);
    }

    #[test]
    fn find_service_by_name() {
        /* find returns matching service */
        let mut e = new_grpc_service_export();
        add_grpc_service(&mut e, "AvatarService", "oxi");
        assert!(find_grpc_service(&e, "AvatarService").is_some());
    }

    #[test]
    fn find_missing_service_none() {
        /* missing service name returns None */
        let e = new_grpc_service_export();
        assert!(find_grpc_service(&e, "Ghost").is_none());
    }

    #[test]
    fn server_stream_method_flag_set() {
        /* server streaming method should have flag true */
        let mut e = new_grpc_service_export();
        add_grpc_service(&mut e, "S", "p");
        add_grpc_server_stream_method(&mut e, "StreamMesh", "Req", "MeshFrame");
        assert!(e.services[0].methods[0].server_streaming);
    }

    #[test]
    fn unary_method_has_no_streaming() {
        /* unary method should have both streaming flags false */
        let mut e = new_grpc_service_export();
        add_grpc_service(&mut e, "S", "p");
        add_grpc_unary_method(&mut e, "Get", "Req", "Resp");
        let m = &e.services[0].methods[0];
        assert!(!m.client_streaming && !m.server_streaming);
    }

    #[test]
    fn render_proto3_contains_service_name() {
        /* rendered proto3 stub includes the service name */
        let svc = GrpcService {
            name: "TestSvc".into(),
            package: "p".into(),
            methods: vec![],
        };
        assert!(render_proto3_service(&svc).contains("TestSvc"));
    }

    #[test]
    fn json_contains_service_count() {
        /* JSON includes service_count */
        let e = new_grpc_service_export();
        assert!(grpc_service_export_to_json(&e).contains("service_count"));
    }
}
