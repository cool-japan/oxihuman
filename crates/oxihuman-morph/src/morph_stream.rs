#![allow(dead_code)]

//! Streaming morph target loader (chunked playback).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphChunk {
    pub chunk_index: u32,
    pub vertex_start: u32,
    pub vertex_count: u32,
    pub data: Vec<[f32; 3]>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphStream {
    pub name: String,
    pub chunks: Vec<MorphChunk>,
    pub chunk_size: u32,
    pub current_chunk: usize,
    pub total_vertices: u32,
    pub loaded: bool,
}

#[allow(dead_code)]
pub fn new_morph_stream(name: &str, total_vertices: u32, chunk_size: u32) -> MorphStream {
    MorphStream {
        name: name.to_string(),
        chunks: Vec::new(),
        chunk_size,
        current_chunk: 0,
        total_vertices,
        loaded: false,
    }
}

#[allow(dead_code)]
pub fn ms_push_chunk(stream: &mut MorphStream, data: Vec<[f32; 3]>) {
    let idx = stream.chunks.len() as u32;
    let start = idx * stream.chunk_size;
    let count = data.len() as u32;
    stream.chunks.push(MorphChunk {
        chunk_index: idx,
        vertex_start: start,
        vertex_count: count,
        data,
    });
}

#[allow(dead_code)]
pub fn ms_advance(stream: &mut MorphStream) -> bool {
    if stream.current_chunk + 1 < stream.chunks.len() {
        stream.current_chunk += 1;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn ms_current_chunk(stream: &MorphStream) -> Option<&MorphChunk> {
    stream.chunks.get(stream.current_chunk)
}

#[allow(dead_code)]
pub fn ms_is_complete(stream: &MorphStream) -> bool {
    stream.loaded
        || stream
            .chunks
            .last()
            .is_some_and(|c| c.vertex_start + c.vertex_count >= stream.total_vertices)
}

#[allow(dead_code)]
pub fn ms_reset(stream: &mut MorphStream) {
    stream.current_chunk = 0;
}

#[allow(dead_code)]
pub fn ms_chunk_count(stream: &MorphStream) -> usize {
    stream.chunks.len()
}

#[allow(dead_code)]
pub fn ms_loaded_vertices(stream: &MorphStream) -> u32 {
    stream.chunks.iter().map(|c| c.vertex_count).sum()
}

#[allow(dead_code)]
pub fn ms_to_json(stream: &MorphStream) -> String {
    format!(
        "{{\"name\":\"{}\",\"chunk_count\":{},\"total_vertices\":{},\"loaded_vertices\":{}}}",
        stream.name,
        stream.chunks.len(),
        stream.total_vertices,
        ms_loaded_vertices(stream)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_stream() {
        let s = new_morph_stream("skin", 1000, 256);
        assert_eq!(ms_chunk_count(&s), 0);
    }

    #[test]
    fn test_push_chunk() {
        let mut s = new_morph_stream("skin", 100, 50);
        ms_push_chunk(&mut s, vec![[0.0; 3]; 50]);
        assert_eq!(ms_chunk_count(&s), 1);
    }

    #[test]
    fn test_advance() {
        let mut s = new_morph_stream("skin", 100, 50);
        ms_push_chunk(&mut s, vec![[0.0; 3]; 50]);
        ms_push_chunk(&mut s, vec![[0.0; 3]; 50]);
        let advanced = ms_advance(&mut s);
        assert!(advanced);
        assert_eq!(s.current_chunk, 1);
    }

    #[test]
    fn test_advance_at_end() {
        let mut s = new_morph_stream("skin", 50, 50);
        ms_push_chunk(&mut s, vec![[0.0; 3]; 50]);
        let advanced = ms_advance(&mut s);
        assert!(!advanced);
    }

    #[test]
    fn test_current_chunk() {
        let mut s = new_morph_stream("skin", 100, 50);
        ms_push_chunk(&mut s, vec![[1.0; 3]; 50]);
        let chunk = ms_current_chunk(&s);
        assert!(chunk.is_some());
    }

    #[test]
    fn test_loaded_vertices() {
        let mut s = new_morph_stream("skin", 100, 50);
        ms_push_chunk(&mut s, vec![[0.0; 3]; 50]);
        assert_eq!(ms_loaded_vertices(&s), 50);
    }

    #[test]
    fn test_is_complete() {
        let mut s = new_morph_stream("skin", 50, 50);
        ms_push_chunk(&mut s, vec![[0.0; 3]; 50]);
        assert!(ms_is_complete(&s));
    }

    #[test]
    fn test_reset() {
        let mut s = new_morph_stream("skin", 100, 50);
        ms_push_chunk(&mut s, vec![[0.0; 3]; 50]);
        ms_push_chunk(&mut s, vec![[0.0; 3]; 50]);
        ms_advance(&mut s);
        ms_reset(&mut s);
        assert_eq!(s.current_chunk, 0);
    }

    #[test]
    fn test_to_json() {
        let s = new_morph_stream("face", 200, 100);
        let json = ms_to_json(&s);
        assert!(json.contains("\"name\":\"face\""));
    }

    #[test]
    fn test_chunk_vertex_start() {
        let mut s = new_morph_stream("test", 200, 100);
        ms_push_chunk(&mut s, vec![[0.0; 3]; 100]);
        ms_push_chunk(&mut s, vec![[0.0; 3]; 100]);
        assert_eq!(s.chunks[1].vertex_start, 100);
    }
}
