#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! ASCII table formatter.

/// A row of cells.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<String>,
}

/// A simple text table with columns and rows.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FormatTable {
    columns: Vec<String>,
    rows: Vec<TableRow>,
}

#[allow(dead_code)]
pub fn new_format_table() -> FormatTable {
    FormatTable {
        columns: Vec::new(),
        rows: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_column(table: &mut FormatTable, name: &str) {
    table.columns.push(name.to_string());
}

#[allow(dead_code)]
pub fn add_row(table: &mut FormatTable, cells: Vec<String>) {
    table.rows.push(TableRow { cells });
}

#[allow(dead_code)]
pub fn render_table(table: &FormatTable) -> String {
    if table.columns.is_empty() {
        return String::new();
    }
    let ncols = table.columns.len();
    let mut widths = vec![0usize; ncols];
    for (i, col) in table.columns.iter().enumerate() {
        widths[i] = col.len();
    }
    for row in &table.rows {
        for (i, cell) in row.cells.iter().enumerate() {
            if i < ncols && cell.len() > widths[i] {
                widths[i] = cell.len();
            }
        }
    }

    let mut out = String::new();
    // Header
    for (i, col) in table.columns.iter().enumerate() {
        if i > 0 {
            out.push_str(" | ");
        }
        out.push_str(&format!("{:width$}", col, width = widths[i]));
    }
    out.push('\n');
    // Separator
    for (i, w) in widths.iter().enumerate() {
        if i > 0 {
            out.push_str("-+-");
        }
        for _ in 0..*w {
            out.push('-');
        }
    }
    out.push('\n');
    // Rows
    for row in &table.rows {
        for (i, w) in widths.iter().enumerate() {
            if i > 0 {
                out.push_str(" | ");
            }
            let cell = row.cells.get(i).map(|s| s.as_str()).unwrap_or("");
            out.push_str(&format!("{:width$}", cell, width = *w));
        }
        out.push('\n');
    }
    out
}

#[allow(dead_code)]
pub fn column_count(table: &FormatTable) -> usize {
    table.columns.len()
}

#[allow(dead_code)]
pub fn row_count(table: &FormatTable) -> usize {
    table.rows.len()
}

#[allow(dead_code)]
pub fn cell_at(table: &FormatTable, row: usize, col: usize) -> Option<&str> {
    table.rows.get(row).and_then(|r| r.cells.get(col)).map(|s| s.as_str())
}

#[allow(dead_code)]
pub fn table_width(table: &FormatTable) -> usize {
    if table.columns.is_empty() {
        return 0;
    }
    let ncols = table.columns.len();
    let mut widths = vec![0usize; ncols];
    for (i, col) in table.columns.iter().enumerate() {
        widths[i] = col.len();
    }
    for row in &table.rows {
        for (i, cell) in row.cells.iter().enumerate() {
            if i < ncols && cell.len() > widths[i] {
                widths[i] = cell.len();
            }
        }
    }
    let content: usize = widths.iter().sum();
    let separators = if ncols > 1 { (ncols - 1) * 3 } else { 0 };
    content + separators
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_table() {
        let t = new_format_table();
        assert_eq!(column_count(&t), 0);
        assert_eq!(row_count(&t), 0);
    }

    #[test]
    fn test_add_column() {
        let mut t = new_format_table();
        add_column(&mut t, "Name");
        assert_eq!(column_count(&t), 1);
    }

    #[test]
    fn test_add_row() {
        let mut t = new_format_table();
        add_column(&mut t, "A");
        add_row(&mut t, vec!["1".to_string()]);
        assert_eq!(row_count(&t), 1);
    }

    #[test]
    fn test_cell_at() {
        let mut t = new_format_table();
        add_column(&mut t, "X");
        add_row(&mut t, vec!["val".to_string()]);
        assert_eq!(cell_at(&t, 0, 0), Some("val"));
    }

    #[test]
    fn test_cell_at_oob() {
        let t = new_format_table();
        assert_eq!(cell_at(&t, 0, 0), None);
    }

    #[test]
    fn test_render_empty() {
        let t = new_format_table();
        assert!(render_table(&t).is_empty());
    }

    #[test]
    fn test_render_basic() {
        let mut t = new_format_table();
        add_column(&mut t, "A");
        add_row(&mut t, vec!["1".to_string()]);
        let s = render_table(&t);
        assert!(s.contains('A'));
        assert!(s.contains('1'));
    }

    #[test]
    fn test_table_width() {
        let mut t = new_format_table();
        add_column(&mut t, "AB");
        add_column(&mut t, "CD");
        // width = 2 + 3 + 2 = 7 (2 cols + " | " separator)
        assert_eq!(table_width(&t), 7);
    }

    #[test]
    fn test_render_two_columns() {
        let mut t = new_format_table();
        add_column(&mut t, "Name");
        add_column(&mut t, "Val");
        add_row(&mut t, vec!["x".to_string(), "1".to_string()]);
        let s = render_table(&t);
        assert!(s.contains("Name"));
        assert!(s.contains("Val"));
    }

    #[test]
    fn test_width_empty() {
        let t = new_format_table();
        assert_eq!(table_width(&t), 0);
    }
}
