// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A simple column-oriented data table with string column names and f64 values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DataTable {
    columns: Vec<String>,
    rows: Vec<Vec<f64>>,
}

#[allow(dead_code)]
impl DataTable {
    pub fn new(columns: &[&str]) -> Self {
        Self {
            columns: columns.iter().map(|s| s.to_string()).collect(),
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, values: &[f64]) -> bool {
        if values.len() != self.columns.len() {
            return false;
        }
        self.rows.push(values.to_vec());
        true
    }

    pub fn get(&self, row: usize, col: usize) -> Option<f64> {
        self.rows.get(row).and_then(|r| r.get(col).copied())
    }

    pub fn col_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c == name)
    }

    pub fn get_by_name(&self, row: usize, col_name: &str) -> Option<f64> {
        let col = self.col_index(col_name)?;
        self.get(row, col)
    }

    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }

    pub fn num_cols(&self) -> usize {
        self.columns.len()
    }

    pub fn column_sum(&self, col: usize) -> f64 {
        self.rows.iter().filter_map(|r| r.get(col)).sum()
    }

    pub fn column_avg(&self, col: usize) -> f64 {
        if self.rows.is_empty() {
            return 0.0;
        }
        self.column_sum(col) / self.rows.len() as f64
    }

    pub fn column_names(&self) -> &[String] {
        &self.columns
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn clear(&mut self) {
        self.rows.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let dt = DataTable::new(&["x", "y"]);
        assert_eq!(dt.num_cols(), 2);
        assert!(dt.is_empty());
    }

    #[test]
    fn test_add_row() {
        let mut dt = DataTable::new(&["a", "b"]);
        assert!(dt.add_row(&[1.0, 2.0]));
        assert_eq!(dt.num_rows(), 1);
    }

    #[test]
    fn test_add_row_wrong_size() {
        let mut dt = DataTable::new(&["a", "b"]);
        assert!(!dt.add_row(&[1.0]));
    }

    #[test]
    fn test_get() {
        let mut dt = DataTable::new(&["x"]);
        dt.add_row(&[42.0]);
        assert_eq!(dt.get(0, 0), Some(42.0));
        assert_eq!(dt.get(1, 0), None);
    }

    #[test]
    fn test_get_by_name() {
        let mut dt = DataTable::new(&["x", "y"]);
        dt.add_row(&[1.0, 2.0]);
        assert_eq!(dt.get_by_name(0, "y"), Some(2.0));
        assert_eq!(dt.get_by_name(0, "z"), None);
    }

    #[test]
    fn test_column_sum() {
        let mut dt = DataTable::new(&["val"]);
        dt.add_row(&[1.0]);
        dt.add_row(&[2.0]);
        dt.add_row(&[3.0]);
        assert!((dt.column_sum(0) - 6.0).abs() < 1e-12);
    }

    #[test]
    fn test_column_avg() {
        let mut dt = DataTable::new(&["val"]);
        dt.add_row(&[2.0]);
        dt.add_row(&[4.0]);
        assert!((dt.column_avg(0) - 3.0).abs() < 1e-12);
    }

    #[test]
    fn test_column_avg_empty() {
        let dt = DataTable::new(&["val"]);
        assert!((dt.column_avg(0)).abs() < 1e-12);
    }

    #[test]
    fn test_clear() {
        let mut dt = DataTable::new(&["a"]);
        dt.add_row(&[1.0]);
        dt.clear();
        assert!(dt.is_empty());
    }

    #[test]
    fn test_col_index() {
        let dt = DataTable::new(&["alpha", "beta", "gamma"]);
        assert_eq!(dt.col_index("beta"), Some(1));
        assert_eq!(dt.col_index("delta"), None);
    }
}
