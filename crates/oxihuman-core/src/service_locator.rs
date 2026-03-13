// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A service locator registry that maps type-tagged names to boxed trait objects.

use std::collections::HashMap;

/// Descriptor of a registered service.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ServiceDescriptor {
    pub name: String,
    pub type_tag: String,
    pub version: u32,
    pub enabled: bool,
}

/// Service locator holding named service descriptors and opaque payloads as JSON strings.
#[allow(dead_code)]
pub struct ServiceLocator {
    services: HashMap<String, ServiceDescriptor>,
    payloads: HashMap<String, String>,
    lookup_count: u64,
}

#[allow(dead_code)]
impl ServiceLocator {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            payloads: HashMap::new(),
            lookup_count: 0,
        }
    }

    pub fn register(&mut self, name: &str, type_tag: &str, version: u32, payload: &str) {
        let desc = ServiceDescriptor {
            name: name.to_string(),
            type_tag: type_tag.to_string(),
            version,
            enabled: true,
        };
        self.services.insert(name.to_string(), desc);
        self.payloads.insert(name.to_string(), payload.to_string());
    }

    pub fn unregister(&mut self, name: &str) -> bool {
        self.payloads.remove(name);
        self.services.remove(name).is_some()
    }

    pub fn lookup(&mut self, name: &str) -> Option<&ServiceDescriptor> {
        self.lookup_count += 1;
        let enabled = self.services.get(name).is_some_and(|s| s.enabled);
        if enabled {
            self.services.get(name)
        } else {
            None
        }
    }

    pub fn payload(&self, name: &str) -> Option<&str> {
        self.payloads.get(name).map(|s| s.as_str())
    }

    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(s) = self.services.get_mut(name) {
            s.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn is_registered(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }

    pub fn count(&self) -> usize {
        self.services.len()
    }

    pub fn lookup_count(&self) -> u64 {
        self.lookup_count
    }

    pub fn names(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self.services.keys().map(|s| s.as_str()).collect();
        v.sort_unstable();
        v
    }

    pub fn by_type(&self, type_tag: &str) -> Vec<&ServiceDescriptor> {
        let mut v: Vec<&ServiceDescriptor> = self
            .services
            .values()
            .filter(|s| s.type_tag == type_tag)
            .collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }

    pub fn clear(&mut self) {
        self.services.clear();
        self.payloads.clear();
    }
}

impl Default for ServiceLocator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_service_locator() -> ServiceLocator {
    ServiceLocator::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup() {
        let mut loc = new_service_locator();
        loc.register("logger", "Logger", 1, "{}");
        let d = loc.lookup("logger").expect("should succeed");
        assert_eq!(d.name, "logger");
    }

    #[test]
    fn lookup_missing_returns_none() {
        let mut loc = new_service_locator();
        assert!(loc.lookup("nope").is_none());
    }

    #[test]
    fn unregister() {
        let mut loc = new_service_locator();
        loc.register("svc", "T", 1, "");
        assert!(loc.unregister("svc"));
        assert!(!loc.is_registered("svc"));
    }

    #[test]
    fn disabled_not_found() {
        let mut loc = new_service_locator();
        loc.register("svc", "T", 1, "");
        loc.set_enabled("svc", false);
        assert!(loc.lookup("svc").is_none());
    }

    #[test]
    fn payload_retrieved() {
        let mut loc = new_service_locator();
        loc.register("svc", "T", 1, r#"{"key":"val"}"#);
        assert_eq!(loc.payload("svc"), Some(r#"{"key":"val"}"#));
    }

    #[test]
    fn by_type_filter() {
        let mut loc = new_service_locator();
        loc.register("a", "Renderer", 1, "");
        loc.register("b", "Audio", 1, "");
        loc.register("c", "Renderer", 2, "");
        let r = loc.by_type("Renderer");
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn lookup_count_increments() {
        let mut loc = new_service_locator();
        loc.register("x", "T", 1, "");
        loc.lookup("x");
        loc.lookup("x");
        assert_eq!(loc.lookup_count(), 2);
    }

    #[test]
    fn count_correct() {
        let mut loc = new_service_locator();
        loc.register("a", "T", 1, "");
        loc.register("b", "T", 1, "");
        assert_eq!(loc.count(), 2);
    }

    #[test]
    fn clear_empties() {
        let mut loc = new_service_locator();
        loc.register("a", "T", 1, "");
        loc.clear();
        assert_eq!(loc.count(), 0);
    }

    #[test]
    fn names_sorted() {
        let mut loc = new_service_locator();
        loc.register("b", "T", 1, "");
        loc.register("a", "T", 1, "");
        assert_eq!(loc.names(), vec!["a", "b"]);
    }
}
