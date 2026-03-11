// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A recursive-descent XML parser supporting elements, attributes, namespaces,
//! CDATA, comments, processing instructions, and character references.

#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single XML attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct XmlAttr {
    pub name: String,
    pub namespace: Option<String>,
    pub value: String,
}

/// An XML declaration (`<?xml version="1.0" encoding="utf-8"?>`).
#[derive(Debug, Clone, PartialEq)]
pub struct XmlDeclaration {
    pub version: String,
    pub encoding: Option<String>,
    pub standalone: Option<bool>,
}

/// A node in the XML tree.
#[derive(Debug, Clone, PartialEq)]
pub enum XmlNode {
    Element {
        name: String,
        namespace: Option<String>,
        attributes: Vec<XmlAttr>,
        children: Vec<XmlNode>,
    },
    Text(String),
    CData(String),
    Comment(String),
    ProcessingInstruction {
        target: String,
        data: String,
    },
}

/// A parsed XML document.
#[derive(Debug, Clone, PartialEq)]
pub struct XmlDocument {
    pub declaration: Option<XmlDeclaration>,
    pub root: XmlNode,
}

/// A parse error with line/column info.
#[derive(Debug, Clone, PartialEq)]
pub struct XmlParseError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl fmt::Display for XmlParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "XML parse error at {}:{}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for XmlParseError {}

