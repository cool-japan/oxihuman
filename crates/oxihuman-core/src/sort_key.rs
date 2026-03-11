// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sort-key builder for composite sort keys over multiple fields.

/// A sort direction.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum SortDir {
    Asc,
    Desc,
}

/// A single sort criterion.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SortCriterion {
    pub field: String,
    pub dir: SortDir,
    pub numeric: bool,
}

/// Composite sort key builder.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct SortKey {
    criteria: Vec<SortCriterion>,
}

#[allow(dead_code)]
impl SortKey {
    pub fn new() -> Self {
        Self {
            criteria: Vec::new(),
        }
    }

    /// Add an ascending field.
    pub fn asc(mut self, field: &str) -> Self {
        self.criteria.push(SortCriterion {
            field: field.to_string(),
            dir: SortDir::Asc,
            numeric: false,
        });
        self
    }

    /// Add a descending field.
    pub fn desc(mut self, field: &str) -> Self {
        self.criteria.push(SortCriterion {
            field: field.to_string(),
            dir: SortDir::Desc,
            numeric: false,
        });
        self
    }

    /// Add a numeric ascending field.
    pub fn asc_num(mut self, field: &str) -> Self {
        self.criteria.push(SortCriterion {
            field: field.to_string(),
            dir: SortDir::Asc,
            numeric: true,
        });
        self
    }

    /// Add a numeric descending field.
    pub fn desc_num(mut self, field: &str) -> Self {
        self.criteria.push(SortCriterion {
            field: field.to_string(),
            dir: SortDir::Desc,
            numeric: true,
        });
        self
    }

    pub fn criterion_count(&self) -> usize {
        self.criteria.len()
    }

    pub fn is_empty(&self) -> bool {
        self.criteria.is_empty()
    }

    /// Compare two string-keyed records using the composite key.
    pub fn compare(
        &self,
        a: &std::collections::HashMap<String, String>,
        b: &std::collections::HashMap<String, String>,
    ) -> std::cmp::Ordering {
        for c in &self.criteria {
            let va = a.get(&c.field).map(|s| s.as_str()).unwrap_or("");
            let vb = b.get(&c.field).map(|s| s.as_str()).unwrap_or("");
            let ord = if c.numeric {
                let na: f64 = va.parse().unwrap_or(0.0);
                let nb: f64 = vb.parse().unwrap_or(0.0);
                na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                va.cmp(vb)
            };
            let ord = if c.dir == SortDir::Desc {
                ord.reverse()
            } else {
                ord
            };
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    }

    /// Sort a slice of records in-place.
    pub fn sort(&self, records: &mut [std::collections::HashMap<String, String>]) {
        records.sort_by(|a, b| self.compare(a, b));
    }

    pub fn clear(&mut self) {
        self.criteria.clear();
    }

    /// Return a string representation of the sort key.
    pub fn to_string_repr(&self) -> String {
        self.criteria
            .iter()
            .map(|c| {
                let dir = if c.dir == SortDir::Asc { "asc" } else { "desc" };
                format!("{}:{}", c.field, dir)
            })
            .collect::<Vec<_>>()
            .join(",")
    }
}

pub fn new_sort_key() -> SortKey {
    SortKey::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn rec(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn asc_sort() {
        let key = SortKey::new().asc("name");
        let mut rows = vec![rec(&[("name", "bob")]), rec(&[("name", "alice")])];
        key.sort(&mut rows);
        assert_eq!(rows[0]["name"], "alice");
    }

    #[test]
    fn desc_sort() {
        let key = SortKey::new().desc("name");
        let mut rows = vec![rec(&[("name", "alice")]), rec(&[("name", "zoo")])];
        key.sort(&mut rows);
        assert_eq!(rows[0]["name"], "zoo");
    }

    #[test]
    fn numeric_asc() {
        let key = SortKey::new().asc_num("score");
        let mut rows = vec![
            rec(&[("score", "10")]),
            rec(&[("score", "2")]),
            rec(&[("score", "20")]),
        ];
        key.sort(&mut rows);
        assert_eq!(rows[0]["score"], "2");
    }

    #[test]
    fn numeric_desc() {
        let key = SortKey::new().desc_num("score");
        let mut rows = vec![rec(&[("score", "5")]), rec(&[("score", "100")])];
        key.sort(&mut rows);
        assert_eq!(rows[0]["score"], "100");
    }

    #[test]
    fn criterion_count() {
        let key = SortKey::new().asc("a").desc("b");
        assert_eq!(key.criterion_count(), 2);
    }

    #[test]
    fn empty_key_no_change() {
        let key = SortKey::new();
        let mut rows = vec![rec(&[("x", "z")]), rec(&[("x", "a")])];
        key.sort(&mut rows);
        assert_eq!(rows[0]["x"], "z"); // unchanged
    }

    #[test]
    fn to_string_repr() {
        let key = SortKey::new().asc("name").desc("score");
        assert_eq!(key.to_string_repr(), "name:asc,score:desc");
    }

    #[test]
    fn missing_field_treats_as_empty() {
        let key = SortKey::new().asc("missing");
        let mut rows = vec![rec(&[("x", "a")]), rec(&[("y", "b")])];
        key.sort(&mut rows);
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn clear_criteria() {
        let mut key = SortKey::new().asc("a");
        key.clear();
        assert!(key.is_empty());
    }

    #[test]
    fn stable_equal_elements() {
        let key = SortKey::new().asc("val");
        let mut rows = vec![rec(&[("val", "same")]), rec(&[("val", "same")])];
        key.sort(&mut rows);
        assert_eq!(rows.len(), 2);
    }
}
