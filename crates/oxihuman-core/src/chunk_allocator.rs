// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A simple chunk-based allocator that tracks allocated slabs.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChunkAllocator {
    chunk_size: usize,
    chunks: Vec<Chunk>,
    total_allocated: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Chunk {
    pub offset: usize,
    pub size: usize,
    pub used: usize,
}

#[allow(dead_code)]
impl Chunk {
    pub fn new(offset: usize, size: usize) -> Self {
        Self {
            offset,
            size,
            used: 0,
        }
    }

    pub fn remaining(&self) -> usize {
        self.size.saturating_sub(self.used)
    }

    pub fn is_full(&self) -> bool {
        self.used >= self.size
    }

    pub fn utilization(&self) -> f32 {
        if self.size == 0 {
            return 0.0;
        }
        self.used as f32 / self.size as f32
    }
}

#[allow(dead_code)]
impl ChunkAllocator {
    pub fn new(chunk_size: usize) -> Self {
        assert!(chunk_size > 0);
        Self {
            chunk_size,
            chunks: Vec::new(),
            total_allocated: 0,
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<(usize, usize)> {
        if size > self.chunk_size {
            return None;
        }
        for (i, chunk) in self.chunks.iter_mut().enumerate() {
            if chunk.remaining() >= size {
                let off = chunk.offset + chunk.used;
                chunk.used += size;
                self.total_allocated += size;
                return Some((i, off));
            }
        }
        let offset = self.chunks.len() * self.chunk_size;
        let mut chunk = Chunk::new(offset, self.chunk_size);
        chunk.used = size;
        self.total_allocated += size;
        let idx = self.chunks.len();
        self.chunks.push(chunk);
        Some((idx, offset))
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    pub fn total_capacity(&self) -> usize {
        self.chunks.len() * self.chunk_size
    }

    pub fn average_utilization(&self) -> f32 {
        if self.chunks.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.chunks.iter().map(|c| c.utilization()).sum();
        sum / self.chunks.len() as f32
    }

    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    pub fn reset(&mut self) {
        self.chunks.clear();
        self.total_allocated = 0;
    }

    pub fn get_chunk(&self, index: usize) -> Option<&Chunk> {
        self.chunks.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let alloc = ChunkAllocator::new(256);
        assert_eq!(alloc.chunk_count(), 0);
        assert_eq!(alloc.chunk_size(), 256);
    }

    #[test]
    fn test_allocate_creates_chunk() {
        let mut alloc = ChunkAllocator::new(256);
        let result = alloc.allocate(64);
        assert!(result.is_some());
        assert_eq!(alloc.chunk_count(), 1);
    }

    #[test]
    fn test_allocate_reuses_chunk() {
        let mut alloc = ChunkAllocator::new(256);
        alloc.allocate(64);
        alloc.allocate(64);
        assert_eq!(alloc.chunk_count(), 1);
    }

    #[test]
    fn test_allocate_overflow_new_chunk() {
        let mut alloc = ChunkAllocator::new(100);
        alloc.allocate(80);
        alloc.allocate(80);
        assert_eq!(alloc.chunk_count(), 2);
    }

    #[test]
    fn test_too_large_returns_none() {
        let mut alloc = ChunkAllocator::new(64);
        assert!(alloc.allocate(128).is_none());
    }

    #[test]
    fn test_total_allocated() {
        let mut alloc = ChunkAllocator::new(256);
        alloc.allocate(50);
        alloc.allocate(30);
        assert_eq!(alloc.total_allocated(), 80);
    }

    #[test]
    fn test_reset() {
        let mut alloc = ChunkAllocator::new(256);
        alloc.allocate(50);
        alloc.reset();
        assert_eq!(alloc.chunk_count(), 0);
        assert_eq!(alloc.total_allocated(), 0);
    }

    #[test]
    fn test_chunk_utilization() {
        let c = Chunk::new(0, 100);
        assert!((c.utilization() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_average_utilization() {
        let mut alloc = ChunkAllocator::new(100);
        alloc.allocate(50);
        let u = alloc.average_utilization();
        assert!((u - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_get_chunk() {
        let mut alloc = ChunkAllocator::new(64);
        alloc.allocate(10);
        let chunk = alloc.get_chunk(0).unwrap();
        assert_eq!(chunk.used, 10);
    }
}
