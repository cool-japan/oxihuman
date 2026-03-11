//! CSV data export for morph weights, bone transforms, and keyframe data.
//!
//! Provides a simple columnar CSV builder used by the OxiHuman export pipeline
//! to serialise per-vertex weights, bone channels, and animation keyframes.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for CSV export (delimiter, quoting, …).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CsvExportConfig {
    /// Field delimiter character (default: `','`).
    pub delimiter: char,
    /// Whether to include a header row when serialising.
    pub include_header: bool,
    /// Whether to quote all string fields.
    pub quote_strings: bool,
}

/// A single named column in a CSV document.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CsvColumn {
    /// Column header label.
    pub name: String,
}

/// A single data row in a CSV document.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CsvRow {
    /// Ordered cell values (strings) for this row.
    pub values: Vec<String>,
}

/// An in-memory CSV document composed of columns and rows.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CsvDocument {
    /// Column definitions (ordered).
    pub columns: Vec<CsvColumn>,
    /// Data rows.
    pub rows: Vec<CsvRow>,
    /// Export configuration snapshot.
    pub config: CsvExportConfig,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Returns a default [`CsvExportConfig`] with comma delimiter and header enabled.
#[allow(dead_code)]
pub fn default_csv_config() -> CsvExportConfig {
    CsvExportConfig {
        delimiter: ',',
        include_header: true,
        quote_strings: false,
    }
}

/// Creates a new empty [`CsvDocument`] using the given configuration.
#[allow(dead_code)]
pub fn new_csv_document(cfg: &CsvExportConfig) -> CsvDocument {
    CsvDocument {
        columns: Vec::new(),
        rows: Vec::new(),
        config: cfg.clone(),
    }
}

/// Appends a column with the given name to `doc`.
#[allow(dead_code)]
pub fn csv_add_column(doc: &mut CsvDocument, name: &str) {
    doc.columns.push(CsvColumn { name: name.to_owned() });
}

/// Appends a data row to `doc`.
#[allow(dead_code)]
pub fn csv_add_row(doc: &mut CsvDocument, values: Vec<String>) {
    doc.rows.push(CsvRow { values });
}

/// Serialises `doc` to a CSV string.
#[allow(dead_code)]
pub fn csv_to_string(doc: &CsvDocument) -> String {
    let delim = doc.config.delimiter;
    let mut out = String::new();

    if doc.config.include_header && !doc.columns.is_empty() {
        let header: Vec<&str> = doc.columns.iter().map(|c| c.name.as_str()).collect();
        out.push_str(&header.join(&delim.to_string()));
        out.push('\n');
    }

    for row in &doc.rows {
        let line: Vec<&str> = row.values.iter().map(|v| v.as_str()).collect();
        out.push_str(&line.join(&delim.to_string()));
        out.push('\n');
    }

    out
}

/// Writes a CSV document to `path`.  Returns an error string on failure.
#[allow(dead_code)]
pub fn csv_write_to_file(doc: &CsvDocument, path: &str) -> Result<(), String> {
    let content = csv_to_string(doc);
    std::fs::write(path, content).map_err(|e| e.to_string())
}

/// Returns the number of rows in `doc`.
#[allow(dead_code)]
pub fn csv_row_count(doc: &CsvDocument) -> usize {
    doc.rows.len()
}

/// Returns the number of columns in `doc`.
#[allow(dead_code)]
pub fn csv_column_count(doc: &CsvDocument) -> usize {
    doc.columns.len()
}

/// Exports a slice of morph weights and their names into a new [`CsvDocument`].
#[allow(dead_code)]
pub fn csv_export_morph_weights(
    weights: &[f32],
    names: &[String],
    cfg: &CsvExportConfig,
) -> CsvDocument {
    let mut doc = new_csv_document(cfg);
    csv_add_column(&mut doc, "name");
    csv_add_column(&mut doc, "weight");

    let len = weights.len().min(names.len());
    for i in 0..len {
        csv_add_row(&mut doc, vec![names[i].clone(), format!("{:.6}", weights[i])]);
    }
    doc
}

