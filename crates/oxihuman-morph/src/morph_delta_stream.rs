// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Streaming morph delta application for incremental updates.

#[allow(dead_code)]
pub struct DeltaChunk {
    pub target_name: String,
    pub weight: f32,
    pub start_vertex: usize,
    pub deltas: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub struct DeltaStream {
    pub chunks: Vec<DeltaChunk>,
    pub total_vertices: usize,
    pub dirty: bool,
}

#[allow(dead_code)]
pub struct StreamConfig {
    pub chunk_size: usize,
    pub threshold: f32,
    pub accumulate: bool,
}

#[allow(dead_code)]
pub fn default_stream_config() -> StreamConfig {
    StreamConfig {
        chunk_size: 256,
        threshold: 1e-6,
        accumulate: true,
    }
}

#[allow(dead_code)]
pub fn new_delta_stream(vertex_count: usize) -> DeltaStream {
    DeltaStream {
        chunks: Vec::new(),
        total_vertices: vertex_count,
        dirty: false,
    }
}

#[allow(dead_code)]
pub fn push_chunk(stream: &mut DeltaStream, chunk: DeltaChunk) {
    stream.chunks.push(chunk);
    stream.dirty = true;
}

#[allow(dead_code)]
pub fn apply_stream(stream: &DeltaStream, positions: &mut [[f32; 3]]) {
    for chunk in &stream.chunks {
        apply_chunk(chunk, positions);
    }
}

#[allow(dead_code)]
pub fn apply_chunk(chunk: &DeltaChunk, positions: &mut [[f32; 3]]) {
    for (i, delta) in chunk.deltas.iter().enumerate() {
        let vi = chunk.start_vertex + i;
        if vi < positions.len() {
            positions[vi][0] += delta[0] * chunk.weight;
            positions[vi][1] += delta[1] * chunk.weight;
            positions[vi][2] += delta[2] * chunk.weight;
        }
    }
}

#[allow(dead_code)]
pub fn split_deltas_into_chunks(
    target: &str,
    weight: f32,
    deltas: &[(usize, [f32; 3])],
    chunk_size: usize,
) -> Vec<DeltaChunk> {
    if deltas.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    for window in deltas.chunks(chunk_size) {
        let start_vertex = window[0].0;
        let chunk_deltas: Vec<[f32; 3]> = window.iter().map(|(_, d)| *d).collect();
        result.push(DeltaChunk {
            target_name: target.to_string(),
            weight,
            start_vertex,
            deltas: chunk_deltas,
        });
    }
    result
}

#[allow(dead_code)]
pub fn merge_chunks(chunks: &[DeltaChunk]) -> DeltaChunk {
    if chunks.is_empty() {
        return DeltaChunk {
            target_name: String::new(),
            weight: 0.0,
            start_vertex: 0,
            deltas: Vec::new(),
        };
    }
    let target_name = chunks[0].target_name.clone();
    let weight = chunks[0].weight;
    let start_vertex = chunks.iter().map(|c| c.start_vertex).min().unwrap_or(0);
    let max_end = chunks
        .iter()
        .map(|c| c.start_vertex + c.deltas.len())
        .max()
        .unwrap_or(0);
    let capacity = max_end.saturating_sub(start_vertex);
    let mut merged: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; capacity];
    for chunk in chunks {
        for (i, delta) in chunk.deltas.iter().enumerate() {
            let vi = chunk.start_vertex + i;
            if vi >= start_vertex && vi - start_vertex < merged.len() {
                let idx = vi - start_vertex;
                merged[idx][0] += delta[0];
                merged[idx][1] += delta[1];
                merged[idx][2] += delta[2];
            }
        }
    }
    DeltaChunk {
        target_name,
        weight,
        start_vertex,
        deltas: merged,
    }
}

#[allow(dead_code)]
pub fn stream_delta_count(stream: &DeltaStream) -> usize {
    stream.chunks.iter().map(|c| c.deltas.len()).sum()
}

#[allow(dead_code)]
pub fn clear_stream(stream: &mut DeltaStream) {
    stream.chunks.clear();
    stream.dirty = false;
}

#[allow(dead_code)]
pub fn stream_chunk_count(stream: &DeltaStream) -> usize {
    stream.chunks.len()
}

