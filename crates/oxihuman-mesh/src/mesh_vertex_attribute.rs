#![allow(dead_code)]
//! Per-vertex attribute storage.

/// Attribute data type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Int,
}

/// A named per-vertex attribute.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VertexAttribute {
    pub name: String,
    pub attr_type: AttributeType,
    pub data: Vec<[f32; 4]>,
}

/// Create a new vertex attribute.
#[allow(dead_code)]
pub fn new_vertex_attribute(name: &str, attr_type: AttributeType, count: usize) -> VertexAttribute {
    VertexAttribute {
        name: name.to_string(),
        attr_type,
        data: vec![[0.0; 4]; count],
    }
}

/// Set attribute value at a vertex.
#[allow(dead_code)]
pub fn set_attribute(attr: &mut VertexAttribute, index: usize, value: [f32; 4]) {
    if index < attr.data.len() {
        attr.data[index] = value;
    }
}

/// Get attribute value at a vertex.
#[allow(dead_code)]
pub fn get_attribute(attr: &VertexAttribute, index: usize) -> [f32; 4] {
    if index < attr.data.len() {
        attr.data[index]
    } else {
        [0.0; 4]
    }
}

/// Return the number of attribute entries.
#[allow(dead_code)]
pub fn attribute_count_va(attr: &VertexAttribute) -> usize {
    attr.data.len()
}

/// Return the attribute type.
#[allow(dead_code)]
pub fn attribute_type(attr: &VertexAttribute) -> AttributeType {
    attr.attr_type
}

/// Serialize attribute data to bytes.
#[allow(dead_code)]
pub fn attribute_to_bytes(attr: &VertexAttribute) -> Vec<u8> {
    let mut buf = Vec::new();
    for v in &attr.data {
        for &f in v {
            buf.extend_from_slice(&f.to_le_bytes());
        }
    }
    buf
}

/// Return the attribute name.
#[allow(dead_code)]
pub fn attribute_name_va(attr: &VertexAttribute) -> &str {
    &attr.name
}

/// Clear all attribute values to zero.
#[allow(dead_code)]
pub fn clear_attribute(attr: &mut VertexAttribute) {
    for v in &mut attr.data {
        *v = [0.0; 4];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vertex_attribute() {
        let a = new_vertex_attribute("color", AttributeType::Vec3, 4);
        assert_eq!(attribute_count_va(&a), 4);
    }

    #[test]
    fn test_set_get_attribute() {
        let mut a = new_vertex_attribute("uv", AttributeType::Vec2, 2);
        set_attribute(&mut a, 0, [0.5, 0.5, 0.0, 0.0]);
        assert_eq!(get_attribute(&a, 0), [0.5, 0.5, 0.0, 0.0]);
    }

    #[test]
    fn test_get_attribute_oob() {
        let a = new_vertex_attribute("t", AttributeType::Float, 1);
        assert_eq!(get_attribute(&a, 10), [0.0; 4]);
    }

    #[test]
    fn test_attribute_type() {
        let a = new_vertex_attribute("n", AttributeType::Vec3, 1);
        assert_eq!(attribute_type(&a), AttributeType::Vec3);
    }

    #[test]
    fn test_attribute_to_bytes() {
        let a = new_vertex_attribute("x", AttributeType::Float, 1);
        let bytes = attribute_to_bytes(&a);
        assert_eq!(bytes.len(), 16); // 4 floats * 4 bytes
    }

    #[test]
    fn test_attribute_name() {
        let a = new_vertex_attribute("pos", AttributeType::Vec3, 1);
        assert_eq!(attribute_name_va(&a), "pos");
    }

    #[test]
    fn test_clear_attribute() {
        let mut a = new_vertex_attribute("c", AttributeType::Vec4, 2);
        set_attribute(&mut a, 0, [1.0, 2.0, 3.0, 4.0]);
        clear_attribute(&mut a);
        assert_eq!(get_attribute(&a, 0), [0.0; 4]);
    }

    #[test]
    fn test_set_attribute_oob() {
        let mut a = new_vertex_attribute("x", AttributeType::Float, 1);
        set_attribute(&mut a, 100, [1.0; 4]); // should not panic
        assert_eq!(attribute_count_va(&a), 1);
    }

    #[test]
    fn test_empty_attribute() {
        let a = new_vertex_attribute("e", AttributeType::Int, 0);
        assert_eq!(attribute_count_va(&a), 0);
        assert!(attribute_to_bytes(&a).is_empty());
    }

    #[test]
    fn test_attribute_type_int() {
        let a = new_vertex_attribute("i", AttributeType::Int, 1);
        assert_eq!(attribute_type(&a), AttributeType::Int);
    }
}
