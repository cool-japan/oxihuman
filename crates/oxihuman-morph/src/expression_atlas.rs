#![allow(dead_code)]

use std::collections::HashMap;

/// Atlas of named expressions with associated weight vectors.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionAtlas {
    entries: HashMap<String, Vec<f32>>,
}

#[allow(dead_code)]
pub fn new_expression_atlas() -> ExpressionAtlas {
    ExpressionAtlas { entries: HashMap::new() }
}

#[allow(dead_code)]
pub fn add_atlas_expression(atlas: &mut ExpressionAtlas, name: &str, weights: &[f32]) {
    atlas.entries.insert(name.to_string(), weights.to_vec());
}

#[allow(dead_code)]
pub fn get_atlas_expression<'a>(atlas: &'a ExpressionAtlas, name: &str) -> Option<&'a [f32]> {
    atlas.entries.get(name).map(|v| v.as_slice())
}

#[allow(dead_code)]
pub fn atlas_count(atlas: &ExpressionAtlas) -> usize {
    atlas.entries.len()
}

#[allow(dead_code)]
pub fn atlas_names(atlas: &ExpressionAtlas) -> Vec<String> {
    let mut names: Vec<String> = atlas.entries.keys().cloned().collect();
    names.sort();
    names
}

#[allow(dead_code)]
pub fn atlas_to_json(atlas: &ExpressionAtlas) -> String {
    let mut names = atlas_names(atlas);
    names.sort();
    let entries: Vec<String> = names.iter().map(|n| {
        let w = &atlas.entries[n];
        let ws: Vec<String> = w.iter().map(|v| format!("{:.4}", v)).collect();
        format!("\"{}\":[{}]", n, ws.join(","))
    }).collect();
    format!("{{{}}}", entries.join(","))
}

#[allow(dead_code)]
pub fn atlas_contains(atlas: &ExpressionAtlas, name: &str) -> bool {
    atlas.entries.contains_key(name)
}

#[allow(dead_code)]
pub fn atlas_clear(atlas: &mut ExpressionAtlas) {
    atlas.entries.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_atlas() {
        let a = new_expression_atlas();
        assert_eq!(atlas_count(&a), 0);
    }

    #[test]
    fn test_add_get() {
        let mut a = new_expression_atlas();
        add_atlas_expression(&mut a, "smile", &[1.0, 0.5]);
        let w = get_atlas_expression(&a, "smile").expect("should succeed");
        assert!((w[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_get_missing() {
        let a = new_expression_atlas();
        assert!(get_atlas_expression(&a, "nope").is_none());
    }

    #[test]
    fn test_count() {
        let mut a = new_expression_atlas();
        add_atlas_expression(&mut a, "a", &[1.0]);
        add_atlas_expression(&mut a, "b", &[2.0]);
        assert_eq!(atlas_count(&a), 2);
    }

    #[test]
    fn test_names() {
        let mut a = new_expression_atlas();
        add_atlas_expression(&mut a, "b", &[1.0]);
        add_atlas_expression(&mut a, "a", &[2.0]);
        let n = atlas_names(&a);
        assert_eq!(n[0], "a");
        assert_eq!(n[1], "b");
    }

    #[test]
    fn test_contains() {
        let mut a = new_expression_atlas();
        add_atlas_expression(&mut a, "x", &[1.0]);
        assert!(atlas_contains(&a, "x"));
        assert!(!atlas_contains(&a, "y"));
    }

    #[test]
    fn test_clear() {
        let mut a = new_expression_atlas();
        add_atlas_expression(&mut a, "x", &[1.0]);
        atlas_clear(&mut a);
        assert_eq!(atlas_count(&a), 0);
    }

    #[test]
    fn test_to_json() {
        let mut a = new_expression_atlas();
        add_atlas_expression(&mut a, "smile", &[1.0]);
        let j = atlas_to_json(&a);
        assert!(j.contains("smile"));
    }

    #[test]
    fn test_overwrite() {
        let mut a = new_expression_atlas();
        add_atlas_expression(&mut a, "x", &[1.0]);
        add_atlas_expression(&mut a, "x", &[2.0]);
        assert_eq!(atlas_count(&a), 1);
        let w = get_atlas_expression(&a, "x").expect("should succeed");
        assert!((w[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_empty_names() {
        let a = new_expression_atlas();
        assert!(atlas_names(&a).is_empty());
    }
}
