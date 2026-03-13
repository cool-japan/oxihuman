// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Simple string-pattern route table with parameter extraction.

/// A route entry mapping a pattern to a handler name.
#[derive(Debug, Clone)]
pub struct RouteEntry {
    pub pattern: String,
    pub handler: String,
    pub priority: i32,
}

/// Result of a route match.
#[derive(Debug, Clone)]
pub struct RouteMatch {
    pub handler: String,
    pub params: Vec<(String, String)>,
}

/// Route table mapping URL-like paths to handlers.
pub struct RouteTable {
    routes: Vec<RouteEntry>,
    dispatch_count: u64,
}

fn match_pattern(pattern: &str, path: &str) -> Option<Vec<(String, String)>> {
    let pat_parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();
    if pat_parts.len() != path_parts.len() {
        return None;
    }
    let mut params = Vec::new();
    for (pp, sp) in pat_parts.iter().zip(path_parts.iter()) {
        if let Some(name) = pp.strip_prefix(':') {
            params.push((name.to_string(), (*sp).to_string()));
        } else if pp != sp {
            return None;
        }
    }
    Some(params)
}

#[allow(dead_code)]
impl RouteTable {
    pub fn new() -> Self {
        RouteTable {
            routes: Vec::new(),
            dispatch_count: 0,
        }
    }

    pub fn add_route(&mut self, pattern: &str, handler: &str, priority: i32) {
        let pos = self.routes.partition_point(|r| r.priority > priority);
        self.routes.insert(
            pos,
            RouteEntry {
                pattern: pattern.to_string(),
                handler: handler.to_string(),
                priority,
            },
        );
    }

    pub fn dispatch(&mut self, path: &str) -> Option<RouteMatch> {
        self.dispatch_count += 1;
        for route in &self.routes {
            if let Some(params) = match_pattern(&route.pattern, path) {
                return Some(RouteMatch {
                    handler: route.handler.clone(),
                    params,
                });
            }
        }
        None
    }

    pub fn remove_handler(&mut self, handler: &str) -> usize {
        let before = self.routes.len();
        self.routes.retain(|r| r.handler != handler);
        before - self.routes.len()
    }

    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    pub fn dispatch_count(&self) -> u64 {
        self.dispatch_count
    }

    pub fn has_pattern(&self, pattern: &str) -> bool {
        self.routes.iter().any(|r| r.pattern == pattern)
    }

    pub fn handlers(&self) -> Vec<&str> {
        self.routes.iter().map(|r| r.handler.as_str()).collect()
    }

    pub fn clear(&mut self) {
        self.routes.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }
}

impl Default for RouteTable {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_route_table() -> RouteTable {
    RouteTable::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        let mut t = new_route_table();
        t.add_route("/health", "health_handler", 0);
        let m = t.dispatch("/health").expect("should succeed");
        assert_eq!(m.handler, "health_handler");
        assert!(m.params.is_empty());
    }

    #[test]
    fn param_extraction() {
        let mut t = new_route_table();
        t.add_route("/user/:id", "user_handler", 0);
        let m = t.dispatch("/user/42").expect("should succeed");
        assert_eq!(m.handler, "user_handler");
        assert_eq!(m.params[0], ("id".to_string(), "42".to_string()));
    }

    #[test]
    fn no_match() {
        let mut t = new_route_table();
        t.add_route("/a", "h", 0);
        assert!(t.dispatch("/b").is_none());
    }

    #[test]
    fn priority_ordering() {
        let mut t = new_route_table();
        t.add_route("/item/:id", "generic", 0);
        t.add_route("/item/special", "specific", 10);
        let m = t.dispatch("/item/special").expect("should succeed");
        assert_eq!(m.handler, "specific");
    }

    #[test]
    fn remove_handler() {
        let mut t = new_route_table();
        t.add_route("/a", "h", 0);
        t.add_route("/b", "h", 0);
        assert_eq!(t.remove_handler("h"), 2);
        assert!(t.is_empty());
    }

    #[test]
    fn dispatch_count_tracked() {
        let mut t = new_route_table();
        t.add_route("/x", "h", 0);
        t.dispatch("/x");
        t.dispatch("/y");
        assert_eq!(t.dispatch_count(), 2);
    }

    #[test]
    fn has_pattern() {
        let mut t = new_route_table();
        t.add_route("/foo", "h", 0);
        assert!(t.has_pattern("/foo"));
        assert!(!t.has_pattern("/bar"));
    }

    #[test]
    fn multiple_params() {
        let mut t = new_route_table();
        t.add_route("/a/:x/b/:y", "h", 0);
        let m = t.dispatch("/a/1/b/2").expect("should succeed");
        assert_eq!(m.params.len(), 2);
    }

    #[test]
    fn clear_table() {
        let mut t = new_route_table();
        t.add_route("/x", "h", 0);
        t.clear();
        assert_eq!(t.route_count(), 0);
    }
}
