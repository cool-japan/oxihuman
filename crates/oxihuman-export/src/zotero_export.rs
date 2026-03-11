// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Zotero CSL JSON export stub.

/// A CSL-JSON creator (author/editor).
#[derive(Debug, Clone)]
pub struct CslCreator {
    pub family: String,
    pub given: String,
    pub role: String,
}

/// A CSL-JSON item representing one reference.
#[derive(Debug, Clone)]
pub struct CslItem {
    pub id: String,
    pub item_type: String,
    pub title: String,
    pub creators: Vec<CslCreator>,
    pub issued_year: Option<i32>,
    pub doi: Option<String>,
    pub url: Option<String>,
    pub journal: Option<String>,
}

impl CslItem {
    /// Create a new CSL item.
    pub fn new(
        id: impl Into<String>,
        item_type: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            item_type: item_type.into(),
            title: title.into(),
            creators: Vec::new(),
            issued_year: None,
            doi: None,
            url: None,
            journal: None,
        }
    }

    /// Add an author.
    pub fn add_author(&mut self, family: impl Into<String>, given: impl Into<String>) {
        self.creators.push(CslCreator {
            family: family.into(),
            given: given.into(),
            role: "author".into(),
        });
    }
}

/// A Zotero CSL JSON library.
#[derive(Debug, Clone, Default)]
pub struct ZoteroLibrary {
    pub items: Vec<CslItem>,
}

impl ZoteroLibrary {
    /// Add an item.
    pub fn add_item(&mut self, item: CslItem) {
        self.items.push(item);
    }

    /// Number of items.
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Find by id.
    pub fn find_by_id(&self, id: &str) -> Option<&CslItem> {
        self.items.iter().find(|i| i.id == id)
    }
}

/// Serialize a CslItem to a JSON object string.
pub fn item_to_json(item: &CslItem) -> String {
    let creators: Vec<String> = item
        .creators
        .iter()
        .map(|c| {
            format!(
                r#"{{"family":"{}","given":"{}","role":"{}"}}"#,
                c.family, c.given, c.role
            )
        })
        .collect();
    let doi_json = item
        .doi
        .as_deref()
        .map(|d| format!(r#","DOI":"{d}""#))
        .unwrap_or_default();
    let year_json = item
        .issued_year
        .map(|y| format!(r#","issued":{{"date-parts":[[{y}]]}}"#))
        .unwrap_or_default();
    format!(
        r#"{{"id":"{}","type":"{}","title":"{}","author":[{}]{}{}}}"#,
        item.id,
        item.item_type,
        item.title,
        creators.join(","),
        doi_json,
        year_json,
    )
}

/// Serialize the entire library to CSL JSON array string.
pub fn library_to_json(lib: &ZoteroLibrary) -> String {
    let items: Vec<String> = lib.items.iter().map(item_to_json).collect();
    format!("[{}]", items.join(","))
}

/// Validate that an item has a non-empty id and title.
pub fn validate_item(item: &CslItem) -> bool {
    !item.id.is_empty() && !item.title.is_empty()
}

/// Count items of a given type.
pub fn count_by_type(lib: &ZoteroLibrary, item_type: &str) -> usize {
    lib.items
        .iter()
        .filter(|i| i.item_type == item_type)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_item() -> CslItem {
        let mut item = CslItem::new("smith2026", "article-journal", "A Great Paper");
        item.add_author("Smith", "John");
        item.issued_year = Some(2026);
        item.doi = Some("10.1234/x".into());
        item
    }

    #[test]
    fn item_count() {
        let mut lib = ZoteroLibrary::default();
        lib.add_item(sample_item());
        assert_eq!(lib.item_count(), 1);
    }

    #[test]
    fn find_by_id_found() {
        let mut lib = ZoteroLibrary::default();
        lib.add_item(sample_item());
        assert!(lib.find_by_id("smith2026").is_some());
    }

    #[test]
    fn find_by_id_missing() {
        assert!(ZoteroLibrary::default().find_by_id("nope").is_none());
    }

    #[test]
    fn json_contains_type() {
        let s = item_to_json(&sample_item());
        assert!(s.contains("article-journal"));
    }

    #[test]
    fn json_contains_title() {
        assert!(item_to_json(&sample_item()).contains("A Great Paper"));
    }

    #[test]
    fn json_contains_doi() {
        assert!(item_to_json(&sample_item()).contains("10.1234/x"));
    }

    #[test]
    fn library_json_starts_bracket() {
        let mut lib = ZoteroLibrary::default();
        lib.add_item(sample_item());
        assert!(library_to_json(&lib).starts_with('['));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_item(&sample_item()));
    }

    #[test]
    fn validate_empty_id() {
        let item = CslItem::new("", "book", "Title");
        assert!(!validate_item(&item));
    }

    #[test]
    fn count_by_type_correct() {
        let mut lib = ZoteroLibrary::default();
        lib.add_item(sample_item());
        lib.add_item(CslItem::new("b1", "book", "A Book"));
        assert_eq!(count_by_type(&lib, "article-journal"), 1);
    }
}
