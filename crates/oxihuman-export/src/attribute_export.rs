// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export generic vertex/face attributes.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AttrDomain { Vertex, Face, Edge, Corner }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AttributeEntry {
    pub name: String,
    pub domain: AttrDomain,
    pub values: Vec<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AttributeExportSet {
    pub attributes: Vec<AttributeEntry>,
}

#[allow(dead_code)]
pub fn new_attribute_export_set() -> AttributeExportSet {
    AttributeExportSet { attributes: Vec::new() }
}

#[allow(dead_code)]
pub fn aes_add(set: &mut AttributeExportSet, name: &str, domain: AttrDomain, values: Vec<f32>) {
    set.attributes.push(AttributeEntry { name: name.to_string(), domain, values });
}

#[allow(dead_code)]
pub fn aes_count(set: &AttributeExportSet) -> usize { set.attributes.len() }

#[allow(dead_code)]
pub fn aes_find<'a>(set: &'a AttributeExportSet, name: &str) -> Option<&'a AttributeEntry> {
    set.attributes.iter().find(|a| a.name == name)
}

#[allow(dead_code)]
pub fn aes_remove(set: &mut AttributeExportSet, name: &str) -> bool {
    let before = set.attributes.len();
    set.attributes.retain(|a| a.name != name);
    set.attributes.len() < before
}

#[allow(dead_code)]
pub fn aes_total_values(set: &AttributeExportSet) -> usize {
    set.attributes.iter().map(|a| a.values.len()).sum()
}

#[allow(dead_code)]
pub fn aes_validate(set: &AttributeExportSet) -> bool {
    set.attributes.iter().all(|a| !a.name.is_empty() && !a.values.is_empty())
}

#[allow(dead_code)]
pub fn aes_to_json(set: &AttributeExportSet) -> String {
    let items: Vec<String> = set.attributes.iter().map(|a| format!("{{\"name\":\"{}\",\"count\":{}}}", a.name, a.values.len())).collect();
    format!("{{\"attributes\":[{}]}}", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> AttributeExportSet {
        let mut s = new_attribute_export_set();
        aes_add(&mut s, "weight", AttrDomain::Vertex, vec![0.5, 0.8, 1.0]);
        s
    }

    #[test] fn test_new() { let s = new_attribute_export_set(); assert_eq!(aes_count(&s), 0); }
    #[test] fn test_add() { assert_eq!(aes_count(&sample()), 1); }
    #[test] fn test_find() { let s = sample(); assert!(aes_find(&s, "weight").is_some()); }
    #[test] fn test_find_missing() { let s = sample(); assert!(aes_find(&s, "nope").is_none()); }
    #[test] fn test_remove() { let mut s = sample(); assert!(aes_remove(&mut s, "weight")); assert_eq!(aes_count(&s), 0); }
    #[test] fn test_total_values() { let s = sample(); assert_eq!(aes_total_values(&s), 3); }
    #[test] fn test_validate() { let s = sample(); assert!(aes_validate(&s)); }
    #[test] fn test_to_json() { let s = sample(); assert!(aes_to_json(&s).contains("weight")); }
    #[test] fn test_domain() { let s = sample(); assert_eq!(s.attributes[0].domain, AttrDomain::Vertex); }
    #[test] fn test_multi() {
        let mut s = sample();
        aes_add(&mut s, "color", AttrDomain::Face, vec![1.0,0.0,0.0]);
        assert_eq!(aes_count(&s), 2);
    }
}
