// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// An archive of named expression weight snapshots.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionArchive {
    pub entries: HashMap<String, Vec<f32>>,
}

/// Create a new empty expression archive.
#[allow(dead_code)]
pub fn new_expression_archive() -> ExpressionArchive {
    ExpressionArchive {
        entries: HashMap::new(),
    }
}

/// Store an expression snapshot under the given name.
#[allow(dead_code)]
pub fn archive_expression(archive: &mut ExpressionArchive, name: &str, weights: Vec<f32>) {
    archive.entries.insert(name.to_string(), weights);
}

/// Retrieve a stored expression by name.
#[allow(dead_code)]
pub fn retrieve_expression<'a>(archive: &'a ExpressionArchive, name: &str) -> Option<&'a [f32]> {
    archive.entries.get(name).map(|v| v.as_slice())
}

/// Return the number of archived expressions.
#[allow(dead_code)]
pub fn archive_count(archive: &ExpressionArchive) -> usize {
    archive.entries.len()
}

/// Return all archived expression names (sorted).
#[allow(dead_code)]
pub fn archive_names(archive: &ExpressionArchive) -> Vec<String> {
    let mut names: Vec<String> = archive.entries.keys().cloned().collect();
    names.sort();
    names
}

/// Serialize the archive to a JSON string.
#[allow(dead_code)]
pub fn archive_to_json(archive: &ExpressionArchive) -> String {
    let mut entries: Vec<String> = archive
        .entries
        .iter()
        .map(|(k, v)| {
            let vals: Vec<String> = v.iter().map(|f| format!("{:.4}", f)).collect();
            format!("\"{}\":[{}]", k, vals.join(","))
        })
        .collect();
    entries.sort();
    format!("{{{}}}", entries.join(","))
}

/// Clear all archived expressions.
#[allow(dead_code)]
pub fn archive_clear(archive: &mut ExpressionArchive) {
    archive.entries.clear();
}

/// Check whether the archive contains an expression with the given name.
#[allow(dead_code)]
pub fn archive_contains(archive: &ExpressionArchive, name: &str) -> bool {
    archive.entries.contains_key(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_archive_is_empty() {
        let a = new_expression_archive();
        assert_eq!(archive_count(&a), 0);
    }

    #[test]
    fn archive_and_retrieve() {
        let mut a = new_expression_archive();
        archive_expression(&mut a, "smile", vec![0.5, 0.3]);
        let w = retrieve_expression(&a, "smile").expect("should succeed");
        assert!((w[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn retrieve_missing() {
        let a = new_expression_archive();
        assert!(retrieve_expression(&a, "nope").is_none());
    }

    #[test]
    fn archive_count_grows() {
        let mut a = new_expression_archive();
        archive_expression(&mut a, "a", vec![1.0]);
        archive_expression(&mut a, "b", vec![0.5]);
        assert_eq!(archive_count(&a), 2);
    }

    #[test]
    fn names_sorted() {
        let mut a = new_expression_archive();
        archive_expression(&mut a, "z", vec![]);
        archive_expression(&mut a, "a", vec![]);
        let names = archive_names(&a);
        assert_eq!(names[0], "a");
        assert_eq!(names[1], "z");
    }

    #[test]
    fn clear_works() {
        let mut a = new_expression_archive();
        archive_expression(&mut a, "x", vec![1.0]);
        archive_clear(&mut a);
        assert_eq!(archive_count(&a), 0);
    }

    #[test]
    fn contains_check() {
        let mut a = new_expression_archive();
        archive_expression(&mut a, "smile", vec![0.5]);
        assert!(archive_contains(&a, "smile"));
        assert!(!archive_contains(&a, "frown"));
    }

    #[test]
    fn to_json_not_empty() {
        let mut a = new_expression_archive();
        archive_expression(&mut a, "test", vec![0.1]);
        let j = archive_to_json(&a);
        assert!(j.contains("\"test\""));
    }

    #[test]
    fn overwrite_entry() {
        let mut a = new_expression_archive();
        archive_expression(&mut a, "x", vec![0.1]);
        archive_expression(&mut a, "x", vec![0.9]);
        let w = retrieve_expression(&a, "x").expect("should succeed");
        assert!((w[0] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn empty_archive_json() {
        let a = new_expression_archive();
        assert_eq!(archive_to_json(&a), "{}");
    }
}
