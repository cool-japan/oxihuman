#![allow(dead_code)]
//! Edge crease values for subdivision control.

use std::collections::HashMap;

/// A set of edge crease values.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeCrease {
    pub creases: HashMap<(u32, u32), f32>,
}

fn ordered_edge(a: u32, b: u32) -> (u32, u32) {
    if a < b { (a, b) } else { (b, a) }
}

/// Create a new empty edge crease set.
#[allow(dead_code)]
pub fn new_edge_crease_set() -> EdgeCrease {
    EdgeCrease {
        creases: HashMap::new(),
    }
}

/// Set crease value for an edge.
#[allow(dead_code)]
pub fn set_crease(ec: &mut EdgeCrease, a: u32, b: u32, value: f32) {
    ec.creases.insert(ordered_edge(a, b), value.clamp(0.0, 1.0));
}

/// Get crease value for an edge.
#[allow(dead_code)]
pub fn get_crease(ec: &EdgeCrease, a: u32, b: u32) -> f32 {
    ec.creases.get(&ordered_edge(a, b)).copied().unwrap_or(0.0)
}

/// Return number of creased edges.
#[allow(dead_code)]
pub fn crease_count(ec: &EdgeCrease) -> usize {
    ec.creases.len()
}

/// Return all creased edges as (a, b, value).
#[allow(dead_code)]
pub fn all_creased_edges(ec: &EdgeCrease) -> Vec<(u32, u32, f32)> {
    let mut result: Vec<(u32, u32, f32)> = ec
        .creases
        .iter()
        .map(|(&(a, b), &v)| (a, b, v))
        .collect();
    result.sort_by_key(|&(a, b, _)| (a, b));
    result
}

/// Remove crease from an edge.
#[allow(dead_code)]
pub fn remove_crease(ec: &mut EdgeCrease, a: u32, b: u32) {
    ec.creases.remove(&ordered_edge(a, b));
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn crease_to_json(ec: &EdgeCrease) -> String {
    let edges: Vec<String> = all_creased_edges(ec)
        .iter()
        .map(|(a, b, v)| format!("{{\"edge\":[{},{}],\"value\":{:.4}}}", a, b, v))
        .collect();
    format!("{{\"creases\":[{}]}}", edges.join(","))
}

/// Clear all creases.
#[allow(dead_code)]
pub fn clear_creases(ec: &mut EdgeCrease) {
    ec.creases.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_edge_crease_set() {
        let ec = new_edge_crease_set();
        assert_eq!(crease_count(&ec), 0);
    }

    #[test]
    fn test_set_get_crease() {
        let mut ec = new_edge_crease_set();
        set_crease(&mut ec, 0, 1, 0.5);
        assert!((get_crease(&ec, 0, 1) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_crease_symmetric() {
        let mut ec = new_edge_crease_set();
        set_crease(&mut ec, 3, 1, 0.7);
        assert!((get_crease(&ec, 1, 3) - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_crease_clamp() {
        let mut ec = new_edge_crease_set();
        set_crease(&mut ec, 0, 1, 2.0);
        assert!((get_crease(&ec, 0, 1) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_crease_count() {
        let mut ec = new_edge_crease_set();
        set_crease(&mut ec, 0, 1, 0.5);
        set_crease(&mut ec, 2, 3, 0.8);
        assert_eq!(crease_count(&ec), 2);
    }

    #[test]
    fn test_all_creased_edges() {
        let mut ec = new_edge_crease_set();
        set_crease(&mut ec, 0, 1, 0.5);
        let edges = all_creased_edges(&ec);
        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn test_remove_crease() {
        let mut ec = new_edge_crease_set();
        set_crease(&mut ec, 0, 1, 0.5);
        remove_crease(&mut ec, 0, 1);
        assert_eq!(crease_count(&ec), 0);
    }

    #[test]
    fn test_crease_to_json() {
        let mut ec = new_edge_crease_set();
        set_crease(&mut ec, 0, 1, 0.5);
        let j = crease_to_json(&ec);
        assert!(j.contains("\"creases\""));
    }

    #[test]
    fn test_clear_creases() {
        let mut ec = new_edge_crease_set();
        set_crease(&mut ec, 0, 1, 0.5);
        set_crease(&mut ec, 2, 3, 0.8);
        clear_creases(&mut ec);
        assert_eq!(crease_count(&ec), 0);
    }

    #[test]
    fn test_get_missing_crease() {
        let ec = new_edge_crease_set();
        assert_eq!(get_crease(&ec, 0, 1), 0.0);
    }
}
