// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Lightweight search index over morph target names.
//!
//! Supports substring (case-insensitive) and fuzzy (subsequence) search,
//! as well as filtering by category and listing unique categories.

/// A single indexed target entry.
#[derive(Debug, Clone)]
pub struct TargetEntry {
    /// Full name of the target (e.g. `"head/head-age-young.target"`).
    pub name: String,
    /// Top-level category (first path component before `/`).
    pub category: String,
    /// Sub-category (second path component, if present).
    pub subcategory: Option<String>,
}

impl TargetEntry {
    /// Parse a target name into a `TargetEntry`.
    ///
    /// Expects the format `"category/subcategory/filename.target"`.
    pub fn from_name(name: &str) -> Self {
        let parts: Vec<&str> = name.split('/').collect();
        let category = parts.first().map(|s| s.to_string()).unwrap_or_default();
        let subcategory = if parts.len() > 2 {
            Some(parts[1].to_string())
        } else {
            None
        };
        Self {
            name: name.to_string(),
            category,
            subcategory,
        }
    }
}

// ---------------------------------------------------------------------------
// TargetIndex
// ---------------------------------------------------------------------------

/// A lightweight search index over target names.
pub struct TargetIndex {
    /// All target entries in the library.
    entries: Vec<TargetEntry>,
}

impl TargetIndex {
    /// Build an index from a slice of target name strings.
    pub fn from_names(names: &[&str]) -> Self {
        let entries = names.iter().map(|n| TargetEntry::from_name(n)).collect();
        Self { entries }
    }

    /// Build an index from a [`crate::target_lib::TargetLibrary`].
    pub fn from_library(lib: &crate::target_lib::TargetLibrary) -> Self {
        let entries = lib
            .iter()
            .map(|(name, _)| TargetEntry::from_name(name))
            .collect();
        Self { entries }
    }

    /// Exact substring search (case-insensitive).
    ///
    /// Returns all entries whose name contains `query` as a substring.
    pub fn search(&self, query: &str) -> Vec<&TargetEntry> {
        let q = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.name.to_lowercase().contains(&q))
            .collect()
    }

    /// Fuzzy search: returns entries where `query` is a subsequence of the name.
    ///
    /// Results are sorted by [`fuzzy_score`] descending (best matches first).
    pub fn fuzzy_search(&self, query: &str) -> Vec<&TargetEntry> {
        let mut matches: Vec<(&TargetEntry, usize)> = self
            .entries
            .iter()
            .filter(|e| is_subsequence(query, &e.name))
            .map(|e| (e, fuzzy_score(query, &e.name)))
            .collect();
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        matches.into_iter().map(|(e, _)| e).collect()
    }

    /// Filter by top-level category (exact, case-insensitive).
    pub fn by_category(&self, category: &str) -> Vec<&TargetEntry> {
        let cat = category.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.category.to_lowercase() == cat)
            .collect()
    }

    /// Return all unique top-level categories in index order (deduped, stable).
    pub fn categories(&self) -> Vec<&str> {
        let mut seen: Vec<&str> = Vec::new();
        for entry in &self.entries {
            let cat = entry.category.as_str();
            if !seen.contains(&cat) {
                seen.push(cat);
            }
        }
        seen
    }

    /// Number of indexed targets.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the index contains no targets.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return all entries sorted alphabetically by name.
    pub fn sorted_by_name(&self) -> Vec<&TargetEntry> {
        let mut sorted: Vec<&TargetEntry> = self.entries.iter().collect();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));
        sorted
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// Check whether `query` is a subsequence of `text` (case-insensitive).
///
/// E.g. `is_subsequence("hag", "head-age")` returns `true` because
/// `h`, `a`, `g` all appear in order inside `"head-age"`.
pub fn is_subsequence(query: &str, text: &str) -> bool {
    let mut qi = query.chars().flat_map(char::to_lowercase);
    let mut ti = text.chars().flat_map(char::to_lowercase);
    let mut qc = qi.next();
    for tc in &mut ti {
        if let Some(q) = qc {
            if q == tc {
                qc = qi.next();
            }
        } else {
            break;
        }
    }
    qc.is_none()
}

/// Score a fuzzy match by counting the length of the longest consecutive run
/// of query characters found contiguously in `text` (case-insensitive).
///
/// Higher score means a better (more consecutive) match.
pub fn fuzzy_score(query: &str, text: &str) -> usize {
    if query.is_empty() {
        return 0;
    }
    let q_chars: Vec<char> = query.chars().flat_map(char::to_lowercase).collect();
    let t_chars: Vec<char> = text.chars().flat_map(char::to_lowercase).collect();

    let mut best_run = 0usize;
    let mut current_run = 0usize;
    let mut qi = 0usize;

    for tc in &t_chars {
        if qi < q_chars.len() && *tc == q_chars[qi] {
            qi += 1;
            current_run += 1;
            if current_run > best_run {
                best_run = current_run;
            }
        } else {
            current_run = 0;
        }
    }

    best_run
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_names() -> Vec<&'static str> {
        vec![
            "head/head-age-young.target",
            "head/head-age-old.target",
            "head/head-weight-fat.target",
            "torso/torso-height-tall.target",
            "l-arm/l-arm-muscle.target",
            "r-arm/r-arm-muscle.target",
        ]
    }

    #[test]
    fn index_len() {
        let index = TargetIndex::from_names(&sample_names());
        assert_eq!(index.len(), 6);
    }

    #[test]
    fn search_substring() {
        let index = TargetIndex::from_names(&sample_names());
        let results = index.search("age");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_case_insensitive() {
        let index = TargetIndex::from_names(&sample_names());
        let results = index.search("HEAD");
        // All three head/* targets contain "head"
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn search_no_match() {
        let index = TargetIndex::from_names(&sample_names());
        let results = index.search("zzz");
        assert!(results.is_empty());
    }

    #[test]
    fn by_category_head() {
        let index = TargetIndex::from_names(&sample_names());
        let results = index.by_category("head");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn categories_count() {
        let index = TargetIndex::from_names(&sample_names());
        let cats = index.categories();
        assert_eq!(cats.len(), 4); // head, torso, l-arm, r-arm
    }

    #[test]
    fn is_subsequence_true() {
        assert!(is_subsequence("hag", "head-age"));
    }

    #[test]
    fn is_subsequence_false() {
        assert!(!is_subsequence("zz", "head-age"));
    }

    #[test]
    fn fuzzy_search_returns_results() {
        let index = TargetIndex::from_names(&sample_names());
        let results = index.fuzzy_search("hag");
        assert!(!results.is_empty());
    }

    #[test]
    fn sorted_by_name_is_alphabetical() {
        let index = TargetIndex::from_names(&sample_names());
        let sorted = index.sorted_by_name();
        assert!(!sorted.is_empty());
        let first = sorted.first().expect("should succeed").name.as_str();
        let last = sorted.last().expect("should succeed").name.as_str();
        assert!(
            first <= last,
            "first ({first}) should come before last ({last}) alphabetically"
        );
    }

    #[test]
    fn entry_from_name_parses_category() {
        let entry = TargetEntry::from_name("head/sub/file.target");
        assert_eq!(entry.category, "head");
        assert_eq!(entry.subcategory.as_deref(), Some("sub"));
    }
}
