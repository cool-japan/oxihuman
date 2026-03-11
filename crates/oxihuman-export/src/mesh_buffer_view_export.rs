#![allow(dead_code)]
//! Mesh buffer view export.

/// Buffer view export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MeshBufferViewExport {
    pub views: Vec<BufferView>,
}

/// A single buffer view.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BufferView {
    pub offset: usize,
    pub length: usize,
    pub stride: Option<usize>,
    pub target: u32,
}

/// Export buffer views.
#[allow(dead_code)]
pub fn export_buffer_view(views: Vec<BufferView>) -> MeshBufferViewExport {
    MeshBufferViewExport { views }
}

/// Get view offset.
#[allow(dead_code)]
pub fn view_offset(e: &MeshBufferViewExport, index: usize) -> usize {
    if index < e.views.len() { e.views[index].offset } else { 0 }
}

/// Get view length.
#[allow(dead_code)]
pub fn view_length(e: &MeshBufferViewExport, index: usize) -> usize {
    if index < e.views.len() { e.views[index].length } else { 0 }
}

/// Get view stride.
#[allow(dead_code)]
pub fn view_stride_bve(e: &MeshBufferViewExport, index: usize) -> Option<usize> {
    if index < e.views.len() { e.views[index].stride } else { None }
}

/// Get view target (34962 = array buffer, 34963 = element array buffer).
#[allow(dead_code)]
pub fn view_target(e: &MeshBufferViewExport, index: usize) -> u32 {
    if index < e.views.len() { e.views[index].target } else { 0 }
}

/// Serialize view to JSON.
#[allow(dead_code)]
pub fn view_to_json(e: &MeshBufferViewExport, index: usize) -> String {
    if index < e.views.len() {
        let v = &e.views[index];
        format!(
            "{{\"offset\":{},\"length\":{},\"target\":{}}}",
            v.offset, v.length, v.target
        )
    } else {
        "{}".to_string()
    }
}

/// Get buffer view count.
#[allow(dead_code)]
pub fn buffer_view_count(e: &MeshBufferViewExport) -> usize {
    e.views.len()
}

/// Validate buffer views.
#[allow(dead_code)]
pub fn validate_buffer_view(e: &MeshBufferViewExport) -> bool {
    e.views.iter().all(|v| v.length > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bv(offset: usize, length: usize, target: u32) -> BufferView {
        BufferView { offset, length, stride: None, target }
    }

    #[test]
    fn test_export_buffer_view() {
        let e = export_buffer_view(vec![bv(0, 100, 34962)]);
        assert_eq!(e.views.len(), 1);
    }

    #[test]
    fn test_view_offset() {
        let e = export_buffer_view(vec![bv(64, 100, 34962)]);
        assert_eq!(view_offset(&e, 0), 64);
        assert_eq!(view_offset(&e, 5), 0);
    }

    #[test]
    fn test_view_length() {
        let e = export_buffer_view(vec![bv(0, 200, 34962)]);
        assert_eq!(view_length(&e, 0), 200);
    }

    #[test]
    fn test_view_stride() {
        let e = export_buffer_view(vec![BufferView { offset: 0, length: 100, stride: Some(12), target: 34962 }]);
        assert_eq!(view_stride_bve(&e, 0), Some(12));
        assert_eq!(view_stride_bve(&e, 5), None);
    }

    #[test]
    fn test_view_target() {
        let e = export_buffer_view(vec![bv(0, 100, 34963)]);
        assert_eq!(view_target(&e, 0), 34963);
    }

    #[test]
    fn test_view_to_json() {
        let e = export_buffer_view(vec![bv(0, 100, 34962)]);
        let j = view_to_json(&e, 0);
        assert!(j.contains("offset"));
    }

    #[test]
    fn test_view_to_json_oob() {
        let e = export_buffer_view(vec![]);
        assert_eq!(view_to_json(&e, 0), "{}");
    }

    #[test]
    fn test_buffer_view_count() {
        let e = export_buffer_view(vec![bv(0, 100, 0), bv(100, 200, 0)]);
        assert_eq!(buffer_view_count(&e), 2);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_buffer_view(vec![bv(0, 100, 34962)]);
        assert!(validate_buffer_view(&e));
    }

    #[test]
    fn test_validate_zero_length() {
        let e = export_buffer_view(vec![bv(0, 0, 34962)]);
        assert!(!validate_buffer_view(&e));
    }
}
