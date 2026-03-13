// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! EPUB document stub export.

/// An EPUB chapter.
#[derive(Debug, Clone)]
pub struct EpubChapter {
    pub title: String,
    pub content_html: String,
    pub file_name: String,
}

impl EpubChapter {
    /// Create a new chapter.
    pub fn new(title: &str, content_html: &str) -> Self {
        let file_name = format!("{}.xhtml", title.to_lowercase().replace(' ', "_"));
        Self {
            title: title.to_string(),
            content_html: content_html.to_string(),
            file_name,
        }
    }

    /// Content length in bytes.
    pub fn content_bytes(&self) -> usize {
        self.content_html.len()
    }
}

/// EPUB document metadata.
#[derive(Debug, Clone)]
pub struct EpubMeta {
    pub title: String,
    pub author: String,
    pub language: String,
    pub identifier: String,
}

impl Default for EpubMeta {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            author: "Unknown".to_string(),
            language: "en".to_string(),
            identifier: "urn:uuid:00000000-0000-0000-0000-000000000000".to_string(),
        }
    }
}

/// EPUB export document.
#[derive(Debug, Clone)]
pub struct EpubExport {
    pub meta: EpubMeta,
    pub chapters: Vec<EpubChapter>,
}

impl EpubExport {
    /// Create a new EPUB document.
    pub fn new(meta: EpubMeta) -> Self {
        Self {
            meta,
            chapters: Vec::new(),
        }
    }

    /// Add a chapter.
    pub fn add_chapter(&mut self, chapter: EpubChapter) {
        self.chapters.push(chapter);
    }

    /// Chapter count.
    pub fn chapter_count(&self) -> usize {
        self.chapters.len()
    }

    /// Total content length in bytes.
    pub fn total_content_bytes(&self) -> usize {
        self.chapters.iter().map(|c| c.content_bytes()).sum()
    }
}

/// Serialize OPF manifest (stub).
pub fn opf_manifest_stub(doc: &EpubExport) -> String {
    let items: String = doc
        .chapters
        .iter()
        .enumerate()
        .map(|(i, c)| {
            format!(
                "<item id=\"ch{}\" href=\"{}\" media-type=\"application/xhtml+xml\"/>",
                i, c.file_name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("<manifest>\n{}\n</manifest>", items)
}

/// Validate EPUB document.
pub fn validate_epub(doc: &EpubExport) -> bool {
    !doc.meta.title.is_empty() && !doc.chapters.is_empty()
}

/// Serialize EPUB metadata to JSON (stub).
pub fn epub_metadata_json(doc: &EpubExport) -> String {
    format!(
        "{{\"title\":\"{}\",\"author\":\"{}\",\"chapters\":{}}}",
        doc.meta.title,
        doc.meta.author,
        doc.chapter_count()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> EpubExport {
        let meta = EpubMeta {
            title: "OxiHuman Guide".into(),
            author: "KitaSan".into(),
            language: "en".into(),
            identifier: "urn:uuid:1234".into(),
        };
        let mut doc = EpubExport::new(meta);
        doc.add_chapter(EpubChapter::new("Introduction", "<p>Hello!</p>"));
        doc.add_chapter(EpubChapter::new("Mesh Basics", "<p>Meshes...</p>"));
        doc
    }

    #[test]
    fn test_chapter_count() {
        /* chapter count is correct */
        assert_eq!(sample_doc().chapter_count(), 2);
    }

    #[test]
    fn test_total_content_bytes() {
        /* total content bytes sums chapter content */
        let d = sample_doc();
        assert!(d.total_content_bytes() > 0);
    }

    #[test]
    fn test_validate_valid() {
        /* valid document passes */
        assert!(validate_epub(&sample_doc()));
    }

    #[test]
    fn test_validate_empty_title() {
        /* empty title fails validation */
        let meta = EpubMeta {
            title: "".into(),
            ..Default::default()
        };
        let mut doc = EpubExport::new(meta);
        doc.add_chapter(EpubChapter::new("Ch1", "content"));
        assert!(!validate_epub(&doc));
    }

    #[test]
    fn test_opf_manifest_stub() {
        /* OPF manifest includes chapter filenames */
        let d = sample_doc();
        let opf = opf_manifest_stub(&d);
        assert!(opf.contains("application/xhtml+xml"));
    }

    #[test]
    fn test_metadata_json_contains_title() {
        /* metadata JSON contains title */
        let json = epub_metadata_json(&sample_doc());
        assert!(json.contains("OxiHuman Guide"));
    }

    #[test]
    fn test_chapter_filename() {
        /* chapter filename is derived from title */
        let c = EpubChapter::new("Mesh Basics", "content");
        assert!(c.file_name.contains("mesh_basics"));
    }

    #[test]
    fn test_empty_document_invalid() {
        /* document with no chapters fails validation */
        let meta = EpubMeta {
            title: "Valid Title".into(),
            ..Default::default()
        };
        let doc = EpubExport::new(meta);
        assert!(!validate_epub(&doc));
    }
}
