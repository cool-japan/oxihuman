// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Simple CSV parser (no external crates).

#![allow(dead_code)]

/// A single parsed CSV record (row).
#[allow(dead_code)]
pub struct CsvRecord {
    pub fields: Vec<String>,
}

/// A parsed CSV table with named headers.
#[allow(dead_code)]
pub struct CsvTable {
    pub headers: Vec<String>,
    pub records: Vec<CsvRecord>,
}

/// Parse a single CSV line into fields, handling quoted fields.
#[allow(dead_code)]
pub fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes {
                    if chars.peek() == Some(&'"') {
                        // Escaped quote
                        chars.next();
                        current.push('"');
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.clone());
                current.clear();
            }
            other => current.push(other),
        }
    }
    fields.push(current);
    fields
}

/// Parse a CSV text with a header line into a CsvTable.
#[allow(dead_code)]
pub fn parse_csv(text: &str) -> CsvTable {
    let mut lines = text.lines();
    let headers = match lines.next() {
        Some(h) => parse_csv_line(h),
        None => {
            return CsvTable {
                headers: Vec::new(),
                records: Vec::new(),
            }
        }
    };
    let records = lines
        .filter(|l| !l.trim().is_empty())
        .map(|l| CsvRecord {
            fields: parse_csv_line(l),
        })
        .collect();
    CsvTable { headers, records }
}

/// Get the value of column `col` in row `row` (0-indexed), or None.
#[allow(dead_code)]
pub fn csv_field<'a>(table: &'a CsvTable, row: usize, col: &str) -> Option<&'a str> {
    let col_idx = table.headers.iter().position(|h| h == col)?;
    let record = table.records.get(row)?;
    record.fields.get(col_idx).map(|s| s.as_str())
}

/// Return the number of data rows (excluding the header).
#[allow(dead_code)]
pub fn csv_row_count(table: &CsvTable) -> usize {
    table.records.len()
}

/// Return the number of columns (header fields).
#[allow(dead_code)]
pub fn csv_col_count(table: &CsvTable) -> usize {
    table.headers.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CSV: &str = "name,age,city\nAlice,30,Tokyo\nBob,25,Osaka";

    #[test]
    fn parse_line_basic() {
        let fields = parse_csv_line("a,b,c");
        assert_eq!(fields, vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_line_with_quotes() {
        let fields = parse_csv_line("\"hello, world\",b");
        assert_eq!(fields, vec!["hello, world", "b"]);
    }

    #[test]
    fn parse_csv_headers() {
        let t = parse_csv(SAMPLE_CSV);
        assert_eq!(t.headers, vec!["name", "age", "city"]);
    }

    #[test]
    fn parse_csv_row_count() {
        let t = parse_csv(SAMPLE_CSV);
        assert_eq!(csv_row_count(&t), 2);
    }

    #[test]
    fn parse_csv_col_count() {
        let t = parse_csv(SAMPLE_CSV);
        assert_eq!(csv_col_count(&t), 3);
    }

    #[test]
    fn csv_field_access() {
        let t = parse_csv(SAMPLE_CSV);
        assert_eq!(csv_field(&t, 0, "name"), Some("Alice"));
        assert_eq!(csv_field(&t, 1, "city"), Some("Osaka"));
    }

    #[test]
    fn csv_field_missing_col() {
        let t = parse_csv(SAMPLE_CSV);
        assert!(csv_field(&t, 0, "unknown").is_none());
    }

    #[test]
    fn csv_field_missing_row() {
        let t = parse_csv(SAMPLE_CSV);
        assert!(csv_field(&t, 99, "name").is_none());
    }

    #[test]
    fn parse_empty_csv() {
        let t = parse_csv("");
        assert_eq!(csv_row_count(&t), 0);
        assert_eq!(csv_col_count(&t), 0);
    }

    #[test]
    fn parse_single_row() {
        let t = parse_csv("a,b\n1,2");
        assert_eq!(csv_row_count(&t), 1);
        assert_eq!(csv_field(&t, 0, "a"), Some("1"));
    }
}
