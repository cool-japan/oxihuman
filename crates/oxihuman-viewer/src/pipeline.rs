// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! WebGPU-style render pipeline descriptors (pure data, no GPU calls).

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Vertex attribute data format.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VertexFormat {
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
}

/// Index buffer element format.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexFormat {
    Uint16,
    Uint32,
}

/// Primitive assembly topology.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveTopology {
    TriangleList,
    TriangleStrip,
    LineList,
    PointList,
}

/// Back/front/none face culling.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CullMode {
    None,
    Front,
    Back,
}

/// Blend factor used in src/dst blending.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlendFactor {
    Zero,
    One,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
}

/// Depth comparison function.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareFunction {
    Never,
    Less,
    LessEqual,
    Equal,
    GreaterEqual,
    Greater,
    Always,
}

// ── Structs ───────────────────────────────────────────────────────────────────

/// A single vertex attribute within a buffer layout.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub name: String,
    pub format: VertexFormat,
    pub offset: u32,
    pub shader_location: u32,
}

/// Describes the stride and attributes of one vertex buffer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexBufferLayout {
    pub array_stride: u64,
    pub attributes: Vec<VertexAttribute>,
}

/// Alpha blending state for a color attachment.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendState {
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
}

/// Depth/stencil attachment configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DepthStencilState {
    pub depth_write: bool,
    pub depth_compare: CompareFunction,
}

/// Full render pipeline descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPipelineDescriptor {
    pub label: String,
    pub vertex_layout: VertexBufferLayout,
    pub topology: PrimitiveTopology,
    pub cull_mode: CullMode,
    pub blend: Option<BlendState>,
    pub depth_stencil: Option<DepthStencilState>,
    pub index_format: IndexFormat,
}

// ── Public functions ──────────────────────────────────────────────────────────

/// Returns the byte size of a single element in the given [`VertexFormat`].
#[allow(dead_code)]
pub fn vertex_format_size(fmt: &VertexFormat) -> u32 {
    match fmt {
        VertexFormat::Float32x2 => 8,
        VertexFormat::Float32x3 => 12,
        VertexFormat::Float32x4 => 16,
        VertexFormat::Uint32 => 4,
    }
}

/// Builds the standard mesh vertex layout:
/// position(Float32x3) + normal(Float32x3) + uv(Float32x2) with stride=32.
#[allow(dead_code)]
pub fn standard_vertex_layout() -> VertexBufferLayout {
    VertexBufferLayout {
        array_stride: 32,
        attributes: vec![
            VertexAttribute {
                name: "position".to_string(),
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            },
            VertexAttribute {
                name: "normal".to_string(),
                format: VertexFormat::Float32x3,
                offset: 12,
                shader_location: 1,
            },
            VertexAttribute {
                name: "uv".to_string(),
                format: VertexFormat::Float32x2,
                offset: 24,
                shader_location: 2,
            },
        ],
    }
}

/// Standard opaque mesh pipeline (back-cull, depth LessEqual, no alpha blend).
#[allow(dead_code)]
pub fn default_mesh_pipeline() -> RenderPipelineDescriptor {
    RenderPipelineDescriptor {
        label: "default_mesh".to_string(),
        vertex_layout: standard_vertex_layout(),
        topology: PrimitiveTopology::TriangleList,
        cull_mode: CullMode::Back,
        blend: None,
        depth_stencil: Some(DepthStencilState {
            depth_write: true,
            depth_compare: CompareFunction::LessEqual,
        }),
        index_format: IndexFormat::Uint32,
    }
}

/// Alpha-blended transparent pipeline (depth-write disabled).
#[allow(dead_code)]
pub fn transparent_pipeline() -> RenderPipelineDescriptor {
    RenderPipelineDescriptor {
        label: "transparent".to_string(),
        vertex_layout: standard_vertex_layout(),
        topology: PrimitiveTopology::TriangleList,
        cull_mode: CullMode::None,
        blend: Some(BlendState {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
        }),
        depth_stencil: Some(DepthStencilState {
            depth_write: false,
            depth_compare: CompareFunction::LessEqual,
        }),
        index_format: IndexFormat::Uint32,
    }
}

/// Wireframe pipeline using LineList topology, no culling, no blend.
#[allow(dead_code)]
pub fn wireframe_pipeline() -> RenderPipelineDescriptor {
    RenderPipelineDescriptor {
        label: "wireframe".to_string(),
        vertex_layout: standard_vertex_layout(),
        topology: PrimitiveTopology::LineList,
        cull_mode: CullMode::None,
        blend: None,
        depth_stencil: Some(DepthStencilState {
            depth_write: true,
            depth_compare: CompareFunction::LessEqual,
        }),
        index_format: IndexFormat::Uint32,
    }
}

