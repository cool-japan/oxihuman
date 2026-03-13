//! Generic JSON data export — builds a structured JSON document from key-value data.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for JSON export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JsonExportConfig {
    /// Pretty-print with indentation when true.
    pub pretty: bool,
    /// Indentation string (default: two spaces).
    pub indent: String,
    /// Optional document title stored as a metadata field.
    pub title: String,
}

/// A JSON value variant.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum JsonValue {
    /// JSON null.
    Null,
    /// JSON boolean.
    Bool(bool),
    /// JSON number.
    Number(f64),
    /// JSON string.
    Str(String),
    /// JSON array.
    Array(Vec<JsonValue>),
    /// JSON object (ordered key-value pairs).
    Object(Vec<(String, JsonValue)>),
}

/// A JSON document containing an ordered list of top-level fields.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JsonDocument {
    /// Export configuration.
    pub config: JsonExportConfig,
    /// Top-level fields in insertion order.
    pub fields: Vec<(String, JsonValue)>,
}

// ── Construction ──────────────────────────────────────────────────────────────

/// Returns a default [`JsonExportConfig`].
#[allow(dead_code)]
pub fn default_json_export_config() -> JsonExportConfig {
    JsonExportConfig {
        pretty: true,
        indent: "  ".to_string(),
        title: String::new(),
    }
}

/// Creates a new empty [`JsonDocument`] with the given config.
#[allow(dead_code)]
pub fn new_json_document(cfg: &JsonExportConfig) -> JsonDocument {
    JsonDocument {
        config: cfg.clone(),
        fields: Vec::new(),
    }
}

// ── Mutation ──────────────────────────────────────────────────────────────────

/// Inserts or replaces a top-level field in the document.
#[allow(dead_code)]
pub fn json_set_field(doc: &mut JsonDocument, key: &str, value: JsonValue) {
    if let Some(entry) = doc.fields.iter_mut().find(|(k, _)| k == key) {
        entry.1 = value;
    } else {
        doc.fields.push((key.to_string(), value));
    }
}

// ── Serialisation ─────────────────────────────────────────────────────────────

/// Converts a [`JsonValue`] to its JSON string representation.
#[allow(dead_code)]
pub fn json_value_to_string(val: &JsonValue) -> String {
    json_value_to_string_indent(val, 0, "  ")
}

fn json_value_to_string_indent(val: &JsonValue, depth: usize, indent: &str) -> String {
    match val {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        JsonValue::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{:.1}", n)
            } else {
                format!("{}", n)
            }
        }
        JsonValue::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            let inner_depth = depth + 1;
            let pad = indent.repeat(inner_depth);
            let close_pad = indent.repeat(depth);
            let items: Vec<String> = arr
                .iter()
                .map(|v| format!("{}{}", pad, json_value_to_string_indent(v, inner_depth, indent)))
                .collect();
            format!("[\n{}\n{}]", items.join(",\n"), close_pad)
        }
        JsonValue::Object(pairs) => {
            if pairs.is_empty() {
                return "{}".to_string();
            }
            let inner_depth = depth + 1;
            let pad = indent.repeat(inner_depth);
            let close_pad = indent.repeat(depth);
            let items: Vec<String> = pairs
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}\"{}\": {}",
                        pad,
                        k,
                        json_value_to_string_indent(v, inner_depth, indent)
                    )
                })
                .collect();
            format!("{{\n{}\n{}}}", items.join(",\n"), close_pad)
        }
    }
}

/// Serialises a [`JsonDocument`] to a JSON string.
#[allow(dead_code)]
pub fn json_to_string(doc: &JsonDocument) -> String {
    let indent = if doc.config.pretty {
        doc.config.indent.as_str()
    } else {
        ""
    };
    let obj = JsonValue::Object(doc.fields.clone());
    json_value_to_string_indent(&obj, 0, indent)
}

