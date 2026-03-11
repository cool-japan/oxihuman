#![allow(dead_code)]

use std::collections::HashMap;

/// A palette of named expression presets.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionPalette {
    entries: HashMap<String, Vec<f32>>,
}

#[allow(dead_code)]
pub fn new_expression_palette() -> ExpressionPalette {
    ExpressionPalette { entries: HashMap::new() }
}

#[allow(dead_code)]
pub fn add_palette_entry(p: &mut ExpressionPalette, name: &str, weights: &[f32]) {
    p.entries.insert(name.to_string(), weights.to_vec());
}

#[allow(dead_code)]
pub fn get_palette_entry<'a>(p: &'a ExpressionPalette, name: &str) -> Option<&'a [f32]> {
    p.entries.get(name).map(|v| v.as_slice())
}

#[allow(dead_code)]
pub fn palette_count(p: &ExpressionPalette) -> usize { p.entries.len() }

#[allow(dead_code)]
pub fn palette_names(p: &ExpressionPalette) -> Vec<String> {
    let mut n: Vec<String> = p.entries.keys().cloned().collect();
    n.sort();
    n
}

#[allow(dead_code)]
pub fn palette_to_json(p: &ExpressionPalette) -> String {
    let names = palette_names(p);
    let e: Vec<String> = names.iter().map(|n| format!("\"{}\":true", n)).collect();
    format!("{{\"count\":{},{}}}", p.entries.len(), e.join(","))
}

#[allow(dead_code)]
pub fn palette_remove(p: &mut ExpressionPalette, name: &str) -> bool {
    p.entries.remove(name).is_some()
}

#[allow(dead_code)]
pub fn palette_clear(p: &mut ExpressionPalette) { p.entries.clear(); }

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_new() { assert_eq!(palette_count(&new_expression_palette()), 0); }
    #[test] fn test_add_get() {
        let mut p = new_expression_palette();
        add_palette_entry(&mut p, "smile", &[1.0]);
        assert!(get_palette_entry(&p, "smile").is_some());
    }
    #[test] fn test_get_missing() { assert!(get_palette_entry(&new_expression_palette(), "x").is_none()); }
    #[test] fn test_count() {
        let mut p = new_expression_palette();
        add_palette_entry(&mut p, "a", &[1.0]);
        add_palette_entry(&mut p, "b", &[2.0]);
        assert_eq!(palette_count(&p), 2);
    }
    #[test] fn test_names() {
        let mut p = new_expression_palette();
        add_palette_entry(&mut p, "b", &[1.0]);
        add_palette_entry(&mut p, "a", &[2.0]);
        assert_eq!(palette_names(&p)[0], "a");
    }
    #[test] fn test_remove() {
        let mut p = new_expression_palette();
        add_palette_entry(&mut p, "x", &[1.0]);
        assert!(palette_remove(&mut p, "x"));
        assert!(!palette_remove(&mut p, "x"));
    }
    #[test] fn test_clear() {
        let mut p = new_expression_palette();
        add_palette_entry(&mut p, "x", &[1.0]);
        palette_clear(&mut p);
        assert_eq!(palette_count(&p), 0);
    }
    #[test] fn test_to_json() {
        let mut p = new_expression_palette();
        add_palette_entry(&mut p, "s", &[1.0]);
        assert!(palette_to_json(&p).contains("count"));
    }
    #[test] fn test_overwrite() {
        let mut p = new_expression_palette();
        add_palette_entry(&mut p, "x", &[1.0]);
        add_palette_entry(&mut p, "x", &[2.0]);
        assert_eq!(palette_count(&p), 1);
    }
    #[test] fn test_empty_names() { assert!(palette_names(&new_expression_palette()).is_empty()); }
}
