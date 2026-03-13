// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! DirectX HLSL stub export.

/// DXGI format codes (subset).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DxgiFormat {
    R32G32B32A32Float,
    R32G32B32Float,
    R32G32Float,
    R32Float,
    R8G8B8A8Unorm,
    D32Float,
    D24UnormS8Uint,
}

impl DxgiFormat {
    pub fn name(self) -> &'static str {
        match self {
            Self::R32G32B32A32Float => "DXGI_FORMAT_R32G32B32A32_FLOAT",
            Self::R32G32B32Float => "DXGI_FORMAT_R32G32B32_FLOAT",
            Self::R32G32Float => "DXGI_FORMAT_R32G32_FLOAT",
            Self::R32Float => "DXGI_FORMAT_R32_FLOAT",
            Self::R8G8B8A8Unorm => "DXGI_FORMAT_R8G8B8A8_UNORM",
            Self::D32Float => "DXGI_FORMAT_D32_FLOAT",
            Self::D24UnormS8Uint => "DXGI_FORMAT_D24_UNORM_S8_UINT",
        }
    }

    pub fn bytes_per_element(self) -> usize {
        match self {
            Self::R32Float | Self::D32Float => 4,
            Self::R32G32Float => 8,
            Self::R32G32B32Float => 12,
            Self::R32G32B32A32Float => 16,
            Self::R8G8B8A8Unorm => 4,
            Self::D24UnormS8Uint => 4,
        }
    }
}

/// An HLSL input element descriptor.
#[derive(Debug, Clone)]
pub struct HlslInputElement {
    pub semantic_name: String,
    pub semantic_index: u32,
    pub format: DxgiFormat,
    pub input_slot: u32,
    pub aligned_byte_offset: u32,
}

impl HlslInputElement {
    pub fn new(semantic: &str, index: u32, format: DxgiFormat, offset: u32) -> Self {
        Self {
            semantic_name: semantic.to_string(),
            semantic_index: index,
            format,
            input_slot: 0,
            aligned_byte_offset: offset,
        }
    }
}

/// A constant buffer layout entry.
#[derive(Debug, Clone)]
pub struct HlslConstantBufferEntry {
    pub name: String,
    pub type_name: String,
    pub offset: u32,
    pub size: u32,
}

impl HlslConstantBufferEntry {
    pub fn new(name: &str, type_name: &str, offset: u32, size: u32) -> Self {
        Self {
            name: name.to_string(),
            type_name: type_name.to_string(),
            offset,
            size,
        }
    }
}

/// DirectX pipeline export descriptor.
#[derive(Debug, Clone, Default)]
pub struct DirectXExport {
    pub input_layout: Vec<HlslInputElement>,
    pub constant_buffer: Vec<HlslConstantBufferEntry>,
    pub vs_profile: String,
    pub ps_profile: String,
    pub vertex_shader_source: String,
    pub pixel_shader_source: String,
}

impl DirectXExport {
    pub fn new(vs_profile: &str, ps_profile: &str) -> Self {
        Self {
            vs_profile: vs_profile.to_string(),
            ps_profile: ps_profile.to_string(),
            ..Default::default()
        }
    }

    pub fn add_input_element(&mut self, elem: HlslInputElement) {
        self.input_layout.push(elem);
    }

    pub fn add_constant_buffer_entry(&mut self, entry: HlslConstantBufferEntry) {
        self.constant_buffer.push(entry);
    }

    pub fn set_vertex_shader(&mut self, src: &str) {
        self.vertex_shader_source = src.to_string();
    }

    pub fn set_pixel_shader(&mut self, src: &str) {
        self.pixel_shader_source = src.to_string();
    }
}

/// Serialize to a pipeline JSON stub.
pub fn to_directx_pipeline_json(d: &DirectXExport) -> String {
    let total_stride: usize = d.input_layout.iter().map(|e| e.format.bytes_per_element()).sum();
    format!(
        "{{\"vs_profile\":\"{}\",\"ps_profile\":\"{}\",\
         \"input_element_count\":{},\"constant_buffer_entry_count\":{},\
         \"vertex_stride\":{}}}",
        d.vs_profile,
        d.ps_profile,
        d.input_layout.len(),
        d.constant_buffer.len(),
        total_stride
    )
}

/// Compute input layout stride.
pub fn directx_input_stride(d: &DirectXExport) -> usize {
    d.input_layout.iter().map(|e| e.format.bytes_per_element()).sum()
}

/// Create a new DirectX export.
pub fn new_directx_export(vs_profile: &str, ps_profile: &str) -> DirectXExport {
    DirectXExport::new(vs_profile, ps_profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dxgi_format_name() {
        assert_eq!(DxgiFormat::R32G32B32Float.name(), "DXGI_FORMAT_R32G32B32_FLOAT");
    }

    #[test]
    fn test_dxgi_format_bytes() {
        assert_eq!(DxgiFormat::R32G32B32A32Float.bytes_per_element(), 16);
    }

    #[test]
    fn test_new_directx_export() {
        let d = new_directx_export("vs_5_0", "ps_5_0");
        assert_eq!(d.vs_profile, "vs_5_0");
    }

    #[test]
    fn test_add_input_element() {
        let mut d = DirectXExport::new("vs_5_0", "ps_5_0");
        d.add_input_element(HlslInputElement::new("POSITION", 0, DxgiFormat::R32G32B32Float, 0));
        assert_eq!(d.input_layout.len(), 1);
    }

    #[test]
    fn test_add_constant_buffer_entry() {
        let mut d = DirectXExport::new("vs_5_0", "ps_5_0");
        d.add_constant_buffer_entry(HlslConstantBufferEntry::new("gWorld", "float4x4", 0, 64));
        assert_eq!(d.constant_buffer.len(), 1);
    }

    #[test]
    fn test_to_directx_pipeline_json_profiles() {
        let d = DirectXExport::new("vs_5_1", "ps_5_1");
        let s = to_directx_pipeline_json(&d);
        assert!(s.contains("vs_5_1"));
        assert!(s.contains("ps_5_1"));
    }

    #[test]
    fn test_directx_input_stride() {
        let mut d = DirectXExport::new("vs_5_0", "ps_5_0");
        d.add_input_element(HlslInputElement::new("POSITION", 0, DxgiFormat::R32G32B32Float, 0));
        d.add_input_element(HlslInputElement::new("TEXCOORD", 0, DxgiFormat::R32G32Float, 12));
        assert_eq!(directx_input_stride(&d), 20);
    }

    #[test]
    fn test_set_vertex_shader() {
        let mut d = DirectXExport::new("vs_5_0", "ps_5_0");
        d.set_vertex_shader("float4 VSMain() : SV_Position { return 0; }");
        assert!(!d.vertex_shader_source.is_empty());
    }

    #[test]
    fn test_d24_format_bytes() {
        assert_eq!(DxgiFormat::D24UnormS8Uint.bytes_per_element(), 4);
    }

    #[test]
    fn test_to_directx_json_stride_count() {
        let mut d = DirectXExport::new("vs_5_0", "ps_5_0");
        d.add_input_element(HlslInputElement::new("POS", 0, DxgiFormat::R32G32B32Float, 0));
        let s = to_directx_pipeline_json(&d);
        assert!(s.contains("\"vertex_stride\":12"));
    }
}
