#![allow(dead_code)]

use std::collections::HashSet;

/// Filters which body pairs can collide.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyPairFilter {
    denied: HashSet<(u32, u32)>,
}

fn normalize_pair(a: u32, b: u32) -> (u32, u32) {
    if a <= b { (a, b) } else { (b, a) }
}

/// Creates a new empty pair filter.
#[allow(dead_code)]
pub fn new_body_pair_filter() -> BodyPairFilter {
    BodyPairFilter {
        denied: HashSet::new(),
    }
}

/// Allows a pair (removes from denied set).
#[allow(dead_code)]
pub fn allow_pair(filter: &mut BodyPairFilter, a: u32, b: u32) {
    filter.denied.remove(&normalize_pair(a, b));
}

/// Denies a pair.
#[allow(dead_code)]
pub fn deny_pair(filter: &mut BodyPairFilter, a: u32, b: u32) {
    filter.denied.insert(normalize_pair(a, b));
}

/// Returns true if the pair is allowed (not denied).
#[allow(dead_code)]
pub fn is_pair_allowed(filter: &BodyPairFilter, a: u32, b: u32) -> bool {
    !filter.denied.contains(&normalize_pair(a, b))
}

/// Returns the total number of filter entries.
#[allow(dead_code)]
pub fn filter_count(filter: &BodyPairFilter) -> usize {
    filter.denied.len()
}

/// Clears all filter entries.
#[allow(dead_code)]
pub fn clear_filter(filter: &mut BodyPairFilter) {
    filter.denied.clear();
}

/// Returns the number of denied pairs.
#[allow(dead_code)]
pub fn denied_count(filter: &BodyPairFilter) -> usize {
    filter.denied.len()
}

/// Serializes the filter to JSON.
#[allow(dead_code)]
pub fn filter_to_json(filter: &BodyPairFilter) -> String {
    let mut pairs: Vec<(u32, u32)> = filter.denied.iter().copied().collect();
    pairs.sort();
    let entries: Vec<String> = pairs
        .iter()
        .map(|(a, b)| format!("[{a},{b}]"))
        .collect();
    format!("{{\"denied\":[{}]}}", entries.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_filter() {
        let f = new_body_pair_filter();
        assert_eq!(filter_count(&f), 0);
    }

    #[test]
    fn test_deny_pair() {
        let mut f = new_body_pair_filter();
        deny_pair(&mut f, 1, 2);
        assert!(!is_pair_allowed(&f, 1, 2));
    }

    #[test]
    fn test_allow_pair() {
        let mut f = new_body_pair_filter();
        deny_pair(&mut f, 1, 2);
        allow_pair(&mut f, 1, 2);
        assert!(is_pair_allowed(&f, 1, 2));
    }

    #[test]
    fn test_symmetric() {
        let mut f = new_body_pair_filter();
        deny_pair(&mut f, 1, 2);
        assert!(!is_pair_allowed(&f, 2, 1));
    }

    #[test]
    fn test_default_allowed() {
        let f = new_body_pair_filter();
        assert!(is_pair_allowed(&f, 5, 10));
    }

    #[test]
    fn test_filter_count() {
        let mut f = new_body_pair_filter();
        deny_pair(&mut f, 1, 2);
        deny_pair(&mut f, 3, 4);
        assert_eq!(filter_count(&f), 2);
    }

    #[test]
    fn test_clear() {
        let mut f = new_body_pair_filter();
        deny_pair(&mut f, 1, 2);
        clear_filter(&mut f);
        assert_eq!(filter_count(&f), 0);
    }

    #[test]
    fn test_denied_count() {
        let mut f = new_body_pair_filter();
        deny_pair(&mut f, 1, 2);
        assert_eq!(denied_count(&f), 1);
    }

    #[test]
    fn test_to_json() {
        let mut f = new_body_pair_filter();
        deny_pair(&mut f, 1, 2);
        let json = filter_to_json(&f);
        assert!(json.contains("[1,2]"));
    }

    #[test]
    fn test_duplicate_deny() {
        let mut f = new_body_pair_filter();
        deny_pair(&mut f, 1, 2);
        deny_pair(&mut f, 1, 2);
        assert_eq!(filter_count(&f), 1);
    }
}
