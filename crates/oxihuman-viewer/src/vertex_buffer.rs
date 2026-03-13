// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Description of a single vertex attribute.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexAttrDesc {
    pub name: String,
    pub components: u8,
    pub stride: u32,
    pub offset: u32,
    pub normalized: bool,
}

/// Descriptor for a vertex buffer with its layout.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexBufferDesc {
    pub attrs: Vec<VertexAttrDesc>,
    pub vertex_count: u32,
    pub buffer_id: u32,
}

/// Create a new vertex buffer descriptor with the given GPU buffer ID.
#[allow(dead_code)]
pub fn new_vertex_buffer_desc(buffer_id: u32) -> VertexBufferDesc {
    VertexBufferDesc { attrs: Vec::new(), vertex_count: 0, buffer_id }
}

/// Append a vertex attribute to the descriptor.
#[allow(dead_code)]
pub fn add_attr(vbd: &mut VertexBufferDesc, name: &str, components: u8, stride: u32, offset: u32) {
    vbd.attrs.push(VertexAttrDesc {
        name: name.to_string(),
        components,
        stride,
        offset,
        normalized: false,
    });
}

/// Set the vertex count.
#[allow(dead_code)]
pub fn set_vertex_count(vbd: &mut VertexBufferDesc, n: u32) {
    vbd.vertex_count = n;
}

/// Return the number of attributes in the descriptor.
#[allow(dead_code)]
pub fn attr_count(vbd: &VertexBufferDesc) -> usize {
    vbd.attrs.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_desc_no_attrs() {
        let vbd = new_vertex_buffer_desc(1);
        assert_eq!(attr_count(&vbd), 0);
    }

    #[test]
    fn add_attr_increments() {
        let mut vbd = new_vertex_buffer_desc(1);
        add_attr(&mut vbd, "position", 3, 32, 0);
        assert_eq!(attr_count(&vbd), 1);
    }

    #[test]
    fn add_multiple_attrs() {
        let mut vbd = new_vertex_buffer_desc(1);
        add_attr(&mut vbd, "pos", 3, 32, 0);
        add_attr(&mut vbd, "normal", 3, 32, 12);
        add_attr(&mut vbd, "uv", 2, 32, 24);
        assert_eq!(attr_count(&vbd), 3);
    }

    #[test]
    fn attr_name_stored() {
        let mut vbd = new_vertex_buffer_desc(1);
        add_attr(&mut vbd, "texcoord", 2, 16, 0);
        assert_eq!(vbd.attrs[0].name, "texcoord");
    }

    #[test]
    fn attr_components_stored() {
        let mut vbd = new_vertex_buffer_desc(1);
        add_attr(&mut vbd, "col", 4, 16, 0);
        assert_eq!(vbd.attrs[0].components, 4);
    }

    #[test]
    fn set_vertex_count_updates() {
        let mut vbd = new_vertex_buffer_desc(1);
        set_vertex_count(&mut vbd, 512);
        assert_eq!(vbd.vertex_count, 512);
    }

    #[test]
    fn buffer_id_stored() {
        let vbd = new_vertex_buffer_desc(77);
        assert_eq!(vbd.buffer_id, 77);
    }

    #[test]
    fn attr_stride_stored() {
        let mut vbd = new_vertex_buffer_desc(1);
        add_attr(&mut vbd, "pos", 3, 48, 0);
        assert_eq!(vbd.attrs[0].stride, 48);
    }

    #[test]
    fn attr_offset_stored() {
        let mut vbd = new_vertex_buffer_desc(1);
        add_attr(&mut vbd, "nrm", 3, 32, 12);
        assert_eq!(vbd.attrs[0].offset, 12);
    }

    #[test]
    fn vertex_count_defaults_zero() {
        let vbd = new_vertex_buffer_desc(1);
        assert_eq!(vbd.vertex_count, 0);
    }
}
