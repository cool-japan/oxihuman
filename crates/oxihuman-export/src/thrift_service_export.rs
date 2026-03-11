// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Thrift service stub export — generates Apache Thrift IDL service stubs for mesh data.

/// A Thrift field descriptor.
#[derive(Debug, Clone)]
pub struct ThriftField {
    pub field_id: i16,
    pub name: String,
    pub type_name: String,
    pub required: bool,
}

/// A Thrift struct descriptor.
#[derive(Debug, Clone)]
pub struct ThriftStruct {
    pub name: String,
    pub fields: Vec<ThriftField>,
}

/// A Thrift function (service method).
#[derive(Debug, Clone)]
pub struct ThriftFunction {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<ThriftField>,
    pub is_oneway: bool,
}

/// A Thrift service descriptor.
#[derive(Debug, Clone)]
pub struct ThriftService {
    pub name: String,
    pub functions: Vec<ThriftFunction>,
}

/// A Thrift export session.
#[derive(Debug, Default)]
pub struct ThriftServiceExport {
    pub namespace: String,
    pub structs: Vec<ThriftStruct>,
    pub services: Vec<ThriftService>,
}

/// Create a new Thrift export session.
pub fn new_thrift_service_export(namespace: &str) -> ThriftServiceExport {
    ThriftServiceExport {
        namespace: namespace.to_owned(),
        structs: Vec::new(),
        services: Vec::new(),
    }
}

/// Add a struct definition.
pub fn add_thrift_struct(export: &mut ThriftServiceExport, name: &str) {
    export.structs.push(ThriftStruct {
        name: name.to_owned(),
        fields: Vec::new(),
    });
}

/// Add a field to the last struct.
pub fn add_thrift_field(
    export: &mut ThriftServiceExport,
    id: i16,
    name: &str,
    type_name: &str,
    required: bool,
) {
    if let Some(s) = export.structs.last_mut() {
        s.fields.push(ThriftField {
            field_id: id,
            name: name.to_owned(),
            type_name: type_name.to_owned(),
            required,
        });
    }
}

/// Add a service.
pub fn add_thrift_service(export: &mut ThriftServiceExport, name: &str) {
    export.services.push(ThriftService {
        name: name.to_owned(),
        functions: Vec::new(),
    });
}

/// Add a function to the last service.
pub fn add_thrift_function(
    export: &mut ThriftServiceExport,
    name: &str,
    return_type: &str,
    oneway: bool,
) {
    if let Some(svc) = export.services.last_mut() {
        svc.functions.push(ThriftFunction {
            name: name.to_owned(),
            return_type: return_type.to_owned(),
            parameters: Vec::new(),
            is_oneway: oneway,
        });
    }
}

/// Number of structs.
pub fn thrift_struct_count(export: &ThriftServiceExport) -> usize {
    export.structs.len()
}

/// Number of services.
pub fn thrift_service_count(export: &ThriftServiceExport) -> usize {
    export.services.len()
}

/// Total functions across all services.
pub fn total_thrift_functions(export: &ThriftServiceExport) -> usize {
    export.services.iter().map(|s| s.functions.len()).sum()
}

/// Find a service by name.
pub fn find_thrift_service<'a>(
    export: &'a ThriftServiceExport,
    name: &str,
) -> Option<&'a ThriftService> {
    export.services.iter().find(|s| s.name == name)
}

/// Render a minimal IDL stub for a service.
pub fn render_thrift_service_idl(svc: &ThriftService) -> String {
    let fns: Vec<String> = svc
        .functions
        .iter()
        .map(|f| {
            let oneway = if f.is_oneway { "oneway " } else { "" };
            format!("  {}{} {}()", oneway, f.return_type, f.name)
        })
        .collect();
    format!("service {} {{\n{}\n}}", svc.name, fns.join("\n"))
}

/// Serialize to JSON-style string.
pub fn thrift_service_export_to_json(export: &ThriftServiceExport) -> String {
    format!(
        r#"{{"namespace":"{}", "struct_count":{}, "service_count":{}}}"#,
        export.namespace,
        thrift_struct_count(export),
        thrift_service_count(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        /* fresh export has no structs or services */
        let e = new_thrift_service_export("oxihuman");
        assert_eq!(thrift_struct_count(&e), 0);
        assert_eq!(thrift_service_count(&e), 0);
    }

    #[test]
    fn add_struct_increments_count() {
        /* adding a struct increases count */
        let mut e = new_thrift_service_export("oxi");
        add_thrift_struct(&mut e, "MeshData");
        assert_eq!(thrift_struct_count(&e), 1);
    }

    #[test]
    fn add_service_increments_count() {
        /* adding a service increases count */
        let mut e = new_thrift_service_export("oxi");
        add_thrift_service(&mut e, "MeshSvc");
        assert_eq!(thrift_service_count(&e), 1);
    }

    #[test]
    fn add_function_tracked() {
        /* total functions counted across services */
        let mut e = new_thrift_service_export("oxi");
        add_thrift_service(&mut e, "Svc");
        add_thrift_function(&mut e, "getMesh", "MeshData", false);
        assert_eq!(total_thrift_functions(&e), 1);
    }

    #[test]
    fn find_service_by_name() {
        /* find returns matching service */
        let mut e = new_thrift_service_export("oxi");
        add_thrift_service(&mut e, "AvatarSvc");
        assert!(find_thrift_service(&e, "AvatarSvc").is_some());
    }

    #[test]
    fn find_missing_service_none() {
        /* missing service returns None */
        let e = new_thrift_service_export("oxi");
        assert!(find_thrift_service(&e, "Ghost").is_none());
    }

    #[test]
    fn oneway_function_flag_set() {
        /* oneway function should have is_oneway true */
        let mut e = new_thrift_service_export("oxi");
        add_thrift_service(&mut e, "Svc");
        add_thrift_function(&mut e, "notify", "void", true);
        assert!(e.services[0].functions[0].is_oneway);
    }

    #[test]
    fn render_idl_contains_service_name() {
        /* rendered IDL includes the service name */
        let svc = ThriftService {
            name: "MySvc".into(),
            functions: vec![],
        };
        assert!(render_thrift_service_idl(&svc).contains("MySvc"));
    }

    #[test]
    fn json_contains_namespace() {
        /* JSON includes namespace */
        let e = new_thrift_service_export("com.oxihuman");
        assert!(thrift_service_export_to_json(&e).contains("com.oxihuman"));
    }
}
