// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Middleware request pipeline — ordered chain of processing stages.

/// A request context passed through the pipeline.
#[derive(Clone, Debug)]
pub struct RequestContext {
    pub path: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub attributes: Vec<(String, String)>,
}

/// Result returned by a middleware stage.
#[derive(Clone, Debug)]
pub enum MiddlewareResult {
    Continue(RequestContext),
    Halt { status: u16, body: String },
}

/// A single middleware stage descriptor.
#[derive(Clone, Debug)]
pub struct MiddlewareStage {
    pub name: String,
    pub order: i32,
    pub enabled: bool,
}

/// Configuration for the request pipeline.
#[derive(Clone, Debug)]
pub struct PipelineConfig {
    pub max_stages: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self { max_stages: 32 }
    }
}

/// An ordered request pipeline.
pub struct RequestPipeline {
    pub config: PipelineConfig,
    stages: Vec<MiddlewareStage>,
}

impl RequestContext {
    /// Creates a simple GET request context.
    pub fn simple(path: &str) -> Self {
        Self {
            path: path.into(),
            method: "GET".into(),
            headers: Vec::new(),
            body: None,
            attributes: Vec::new(),
        }
    }

    /// Sets an attribute key-value pair.
    pub fn set_attr(&mut self, key: &str, value: &str) {
        self.attributes.retain(|(k, _)| k != key);
        self.attributes.push((key.into(), value.into()));
    }

    /// Gets an attribute value by key.
    pub fn get_attr(&self, key: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// Creates a new request pipeline.
pub fn new_pipeline(config: PipelineConfig) -> RequestPipeline {
    RequestPipeline {
        config,
        stages: Vec::new(),
    }
}

/// Registers a middleware stage, returning false if pipeline is full.
pub fn register_stage(pipeline: &mut RequestPipeline, stage: MiddlewareStage) -> bool {
    if pipeline.stages.len() >= pipeline.config.max_stages {
        return false;
    }
    pipeline.stages.push(stage);
    pipeline.stages.sort_by_key(|s| s.order);
    true
}

/// Removes a stage by name.
pub fn remove_stage(pipeline: &mut RequestPipeline, name: &str) -> bool {
    let before = pipeline.stages.len();
    pipeline.stages.retain(|s| s.name != name);
    pipeline.stages.len() < before
}

/// Returns the number of enabled stages.
pub fn enabled_stage_count(pipeline: &RequestPipeline) -> usize {
    pipeline.stages.iter().filter(|s| s.enabled).count()
}

/// Returns stage names in pipeline order.
pub fn stage_names(pipeline: &RequestPipeline) -> Vec<&str> {
    pipeline.stages.iter().map(|s| s.as_str()).collect()
}

impl MiddlewareStage {
    fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

/// Runs a stub pipeline that just passes the context through enabled stages.
pub fn run_pipeline(pipeline: &RequestPipeline, ctx: RequestContext) -> MiddlewareResult {
    let mut current = ctx;
    for stage in pipeline.stages.iter().filter(|s| s.enabled) {
        /* In a real pipeline each stage would call a handler fn; here we stamp the stage name */
        current.set_attr(&format!("passed_{}", stage.name), "true");
    }
    MiddlewareResult::Continue(current)
}

impl RequestPipeline {
    /// Creates a new pipeline with default config.
    pub fn new(config: PipelineConfig) -> Self {
        new_pipeline(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pipeline() -> RequestPipeline {
        new_pipeline(PipelineConfig::default())
    }

    fn stage(name: &str, order: i32) -> MiddlewareStage {
        MiddlewareStage {
            name: name.into(),
            order,
            enabled: true,
        }
    }

    #[test]
    fn test_register_stage_added() {
        let mut p = make_pipeline();
        assert!(register_stage(&mut p, stage("auth", 1)));
        assert_eq!(enabled_stage_count(&p), 1);
    }

    #[test]
    fn test_stages_ordered_by_order_field() {
        let mut p = make_pipeline();
        register_stage(&mut p, stage("logging", 20));
        register_stage(&mut p, stage("auth", 5));
        let names = stage_names(&p);
        assert_eq!(names[0], "auth");
        assert_eq!(names[1], "logging");
    }

    #[test]
    fn test_remove_stage() {
        let mut p = make_pipeline();
        register_stage(&mut p, stage("cors", 1));
        assert!(remove_stage(&mut p, "cors"));
        assert_eq!(enabled_stage_count(&p), 0);
    }

    #[test]
    fn test_remove_nonexistent_stage_returns_false() {
        let mut p = make_pipeline();
        assert!(!remove_stage(&mut p, "ghost"));
    }

    #[test]
    fn test_disabled_stage_not_counted() {
        let mut p = make_pipeline();
        let mut s = stage("x", 1);
        s.enabled = false;
        register_stage(&mut p, s);
        assert_eq!(enabled_stage_count(&p), 0);
    }

    #[test]
    fn test_run_pipeline_stamps_stage_names() {
        let mut p = make_pipeline();
        register_stage(&mut p, stage("logger", 1));
        let ctx = RequestContext::simple("/api");
        if let MiddlewareResult::Continue(out) = run_pipeline(&p, ctx) {
            assert_eq!(out.get_attr("passed_logger"), Some("true"));
        } else {
            panic!("expected Continue");
        }
    }

    #[test]
    fn test_request_context_set_get_attr() {
        let mut ctx = RequestContext::simple("/");
        ctx.set_attr("user", "alice");
        assert_eq!(ctx.get_attr("user"), Some("alice"));
    }

    #[test]
    fn test_capacity_limit() {
        let mut p = new_pipeline(PipelineConfig { max_stages: 1 });
        register_stage(&mut p, stage("a", 1));
        assert!(!register_stage(&mut p, stage("b", 2)));
    }

    #[test]
    fn test_get_attr_missing_returns_none() {
        let ctx = RequestContext::simple("/");
        assert!(ctx.get_attr("missing").is_none());
    }
}
