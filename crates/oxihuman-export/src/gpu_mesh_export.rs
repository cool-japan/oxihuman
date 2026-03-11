#![allow(dead_code)]
//! Export GPU-ready mesh data.

/// GPU mesh export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct GpuMeshExport {
    pub vertex_buffer: Vec<u8>,
    pub index_buffer: Vec<u8>,
    pub format: String,
}

/// Export GPU mesh.
#[allow(dead_code)]
pub fn export_gpu_mesh(
    vertex_buffer: &[u8],
    index_buffer: &[u8],
    format: &str,
) -> GpuMeshExport {
    GpuMeshExport {
        vertex_buffer: vertex_buffer.to_vec(),
        index_buffer: index_buffer.to_vec(),
        format: format.to_string(),
    }
}

/// Return vertex buffer size.
#[allow(dead_code)]
pub fn gpu_vertex_buffer_size(exp: &GpuMeshExport) -> usize {
    exp.vertex_buffer.len()
}

/// Return index buffer size.
#[allow(dead_code)]
pub fn gpu_index_buffer_size(exp: &GpuMeshExport) -> usize {
    exp.index_buffer.len()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn gpu_mesh_to_json(exp: &GpuMeshExport) -> String {
    format!(
        "{{\"format\":\"{}\",\"vertex_buffer_size\":{},\"index_buffer_size\":{}}}",
        exp.format,
        exp.vertex_buffer.len(),
        exp.index_buffer.len()
    )
}

/// Return format.
#[allow(dead_code)]
pub fn gpu_mesh_format(exp: &GpuMeshExport) -> &str {
    &exp.format
}

/// Return combined bytes.
#[allow(dead_code)]
pub fn gpu_mesh_to_bytes(exp: &GpuMeshExport) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&exp.vertex_buffer);
    buf.extend_from_slice(&exp.index_buffer);
    buf
}

/// Compute export size.
#[allow(dead_code)]
pub fn gpu_mesh_export_size(exp: &GpuMeshExport) -> usize {
    exp.vertex_buffer.len() + exp.index_buffer.len()
}

/// Validate GPU mesh.
#[allow(dead_code)]
pub fn validate_gpu_mesh(exp: &GpuMeshExport) -> bool {
    !exp.format.is_empty() && !exp.vertex_buffer.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_gpu_mesh() {
        let e = export_gpu_mesh(&[0u8; 48], &[0u8; 12], "float32");
        assert_eq!(gpu_vertex_buffer_size(&e), 48);
    }

    #[test]
    fn test_gpu_index_buffer_size() {
        let e = export_gpu_mesh(&[0u8; 10], &[0u8; 6], "uint16");
        assert_eq!(gpu_index_buffer_size(&e), 6);
    }

    #[test]
    fn test_gpu_mesh_to_json() {
        let e = export_gpu_mesh(&[0], &[0], "f32");
        let j = gpu_mesh_to_json(&e);
        assert!(j.contains("\"format\":\"f32\""));
    }

    #[test]
    fn test_gpu_mesh_format() {
        let e = export_gpu_mesh(&[0], &[], "f16");
        assert_eq!(gpu_mesh_format(&e), "f16");
    }

    #[test]
    fn test_gpu_mesh_to_bytes() {
        let e = export_gpu_mesh(&[1, 2], &[3, 4], "f32");
        assert_eq!(gpu_mesh_to_bytes(&e), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_gpu_mesh_export_size() {
        let e = export_gpu_mesh(&[0u8; 100], &[0u8; 50], "f32");
        assert_eq!(gpu_mesh_export_size(&e), 150);
    }

    #[test]
    fn test_validate_gpu_mesh() {
        let e = export_gpu_mesh(&[0], &[], "f32");
        assert!(validate_gpu_mesh(&e));
    }

    #[test]
    fn test_validate_empty_vb() {
        let e = export_gpu_mesh(&[], &[0], "f32");
        assert!(!validate_gpu_mesh(&e));
    }

    #[test]
    fn test_validate_empty_format() {
        let e = export_gpu_mesh(&[0], &[], "");
        assert!(!validate_gpu_mesh(&e));
    }

    #[test]
    fn test_empty_buffers() {
        let e = export_gpu_mesh(&[], &[], "f32");
        assert_eq!(gpu_mesh_export_size(&e), 0);
    }
}
