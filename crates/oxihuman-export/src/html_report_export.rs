// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! HTML body measurement report export.

#[derive(Debug, Clone)]
pub struct HtmlBodyReport {
    pub title: String,
    pub measurements: Vec<(String, String, String)>, /* name, value, unit */
    pub notes: Vec<String>,
}

pub fn new_html_body_report(title: &str) -> HtmlBodyReport {
    HtmlBodyReport {
        title: title.to_string(),
        measurements: Vec::new(),
        notes: Vec::new(),
    }
}

pub fn add_measurement(report: &mut HtmlBodyReport, name: &str, value: f32, unit: &str) {
    report
        .measurements
        .push((name.to_string(), format!("{:.2}", value), unit.to_string()));
}

pub fn add_note(report: &mut HtmlBodyReport, note: &str) {
    report.notes.push(note.to_string());
}

fn html_escape_str(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn render_html_report(report: &HtmlBodyReport) -> String {
    let mut html = format!(
        "<!DOCTYPE html><html><head><meta charset=\"UTF-8\"><title>{}</title></head><body>\n",
        html_escape_str(&report.title)
    );
    html.push_str(&format!("<h1>{}</h1>\n", html_escape_str(&report.title)));
    html.push_str("<table border=\"1\"><tr><th>Measurement</th><th>Value</th><th>Unit</th></tr>\n");
    for (name, value, unit) in &report.measurements {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            html_escape_str(name),
            html_escape_str(value),
            html_escape_str(unit)
        ));
    }
    html.push_str("</table>\n");
    if !report.notes.is_empty() {
        html.push_str("<ul>\n");
        for note in &report.notes {
            html.push_str(&format!("<li>{}</li>\n", html_escape_str(note)));
        }
        html.push_str("</ul>\n");
    }
    html.push_str("</body></html>\n");
    html
}

pub fn export_html_body_report(report: &HtmlBodyReport) -> Vec<u8> {
    render_html_report(report).into_bytes()
}

pub fn measurement_count(report: &HtmlBodyReport) -> usize {
    report.measurements.len()
}
pub fn note_count(report: &HtmlBodyReport) -> usize {
    report.notes.len()
}
pub fn validate_html_report(report: &HtmlBodyReport) -> bool {
    !report.title.is_empty()
}
pub fn html_report_size_bytes(report: &HtmlBodyReport) -> usize {
    render_html_report(report).len()
}

pub fn default_html_body_report(height_cm: f32, weight_kg: f32) -> HtmlBodyReport {
    let mut r = new_html_body_report("Body Report");
    add_measurement(&mut r, "Height", height_cm, "cm");
    add_measurement(&mut r, "Weight", weight_kg, "kg");
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_html_body_report() {
        let r = new_html_body_report("Test");
        assert_eq!(r.title, "Test");
    }

    #[test]
    fn test_add_measurement() {
        let mut r = new_html_body_report("T");
        add_measurement(&mut r, "Height", 170.0, "cm");
        assert_eq!(measurement_count(&r), 1);
    }

    #[test]
    fn test_render_contains_doctype() {
        let r = new_html_body_report("X");
        let html = render_html_report(&r);
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_render_contains_title() {
        let r = new_html_body_report("MyTitle");
        let html = render_html_report(&r);
        assert!(html.contains("MyTitle"));
    }

    #[test]
    fn test_export_bytes_nonempty() {
        let r = new_html_body_report("T");
        assert!(!export_html_body_report(&r).is_empty());
    }

    #[test]
    fn test_validate_html_report() {
        let r = new_html_body_report("V");
        assert!(validate_html_report(&r));
    }

    #[test]
    fn test_note_count() {
        let mut r = new_html_body_report("T");
        add_note(&mut r, "Note 1");
        assert_eq!(note_count(&r), 1);
    }

    #[test]
    fn test_default_html_body_report() {
        let r = default_html_body_report(165.0, 55.0);
        assert_eq!(measurement_count(&r), 2);
    }
}
