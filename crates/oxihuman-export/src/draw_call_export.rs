#![allow(dead_code)]
//! Export draw call data.

/// Draw call export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct DrawCallExport {
    pub calls: Vec<DrawCall>,
}

/// A single draw call.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct DrawCall {
    pub material: String,
    pub index_start: u32,
    pub index_count: u32,
    pub vertex_start: u32,
    pub vertex_count: u32,
}

/// Export draw calls.
#[allow(dead_code)]
pub fn export_draw_calls(calls: Vec<DrawCall>) -> DrawCallExport {
    DrawCallExport { calls }
}

/// Return draw call count.
#[allow(dead_code)]
pub fn draw_call_count_export(exp: &DrawCallExport) -> usize {
    exp.calls.len()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn draw_call_to_json(exp: &DrawCallExport) -> String {
    let items: Vec<String> = exp
        .calls
        .iter()
        .map(|c| {
            format!(
                "{{\"material\":\"{}\",\"indices\":{},\"vertices\":{}}}",
                c.material, c.index_count, c.vertex_count
            )
        })
        .collect();
    format!("{{\"draw_calls\":[{}]}}", items.join(","))
}

/// Return material for a draw call.
#[allow(dead_code)]
pub fn draw_call_material(exp: &DrawCallExport, index: usize) -> &str {
    if index < exp.calls.len() {
        &exp.calls[index].material
    } else {
        ""
    }
}

/// Return index range for a draw call.
#[allow(dead_code)]
pub fn draw_call_index_range(exp: &DrawCallExport, index: usize) -> (u32, u32) {
    if index < exp.calls.len() {
        let c = &exp.calls[index];
        (c.index_start, c.index_start + c.index_count)
    } else {
        (0, 0)
    }
}

/// Return vertex range for a draw call.
#[allow(dead_code)]
pub fn draw_call_vertex_range(exp: &DrawCallExport, index: usize) -> (u32, u32) {
    if index < exp.calls.len() {
        let c = &exp.calls[index];
        (c.vertex_start, c.vertex_start + c.vertex_count)
    } else {
        (0, 0)
    }
}

/// Compute export size.
#[allow(dead_code)]
pub fn draw_call_export_size(exp: &DrawCallExport) -> usize {
    exp.calls.len() * 20
}

/// Validate draw calls.
#[allow(dead_code)]
pub fn validate_draw_calls(exp: &DrawCallExport) -> bool {
    !exp.calls.is_empty() && exp.calls.iter().all(|c| !c.material.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_call() -> DrawCall {
        DrawCall {
            material: "default".to_string(),
            index_start: 0,
            index_count: 300,
            vertex_start: 0,
            vertex_count: 100,
        }
    }

    #[test]
    fn test_export_draw_calls() {
        let e = export_draw_calls(vec![sample_call()]);
        assert_eq!(draw_call_count_export(&e), 1);
    }

    #[test]
    fn test_draw_call_to_json() {
        let e = export_draw_calls(vec![sample_call()]);
        let j = draw_call_to_json(&e);
        assert!(j.contains("\"draw_calls\""));
    }

    #[test]
    fn test_draw_call_material() {
        let e = export_draw_calls(vec![sample_call()]);
        assert_eq!(draw_call_material(&e, 0), "default");
    }

    #[test]
    fn test_draw_call_material_oob() {
        let e = export_draw_calls(vec![]);
        assert_eq!(draw_call_material(&e, 0), "");
    }

    #[test]
    fn test_draw_call_index_range() {
        let e = export_draw_calls(vec![sample_call()]);
        assert_eq!(draw_call_index_range(&e, 0), (0, 300));
    }

    #[test]
    fn test_draw_call_vertex_range() {
        let e = export_draw_calls(vec![sample_call()]);
        assert_eq!(draw_call_vertex_range(&e, 0), (0, 100));
    }

    #[test]
    fn test_draw_call_export_size() {
        let e = export_draw_calls(vec![sample_call(), sample_call()]);
        assert_eq!(draw_call_export_size(&e), 40);
    }

    #[test]
    fn test_validate_draw_calls() {
        let e = export_draw_calls(vec![sample_call()]);
        assert!(validate_draw_calls(&e));
    }

    #[test]
    fn test_validate_empty() {
        let e = export_draw_calls(vec![]);
        assert!(!validate_draw_calls(&e));
    }

    #[test]
    fn test_index_range_oob() {
        let e = export_draw_calls(vec![]);
        assert_eq!(draw_call_index_range(&e, 0), (0, 0));
    }
}
