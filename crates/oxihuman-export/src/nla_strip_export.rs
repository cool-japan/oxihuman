// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! NLA strip export.

/// A Non-Linear Animation strip.
#[derive(Debug, Clone)]
pub struct NlaStrip {
    pub name: String,
    pub action_name: String,
    pub frame_start: f32,
    pub frame_end: f32,
    pub scale: f32,
}

/// Create a new `NlaStrip`.
pub fn new_nla_strip(name: &str, action_name: &str, frame_start: f32, frame_end: f32) -> NlaStrip {
    NlaStrip {
        name: name.to_string(),
        action_name: action_name.to_string(),
        frame_start,
        frame_end,
        scale: 1.0,
    }
}

/// Serialize a single strip to JSON.
pub fn strip_to_json(s: &NlaStrip) -> String {
    format!(
        "{{\"name\":\"{}\",\"action\":\"{}\",\"start\":{},\"end\":{},\"scale\":{}}}",
        s.name, s.action_name, s.frame_start, s.frame_end, s.scale
    )
}

/// Serialize multiple strips to a JSON array.
pub fn strips_to_json(strips: &[NlaStrip]) -> String {
    let inner: Vec<String> = strips.iter().map(strip_to_json).collect();
    format!("[{}]", inner.join(","))
}

/// Duration of a strip in frames.
pub fn strip_duration(s: &NlaStrip) -> f32 {
    (s.frame_end - s.frame_start).max(0.0)
}

/// Returns true if two strips have overlapping frame ranges.
pub fn strip_overlaps(a: &NlaStrip, b: &NlaStrip) -> bool {
    a.frame_start < b.frame_end && b.frame_start < a.frame_end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_nla_strip() {
        let s = new_nla_strip("walk_cycle", "walk", 0.0, 30.0);
        assert_eq!(s.name, "walk_cycle");
    }

    #[test]
    fn test_strip_to_json() {
        let s = new_nla_strip("s", "a", 0.0, 24.0);
        let j = strip_to_json(&s);
        assert!(j.contains("\"name\":\"s\""));
    }

    #[test]
    fn test_strip_duration() {
        let s = new_nla_strip("s", "a", 10.0, 40.0);
        assert!((strip_duration(&s) - 30.0).abs() < 1e-5);
    }

    #[test]
    fn test_strip_overlaps_true() {
        let a = new_nla_strip("a", "x", 0.0, 20.0);
        let b = new_nla_strip("b", "y", 10.0, 30.0);
        assert!(strip_overlaps(&a, &b));
    }

    #[test]
    fn test_strip_overlaps_false() {
        let a = new_nla_strip("a", "x", 0.0, 10.0);
        let b = new_nla_strip("b", "y", 20.0, 30.0);
        assert!(!strip_overlaps(&a, &b));
    }
}