#[allow(dead_code)]
pub fn filter_stream_threshold(stream: &mut DeltaStream, threshold: f32) {
    for chunk in &mut stream.chunks {
        chunk.deltas.retain(|d| {
            let mag = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            mag >= threshold
        });
    }
    stream.chunks.retain(|c| !c.deltas.is_empty());
}

#[allow(dead_code)]
pub fn stream_memory_bytes(stream: &DeltaStream) -> usize {
    let chunk_overhead = stream.chunks.len()
        * (std::mem::size_of::<usize>() * 2
            + std::mem::size_of::<f32>()
            + std::mem::size_of::<usize>());
    let delta_bytes: usize = stream
        .chunks
        .iter()
        .map(|c| c.deltas.len() * std::mem::size_of::<[f32; 3]>())
        .sum();
    chunk_overhead + delta_bytes
}

#[allow(dead_code)]
pub fn stream_to_flat_deltas(stream: &DeltaStream) -> Vec<(usize, [f32; 3])> {
    let mut per_vertex: std::collections::HashMap<usize, [f32; 3]> =
        std::collections::HashMap::new();
    for chunk in &stream.chunks {
        for (i, delta) in chunk.deltas.iter().enumerate() {
            let vi = chunk.start_vertex + i;
            let entry = per_vertex.entry(vi).or_insert([0.0, 0.0, 0.0]);
            entry[0] += delta[0] * chunk.weight;
            entry[1] += delta[1] * chunk.weight;
            entry[2] += delta[2] * chunk.weight;
        }
    }
    let mut result: Vec<(usize, [f32; 3])> = per_vertex.into_iter().collect();
    result.sort_by_key(|(vi, _)| *vi);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_stream_config() {
        let cfg = default_stream_config();
        assert_eq!(cfg.chunk_size, 256);
        assert!(cfg.threshold < 1e-5);
        assert!(cfg.accumulate);
    }

    #[test]
    fn test_new_delta_stream() {
        let stream = new_delta_stream(1000);
        assert_eq!(stream.total_vertices, 1000);
        assert_eq!(stream.chunks.len(), 0);
        assert!(!stream.dirty);
    }

    #[test]
    fn test_push_chunk() {
        let mut stream = new_delta_stream(100);
        let chunk = DeltaChunk {
            target_name: "smile".to_string(),
            weight: 1.0,
            start_vertex: 0,
            deltas: vec![[1.0, 0.0, 0.0]],
        };
        push_chunk(&mut stream, chunk);
        assert_eq!(stream_chunk_count(&stream), 1);
        assert!(stream.dirty);
    }

    #[test]
    fn test_apply_stream() {
        let mut stream = new_delta_stream(4);
        let chunk = DeltaChunk {
            target_name: "test".to_string(),
            weight: 0.5,
            start_vertex: 0,
            deltas: vec![[2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
        };
        push_chunk(&mut stream, chunk);
        let mut positions = [[0.0f32; 3]; 4];
        apply_stream(&stream, &mut positions);
        assert!((positions[0][0] - 1.0).abs() < 1e-5);
        assert!((positions[1][1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_chunk() {
        let chunk = DeltaChunk {
            target_name: "brow".to_string(),
            weight: 2.0,
            start_vertex: 1,
            deltas: vec![[1.0, 0.5, 0.0]],
        };
        let mut positions = [[0.0f32; 3]; 4];
        apply_chunk(&chunk, &mut positions);
        assert!((positions[1][0] - 2.0).abs() < 1e-5);
        assert!((positions[1][1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_split_deltas_into_chunks() {
        let deltas: Vec<(usize, [f32; 3])> = (0..10).map(|i| (i, [1.0, 0.0, 0.0])).collect();
        let chunks = split_deltas_into_chunks("target", 1.0, &deltas, 4);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].deltas.len(), 4);
        assert_eq!(chunks[1].deltas.len(), 4);
        assert_eq!(chunks[2].deltas.len(), 2);
    }

    #[test]
    fn test_split_empty_deltas() {
        let chunks = split_deltas_into_chunks("target", 1.0, &[], 4);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_stream_chunk_count() {
        let mut stream = new_delta_stream(100);
        assert_eq!(stream_chunk_count(&stream), 0);
        push_chunk(
            &mut stream,
            DeltaChunk {
                target_name: "t".to_string(),
                weight: 1.0,
                start_vertex: 0,
                deltas: vec![[0.0, 0.0, 0.0]],
            },
        );
        assert_eq!(stream_chunk_count(&stream), 1);
    }

    #[test]
    fn test_stream_delta_count() {
        let mut stream = new_delta_stream(100);
        push_chunk(
            &mut stream,
            DeltaChunk {
                target_name: "a".to_string(),
                weight: 1.0,
                start_vertex: 0,
                deltas: vec![[1.0, 0.0, 0.0]; 5],
            },
        );
        push_chunk(
            &mut stream,
            DeltaChunk {
                target_name: "b".to_string(),
                weight: 1.0,
                start_vertex: 5,
                deltas: vec![[0.0, 1.0, 0.0]; 3],
            },
        );
        assert_eq!(stream_delta_count(&stream), 8);
    }

    #[test]
    fn test_clear_stream() {
        let mut stream = new_delta_stream(100);
        push_chunk(
            &mut stream,
            DeltaChunk {
                target_name: "t".to_string(),
                weight: 1.0,
                start_vertex: 0,
                deltas: vec![[1.0, 0.0, 0.0]],
            },
        );
        clear_stream(&mut stream);
        assert_eq!(stream_chunk_count(&stream), 0);
        assert!(!stream.dirty);
    }

    #[test]
    fn test_filter_stream_threshold() {
        let mut stream = new_delta_stream(10);
        push_chunk(
            &mut stream,
            DeltaChunk {
                target_name: "t".to_string(),
                weight: 1.0,
                start_vertex: 0,
                deltas: vec![[0.0001, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0001, 0.0]],
            },
        );
        filter_stream_threshold(&mut stream, 0.01);
        assert_eq!(stream_delta_count(&stream), 1);
    }

    #[test]
    fn test_stream_to_flat_deltas() {
        let mut stream = new_delta_stream(10);
        push_chunk(
            &mut stream,
            DeltaChunk {
                target_name: "a".to_string(),
                weight: 1.0,
                start_vertex: 0,
                deltas: vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            },
        );
        push_chunk(
            &mut stream,
            DeltaChunk {
                target_name: "b".to_string(),
                weight: 2.0,
                start_vertex: 0,
                deltas: vec![[0.5, 0.0, 0.0]],
            },
        );
        let flat = stream_to_flat_deltas(&stream);
        assert!(!flat.is_empty());
        let v0 = flat.iter().find(|(vi, _)| *vi == 0).unwrap();
        assert!((v0.1[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_merge_chunks() {
        let chunks = vec![
            DeltaChunk {
                target_name: "smile".to_string(),
                weight: 1.0,
                start_vertex: 0,
                deltas: vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            },
            DeltaChunk {
                target_name: "smile".to_string(),
                weight: 1.0,
                start_vertex: 2,
                deltas: vec![[0.0, 0.0, 1.0]],
            },
        ];
        let merged = merge_chunks(&chunks);
        assert_eq!(merged.start_vertex, 0);
        assert_eq!(merged.deltas.len(), 3);
    }

    #[test]
    fn test_stream_memory_bytes() {
        let mut stream = new_delta_stream(100);
        push_chunk(
            &mut stream,
            DeltaChunk {
                target_name: "t".to_string(),
                weight: 1.0,
                start_vertex: 0,
                deltas: vec![[1.0, 0.0, 0.0]; 10],
            },
        );
        let bytes = stream_memory_bytes(&stream);
        assert!(bytes > 0);
    }

    #[test]
    fn test_apply_stream_out_of_bounds() {
        let mut stream = new_delta_stream(2);
        push_chunk(
            &mut stream,
            DeltaChunk {
                target_name: "t".to_string(),
                weight: 1.0,
                start_vertex: 100,
                deltas: vec![[1.0, 0.0, 0.0]],
            },
        );
        let mut positions = [[0.0f32; 3]; 2];
        // should not panic
        apply_stream(&stream, &mut positions);
        assert!((positions[0][0]).abs() < 1e-5);
    }
}
