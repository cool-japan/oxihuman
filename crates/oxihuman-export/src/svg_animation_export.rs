// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! SVG SMIL animation export stub.

/// A single SMIL animation element.
pub struct SmilAnimElement {
    pub attribute_name: String,
    pub from: String,
    pub to: String,
    pub duration_ms: u32,
    pub repeat_count: String,
}

/// An SVG SMIL animation document.
pub struct SvgAnimDocument {
    pub elements: Vec<SmilAnimElement>,
    pub width: u32,
    pub height: u32,
}

/// Create a new SVG animation document.
pub fn new_svg_anim_document(width: u32, height: u32) -> SvgAnimDocument {
    SvgAnimDocument {
        elements: Vec::new(),
        width,
        height,
    }
}

/// Add a SMIL animation element.
pub fn add_smil_element(
    doc: &mut SvgAnimDocument,
    attr: &str,
    from: &str,
    to: &str,
    duration_ms: u32,
    repeat: &str,
) {
    doc.elements.push(SmilAnimElement {
        attribute_name: attr.to_string(),
        from: from.to_string(),
        to: to.to_string(),
        duration_ms,
        repeat_count: repeat.to_string(),
    });
}

/// Total animation duration (max of all elements).
pub fn total_anim_duration_ms(doc: &SvgAnimDocument) -> u32 {
    doc.elements
        .iter()
        .map(|e| e.duration_ms)
        .max()
        .unwrap_or(0)
}

/// Element count.
pub fn anim_element_count(doc: &SvgAnimDocument) -> usize {
    doc.elements.len()
}

/// Render to SVG string stub.
pub fn render_svg_anim(doc: &SvgAnimDocument) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">{} elements</svg>"#,
        doc.width,
        doc.height,
        doc.elements.len()
    )
}

/// Validate document (non-zero dimensions).
pub fn validate_svg_anim(doc: &SvgAnimDocument) -> bool {
    doc.width > 0 && doc.height > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_doc_empty() {
        let doc = new_svg_anim_document(800, 600);
        assert_eq!(anim_element_count(&doc), 0 /* empty */);
    }

    #[test]
    fn add_element_increments_count() {
        let mut doc = new_svg_anim_document(100, 100);
        add_smil_element(&mut doc, "opacity", "0", "1", 1000, "indefinite");
        assert_eq!(anim_element_count(&doc), 1 /* one element */);
    }

    #[test]
    fn total_duration_zero_when_empty() {
        let doc = new_svg_anim_document(100, 100);
        assert_eq!(total_anim_duration_ms(&doc), 0 /* empty */);
    }

    #[test]
    fn total_duration_max_of_elements() {
        let mut doc = new_svg_anim_document(100, 100);
        add_smil_element(&mut doc, "x", "0", "100", 500, "1");
        add_smil_element(&mut doc, "y", "0", "50", 1000, "1");
        assert_eq!(total_anim_duration_ms(&doc), 1000 /* max is 1000 */);
    }

    #[test]
    fn validate_valid_doc() {
        let doc = new_svg_anim_document(800, 600);
        assert!(validate_svg_anim(&doc) /* valid */);
    }

    #[test]
    fn validate_zero_width_fails() {
        let doc = new_svg_anim_document(0, 600);
        assert!(!validate_svg_anim(&doc) /* invalid */);
    }

    #[test]
    fn render_contains_dimensions() {
        let doc = new_svg_anim_document(200, 150);
        let svg = render_svg_anim(&doc);
        assert!(svg.contains("200") /* width */);
        assert!(svg.contains("150") /* height */);
    }

    #[test]
    fn element_fields_stored() {
        let mut doc = new_svg_anim_document(100, 100);
        add_smil_element(&mut doc, "fill", "red", "blue", 2000, "3");
        let e = &doc.elements[0];
        assert_eq!(e.attribute_name, "fill" /* attr name */);
        assert_eq!(e.duration_ms, 2000 /* duration */);
    }

    #[test]
    fn multiple_elements_independent() {
        let mut doc = new_svg_anim_document(500, 500);
        for i in 0..5 {
            add_smil_element(&mut doc, "x", "0", &i.to_string(), i * 100, "1");
        }
        assert_eq!(anim_element_count(&doc), 5 /* five elements */);
    }
}
