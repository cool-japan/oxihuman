#![allow(dead_code)]

//! Mesh attribute data export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshAttributeExport {
    pub attributes: Vec<MeshAttr>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshAttr {
    pub name: String,
    pub component_type: String,
    pub data: Vec<f32>,
}

#[allow(dead_code)]
pub fn export_mesh_attributes(attributes: Vec<MeshAttr>) -> MeshAttributeExport {
    MeshAttributeExport { attributes }
}

#[allow(dead_code)]
pub fn attribute_count_mae2(exp: &MeshAttributeExport) -> usize { exp.attributes.len() }

#[allow(dead_code)]
pub fn attribute_name_mae2(exp: &MeshAttributeExport, idx: usize) -> Option<&str> {
    exp.attributes.get(idx).map(|a| a.name.as_str())
}

#[allow(dead_code)]
pub fn attribute_to_json(exp: &MeshAttributeExport) -> String {
    let items: Vec<String> = exp.attributes.iter().map(|a|
        format!("{{\"name\":\"{}\",\"type\":\"{}\",\"count\":{}}}", a.name, a.component_type, a.data.len())
    ).collect();
    format!("{{\"attribute_count\":{},\"attributes\":[{}]}}", exp.attributes.len(), items.join(","))
}

#[allow(dead_code)]
pub fn attribute_component_type(exp: &MeshAttributeExport, idx: usize) -> Option<&str> {
    exp.attributes.get(idx).map(|a| a.component_type.as_str())
}

#[allow(dead_code)]
pub fn attribute_to_bytes_mae2(exp: &MeshAttributeExport) -> Vec<u8> {
    let mut bytes = Vec::new();
    for a in &exp.attributes { for &v in &a.data { bytes.extend_from_slice(&v.to_le_bytes()); } }
    bytes
}

#[allow(dead_code)]
pub fn attribute_export_size(exp: &MeshAttributeExport) -> usize {
    exp.attributes.iter().map(|a| a.data.len() * 4).sum()
}

#[allow(dead_code)]
pub fn validate_mesh_attributes(exp: &MeshAttributeExport) -> bool {
    exp.attributes.iter().all(|a| !a.name.is_empty() && !a.component_type.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn attr(n: &str) -> MeshAttr { MeshAttr { name: n.into(), component_type: "float".into(), data: vec![1.0, 2.0, 3.0] } }

    #[test]
    fn test_export() { let e = export_mesh_attributes(vec![attr("pos")]); assert_eq!(attribute_count_mae2(&e), 1); }
    #[test]
    fn test_name() { let e = export_mesh_attributes(vec![attr("normal")]); assert_eq!(attribute_name_mae2(&e, 0), Some("normal")); }
    #[test]
    fn test_name_none() { let e = export_mesh_attributes(vec![]); assert!(attribute_name_mae2(&e, 0).is_none()); }
    #[test]
    fn test_to_json() { let e = export_mesh_attributes(vec![attr("a")]); assert!(attribute_to_json(&e).contains("\"attribute_count\":1")); }
    #[test]
    fn test_component_type() { let e = export_mesh_attributes(vec![attr("a")]); assert_eq!(attribute_component_type(&e, 0), Some("float")); }
    #[test]
    fn test_to_bytes() { let e = export_mesh_attributes(vec![attr("a")]); assert_eq!(attribute_to_bytes_mae2(&e).len(), 12); }
    #[test]
    fn test_export_size() { let e = export_mesh_attributes(vec![attr("a")]); assert_eq!(attribute_export_size(&e), 12); }
    #[test]
    fn test_validate() { assert!(validate_mesh_attributes(&export_mesh_attributes(vec![attr("a")]))); }
    #[test]
    fn test_validate_empty_name() {
        let a = MeshAttr { name: "".into(), component_type: "float".into(), data: vec![] };
        assert!(!validate_mesh_attributes(&export_mesh_attributes(vec![a])));
    }
    #[test]
    fn test_multi() { let e = export_mesh_attributes(vec![attr("a"),attr("b")]); assert_eq!(attribute_count_mae2(&e), 2); }
}
