// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Streaming mesh export pipeline — write large meshes in chunks without
//! holding everything in memory.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum StreamFormat {
    BinaryFloat32,
    BinaryFloat16,
    AsciiCsv,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StreamingExportConfig {
    /// Vertices per chunk (default 4096).
    pub chunk_size: usize,
    pub format: StreamFormat,
    /// Run-length encode position deltas (default false).
    pub compress: bool,
}

impl Default for StreamingExportConfig {
    fn default() -> Self {
        Self {
            chunk_size: 4096,
            format: StreamFormat::BinaryFloat32,
            compress: false,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub chunk_index: u32,
    pub vertex_offset: u32,
    pub vertex_count: u32,
    /// Encoded chunk bytes.
    pub data: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StreamingExportResult {
    pub total_chunks: u32,
    pub total_vertices: u32,
    pub total_bytes: usize,
    pub format: StreamFormat,
}

// ── encoding helpers ──────────────────────────────────────────────────────────

/// Encode positions as little-endian f32 xyz tightly packed.
#[allow(dead_code)]
pub fn encode_chunk_f32(positions: &[[f32; 3]]) -> Vec<u8> {
    let mut out = Vec::with_capacity(positions.len() * 12);
    for p in positions {
        out.extend_from_slice(&p[0].to_le_bytes());
        out.extend_from_slice(&p[1].to_le_bytes());
        out.extend_from_slice(&p[2].to_le_bytes());
    }
    out
}

/// Quantize each component to u16: round((x + 100.0) * 65535.0 / 200.0),
/// range -100..100 m, stored as little-endian u16.
#[allow(dead_code)]
pub fn encode_chunk_f16(positions: &[[f32; 3]]) -> Vec<u8> {
    let mut out = Vec::with_capacity(positions.len() * 6);
    for p in positions {
        for component in p {
            let q = ((*component + 100.0) * 65535.0 / 200.0).round() as u16;
            out.extend_from_slice(&q.to_le_bytes());
        }
    }
    out
}

/// Encode positions as "x,y,z\n" per vertex.
#[allow(dead_code)]
pub fn encode_chunk_csv(positions: &[[f32; 3]]) -> Vec<u8> {
    let mut out = Vec::new();
    for p in positions {
        out.extend_from_slice(format!("{},{},{}\n", p[0], p[1], p[2]).as_bytes());
    }
    out
}

// ── decoding helpers ──────────────────────────────────────────────────────────

/// Inverse of `encode_chunk_f32`.
#[allow(dead_code)]
pub fn decode_chunk_f32(data: &[u8]) -> Vec<[f32; 3]> {
    let vertex_count = data.len() / 12;
    let mut out = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let base = i * 12;
        let x = f32::from_le_bytes(data[base..base + 4].try_into().unwrap_or_default());
        let y = f32::from_le_bytes(data[base + 4..base + 8].try_into().unwrap_or_default());
        let z = f32::from_le_bytes(data[base + 8..base + 12].try_into().unwrap_or_default());
        out.push([x, y, z]);
    }
    out
}

/// Inverse of `encode_chunk_f16`.
#[allow(dead_code)]
pub fn decode_chunk_f16(data: &[u8]) -> Vec<[f32; 3]> {
    let vertex_count = data.len() / 6;
    let mut out = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let base = i * 6;
        let mut components = [0.0f32; 3];
        for (j, c) in components.iter_mut().enumerate() {
            let q = u16::from_le_bytes(
                data[base + j * 2..base + j * 2 + 2]
                    .try_into()
                    .unwrap_or_default(),
            );
            *c = (q as f32) * 200.0 / 65535.0 - 100.0;
        }
        out.push(components);
    }
    out
}

// ── streaming pipeline ────────────────────────────────────────────────────────

/// Split positions into `cfg.chunk_size` chunks and encode each one.
#[allow(dead_code)]
pub fn stream_mesh_positions(
    positions: &[[f32; 3]],
    cfg: &StreamingExportConfig,
) -> Vec<StreamChunk> {
    if positions.is_empty() {
        return Vec::new();
    }
    let chunk_size = if cfg.chunk_size == 0 {
        4096
    } else {
        cfg.chunk_size
    };
    let total = positions.len();
    let total_chunks = total.div_ceil(chunk_size);
    let mut chunks = Vec::with_capacity(total_chunks);

    for chunk_index in 0..total_chunks {
        let offset = chunk_index * chunk_size;
        let end = (offset + chunk_size).min(total);
        let slice = &positions[offset..end];
        let data = match cfg.format {
            StreamFormat::BinaryFloat32 => encode_chunk_f32(slice),
            StreamFormat::BinaryFloat16 => encode_chunk_f16(slice),
            StreamFormat::AsciiCsv => encode_chunk_csv(slice),
        };
        chunks.push(StreamChunk {
            chunk_index: chunk_index as u32,
            vertex_offset: offset as u32,
            vertex_count: slice.len() as u32,
            data,
        });
    }
    chunks
}

/// Reconstruct the full position array from sorted chunks (sorted by chunk_index).
#[allow(dead_code)]
pub fn reassemble_chunks(chunks: &[StreamChunk]) -> Vec<[f32; 3]> {
    if chunks.is_empty() {
        return Vec::new();
    }
    // Sort a local index vector rather than cloning chunks.
    let mut sorted_indices: Vec<usize> = (0..chunks.len()).collect();
    sorted_indices.sort_by_key(|&i| chunks[i].chunk_index);

    // Determine format from chunk byte/vertex ratio heuristic.
    // We try to infer from byte density (12 = f32, 6 = f16, else csv).
    let total_vertices: usize = chunks.iter().map(|c| c.vertex_count as usize).sum();
    let total_bytes: usize = chunks.iter().map(|c| c.data.len()).sum();

    let mut out = Vec::with_capacity(total_vertices);

    for idx in sorted_indices {
        let chunk = &chunks[idx];
        let vertex_count = chunk.vertex_count as usize;
        let byte_count = chunk.data.len();

        let decoded = if vertex_count > 0 && byte_count == vertex_count * 12 {
            decode_chunk_f32(&chunk.data)
        } else if vertex_count > 0 && byte_count == vertex_count * 6 {
            decode_chunk_f16(&chunk.data)
        } else {
            // CSV — parse text lines
            let text = std::str::from_utf8(&chunk.data).unwrap_or("");
            text.lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() == 3 {
                        let x = parts[0].trim().parse::<f32>().ok()?;
                        let y = parts[1].trim().parse::<f32>().ok()?;
                        let z = parts[2].trim().parse::<f32>().ok()?;
                        Some([x, y, z])
                    } else {
                        None
                    }
                })
                .collect()
        };
        out.extend_from_slice(&decoded);
    }

    // Suppress unused warning for total_bytes in non-test builds.
    let _ = total_bytes;
    out
}

