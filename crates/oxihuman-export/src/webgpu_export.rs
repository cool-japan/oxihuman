// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! WebGPU buffer layout export.

/// WebGPU vertex format type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WgpuVertexFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
    Sint32,
    Uint16x2,
    Uint16x4,
}

impl WgpuVertexFormat {
    pub fn byte_size(self) -> usize {
        match self {
            Self::Float32 | Self::Uint32 | Self::Sint32 => 4,
            Self::Float32x2 | Self::Uint16x4 => 8,
            Self::Float32x3 => 12,
            Self::Float32x4 => 16,
            Self::Uint16x2 => 4,
        }
    }

    pub fn wgsl_name(self) -> &'static str {
        match self {
            Self::Float32 => "f32",
            Self::Float32x2 => "vec2<f32>",
            Self::Float32x3 => "vec3<f32>",
            Self::Float32x4 => "vec4<f32>",
            Self::Uint32 => "u32",
            Self::Sint32 => "i32",
            Self::Uint16x2 => "vec2<u16>",
            Self::Uint16x4 => "vec4<u16>",
        }
    }
}

/// A single attribute in a WebGPU vertex buffer layout.
#[derive(Debug, Clone)]
pub struct WgpuVertexAttribute {
    pub format: WgpuVertexFormat,
    pub offset: u64,
    pub shader_location: u32,
    pub name: String,
}

impl WgpuVertexAttribute {
    pub fn new(name: &str, format: WgpuVertexFormat, offset: u64, location: u32) -> Self {
        Self { format, offset, shader_location: location, name: name.to_string() }
    }
}

/// A WebGPU vertex buffer layout descriptor.
#[derive(Debug, Clone, Default)]
pub struct WgpuBufferLayout {
    pub array_stride: u64,
    pub attributes: Vec<WgpuVertexAttribute>,
    pub step_mode: WgpuStepMode,
}

/// Step mode for vertex buffer.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum WgpuStepMode {
    #[default]
    Vertex,
    Instance,
}

impl WgpuBufferLayout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_attribute(&mut self, attr: WgpuVertexAttribute) {
        self.array_stride += attr.format.byte_size() as u64;
        self.attributes.push(attr);
    }
}

/// A WebGPU pipeline layout export.
#[derive(Debug, Clone, Default)]
pub struct WebGpuExport {
    pub vertex_layouts: Vec<WgpuBufferLayout>,
    pub wgsl_vertex_shader: String,
    pub wgsl_fragment_shader: String,
}

impl WebGpuExport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_vertex_layout(&mut self, layout: WgpuBufferLayout) {
        self.vertex_layouts.push(layout);
    }

    pub fn set_vertex_shader(&mut self, src: &str) {
        self.wgsl_vertex_shader = src.to_string();
    }
}

/// Serialize layout to a JSON descriptor.
pub fn to_webgpu_layout_json(d: &WebGpuExport) -> String {
    let layout_count = d.vertex_layouts.len();
    let attr_count: usize = d.vertex_layouts.iter().map(|l| l.attributes.len()).sum();
    format!(
        "{{\"vertex_layout_count\":{},\"total_attribute_count\":{}}}",
        layout_count, attr_count
    )
}

/// Compute total stride across all vertex layouts.
pub fn total_vertex_stride(d: &WebGpuExport) -> u64 {
    d.vertex_layouts.iter().map(|l| l.array_stride).sum()
}

/// Create a new WebGPU export.
pub fn new_webgpu_export() -> WebGpuExport {
    WebGpuExport::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float32x3_size() {
        assert_eq!(WgpuVertexFormat::Float32x3.byte_size(), 12);
    }

    #[test]
    fn test_float32_wgsl_name() {
        assert_eq!(WgpuVertexFormat::Float32.wgsl_name(), "f32");
    }

    #[test]
    fn test_buffer_layout_add_attribute() {
        let mut layout = WgpuBufferLayout::new();
        layout.add_attribute(WgpuVertexAttribute::new(
            "position",
            WgpuVertexFormat::Float32x3,
            0,
            0,
        ));
        assert_eq!(layout.attributes.len(), 1);
        assert_eq!(layout.array_stride, 12);
    }

    #[test]
    fn test_webgpu_export_add_layout() {
        let mut d = WebGpuExport::new();
        d.add_vertex_layout(WgpuBufferLayout::new());
        assert_eq!(d.vertex_layouts.len(), 1);
    }

    #[test]
    fn test_to_webgpu_layout_json() {
        let d = WebGpuExport::new();
        let s = to_webgpu_layout_json(&d);
        assert!(s.contains("vertex_layout_count"));
    }

    #[test]
    fn test_total_vertex_stride_empty() {
        let d = WebGpuExport::new();
        assert_eq!(total_vertex_stride(&d), 0);
    }

    #[test]
    fn test_total_vertex_stride_with_attributes() {
        let mut d = WebGpuExport::new();
        let mut layout = WgpuBufferLayout::new();
        layout.add_attribute(WgpuVertexAttribute::new("pos", WgpuVertexFormat::Float32x3, 0, 0));
        layout.add_attribute(WgpuVertexAttribute::new("uv", WgpuVertexFormat::Float32x2, 12, 1));
        d.add_vertex_layout(layout);
        assert_eq!(total_vertex_stride(&d), 20);
    }

    #[test]
    fn test_set_vertex_shader() {
        let mut d = WebGpuExport::new();
        d.set_vertex_shader("@vertex fn main() {}");
        assert!(d.wgsl_vertex_shader.contains("main"));
    }

    #[test]
    fn test_step_mode_default() {
        let layout = WgpuBufferLayout::new();
        assert_eq!(layout.step_mode, WgpuStepMode::Vertex);
    }

    #[test]
    fn test_new_webgpu_export() {
        let d = new_webgpu_export();
        assert!(d.vertex_layouts.is_empty());
    }
}
