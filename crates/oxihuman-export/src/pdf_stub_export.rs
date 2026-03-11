// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! PDF generation stub: cross-reference table + content stream.

/// A minimal PDF object (number + content bytes).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PdfObject {
    pub number: u32,
    pub content: Vec<u8>,
}

/// A minimal PDF document stub.
#[allow(dead_code)]
pub struct PdfStub {
    pub title: String,
    pub author: String,
    pub objects: Vec<PdfObject>,
    pub page_width_pt: f32,
    pub page_height_pt: f32,
}

/// Create a new PDF stub.
#[allow(dead_code)]
pub fn new_pdf_stub(title: &str, author: &str) -> PdfStub {
    PdfStub {
        title: title.to_string(),
        author: author.to_string(),
        objects: Vec::new(),
        page_width_pt: 595.0,
        page_height_pt: 842.0,
    }
}

/// Add a raw content stream (e.g. drawing commands) as a PDF object.
#[allow(dead_code)]
pub fn add_content_stream(stub: &mut PdfStub, content: &str) -> u32 {
    let number = (stub.objects.len() + 1) as u32;
    stub.objects.push(PdfObject {
        number,
        content: content.as_bytes().to_vec(),
    });
    number
}

/// Serialize the stub to a minimal PDF byte sequence.
#[allow(dead_code)]
pub fn export_pdf_stub(stub: &PdfStub) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n");
    let mut offsets: Vec<usize> = Vec::new();
    for obj in &stub.objects {
        offsets.push(out.len());
        let header = format!(
            "{} 0 obj\n<< /Length {} >>\nstream\n",
            obj.number,
            obj.content.len()
        );
        out.extend_from_slice(header.as_bytes());
        out.extend_from_slice(&obj.content);
        out.extend_from_slice(b"\nendstream\nendobj\n");
    }
    let xref_offset = out.len();
    out.extend_from_slice(b"xref\n");
    let total = stub.objects.len() + 1;
    out.extend_from_slice(format!("0 {}\n", total).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for &off in &offsets {
        out.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    let trailer = format!(
        "trailer\n<< /Size {} /Root 1 0 R /Info << /Title ({}) /Author ({}) >> >>\nstartxref\n{}\n%%EOF\n",
        total, stub.title, stub.author, xref_offset
    );
    out.extend_from_slice(trailer.as_bytes());
    out
}

/// Object count.
#[allow(dead_code)]
pub fn pdf_object_count(stub: &PdfStub) -> usize {
    stub.objects.len()
}

/// Estimated file size in bytes (of the serialized output).
#[allow(dead_code)]
pub fn pdf_estimated_size(stub: &PdfStub) -> usize {
    export_pdf_stub(stub).len()
}

/// Add a simple text drawing command stream.
#[allow(dead_code)]
pub fn add_text_stream(stub: &mut PdfStub, text: &str, x: f32, y: f32) -> u32 {
    let stream = format!("BT /F1 12 Tf {:.2} {:.2} Td ({}) Tj ET", x, y, text);
    add_content_stream(stub, &stream)
}

/// Check that the PDF output starts with the PDF magic bytes.
#[allow(dead_code)]
pub fn is_valid_pdf_header(bytes: &[u8]) -> bool {
    bytes.starts_with(b"%PDF-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_starts_with_magic() {
        let stub = new_pdf_stub("Test", "Author");
        let bytes = export_pdf_stub(&stub);
        assert!(is_valid_pdf_header(&bytes));
    }

    #[test]
    fn pdf_ends_with_eof() {
        let stub = new_pdf_stub("Test", "Author");
        let bytes = export_pdf_stub(&stub);
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("%%EOF"));
    }

    #[test]
    fn add_content_increases_count() {
        let mut stub = new_pdf_stub("Test", "Author");
        add_content_stream(&mut stub, "q Q");
        assert_eq!(pdf_object_count(&stub), 1);
    }

    #[test]
    fn object_numbers_sequential() {
        let mut stub = new_pdf_stub("Test", "Author");
        let n1 = add_content_stream(&mut stub, "q Q");
        let n2 = add_content_stream(&mut stub, "q Q");
        assert_eq!(n2, n1 + 1);
    }

    #[test]
    fn pdf_contains_xref() {
        let stub = new_pdf_stub("Test", "Author");
        let bytes = export_pdf_stub(&stub);
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("xref"));
    }

    #[test]
    fn pdf_contains_title() {
        let stub = new_pdf_stub("MyTitle", "Author");
        let bytes = export_pdf_stub(&stub);
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("MyTitle"));
    }

    #[test]
    fn pdf_estimated_size_positive() {
        let stub = new_pdf_stub("Test", "Author");
        assert!(pdf_estimated_size(&stub) > 0);
    }

    #[test]
    fn add_text_stream_creates_object() {
        let mut stub = new_pdf_stub("Test", "Author");
        add_text_stream(&mut stub, "Hello", 100.0, 700.0);
        assert_eq!(pdf_object_count(&stub), 1);
    }

    #[test]
    fn stream_content_in_output() {
        let mut stub = new_pdf_stub("Test", "Author");
        add_content_stream(&mut stub, "q Q");
        let bytes = export_pdf_stub(&stub);
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("q Q"));
    }

    #[test]
    fn empty_stub_valid_header() {
        let stub = new_pdf_stub("Empty", "Nobody");
        let bytes = export_pdf_stub(&stub);
        assert!(is_valid_pdf_header(&bytes));
    }
}
