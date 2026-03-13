// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! DrawCallBatcher — batches draw calls by material.

#![allow(dead_code)]

/// A single draw-call entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawCallEntry {
    pub vertex_count: u32,
    pub index_count: u32,
}

/// Configuration for batching.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub material_id: u32,
    pub max_draws: usize,
}

/// A batch of draw calls sharing the same material.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawCallBatch {
    pub config: BatchConfig,
    pub draws: Vec<DrawCallEntry>,
}

/// Create a new empty batch.
#[allow(dead_code)]
pub fn new_batch(config: BatchConfig) -> DrawCallBatch {
    DrawCallBatch {
        config,
        draws: Vec::new(),
    }
}

/// Add a draw call to the batch.
#[allow(dead_code)]
pub fn add_draw_call(batch: &mut DrawCallBatch, vertex_count: u32, index_count: u32) {
    batch.draws.push(DrawCallEntry {
        vertex_count,
        index_count,
    });
}

/// Number of draw calls in the batch.
#[allow(dead_code)]
pub fn batch_draw_count(batch: &DrawCallBatch) -> usize {
    batch.draws.len()
}

/// Flush (clear) the batch, returning the number of draws flushed.
#[allow(dead_code)]
pub fn flush_batch(batch: &mut DrawCallBatch) -> usize {
    let count = batch.draws.len();
    batch.draws.clear();
    count
}

/// Total vertex count across all draws.
#[allow(dead_code)]
pub fn batch_vertex_count(batch: &DrawCallBatch) -> u64 {
    batch.draws.iter().map(|d| d.vertex_count as u64).sum()
}

/// Total index count across all draws.
#[allow(dead_code)]
pub fn batch_index_count(batch: &DrawCallBatch) -> u64 {
    batch.draws.iter().map(|d| d.index_count as u64).sum()
}

/// Whether the batch has no draw calls.
#[allow(dead_code)]
pub fn batch_is_empty(batch: &DrawCallBatch) -> bool {
    batch.draws.is_empty()
}

/// Return the material id of the batch.
#[allow(dead_code)]
pub fn batch_material_id(batch: &DrawCallBatch) -> u32 {
    batch.config.material_id
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> BatchConfig {
        BatchConfig {
            material_id: 42,
            max_draws: 100,
        }
    }

    #[test]
    fn test_new_batch() {
        let b = new_batch(test_config());
        assert!(batch_is_empty(&b));
    }

    #[test]
    fn test_add_draw_call() {
        let mut b = new_batch(test_config());
        add_draw_call(&mut b, 100, 300);
        assert_eq!(batch_draw_count(&b), 1);
    }

    #[test]
    fn test_batch_draw_count() {
        let mut b = new_batch(test_config());
        add_draw_call(&mut b, 10, 30);
        add_draw_call(&mut b, 20, 60);
        assert_eq!(batch_draw_count(&b), 2);
    }

    #[test]
    fn test_flush_batch() {
        let mut b = new_batch(test_config());
        add_draw_call(&mut b, 10, 30);
        let flushed = flush_batch(&mut b);
        assert_eq!(flushed, 1);
        assert!(batch_is_empty(&b));
    }

    #[test]
    fn test_batch_vertex_count() {
        let mut b = new_batch(test_config());
        add_draw_call(&mut b, 10, 30);
        add_draw_call(&mut b, 20, 60);
        assert_eq!(batch_vertex_count(&b), 30);
    }

    #[test]
    fn test_batch_index_count() {
        let mut b = new_batch(test_config());
        add_draw_call(&mut b, 10, 30);
        add_draw_call(&mut b, 20, 60);
        assert_eq!(batch_index_count(&b), 90);
    }

    #[test]
    fn test_batch_is_empty() {
        let b = new_batch(test_config());
        assert!(batch_is_empty(&b));
    }

    #[test]
    fn test_batch_material_id() {
        let b = new_batch(test_config());
        assert_eq!(batch_material_id(&b), 42);
    }

    #[test]
    fn test_flush_empty() {
        let mut b = new_batch(test_config());
        let flushed = flush_batch(&mut b);
        assert_eq!(flushed, 0);
    }

    #[test]
    fn test_vertex_count_empty() {
        let b = new_batch(test_config());
        assert_eq!(batch_vertex_count(&b), 0);
    }
}
