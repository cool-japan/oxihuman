#![allow(dead_code)]

//! Vertex format descriptor export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexFormatExport {
    pub attributes: Vec<VertexAttribute>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub name: String,
    pub component_count: usize,
    pub byte_size: usize,
}

#[allow(dead_code)]
pub fn export_vertex_format(attributes: Vec<VertexAttribute>) -> VertexFormatExport {
    VertexFormatExport { attributes }
}

#[allow(dead_code)]
pub fn format_attribute_count(fmt: &VertexFormatExport) -> usize { fmt.attributes.len() }

#[allow(dead_code)]
pub fn format_stride_vfe(fmt: &VertexFormatExport) -> usize {
    fmt.attributes.iter().map(|a| a.byte_size).sum()
}

#[allow(dead_code)]
pub fn format_to_json(fmt: &VertexFormatExport) -> String {
    let attrs: Vec<String> = fmt.attributes.iter().map(|a|
        format!("{{\"name\":\"{}\",\"components\":{},\"bytes\":{}}}", a.name, a.component_count, a.byte_size)
    ).collect();
    format!("{{\"attribute_count\":{},\"stride\":{},\"attributes\":[{}]}}", fmt.attributes.len(), format_stride_vfe(fmt), attrs.join(","))
}

#[allow(dead_code)]
pub fn format_has_normals(fmt: &VertexFormatExport) -> bool {
    fmt.attributes.iter().any(|a| a.name.to_lowercase().contains("normal"))
}

#[allow(dead_code)]
pub fn format_has_uvs(fmt: &VertexFormatExport) -> bool {
    fmt.attributes.iter().any(|a| a.name.to_lowercase().contains("uv") || a.name.to_lowercase().contains("texcoord"))
}

#[allow(dead_code)]
pub fn format_export_size(fmt: &VertexFormatExport) -> usize { format_stride_vfe(fmt) }

#[allow(dead_code)]
pub fn validate_format(fmt: &VertexFormatExport) -> bool {
    !fmt.attributes.is_empty() && fmt.attributes.iter().all(|a| a.byte_size > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn attr(name: &str, c: usize, b: usize) -> VertexAttribute { VertexAttribute { name: name.into(), component_count: c, byte_size: b } }

    #[test]
    fn test_export() { let f = export_vertex_format(vec![attr("position",3,12)]); assert_eq!(format_attribute_count(&f), 1); }
    #[test]
    fn test_stride() { let f = export_vertex_format(vec![attr("pos",3,12),attr("nrm",3,12)]); assert_eq!(format_stride_vfe(&f), 24); }
    #[test]
    fn test_to_json() { let f = export_vertex_format(vec![attr("pos",3,12)]); assert!(format_to_json(&f).contains("\"attribute_count\":1")); }
    #[test]
    fn test_has_normals() { let f = export_vertex_format(vec![attr("normal",3,12)]); assert!(format_has_normals(&f)); }
    #[test]
    fn test_no_normals() { let f = export_vertex_format(vec![attr("position",3,12)]); assert!(!format_has_normals(&f)); }
    #[test]
    fn test_has_uvs() { let f = export_vertex_format(vec![attr("texcoord0",2,8)]); assert!(format_has_uvs(&f)); }
    #[test]
    fn test_no_uvs() { let f = export_vertex_format(vec![attr("position",3,12)]); assert!(!format_has_uvs(&f)); }
    #[test]
    fn test_validate() { assert!(validate_format(&export_vertex_format(vec![attr("pos",3,12)]))); }
    #[test]
    fn test_validate_empty() { assert!(!validate_format(&export_vertex_format(vec![]))); }
    #[test]
    fn test_export_size() { let f = export_vertex_format(vec![attr("pos",3,12)]); assert_eq!(format_export_size(&f), 12); }
}
