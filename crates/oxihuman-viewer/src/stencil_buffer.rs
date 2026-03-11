// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Stencil buffer management utilities.

/// Stencil operation.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    IncrClamp,
    DecrClamp,
    Invert,
    IncrWrap,
    DecrWrap,
}

/// Stencil compare function.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StencilCompare {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

/// Stencil state configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StencilState {
    pub enabled: bool,
    pub read_mask: u8,
    pub write_mask: u8,
    pub reference: u8,
    pub compare: StencilCompare,
    pub pass_op: StencilOp,
    pub fail_op: StencilOp,
    pub depth_fail_op: StencilOp,
}

/// Default stencil state.
#[allow(dead_code)]
pub fn default_stencil_state() -> StencilState {
    StencilState {
        enabled: false,
        read_mask: 0xFF,
        write_mask: 0xFF,
        reference: 0,
        compare: StencilCompare::Always,
        pass_op: StencilOp::Keep,
        fail_op: StencilOp::Keep,
        depth_fail_op: StencilOp::Keep,
    }
}

/// Create a stencil write state.
#[allow(dead_code)]
pub fn stencil_write_state(reference: u8) -> StencilState {
    StencilState {
        enabled: true,
        read_mask: 0xFF,
        write_mask: 0xFF,
        reference,
        compare: StencilCompare::Always,
        pass_op: StencilOp::Replace,
        fail_op: StencilOp::Keep,
        depth_fail_op: StencilOp::Keep,
    }
}

/// Create a stencil test state.
#[allow(dead_code)]
pub fn stencil_test_state(reference: u8) -> StencilState {
    StencilState {
        enabled: true,
        read_mask: 0xFF,
        write_mask: 0x00,
        reference,
        compare: StencilCompare::Equal,
        pass_op: StencilOp::Keep,
        fail_op: StencilOp::Keep,
        depth_fail_op: StencilOp::Keep,
    }
}

/// Enable stencil.
#[allow(dead_code)]
pub fn enable_stencil(state: &mut StencilState) {
    state.enabled = true;
}

/// Disable stencil.
#[allow(dead_code)]
pub fn disable_stencil(state: &mut StencilState) {
    state.enabled = false;
}

/// Evaluate stencil test.
#[allow(dead_code)]
pub fn stencil_test(buffer_value: u8, state: &StencilState) -> bool {
    let masked_ref = state.reference & state.read_mask;
    let masked_buf = buffer_value & state.read_mask;
    match state.compare {
        StencilCompare::Never => false,
        StencilCompare::Less => masked_ref < masked_buf,
        StencilCompare::Equal => masked_ref == masked_buf,
        StencilCompare::LessEqual => masked_ref <= masked_buf,
        StencilCompare::Greater => masked_ref > masked_buf,
        StencilCompare::NotEqual => masked_ref != masked_buf,
        StencilCompare::GreaterEqual => masked_ref >= masked_buf,
        StencilCompare::Always => true,
    }
}

/// Apply stencil operation.
#[allow(dead_code)]
pub fn apply_stencil_op(current: u8, op: StencilOp, reference: u8) -> u8 {
    match op {
        StencilOp::Keep => current,
        StencilOp::Zero => 0,
        StencilOp::Replace => reference,
        StencilOp::IncrClamp => current.saturating_add(1),
        StencilOp::DecrClamp => current.saturating_sub(1),
        StencilOp::Invert => !current,
        StencilOp::IncrWrap => current.wrapping_add(1),
        StencilOp::DecrWrap => current.wrapping_sub(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let s = default_stencil_state();
        assert!(!s.enabled);
    }

    #[test]
    fn test_write_state() {
        let s = stencil_write_state(1);
        assert!(s.enabled);
        assert_eq!(s.reference, 1);
    }

    #[test]
    fn test_test_state() {
        let s = stencil_test_state(1);
        assert_eq!(s.compare, StencilCompare::Equal);
    }

    #[test]
    fn test_stencil_always() {
        let s = default_stencil_state();
        assert!(stencil_test(0, &s));
    }

    #[test]
    fn test_stencil_equal() {
        let s = stencil_test_state(5);
        assert!(stencil_test(5, &s));
        assert!(!stencil_test(3, &s));
    }

    #[test]
    fn test_stencil_never() {
        let mut s = default_stencil_state();
        s.compare = StencilCompare::Never;
        assert!(!stencil_test(0, &s));
    }

    #[test]
    fn test_apply_keep() {
        assert_eq!(apply_stencil_op(5, StencilOp::Keep, 0), 5);
    }

    #[test]
    fn test_apply_replace() {
        assert_eq!(apply_stencil_op(5, StencilOp::Replace, 10), 10);
    }

    #[test]
    fn test_apply_incr_clamp() {
        assert_eq!(apply_stencil_op(254, StencilOp::IncrClamp, 0), 255);
        assert_eq!(apply_stencil_op(255, StencilOp::IncrClamp, 0), 255);
    }

    #[test]
    fn test_enable_disable() {
        let mut s = default_stencil_state();
        enable_stencil(&mut s);
        assert!(s.enabled);
        disable_stencil(&mut s);
        assert!(!s.enabled);
    }
}
