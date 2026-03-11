// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! KML format for geographic data.

#[derive(Debug, Clone)]
pub struct KmlPlacemark {
    pub name: String,
    pub description: String,
    pub lon: f64,
    pub lat: f64,
    pub alt: f64,
}

#[derive(Debug, Clone)]
pub struct KmlDocument {
    pub name: String,
    pub placemarks: Vec<KmlPlacemark>,
}

pub fn new_kml_document(name: &str) -> KmlDocument {
    KmlDocument {
        name: name.to_string(),
        placemarks: Vec::new(),
    }
}

pub fn add_kml_placemark(
    doc: &mut KmlDocument,
    name: &str,
    desc: &str,
    lon: f64,
    lat: f64,
    alt: f64,
) {
    doc.placemarks.push(KmlPlacemark {
        name: name.to_string(),
        description: desc.to_string(),
        lon,
        lat,
        alt,
    });
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn render_kml(doc: &KmlDocument) -> String {
    let mut s = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
        <kml xmlns=\"http://www.opengis.net/kml/2.2\">\n<Document>\n"
        .to_string();
    s.push_str(&format!("  <name>{}</name>\n", xml_escape(&doc.name)));
    for pm in &doc.placemarks {
        s.push_str(&format!(
            "  <Placemark>\n    <name>{}</name>\n    <description>{}</description>\n\
             <Point><coordinates>{},{},{}</coordinates></Point>\n  </Placemark>\n",
            xml_escape(&pm.name),
            xml_escape(&pm.description),
            pm.lon,
            pm.lat,
            pm.alt
        ));
    }
    s.push_str("</Document>\n</kml>\n");
    s
}

pub fn export_kml(doc: &KmlDocument) -> Vec<u8> {
    render_kml(doc).into_bytes()
}
pub fn kml_placemark_count(doc: &KmlDocument) -> usize {
    doc.placemarks.len()
}
pub fn validate_kml(doc: &KmlDocument) -> bool {
    !doc.name.is_empty()
}
pub fn kml_size_bytes(doc: &KmlDocument) -> usize {
    render_kml(doc).len()
}

pub fn body_scan_to_kml(scan_name: &str, lon: f64, lat: f64) -> KmlDocument {
    let mut doc = new_kml_document(scan_name);
    add_kml_placemark(&mut doc, "Scan Location", "Body scan point", lon, lat, 0.0);
    doc
}

pub fn kml_to_geojson_stub(doc: &KmlDocument) -> String {
    format!(
        "{{\"type\":\"FeatureCollection\",\"source\":\"kml\",\"name\":\"{}\"}}",
        doc.name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_kml_document() {
        let d = new_kml_document("Test");
        assert_eq!(d.name, "Test");
    }

    #[test]
    fn test_add_kml_placemark() {
        let mut d = new_kml_document("T");
        add_kml_placemark(&mut d, "P1", "Desc", 139.0, 35.0, 0.0);
        assert_eq!(kml_placemark_count(&d), 1);
    }

    #[test]
    fn test_render_kml_contains_kml_tag() {
        let d = new_kml_document("T");
        let s = render_kml(&d);
        assert!(s.contains("<kml"));
    }

    #[test]
    fn test_render_kml_contains_placemark() {
        let mut d = new_kml_document("T");
        add_kml_placemark(&mut d, "P", "D", 0.0, 0.0, 0.0);
        let s = render_kml(&d);
        assert!(s.contains("Placemark"));
    }

    #[test]
    fn test_export_kml_nonempty() {
        let d = new_kml_document("T");
        assert!(!export_kml(&d).is_empty());
    }

    #[test]
    fn test_validate_kml() {
        let d = new_kml_document("V");
        assert!(validate_kml(&d));
    }

    #[test]
    fn test_body_scan_to_kml() {
        let d = body_scan_to_kml("Scan01", 139.0, 35.0);
        assert_eq!(kml_placemark_count(&d), 1);
    }

    #[test]
    fn test_kml_size_bytes() {
        let d = new_kml_document("T");
        assert!(kml_size_bytes(&d) > 0);
    }
}
