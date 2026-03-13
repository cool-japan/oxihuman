// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! HAL (Hypertext Application Language) JSON export.

/// A HAL link.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HalLink {
    pub rel: String,
    pub href: String,
    pub templated: bool,
}

/// A HAL resource.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HalResource {
    pub links: Vec<HalLink>,
    pub properties: Vec<(String, String)>,
    pub embedded: Vec<(String, Vec<HalResource>)>,
}

impl HalResource {
    #[allow(dead_code)]
    pub fn new() -> Self {
        HalResource {
            links: Vec::new(),
            properties: Vec::new(),
            embedded: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn add_link(&mut self, rel: &str, href: &str) {
        self.links.push(HalLink {
            rel: rel.to_string(),
            href: href.to_string(),
            templated: false,
        });
    }

    #[allow(dead_code)]
    pub fn add_property(&mut self, key: &str, val: &str) {
        self.properties.push((key.to_string(), val.to_string()));
    }

    #[allow(dead_code)]
    pub fn embed(&mut self, rel: &str, resources: Vec<HalResource>) {
        self.embedded.push((rel.to_string(), resources));
    }
}

impl Default for HalResource {
    fn default() -> Self {
        HalResource::new()
    }
}

/// Serialize a HAL resource to JSON.
#[allow(dead_code)]
pub fn serialize_hal(res: &HalResource) -> String {
    let mut parts: Vec<String> = Vec::new();

    if !res.links.is_empty() {
        let link_parts: Vec<String> = res
            .links
            .iter()
            .map(|l| {
                format!(
                    r#""{}":{{"href":"{}","templated":{}}}"#,
                    l.rel, l.href, l.templated
                )
            })
            .collect();
        parts.push(format!(r#""_links":{{{}}}"#, link_parts.join(",")));
    }

    for (k, v) in &res.properties {
        parts.push(format!(r#""{}":"{}""#, k, v));
    }

    if !res.embedded.is_empty() {
        let emb_parts: Vec<String> = res
            .embedded
            .iter()
            .map(|(rel, items)| {
                let items_json: Vec<String> = items.iter().map(serialize_hal).collect();
                format!(r#""{}":[{}]"#, rel, items_json.join(","))
            })
            .collect();
        parts.push(format!(r#""_embedded":{{{}}}"#, emb_parts.join(",")));
    }

    format!("{{{}}}", parts.join(","))
}

/// Link count.
#[allow(dead_code)]
pub fn link_count(res: &HalResource) -> usize {
    res.links.len()
}

/// Property count.
#[allow(dead_code)]
pub fn property_count(res: &HalResource) -> usize {
    res.properties.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_resource_empty() {
        let res = HalResource::new();
        assert!(res.links.is_empty() && res.properties.is_empty());
    }

    #[test]
    fn add_link_count() {
        let mut res = HalResource::new();
        res.add_link("self", "/api/user/1");
        assert_eq!(link_count(&res), 1);
    }

    #[test]
    fn add_property_count() {
        let mut res = HalResource::new();
        res.add_property("name", "Alice");
        assert_eq!(property_count(&res), 1);
    }

    #[test]
    fn serialize_empty_is_braces() {
        let res = HalResource::new();
        assert_eq!(serialize_hal(&res), "{}");
    }

    #[test]
    fn serialize_contains_links() {
        let mut res = HalResource::new();
        res.add_link("self", "/api/user/1");
        let s = serialize_hal(&res);
        assert!(s.contains("_links"));
    }

    #[test]
    fn serialize_contains_href() {
        let mut res = HalResource::new();
        res.add_link("self", "/api/user/1");
        let s = serialize_hal(&res);
        assert!(s.contains("/api/user/1"));
    }

    #[test]
    fn serialize_contains_property() {
        let mut res = HalResource::new();
        res.add_property("name", "Alice");
        let s = serialize_hal(&res);
        assert!(s.contains("Alice"));
    }

    #[test]
    fn embed_in_output() {
        let mut res = HalResource::new();
        let mut child = HalResource::new();
        child.add_property("id", "1");
        res.embed("items", vec![child]);
        let s = serialize_hal(&res);
        assert!(s.contains("_embedded"));
    }

    #[test]
    fn link_href_stored() {
        let mut res = HalResource::new();
        res.add_link("next", "/api/user/2");
        assert_eq!(res.links[0].href, "/api/user/2");
    }

    #[test]
    fn multiple_properties() {
        let mut res = HalResource::new();
        res.add_property("a", "1");
        res.add_property("b", "2");
        assert_eq!(property_count(&res), 2);
    }
}
