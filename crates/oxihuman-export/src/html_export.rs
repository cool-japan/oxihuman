// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export mesh stats as a simple HTML page.

#![allow(dead_code)]

/// An HTML export document.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct HtmlExport {
    pub title: String,
    pub sections: Vec<HtmlSection>,
}

/// A named section containing HTML table rows.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HtmlSection {
    pub heading: String,
    pub rows: Vec<[String; 2]>,
}

/// Create a new HTML export document.
#[allow(dead_code)]
pub fn new_html_export(title: &str) -> HtmlExport {
    HtmlExport {
        title: title.to_string(),
        sections: Vec::new(),
    }
}

/// Add a section with a heading.
#[allow(dead_code)]
pub fn add_html_section(doc: &mut HtmlExport, heading: &str) {
    doc.sections.push(HtmlSection {
        heading: heading.to_string(),
        rows: Vec::new(),
    });
}

/// Add a key-value row to the last section.
#[allow(dead_code)]
pub fn add_html_row(doc: &mut HtmlExport, key: &str, value: &str) {
    if let Some(sec) = doc.sections.last_mut() {
        sec.rows.push([key.to_string(), value.to_string()]);
    }
}

/// Return the total number of rows across all sections.
#[allow(dead_code)]
pub fn total_row_count(doc: &HtmlExport) -> usize {
    doc.sections.iter().map(|s| s.rows.len()).sum()
}

/// Return the number of sections.
#[allow(dead_code)]
pub fn section_count(doc: &HtmlExport) -> usize {
    doc.sections.len()
}

/// Render the document as an HTML string.
#[allow(dead_code)]
pub fn to_html_string(doc: &HtmlExport) -> String {
    let mut out = format!(
        "<!DOCTYPE html>\n<html>\n<head><title>{}</title></head>\n<body>\n<h1>{}</h1>\n",
        doc.title, doc.title
    );
    for sec in &doc.sections {
        out.push_str(&format!("<h2>{}</h2>\n<table border=\"1\">\n", sec.heading));
        for row in &sec.rows {
            out.push_str(&format!(
                "<tr><td>{}</td><td>{}</td></tr>\n",
                row[0], row[1]
            ));
        }
        out.push_str("</table>\n");
    }
    out.push_str("</body>\n</html>");
    out
}

/// Export mesh stats as an HTML page.
#[allow(dead_code)]
pub fn export_mesh_stats_html(vertex_count: usize, index_count: usize, name: &str) -> String {
    let mut doc = new_html_export("Mesh Stats");
    add_html_section(&mut doc, name);
    add_html_row(&mut doc, "Vertices", &vertex_count.to_string());
    add_html_row(&mut doc, "Indices", &index_count.to_string());
    add_html_row(&mut doc, "Triangles", &(index_count / 3).to_string());
    to_html_string(&doc)
}

/// Escape HTML special characters in a string.
#[allow(dead_code)]
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_html_export_empty() {
        let doc = new_html_export("Test");
        assert_eq!(section_count(&doc), 0);
        assert_eq!(total_row_count(&doc), 0);
    }

    #[test]
    fn test_add_section() {
        let mut doc = new_html_export("T");
        add_html_section(&mut doc, "Section 1");
        assert_eq!(section_count(&doc), 1);
    }

    #[test]
    fn test_add_row() {
        let mut doc = new_html_export("T");
        add_html_section(&mut doc, "S");
        add_html_row(&mut doc, "key", "val");
        assert_eq!(total_row_count(&doc), 1);
    }

    #[test]
    fn test_to_html_contains_title() {
        let doc = new_html_export("My Page");
        let s = to_html_string(&doc);
        assert!(s.contains("My Page"));
    }

    #[test]
    fn test_to_html_contains_table() {
        let mut doc = new_html_export("T");
        add_html_section(&mut doc, "S");
        add_html_row(&mut doc, "k", "v");
        let s = to_html_string(&doc);
        assert!(s.contains("<table"));
    }

    #[test]
    fn test_to_html_contains_row_data() {
        let mut doc = new_html_export("T");
        add_html_section(&mut doc, "S");
        add_html_row(&mut doc, "vertices", "512");
        let s = to_html_string(&doc);
        assert!(s.contains("512"));
    }

    #[test]
    fn test_export_mesh_stats_html() {
        let s = export_mesh_stats_html(100, 300, "head");
        assert!(s.contains("Vertices"));
        assert!(s.contains("head"));
    }

    #[test]
    fn test_html_escape_ampersand() {
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_html_escape_angle_brackets() {
        assert_eq!(html_escape("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn test_doctype_in_output() {
        let doc = new_html_export("T");
        let s = to_html_string(&doc);
        assert!(s.starts_with("<!DOCTYPE html>"));
    }
}
