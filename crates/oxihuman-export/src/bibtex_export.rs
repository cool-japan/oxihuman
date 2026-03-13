// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! BibTeX bibliography export.

use std::collections::BTreeMap;

/// Type of BibTeX entry.
#[derive(Debug, Clone, PartialEq)]
pub enum BibtexEntryType {
    Article,
    Book,
    InProceedings,
    Misc,
    TechReport,
    PhDThesis,
}

impl BibtexEntryType {
    /// Return the BibTeX type keyword.
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::Article => "article",
            Self::Book => "book",
            Self::InProceedings => "inproceedings",
            Self::Misc => "misc",
            Self::TechReport => "techreport",
            Self::PhDThesis => "phdthesis",
        }
    }
}

/// A BibTeX entry.
#[derive(Debug, Clone)]
pub struct BibtexEntry {
    pub entry_type: BibtexEntryType,
    pub cite_key: String,
    pub fields: BTreeMap<String, String>,
}

impl BibtexEntry {
    /// Create a new entry.
    pub fn new(entry_type: BibtexEntryType, cite_key: impl Into<String>) -> Self {
        Self {
            entry_type,
            cite_key: cite_key.into(),
            fields: BTreeMap::new(),
        }
    }

    /// Set a field value.
    pub fn set_field(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.fields.insert(key.into(), value.into());
    }

    /// Get a field value.
    pub fn get_field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }
}

/// A BibTeX bibliography.
#[derive(Debug, Clone, Default)]
pub struct BibtexBibliography {
    pub entries: Vec<BibtexEntry>,
}

impl BibtexBibliography {
    /// Add an entry.
    pub fn add_entry(&mut self, entry: BibtexEntry) {
        self.entries.push(entry);
    }

    /// Number of entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Find entry by cite key.
    pub fn find_by_key(&self, key: &str) -> Option<&BibtexEntry> {
        self.entries.iter().find(|e| e.cite_key == key)
    }
}

/// Render a single BibTeX entry.
pub fn render_entry(entry: &BibtexEntry) -> String {
    let mut out = format!("@{}{{{},\n", entry.entry_type.keyword(), entry.cite_key);
    for (k, v) in &entry.fields {
        out.push_str(&format!("  {k} = {{{v}}},\n"));
    }
    out.push_str("}\n");
    out
}

/// Render the entire bibliography.
pub fn render_bibtex(bib: &BibtexBibliography) -> String {
    bib.entries
        .iter()
        .map(render_entry)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Validate that every entry has at least an `author` or `editor` field.
pub fn validate_entry(entry: &BibtexEntry) -> bool {
    !entry.cite_key.is_empty()
        && (entry.fields.contains_key("author") || entry.fields.contains_key("editor"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry() -> BibtexEntry {
        let mut e = BibtexEntry::new(BibtexEntryType::Article, "smith2026");
        e.set_field("author", "Smith, John");
        e.set_field("title", "A Survey");
        e.set_field("year", "2026");
        e
    }

    #[test]
    fn entry_type_keyword() {
        assert_eq!(BibtexEntryType::Article.keyword(), "article");
    }

    #[test]
    fn set_and_get_field() {
        let e = sample_entry();
        assert_eq!(e.get_field("author"), Some("Smith, John"));
    }

    #[test]
    fn entry_count() {
        let mut bib = BibtexBibliography::default();
        bib.add_entry(sample_entry());
        assert_eq!(bib.entry_count(), 1);
    }

    #[test]
    fn find_by_key() {
        let mut bib = BibtexBibliography::default();
        bib.add_entry(sample_entry());
        assert!(bib.find_by_key("smith2026").is_some());
    }

    #[test]
    fn find_missing_key() {
        let bib = BibtexBibliography::default();
        assert!(bib.find_by_key("nobody").is_none());
    }

    #[test]
    fn render_entry_at_sign() {
        /* rendered entry starts with @ */
        let s = render_entry(&sample_entry());
        assert!(s.starts_with('@'));
    }

    #[test]
    fn render_entry_contains_key() {
        assert!(render_entry(&sample_entry()).contains("smith2026"));
    }

    #[test]
    fn validate_entry_ok() {
        assert!(validate_entry(&sample_entry()));
    }

    #[test]
    fn validate_no_author() {
        let e = BibtexEntry::new(BibtexEntryType::Misc, "key");
        assert!(!validate_entry(&e));
    }

    #[test]
    fn render_bibtex_nonempty() {
        let mut bib = BibtexBibliography::default();
        bib.add_entry(sample_entry());
        assert!(!render_bibtex(&bib).is_empty());
    }
}
