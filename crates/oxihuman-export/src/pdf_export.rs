// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! PDF document stub export.

/// PDF page size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PdfPageSize {
    pub width_pt: f32,
    pub height_pt: f32,
}

impl PdfPageSize {
    /// A4 page size in points.
    pub fn a4() -> Self {
        Self {
            width_pt: 595.28,
            height_pt: 841.89,
        }
    }

    /// Letter page size in points.
    pub fn letter() -> Self {
        Self {
            width_pt: 612.0,
            height_pt: 792.0,
        }
    }

    /// Area in square points.
    pub fn area(&self) -> f32 {
        self.width_pt * self.height_pt
    }
}

/// A single PDF page.
#[derive(Debug, Clone)]
pub struct PdfPage {
    pub size: PdfPageSize,
    pub content: String,
}

impl PdfPage {
    /// Create an empty page.
    pub fn new(size: PdfPageSize) -> Self {
        Self {
            size,
            content: String::new(),
        }
    }

    /// Append a text content stream snippet.
    pub fn append_text(&mut self, text: &str) {
        self.content.push_str(text);
    }

    /// Whether the page has any content.
    pub fn has_content(&self) -> bool {
        !self.content.is_empty()
    }
}

/// PDF document stub.
#[derive(Debug, Clone)]
pub struct PdfExport {
    pub title: String,
    pub author: String,
    pub pages: Vec<PdfPage>,
}

impl PdfExport {
    /// Create a new PDF document.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            author: String::new(),
            pages: Vec::new(),
        }
    }

    /// Add a page.
    pub fn add_page(&mut self, page: PdfPage) {
        self.pages.push(page);
    }

    /// Return page count.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}

/// Serialize PDF header (stub).
pub fn pdf_header_bytes() -> Vec<u8> {
    b"%PDF-1.7\n".to_vec()
}

/// Estimate PDF file size (stub).
pub fn estimate_pdf_bytes(doc: &PdfExport) -> usize {
    let content_len: usize = doc.pages.iter().map(|p| p.content.len() + 256).sum();
    512 + content_len
}

/// Validate PDF document.
pub fn validate_pdf(doc: &PdfExport) -> bool {
    !doc.title.is_empty() && !doc.pages.is_empty()
}

/// Serialize PDF metadata to JSON (stub).
pub fn pdf_metadata_json(doc: &PdfExport) -> String {
    format!(
        "{{\"title\":\"{}\",\"author\":\"{}\",\"pages\":{}}}",
        doc.title,
        doc.author,
        doc.page_count()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> PdfExport {
        let mut doc = PdfExport::new("Mesh Report");
        let mut p = PdfPage::new(PdfPageSize::a4());
        p.append_text("Hello, world!");
        doc.add_page(p);
        doc
    }

    #[test]
    fn test_page_count() {
        /* page count is correct */
        assert_eq!(sample_doc().page_count(), 1);
    }

    #[test]
    fn test_validate_valid() {
        /* valid document passes */
        assert!(validate_pdf(&sample_doc()));
    }

    #[test]
    fn test_validate_empty_title() {
        /* empty title fails validation */
        let doc = PdfExport::new("");
        assert!(!validate_pdf(&doc));
    }

    #[test]
    fn test_pdf_header_magic() {
        /* PDF header starts with correct magic */
        let h = pdf_header_bytes();
        assert_eq!(&h[..4], b"%PDF");
    }

    #[test]
    fn test_estimate_bytes_positive() {
        /* estimated size is positive */
        assert!(estimate_pdf_bytes(&sample_doc()) > 0);
    }

    #[test]
    fn test_metadata_json_contains_title() {
        /* metadata JSON contains title */
        let json = pdf_metadata_json(&sample_doc());
        assert!(json.contains("Mesh Report"));
    }

    #[test]
    fn test_page_has_content() {
        /* page with text has content */
        let mut p = PdfPage::new(PdfPageSize::a4());
        p.append_text("test");
        assert!(p.has_content());
    }

    #[test]
    fn test_a4_area() {
        /* A4 page area is reasonable */
        let a4 = PdfPageSize::a4();
        assert!(a4.area() > 100_000.0);
    }
}
