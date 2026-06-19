// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Multi-stage compression pipeline backed by real codec implementations.

use crate::_core_part2::compression_brotli::{
    brotli_compress, brotli_decompress, brotli_max_compressed_size,
};
use crate::_core_part2::compression_lz::{lz_compress, lz_compress_bound, lz_decompress};
use crate::_core_part2::compression_lz4::{lz4_compress, lz4_compress_bound, lz4_decompress};
use crate::_core_part2::compression_snappy::{
    snappy_compress, snappy_decompress, snappy_max_compressed_length,
};
use crate::_core_part2::compression_zstd::{
    zstd_compress, zstd_decompress, zstd_frame_size_estimate,
};

/// Compression algorithm selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressAlgo {
    None,
    Lz4,
    Zstd,
    Brotli,
    Deflate, // mapped to LZ77 (our built-in LZSS)
    Snappy,
    Lz77,
}

impl CompressAlgo {
    pub fn name(&self) -> &'static str {
        match self {
            CompressAlgo::None => "none",
            CompressAlgo::Lz4 => "lz4",
            CompressAlgo::Zstd => "zstd",
            CompressAlgo::Brotli => "brotli",
            CompressAlgo::Deflate => "deflate",
            CompressAlgo::Snappy => "snappy",
            CompressAlgo::Lz77 => "lz77",
        }
    }

    /// Compress `data` using this algorithm.
    pub fn compress(&self, data: &[u8]) -> Vec<u8> {
        match self {
            CompressAlgo::None => data.to_vec(),
            CompressAlgo::Lz4 => lz4_compress(data),
            CompressAlgo::Zstd => zstd_compress(data),
            CompressAlgo::Brotli => brotli_compress(data),
            CompressAlgo::Deflate | CompressAlgo::Lz77 => lz_compress(data),
            CompressAlgo::Snappy => snappy_compress(data),
        }
    }

    /// Decompress `data` using this algorithm.
    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        match self {
            CompressAlgo::None => Ok(data.to_vec()),
            CompressAlgo::Lz4 => lz4_decompress(data),
            CompressAlgo::Zstd => zstd_decompress(data),
            CompressAlgo::Brotli => brotli_decompress(data),
            CompressAlgo::Deflate | CompressAlgo::Lz77 => lz_decompress(data),
            CompressAlgo::Snappy => snappy_decompress(data),
        }
    }

    /// Return an upper bound on the compressed size for `input_len` bytes.
    pub fn max_compressed_size(&self, input_len: usize) -> usize {
        match self {
            CompressAlgo::None => input_len,
            CompressAlgo::Lz4 => lz4_compress_bound(input_len),
            CompressAlgo::Zstd => zstd_frame_size_estimate(input_len),
            CompressAlgo::Brotli => brotli_max_compressed_size(input_len),
            CompressAlgo::Deflate | CompressAlgo::Lz77 => lz_compress_bound(input_len),
            CompressAlgo::Snappy => snappy_max_compressed_length(input_len),
        }
    }
}

/// A single pipeline stage.
#[derive(Debug, Clone)]
pub struct PipelineStage {
    pub algo: CompressAlgo,
    pub level: u8,
}

impl PipelineStage {
    pub fn new(algo: CompressAlgo, level: u8) -> Self {
        PipelineStage {
            algo,
            level: level.min(9),
        }
    }
}

/// Multi-stage compression pipeline.
pub struct CompressionPipeline {
    stages: Vec<PipelineStage>,
}

impl CompressionPipeline {
    pub fn new() -> Self {
        CompressionPipeline { stages: Vec::new() }
    }

    pub fn add_stage(&mut self, stage: PipelineStage) {
        self.stages.push(stage);
    }

    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Compress data through all stages using real codec implementations.
    pub fn compress(&self, data: &[u8]) -> CompressResult {
        let original_len = data.len();
        let mut current = data.to_vec();

        for stage in &self.stages {
            if stage.algo != CompressAlgo::None {
                current = stage.algo.compress(&current);
            }
        }

        CompressResult {
            data: current,
            original_size: original_len,
            stages_applied: self.stages.len(),
        }
    }

    /// Decompress data through all stages in reverse order.
    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let mut current = data.to_vec();

        for stage in self.stages.iter().rev() {
            if stage.algo != CompressAlgo::None {
                current = stage.algo.decompress(&current)?;
            }
        }

        Ok(current)
    }

    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }
}

impl Default for CompressionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of pipeline compression.
pub struct CompressResult {
    pub data: Vec<u8>,
    pub original_size: usize,
    pub stages_applied: usize,
}

impl CompressResult {
    pub fn ratio(&self) -> f64 {
        if self.original_size == 0 {
            1.0
        } else {
            self.data.len() as f64 / self.original_size as f64
        }
    }

    pub fn bytes_saved(&self) -> usize {
        self.original_size.saturating_sub(self.data.len())
    }
}

