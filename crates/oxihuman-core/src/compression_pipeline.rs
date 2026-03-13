// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Multi-stage compression pipeline stub.

/// Compression algorithm selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressAlgo {
    None,
    Lz4,
    Zstd,
    Brotli,
    Deflate,
}

impl CompressAlgo {
    pub fn name(&self) -> &'static str {
        match self {
            CompressAlgo::None => "none",
            CompressAlgo::Lz4 => "lz4",
            CompressAlgo::Zstd => "zstd",
            CompressAlgo::Brotli => "brotli",
            CompressAlgo::Deflate => "deflate",
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

    /// Compress data through all stages (stub: returns same data with metadata).
    pub fn compress(&self, data: &[u8]) -> CompressResult {
        let mut out = data.to_vec();
        let original_len = data.len();
        for stage in &self.stages {
            if stage.algo != CompressAlgo::None && !out.is_empty() {
                /* Stub: simulate compression by reducing size by 10% per stage */
                let new_len = (out.len() * 9 / 10).max(1);
                out.truncate(new_len);
            }
        }
        CompressResult {
            data: out,
            original_size: original_len,
            stages_applied: self.stages.len(),
        }
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
    let mut p = CompressionPipeline::new();
    p.add_stage(PipelineStage::new(algo, 6));
    p.compress(data).data
}

/// Estimate compressed size (stub: 90% of original).
pub fn estimate_compressed_size(original: usize) -> usize {
    original * 9 / 10
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
    fn test_compress_reduces_size() {
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
    fn test_compress_bytes_helper() {
        let compressed = compress_bytes(CompressAlgo::Lz4, &[0u8; 100]);
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_algo_name() {
        assert_eq!(CompressAlgo::Zstd.name(), "zstd");
        assert_eq!(CompressAlgo::None.name(), "none");
    }

    #[test]
    fn test_stage_level_clamped() {
        let s = PipelineStage::new(CompressAlgo::Deflate, 99);
        assert_eq!(s.level, 9);
    }
}
