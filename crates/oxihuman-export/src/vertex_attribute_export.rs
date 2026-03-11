#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A single vertex attribute layer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub name: String,
    pub type_name: String,
    pub components: usize,
    pub data: Vec<f32>,
}

/// Container for multiple vertex attributes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexAttributeExport {
    pub attributes: Vec<VertexAttribute>,
}

/// Build a vertex attribute export.
#[allow(dead_code)]
pub fn export_vertex_attributes(attrs: Vec<VertexAttribute>) -> VertexAttributeExport {
    VertexAttributeExport { attributes: attrs }
}

/// Return the number of attribute layers.
#[allow(dead_code)]
pub fn attribute_count(exp: &VertexAttributeExport) -> usize {
    exp.attributes.len()
}

/// Return the name of an attribute.
#[allow(dead_code)]
pub fn attribute_name(exp: &VertexAttributeExport, index: usize) -> Option<&str> {
    exp.attributes.get(index).map(|a| a.name.as_str())
}

/// Return the number of components per vertex for an attribute.
#[allow(dead_code)]
pub fn attribute_component_count(exp: &VertexAttributeExport, index: usize) -> usize {
    exp.attributes.get(index).map_or(0, |a| a.components)
}

/// Serialize an attribute to bytes (component count u32 LE, then float data).
#[allow(dead_code)]
pub fn attribute_to_bytes(exp: &VertexAttributeExport, index: usize) -> Vec<u8> {
    let attr = match exp.attributes.get(index) {
        Some(a) => a,
        None => return Vec::new(),
    };
    let mut buf = Vec::new();
    buf.extend_from_slice(&(attr.components as u32).to_le_bytes());
    buf.extend_from_slice(&(attr.data.len() as u32).to_le_bytes());
    for &v in &attr.data {
        buf.extend_from_slice(&v.to_le_bytes());
    }
    buf
}

/// Return the type name of an attribute.
#[allow(dead_code)]
pub fn attribute_type_name(exp: &VertexAttributeExport, index: usize) -> Option<&str> {
    exp.attributes.get(index).map(|a| a.type_name.as_str())
}

/// Return total byte size of all attributes.
#[allow(dead_code)]
pub fn attribute_export_size(exp: &VertexAttributeExport) -> usize {
    exp.attributes.iter().map(|a| 8 + a.data.len() * 4).sum()
}

/// Validate: all attributes have non-empty name and data length divisible by components.
#[allow(dead_code)]
pub fn validate_attribute_export(exp: &VertexAttributeExport) -> bool {
    exp.attributes.iter().all(|a| {
        !a.name.is_empty() && a.components > 0 && a.data.len() % a.components == 0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> VertexAttributeExport {
        export_vertex_attributes(vec![
            VertexAttribute {
                name: "position".to_string(),
                type_name: "float3".to_string(),
                components: 3,
                data: vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
            },
            VertexAttribute {
                name: "uv".to_string(),
                type_name: "float2".to_string(),
                components: 2,
                data: vec![0.0, 0.0, 1.0, 1.0],
            },
        ])
    }

    #[test]
    fn test_attribute_count() {
        assert_eq!(attribute_count(&sample()), 2);
    }

    #[test]
    fn test_attribute_name() {
        assert_eq!(attribute_name(&sample(), 0), Some("position"));
        assert_eq!(attribute_name(&sample(), 5), None);
    }

    #[test]
    fn test_component_count() {
        assert_eq!(attribute_component_count(&sample(), 0), 3);
        assert_eq!(attribute_component_count(&sample(), 1), 2);
    }

    #[test]
    fn test_to_bytes() {
        let b = attribute_to_bytes(&sample(), 0);
        assert!(!b.is_empty());
        let comp = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        assert_eq!(comp, 3);
    }

    #[test]
    fn test_to_bytes_oob() {
        assert!(attribute_to_bytes(&sample(), 99).is_empty());
    }

    #[test]
    fn test_type_name() {
        assert_eq!(attribute_type_name(&sample(), 0), Some("float3"));
    }

    #[test]
    fn test_export_size() {
        assert!(attribute_export_size(&sample()) > 0);
    }

    #[test]
    fn test_validate_ok() {
        assert!(validate_attribute_export(&sample()));
    }

    #[test]
    fn test_validate_bad_name() {
        let e = export_vertex_attributes(vec![VertexAttribute {
            name: String::new(),
            type_name: "f".to_string(),
            components: 1,
            data: vec![1.0],
        }]);
        assert!(!validate_attribute_export(&e));
    }

    #[test]
    fn test_validate_bad_components() {
        let e = export_vertex_attributes(vec![VertexAttribute {
            name: "x".to_string(),
            type_name: "f".to_string(),
            components: 3,
            data: vec![1.0, 2.0], // not divisible by 3
        }]);
        assert!(!validate_attribute_export(&e));
    }
}