// ---------------------------------------------------------------------------
// Parser state
// ---------------------------------------------------------------------------

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
    ns_stack: Vec<Vec<(String, String)>>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
            ns_stack: Vec::new(),
        }
    }

    fn remaining(&self) -> usize {
        self.input.len().saturating_sub(self.pos)
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.input.len());
    }

    fn starts_with(&self, s: &[u8]) -> bool {
        self.input
            .get(self.pos..)
            .map_or(false, |r| r.starts_with(s))
    }

    fn line_col(&self) -> (usize, usize) {
        let slice = &self.input[..self.pos.min(self.input.len())];
        let line = slice.iter().filter(|&&b| b == b'\n').count() + 1;
        let col = match slice.iter().rposition(|&b| b == b'\n') {
            Some(p) => self.pos - p,
            None => self.pos + 1,
        };
        (line, col)
    }

    fn error(&self, msg: impl Into<String>) -> XmlParseError {
        let (line, column) = self.line_col();
        XmlParseError {
            line,
            column,
            message: msg.into(),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek() {
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.advance(1);
            } else {
                break;
            }
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), XmlParseError> {
        match self.peek() {
            Some(b) if b == expected => {
                self.advance(1);
                Ok(())
            }
            Some(b) => Err(self.error(format!(
                "expected '{}', found '{}'",
                expected as char, b as char
            ))),
            None => Err(self.error(format!(
                "expected '{}', found end of input",
                expected as char
            ))),
        }
    }

    fn read_while(&mut self, pred: impl Fn(u8) -> bool) -> &'a [u8] {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if pred(b) {
                self.advance(1);
            } else {
                break;
            }
        }
        &self.input[start..self.pos]
    }

    fn slice_str(&self, start: usize, end: usize) -> Result<&'a str, XmlParseError> {
        std::str::from_utf8(&self.input[start..end])
            .map_err(|_| self.error("invalid UTF-8 in input"))
    }

    fn is_name_start(b: u8) -> bool {
        b.is_ascii_alphabetic() || b == b'_' || b == b':'
    }

    fn is_name_char(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_' || b == b':' || b == b'-' || b == b'.'
    }

    fn read_name(&mut self) -> Result<String, XmlParseError> {
        let start = self.pos;
        match self.peek() {
            Some(b) if Self::is_name_start(b) => {
                self.advance(1);
            }
            _ => return Err(self.error("expected element/attribute name")),
        }
        self.read_while(|b| Self::is_name_char(b));
        let s = self.slice_str(start, self.pos)?;
        Ok(s.to_string())
    }

    fn decode_entity_ref(name: &str) -> Option<char> {
        match name {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => None,
        }
    }

    fn decode_char_ref(raw: &str) -> Option<char> {
        let code = if let Some(hex) = raw.strip_prefix('x') {
            u32::from_str_radix(hex, 16).ok()?
        } else {
            raw.parse::<u32>().ok()?
        };
        char::from_u32(code)
    }

    fn resolve_references(s: &str) -> Result<String, String> {
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '&' {
                let mut ref_buf = String::new();
                let mut terminated = false;
                for rc in chars.by_ref() {
                    if rc == ';' {
                        terminated = true;
                        break;
                    }
                    ref_buf.push(rc);
                }
                if !terminated {
                    return Err(format!("unterminated reference: &{}", ref_buf));
                }
                if let Some(rest) = ref_buf.strip_prefix('#') {
                    match Self::decode_char_ref(rest) {
                        Some(ch) => out.push(ch),
                        None => {
                            return Err(format!("invalid numeric reference: &#{};", rest));
                        }
                    }
                } else {
                    match Self::decode_entity_ref(&ref_buf) {
                        Some(ch) => out.push(ch),
                        None => {
                            return Err(format!("unknown entity: &{};", ref_buf));
                        }
                    }
                }
            } else {
                out.push(c);
            }
        }
        Ok(out)
    }

    fn read_attr_value(&mut self) -> Result<String, XmlParseError> {
        let quote = match self.peek() {
            Some(q @ b'"') | Some(q @ b'\'') => {
                self.advance(1);
                q
            }
            _ => return Err(self.error("expected quote to start attribute value")),
        };
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b == quote {
                break;
            }
            if b == b'<' {
                return Err(self.error("'<' not allowed in attribute value"));
            }
            self.advance(1);
        }
        let raw = self.slice_str(start, self.pos)?;
        self.expect_byte(quote)?;
        Self::resolve_references(raw).map_err(|e| self.error(e))
    }

    fn push_ns_scope(&mut self) {
        self.ns_stack.push(Vec::new());
    }

    fn pop_ns_scope(&mut self) {
        self.ns_stack.pop();
    }

    fn add_ns(&mut self, prefix: &str, uri: &str) {
        if let Some(scope) = self.ns_stack.last_mut() {
            scope.push((prefix.to_string(), uri.to_string()));
        }
    }

    fn resolve_ns(&self, prefix: &str) -> Option<String> {
        for scope in self.ns_stack.iter().rev() {
            for (p, u) in scope.iter().rev() {
                if p == prefix {
                    return Some(u.clone());
                }
            }
        }
        None
    }

    fn split_prefixed(name: &str) -> (Option<&str>, &str) {
        if let Some(idx) = name.find(':') {
            let prefix = &name[..idx];
            if prefix == "xmlns" {
                (None, name)
            } else {
                (Some(prefix), &name[idx + 1..])
            }
        } else {
            (None, name)
        }
    }

    fn parse_comment(&mut self) -> Result<XmlNode, XmlParseError> {
        self.advance(4);
        let start = self.pos;
        loop {
            if self.remaining() < 3 {
                return Err(self.error("unterminated comment"));
            }
            if self.starts_with(b"-->") {
                let text = self.slice_str(start, self.pos)?;
                self.advance(3);
                return Ok(XmlNode::Comment(text.to_string()));
            }
            self.advance(1);
        }
    }

    fn parse_cdata(&mut self) -> Result<XmlNode, XmlParseError> {
        self.advance(9);
        let start = self.pos;
        loop {
            if self.remaining() < 3 {
                return Err(self.error("unterminated CDATA section"));
            }
            if self.starts_with(b"]]>") {
                let text = self.slice_str(start, self.pos)?;
                self.advance(3);
                return Ok(XmlNode::CData(text.to_string()));
            }
            self.advance(1);
        }
    }

    fn parse_pi(&mut self) -> Result<XmlNode, XmlParseError> {
        self.advance(2);
        let target = self.read_name()?;
        self.skip_whitespace();
        let data_start = self.pos;
        loop {
            if self.remaining() < 2 {
                return Err(self.error("unterminated processing instruction"));
            }
            if self.starts_with(b"?>") {
                let data = self.slice_str(data_start, self.pos)?.trim().to_string();
                self.advance(2);
                return Ok(XmlNode::ProcessingInstruction { target, data });
            }
            self.advance(1);
        }
    }

    fn parse_declaration(&mut self) -> Result<XmlDeclaration, XmlParseError> {
        self.advance(5);
        let mut attrs: HashMap<String, String> = HashMap::new();
        loop {
            self.skip_whitespace();
            if self.starts_with(b"?>") {
                self.advance(2);
                break;
            }
            if self.peek().is_none() {
                return Err(self.error("unterminated XML declaration"));
            }
            let name = self.read_name()?;
            self.skip_whitespace();
            self.expect_byte(b'=')?;
            self.skip_whitespace();
            let value = self.read_attr_value()?;
            attrs.insert(name, value);
        }
        let version = attrs
            .remove("version")
            .unwrap_or_else(|| "1.0".to_string());
        let encoding = attrs.remove("encoding");
        let standalone = attrs.remove("standalone").and_then(|v| match v.as_str() {
            "yes" => Some(true),
            "no" => Some(false),
            _ => None,
        });
        Ok(XmlDeclaration {
            version,
            encoding,
            standalone,
        })
    }

    fn parse_attributes(&mut self) -> Result<(Vec<XmlAttr>, bool), XmlParseError> {
        let mut attrs = Vec::new();
        loop {
            self.skip_whitespace();
            if self.starts_with(b"/>") {
                self.advance(2);
                return Ok((attrs, true));
            }
            if self.peek() == Some(b'>') {
                self.advance(1);
                return Ok((attrs, false));
            }
            if self.peek().is_none() {
                return Err(self.error("unexpected end of input in element"));
            }
            let attr_name = self.read_name()?;
            self.skip_whitespace();
            self.expect_byte(b'=')?;
            self.skip_whitespace();
            let value = self.read_attr_value()?;

            if attr_name == "xmlns" {
                self.add_ns("", &value);
            } else if let Some(prefix) = attr_name.strip_prefix("xmlns:") {
                self.add_ns(prefix, &value);
            }

            let (ns_prefix, local) = Self::split_prefixed(&attr_name);
            let namespace = ns_prefix.and_then(|p| self.resolve_ns(p));
            attrs.push(XmlAttr {
                name: local.to_string(),
                namespace,
                value,
            });
        }
    }

    fn parse_element(&mut self) -> Result<XmlNode, XmlParseError> {
        self.expect_byte(b'<')?;
        let tag_name = self.read_name()?;

        self.push_ns_scope();

        let (attributes, self_closing) = self.parse_attributes()?;

        let (prefix, local) = Self::split_prefixed(&tag_name);
        let namespace = prefix.and_then(|p| self.resolve_ns(p));
        let elem_name = local.to_string();

        if self_closing {
            self.pop_ns_scope();
            return Ok(XmlNode::Element {
                name: elem_name,
                namespace,
                attributes,
                children: Vec::new(),
            });
        }

        let children = self.parse_children()?;

        if !self.starts_with(b"</") {
            return Err(self.error(format!("expected closing tag for <{}>", tag_name)));
        }
        self.advance(2);
        let close_name = self.read_name()?;
        if close_name != tag_name {
            return Err(self.error(format!(
                "mismatched closing tag: expected </{}>, found </{}>",
                tag_name, close_name
            )));
        }
        self.skip_whitespace();
        self.expect_byte(b'>')?;

        self.pop_ns_scope();

        Ok(XmlNode::Element {
            name: elem_name,
            namespace,
            attributes,
            children,
        })
    }

    fn parse_children(&mut self) -> Result<Vec<XmlNode>, XmlParseError> {
        let mut children = Vec::new();
        loop {
            if self.peek().is_none() || self.starts_with(b"</") {
                break;
            }
            if self.peek() == Some(b'<') {
                if self.starts_with(b"<!--") {
                    children.push(self.parse_comment()?);
                } else if self.starts_with(b"<![CDATA[") {
                    children.push(self.parse_cdata()?);
                } else if self.starts_with(b"<?") {
                    children.push(self.parse_pi()?);
                } else {
                    children.push(self.parse_element()?);
                }
            } else {
                let start = self.pos;
                while let Some(b) = self.peek() {
                    if b == b'<' {
                        break;
                    }
                    self.advance(1);
                }
                let raw = self.slice_str(start, self.pos)?;
                let text = Self::resolve_references(raw).map_err(|e| self.error(e))?;
                if !text.is_empty() {
                    children.push(XmlNode::Text(text));
                }
            }
        }
        Ok(children)
    }

    fn parse_document(&mut self) -> Result<XmlDocument, XmlParseError> {
        self.skip_whitespace();

        let mut declaration = None;

        if self.starts_with(b"<?xml ") || self.starts_with(b"<?xml?") {
            declaration = Some(self.parse_declaration()?);
            self.skip_whitespace();
        }

        loop {
            self.skip_whitespace();
            if self.starts_with(b"<!--") {
                self.parse_comment()?;
            } else if self.starts_with(b"<?") {
                self.parse_pi()?;
            } else {
                break;
            }
        }

        if self.peek() != Some(b'<') {
            return Err(self.error("expected root element"));
        }

        let root = self.parse_element()?;

        Ok(XmlDocument { declaration, root })
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse a complete XML document string.
pub fn parse_xml_document(input: &str) -> Result<XmlDocument, XmlParseError> {
    let mut parser = Parser::new(input);
    parser.parse_document()
}

/// Parse XML into an `XmlNode` tree (compatibility wrapper).
///
/// Returns `None` if parsing fails.
pub fn parse_xml_simple(input: &str) -> Option<XmlNode> {
    let input = input.trim();
    if input.is_empty() || !input.starts_with('<') {
        return None;
    }
    let mut parser = Parser::new(input);
    parser.parse_element().ok()
}

// ---------------------------------------------------------------------------
// Convenience methods on XmlNode
// ---------------------------------------------------------------------------

impl XmlNode {
    /// Return the tag/element name, or empty string for non-elements.
    pub fn tag(&self) -> &str {
        match self {
            XmlNode::Element { name, .. } => name.as_str(),
            _ => "",
        }
    }

    /// Return the direct text content of this node.
    /// For elements, concatenates immediate Text/CData children.
    pub fn text_content(&self) -> String {
        match self {
            XmlNode::Text(t) | XmlNode::CData(t) => t.clone(),
            XmlNode::Element { children, .. } => {
                let mut s = String::new();
                for c in children {
                    match c {
                        XmlNode::Text(t) | XmlNode::CData(t) => s.push_str(t),
                        _ => {}
                    }
                }
                s
            }
            _ => String::new(),
        }
    }

    /// Return element children, or empty slice for non-elements.
    pub fn children(&self) -> &[XmlNode] {
        match self {
            XmlNode::Element { children, .. } => children,
            _ => &[],
        }
    }

    /// Return element attributes, or empty slice for non-elements.
    pub fn attrs(&self) -> &[XmlAttr] {
        match self {
            XmlNode::Element { attributes, .. } => attributes,
            _ => &[],
        }
    }
}

// ---------------------------------------------------------------------------
// Legacy free-function helpers
// ---------------------------------------------------------------------------

/// Get the value of a named attribute from a node.
pub fn xml_attr<'a>(node: &'a XmlNode, name: &str) -> Option<&'a str> {
    node.attrs()
        .iter()
        .find(|a| a.name == name)
        .map(|a| a.value.as_str())
}

