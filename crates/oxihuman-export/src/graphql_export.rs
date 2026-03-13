// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! GraphQL query/mutation text export.

/// GraphQL operation type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GqlOpType {
    Query,
    Mutation,
    Subscription,
}

impl GqlOpType {
    fn as_str(self) -> &'static str {
        match self {
            GqlOpType::Query => "query",
            GqlOpType::Mutation => "mutation",
            GqlOpType::Subscription => "subscription",
        }
    }
}

/// A GraphQL variable definition.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GqlVar {
    pub name: String,
    pub gql_type: String,
}

/// A GraphQL operation stub.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GqlOperation {
    pub op_type: GqlOpType,
    pub name: String,
    pub variables: Vec<GqlVar>,
    pub selection: String,
}

/// Build a new GraphQL query.
#[allow(dead_code)]
pub fn new_query(name: &str, selection: &str) -> GqlOperation {
    GqlOperation {
        op_type: GqlOpType::Query,
        name: name.to_string(),
        variables: Vec::new(),
        selection: selection.to_string(),
    }
}

/// Build a new GraphQL mutation.
#[allow(dead_code)]
pub fn new_mutation(name: &str, selection: &str) -> GqlOperation {
    GqlOperation {
        op_type: GqlOpType::Mutation,
        name: name.to_string(),
        variables: Vec::new(),
        selection: selection.to_string(),
    }
}

/// Add a variable to an operation.
#[allow(dead_code)]
pub fn add_variable(op: &mut GqlOperation, name: &str, gql_type: &str) {
    op.variables.push(GqlVar {
        name: name.to_string(),
        gql_type: gql_type.to_string(),
    });
}

/// Serialize a GraphQL operation to text.
#[allow(dead_code)]
pub fn serialize_gql(op: &GqlOperation) -> String {
    let vars = if op.variables.is_empty() {
        String::new()
    } else {
        let parts: Vec<String> = op
            .variables
            .iter()
            .map(|v| format!("${}: {}", v.name, v.gql_type))
            .collect();
        format!("({})", parts.join(", "))
    };
    format!(
        "{} {}{} {{\n  {}\n}}",
        op.op_type.as_str(),
        op.name,
        vars,
        op.selection
    )
}

/// Variable count.
#[allow(dead_code)]
pub fn var_count(op: &GqlOperation) -> usize {
    op.variables.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_op_type() {
        let op = new_query("GetUser", "id name");
        assert_eq!(op.op_type, GqlOpType::Query);
    }

    #[test]
    fn mutation_op_type() {
        let op = new_mutation("CreateUser", "id");
        assert_eq!(op.op_type, GqlOpType::Mutation);
    }

    #[test]
    fn serialize_contains_query_keyword() {
        let op = new_query("GetUser", "id name");
        let s = serialize_gql(&op);
        assert!(s.starts_with("query"));
    }

    #[test]
    fn serialize_contains_name() {
        let op = new_query("GetUser", "id name");
        let s = serialize_gql(&op);
        assert!(s.contains("GetUser"));
    }

    #[test]
    fn serialize_contains_selection() {
        let op = new_query("X", "id name email");
        let s = serialize_gql(&op);
        assert!(s.contains("id name email"));
    }

    #[test]
    fn add_variable_increases_count() {
        let mut op = new_query("X", "id");
        add_variable(&mut op, "userId", "ID!");
        assert_eq!(var_count(&op), 1);
    }

    #[test]
    fn variable_in_output() {
        let mut op = new_query("GetUser", "id");
        add_variable(&mut op, "userId", "ID!");
        let s = serialize_gql(&op);
        assert!(s.contains("$userId: ID!"));
    }

    #[test]
    fn no_vars_no_parens() {
        let op = new_query("X", "id");
        let s = serialize_gql(&op);
        assert!(!s.contains('('));
    }

    #[test]
    fn mutation_keyword_in_output() {
        let op = new_mutation("CreateUser", "id");
        let s = serialize_gql(&op);
        assert!(s.starts_with("mutation"));
    }

    #[test]
    fn subscription_keyword() {
        let op = GqlOperation {
            op_type: GqlOpType::Subscription,
            name: "OnUpdate".to_string(),
            variables: Vec::new(),
            selection: "id".to_string(),
        };
        assert!(serialize_gql(&op).starts_with("subscription"));
    }

    #[test]
    fn braces_in_output() {
        let op = new_query("X", "id");
        let s = serialize_gql(&op);
        assert!(s.contains('{') && s.contains('}'));
    }
}