/// Build a default Zstd pipeline.
pub fn zstd_pipeline(level: u8) -> CompressionPipeline {
    let mut p = CompressionPipeline::new();
    p.add_stage(PipelineStage::new(CompressAlgo::Zstd, level));
    p
}

/// Build a two-stage LZ4 + Brotli pipeline.
pub fn lz4_brotli_pipeline() -> CompressionPipeline {
    let mut p = CompressionPipeline::new();
    p.add_stage(PipelineStage::new(CompressAlgo::Lz4, 1));
    p.add_stage(PipelineStage::new(CompressAlgo::Brotli, 6));
    p
}

/// Compress bytes with a given algorithm at level 6.
pub fn compress_bytes(algo: CompressAlgo, data: &[u8]) -> Vec<u8> {
    algo.compress(data)
}

/// Estimate the compressed size for `original` bytes.
/// Uses the compress-bound functions of the default algorithm (Zstd).
pub fn estimate_compressed_size(original: usize) -> usize {
    zstd_frame_size_estimate(original)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pipeline() {
        let p = CompressionPipeline::new();
        assert!(p.is_empty());
    }

    #[test]
    fn test_compress_passthrough_no_stages() {
        let p = CompressionPipeline::new();
        let r = p.compress(b"hello world");
        assert_eq!(r.data, b"hello world");
        assert_eq!(r.stages_applied, 0);
    }

    #[test]
    fn test_zstd_pipeline_has_one_stage() {
        let p = zstd_pipeline(3);
        assert_eq!(p.stage_count(), 1);
    }

    #[test]
    fn test_lz4_brotli_two_stages() {
        let p = lz4_brotli_pipeline();
        assert_eq!(p.stage_count(), 2);
    }

    #[test]
    fn test_zstd_compress_reduces_size_for_repetitive() {
        let p = zstd_pipeline(6);
        let data = vec![0u8; 100];
        let r = p.compress(&data);
        assert!(r.data.len() <= data.len());
    }

    #[test]
    fn test_compress_result_ratio() {
        let r = CompressResult {
            data: vec![0u8; 90],
            original_size: 100,
            stages_applied: 1,
        };
        assert!((r.ratio() - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_bytes_saved() {
        let r = CompressResult {
            data: vec![0u8; 80],
            original_size: 100,
            stages_applied: 1,
        };
        assert_eq!(r.bytes_saved(), 20);
    }

    #[test]
    fn test_compress_bytes_helper_lz4() {
        let data = vec![0u8; 100];
        let compressed = compress_bytes(CompressAlgo::Lz4, &data);
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_compress_bytes_helper_zstd() {
        let data = b"hello zstd pipeline";
        let compressed = compress_bytes(CompressAlgo::Zstd, data);
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_compress_bytes_helper_snappy() {
        let data = b"hello snappy pipeline";
        let compressed = compress_bytes(CompressAlgo::Snappy, data);
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_compress_bytes_helper_lz77() {
        let data = b"hello lz77 pipeline";
        let compressed = compress_bytes(CompressAlgo::Lz77, data);
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_algo_name() {
        assert_eq!(CompressAlgo::Zstd.name(), "zstd");
        assert_eq!(CompressAlgo::None.name(), "none");
        assert_eq!(CompressAlgo::Snappy.name(), "snappy");
        assert_eq!(CompressAlgo::Lz77.name(), "lz77");
    }

    #[test]
    fn test_stage_level_clamped() {
        let s = PipelineStage::new(CompressAlgo::Deflate, 99);
        assert_eq!(s.level, 9);
    }

    #[test]
    fn test_roundtrip_single_stage_lz4() {
        let p = {
            let mut p = CompressionPipeline::new();
            p.add_stage(PipelineStage::new(CompressAlgo::Lz4, 1));
            p
        };
        let data = b"hello lz4 roundtrip";
        let compressed = p.compress(data);
        let decompressed = p.decompress(&compressed.data).expect("decompress failed");
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_roundtrip_single_stage_zstd() {
        let p = zstd_pipeline(3);
        let data = b"hello zstd roundtrip";
        let compressed = p.compress(data);
        let decompressed = p.decompress(&compressed.data).expect("decompress failed");
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_roundtrip_single_stage_snappy() {
        let p = {
            let mut p = CompressionPipeline::new();
            p.add_stage(PipelineStage::new(CompressAlgo::Snappy, 1));
            p
        };
        let data = b"snappy roundtrip test";
        let compressed = p.compress(data);
        let decompressed = p.decompress(&compressed.data).expect("decompress failed");
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_estimate_compressed_size() {
        let est = estimate_compressed_size(100);
        assert!(est >= 100);
    }

    #[test]
    fn test_max_compressed_size_for_algos() {
        for algo in [
            CompressAlgo::Lz4,
            CompressAlgo::Zstd,
            CompressAlgo::Snappy,
            CompressAlgo::Lz77,
            CompressAlgo::Brotli,
        ] {
            let bound = algo.max_compressed_size(100);
            assert!(bound >= 100, "bound for {:?} was {}", algo, bound);
        }
    }
}
