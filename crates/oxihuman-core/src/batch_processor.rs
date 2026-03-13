// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Processes items in configurable batch sizes with progress tracking.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BatchProcessor {
    batch_size: usize,
    total_items: usize,
    processed: usize,
    failed: usize,
    current_batch: usize,
}

#[allow(dead_code)]
impl BatchProcessor {
    pub fn new(batch_size: usize, total_items: usize) -> Self {
        Self {
            batch_size: batch_size.max(1),
            total_items,
            processed: 0,
            failed: 0,
            current_batch: 0,
        }
    }

    pub fn batch_count(&self) -> usize {
        if self.total_items == 0 {
            return 0;
        }
        self.total_items.div_ceil(self.batch_size)
    }

    pub fn current_batch(&self) -> usize {
        self.current_batch
    }

    pub fn batch_range(&self, batch_idx: usize) -> (usize, usize) {
        let start = batch_idx * self.batch_size;
        let end = (start + self.batch_size).min(self.total_items);
        (start, end)
    }

    pub fn advance_batch(&mut self, success_count: usize, fail_count: usize) {
        self.processed += success_count;
        self.failed += fail_count;
        self.current_batch += 1;
    }

    pub fn is_complete(&self) -> bool {
        self.processed + self.failed >= self.total_items
    }

    pub fn progress(&self) -> f32 {
        if self.total_items == 0 {
            return 1.0;
        }
        (self.processed + self.failed) as f32 / self.total_items as f32
    }

    pub fn remaining(&self) -> usize {
        self.total_items
            .saturating_sub(self.processed + self.failed)
    }

    pub fn processed(&self) -> usize {
        self.processed
    }

    pub fn failed(&self) -> usize {
        self.failed
    }

    pub fn total_items(&self) -> usize {
        self.total_items
    }

    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    pub fn success_rate(&self) -> f32 {
        let done = self.processed + self.failed;
        if done == 0 {
            return 1.0;
        }
        self.processed as f32 / done as f32
    }

    pub fn reset(&mut self) {
        self.processed = 0;
        self.failed = 0;
        self.current_batch = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bp = BatchProcessor::new(10, 100);
        assert_eq!(bp.batch_size(), 10);
        assert_eq!(bp.total_items(), 100);
    }

    #[test]
    fn test_batch_count() {
        assert_eq!(BatchProcessor::new(10, 100).batch_count(), 10);
        assert_eq!(BatchProcessor::new(10, 95).batch_count(), 10);
        assert_eq!(BatchProcessor::new(10, 0).batch_count(), 0);
    }

    #[test]
    fn test_batch_range() {
        let bp = BatchProcessor::new(10, 25);
        assert_eq!(bp.batch_range(0), (0, 10));
        assert_eq!(bp.batch_range(2), (20, 25));
    }

    #[test]
    fn test_advance_batch() {
        let mut bp = BatchProcessor::new(10, 20);
        bp.advance_batch(10, 0);
        assert_eq!(bp.processed(), 10);
        assert_eq!(bp.current_batch(), 1);
    }

    #[test]
    fn test_is_complete() {
        let mut bp = BatchProcessor::new(5, 10);
        assert!(!bp.is_complete());
        bp.advance_batch(5, 0);
        bp.advance_batch(5, 0);
        assert!(bp.is_complete());
    }

    #[test]
    fn test_progress() {
        let mut bp = BatchProcessor::new(5, 10);
        bp.advance_batch(5, 0);
        assert!((bp.progress() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_remaining() {
        let mut bp = BatchProcessor::new(5, 10);
        bp.advance_batch(3, 2);
        assert_eq!(bp.remaining(), 5);
    }

    #[test]
    fn test_success_rate() {
        let mut bp = BatchProcessor::new(10, 20);
        bp.advance_batch(8, 2);
        assert!((bp.success_rate() - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_reset() {
        let mut bp = BatchProcessor::new(5, 10);
        bp.advance_batch(5, 0);
        bp.reset();
        assert_eq!(bp.processed(), 0);
        assert_eq!(bp.current_batch(), 0);
    }

    #[test]
    fn test_empty_total() {
        let bp = BatchProcessor::new(10, 0);
        assert!(bp.is_complete());
        assert!((bp.progress() - 1.0).abs() < f32::EPSILON);
    }
}
