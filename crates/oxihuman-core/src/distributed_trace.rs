// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Distributed trace context propagation.

/// W3C-style trace context.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceContext {
    pub trace_id: u128,
    pub span_id: u64,
    pub parent_span_id: Option<u64>,
    pub sampled: bool,
}

impl TraceContext {
    pub fn new(trace_id: u128, span_id: u64, sampled: bool) -> Self {
        Self {
            trace_id,
            span_id,
            parent_span_id: None,
            sampled,
        }
    }

    pub fn child(&self, new_span_id: u64) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id: new_span_id,
            parent_span_id: Some(self.span_id),
            sampled: self.sampled,
        }
    }

    /// Serialize to W3C traceparent header format (simplified).
    pub fn to_traceparent(&self) -> String {
        format!(
            "00-{:032x}-{:016x}-{:02x}",
            self.trace_id,
            self.span_id,
            if self.sampled { 1u8 } else { 0u8 }
        )
    }

    /// Parse from a W3C traceparent header string.
    pub fn from_traceparent(header: &str) -> Option<Self> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() != 4 {
            return None;
        }
        let trace_id = u128::from_str_radix(parts[1], 16).ok()?;
        let span_id = u64::from_str_radix(parts[2], 16).ok()?;
        let flags = u8::from_str_radix(parts[3], 16).ok()?;
        Some(Self {
            trace_id,
            span_id,
            parent_span_id: None,
            sampled: flags & 1 == 1,
        })
    }
}

pub fn new_trace_context(trace_id: u128, span_id: u64, sampled: bool) -> TraceContext {
    TraceContext::new(trace_id, span_id, sampled)
}

pub fn trace_child(ctx: &TraceContext, new_span_id: u64) -> TraceContext {
    ctx.child(new_span_id)
}

pub fn trace_to_header(ctx: &TraceContext) -> String {
    ctx.to_traceparent()
}

pub fn trace_from_header(header: &str) -> Option<TraceContext> {
    TraceContext::from_traceparent(header)
}

pub fn trace_is_sampled(ctx: &TraceContext) -> bool {
    ctx.sampled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_context() {
        let ctx = new_trace_context(42, 1, true);
        assert_eq!(ctx.trace_id, 42);
        assert_eq!(ctx.span_id, 1);
        assert!(trace_is_sampled(&ctx));
    }

    #[test]
    fn test_child_inherits_trace_id() {
        let parent = new_trace_context(100, 1, true);
        let child = trace_child(&parent, 2);
        assert_eq!(child.trace_id, 100);
        assert_eq!(child.parent_span_id, Some(1));
    }

    #[test]
    fn test_child_has_new_span_id() {
        let parent = new_trace_context(1, 10, false);
        let child = trace_child(&parent, 20);
        assert_eq!(child.span_id, 20);
    }

    #[test]
    fn test_to_traceparent_format() {
        let ctx = new_trace_context(1, 2, true);
        let h = trace_to_header(&ctx);
        assert!(h.starts_with("00-"));
        assert!(h.ends_with("-01"));
    }

    #[test]
    fn test_from_traceparent_roundtrip() {
        let ctx = new_trace_context(0xdeadbeef, 0xcafe, true);
        let h = trace_to_header(&ctx);
        let parsed = trace_from_header(&h).unwrap();
        assert_eq!(parsed.trace_id, 0xdeadbeef);
        assert_eq!(parsed.span_id, 0xcafe);
        assert!(parsed.sampled);
    }

    #[test]
    fn test_not_sampled_flag() {
        let ctx = new_trace_context(1, 1, false);
        let h = trace_to_header(&ctx);
        assert!(h.ends_with("-00"));
    }

    #[test]
    fn test_invalid_header_returns_none() {
        assert_eq!(trace_from_header("bad-header"), None);
    }

    #[test]
    fn test_child_inherits_sampled() {
        let parent = new_trace_context(1, 1, false);
        let child = trace_child(&parent, 2);
        assert!(!trace_is_sampled(&child));
    }

    #[test]
    fn test_no_parent_span_for_root() {
        let ctx = new_trace_context(1, 1, true);
        assert_eq!(ctx.parent_span_id, None);
    }
}
