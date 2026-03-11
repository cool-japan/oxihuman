#![allow(dead_code)]
//! Draco mesh compression export stub.

/// Draco export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MeshDracoExport {
    pub original_size: usize,
    pub compressed_size: usize,
    pub method: String,
}

/// Export draco stub (no actual compression).
#[allow(dead_code)]
pub fn export_draco_stub(original_size: usize, method: &str) -> MeshDracoExport {
    // Simulate ~60% compression
    let compressed = original_size * 4 / 10;
    MeshDracoExport {
        original_size,
        compressed_size: compressed,
        method: method.to_string(),
    }
}

/// Get compressed size.
#[allow(dead_code)]
pub fn draco_compressed_size(e: &MeshDracoExport) -> usize {
    e.compressed_size
}

/// Get original size.
#[allow(dead_code)]
pub fn draco_original_size(e: &MeshDracoExport) -> usize {
    e.original_size
}

/// Get compression ratio.
#[allow(dead_code)]
pub fn draco_ratio(e: &MeshDracoExport) -> f32 {
    if e.original_size == 0 {
        return 0.0;
    }
    e.compressed_size as f32 / e.original_size as f32
}

/// Get as bytes (stub returns empty).
#[allow(dead_code)]
pub fn draco_to_bytes(e: &MeshDracoExport) -> Vec<u8> {
    vec![0u8; e.compressed_size]
}

/// Get compression method.
#[allow(dead_code)]
pub fn draco_method(e: &MeshDracoExport) -> &str {
    &e.method
}

/// Get export size.
#[allow(dead_code)]
pub fn draco_export_size(e: &MeshDracoExport) -> usize {
    e.compressed_size
}

/// Validate draco export.
#[allow(dead_code)]
pub fn validate_draco(e: &MeshDracoExport) -> bool {
    e.compressed_size <= e.original_size && !e.method.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_draco_stub() {
        let e = export_draco_stub(1000, "edgebreaker");
        assert!(e.compressed_size < e.original_size);
    }

    #[test]
    fn test_draco_compressed_size() {
        let e = export_draco_stub(1000, "edgebreaker");
        assert_eq!(draco_compressed_size(&e), 400);
    }

    #[test]
    fn test_draco_original_size() {
        let e = export_draco_stub(500, "sequential");
        assert_eq!(draco_original_size(&e), 500);
    }

    #[test]
    fn test_draco_ratio() {
        let e = export_draco_stub(1000, "edgebreaker");
        assert!((draco_ratio(&e) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_draco_ratio_zero() {
        let e = export_draco_stub(0, "none");
        assert!((draco_ratio(&e)).abs() < 1e-6);
    }

    #[test]
    fn test_draco_to_bytes() {
        let e = export_draco_stub(100, "edgebreaker");
        assert_eq!(draco_to_bytes(&e).len(), e.compressed_size);
    }

    #[test]
    fn test_draco_method() {
        let e = export_draco_stub(100, "edgebreaker");
        assert_eq!(draco_method(&e), "edgebreaker");
    }

    #[test]
    fn test_draco_export_size() {
        let e = export_draco_stub(100, "edgebreaker");
        assert_eq!(draco_export_size(&e), e.compressed_size);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_draco_stub(100, "edgebreaker");
        assert!(validate_draco(&e));
    }

    #[test]
    fn test_validate_empty_method() {
        let e = export_draco_stub(100, "");
        assert!(!validate_draco(&e));
    }
}
