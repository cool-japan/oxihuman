// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Generic compute shader stub export.

/// Target API for compute shader.
#[derive(Clone, Copy, PartialEq)]
pub enum ComputeApi {
    WebGpu,
    Vulkan,
    Metal,
    DirectX12,
}

impl ComputeApi {
    pub fn name(&self) -> &'static str {
        match self {
            ComputeApi::WebGpu => "WebGPU",
            ComputeApi::Vulkan => "Vulkan",
            ComputeApi::Metal => "Metal",
            ComputeApi::DirectX12 => "DirectX 12",
        }
    }
}

/// Compute dispatch configuration.
pub struct DispatchConfig {
    pub group_size_x: u32,
    pub group_size_y: u32,
    pub group_size_z: u32,
}

impl Default for DispatchConfig {
    fn default() -> Self {
        Self {
            group_size_x: 64,
            group_size_y: 1,
            group_size_z: 1,
        }
    }
}

/// A generic compute shader export.
pub struct ComputeShaderExport {
    pub api: ComputeApi,
    pub source: String,
    pub entry_point: String,
    pub dispatch: DispatchConfig,
    pub bindings: Vec<String>,
}

/// Create a new compute shader export.
pub fn new_compute_shader_export(api: ComputeApi, entry: &str) -> ComputeShaderExport {
    ComputeShaderExport {
        api,
        source: String::new(),
        entry_point: entry.to_string(),
        dispatch: DispatchConfig::default(),
        bindings: Vec::new(),
    }
}

/// Set the shader source.
pub fn set_compute_source(exp: &mut ComputeShaderExport, src: &str) {
    exp.source = src.to_string();
}

/// Add a binding declaration.
pub fn add_compute_binding(exp: &mut ComputeShaderExport, binding: &str) {
    exp.bindings.push(binding.to_string());
}

/// Binding count.
pub fn compute_binding_count(exp: &ComputeShaderExport) -> usize {
    exp.bindings.len()
}

/// Compute number of groups for a given element count.
pub fn compute_group_count(exp: &ComputeShaderExport, element_count: u32) -> u32 {
    let gs = exp.dispatch.group_size_x.max(1);
    element_count.div_ceil(gs)
}

/// Validate (non-empty source and entry point).
pub fn validate_compute_shader(exp: &ComputeShaderExport) -> bool {
    !exp.source.is_empty() && !exp.entry_point.is_empty()
}

/// Render a summary string.
pub fn render_compute_summary(exp: &ComputeShaderExport) -> String {
    format!(
        "API:{} Entry:{} Bindings:{} GroupSize:{}x{}x{}",
        exp.api.name(),
        exp.entry_point,
        exp.bindings.len(),
        exp.dispatch.group_size_x,
        exp.dispatch.group_size_y,
        exp.dispatch.group_size_z,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty_source() {
        let exp = new_compute_shader_export(ComputeApi::Vulkan, "cs_main");
        assert!(exp.source.is_empty() /* no source */);
    }

    #[test]
    fn set_source_updates() {
        let mut exp = new_compute_shader_export(ComputeApi::Metal, "main");
        set_compute_source(&mut exp, "kernel void main(){}");
        assert!(!exp.source.is_empty() /* has source */);
    }

    #[test]
    fn add_binding_increments() {
        let mut exp = new_compute_shader_export(ComputeApi::WebGpu, "cs");
        add_compute_binding(
            &mut exp,
            "@group(0) @binding(0) var<storage> buf: array<f32>",
        );
        assert_eq!(compute_binding_count(&exp), 1 /* one binding */);
    }

    #[test]
    fn api_name_correct() {
        assert_eq!(ComputeApi::DirectX12.name(), "DirectX 12" /* DX12 */);
    }

    #[test]
    fn compute_group_count_correct() {
        let exp = new_compute_shader_export(ComputeApi::Vulkan, "cs");
        let groups = compute_group_count(&exp, 128);
        assert_eq!(groups, 2 /* 128 / 64 */);
    }

    #[test]
    fn compute_group_count_ceiling() {
        let exp = new_compute_shader_export(ComputeApi::Vulkan, "cs");
        let groups = compute_group_count(&exp, 65);
        assert_eq!(groups, 2 /* ceil(65/64) = 2 */);
    }

    #[test]
    fn validate_needs_source_and_entry() {
        let mut exp = new_compute_shader_export(ComputeApi::Metal, "main");
        assert!(!validate_compute_shader(&exp) /* no source */);
        set_compute_source(&mut exp, "// code");
        assert!(validate_compute_shader(&exp) /* now valid */);
    }

    #[test]
    fn render_summary_contains_api() {
        let exp = new_compute_shader_export(ComputeApi::WebGpu, "main");
        let s = render_compute_summary(&exp);
        assert!(s.contains("WebGPU") /* API in summary */);
    }

    #[test]
    fn default_dispatch_group_size() {
        let exp = new_compute_shader_export(ComputeApi::Vulkan, "cs");
        assert_eq!(exp.dispatch.group_size_x, 64 /* default 64 */);
    }
}