/// Writes a [`JsonDocument`] to a file at the given path.
#[allow(dead_code)]
pub fn json_write_to_file(doc: &JsonDocument, path: &str) -> Result<(), String> {
    use std::io::Write;
    let content = json_to_string(doc);
    let mut file =
        std::fs::File::create(path).map_err(|e| format!("json_write_to_file: {}", e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("json_write_to_file write: {}", e))
}

// ── Query ─────────────────────────────────────────────────────────────────────

/// Returns the number of top-level fields in the document.
#[allow(dead_code)]
pub fn json_field_count(doc: &JsonDocument) -> usize {
    doc.fields.len()
}

/// Returns a reference to the value for the given key, if present.
#[allow(dead_code)]
pub fn json_get_field<'a>(doc: &'a JsonDocument, key: &str) -> Option<&'a JsonValue> {
    doc.fields
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Builds a [`JsonValue::Array`] from a slice of `f32` values.
#[allow(dead_code)]
pub fn json_array_from_floats(values: &[f32]) -> JsonValue {
    JsonValue::Array(values.iter().map(|&f| JsonValue::Number(f as f64)).collect())
}

/// Merges `overlay` into `base`, with overlay fields taking priority.
/// Returns a new document using `base`'s config.
#[allow(dead_code)]
pub fn json_merge_documents(base: &JsonDocument, overlay: &JsonDocument) -> JsonDocument {
    let mut merged = base.clone();
    for (key, value) in &overlay.fields {
        json_set_field(&mut merged, key, value.clone());
    }
    merged
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_json_export_config();
        assert!(cfg.pretty);
        assert_eq!(cfg.indent, "  ");
        assert!(cfg.title.is_empty());
    }

    #[test]
    fn test_new_document_empty() {
        let cfg = default_json_export_config();
        let doc = new_json_document(&cfg);
        assert_eq!(json_field_count(&doc), 0);
    }

    #[test]
    fn test_set_and_get_field() {
        let cfg = default_json_export_config();
        let mut doc = new_json_document(&cfg);
        json_set_field(&mut doc, "name", JsonValue::Str("Alice".to_string()));
        assert_eq!(json_field_count(&doc), 1);
        let val = json_get_field(&doc, "name").expect("should succeed");
        if let JsonValue::Str(s) = val {
            assert_eq!(s, "Alice");
        } else {
            panic!("expected Str");
        }
    }

    #[test]
    fn test_set_field_updates_existing() {
        let cfg = default_json_export_config();
        let mut doc = new_json_document(&cfg);
        json_set_field(&mut doc, "x", JsonValue::Number(1.0));
        json_set_field(&mut doc, "x", JsonValue::Number(2.0));
        assert_eq!(json_field_count(&doc), 1);
        if let Some(JsonValue::Number(n)) = json_get_field(&doc, "x") {
            assert!((n - 2.0).abs() < 1e-9);
        } else {
            panic!("expected Number");
        }
    }

    #[test]
    fn test_json_value_to_string_null() {
        assert_eq!(json_value_to_string(&JsonValue::Null), "null");
    }

    #[test]
    fn test_json_value_to_string_bool() {
        assert_eq!(json_value_to_string(&JsonValue::Bool(true)), "true");
        assert_eq!(json_value_to_string(&JsonValue::Bool(false)), "false");
    }

    #[test]
    fn test_json_array_from_floats() {
        let arr = json_array_from_floats(&[1.0, 2.0, 3.0]);
        if let JsonValue::Array(items) = arr {
            assert_eq!(items.len(), 3);
        } else {
            panic!("expected Array");
        }
    }

    #[test]
    fn test_json_merge_documents() {
        let cfg = default_json_export_config();
        let mut base = new_json_document(&cfg);
        json_set_field(&mut base, "a", JsonValue::Number(1.0));
        json_set_field(&mut base, "b", JsonValue::Number(2.0));

        let mut overlay = new_json_document(&cfg);
        json_set_field(&mut overlay, "b", JsonValue::Number(99.0));
        json_set_field(&mut overlay, "c", JsonValue::Number(3.0));

        let merged = json_merge_documents(&base, &overlay);
        assert_eq!(json_field_count(&merged), 3);
        if let Some(JsonValue::Number(n)) = json_get_field(&merged, "b") {
            assert!((n - 99.0).abs() < 1e-9);
        } else {
            panic!("expected Number 99");
        }
    }

    #[test]
    fn test_json_to_string_contains_key() {
        let cfg = default_json_export_config();
        let mut doc = new_json_document(&cfg);
        json_set_field(&mut doc, "hello", JsonValue::Str("world".to_string()));
        let s = json_to_string(&doc);
        assert!(s.contains("\"hello\""));
        assert!(s.contains("\"world\""));
    }

    #[test]
    fn test_get_missing_field_returns_none() {
        let cfg = default_json_export_config();
        let doc = new_json_document(&cfg);
        assert!(json_get_field(&doc, "missing").is_none());
    }
}
