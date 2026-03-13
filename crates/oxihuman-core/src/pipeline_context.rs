// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// Context object passed through a pipeline of processing stages.
#[allow(dead_code)]
pub struct PipelineContext {
    values: HashMap<String, String>,
    stage: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
    done: bool,
}

#[allow(dead_code)]
impl PipelineContext {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            stage: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            done: false,
        }
    }
    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }
    pub fn remove(&mut self, key: &str) -> bool {
        self.values.remove(key).is_some()
    }
    pub fn has(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    pub fn advance_stage(&mut self) {
        self.stage += 1;
    }
    pub fn stage(&self) -> usize {
        self.stage
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
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
    pub fn errors(&self) -> &[String] {
        &self.errors
    }
    pub fn mark_done(&mut self) {
        self.done = true;
    }
    pub fn is_done(&self) -> bool {
        self.done
    }
    pub fn value_count(&self) -> usize {
        self.values.len()
    }
    pub fn reset(&mut self) {
        self.values.clear();
        self.stage = 0;
        self.errors.clear();
        self.warnings.clear();
        self.done = false;
    }
}

impl Default for PipelineContext {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub fn new_pipeline_context() -> PipelineContext {
    PipelineContext::new()
}
#[allow(dead_code)]
pub fn ctx_set(c: &mut PipelineContext, k: &str, v: &str) {
    c.set(k, v);
}
#[allow(dead_code)]
pub fn ctx_get<'a>(c: &'a PipelineContext, k: &str) -> Option<&'a str> {
    c.get(k)
}
#[allow(dead_code)]
pub fn ctx_has(c: &PipelineContext, k: &str) -> bool {
    c.has(k)
}
#[allow(dead_code)]
pub fn ctx_advance(c: &mut PipelineContext) {
    c.advance_stage();
}
#[allow(dead_code)]
pub fn ctx_stage(c: &PipelineContext) -> usize {
    c.stage()
}
#[allow(dead_code)]
pub fn ctx_add_error(c: &mut PipelineContext, msg: &str) {
    c.add_error(msg);
}
#[allow(dead_code)]
pub fn ctx_add_warning(c: &mut PipelineContext, msg: &str) {
    c.add_warning(msg);
}
#[allow(dead_code)]
pub fn ctx_has_errors(c: &PipelineContext) -> bool {
    c.has_errors()
}
#[allow(dead_code)]
pub fn ctx_mark_done(c: &mut PipelineContext) {
    c.mark_done();
}
#[allow(dead_code)]
pub fn ctx_is_done(c: &PipelineContext) -> bool {
    c.is_done()
}
#[allow(dead_code)]
pub fn ctx_reset(c: &mut PipelineContext) {
    c.reset();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_set_get() {
        let mut c = new_pipeline_context();
        ctx_set(&mut c, "key", "val");
        assert_eq!(ctx_get(&c, "key"), Some("val"));
    }
    #[test]
    fn test_has() {
        let mut c = new_pipeline_context();
        ctx_set(&mut c, "k", "v");
        assert!(ctx_has(&c, "k"));
        assert!(!ctx_has(&c, "missing"));
    }
    #[test]
    fn test_stage_advance() {
        let mut c = new_pipeline_context();
        assert_eq!(ctx_stage(&c), 0);
        ctx_advance(&mut c);
        ctx_advance(&mut c);
        assert_eq!(ctx_stage(&c), 2);
    }
    #[test]
    fn test_add_error() {
        let mut c = new_pipeline_context();
        ctx_add_error(&mut c, "oops");
        assert!(ctx_has_errors(&c));
        assert_eq!(c.error_count(), 1);
    }
    #[test]
    fn test_add_warning() {
        let mut c = new_pipeline_context();
        ctx_add_warning(&mut c, "careful");
        assert!(c.has_warnings());
        assert_eq!(c.warning_count(), 1);
    }
    #[test]
    fn test_mark_done() {
        let mut c = new_pipeline_context();
        assert!(!ctx_is_done(&c));
        ctx_mark_done(&mut c);
        assert!(ctx_is_done(&c));
    }
    #[test]
    fn test_reset() {
        let mut c = new_pipeline_context();
        ctx_set(&mut c, "k", "v");
        ctx_add_error(&mut c, "e");
        ctx_advance(&mut c);
        ctx_reset(&mut c);
        assert!(!ctx_has_errors(&c));
        assert_eq!(ctx_stage(&c), 0);
        assert!(!ctx_has(&c, "k"));
    }
    #[test]
    fn test_remove() {
        let mut c = new_pipeline_context();
        ctx_set(&mut c, "x", "y");
        assert!(c.remove("x"));
        assert!(!ctx_has(&c, "x"));
    }
    #[test]
    fn test_value_count() {
        let mut c = new_pipeline_context();
        ctx_set(&mut c, "a", "1");
        ctx_set(&mut c, "b", "2");
        assert_eq!(c.value_count(), 2);
    }
    #[test]
    fn test_errors_slice() {
        let mut c = new_pipeline_context();
        ctx_add_error(&mut c, "e1");
        ctx_add_error(&mut c, "e2");
        assert_eq!(c.errors().len(), 2);
    }
}
