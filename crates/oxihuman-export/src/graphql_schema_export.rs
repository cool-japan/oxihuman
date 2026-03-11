// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! GraphQL schema stub export (SDL format).

/// A GraphQL field type reference.
#[derive(Debug, Clone)]
pub struct GqlFieldType {
    pub base_type: String,
    pub non_null: bool,
    pub is_list: bool,
}

impl GqlFieldType {
    /// Render as SDL type string, e.g. `[String!]!`.
    pub fn to_sdl(&self) -> String {
        let inner = if self.non_null {
            format!("{}!", self.base_type)
        } else {
            self.base_type.clone()
        };
        if self.is_list {
            format!("[{inner}]")
        } else {
            inner
        }
    }
}

/// A field in a GraphQL type.
#[derive(Debug, Clone)]
pub struct GqlField {
    pub name: String,
    pub field_type: GqlFieldType,
    pub description: Option<String>,
}

/// A GraphQL object type.
#[derive(Debug, Clone)]
pub struct GqlObjectType {
    pub name: String,
    pub fields: Vec<GqlField>,
    pub implements: Vec<String>,
    pub description: Option<String>,
}

impl GqlObjectType {
    /// Create a new object type.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            implements: Vec::new(),
            description: None,
        }
    }

    /// Add a field.
    pub fn add_field(
        &mut self,
        name: impl Into<String>,
        base_type: impl Into<String>,
        non_null: bool,
    ) {
        self.fields.push(GqlField {
            name: name.into(),
            field_type: GqlFieldType {
                base_type: base_type.into(),
                non_null,
                is_list: false,
            },
            description: None,
        });
    }
}

/// A GraphQL schema.
#[derive(Debug, Clone, Default)]
pub struct GqlSchema {
    pub types: Vec<GqlObjectType>,
    pub query_type: Option<String>,
    pub mutation_type: Option<String>,
}

impl GqlSchema {
    /// Add a type.
    pub fn add_type(&mut self, t: GqlObjectType) {
        self.types.push(t);
    }

    /// Number of types.
    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    /// Find type by name.
    pub fn find_type(&self, name: &str) -> Option<&GqlObjectType> {
        self.types.iter().find(|t| t.name == name)
    }
}

/// Render a GqlObjectType as SDL.
pub fn render_type_sdl(t: &GqlObjectType) -> String {
    let implements = if t.implements.is_empty() {
        String::new()
    } else {
        format!(" implements {}", t.implements.join(" & "))
    };
    let mut out = format!("type {}{} {{\n", t.name, implements);
    for field in &t.fields {
        out.push_str(&format!(
            "  {}: {}\n",
            field.name,
            field.field_type.to_sdl()
        ));
    }
    out.push_str("}\n");
    out
}

/// Render the full schema SDL.
pub fn render_schema_sdl(schema: &GqlSchema) -> String {
    let types_sdl: Vec<String> = schema.types.iter().map(render_type_sdl).collect();
    let mut out = types_sdl.join("\n");
    if let Some(q) = &schema.query_type {
        out.push_str(&format!("\nschema {{\n  query: {q}\n}}\n"));
    }
    out
}

/// Validate schema (non-empty type names, at least one field per type).
pub fn validate_schema(schema: &GqlSchema) -> bool {
    schema
        .types
        .iter()
        .all(|t| !t.name.is_empty() && !t.fields.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_schema() -> GqlSchema {
        let mut schema = GqlSchema {
            query_type: Some("Query".into()),
            ..Default::default()
        };
        let mut query_type = GqlObjectType::new("Query");
        query_type.add_field("version", "String", true);
        query_type.add_field("name", "String", false);
        schema.add_type(query_type);
        schema
    }

    #[test]
    fn type_count() {
        assert_eq!(sample_schema().type_count(), 1);
    }

    #[test]
    fn find_type_found() {
        assert!(sample_schema().find_type("Query").is_some());
    }

    #[test]
    fn find_type_missing() {
        assert!(sample_schema().find_type("Mutation").is_none());
    }

    #[test]
    fn field_type_non_null_sdl() {
        let ft = GqlFieldType {
            base_type: "String".into(),
            non_null: true,
            is_list: false,
        };
        assert_eq!(ft.to_sdl(), "String!");
    }

    #[test]
    fn field_type_list_sdl() {
        let ft = GqlFieldType {
            base_type: "String".into(),
            non_null: false,
            is_list: true,
        };
        assert_eq!(ft.to_sdl(), "[String]");
    }

    #[test]
    fn render_type_contains_type_keyword() {
        let t = sample_schema().types[0].clone();
        assert!(render_type_sdl(&t).contains("type Query"));
    }

    #[test]
    fn render_schema_contains_schema_block() {
        assert!(render_schema_sdl(&sample_schema()).contains("schema {"));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_schema(&sample_schema()));
    }

    #[test]
    fn validate_empty_type_name() {
        let mut schema = GqlSchema::default();
        schema.add_type(GqlObjectType::new(""));
        assert!(!validate_schema(&schema));
    }
}
