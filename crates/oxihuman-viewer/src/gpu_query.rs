// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Type of GPU query.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    TimeElapsed,
    SamplesPassed,
    PrimitivesGenerated,
}

/// A GPU query object.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GpuQuery {
    pub query_type: QueryType,
    pub active: bool,
    pub result_ns: u64,
    pub ready: bool,
}

/// Create a new GPU query of the given type.
#[allow(dead_code)]
pub fn new_gpu_query(query_type: QueryType) -> GpuQuery {
    GpuQuery {
        query_type,
        active: false,
        result_ns: 0,
        ready: false,
    }
}

/// Begin the query.
#[allow(dead_code)]
pub fn begin_query(q: &mut GpuQuery) {
    q.active = true;
    q.ready = false;
}

/// End the query (stub: sets a deterministic result).
#[allow(dead_code)]
pub fn end_query(q: &mut GpuQuery) {
    q.active = false;
    q.result_ns = 16_666_667; // ~60fps frame time
    q.ready = true;
}

/// Return the query result (nanoseconds for time, count for others).
#[allow(dead_code)]
pub fn query_result(q: &GpuQuery) -> u64 {
    q.result_ns
}

/// Return the query type name.
#[allow(dead_code)]
pub fn query_type_name(q: &GpuQuery) -> &'static str {
    match q.query_type {
        QueryType::TimeElapsed => "time_elapsed",
        QueryType::SamplesPassed => "samples_passed",
        QueryType::PrimitivesGenerated => "primitives_generated",
    }
}

/// Return the elapsed time in nanoseconds.
#[allow(dead_code)]
pub fn query_elapsed_ns(q: &GpuQuery) -> u64 {
    q.result_ns
}

/// Reset the query.
#[allow(dead_code)]
pub fn query_reset(q: &mut GpuQuery) {
    q.active = false;
    q.result_ns = 0;
    q.ready = false;
}

/// Check if the query result is ready.
#[allow(dead_code)]
pub fn query_is_ready(q: &GpuQuery) -> bool {
    q.ready
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_query() {
        let q = new_gpu_query(QueryType::TimeElapsed);
        assert!(!q.active);
    }

    #[test]
    fn begin_sets_active() {
        let mut q = new_gpu_query(QueryType::TimeElapsed);
        begin_query(&mut q);
        assert!(q.active);
    }

    #[test]
    fn end_sets_result() {
        let mut q = new_gpu_query(QueryType::TimeElapsed);
        begin_query(&mut q);
        end_query(&mut q);
        assert!(query_result(&q) > 0);
    }

    #[test]
    fn not_ready_initially() {
        let q = new_gpu_query(QueryType::TimeElapsed);
        assert!(!query_is_ready(&q));
    }

    #[test]
    fn ready_after_end() {
        let mut q = new_gpu_query(QueryType::TimeElapsed);
        begin_query(&mut q);
        end_query(&mut q);
        assert!(query_is_ready(&q));
    }

    #[test]
    fn type_name() {
        let q = new_gpu_query(QueryType::SamplesPassed);
        assert_eq!(query_type_name(&q), "samples_passed");
    }

    #[test]
    fn elapsed_ns() {
        let mut q = new_gpu_query(QueryType::TimeElapsed);
        begin_query(&mut q);
        end_query(&mut q);
        assert_eq!(query_elapsed_ns(&q), 16_666_667);
    }

    #[test]
    fn reset_clears() {
        let mut q = new_gpu_query(QueryType::TimeElapsed);
        begin_query(&mut q);
        end_query(&mut q);
        query_reset(&mut q);
        assert!(!query_is_ready(&q));
        assert_eq!(query_result(&q), 0);
    }

    #[test]
    fn primitives_type() {
        let q = new_gpu_query(QueryType::PrimitivesGenerated);
        assert_eq!(query_type_name(&q), "primitives_generated");
    }

    #[test]
    fn begin_clears_ready() {
        let mut q = new_gpu_query(QueryType::TimeElapsed);
        begin_query(&mut q);
        end_query(&mut q);
        begin_query(&mut q);
        assert!(!query_is_ready(&q));
    }
}
