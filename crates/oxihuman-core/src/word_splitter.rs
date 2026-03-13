// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Word splitter — splits strings by a configurable separator.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WordSplitter {
    pub splits: Vec<String>,
    pub separator: String,
}

#[allow(dead_code)]
pub fn new_word_splitter(sep: &str) -> WordSplitter {
    WordSplitter {
        splits: Vec::new(),
        separator: sep.to_string(),
    }
}

#[allow(dead_code)]
pub fn split_string(ws: &mut WordSplitter, s: &str) -> Vec<String> {
    let result: Vec<String> = s.split(&ws.separator as &str).map(|p| p.to_string()).collect();
    ws.splits = result.clone();
    result
}

#[allow(dead_code)]
pub fn split_count(ws: &WordSplitter) -> usize {
    ws.splits.len()
}

#[allow(dead_code)]
pub fn longest_split(ws: &WordSplitter) -> usize {
    ws.splits.iter().map(|s| s.len()).max().unwrap_or(0)
}

#[allow(dead_code)]
pub fn shortest_split(ws: &WordSplitter) -> usize {
    ws.splits.iter().map(|s| s.len()).min().unwrap_or(0)
}

#[allow(dead_code)]
pub fn filter_empty(ws: &WordSplitter) -> Vec<String> {
    ws.splits.iter().filter(|s| !s.is_empty()).cloned().collect()
}

#[allow(dead_code)]
pub fn rejoin(ws: &WordSplitter) -> String {
    ws.splits.join(&ws.separator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_split() {
        let mut ws = new_word_splitter(",");
        let parts = split_string(&mut ws, "a,b,c");
        assert_eq!(parts, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_split_count() {
        let mut ws = new_word_splitter(" ");
        split_string(&mut ws, "hello world foo");
        assert_eq!(split_count(&ws), 3);
    }

    #[test]
    fn test_longest() {
        let mut ws = new_word_splitter(",");
        split_string(&mut ws, "ab,cde,f");
        assert_eq!(longest_split(&ws), 3);
    }

    #[test]
    fn test_shortest() {
        let mut ws = new_word_splitter(",");
        split_string(&mut ws, "ab,cde,f");
        assert_eq!(shortest_split(&ws), 1);
    }

    #[test]
    fn test_filter_empty() {
        let mut ws = new_word_splitter(",");
        split_string(&mut ws, "a,,b,");
        let filtered = filter_empty(&ws);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_rejoin() {
        let mut ws = new_word_splitter("-");
        split_string(&mut ws, "a-b-c");
        assert_eq!(rejoin(&ws), "a-b-c");
    }

    #[test]
    fn test_empty_string() {
        let mut ws = new_word_splitter(",");
        let parts = split_string(&mut ws, "");
        assert_eq!(parts, vec![""]);
    }

    #[test]
    fn test_no_separator() {
        let mut ws = new_word_splitter(",");
        split_string(&mut ws, "hello");
        assert_eq!(split_count(&ws), 1);
        assert_eq!(longest_split(&ws), 5);
    }
}
