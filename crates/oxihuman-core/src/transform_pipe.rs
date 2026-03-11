// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Data transform pipeline: chain of named f32 transforms.

/// A named transform stage (maps f32 -> f32).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TransformStage {
    pub name: String,
    pub kind: TransformKind,
}

/// Built-in transform kinds.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TransformKind {
    Scale(f32),
    Offset(f32),
    Clamp(f32, f32),
    Abs,
    Negate,
    Reciprocal,
    Sin,
    Cos,
}

impl TransformKind {
    #[allow(dead_code)]
    fn apply(&self, x: f32) -> f32 {
        match self {
            TransformKind::Scale(s) => x * s,
            TransformKind::Offset(o) => x + o,
            TransformKind::Clamp(lo, hi) => x.clamp(*lo, *hi),
            TransformKind::Abs => x.abs(),
            TransformKind::Negate => -x,
            TransformKind::Reciprocal => {
                if x.abs() > 1e-9 {
                    1.0 / x
                } else {
                    0.0
                }
            }
            TransformKind::Sin => x.sin(),
            TransformKind::Cos => x.cos(),
        }
    }
}

/// A pipeline of transform stages.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TransformPipe {
    stages: Vec<TransformStage>,
}

/// Create an empty `TransformPipe`.
#[allow(dead_code)]
pub fn new_transform_pipe() -> TransformPipe {
    TransformPipe::default()
}

/// Add a stage to the pipe.
#[allow(dead_code)]
pub fn tp_add(pipe: &mut TransformPipe, name: &str, kind: TransformKind) {
    pipe.stages.push(TransformStage {
        name: name.to_string(),
        kind,
    });
}

/// Apply the full pipeline to a value.
#[allow(dead_code)]
pub fn tp_apply(pipe: &TransformPipe, mut val: f32) -> f32 {
    for stage in &pipe.stages {
        val = stage.kind.apply(val);
    }
    val
}

/// Number of stages.
#[allow(dead_code)]
pub fn tp_len(pipe: &TransformPipe) -> usize {
    pipe.stages.len()
}

/// Whether the pipeline is empty.
#[allow(dead_code)]
pub fn tp_is_empty(pipe: &TransformPipe) -> bool {
    pipe.stages.is_empty()
}

/// Remove the last stage.
#[allow(dead_code)]
pub fn tp_pop(pipe: &mut TransformPipe) -> Option<TransformStage> {
    pipe.stages.pop()
}

/// Clear all stages.
#[allow(dead_code)]
pub fn tp_clear(pipe: &mut TransformPipe) {
    pipe.stages.clear();
}

/// Get stage by index.
#[allow(dead_code)]
pub fn tp_get(pipe: &TransformPipe, idx: usize) -> Option<&TransformStage> {
    pipe.stages.get(idx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_empty_pipe_passthrough() {
        let pipe = new_transform_pipe();
        assert!((tp_apply(&pipe, 3.5) - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_scale() {
        let mut pipe = new_transform_pipe();
        tp_add(&mut pipe, "x2", TransformKind::Scale(2.0));
        assert!((tp_apply(&pipe, 3.0) - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_offset() {
        let mut pipe = new_transform_pipe();
        tp_add(&mut pipe, "shift", TransformKind::Offset(1.0));
        assert!((tp_apply(&pipe, 4.0) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp() {
        let mut pipe = new_transform_pipe();
        tp_add(&mut pipe, "clamp", TransformKind::Clamp(0.0, 1.0));
        assert!((tp_apply(&pipe, 2.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_abs() {
        let mut pipe = new_transform_pipe();
        tp_add(&mut pipe, "abs", TransformKind::Abs);
        assert!((tp_apply(&pipe, -7.0) - 7.0).abs() < 1e-6);
    }

    #[test]
    fn test_negate() {
        let mut pipe = new_transform_pipe();
        tp_add(&mut pipe, "neg", TransformKind::Negate);
        assert!((tp_apply(&pipe, 5.0) - (-5.0)).abs() < 1e-6);
    }

    #[test]
    fn test_chain() {
        let mut pipe = new_transform_pipe();
        tp_add(&mut pipe, "scale", TransformKind::Scale(2.0));
        tp_add(&mut pipe, "offset", TransformKind::Offset(-1.0));
        assert!((tp_apply(&pipe, 3.0) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_sin_at_pi() {
        let mut pipe = new_transform_pipe();
        tp_add(&mut pipe, "sin", TransformKind::Sin);
        assert!(tp_apply(&pipe, PI).abs() < 1e-5);
    }

    #[test]
    fn test_pop_removes_stage() {
        let mut pipe = new_transform_pipe();
        tp_add(&mut pipe, "a", TransformKind::Abs);
        tp_pop(&mut pipe);
        assert!(tp_is_empty(&pipe));
    }

    #[test]
    fn test_clear() {
        let mut pipe = new_transform_pipe();
        tp_add(&mut pipe, "a", TransformKind::Abs);
        tp_clear(&mut pipe);
        assert_eq!(tp_len(&pipe), 0);
    }
}