/// Validates that the declared stride matches the sum of all attribute sizes.
///
/// Returns `Ok(())` when valid, or `Err(message)` describing the mismatch.
#[allow(dead_code)]
pub fn validate_pipeline(desc: &RenderPipelineDescriptor) -> Result<(), String> {
    let attrs = &desc.vertex_layout.attributes;
    if attrs.is_empty() {
        return Err("vertex layout has no attributes".to_string());
    }

    // Find the last attribute (by offset) and compute the expected minimum stride.
    let expected_stride: u32 = attrs.iter().fold(0u32, |acc, attr| {
        acc.max(attr.offset + vertex_format_size(&attr.format))
    });

    let declared = desc.vertex_layout.array_stride as u32;
    if declared < expected_stride {
        return Err(format!(
            "declared stride {} is less than required stride {}",
            declared, expected_stride
        ));
    }
    Ok(())
}

/// Returns a human-readable one-line summary of the pipeline descriptor.
#[allow(dead_code)]
pub fn pipeline_summary(desc: &RenderPipelineDescriptor) -> String {
    let blend_str = if desc.blend.is_some() {
        "alpha-blend"
    } else {
        "opaque"
    };
    let depth_str = desc
        .depth_stencil
        .as_ref()
        .map(|d| {
            format!(
                "depth_write={} compare={:?}",
                d.depth_write, d.depth_compare
            )
        })
        .unwrap_or_else(|| "no-depth".to_string());
    format!(
        "[{}] topology={:?} cull={:?} {} {}",
        desc.label, desc.topology, desc.cull_mode, blend_str, depth_str
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_format_size_float32x2() {
        assert_eq!(vertex_format_size(&VertexFormat::Float32x2), 8);
    }

    #[test]
    fn vertex_format_size_float32x3() {
        assert_eq!(vertex_format_size(&VertexFormat::Float32x3), 12);
    }

    #[test]
    fn vertex_format_size_float32x4() {
        assert_eq!(vertex_format_size(&VertexFormat::Float32x4), 16);
    }

    #[test]
    fn vertex_format_size_uint32() {
        assert_eq!(vertex_format_size(&VertexFormat::Uint32), 4);
    }

    #[test]
    fn standard_vertex_layout_stride_is_32() {
        let layout = standard_vertex_layout();
        assert_eq!(layout.array_stride, 32);
    }

    #[test]
    fn standard_vertex_layout_has_three_attributes() {
        let layout = standard_vertex_layout();
        assert_eq!(layout.attributes.len(), 3);
    }

    #[test]
    fn validate_pipeline_valid() {
        let desc = default_mesh_pipeline();
        assert!(validate_pipeline(&desc).is_ok());
    }

    #[test]
    fn validate_pipeline_invalid_stride() {
        let mut desc = default_mesh_pipeline();
        // Shrink the stride below what the attributes require.
        desc.vertex_layout.array_stride = 4;
        assert!(validate_pipeline(&desc).is_err());
    }

    #[test]
    fn default_mesh_pipeline_cull_back() {
        let desc = default_mesh_pipeline();
        assert_eq!(desc.cull_mode, CullMode::Back);
    }

    #[test]
    fn default_mesh_pipeline_depth_write_true() {
        let desc = default_mesh_pipeline();
        let ds = desc
            .depth_stencil
            .as_ref()
            .expect("should have depth stencil");
        assert!(ds.depth_write);
    }

    #[test]
    fn transparent_pipeline_depth_write_false() {
        let desc = transparent_pipeline();
        let ds = desc
            .depth_stencil
            .as_ref()
            .expect("should have depth stencil");
        assert!(!ds.depth_write);
    }

    #[test]
    fn transparent_pipeline_has_blend_state() {
        let desc = transparent_pipeline();
        assert!(desc.blend.is_some());
    }

    #[test]
    fn wireframe_pipeline_topology_line_list() {
        let desc = wireframe_pipeline();
        assert_eq!(desc.topology, PrimitiveTopology::LineList);
    }

    #[test]
    fn wireframe_pipeline_no_blend() {
        let desc = wireframe_pipeline();
        assert!(desc.blend.is_none());
    }

    #[test]
    fn pipeline_summary_non_empty() {
        let desc = default_mesh_pipeline();
        let s = pipeline_summary(&desc);
        assert!(!s.is_empty());
        assert!(s.contains("default_mesh"));
    }

    #[test]
    fn blend_state_factors() {
        let bs = BlendState {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
        };
        assert_eq!(bs.src_factor, BlendFactor::SrcAlpha);
        assert_eq!(bs.dst_factor, BlendFactor::OneMinusSrcAlpha);
    }
}