/// Changes the delimiter character on `cfg`.
#[allow(dead_code)]
pub fn csv_set_delimiter(cfg: &mut CsvExportConfig, delim: char) {
    cfg.delimiter = delim;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_comma_delimiter() {
        let cfg = default_csv_config();
        assert_eq!(cfg.delimiter, ',');
        assert!(cfg.include_header);
    }

    #[test]
    fn new_document_is_empty() {
        let cfg = default_csv_config();
        let doc = new_csv_document(&cfg);
        assert_eq!(csv_row_count(&doc), 0);
        assert_eq!(csv_column_count(&doc), 0);
    }

    #[test]
    fn add_columns_and_rows() {
        let cfg = default_csv_config();
        let mut doc = new_csv_document(&cfg);
        csv_add_column(&mut doc, "x");
        csv_add_column(&mut doc, "y");
        csv_add_row(&mut doc, vec!["1.0".to_owned(), "2.0".to_owned()]);
        assert_eq!(csv_column_count(&doc), 2);
        assert_eq!(csv_row_count(&doc), 1);
    }

    #[test]
    fn csv_to_string_includes_header() {
        let cfg = default_csv_config();
        let mut doc = new_csv_document(&cfg);
        csv_add_column(&mut doc, "a");
        csv_add_column(&mut doc, "b");
        csv_add_row(&mut doc, vec!["1".to_owned(), "2".to_owned()]);
        let s = csv_to_string(&doc);
        assert!(s.starts_with("a,b\n"));
        assert!(s.contains("1,2\n"));
    }

    #[test]
    fn csv_to_string_tab_delimiter() {
        let mut cfg = default_csv_config();
        csv_set_delimiter(&mut cfg, '\t');
        let mut doc = new_csv_document(&cfg);
        csv_add_column(&mut doc, "col1");
        csv_add_column(&mut doc, "col2");
        csv_add_row(&mut doc, vec!["hello".to_owned(), "world".to_owned()]);
        let s = csv_to_string(&doc);
        assert!(s.contains("col1\tcol2\n"));
    }

    #[test]
    fn morph_weight_export_row_count() {
        let cfg = default_csv_config();
        let weights = vec![0.1_f32, 0.5, 0.9];
        let names: Vec<String> = ["jaw", "blink", "smile"].iter().map(|&s| s.to_owned()).collect();
        let doc = csv_export_morph_weights(&weights, &names, &cfg);
        assert_eq!(csv_row_count(&doc), 3);
        assert_eq!(csv_column_count(&doc), 2);
    }

    #[test]
    fn morph_weight_shorter_names_slice() {
        let cfg = default_csv_config();
        let weights = vec![0.0_f32, 1.0, 0.5, 0.25];
        let names: Vec<String> = ["a", "b"].iter().map(|&s| s.to_owned()).collect();
        let doc = csv_export_morph_weights(&weights, &names, &cfg);
        // Only 2 rows because names is the shorter slice
        assert_eq!(csv_row_count(&doc), 2);
    }

    #[test]
    fn set_delimiter_changes_config() {
        let mut cfg = default_csv_config();
        csv_set_delimiter(&mut cfg, ';');
        assert_eq!(cfg.delimiter, ';');
    }

    #[test]
    fn csv_write_to_file_roundtrip() {
        let cfg = default_csv_config();
        let mut doc = new_csv_document(&cfg);
        csv_add_column(&mut doc, "v");
        csv_add_row(&mut doc, vec!["42".to_owned()]);
        let path = "/tmp/oxihuman_csv_export_test.csv";
        assert!(csv_write_to_file(&doc, path).is_ok());
        let read_back = std::fs::read_to_string(path).unwrap();
        assert!(read_back.contains("42"));
        let _ = std::fs::remove_file(path);
    }
}
