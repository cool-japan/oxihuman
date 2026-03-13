// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vulkan pipeline stub export.

/// Vulkan shader stage flags.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VkShaderStage {
    Vertex,
    Fragment,
    Geometry,
    Compute,
    TessellationControl,
    TessellationEval,
}

impl VkShaderStage {
    pub fn stage_flag(self) -> u32 {
        match self {
            Self::Vertex => 0x00000001,
            Self::TessellationControl => 0x00000002,
            Self::TessellationEval => 0x00000004,
            Self::Geometry => 0x00000008,
            Self::Fragment => 0x00000010,
            Self::Compute => 0x00000020,
        }
    }
}

/// Vulkan vertex input attribute.
#[derive(Debug, Clone)]
pub struct VkVertexInputAttribute {
    pub location: u32,
    pub binding: u32,
    pub format: u32, /* VkFormat constant */
    pub offset: u32,
    pub name: String,
}

impl VkVertexInputAttribute {
    pub fn new(location: u32, binding: u32, format: u32, offset: u32, name: &str) -> Self {
        Self { location, binding, format, offset, name: name.to_string() }
    }
}

/// Vulkan descriptor set layout binding.
#[derive(Debug, Clone)]
pub struct VkDescriptorBinding {
    pub binding: u32,
    pub descriptor_type: u32, /* VkDescriptorType constant */
    pub descriptor_count: u32,
    pub stage_flags: u32,
}

impl VkDescriptorBinding {
    pub fn new(binding: u32, desc_type: u32, count: u32, stages: u32) -> Self {
        Self { binding, descriptor_type: desc_type, descriptor_count: count, stage_flags: stages }
    }
}

/// Vulkan pipeline export descriptor.
#[derive(Debug, Clone, Default)]
pub struct VulkanExport {
    pub vertex_attributes: Vec<VkVertexInputAttribute>,
    pub descriptor_bindings: Vec<VkDescriptorBinding>,
    pub push_constant_size: u32,
    pub shader_stages: Vec<VkShaderStage>,
}

impl VulkanExport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_vertex_attribute(&mut self, attr: VkVertexInputAttribute) {
        self.vertex_attributes.push(attr);
    }

    pub fn add_descriptor_binding(&mut self, binding: VkDescriptorBinding) {
        self.descriptor_bindings.push(binding);
    }

    pub fn add_shader_stage(&mut self, stage: VkShaderStage) {
        self.shader_stages.push(stage);
    }
}

/// Serialize to a pipeline descriptor JSON stub.
pub fn to_vulkan_pipeline_json(d: &VulkanExport) -> String {
    let stage_flags: u32 = d.shader_stages.iter().map(|s| s.stage_flag()).fold(0, |a, b| a | b);
    format!(
        "{{\"vertex_attribute_count\":{},\"descriptor_binding_count\":{},\
         \"stage_flags\":{},\"push_constant_size\":{}}}",
        d.vertex_attributes.len(),
        d.descriptor_bindings.len(),
        stage_flags,
        d.push_constant_size
    )
}

/// Compute total vertex stride.
pub fn vulkan_vertex_stride(attrs: &[VkVertexInputAttribute]) -> u32 {
    attrs.iter().map(|a| a.offset).max().unwrap_or(0)
}

/// Create a new Vulkan export.
pub fn new_vulkan_export() -> VulkanExport {
    VulkanExport::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_stage_flag() {
        assert_eq!(VkShaderStage::Vertex.stage_flag(), 0x00000001);
    }

    #[test]
    fn test_fragment_stage_flag() {
        assert_eq!(VkShaderStage::Fragment.stage_flag(), 0x00000010);
    }

    #[test]
    fn test_add_vertex_attribute() {
        let mut d = VulkanExport::new();
        d.add_vertex_attribute(VkVertexInputAttribute::new(0, 0, 106, 0, "pos"));
        assert_eq!(d.vertex_attributes.len(), 1);
    }

    #[test]
    fn test_add_descriptor_binding() {
        let mut d = VulkanExport::new();
        d.add_descriptor_binding(VkDescriptorBinding::new(0, 7, 1, 0x00000001));
        assert_eq!(d.descriptor_bindings.len(), 1);
    }

    #[test]
    fn test_add_shader_stage() {
        let mut d = VulkanExport::new();
        d.add_shader_stage(VkShaderStage::Vertex);
        d.add_shader_stage(VkShaderStage::Fragment);
        assert_eq!(d.shader_stages.len(), 2);
    }

    #[test]
    fn test_to_vulkan_pipeline_json_structure() {
        let d = VulkanExport::new();
        let s = to_vulkan_pipeline_json(&d);
        assert!(s.contains("vertex_attribute_count"));
    }

    #[test]
    fn test_to_vulkan_pipeline_json_stage_flags() {
        let mut d = VulkanExport::new();
        d.add_shader_stage(VkShaderStage::Vertex);
        d.add_shader_stage(VkShaderStage::Fragment);
        let s = to_vulkan_pipeline_json(&d);
        /* vertex (1) | fragment (16) = 17 */
        assert!(s.contains("17"));
    }

    #[test]
    fn test_vulkan_vertex_stride_empty() {
        assert_eq!(vulkan_vertex_stride(&[]), 0);
    }

    #[test]
    fn test_vulkan_vertex_stride_max() {
        let attrs = vec![
            VkVertexInputAttribute::new(0, 0, 106, 0, "pos"),
            VkVertexInputAttribute::new(1, 0, 109, 12, "uv"),
        ];
        assert_eq!(vulkan_vertex_stride(&attrs), 12);
    }

    #[test]
    fn test_new_vulkan_export_empty() {
        let d = new_vulkan_export();
        assert!(d.vertex_attributes.is_empty());
    }

    #[test]
    fn test_compute_stage_flag() {
        assert_eq!(VkShaderStage::Compute.stage_flag(), 0x00000020);
    }
}
