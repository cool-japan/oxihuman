// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! GraphQL query export stub — generates GraphQL query/mutation documents for mesh data.

/// GraphQL operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GqlQueryOpType {
    Query,
    Mutation,
    Subscription,
}

/// A GraphQL variable definition.
#[derive(Debug, Clone)]
pub struct GqlQueryVar {
    pub name: String,
    pub gql_type: String,
    pub default_value: Option<String>,
}

/// A GraphQL operation stub.
#[derive(Debug, Clone)]
pub struct GqlQueryOp {
    pub op_type: GqlQueryOpType,
    pub name: String,
    pub variables: Vec<GqlQueryVar>,
    pub body: String,
}

/// A GraphQL query export session.
#[derive(Debug, Default)]
pub struct GraphQlQueryExport {
    pub operations: Vec<GqlQueryOp>,
    pub endpoint: String,
}

/// Create a new GraphQL query export.
pub fn new_graphql_query_export(endpoint: &str) -> GraphQlQueryExport {
    GraphQlQueryExport {
        operations: Vec::new(),
        endpoint: endpoint.to_owned(),
    }
}

/// Add a query operation.
pub fn add_gql_query(export: &mut GraphQlQueryExport, name: &str, body: &str) {
    export.operations.push(GqlQueryOp {
        op_type: GqlQueryOpType::Query,
        name: name.to_owned(),
        variables: Vec::new(),
        body: body.to_owned(),
    });
}

/// Add a mutation operation.
pub fn add_gql_mutation(export: &mut GraphQlQueryExport, name: &str, body: &str) {
    export.operations.push(GqlQueryOp {
        op_type: GqlQueryOpType::Mutation,
        name: name.to_owned(),
        variables: Vec::new(),
        body: body.to_owned(),
    });
}

/// Add a variable to the last operation.
pub fn add_gql_query_var(
    export: &mut GraphQlQueryExport,
    name: &str,
    gql_type: &str,
    default_value: Option<&str>,
) {
    if let Some(op) = export.operations.last_mut() {
        op.variables.push(GqlQueryVar {
            name: name.to_owned(),
            gql_type: gql_type.to_owned(),
            default_value: default_value.map(str::to_owned),
        });
    }
}

/// Number of operations.
pub fn gql_query_op_count(export: &GraphQlQueryExport) -> usize {
    export.operations.len()
}

/// Count operations of a given type.
pub fn ops_of_type(export: &GraphQlQueryExport, op_type: GqlQueryOpType) -> usize {
    export
        .operations
        .iter()
        .filter(|o| o.op_type == op_type)
        .count()
}

/// Find an operation by name.
pub fn find_gql_query_op<'a>(export: &'a GraphQlQueryExport, name: &str) -> Option<&'a GqlQueryOp> {
    export.operations.iter().find(|o| o.name == name)
}

/// Render an operation to a GraphQL document string.
pub fn render_gql_query_op(op: &GqlQueryOp) -> String {
    let op_str = match op.op_type {
        GqlQueryOpType::Query => "query",
        GqlQueryOpType::Mutation => "mutation",
        GqlQueryOpType::Subscription => "subscription",
    };
    if op.variables.is_empty() {
        format!("{} {} {{\n  {}\n}}", op_str, op.name, op.body)
    } else {
        let vars: Vec<String> = op
            .variables
            .iter()
            .map(|v| format!("${}: {}", v.name, v.gql_type))
            .collect();
        format!(
            "{} {}({}) {{\n  {}\n}}",
            op_str,
            op.name,
            vars.join(", "),
            op.body
        )
    }
}

/// Serialize metadata to JSON-style string.
pub fn graphql_query_export_to_json(export: &GraphQlQueryExport) -> String {
    format!(
        r#"{{"endpoint":"{}", "operation_count":{}}}"#,
        export.endpoint,
        gql_query_op_count(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        /* fresh export has no operations */
        let e = new_graphql_query_export("https://api.example.com/graphql");
        assert_eq!(gql_query_op_count(&e), 0);
    }

    #[test]
    fn add_query_increments_count() {
        /* adding a query increases count */
        let mut e = new_graphql_query_export("https://api.example.com/graphql");
        add_gql_query(&mut e, "GetMesh", "mesh { id positions }");
        assert_eq!(gql_query_op_count(&e), 1);
    }

    #[test]
    fn add_mutation_increments_count() {
        /* adding a mutation increases count */
        let mut e = new_graphql_query_export("https://api.example.com/graphql");
        add_gql_mutation(&mut e, "UpdateMesh", "updateMesh(id: 1) { id }");
        assert_eq!(gql_query_op_count(&e), 1);
    }

    #[test]
    fn ops_of_type_query() {
        /* query ops counted separately */
        let mut e = new_graphql_query_export("https://api.example.com/graphql");
        add_gql_query(&mut e, "Q", "field");
        add_gql_mutation(&mut e, "M", "field");
        assert_eq!(ops_of_type(&e, GqlQueryOpType::Query), 1);
    }

    #[test]
    fn find_op_by_name() {
        /* find returns matching op */
        let mut e = new_graphql_query_export("https://api.example.com/graphql");
        add_gql_query(&mut e, "GetAvatar", "avatar { id }");
        assert!(find_gql_query_op(&e, "GetAvatar").is_some());
    }

    #[test]
    fn find_missing_op_none() {
        /* missing operation returns None */
        let e = new_graphql_query_export("https://api.example.com/graphql");
        assert!(find_gql_query_op(&e, "Ghost").is_none());
    }

    #[test]
    fn render_query_contains_query_keyword() {
        /* rendered query includes "query" keyword */
        let op = GqlQueryOp {
            op_type: GqlQueryOpType::Query,
            name: "TestQ".into(),
            variables: vec![],
            body: "mesh { id }".into(),
        };
        assert!(render_gql_query_op(&op).contains("query"));
    }

    #[test]
    fn render_mutation_contains_mutation_keyword() {
        /* rendered mutation includes "mutation" keyword */
        let op = GqlQueryOp {
            op_type: GqlQueryOpType::Mutation,
            name: "TestM".into(),
            variables: vec![],
            body: "updateMesh { ok }".into(),
        };
        assert!(render_gql_query_op(&op).contains("mutation"));
    }

    #[test]
    fn json_contains_endpoint() {
        /* JSON includes endpoint */
        let e = new_graphql_query_export("https://gql.example.com");
        assert!(graphql_query_export_to_json(&e).contains("gql.example.com"));
    }
}
