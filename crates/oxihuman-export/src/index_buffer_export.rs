#![allow(dead_code)]
//! Export index buffer data.

/// Index buffer export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct IndexBufferExport {
    pub indices: Vec<u32>,
    pub format: IndexFormat,
}

/// Index format type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexFormat {
    U16,
    U32,
}

/// Export an index buffer from triangle indices.
#[allow(dead_code)]
pub fn export_index_buffer(indices: &[[u32; 3]]) -> IndexBufferExport {
    let flat: Vec<u32> = indices.iter().flat_map(|t| t.iter().copied()).collect();
    let max_idx = flat.iter().copied().max().unwrap_or(0);
    let format = if max_idx <= u16::MAX as u32 { IndexFormat::U16 } else { IndexFormat::U32 };
    IndexBufferExport { indices: flat, format }
}

/// Return the byte size per index.
#[allow(dead_code)]
pub fn index_format_size(format: IndexFormat) -> usize {
    match format {
        IndexFormat::U16 => 2,
        IndexFormat::U32 => 4,
    }
}

/// Convert indices to u16 (clamping if needed).
#[allow(dead_code)]
pub fn index_to_u16(indices: &[u32]) -> Vec<u16> {
    indices.iter().map(|&i| i.min(u16::MAX as u32) as u16).collect()
}

/// Convert indices to u32 (identity).
#[allow(dead_code)]
pub fn index_to_u32(indices: &[u32]) -> Vec<u32> {
    indices.to_vec()
}

/// Convert the index buffer to bytes.
#[allow(dead_code)]
pub fn index_buffer_bytes(export: &IndexBufferExport) -> Vec<u8> {
    match export.format {
        IndexFormat::U16 => {
            let mut bytes = Vec::with_capacity(export.indices.len() * 2);
            for &i in &export.indices {
                bytes.extend_from_slice(&(i as u16).to_le_bytes());
            }
            bytes
        }
        IndexFormat::U32 => {
            let mut bytes = Vec::with_capacity(export.indices.len() * 4);
            for &i in &export.indices {
                bytes.extend_from_slice(&i.to_le_bytes());
            }
            bytes
        }
    }
}

/// Return the triangle count.
#[allow(dead_code)]
pub fn triangle_count_export(export: &IndexBufferExport) -> usize {
    export.indices.len() / 3
}

/// Validate that indices form complete triangles.
#[allow(dead_code)]
pub fn validate_indices(export: &IndexBufferExport) -> bool {
    export.indices.len().is_multiple_of(3)
}

/// Convert index buffer metadata to JSON.
#[allow(dead_code)]
pub fn index_buffer_to_json(export: &IndexBufferExport) -> String {
    let fmt = match export.format {
        IndexFormat::U16 => "UINT16",
        IndexFormat::U32 => "UINT32",
    };
    format!(
        "{{\"index_count\":{},\"format\":\"{}\",\"triangle_count\":{}}}",
        export.indices.len(),
        fmt,
        triangle_count_export(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_index_buffer() {
        let i = vec![[0u32, 1, 2]];
        let exp = export_index_buffer(&i);
        assert_eq!(exp.indices.len(), 3);
    }

    #[test]
    fn test_format_u16() {
        let i = vec![[0u32, 1, 2]];
        let exp = export_index_buffer(&i);
        assert_eq!(exp.format, IndexFormat::U16);
    }

    #[test]
    fn test_format_u32() {
        let i = vec![[0u32, 1, 70000]];
        let exp = export_index_buffer(&i);
        assert_eq!(exp.format, IndexFormat::U32);
    }

    #[test]
    fn test_index_format_size() {
        assert_eq!(index_format_size(IndexFormat::U16), 2);
        assert_eq!(index_format_size(IndexFormat::U32), 4);
    }

    #[test]
    fn test_index_to_u16() {
        let result = index_to_u16(&[0, 1, 2]);
        assert_eq!(result, vec![0u16, 1, 2]);
    }

    #[test]
    fn test_index_to_u32() {
        let result = index_to_u32(&[0, 1, 2]);
        assert_eq!(result, vec![0u32, 1, 2]);
    }

    #[test]
    fn test_index_buffer_bytes_u16() {
        let exp = IndexBufferExport { indices: vec![0, 1, 2], format: IndexFormat::U16 };
        let bytes = index_buffer_bytes(&exp);
        assert_eq!(bytes.len(), 6);
    }

    #[test]
    fn test_triangle_count_export() {
        let i = vec![[0u32, 1, 2], [3, 4, 5]];
        let exp = export_index_buffer(&i);
        assert_eq!(triangle_count_export(&exp), 2);
    }

    #[test]
    fn test_validate_indices() {
        let exp = IndexBufferExport { indices: vec![0, 1, 2], format: IndexFormat::U16 };
        assert!(validate_indices(&exp));
    }

    #[test]
    fn test_index_buffer_to_json() {
        let i = vec![[0u32, 1, 2]];
        let exp = export_index_buffer(&i);
        let j = index_buffer_to_json(&exp);
        assert!(j.contains("index_count"));
    }
}
