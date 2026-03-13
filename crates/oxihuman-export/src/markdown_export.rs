// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export mesh/scene info as a Markdown table.

#![allow(dead_code)]

/// A Markdown table row.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MdRow {
    pub cells: Vec<String>,
}

/// A Markdown table document.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MdTable {
    pub title: String,
    pub headers: Vec<String>,
    pub rows: Vec<MdRow>,
}

/// Create a new Markdown table with the given title and headers.
#[allow(dead_code)]
pub fn new_md_table(title: &str, headers: &[&str]) -> MdTable {
    MdTable {
        title: title.to_string(),
        headers: headers.iter().map(|s| s.to_string()).collect(),
        rows: Vec::new(),
    }
}

/// Add a row to the table.
#[allow(dead_code)]
pub fn add_md_row(table: &mut MdTable, cells: &[&str]) {
    table.rows.push(MdRow {
        cells: cells.iter().map(|s| s.to_string()).collect(),
    });
}

/// Return the number of rows.
#[allow(dead_code)]
pub fn row_count(table: &MdTable) -> usize {
    table.rows.len()
}

/// Return the number of columns (based on headers).
#[allow(dead_code)]
pub fn column_count(table: &MdTable) -> usize {
    table.headers.len()
}

/// Render the table as a Markdown string.
#[allow(dead_code)]
pub fn to_markdown_string(table: &MdTable) -> String {
    let mut out = format!("# {}\n\n", table.title);
    // Header row
    let header_line = table.headers.join(" | ");
    out.push_str(&format!("| {} |\n", header_line));
    // Separator
    let sep: Vec<&str> = table.headers.iter().map(|_| "---").collect();
    out.push_str(&format!("| {} |\n", sep.join(" | ")));
    // Data rows
    for row in &table.rows {
        out.push_str(&format!("| {} |\n", row.cells.join(" | ")));
    }
    out
}

/// Export mesh stats as a Markdown table.
#[allow(dead_code)]
pub fn export_mesh_stats_md(vertex_count: usize, index_count: usize, name: &str) -> String {
    let mut table = new_md_table("Mesh Stats", &["Property", "Value"]);
    add_md_row(&mut table, &["Name", name]);
    add_md_row(&mut table, &["Vertices", &vertex_count.to_string()]);
    add_md_row(&mut table, &["Indices", &index_count.to_string()]);
    add_md_row(&mut table, &["Triangles", &(index_count / 3).to_string()]);
    to_markdown_string(&table)
}

/// Export a list of mesh summaries as a Markdown table.
#[allow(dead_code)]
pub fn export_mesh_list_md(entries: &[(&str, usize, usize)]) -> String {
    let mut table = new_md_table("Mesh List", &["Name", "Vertices", "Triangles"]);
    for &(name, v, i) in entries {
        add_md_row(&mut table, &[name, &v.to_string(), &(i / 3).to_string()]);
    }
    to_markdown_string(&table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_table_empty() {
        let t = new_md_table("Test", &["A", "B"]);
        assert_eq!(row_count(&t), 0);
        assert_eq!(column_count(&t), 2);
    }

    #[test]
    fn test_add_row() {
        let mut t = new_md_table("T", &["X", "Y"]);
        add_md_row(&mut t, &["1", "2"]);
        assert_eq!(row_count(&t), 1);
    }

    #[test]
    fn test_to_markdown_contains_header() {
        let t = new_md_table("Report", &["Name", "Value"]);
        let s = to_markdown_string(&t);
        assert!(s.contains("Name | Value"));
    }

    #[test]
    fn test_to_markdown_contains_separator() {
        let t = new_md_table("T", &["A"]);
        let s = to_markdown_string(&t);
        assert!(s.contains("---"));
    }

    #[test]
    fn test_to_markdown_contains_row_data() {
        let mut t = new_md_table("T", &["K", "V"]);
        add_md_row(&mut t, &["alpha", "99"]);
        let s = to_markdown_string(&t);
        assert!(s.contains("alpha"));
        assert!(s.contains("99"));
    }

    #[test]
    fn test_export_mesh_stats_md() {
        let s = export_mesh_stats_md(100, 300, "body");
        assert!(s.contains("Vertices"));
        assert!(s.contains("body"));
    }

    #[test]
    fn test_export_mesh_list_md() {
        let entries = vec![("head", 500, 900), ("body", 2000, 4000)];
        let s = export_mesh_list_md(&entries);
        assert!(s.contains("head"));
        assert!(s.contains("body"));
    }

    #[test]
    fn test_title_in_output() {
        let t = new_md_table("MyTitle", &["Col"]);
        let s = to_markdown_string(&t);
        assert!(s.contains("MyTitle"));
    }

    #[test]
    fn test_multiple_rows() {
        let mut t = new_md_table("T", &["A", "B"]);
        add_md_row(&mut t, &["1", "2"]);
        add_md_row(&mut t, &["3", "4"]);
        assert_eq!(row_count(&t), 2);
    }

    #[test]
    fn test_column_count_three() {
        let t = new_md_table("T", &["A", "B", "C"]);
        assert_eq!(column_count(&t), 3);
    }
}
