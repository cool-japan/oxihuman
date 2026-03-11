#![allow(dead_code)]
//! Export vertex stream data.

/// Vertex stream export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VertexStreamExport {
    pub streams: Vec<StreamInfo>,
}

/// A single stream info.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct StreamInfo {
    pub semantic: String,
    pub components: u32,
    pub stride: u32,
    pub data: Vec<u8>,
}

/// Export vertex streams.
#[allow(dead_code)]
pub fn export_vertex_stream(streams: Vec<StreamInfo>) -> VertexStreamExport {
    VertexStreamExport { streams }
}

/// Return stream count.
#[allow(dead_code)]
pub fn stream_count(exp: &VertexStreamExport) -> usize {
    exp.streams.len()
}

/// Return stride for a stream.
#[allow(dead_code)]
pub fn stream_stride(exp: &VertexStreamExport, index: usize) -> u32 {
    if index < exp.streams.len() {
        exp.streams[index].stride
    } else {
        0
    }
}

/// Return data bytes for a stream.
#[allow(dead_code)]
pub fn stream_to_bytes(exp: &VertexStreamExport, index: usize) -> &[u8] {
    if index < exp.streams.len() {
        &exp.streams[index].data
    } else {
        &[]
    }
}

/// Return semantic for a stream.
#[allow(dead_code)]
pub fn stream_semantic(exp: &VertexStreamExport, index: usize) -> &str {
    if index < exp.streams.len() {
        &exp.streams[index].semantic
    } else {
        ""
    }
}

/// Return component count for a stream.
#[allow(dead_code)]
pub fn stream_component_count(exp: &VertexStreamExport, index: usize) -> u32 {
    if index < exp.streams.len() {
        exp.streams[index].components
    } else {
        0
    }
}

/// Compute total export size.
#[allow(dead_code)]
pub fn stream_export_size(exp: &VertexStreamExport) -> usize {
    exp.streams.iter().map(|s| s.data.len()).sum()
}

/// Validate vertex streams.
#[allow(dead_code)]
pub fn validate_vertex_stream(exp: &VertexStreamExport) -> bool {
    !exp.streams.is_empty()
        && exp.streams.iter().all(|s| !s.semantic.is_empty() && s.components > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_stream() -> StreamInfo {
        StreamInfo {
            semantic: "POSITION".to_string(),
            components: 3,
            stride: 12,
            data: vec![0u8; 36],
        }
    }

    #[test]
    fn test_export_vertex_stream() {
        let e = export_vertex_stream(vec![sample_stream()]);
        assert_eq!(stream_count(&e), 1);
    }

    #[test]
    fn test_stream_stride() {
        let e = export_vertex_stream(vec![sample_stream()]);
        assert_eq!(stream_stride(&e, 0), 12);
    }

    #[test]
    fn test_stream_stride_oob() {
        let e = export_vertex_stream(vec![]);
        assert_eq!(stream_stride(&e, 0), 0);
    }

    #[test]
    fn test_stream_to_bytes() {
        let e = export_vertex_stream(vec![sample_stream()]);
        assert_eq!(stream_to_bytes(&e, 0).len(), 36);
    }

    #[test]
    fn test_stream_semantic() {
        let e = export_vertex_stream(vec![sample_stream()]);
        assert_eq!(stream_semantic(&e, 0), "POSITION");
    }

    #[test]
    fn test_stream_component_count() {
        let e = export_vertex_stream(vec![sample_stream()]);
        assert_eq!(stream_component_count(&e, 0), 3);
    }

    #[test]
    fn test_stream_export_size() {
        let e = export_vertex_stream(vec![sample_stream()]);
        assert_eq!(stream_export_size(&e), 36);
    }

    #[test]
    fn test_validate_vertex_stream() {
        let e = export_vertex_stream(vec![sample_stream()]);
        assert!(validate_vertex_stream(&e));
    }

    #[test]
    fn test_validate_empty() {
        let e = export_vertex_stream(vec![]);
        assert!(!validate_vertex_stream(&e));
    }

    #[test]
    fn test_stream_semantic_oob() {
        let e = export_vertex_stream(vec![]);
        assert_eq!(stream_semantic(&e, 0), "");
    }
}
