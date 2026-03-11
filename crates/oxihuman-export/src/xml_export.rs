// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Generic XML export for mesh/scene data.

/// An XML element with tag, attributes, and children.
#[derive(Debug, Clone, Default)]
pub struct XmlElement {
    pub tag: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<XmlElement>,
    pub text: Option<String>,
}

impl XmlElement {
    pub fn new(tag: &str) -> Self {
        Self { tag: tag.into(), ..Default::default() }
    }
    pub fn set_attr(&mut self, key: &str, val: &str) {
        self.attributes.push((key.into(), val.into()));
    }
    pub fn add_child(&mut self, child: XmlElement) {
        self.children.push(child);
    }
}

/// An XML document.
#[derive(Debug, Clone, Default)]
pub struct XmlDocument {
    pub root: Option<XmlElement>,
    pub version: String,
    pub encoding: String,
}

/// Create a new XML document.
pub fn new_xml_document(root_tag: &str) -> XmlDocument {
    XmlDocument {
        root: Some(XmlElement::new(root_tag)),
        version: "1.0".into(),
        encoding: "UTF-8".into(),
    }
}

/// Add an attribute to the root element.
pub fn add_root_attr(doc: &mut XmlDocument, key: &str, val: &str) {
    if let Some(ref mut root) = doc.root {
        root.set_attr(key, val);
    }
}

/// Add a child element to the root.
pub fn add_root_child(doc: &mut XmlDocument, child: XmlElement) {
    if let Some(ref mut root) = doc.root {
        root.add_child(child);
    }
}

/// Serialize an XML element to string (indented).
pub fn element_to_string(el: &XmlElement, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let attrs: String = el.attributes.iter().map(|(k, v)| format!(" {}=\"{}\"", xml_escape(k), xml_escape(v))).collect();
    if el.children.is_empty() && el.text.is_none() {
        return format!("{}<{}{}/>\n", pad, el.tag, attrs);
    }
    let mut s = format!("{}<{}{}>\n", pad, el.tag, attrs);
    if let Some(ref t) = el.text {
        s.push_str(&format!("{}{}\n", " ".repeat(indent + 2), xml_escape(t)));
    }
    for child in &el.children {
        s.push_str(&element_to_string(child, indent + 2));
    }
    s.push_str(&format!("{}</{}>\n", pad, el.tag));
    s
}

/// Export an XML document to string.
pub fn export_xml(doc: &XmlDocument) -> String {
    let mut s = format!("<?xml version=\"{}\" encoding=\"{}\"?>\n", doc.version, doc.encoding);
    if let Some(ref root) = doc.root {
        s.push_str(&element_to_string(root, 0));
    }
    s
}

/// Escape special XML characters.
pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Count child elements of the root.
pub fn root_child_count(doc: &XmlDocument) -> usize {
    doc.root.as_ref().map_or(0, |r| r.children.len())
}

/// Export mesh positions as an XML document.
pub fn mesh_to_xml(positions: &[[f32; 3]], indices: &[u32]) -> XmlDocument {
    let mut doc = new_xml_document("mesh");
    add_root_attr(&mut doc, "vertices", &positions.len().to_string());
    add_root_attr(&mut doc, "triangles", &(indices.len() / 3).to_string());
    let mut verts_el = XmlElement::new("vertices");
    for (i, p) in positions.iter().enumerate() {
        let mut v = XmlElement::new("v");
        v.set_attr("id", &i.to_string());
        v.set_attr("x", &format!("{:.6}", p[0]));
        v.set_attr("y", &format!("{:.6}", p[1]));
        v.set_attr("z", &format!("{:.6}", p[2]));
        verts_el.add_child(v);
    }
    add_root_child(&mut doc, verts_el);
    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_xml_document_has_root() {
        /* new document has a root element */
        let doc = new_xml_document("root");
        assert!(doc.root.is_some());
    }

    #[test]
    fn test_export_xml_has_declaration() {
        /* exported XML has XML declaration */
        let doc = new_xml_document("data");
        let s = export_xml(&doc);
        assert!(s.starts_with("<?xml"));
    }

    #[test]
    fn test_add_root_attr() {
        /* root attribute appears in output */
        let mut doc = new_xml_document("mesh");
        add_root_attr(&mut doc, "units", "mm");
        let s = export_xml(&doc);
        assert!(s.contains("units"));
    }

    #[test]
    fn test_add_root_child() {
        /* added child increases count */
        let mut doc = new_xml_document("root");
        add_root_child(&mut doc, XmlElement::new("child"));
        assert_eq!(root_child_count(&doc), 1);
    }

    #[test]
    fn test_xml_escape_ampersand() {
        /* & is escaped to &amp; */
        assert_eq!(xml_escape("a&b"), "a&amp;b");
    }

    #[test]
    fn test_xml_escape_less_than() {
        /* < is escaped to &lt; */
        assert_eq!(xml_escape("a<b"), "a&lt;b");
    }

    #[test]
    fn test_element_self_closing() {
        /* element with no children/text is self-closing */
        let el = XmlElement::new("empty");
        let s = element_to_string(&el, 0);
        assert!(s.contains("/>"));
    }

    #[test]
    fn test_mesh_to_xml_vertex_count() {
        /* mesh_to_xml encodes vertex count in attribute */
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]];
        let i = vec![0u32,1,2];
        let doc = mesh_to_xml(&p, &i);
        let s = export_xml(&doc);
        assert!(s.contains("vertices=\"3\""));
    }

    #[test]
    fn test_root_child_count_zero() {
        /* no children at start */
        let doc = new_xml_document("x");
        assert_eq!(root_child_count(&doc), 0);
    }

    #[test]
    fn test_export_xml_contains_root_tag() {
        /* exported XML contains root tag name */
        let doc = new_xml_document("scene");
        let s = export_xml(&doc);
        assert!(s.contains("<scene"));
    }
}
