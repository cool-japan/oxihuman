// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! RIS reference format export.

/// Reference type tags used in RIS format.
#[derive(Debug, Clone, PartialEq)]
pub enum RisType {
    Journal,
    Book,
    Conference,
    Report,
    Thesis,
    Generic,
}

impl RisType {
    /// RIS type code string.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Journal => "JOUR",
            Self::Book => "BOOK",
            Self::Conference => "CONF",
            Self::Report => "RPRT",
            Self::Thesis => "THES",
            Self::Generic => "GEN",
        }
    }
}

/// A single RIS record field (tag + value).
#[derive(Debug, Clone)]
pub struct RisField {
    pub tag: String,
    pub value: String,
}

/// A single RIS reference record.
#[derive(Debug, Clone)]
pub struct RisRecord {
    pub ref_type: RisType,
    pub fields: Vec<RisField>,
}

impl RisRecord {
    /// Create a new empty record of the given type.
    pub fn new(ref_type: RisType) -> Self {
        Self {
            ref_type,
            fields: Vec::new(),
        }
    }

    /// Add a field to the record.
    pub fn add_field(&mut self, tag: impl Into<String>, value: impl Into<String>) {
        self.fields.push(RisField {
            tag: tag.into(),
            value: value.into(),
        });
    }

    /// Retrieve the first value for a given tag.
    pub fn get_field(&self, tag: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|f| f.tag == tag)
            .map(|f| f.value.as_str())
    }
}

/// A collection of RIS records.
#[derive(Debug, Clone, Default)]
pub struct RisDatabase {
    pub records: Vec<RisRecord>,
}

impl RisDatabase {
    /// Add a record.
    pub fn add_record(&mut self, record: RisRecord) {
        self.records.push(record);
    }

    /// Count of records.
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

/// Render a single RIS record to string.
pub fn render_record(rec: &RisRecord) -> String {
    let mut out = format!("TY  - {}\n", rec.ref_type.code());
    for field in &rec.fields {
        out.push_str(&format!("{}  - {}\n", field.tag, field.value));
    }
    out.push_str("ER  - \n");
    out
}

/// Render the full RIS database.
pub fn render_ris(db: &RisDatabase) -> String {
    db.records
        .iter()
        .map(render_record)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Validate that records have at least a title field (TI or T1).
pub fn validate_record(rec: &RisRecord) -> bool {
    rec.get_field("TI").is_some() || rec.get_field("T1").is_some()
}

/// Count records of a specific type.
pub fn count_by_type(db: &RisDatabase, ref_type: &RisType) -> usize {
    db.records
        .iter()
        .filter(|r| &r.ref_type == ref_type)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record() -> RisRecord {
        let mut r = RisRecord::new(RisType::Journal);
        r.add_field("TI", "Test Title");
        r.add_field("AU", "Smith, J.");
        r.add_field("PY", "2026");
        r
    }

    #[test]
    fn type_code() {
        assert_eq!(RisType::Journal.code(), "JOUR");
    }

    #[test]
    fn get_field_found() {
        assert_eq!(sample_record().get_field("TI"), Some("Test Title"));
    }

    #[test]
    fn get_field_missing() {
        assert!(sample_record().get_field("ZZ").is_none());
    }

    #[test]
    fn record_count() {
        let mut db = RisDatabase::default();
        db.add_record(sample_record());
        assert_eq!(db.record_count(), 1);
    }

    #[test]
    fn render_record_ty_line() {
        let s = render_record(&sample_record());
        assert!(s.starts_with("TY  - JOUR"));
    }

    #[test]
    fn render_record_er_line() {
        assert!(render_record(&sample_record()).contains("ER  - "));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_record(&sample_record()));
    }

    #[test]
    fn validate_no_title() {
        let r = RisRecord::new(RisType::Generic);
        assert!(!validate_record(&r));
    }

    #[test]
    fn count_by_type_correct() {
        let mut db = RisDatabase::default();
        db.add_record(sample_record());
        db.add_record(RisRecord::new(RisType::Book));
        assert_eq!(count_by_type(&db, &RisType::Journal), 1);
    }

    #[test]
    fn render_ris_contains_type() {
        let mut db = RisDatabase::default();
        db.add_record(sample_record());
        assert!(render_ris(&db).contains("JOUR"));
    }
}
