// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Stencil mask: manages stencil buffer operations for selective rendering.

/// Stencil comparison function.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StencilFunc {
    Always,
    Never,
    Equal,
    NotEqual,
    Less,
    Greater,
}

/// Stencil operation.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    Increment,
    Decrement,
    Invert,
}

/// Stencil mask configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StencilMask {
    pub func: StencilFunc,
    pub reference: u8,
    pub read_mask: u8,
    pub write_mask: u8,
    pub pass_op: StencilOp,
    pub fail_op: StencilOp,
    pub depth_fail_op: StencilOp,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn default_stencil_mask() -> StencilMask {
    StencilMask {
        func: StencilFunc::Always,
        reference: 0,
        read_mask: 0xFF,
        write_mask: 0xFF,
        pass_op: StencilOp::Keep,
        fail_op: StencilOp::Keep,
        depth_fail_op: StencilOp::Keep,
        enabled: false,
    }
}

#[allow(dead_code)]
pub fn stencil_write_mask(mask: &mut StencilMask, reference: u8, op: StencilOp) {
    mask.reference = reference;
    mask.pass_op = op;
    mask.func = StencilFunc::Always;
    mask.enabled = true;
}

#[allow(dead_code)]
pub fn stencil_test_equal(mask: &mut StencilMask, reference: u8) {
    mask.reference = reference;
    mask.func = StencilFunc::Equal;
    mask.enabled = true;
}

#[allow(dead_code)]
pub fn stencil_test_not_equal(mask: &mut StencilMask, reference: u8) {
    mask.reference = reference;
    mask.func = StencilFunc::NotEqual;
    mask.enabled = true;
}

/// Evaluate a stencil comparison.
#[allow(dead_code)]
pub fn evaluate_stencil(func: StencilFunc, buf_value: u8, reference: u8, read_mask: u8) -> bool {
    let masked_buf = buf_value & read_mask;
    let masked_ref = reference & read_mask;
    match func {
        StencilFunc::Always => true,
        StencilFunc::Never => false,
        StencilFunc::Equal => masked_buf == masked_ref,
        StencilFunc::NotEqual => masked_buf != masked_ref,
        StencilFunc::Less => masked_buf < masked_ref,
        StencilFunc::Greater => masked_buf > masked_ref,
    }
}

/// Apply a stencil operation to a buffer value.
#[allow(dead_code)]
pub fn apply_stencil_op(op: StencilOp, current: u8, reference: u8, write_mask: u8) -> u8 {
    let new_val = match op {
        StencilOp::Keep => current,
        StencilOp::Zero => 0,
        StencilOp::Replace => reference,
        StencilOp::Increment => current.saturating_add(1),
        StencilOp::Decrement => current.saturating_sub(1),
        StencilOp::Invert => !current,
    };
    (current & !write_mask) | (new_val & write_mask)
}

#[allow(dead_code)]
pub fn stencil_mask_to_json(mask: &StencilMask) -> String {
    format!(
        r#"{{"enabled":{},"reference":{},"func":"{:?}"}}"#,
        mask.enabled, mask.reference, mask.func
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_stencil_mask() {
        let m = default_stencil_mask();
        assert!(!m.enabled);
        assert_eq!(m.reference, 0);
    }

    #[test]
    fn test_stencil_write() {
        let mut m = default_stencil_mask();
        stencil_write_mask(&mut m, 1, StencilOp::Replace);
        assert!(m.enabled);
        assert_eq!(m.reference, 1);
    }

    #[test]
    fn test_stencil_test_equal() {
        let mut m = default_stencil_mask();
        stencil_test_equal(&mut m, 5);
        assert_eq!(m.func, StencilFunc::Equal);
    }

    #[test]
    fn test_evaluate_always() {
        assert!(evaluate_stencil(StencilFunc::Always, 0, 0, 0xFF));
    }

    #[test]
    fn test_evaluate_never() {
        assert!(!evaluate_stencil(StencilFunc::Never, 0, 0, 0xFF));
    }

    #[test]
    fn test_evaluate_equal() {
        assert!(evaluate_stencil(StencilFunc::Equal, 5, 5, 0xFF));
        assert!(!evaluate_stencil(StencilFunc::Equal, 5, 3, 0xFF));
    }

    #[test]
    fn test_evaluate_not_equal() {
        assert!(evaluate_stencil(StencilFunc::NotEqual, 5, 3, 0xFF));
    }

    #[test]
    fn test_apply_replace() {
        let v = apply_stencil_op(StencilOp::Replace, 0, 42, 0xFF);
        assert_eq!(v, 42);
    }

    #[test]
    fn test_apply_increment() {
        let v = apply_stencil_op(StencilOp::Increment, 5, 0, 0xFF);
        assert_eq!(v, 6);
    }

    #[test]
    fn test_stencil_mask_to_json() {
        let m = default_stencil_mask();
        let j = stencil_mask_to_json(&m);
        assert!(j.contains("enabled"));
    }
}
