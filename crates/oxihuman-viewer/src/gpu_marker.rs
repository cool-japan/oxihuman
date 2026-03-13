// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! GPU marker — debug group/label markers for GPU timeline profiling.

/// A single GPU debug marker/label.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GpuDebugMarker {
    pub label: String,
    pub color: [f32; 4],
    pub depth: usize,
}

/// GPU marker stack for nested push/pop groups.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct GpuMarkerStack {
    pub stack: Vec<GpuDebugMarker>,
    pub total_pushed: u64,
}

#[allow(dead_code)]
pub fn gm_push(stack: &mut GpuMarkerStack, label: &str, color: [f32; 4]) {
    let depth = stack.stack.len();
    stack.stack.push(GpuDebugMarker {
        label: label.to_string(),
        color,
        depth,
    });
    stack.total_pushed += 1;
}

#[allow(dead_code)]
pub fn gm_pop(stack: &mut GpuMarkerStack) -> Option<GpuDebugMarker> {
    stack.stack.pop()
}

#[allow(dead_code)]
pub fn gm_depth(stack: &GpuMarkerStack) -> usize {
    stack.stack.len()
}

#[allow(dead_code)]
pub fn gm_current_label(stack: &GpuMarkerStack) -> Option<&str> {
    stack.stack.last().map(|m| m.label.as_str())
}

#[allow(dead_code)]
pub fn gm_clear(stack: &mut GpuMarkerStack) {
    stack.stack.clear();
}

#[allow(dead_code)]
pub fn gm_is_empty(stack: &GpuMarkerStack) -> bool {
    stack.stack.is_empty()
}

#[allow(dead_code)]
pub fn gm_path(stack: &GpuMarkerStack) -> String {
    if stack.stack.is_empty() {
        return String::new();
    }
    let parts: Vec<&str> = stack.stack.iter().map(|m| m.label.as_str()).collect();
    parts.join(" > ")
}

#[allow(dead_code)]
pub fn gm_make_color(r: f32, g: f32, b: f32) -> [f32; 4] {
    [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), 1.0]
}

#[allow(dead_code)]
pub fn gm_to_json(stack: &GpuMarkerStack) -> String {
    format!(
        r#"{{"depth":{},"total_pushed":{}}}"#,
        gm_depth(stack),
        stack.total_pushed
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_increases_depth() {
        let mut s = GpuMarkerStack::default();
        gm_push(&mut s, "Frame", [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(gm_depth(&s), 1);
    }

    #[test]
    fn pop_decreases_depth() {
        let mut s = GpuMarkerStack::default();
        gm_push(&mut s, "Frame", [1.0, 0.0, 0.0, 1.0]);
        gm_pop(&mut s);
        assert_eq!(gm_depth(&s), 0);
    }

    #[test]
    fn pop_empty_returns_none() {
        let mut s = GpuMarkerStack::default();
        assert!(gm_pop(&mut s).is_none());
    }

    #[test]
    fn current_label() {
        let mut s = GpuMarkerStack::default();
        gm_push(&mut s, "Pass", [0.0; 4]);
        assert_eq!(gm_current_label(&s), Some("Pass"));
    }

    #[test]
    fn path_nested() {
        let mut s = GpuMarkerStack::default();
        gm_push(&mut s, "Frame", [0.0; 4]);
        gm_push(&mut s, "GBuffer", [0.0; 4]);
        let p = gm_path(&s);
        assert!(p.contains("Frame"));
        assert!(p.contains("GBuffer"));
    }

    #[test]
    fn clear_empties() {
        let mut s = GpuMarkerStack::default();
        gm_push(&mut s, "X", [0.0; 4]);
        gm_clear(&mut s);
        assert!(gm_is_empty(&s));
    }

    #[test]
    fn total_pushed_increments() {
        let mut s = GpuMarkerStack::default();
        gm_push(&mut s, "A", [0.0; 4]);
        gm_push(&mut s, "B", [0.0; 4]);
        assert_eq!(s.total_pushed, 2);
    }

    #[test]
    fn make_color_clamps() {
        let c = gm_make_color(2.0, -1.0, 0.5);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 0.0).abs() < 1e-6);
        assert!((c[2] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_json_fields() {
        let s = GpuMarkerStack::default();
        let j = gm_to_json(&s);
        assert!(j.contains("depth"));
        assert!(j.contains("total_pushed"));
    }
}
