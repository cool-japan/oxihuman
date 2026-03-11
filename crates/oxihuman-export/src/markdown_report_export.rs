// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Markdown-format human body report export.

#[derive(Debug, Clone)]
pub struct MarkdownReport {
    pub title: String,
    pub sections: Vec<MarkdownSection>,
}

#[derive(Debug, Clone)]
pub struct MarkdownSection {
    pub heading: String,
    pub rows: Vec<(String, String)>,
}

pub fn new_markdown_report(title: &str) -> MarkdownReport {
    MarkdownReport {
        title: title.to_string(),
        sections: Vec::new(),
    }
}

pub fn add_md_section(report: &mut MarkdownReport, heading: &str) {
    report.sections.push(MarkdownSection {
        heading: heading.to_string(),
        rows: Vec::new(),
    });
}

pub fn add_md_row(report: &mut MarkdownReport, key: &str, value: &str) {
    if let Some(s) = report.sections.last_mut() {
        s.rows.push((key.to_string(), value.to_string()));
    }
}

pub fn render_markdown(report: &MarkdownReport) -> String {
    let mut out = format!("# {}\n\n", report.title);
    for sec in &report.sections {
        out.push_str(&format!("## {}\n\n", sec.heading));
        out.push_str("| Key | Value |\n|-----|-------|\n");
        for (k, v) in &sec.rows {
            out.push_str(&format!("| {} | {} |\n", k, v));
        }
        out.push('\n');
    }
    out
}

pub fn md_section_count(report: &MarkdownReport) -> usize {
    report.sections.len()
}
pub fn md_row_count(report: &MarkdownReport) -> usize {
    report.sections.iter().map(|s| s.rows.len()).sum()
}

pub fn export_markdown_report(report: &MarkdownReport) -> Vec<u8> {
    render_markdown(report).into_bytes()
}

pub fn default_body_report(
    height_cm: f32,
    weight_kg: f32,
    chest_cm: f32,
    waist_cm: f32,
    hip_cm: f32,
) -> MarkdownReport {
    let mut r = new_markdown_report("Body Measurement Report");
    add_md_section(&mut r, "Measurements");
    add_md_row(&mut r, "Height (cm)", &format!("{:.1}", height_cm));
    add_md_row(&mut r, "Weight (kg)", &format!("{:.1}", weight_kg));
    add_md_row(&mut r, "Chest (cm)", &format!("{:.1}", chest_cm));
    add_md_row(&mut r, "Waist (cm)", &format!("{:.1}", waist_cm));
    add_md_row(&mut r, "Hip (cm)", &format!("{:.1}", hip_cm));
    r
}

pub fn validate_markdown_report(report: &MarkdownReport) -> bool {
    !report.title.is_empty()
}

pub fn markdown_byte_count(report: &MarkdownReport) -> usize {
    render_markdown(report).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_report() {
        let r = new_markdown_report("Test");
        assert_eq!(r.title, "Test");
    }

    #[test]
    fn test_add_section_and_row() {
        let mut r = new_markdown_report("T");
        add_md_section(&mut r, "S1");
        add_md_row(&mut r, "k", "v");
        assert_eq!(md_section_count(&r), 1);
        assert_eq!(md_row_count(&r), 1);
    }

    #[test]
    fn test_render_markdown_contains_title() {
        let r = new_markdown_report("MyReport");
        let s = render_markdown(&r);
        assert!(s.contains("MyReport"));
    }

    #[test]
    fn test_export_markdown_report() {
        let r = new_markdown_report("X");
        let bytes = export_markdown_report(&r);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_default_body_report_rows() {
        let r = default_body_report(170.0, 70.0, 90.0, 80.0, 95.0);
        assert_eq!(md_row_count(&r), 5);
    }

    #[test]
    fn test_validate_markdown_report() {
        let r = new_markdown_report("Valid");
        assert!(validate_markdown_report(&r));
    }

    #[test]
    fn test_markdown_byte_count() {
        let r = new_markdown_report("Hello");
        assert!(markdown_byte_count(&r) > 0);
    }

    #[test]
    fn test_render_table_headers() {
        let mut r = new_markdown_report("T");
        add_md_section(&mut r, "S");
        add_md_row(&mut r, "k", "v");
        let md = render_markdown(&r);
        assert!(md.contains("| Key | Value |"));
    }
}
