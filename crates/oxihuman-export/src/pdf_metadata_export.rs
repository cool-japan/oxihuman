// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! PDF metadata struct (no rendering, just metadata).

#[derive(Debug, Clone)]
pub struct PdfMetadata {
    pub title: String,
    pub author: String,
    pub subject: String,
    pub keywords: Vec<String>,
    pub creator: String,
    pub producer: String,
    pub creation_date: String,
    pub page_count: usize,
}

pub fn new_pdf_metadata(title: &str, author: &str) -> PdfMetadata {
    PdfMetadata {
        title: title.to_string(),
        author: author.to_string(),
        subject: String::new(),
        keywords: Vec::new(),
        creator: "OxiHuman".to_string(),
        producer: "OxiHuman PDF".to_string(),
        creation_date: String::new(),
        page_count: 1,
    }
}

pub fn pdf_add_keyword(meta: &mut PdfMetadata, kw: &str) {
    meta.keywords.push(kw.to_string());
}

pub fn pdf_set_subject(meta: &mut PdfMetadata, subject: &str) {
    meta.subject = subject.to_string();
}

pub fn pdf_set_page_count(meta: &mut PdfMetadata, n: usize) {
    meta.page_count = n;
}

pub fn pdf_metadata_to_xmp(meta: &PdfMetadata) -> String {
    format!(
        "<?xpacket begin='' id='W5M0MpCehiHzreSzNTczkc9d'?>\n\
         <x:xmpmeta xmlns:x='adobe:ns:meta/'>\n\
           <rdf:RDF xmlns:rdf='http://www.w3.org/1999/02/22-rdf-syntax-ns#'>\n\
             <rdf:Description>\n\
               <dc:title>{}</dc:title>\n\
               <dc:creator>{}</dc:creator>\n\
             </rdf:Description>\n\
           </rdf:RDF>\n\
         </x:xmpmeta>\n\
         <?xpacket end='w'?>",
        meta.title, meta.author
    )
}

pub fn pdf_metadata_to_info_dict(meta: &PdfMetadata) -> String {
    format!(
        "/Title ({})\n/Author ({})\n/Subject ({})\n/Keywords ({})\n/Creator ({})\n/Producer ({})",
        meta.title,
        meta.author,
        meta.subject,
        meta.keywords.join(", "),
        meta.creator,
        meta.producer
    )
}

pub fn validate_pdf_metadata(meta: &PdfMetadata) -> bool {
    !meta.title.is_empty() && !meta.author.is_empty() && meta.page_count > 0
}

pub fn pdf_keyword_count(meta: &PdfMetadata) -> usize {
    meta.keywords.len()
}

pub fn default_pdf_metadata_for_body_report() -> PdfMetadata {
    let mut m = new_pdf_metadata("Body Measurement Report", "OxiHuman");
    pdf_set_subject(&mut m, "Human body measurements");
    pdf_add_keyword(&mut m, "body");
    pdf_add_keyword(&mut m, "measurement");
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pdf_metadata() {
        let m = new_pdf_metadata("T", "A");
        assert_eq!(m.title, "T");
        assert_eq!(m.author, "A");
    }

    #[test]
    fn test_pdf_add_keyword() {
        let mut m = new_pdf_metadata("T", "A");
        pdf_add_keyword(&mut m, "kw");
        assert_eq!(pdf_keyword_count(&m), 1);
    }

    #[test]
    fn test_validate_pdf_metadata() {
        let m = new_pdf_metadata("T", "A");
        assert!(validate_pdf_metadata(&m));
    }

    #[test]
    fn test_validate_empty_title_fails() {
        let m = new_pdf_metadata("", "A");
        assert!(!validate_pdf_metadata(&m));
    }

    #[test]
    fn test_pdf_metadata_to_xmp() {
        let m = new_pdf_metadata("Title", "Author");
        let xmp = pdf_metadata_to_xmp(&m);
        assert!(xmp.contains("Title"));
    }

    #[test]
    fn test_pdf_metadata_to_info_dict() {
        let m = new_pdf_metadata("T", "A");
        let info = pdf_metadata_to_info_dict(&m);
        assert!(info.contains("/Title"));
    }

    #[test]
    fn test_pdf_set_page_count() {
        let mut m = new_pdf_metadata("T", "A");
        pdf_set_page_count(&mut m, 5);
        assert_eq!(m.page_count, 5);
    }

    #[test]
    fn test_default_pdf_metadata() {
        let m = default_pdf_metadata_for_body_report();
        assert!(pdf_keyword_count(&m) >= 2);
    }
}
