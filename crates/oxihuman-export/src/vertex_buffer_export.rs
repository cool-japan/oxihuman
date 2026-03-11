#![allow(dead_code)]
//! Export vertex buffer data.

/// Vertex buffer export configuration and data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VertexBufferExport {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub stride: usize,
}

/// Export a vertex buffer from mesh data.
#[allow(dead_code)]
pub fn export_vertex_buffer(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
) -> VertexBufferExport {
    let stride = 12 + if !normals.is_empty() { 12 } else { 0 } + if !uvs.is_empty() { 8 } else { 0 };
    VertexBufferExport {
        positions: positions.to_vec(),
        normals: normals.to_vec(),
        uvs: uvs.to_vec(),
        stride,
    }
}

/// Return the stride (bytes per vertex).
#[allow(dead_code)]
pub fn buffer_stride(export: &VertexBufferExport) -> usize {
    export.stride
}

/// Return the total buffer size in bytes.
#[allow(dead_code)]
pub fn buffer_size(export: &VertexBufferExport) -> usize {
    export.positions.len() * export.stride
}

/// Return the buffer format description.
#[allow(dead_code)]
pub fn buffer_format(export: &VertexBufferExport) -> String {
    let mut fmt = "POSITION".to_string();
    if !export.normals.is_empty() {
        fmt.push_str("+NORMAL");
    }
    if !export.uvs.is_empty() {
        fmt.push_str("+TEXCOORD0");
    }
    fmt
}

/// Return interleaved layout description.
#[allow(dead_code)]
pub fn interleaved_layout(export: &VertexBufferExport) -> String {
    format!("interleaved stride={} format={}", export.stride, buffer_format(export))
}

/// Return separate (non-interleaved) layout description.
#[allow(dead_code)]
pub fn separate_layout(export: &VertexBufferExport) -> String {
    let mut parts = vec![format!("positions: {}x12B", export.positions.len())];
    if !export.normals.is_empty() {
        parts.push(format!("normals: {}x12B", export.normals.len()));
    }
    if !export.uvs.is_empty() {
        parts.push(format!("uvs: {}x8B", export.uvs.len()));
    }
    parts.join(", ")
}

/// Convert the buffer to bytes (interleaved).
#[allow(dead_code)]
pub fn buffer_to_bytes(export: &VertexBufferExport) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(buffer_size(export));
    for i in 0..export.positions.len() {
        for &v in &export.positions[i] {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        if i < export.normals.len() {
            for &v in &export.normals[i] {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
        if i < export.uvs.len() {
            for &v in &export.uvs[i] {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
    }
    bytes
}

/// Validate the buffer data.
#[allow(dead_code)]
pub fn validate_buffer(export: &VertexBufferExport) -> bool {
    if !export.normals.is_empty() && export.normals.len() != export.positions.len() {
        return false;
    }
    if !export.uvs.is_empty() && export.uvs.len() != export.positions.len() {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> VertexBufferExport {
        export_vertex_buffer(
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            &[[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
            &[[0.0, 0.0], [1.0, 0.0]],
        )
    }

    #[test]
    fn test_export_vertex_buffer() {
        let vb = sample();
        assert_eq!(vb.positions.len(), 2);
    }

    #[test]
    fn test_buffer_stride() {
        let vb = sample();
        assert_eq!(buffer_stride(&vb), 32);
    }

    #[test]
    fn test_buffer_size() {
        let vb = sample();
        assert_eq!(buffer_size(&vb), 64);
    }

    #[test]
    fn test_buffer_format() {
        let vb = sample();
        assert!(buffer_format(&vb).contains("POSITION"));
        assert!(buffer_format(&vb).contains("NORMAL"));
    }

    #[test]
    fn test_interleaved_layout() {
        let vb = sample();
        let layout = interleaved_layout(&vb);
        assert!(layout.contains("interleaved"));
    }

    #[test]
    fn test_separate_layout() {
        let vb = sample();
        let layout = separate_layout(&vb);
        assert!(layout.contains("positions"));
    }

    #[test]
    fn test_buffer_to_bytes() {
        let vb = sample();
        let bytes = buffer_to_bytes(&vb);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_validate_buffer() {
        let vb = sample();
        assert!(validate_buffer(&vb));
    }

    #[test]
    fn test_validate_buffer_mismatch() {
        let vb = VertexBufferExport {
            positions: vec![[0.0; 3]; 3],
            normals: vec![[0.0; 3]; 2],
            uvs: vec![],
            stride: 24,
        };
        assert!(!validate_buffer(&vb));
    }

    #[test]
    fn test_buffer_positions_only() {
        let vb = export_vertex_buffer(&[[0.0; 3]], &[], &[]);
        assert_eq!(buffer_stride(&vb), 12);
    }
}
