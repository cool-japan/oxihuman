//! Morph delta compress — delta-encode and RLE-compress morph vertex displacements
//! for compact storage. Uses a simple threshold-based delta encoding with run-length
//! encoding of zero runs.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphDeltaCompressConfig {
    pub threshold: f32,
    pub quantize_scale: f32,
    pub max_run_length: u32,
}

/// A single stored entry: either a non-zero delta value or a zero-run count.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum DeltaToken {
    Value(i32),
    ZeroRun(u32),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CompressedMorphDelta {
    pub config: MorphDeltaCompressConfig,
    pub tokens: Vec<DeltaToken>,
    pub vertex_count: usize,
    pub original_byte_size: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphDeltaCompressResult {
    pub compressed: CompressedMorphDelta,
    pub ratio: f32,
    pub non_zero_count: usize,
}

#[allow(dead_code)]
pub fn default_morph_delta_compress_config() -> MorphDeltaCompressConfig {
    MorphDeltaCompressConfig {
        threshold: 1e-4,
        quantize_scale: 10000.0,
        max_run_length: 255,
    }
}

#[allow(dead_code)]
pub fn compress_morph_deltas(
    deltas: &[f32],
    cfg: &MorphDeltaCompressConfig,
) -> MorphDeltaCompressResult {
    let vertex_count = deltas.len();
    let original_byte_size = vertex_count * 4; // f32 = 4 bytes each

    let mut tokens = Vec::new();
    let mut zero_run = 0u32;
    let mut non_zero_count = 0usize;

    for &d in deltas {
        if d.abs() < cfg.threshold {
            zero_run += 1;
            if zero_run == cfg.max_run_length {
                tokens.push(DeltaToken::ZeroRun(zero_run));
                zero_run = 0;
            }
        } else {
            if zero_run > 0 {
                tokens.push(DeltaToken::ZeroRun(zero_run));
                zero_run = 0;
            }
            let quantized = (d * cfg.quantize_scale).round() as i32;
            tokens.push(DeltaToken::Value(quantized));
            non_zero_count += 1;
        }
    }

    if zero_run > 0 {
        tokens.push(DeltaToken::ZeroRun(zero_run));
    }

    let compressed = CompressedMorphDelta {
        config: cfg.clone(),
        tokens,
        vertex_count,
        original_byte_size,
    };

    let byte_sz = compressed_byte_size(&compressed);
    let ratio = if byte_sz == 0 {
        1.0
    } else {
        original_byte_size as f32 / byte_sz as f32
    };

    MorphDeltaCompressResult {
        compressed,
        ratio,
        non_zero_count,
    }
}

#[allow(dead_code)]
pub fn decompress_morph_deltas(compressed: &CompressedMorphDelta) -> Vec<f32> {
    let scale = compressed.config.quantize_scale;
    let mut result = Vec::with_capacity(compressed.vertex_count);

    for token in &compressed.tokens {
        match token {
            DeltaToken::Value(v) => result.push(*v as f32 / scale),
            DeltaToken::ZeroRun(n) => {
                result.resize(result.len() + *n as usize, 0.0);
            }
        }
    }

    result
}

#[allow(dead_code)]
pub fn compressed_byte_size(compressed: &CompressedMorphDelta) -> usize {
    // Each Value token: 4 bytes (i32), each ZeroRun token: 1 byte (u8 run length tag + value)
    // We use a simple model: Value = 5 bytes (1 tag + 4 data), ZeroRun = 2 bytes (1 tag + 1 count)
    compressed
        .tokens
        .iter()
        .map(|t| match t {
            DeltaToken::Value(_) => 5,
            DeltaToken::ZeroRun(_) => 2,
        })
        .sum()
}

#[allow(dead_code)]
pub fn compression_ratio(result: &MorphDeltaCompressResult) -> f32 {
    result.ratio
}

#[allow(dead_code)]
pub fn delta_entry_count(compressed: &CompressedMorphDelta) -> usize {
    compressed.tokens.len()
}

#[allow(dead_code)]
pub fn compress_to_json(compressed: &CompressedMorphDelta) -> String {
    format!(
        "{{\"vertex_count\":{},\"token_count\":{},\"original_bytes\":{},\"compressed_bytes\":{}}}",
        compressed.vertex_count,
        compressed.tokens.len(),
        compressed.original_byte_size,
        compressed_byte_size(compressed),
    )
}

#[allow(dead_code)]
pub fn compress_validate(result: &MorphDeltaCompressResult, original: &[f32]) -> bool {
    let decompressed = decompress_morph_deltas(&result.compressed);
    if decompressed.len() != original.len() {
        return false;
    }
    let threshold = result.compressed.config.threshold + 1.0 / result.compressed.config.quantize_scale;
    decompressed
        .iter()
        .zip(original.iter())
        .all(|(d, o)| (d - o).abs() <= threshold + 1e-4)
}

#[allow(dead_code)]
pub fn compress_reset(compressed: &mut CompressedMorphDelta) {
    compressed.tokens.clear();
    compressed.vertex_count = 0;
    compressed.original_byte_size = 0;
}

#[allow(dead_code)]
pub fn compress_threshold(cfg: &MorphDeltaCompressConfig) -> f32 {
    cfg.threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> MorphDeltaCompressConfig {
        default_morph_delta_compress_config()
    }

    #[test]
    fn test_default_config() {
        let c = cfg();
        assert!(c.threshold > 0.0);
        assert!(c.quantize_scale > 0.0);
    }

    #[test]
    fn test_compress_all_zeros() {
        let deltas = vec![0.0f32; 10];
        let r = compress_morph_deltas(&deltas, &cfg());
        assert_eq!(r.non_zero_count, 0);
        // Should have one ZeroRun token
        assert_eq!(r.compressed.tokens.len(), 1);
    }

    #[test]
    fn test_compress_nonzero() {
        let deltas = vec![0.0, 0.5, 0.0, -0.3, 0.0];
        let r = compress_morph_deltas(&deltas, &cfg());
        assert_eq!(r.non_zero_count, 2);
    }

    #[test]
    fn test_decompress_roundtrip() {
        let deltas = vec![0.0, 0.1, 0.2, 0.0, -0.5, 0.0, 0.0, 0.3];
        let r = compress_morph_deltas(&deltas, &cfg());
        assert!(compress_validate(&r, &deltas));
    }

    #[test]
    fn test_compressed_byte_size_smaller() {
        // Sparse deltas should compress well
        let mut deltas = vec![0.0f32; 100];
        deltas[10] = 0.5;
        deltas[50] = -0.2;
        let r = compress_morph_deltas(&deltas, &cfg());
        assert!(compressed_byte_size(&r.compressed) < r.compressed.original_byte_size);
    }

    #[test]
    fn test_compression_ratio_gt_one_sparse() {
        let mut deltas = vec![0.0f32; 100];
        deltas[10] = 0.5;
        let r = compress_morph_deltas(&deltas, &cfg());
        assert!(compression_ratio(&r) > 1.0);
    }

    #[test]
    fn test_delta_entry_count() {
        let deltas = vec![0.0, 0.5, 0.0, -0.3];
        let r = compress_morph_deltas(&deltas, &cfg());
        assert_eq!(delta_entry_count(&r.compressed), r.compressed.tokens.len());
    }

    #[test]
    fn test_to_json() {
        let deltas = vec![0.1, 0.2, 0.3];
        let r = compress_morph_deltas(&deltas, &cfg());
        let j = compress_to_json(&r.compressed);
        assert!(j.contains("vertex_count"));
        assert!(j.contains("token_count"));
    }

    #[test]
    fn test_reset() {
        let deltas = vec![0.1, 0.2, 0.3];
        let r = compress_morph_deltas(&deltas, &cfg());
        let mut c = r.compressed.clone();
        compress_reset(&mut c);
        assert_eq!(c.vertex_count, 0);
        assert!(c.tokens.is_empty());
    }

    #[test]
    fn test_threshold() {
        let c = cfg();
        assert!((compress_threshold(&c) - c.threshold).abs() < 1e-9);
    }
}
