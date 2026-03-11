// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! TSV (tab-separated values) parser.

/// A single parsed TSV record (row).
#[derive(Debug, Clone)]
pub struct TsvRecord {
    pub fields: Vec<String>,
}

/// A parsed TSV table with named headers.
#[derive(Debug, Clone)]
pub struct TsvTable {
    pub headers: Vec<String>,
    pub records: Vec<TsvRecord>,
}

/// Parse a single TSV line into fields (splitting on tab).
pub fn parse_tsv_line(line: &str) -> Vec<String> {
    line.split('\t').map(|f| f.to_string()).collect()
}

/// Parse a TSV text with a header line into a [`TsvTable`].
pub fn parse_tsv(text: &str) -> TsvTable {
    let mut lines = text.lines();
    let headers = match lines.next() {
        Some(h) => parse_tsv_line(h),
        None => {
            return TsvTable {
                headers: Vec::new(),
                records: Vec::new(),
            }
        }
    };
    let records = lines
        .filter(|l| !l.trim().is_empty())
        .map(|l| TsvRecord {
            fields: parse_tsv_line(l),
        })
        .collect();
    TsvTable { headers, records }
}

/// Get the value of column `col` in row `row` (0-indexed), or None.
pub fn tsv_field<'a>(table: &'a TsvTable, row: usize, col: &str) -> Option<&'a str> {
    let col_idx = table.headers.iter().position(|h| h == col)?;
    let record = table.records.get(row)?;
    record.fields.get(col_idx).map(|s| s.as_str())
}

/// Return the number of data rows (excluding the header).
pub fn tsv_row_count(table: &TsvTable) -> usize {
    table.records.len()
}

/// Return the number of columns (header fields).
pub fn tsv_col_count(table: &TsvTable) -> usize {
    table.headers.len()
}

/// Serialize a `TsvTable` back to a TSV string.
pub fn tsv_to_string(table: &TsvTable) -> String {
    let mut out = table.headers.join("\t");
    out.push('\n');
    for rec in &table.records {
        out.push_str(&rec.fields.join("\t"));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "name\tage\tcity\nAlice\t30\tTokyo\nBob\t25\tOsaka";

    #[test]
    fn test_parse_line_basic() {
        /* basic tab-split */
        let fields = parse_tsv_line("a\tb\tc");
        assert_eq!(fields, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_headers() {
        /* header row is parsed correctly */
        let t = parse_tsv(SAMPLE);
        assert_eq!(t.headers, vec!["name", "age", "city"]);
    }

    #[test]
    fn test_row_count() {
        /* two data rows */
        let t = parse_tsv(SAMPLE);
        assert_eq!(tsv_row_count(&t), 2);
    }

    #[test]
    fn test_col_count() {
        /* three columns */
        let t = parse_tsv(SAMPLE);
        assert_eq!(tsv_col_count(&t), 3);
    }

    #[test]
    fn test_field_access() {
        /* field lookup by column name */
        let t = parse_tsv(SAMPLE);
        assert_eq!(tsv_field(&t, 0, "name"), Some("Alice"));
        assert_eq!(tsv_field(&t, 1, "city"), Some("Osaka"));
    }

    #[test]
    fn test_field_missing_col() {
        /* unknown column returns None */
        let t = parse_tsv(SAMPLE);
        assert!(tsv_field(&t, 0, "unknown").is_none());
    }

    #[test]
    fn test_field_missing_row() {
        /* out-of-range row returns None */
        let t = parse_tsv(SAMPLE);
        assert!(tsv_field(&t, 99, "name").is_none());
    }

    #[test]
    fn test_parse_empty() {
        /* empty input gives empty table */
        let t = parse_tsv("");
        assert_eq!(tsv_row_count(&t), 0);
        assert_eq!(tsv_col_count(&t), 0);
    }

    #[test]
    fn test_to_string_roundtrip() {
        /* serialization preserves structure */
        let t = parse_tsv(SAMPLE);
        let s = tsv_to_string(&t);
        let t2 = parse_tsv(&s);
        assert_eq!(t2.headers, t.headers);
        assert_eq!(tsv_row_count(&t2), tsv_row_count(&t));
    }
}
