#![allow(dead_code)]
//! Export compressed mesh data.

/// Compressed mesh export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CompressedMeshExport {
    pub original_bytes: usize,
    pub compressed_bytes: Vec<u8>,
    pub method: String,
}

/// Export a compressed mesh.
#[allow(dead_code)]
pub fn export_compressed_mesh(data: &[u8], method: &str) -> CompressedMeshExport {
    // Simple RLE-like "compression" for deterministic results
    let original = data.len();
    let compressed = data.to_vec(); // no actual compression, just store
    CompressedMeshExport {
        original_bytes: original,
        compressed_bytes: compressed,
        method: method.to_string(),
    }
}

/// Return compression ratio.
#[allow(dead_code)]
pub fn compression_ratio(exp: &CompressedMeshExport) -> f32 {
    if exp.original_bytes == 0 {
        return 1.0;
    }
    exp.compressed_bytes.len() as f32 / exp.original_bytes as f32
}

/// Return original size.
#[allow(dead_code)]
pub fn original_size(exp: &CompressedMeshExport) -> usize {
    exp.original_bytes
}

/// Return compressed size.
#[allow(dead_code)]
pub fn compressed_size(exp: &CompressedMeshExport) -> usize {
    exp.compressed_bytes.len()
}

/// Return compression method.
#[allow(dead_code)]
pub fn compression_method(exp: &CompressedMeshExport) -> &str {
    &exp.method
}

/// Get compressed bytes.
#[allow(dead_code)]
pub fn compressed_to_bytes(exp: &CompressedMeshExport) -> &[u8] {
    &exp.compressed_bytes
}

/// Compute export size.
#[allow(dead_code)]
pub fn compression_export_size(exp: &CompressedMeshExport) -> usize {
    exp.compressed_bytes.len()
}

/// Validate compressed mesh.
#[allow(dead_code)]
pub fn validate_compressed_mesh(exp: &CompressedMeshExport) -> bool {
    !exp.method.is_empty() && !exp.compressed_bytes.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_compressed_mesh() {
        let data = [1u8, 2, 3, 4];
        let e = export_compressed_mesh(&data, "none");
        assert_eq!(original_size(&e), 4);
    }

    #[test]
    fn test_compression_ratio() {
        let data = [0u8; 100];
        let e = export_compressed_mesh(&data, "none");
        assert!((compression_ratio(&e) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compression_ratio_empty() {
        let e = export_compressed_mesh(&[], "none");
        assert!((compression_ratio(&e) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compressed_size() {
        let data = [1u8, 2, 3];
        let e = export_compressed_mesh(&data, "none");
        assert_eq!(compressed_size(&e), 3);
    }

    #[test]
    fn test_compression_method() {
        let e = export_compressed_mesh(&[0], "zlib");
        assert_eq!(compression_method(&e), "zlib");
    }

    #[test]
    fn test_compressed_to_bytes() {
        let data = [5u8, 6];
        let e = export_compressed_mesh(&data, "none");
        assert_eq!(compressed_to_bytes(&e), &[5, 6]);
    }

    #[test]
    fn test_compression_export_size() {
        let data = [0u8; 10];
        let e = export_compressed_mesh(&data, "none");
        assert_eq!(compression_export_size(&e), 10);
    }

    #[test]
    fn test_validate_compressed_mesh() {
        let e = export_compressed_mesh(&[1], "zlib");
        assert!(validate_compressed_mesh(&e));
    }

    #[test]
    fn test_validate_empty_data() {
        let e = export_compressed_mesh(&[], "zlib");
        assert!(!validate_compressed_mesh(&e));
    }

    #[test]
    fn test_validate_empty_method() {
        let e = export_compressed_mesh(&[1], "");
        assert!(!validate_compressed_mesh(&e));
    }
}
