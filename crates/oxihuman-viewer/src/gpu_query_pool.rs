// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! GPU query pool management.

/// Query type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QueryType {
    Timestamp,
    Occlusion,
    PipelineStatistics,
}

/// A GPU query.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GpuQueryEntry {
    pub query_type: QueryType,
    pub index: u32,
    pub result: Option<u64>,
    pub label: String,
}

/// GPU query pool.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GpuQueryPool {
    pub queries: Vec<GpuQueryEntry>,
    pub pool_type: QueryType,
    pub capacity: u32,
}

/// Create a new query pool.
#[allow(dead_code)]
pub fn new_gpu_query_pool(pool_type: QueryType, capacity: u32) -> GpuQueryPool {
    GpuQueryPool {
        queries: Vec::with_capacity(capacity as usize),
        pool_type,
        capacity,
    }
}

/// Allocate a query from the pool.
#[allow(dead_code)]
pub fn allocate_query(pool: &mut GpuQueryPool, label: &str) -> Option<u32> {
    if pool.queries.len() >= pool.capacity as usize {
        return None;
    }
    let idx = pool.queries.len() as u32;
    pool.queries.push(GpuQueryEntry {
        query_type: pool.pool_type,
        index: idx,
        result: None,
        label: label.to_string(),
    });
    Some(idx)
}

/// Set query result.
#[allow(dead_code)]
pub fn set_query_result(pool: &mut GpuQueryPool, index: u32, result: u64) -> bool {
    if let Some(q) = pool.queries.get_mut(index as usize) {
        q.result = Some(result);
        true
    } else {
        false
    }
}

/// Get query result.
#[allow(dead_code)]
pub fn get_query_result(pool: &GpuQueryPool, index: u32) -> Option<u64> {
    pool.queries.get(index as usize).and_then(|q| q.result)
}

/// Reset pool.
#[allow(dead_code)]
pub fn reset_query_pool(pool: &mut GpuQueryPool) {
    pool.queries.clear();
}

/// Active query count.
#[allow(dead_code)]
pub fn active_query_count(pool: &GpuQueryPool) -> usize {
    pool.queries.len()
}

/// Available slots.
#[allow(dead_code)]
pub fn available_slots(pool: &GpuQueryPool) -> u32 {
    pool.capacity.saturating_sub(pool.queries.len() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool() {
        let p = new_gpu_query_pool(QueryType::Timestamp, 64);
        assert_eq!(p.capacity, 64);
    }

    #[test]
    fn test_allocate() {
        let mut p = new_gpu_query_pool(QueryType::Timestamp, 64);
        let idx = allocate_query(&mut p, "test");
        assert_eq!(idx, Some(0));
    }

    #[test]
    fn test_allocate_full() {
        let mut p = new_gpu_query_pool(QueryType::Timestamp, 1);
        allocate_query(&mut p, "q0");
        let idx = allocate_query(&mut p, "q1");
        assert!(idx.is_none());
    }

    #[test]
    fn test_set_result() {
        let mut p = new_gpu_query_pool(QueryType::Timestamp, 64);
        allocate_query(&mut p, "test");
        assert!(set_query_result(&mut p, 0, 12345));
    }

    #[test]
    fn test_get_result() {
        let mut p = new_gpu_query_pool(QueryType::Timestamp, 64);
        allocate_query(&mut p, "test");
        set_query_result(&mut p, 0, 42);
        assert_eq!(get_query_result(&p, 0), Some(42));
    }

    #[test]
    fn test_get_no_result() {
        let mut p = new_gpu_query_pool(QueryType::Timestamp, 64);
        allocate_query(&mut p, "test");
        assert_eq!(get_query_result(&p, 0), None);
    }

    #[test]
    fn test_reset() {
        let mut p = new_gpu_query_pool(QueryType::Timestamp, 64);
        allocate_query(&mut p, "test");
        reset_query_pool(&mut p);
        assert_eq!(active_query_count(&p), 0);
    }

    #[test]
    fn test_available_slots() {
        let mut p = new_gpu_query_pool(QueryType::Timestamp, 10);
        allocate_query(&mut p, "test");
        assert_eq!(available_slots(&p), 9);
    }

    #[test]
    fn test_query_type() {
        let p = new_gpu_query_pool(QueryType::Occlusion, 8);
        assert_eq!(p.pool_type, QueryType::Occlusion);
    }

    #[test]
    fn test_set_invalid_index() {
        let mut p = new_gpu_query_pool(QueryType::Timestamp, 64);
        assert!(!set_query_result(&mut p, 0, 100));
    }
}
