// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Metal shader stub export for Apple platforms.

/// Metal pixel format.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetalPixelFormat {
    Rgba8Unorm,
    Bgra8Unorm,
    Depth32Float,
    Rgba16Float,
    R32Float,
}

impl MetalPixelFormat {
    pub fn name(self) -> &'static str {
        match self {
            Self::Rgba8Unorm => "MTLPixelFormatRGBA8Unorm",
            Self::Bgra8Unorm => "MTLPixelFormatBGRA8Unorm",
            Self::Depth32Float => "MTLPixelFormatDepth32Float",
            Self::Rgba16Float => "MTLPixelFormatRGBA16Float",
            Self::R32Float => "MTLPixelFormatR32Float",
        }
    }

    pub fn bytes_per_pixel(self) -> u32 {
        match self {
            Self::R32Float => 4,
            Self::Rgba8Unorm | Self::Bgra8Unorm | Self::Depth32Float => 4,
            Self::Rgba16Float => 8,
        }
    }
}

/// Metal vertex attribute descriptor.
#[derive(Debug, Clone)]
pub struct MetalVertexAttribute {
    pub name: String,
    pub index: u32,
    pub format: MetalVertexAttributeFormat,
    pub offset: u32,
    pub buffer_index: u32,
}

/// Metal vertex attribute format.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetalVertexAttributeFormat {
    Float,
    Float2,
    Float3,
    Float4,
    Half2,
    Half4,
    UChar4,
}

impl MetalVertexAttributeFormat {
    pub fn name(self) -> &'static str {
        match self {
            Self::Float => "MTLVertexFormatFloat",
            Self::Float2 => "MTLVertexFormatFloat2",
            Self::Float3 => "MTLVertexFormatFloat3",
            Self::Float4 => "MTLVertexFormatFloat4",
            Self::Half2 => "MTLVertexFormatHalf2",
            Self::Half4 => "MTLVertexFormatHalf4",
            Self::UChar4 => "MTLVertexFormatUChar4",
        }
    }
}

impl MetalVertexAttribute {
    pub fn new(name: &str, index: u32, format: MetalVertexAttributeFormat, offset: u32) -> Self {
        Self { name: name.to_string(), index, format, offset, buffer_index: 0 }
    }
}

/// Metal render pipeline descriptor export.
#[derive(Debug, Clone, Default)]
pub struct MetalExport {
    pub vertex_function: String,
    pub fragment_function: String,
    pub color_attachments: Vec<MetalPixelFormat>,
    pub depth_format: Option<MetalPixelFormat>,
    pub vertex_attributes: Vec<MetalVertexAttribute>,
}

impl MetalExport {
    pub fn new(vertex_fn: &str, fragment_fn: &str) -> Self {
        Self {
            vertex_function: vertex_fn.to_string(),
            fragment_function: fragment_fn.to_string(),
            ..Default::default()
        }
    }

    pub fn add_color_attachment(&mut self, fmt: MetalPixelFormat) {
        self.color_attachments.push(fmt);
    }

    pub fn set_depth_format(&mut self, fmt: MetalPixelFormat) {
        self.depth_format = Some(fmt);
    }

    pub fn add_vertex_attribute(&mut self, attr: MetalVertexAttribute) {
        self.vertex_attributes.push(attr);
    }
}

/// Serialize to a Metal pipeline JSON stub.
pub fn to_metal_pipeline_json(d: &MetalExport) -> String {
    let depth = d.depth_format.map(|f| f.name()).unwrap_or("none");
    format!(
        "{{\"vertex_function\":\"{}\",\"fragment_function\":\"{}\",\
         \"color_attachment_count\":{},\"depth_format\":\"{}\"}}",
        d.vertex_function,
        d.fragment_function,
        d.color_attachments.len(),
        depth
    )
}

/// Count vertex attributes.
pub fn metal_vertex_attribute_count(d: &MetalExport) -> usize {
    d.vertex_attributes.len()
}

/// Create a new Metal export.
pub fn new_metal_export(vertex_fn: &str, fragment_fn: &str) -> MetalExport {
    MetalExport::new(vertex_fn, fragment_fn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_format_name() {
        assert_eq!(MetalPixelFormat::Rgba8Unorm.name(), "MTLPixelFormatRGBA8Unorm");
    }

    #[test]
    fn test_pixel_format_bytes_rgba16() {
        assert_eq!(MetalPixelFormat::Rgba16Float.bytes_per_pixel(), 8);
    }

    #[test]
    fn test_vertex_format_name() {
        assert_eq!(MetalVertexAttributeFormat::Float3.name(), "MTLVertexFormatFloat3");
    }

    #[test]
    fn test_new_metal_export() {
        let d = new_metal_export("vertexMain", "fragmentMain");
        assert_eq!(d.vertex_function, "vertexMain");
    }

    #[test]
    fn test_add_color_attachment() {
        let mut d = MetalExport::new("v", "f");
        d.add_color_attachment(MetalPixelFormat::Bgra8Unorm);
        assert_eq!(d.color_attachments.len(), 1);
    }

    #[test]
    fn test_set_depth_format() {
        let mut d = MetalExport::new("v", "f");
        d.set_depth_format(MetalPixelFormat::Depth32Float);
        assert_eq!(d.depth_format, Some(MetalPixelFormat::Depth32Float));
    }

    #[test]
    fn test_to_metal_pipeline_json() {
        let d = MetalExport::new("vertexMain", "fragmentMain");
        let s = to_metal_pipeline_json(&d);
        assert!(s.contains("vertexMain"));
    }

    #[test]
    fn test_to_metal_pipeline_json_depth() {
        let mut d = MetalExport::new("v", "f");
        d.set_depth_format(MetalPixelFormat::Depth32Float);
        let s = to_metal_pipeline_json(&d);
        assert!(s.contains("Depth32Float"));
    }

    #[test]
    fn test_add_vertex_attribute() {
        let mut d = MetalExport::new("v", "f");
        d.add_vertex_attribute(MetalVertexAttribute::new(
            "pos",
            0,
            MetalVertexAttributeFormat::Float3,
            0,
        ));
        assert_eq!(metal_vertex_attribute_count(&d), 1);
    }

    #[test]
    fn test_metal_export_no_depth() {
        let d = MetalExport::new("v", "f");
        let s = to_metal_pipeline_json(&d);
        assert!(s.contains("\"depth_format\":\"none\""));
    }
}
