// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Depth-stencil buffer management and configuration.

/// Depth buffer format.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthStencilFormat {
    D16,
    D24S8,
    D32F,
    D32FS8,
}

/// Depth compare function.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthCmp {
    Never,
    Less,
    Equal,
    LessEq,
    Greater,
    NotEq,
    GreaterEq,
    Always,
}

/// Stencil operation.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StencilOp2 {
    Keep,
    Zero,
    Replace,
    IncrClamp,
    DecrClamp,
    Invert,
    IncrWrap,
    DecrWrap,
}

/// Full depth-stencil state.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct DepthStencilState2 {
    pub format: DepthStencilFormat,
    pub depth_test: bool,
    pub depth_write: bool,
    pub depth_cmp: DepthCmp,
    pub stencil_enabled: bool,
    pub stencil_ref: u8,
    pub stencil_write_mask: u8,
    pub stencil_read_mask: u8,
}

impl Default for DepthStencilState2 {
    fn default() -> Self {
        Self {
            format: DepthStencilFormat::D24S8,
            depth_test: true,
            depth_write: true,
            depth_cmp: DepthCmp::Less,
            stencil_enabled: false,
            stencil_ref: 0,
            stencil_write_mask: 0xFF,
            stencil_read_mask: 0xFF,
        }
    }
}

#[allow(dead_code)]
pub fn new_depth_stencil() -> DepthStencilState2 {
    DepthStencilState2::default()
}

#[allow(dead_code)]
pub fn ds_format_bits(fmt: DepthStencilFormat) -> u32 {
    match fmt {
        DepthStencilFormat::D16 => 16,
        DepthStencilFormat::D24S8 => 32,
        DepthStencilFormat::D32F => 32,
        DepthStencilFormat::D32FS8 => 40,
    }
}

#[allow(dead_code)]
pub fn ds_has_stencil(fmt: DepthStencilFormat) -> bool {
    matches!(fmt, DepthStencilFormat::D24S8 | DepthStencilFormat::D32FS8)
}

#[allow(dead_code)]
pub fn ds_set_depth_cmp(state: &mut DepthStencilState2, cmp: DepthCmp) {
    state.depth_cmp = cmp;
}

#[allow(dead_code)]
pub fn ds_enable_stencil(state: &mut DepthStencilState2, r#ref: u8) {
    state.stencil_enabled = true;
    state.stencil_ref = r#ref;
}

#[allow(dead_code)]
pub fn ds_format_name(fmt: DepthStencilFormat) -> &'static str {
    match fmt {
        DepthStencilFormat::D16 => "d16",
        DepthStencilFormat::D24S8 => "d24s8",
        DepthStencilFormat::D32F => "d32f",
        DepthStencilFormat::D32FS8 => "d32fs8",
    }
}

#[allow(dead_code)]
pub fn ds_cmp_name(cmp: DepthCmp) -> &'static str {
    match cmp {
        DepthCmp::Never => "never",
        DepthCmp::Less => "less",
        DepthCmp::Equal => "equal",
        DepthCmp::LessEq => "less_eq",
        DepthCmp::Greater => "greater",
        DepthCmp::NotEq => "not_eq",
        DepthCmp::GreaterEq => "greater_eq",
        DepthCmp::Always => "always",
    }
}

#[allow(dead_code)]
pub fn ds_to_json(state: &DepthStencilState2) -> String {
    format!(
        "{{\"format\":\"{}\",\"depth_test\":{},\"depth_write\":{},\"cmp\":\"{}\",\"stencil\":{}}}",
        ds_format_name(state.format),
        state.depth_test,
        state.depth_write,
        ds_cmp_name(state.depth_cmp),
        state.stencil_enabled
    )
}

#[allow(dead_code)]
pub fn ds_reset(state: &mut DepthStencilState2) {
    *state = DepthStencilState2::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_depth_test_on() {
        assert!(new_depth_stencil().depth_test);
    }

    #[test]
    fn default_cmp_less() {
        assert_eq!(new_depth_stencil().depth_cmp, DepthCmp::Less);
    }

    #[test]
    fn d16_is_16_bits() {
        assert_eq!(ds_format_bits(DepthStencilFormat::D16), 16);
    }

    #[test]
    fn d24s8_has_stencil() {
        assert!(ds_has_stencil(DepthStencilFormat::D24S8));
    }

    #[test]
    fn d16_no_stencil() {
        assert!(!ds_has_stencil(DepthStencilFormat::D16));
    }

    #[test]
    fn enable_stencil() {
        let mut s = new_depth_stencil();
        ds_enable_stencil(&mut s, 1);
        assert!(s.stencil_enabled);
        assert_eq!(s.stencil_ref, 1);
    }

    #[test]
    fn format_name_d32f() {
        assert_eq!(ds_format_name(DepthStencilFormat::D32F), "d32f");
    }

    #[test]
    fn cmp_name_always() {
        assert_eq!(ds_cmp_name(DepthCmp::Always), "always");
    }

    #[test]
    fn reset_restores_default() {
        let mut s = new_depth_stencil();
        ds_enable_stencil(&mut s, 5);
        ds_reset(&mut s);
        assert!(!s.stencil_enabled);
    }

    #[test]
    fn json_has_format() {
        assert!(ds_to_json(&new_depth_stencil()).contains("format"));
    }
}
