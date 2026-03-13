// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! WebGL buffer/shader stub export.

/// WebGL buffer type.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WebGlBufferType {
    ArrayBuffer,
    ElementArrayBuffer,
}

/// A WebGL buffer export entry.
pub struct WebGlBuffer {
    pub buffer_type: WebGlBufferType,
    pub data: Vec<u8>,
    pub name: String,
}

/// WebGL export containing buffers and shader stubs.
pub struct WebGlExport {
    pub buffers: Vec<WebGlBuffer>,
    pub vertex_shader_src: String,
    pub fragment_shader_src: String,
}

/// Create a new WebGL export with minimal shaders.
pub fn new_webgl_export() -> WebGlExport {
    WebGlExport {
        buffers: Vec::new(),
        vertex_shader_src: "attribute vec3 aPos;\nvoid main(){gl_Position=vec4(aPos,1.0);}"
            .to_string(),
        fragment_shader_src: "void main(){gl_FragColor=vec4(1.0);}".to_string(),
    }
}

/// Add a buffer from f32 data.
pub fn add_webgl_f32_buffer(exp: &mut WebGlExport, name: &str, data: &[f32]) {
    let bytes: Vec<u8> = data.iter().flat_map(|f| f.to_le_bytes()).collect();
    exp.buffers.push(WebGlBuffer {
        buffer_type: WebGlBufferType::ArrayBuffer,
        data: bytes,
        name: name.to_string(),
    });
}

/// Add an index buffer from u16 data.
pub fn add_webgl_index_buffer(exp: &mut WebGlExport, name: &str, indices: &[u16]) {
    let bytes: Vec<u8> = indices.iter().flat_map(|i| i.to_le_bytes()).collect();
    exp.buffers.push(WebGlBuffer {
        buffer_type: WebGlBufferType::ElementArrayBuffer,
        data: bytes,
        name: name.to_string(),
    });
}

/// Buffer count.
pub fn webgl_buffer_count(exp: &WebGlExport) -> usize {
    exp.buffers.len()
}

/// Total byte size of all buffers.
pub fn webgl_total_bytes(exp: &WebGlExport) -> usize {
    exp.buffers.iter().map(|b| b.data.len()).sum()
}

/// Find a buffer by name.
pub fn find_webgl_buffer<'a>(exp: &'a WebGlExport, name: &str) -> Option<&'a WebGlBuffer> {
    exp.buffers.iter().find(|b| b.name == name)
}

/// Validate export (at least vertex shader non-empty).
pub fn validate_webgl_export(exp: &WebGlExport) -> bool {
    !exp.vertex_shader_src.is_empty() && !exp.fragment_shader_src.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_has_no_buffers() {
        let exp = new_webgl_export();
        assert_eq!(webgl_buffer_count(&exp), 0 /* no buffers */);
    }

    #[test]
    fn add_f32_buffer_increments_count() {
        let mut exp = new_webgl_export();
        add_webgl_f32_buffer(&mut exp, "pos", &[0.0, 1.0, 2.0]);
        assert_eq!(webgl_buffer_count(&exp), 1 /* one buffer */);
    }

    #[test]
    fn f32_buffer_correct_byte_size() {
        let mut exp = new_webgl_export();
        add_webgl_f32_buffer(&mut exp, "verts", &[0.0f32; 9]);
        let b = find_webgl_buffer(&exp, "verts").expect("should succeed");
        assert_eq!(b.data.len(), 36 /* 9 * 4 bytes */);
    }

    #[test]
    fn index_buffer_correct_byte_size() {
        let mut exp = new_webgl_export();
        add_webgl_index_buffer(&mut exp, "idx", &[0u16, 1, 2]);
        let b = find_webgl_buffer(&exp, "idx").expect("should succeed");
        assert_eq!(b.data.len(), 6 /* 3 * 2 bytes */);
    }

    #[test]
    fn total_bytes_sum() {
        let mut exp = new_webgl_export();
        add_webgl_f32_buffer(&mut exp, "a", &[0.0f32; 3]);
        add_webgl_index_buffer(&mut exp, "b", &[0u16; 3]);
        assert_eq!(webgl_total_bytes(&exp), 18 /* 12 + 6 */);
    }

    #[test]
    fn find_buffer_by_name() {
        let mut exp = new_webgl_export();
        add_webgl_f32_buffer(&mut exp, "normals", &[0.0, 1.0, 0.0]);
        assert!(find_webgl_buffer(&exp, "normals").is_some() /* found */);
    }

    #[test]
    fn find_missing_buffer_none() {
        let exp = new_webgl_export();
        assert!(find_webgl_buffer(&exp, "x").is_none() /* not found */);
    }

    #[test]
    fn validate_new_export() {
        let exp = new_webgl_export();
        assert!(validate_webgl_export(&exp) /* valid */);
    }

    #[test]
    fn index_buffer_type_correct() {
        let mut exp = new_webgl_export();
        add_webgl_index_buffer(&mut exp, "i", &[0u16]);
        let b = find_webgl_buffer(&exp, "i").expect("should succeed");
        assert_eq!(
            b.buffer_type,
            WebGlBufferType::ElementArrayBuffer /* element buffer */
        );
    }
}