/// Get the first child element node with the given tag name.
pub fn xml_child<'a>(node: &'a XmlNode, tag: &str) -> Option<&'a XmlNode> {
    node.children().iter().find(|c| c.tag() == tag)
}

/// Get the text content of a node.
pub fn xml_text(node: &XmlNode) -> String {
    node.text_content()
}

/// Get the number of children of a node.
pub fn node_child_count(node: &XmlNode) -> usize {
    node.children().len()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_tag() {
        let node = parse_xml_simple("<root></root>").unwrap();
        assert_eq!(node.tag(), "root");
    }

    #[test]
    fn parse_self_closing() {
        let node = parse_xml_simple("<br/>").unwrap();
        assert_eq!(node.tag(), "br");
        assert!(node.children().is_empty());
    }

    #[test]
    fn parse_attr_value() {
        let node = parse_xml_simple(r#"<img src="photo.png"/>"#).unwrap();
        assert_eq!(xml_attr(&node, "src"), Some("photo.png"));
    }

    #[test]
    fn xml_attr_missing_returns_none() {
        let node = parse_xml_simple("<div></div>").unwrap();
        assert_eq!(xml_attr(&node, "class"), None);
    }

    #[test]
    fn parse_text_content() {
        let node = parse_xml_simple("<title>Hello World</title>").unwrap();
        assert!(xml_text(&node).contains("Hello"));
    }

    #[test]
    fn parse_invalid_returns_none() {
        assert!(parse_xml_simple("not xml").is_none());
    }

    #[test]
    fn node_child_count_zero_for_leaf() {
        let node = parse_xml_simple("<leaf/>").unwrap();
        assert_eq!(node_child_count(&node), 0);
    }

    #[test]
    fn xml_child_found() {
        let node = parse_xml_simple("<root><child></child></root>").unwrap();
        assert!(xml_child(&node, "child").is_some());
    }

    #[test]
    fn xml_child_not_found() {
        let node = parse_xml_simple("<root></root>").unwrap();
        assert!(xml_child(&node, "child").is_none());
    }

    #[test]
    fn multiple_attrs() {
        let node = parse_xml_simple(r#"<rect x="1" y="2"/>"#).unwrap();
        assert_eq!(xml_attr(&node, "x"), Some("1"));
        assert_eq!(xml_attr(&node, "y"), Some("2"));
    }

    #[test]
    fn parse_nested_elements() {
        let xml = "<a><b><c/></b></a>";
        let node = parse_xml_simple(xml).unwrap();
        let b = xml_child(&node, "b").unwrap();
        let c = xml_child(b, "c").unwrap();
        assert_eq!(c.tag(), "c");
    }

    #[test]
    fn parse_cdata() {
        let xml = "<root><![CDATA[raw <content> & stuff]]></root>";
        let node = parse_xml_simple(xml).unwrap();
        match &node.children()[0] {
            XmlNode::CData(s) => assert_eq!(s, "raw <content> & stuff"),
            other => panic!("expected CData, got {:?}", other),
        }
    }

    #[test]
    fn parse_comment_node() {
        let xml = "<root><!-- a comment --></root>";
        let node = parse_xml_simple(xml).unwrap();
        match &node.children()[0] {
            XmlNode::Comment(s) => assert_eq!(s, " a comment "),
            other => panic!("expected Comment, got {:?}", other),
        }
    }

    #[test]
    fn parse_processing_instruction() {
        let xml = "<root><?my-pi some data?></root>";
        let node = parse_xml_simple(xml).unwrap();
        match &node.children()[0] {
            XmlNode::ProcessingInstruction { target, data } => {
                assert_eq!(target, "my-pi");
                assert_eq!(data, "some data");
            }
            other => panic!("expected PI, got {:?}", other),
        }
    }

    #[test]
    fn entity_references() {
        let xml = r#"<t>&amp; &lt; &gt; &quot; &apos;</t>"#;
        let node = parse_xml_simple(xml).unwrap();
        assert_eq!(xml_text(&node), "& < > \" '");
    }

    #[test]
    fn numeric_char_ref_decimal() {
        let xml = "<t>&#65;</t>";
        let node = parse_xml_simple(xml).unwrap();
        assert_eq!(xml_text(&node), "A");
    }

    #[test]
    fn numeric_char_ref_hex() {
        let xml = "<t>&#x41;</t>";
        let node = parse_xml_simple(xml).unwrap();
        assert_eq!(xml_text(&node), "A");
    }

    #[test]
    fn namespace_basic() {
        let xml = r#"<root xmlns:ns="http://example.com"><ns:child/></root>"#;
        let node = parse_xml_simple(xml).unwrap();
        let child = xml_child(&node, "child").unwrap();
        match child {
            XmlNode::Element { namespace, .. } => {
                assert_eq!(namespace.as_deref(), Some("http://example.com"));
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn default_namespace() {
        let xml = r#"<root xmlns="http://default.ns"><child/></root>"#;
        let doc = parse_xml_document(xml).unwrap();
        assert!(doc.declaration.is_none());
    }

    #[test]
    fn xml_declaration() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?><root/>"#;
        let doc = parse_xml_document(xml).unwrap();
        let decl = doc.declaration.unwrap();
        assert_eq!(decl.version, "1.0");
        assert_eq!(decl.encoding.as_deref(), Some("utf-8"));
    }

    #[test]
    fn mixed_content() {
        let xml = "<p>Hello <b>world</b>!</p>";
        let node = parse_xml_simple(xml).unwrap();
        assert_eq!(node.children().len(), 3);
        match &node.children()[0] {
            XmlNode::Text(t) => assert_eq!(t, "Hello "),
            other => panic!("expected Text, got {:?}", other),
        }
    }

    #[test]
    fn error_reports_line_column() {
        let xml = "<root>\n  <bad\n";
        let err = parse_xml_document(xml).unwrap_err();
        assert!(err.line >= 2);
        assert!(!err.message.is_empty());
    }

    #[test]
    fn mismatched_tags_error() {
        let xml = "<a></b>";
        let err = parse_xml_document(xml).unwrap_err();
        assert!(err.message.contains("mismatched"));
    }

    #[test]
    fn self_closing_with_space() {
        let node = parse_xml_simple("<br />").unwrap();
        assert_eq!(node.tag(), "br");
    }

    #[test]
    fn attr_with_single_quotes() {
        let node = parse_xml_simple("<div class='main'/>").unwrap();
        assert_eq!(xml_attr(&node, "class"), Some("main"));
    }

    #[test]
    fn deeply_nested() {
        let xml = "<a><b><c><d><e>deep</e></d></c></b></a>";
        let node = parse_xml_simple(xml).unwrap();
        let b = xml_child(&node, "b").unwrap();
        let c = xml_child(b, "c").unwrap();
        let d = xml_child(c, "d").unwrap();
        let e = xml_child(d, "e").unwrap();
        assert_eq!(xml_text(e), "deep");
    }

    #[test]
    fn multiple_children_same_level() {
        let xml = "<root><a/><b/><c/></root>";
        let node = parse_xml_simple(xml).unwrap();
        assert_eq!(node_child_count(&node), 3);
    }

    #[test]
    fn entity_in_attribute() {
        let xml = r#"<t val="a&amp;b"/>"#;
        let node = parse_xml_simple(xml).unwrap();
        assert_eq!(xml_attr(&node, "val"), Some("a&b"));
    }

    #[test]
    fn cdata_preserves_entities() {
        let xml = "<t><![CDATA[&amp; not resolved]]></t>";
        let node = parse_xml_simple(xml).unwrap();
        assert_eq!(xml_text(&node), "&amp; not resolved");
    }

    #[test]
    fn namespace_attr() {
        let xml = r#"<root xmlns:x="urn:x"><t x:color="red"/></root>"#;
        let node = parse_xml_simple(xml).unwrap();
        let t = xml_child(&node, "t").unwrap();
        let attr = &t.attrs()[0];
        assert_eq!(attr.name, "color");
        assert_eq!(attr.namespace.as_deref(), Some("urn:x"));
    }

    #[test]
    fn standalone_declaration() {
        let xml = r#"<?xml version="1.0" standalone="yes"?><r/>"#;
        let doc = parse_xml_document(xml).unwrap();
        assert_eq!(
            doc.declaration.as_ref().and_then(|d| d.standalone),
            Some(true)
        );
    }
}
