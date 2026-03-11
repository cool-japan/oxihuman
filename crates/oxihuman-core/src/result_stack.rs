// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Stack of operation results for accumulating and inspecting outcomes.

/// A single result entry.
#[derive(Debug, Clone, PartialEq)]
pub enum ResultKind {
    Ok,
    Err,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct ResultEntry {
    pub kind: ResultKind,
    pub message: String,
    pub code: i32,
}

/// Stack accumulating operation results.
pub struct ResultStack {
    entries: Vec<ResultEntry>,
    ok_count: usize,
    err_count: usize,
    skip_count: usize,
}

#[allow(dead_code)]
impl ResultStack {
    pub fn new() -> Self {
        ResultStack {
            entries: Vec::new(),
            ok_count: 0,
            err_count: 0,
            skip_count: 0,
        }
    }

    pub fn push_ok(&mut self, msg: &str) {
        self.entries.push(ResultEntry {
            kind: ResultKind::Ok,
            message: msg.to_string(),
            code: 0,
        });
        self.ok_count += 1;
    }

    pub fn push_err(&mut self, msg: &str, code: i32) {
        self.entries.push(ResultEntry {
            kind: ResultKind::Err,
            message: msg.to_string(),
            code,
        });
        self.err_count += 1;
    }

    pub fn push_skipped(&mut self, msg: &str) {
        self.entries.push(ResultEntry {
            kind: ResultKind::Skipped,
            message: msg.to_string(),
            code: 0,
        });
        self.skip_count += 1;
    }

    pub fn pop(&mut self) -> Option<ResultEntry> {
        let entry = self.entries.pop()?;
        match entry.kind {
            ResultKind::Ok => self.ok_count -= 1,
            ResultKind::Err => self.err_count -= 1,
            ResultKind::Skipped => self.skip_count -= 1,
        }
        Some(entry)
    }

    pub fn peek(&self) -> Option<&ResultEntry> {
        self.entries.last()
    }

    pub fn has_errors(&self) -> bool {
        self.err_count > 0
    }

    pub fn all_ok(&self) -> bool {
        self.err_count == 0 && self.skip_count == 0 && self.ok_count > 0
    }

    pub fn ok_count(&self) -> usize {
        self.ok_count
    }

    pub fn err_count(&self) -> usize {
        self.err_count
    }

    pub fn skip_count(&self) -> usize {
        self.skip_count
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.ok_count = 0;
        self.err_count = 0;
        self.skip_count = 0;
    }

    pub fn errors(&self) -> Vec<&ResultEntry> {
        self.entries
            .iter()
            .filter(|e| e.kind == ResultKind::Err)
            .collect()
    }

    pub fn to_summary(&self) -> String {
        format!(
            "ok={} err={} skip={}",
            self.ok_count, self.err_count, self.skip_count
        )
    }
}

impl Default for ResultStack {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_result_stack() -> ResultStack {
    ResultStack::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_ok_increments() {
        let mut s = new_result_stack();
        s.push_ok("done");
        assert_eq!(s.ok_count(), 1);
        assert!(!s.has_errors());
    }

    #[test]
    fn push_err_tracked() {
        let mut s = new_result_stack();
        s.push_err("fail", -1);
        assert!(s.has_errors());
        assert_eq!(s.err_count(), 1);
    }

    #[test]
    fn push_skipped() {
        let mut s = new_result_stack();
        s.push_skipped("skip me");
        assert_eq!(s.skip_count(), 1);
    }

    #[test]
    fn pop_updates_counts() {
        let mut s = new_result_stack();
        s.push_ok("a");
        s.pop();
        assert_eq!(s.ok_count(), 0);
    }

    #[test]
    fn all_ok_flag() {
        let mut s = new_result_stack();
        s.push_ok("a");
        s.push_ok("b");
        assert!(s.all_ok());
        s.push_err("bad", 1);
        assert!(!s.all_ok());
    }

    #[test]
    fn errors_filter() {
        let mut s = new_result_stack();
        s.push_ok("ok");
        s.push_err("e1", 1);
        s.push_err("e2", 2);
        assert_eq!(s.errors().len(), 2);
    }

    #[test]
    fn clear_resets() {
        let mut s = new_result_stack();
        s.push_ok("a");
        s.push_err("b", 1);
        s.clear();
        assert!(s.is_empty());
        assert_eq!(s.ok_count(), 0);
    }

    #[test]
    fn summary_string() {
        let mut s = new_result_stack();
        s.push_ok("a");
        s.push_err("b", 1);
        s.push_skipped("c");
        let summary = s.to_summary();
        assert!(summary.contains("ok=1"));
        assert!(summary.contains("err=1"));
    }

    #[test]
    fn peek_last() {
        let mut s = new_result_stack();
        s.push_ok("first");
        s.push_err("second", 2);
        assert_eq!(s.peek().unwrap().kind, ResultKind::Err);
    }
}
