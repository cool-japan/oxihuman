// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! EndNote XML export stub.

/// Reference type for EndNote.
#[derive(Debug, Clone, PartialEq)]
pub enum EndnoteRefType {
    JournalArticle,
    Book,
    BookSection,
    ConferencePaper,
    Report,
    Thesis,
    WebPage,
}

impl EndnoteRefType {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::JournalArticle => "Journal Article",
            Self::Book => "Book",
            Self::BookSection => "Book Section",
            Self::ConferencePaper => "Conference Paper",
            Self::Report => "Report",
            Self::Thesis => "Thesis",
            Self::WebPage => "Web Page",
        }
    }
}

/// A single EndNote reference.
#[derive(Debug, Clone)]
pub struct EndnoteRef {
    pub ref_type: EndnoteRefType,
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<u32>,
    pub journal: Option<String>,
    pub volume: Option<String>,
    pub pages: Option<String>,
    pub doi: Option<String>,
}

impl EndnoteRef {
    /// Create a minimal reference.
    pub fn new(ref_type: EndnoteRefType, title: impl Into<String>) -> Self {
        Self {
            ref_type,
            title: title.into(),
            authors: Vec::new(),
            year: None,
            journal: None,
            volume: None,
            pages: None,
            doi: None,
        }
    }

    /// Add an author.
    pub fn add_author(&mut self, author: impl Into<String>) {
        self.authors.push(author.into());
    }
}

/// A collection of EndNote references.
#[derive(Debug, Clone, Default)]
pub struct EndnoteLibrary {
    pub refs: Vec<EndnoteRef>,
}

impl EndnoteLibrary {
    /// Add a reference.
    pub fn add_ref(&mut self, r: EndnoteRef) {
        self.refs.push(r);
    }

    /// Number of references.
    pub fn ref_count(&self) -> usize {
        self.refs.len()
    }
}

/// Render a reference as EndNote XML.
pub fn render_ref_xml(r: &EndnoteRef) -> String {
    let mut out = format!(
        "  <record>\n    <ref-type name=\"{}\"/>\n",
        r.ref_type.label()
    );
    out.push_str(&format!("    <title>{}</title>\n", xml_escape(&r.title)));
    for author in &r.authors {
        out.push_str(&format!("    <author>{}</author>\n", xml_escape(author)));
    }
    if let Some(y) = r.year {
        out.push_str(&format!("    <year>{y}</year>\n"));
    }
    if let Some(doi) = &r.doi {
        out.push_str(&format!(
            "    <electronic-resource-num>{doi}</electronic-resource-num>\n"
        ));
    }
    out.push_str("  </record>\n");
    out
}

/// Render the full EndNote XML document.
pub fn render_endnote_xml(lib: &EndnoteLibrary) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<xml>\n  <records>\n");
    for r in &lib.refs {
        out.push_str(&render_ref_xml(r));
    }
    out.push_str("  </records>\n</xml>\n");
    out
}

/// Validate that a reference has a non-empty title.
pub fn validate_ref(r: &EndnoteRef) -> bool {
    !r.title.is_empty()
}

/// Count references by type.
pub fn count_by_type(lib: &EndnoteLibrary, ref_type: &EndnoteRefType) -> usize {
    lib.refs.iter().filter(|r| &r.ref_type == ref_type).count()
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ref() -> EndnoteRef {
        let mut r = EndnoteRef::new(EndnoteRefType::JournalArticle, "Test Paper");
        r.add_author("Smith, J.");
        r.year = Some(2026);
        r.doi = Some("10.1234/test".into());
        r
    }

    #[test]
    fn ref_type_label() {
        assert_eq!(EndnoteRefType::JournalArticle.label(), "Journal Article");
    }

    #[test]
    fn ref_count() {
        let mut lib = EndnoteLibrary::default();
        lib.add_ref(sample_ref());
        assert_eq!(lib.ref_count(), 1);
    }

    #[test]
    fn render_xml_starts_correctly() {
        let mut lib = EndnoteLibrary::default();
        lib.add_ref(sample_ref());
        let s = render_endnote_xml(&lib);
        assert!(s.starts_with("<?xml"));
    }

    #[test]
    fn render_contains_title() {
        let s = render_ref_xml(&sample_ref());
        assert!(s.contains("Test Paper"));
    }

    #[test]
    fn render_contains_author() {
        let s = render_ref_xml(&sample_ref());
        assert!(s.contains("Smith"));
    }

    #[test]
    fn render_contains_doi() {
        let s = render_ref_xml(&sample_ref());
        assert!(s.contains("10.1234/test"));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_ref(&sample_ref()));
    }

    #[test]
    fn validate_empty_title() {
        let r = EndnoteRef::new(EndnoteRefType::Book, "");
        assert!(!validate_ref(&r));
    }

    #[test]
    fn count_by_type_correct() {
        let mut lib = EndnoteLibrary::default();
        lib.add_ref(sample_ref());
        lib.add_ref(EndnoteRef::new(EndnoteRefType::Book, "A Book"));
        assert_eq!(count_by_type(&lib, &EndnoteRefType::JournalArticle), 1);
    }

    #[test]
    fn xml_escape_works() {
        /* & should be escaped */
        let s = render_ref_xml(&EndnoteRef::new(EndnoteRefType::WebPage, "A & B"));
        assert!(s.contains("&amp;"));
    }
}
