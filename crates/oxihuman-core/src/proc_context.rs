// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Processing context: key/value state bag for pipeline stages.

use std::collections::HashMap;

/// Value held in the processing context.
#[derive(Debug, Clone)]
pub enum CtxVal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Text(String),
}

/// A context carrying state through a processing pipeline.
#[derive(Debug, Default)]
pub struct ProcContext {
    entries: HashMap<String, CtxVal>,
    stage: u32,
    errors: Vec<String>,
    warnings: Vec<String>,
    done: bool,
}

#[allow(dead_code)]
impl ProcContext {
    pub fn new() -> Self {
        ProcContext::default()
    }

    pub fn set_int(&mut self, key: &str, v: i64) {
        self.entries.insert(key.to_string(), CtxVal::Int(v));
    }

    pub fn set_float(&mut self, key: &str, v: f64) {
        self.entries.insert(key.to_string(), CtxVal::Float(v));
    }

    pub fn set_bool(&mut self, key: &str, v: bool) {
        self.entries.insert(key.to_string(), CtxVal::Bool(v));
    }

    pub fn set_text(&mut self, key: &str, v: &str) {
        self.entries
            .insert(key.to_string(), CtxVal::Text(v.to_string()));
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.entries.get(key) {
            Some(CtxVal::Int(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.entries.get(key) {
            Some(CtxVal::Float(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.entries.get(key) {
            Some(CtxVal::Bool(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_text(&self, key: &str) -> Option<&str> {
        match self.entries.get(key) {
            Some(CtxVal::Text(v)) => Some(v.as_str()),
            _ => None,
        }
    }

    pub fn has(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.entries.remove(key).is_some()
    }

    pub fn stage(&self) -> u32 {
        self.stage
    }

    pub fn advance_stage(&mut self) {
        self.stage += 1;
    }

    pub fn add_error(&mut self, msg: &str) {
        self.errors.push(msg.to_string());
    }

    pub fn add_warning(&mut self, msg: &str) {
        self.warnings.push(msg.to_string());
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    pub fn mark_done(&mut self) {
        self.done = true;
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn reset(&mut self) {
        self.entries.clear();
        self.errors.clear();
        self.warnings.clear();
        self.stage = 0;
        self.done = false;
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

pub fn new_proc_context() -> ProcContext {
    ProcContext::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_int() {
        let mut ctx = new_proc_context();
        ctx.set_int("count", 42);
        assert_eq!(ctx.get_int("count"), Some(42));
    }

    #[test]
    fn set_and_get_float() {
        let mut ctx = new_proc_context();
        ctx.set_float("ratio", std::f64::consts::PI);
        assert!(ctx.get_float("ratio").unwrap() > std::f64::consts::E);
    }

    #[test]
    fn set_and_get_bool() {
        let mut ctx = new_proc_context();
        ctx.set_bool("ready", true);
        assert_eq!(ctx.get_bool("ready"), Some(true));
    }

    #[test]
    fn set_and_get_text() {
        let mut ctx = new_proc_context();
        ctx.set_text("name", "oxihuman");
        assert_eq!(ctx.get_text("name"), Some("oxihuman"));
    }

    #[test]
    fn has_and_remove() {
        let mut ctx = new_proc_context();
        ctx.set_int("x", 1);
        assert!(ctx.has("x"));
        assert!(ctx.remove("x"));
        assert!(!ctx.has("x"));
    }

    #[test]
    fn stage_advance() {
        let mut ctx = new_proc_context();
        assert_eq!(ctx.stage(), 0);
        ctx.advance_stage();
        ctx.advance_stage();
        assert_eq!(ctx.stage(), 2);
    }

    #[test]
    fn errors_and_warnings() {
        let mut ctx = new_proc_context();
        ctx.add_error("bad");
        ctx.add_warning("meh");
        assert!(ctx.has_errors());
        assert_eq!(ctx.error_count(), 1);
        assert_eq!(ctx.warning_count(), 1);
    }

    #[test]
    fn done_flag() {
        let mut ctx = new_proc_context();
        assert!(!ctx.is_done());
        ctx.mark_done();
        assert!(ctx.is_done());
    }

    #[test]
    fn reset_clears_all() {
        let mut ctx = new_proc_context();
        ctx.set_int("k", 1);
        ctx.add_error("e");
        ctx.advance_stage();
        ctx.mark_done();
        ctx.reset();
        assert_eq!(ctx.entry_count(), 0);
        assert!(!ctx.has_errors());
        assert_eq!(ctx.stage(), 0);
        assert!(!ctx.is_done());
    }
}