/// Return a human-readable summary string for a `StreamingExportResult`.
#[allow(dead_code)]
pub fn streaming_export_stats(result: &StreamingExportResult) -> String {
    let fmt = match result.format {
        StreamFormat::BinaryFloat32 => "BinaryFloat32",
        StreamFormat::BinaryFloat16 => "BinaryFloat16",
        StreamFormat::AsciiCsv => "AsciiCsv",
    };
    format!(
        "StreamingExport: {} vertices, {} chunks, {} bytes, format={}",
        result.total_vertices, result.total_chunks, result.total_bytes, fmt
    )
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_positions(n: usize) -> Vec<[f32; 3]> {
        (0..n)
            .map(|i| {
                let f = i as f32;
                [f * 0.1, f * 0.2, f * 0.3]
            })
            .collect()
    }

    #[test]
    fn encode_decode_f32_round_trip() {
        let positions = sample_positions(10);
        let encoded = encode_chunk_f32(&positions);
        let decoded = decode_chunk_f32(&encoded);
        assert_eq!(decoded.len(), positions.len());
        for (a, b) in positions.iter().zip(decoded.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-6);
            assert!((a[1] - b[1]).abs() < 1e-6);
            assert!((a[2] - b[2]).abs() < 1e-6);
        }
    }

    #[test]
    fn encode_decode_f16_approximate_round_trip() {
        let positions = vec![[0.0f32, 50.0, -50.0], [10.0, -10.0, 99.0]];
        let encoded = encode_chunk_f16(&positions);
        let decoded = decode_chunk_f16(&encoded);
        assert_eq!(decoded.len(), positions.len());
        // Quantization error within 200 / 65535 ≈ 0.00305
        for (a, b) in positions.iter().zip(decoded.iter()) {
            assert!((a[0] - b[0]).abs() < 0.01, "x: {} vs {}", a[0], b[0]);
            assert!((a[1] - b[1]).abs() < 0.01, "y: {} vs {}", a[1], b[1]);
            assert!((a[2] - b[2]).abs() < 0.01, "z: {} vs {}", a[2], b[2]);
        }
    }

    #[test]
    fn encode_chunk_csv_correct_line_count() {
        let positions = sample_positions(7);
        let csv_bytes = encode_chunk_csv(&positions);
        let text = std::str::from_utf8(&csv_bytes).unwrap();
        let line_count = text.lines().count();
        assert_eq!(line_count, 7);
    }

    #[test]
    fn stream_mesh_positions_chunk_count() {
        let positions = sample_positions(10000);
        let cfg = StreamingExportConfig {
            chunk_size: 4096,
            format: StreamFormat::BinaryFloat32,
            compress: false,
        };
        let chunks = stream_mesh_positions(&positions, &cfg);
        let expected = (10000usize).div_ceil(4096);
        assert_eq!(chunks.len(), expected);
    }

    #[test]
    fn stream_mesh_positions_small_chunk() {
        let positions = sample_positions(5);
        let cfg = StreamingExportConfig {
            chunk_size: 2,
            format: StreamFormat::BinaryFloat32,
            compress: false,
        };
        let chunks = stream_mesh_positions(&positions, &cfg);
        assert_eq!(chunks.len(), 3); // ceil(5/2) = 3
    }

    #[test]
    fn reassemble_chunks_f32_reconstructs_full() {
        let positions = sample_positions(100);
        let cfg = StreamingExportConfig {
            chunk_size: 30,
            format: StreamFormat::BinaryFloat32,
            compress: false,
        };
        let chunks = stream_mesh_positions(&positions, &cfg);
        let reconstructed = reassemble_chunks(&chunks);
        assert_eq!(reconstructed.len(), positions.len());
        for (a, b) in positions.iter().zip(reconstructed.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-5);
        }
    }

    #[test]
    fn reassemble_chunks_csv_reconstructs_full() {
        let positions = sample_positions(20);
        let cfg = StreamingExportConfig {
            chunk_size: 8,
            format: StreamFormat::AsciiCsv,
            compress: false,
        };
        let chunks = stream_mesh_positions(&positions, &cfg);
        let reconstructed = reassemble_chunks(&chunks);
        assert_eq!(reconstructed.len(), positions.len());
    }

    #[test]
    fn empty_positions_yields_zero_chunks() {
        let cfg = StreamingExportConfig::default();
        let chunks = stream_mesh_positions(&[], &cfg);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn streaming_export_stats_non_empty() {
        let result = StreamingExportResult {
            total_chunks: 3,
            total_vertices: 100,
            total_bytes: 1200,
            format: StreamFormat::BinaryFloat32,
        };
        let s = streaming_export_stats(&result);
        assert!(!s.is_empty());
        assert!(s.contains("100"));
        assert!(s.contains("BinaryFloat32"));
    }

    #[test]
    fn f32_chunk_size_is_n_times_12() {
        let positions = sample_positions(50);
        let encoded = encode_chunk_f32(&positions);
        assert_eq!(encoded.len(), 50 * 12);
    }

    #[test]
    fn f16_chunk_size_is_n_times_6() {
        let positions = sample_positions(50);
        let encoded = encode_chunk_f16(&positions);
        assert_eq!(encoded.len(), 50 * 6);
    }

    #[test]
    fn stream_chunk_vertex_offsets_are_correct() {
        let positions = sample_positions(10);
        let cfg = StreamingExportConfig {
            chunk_size: 3,
            format: StreamFormat::BinaryFloat32,
            compress: false,
        };
        let chunks = stream_mesh_positions(&positions, &cfg);
        assert_eq!(chunks[0].vertex_offset, 0);
        assert_eq!(chunks[1].vertex_offset, 3);
        assert_eq!(chunks[2].vertex_offset, 6);
    }

    #[test]
    fn total_vertex_count_matches_sum_of_chunk_vertex_counts() {
        let positions = sample_positions(97);
        let cfg = StreamingExportConfig {
            chunk_size: 20,
            format: StreamFormat::BinaryFloat16,
            compress: false,
        };
        let chunks = stream_mesh_positions(&positions, &cfg);
        let total: u32 = chunks.iter().map(|c| c.vertex_count).sum();
        assert_eq!(total, 97);
    }

    #[test]
    fn reassemble_chunks_f16_approximate() {
        let positions = vec![[1.0f32, 2.0, 3.0], [-5.0, 10.0, -10.0]];
        let cfg = StreamingExportConfig {
            chunk_size: 10,
            format: StreamFormat::BinaryFloat16,
            compress: false,
        };
        let chunks = stream_mesh_positions(&positions, &cfg);
        let reconstructed = reassemble_chunks(&chunks);
        assert_eq!(reconstructed.len(), 2);
        for (a, b) in positions.iter().zip(reconstructed.iter()) {
            assert!((a[0] - b[0]).abs() < 0.01);
            assert!((a[1] - b[1]).abs() < 0.01);
            assert!((a[2] - b[2]).abs() < 0.01);
        }
    }
}
