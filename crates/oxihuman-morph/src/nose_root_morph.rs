// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Nose root width morph — controls the nasion width and radix depth.

/// Nose root morph configuration.
#[derive(Debug, Clone)]
pub struct NoseRootMorph {
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    pub bridge_continuity: f32,
    pub intercanthal_ratio: f32,
}

impl NoseRootMorph {
    pub fn new() -> Self {
        Self {
            width: 0.5,
            depth: 0.5,
            height: 0.5,
            bridge_continuity: 0.5,
            intercanthal_ratio: 0.5,
        }
    }
}

impl Default for NoseRootMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new nose root morph.
pub fn new_nose_root_morph() -> NoseRootMorph {
    NoseRootMorph::new()
}

/// Set root width (0 = narrow, 1 = wide).
pub fn nroot_set_width(m: &mut NoseRootMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

/// Set radix depth (0 = flat/shallow, 1 = deep).
pub fn nroot_set_depth(m: &mut NoseRootMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

/// Set root height (vertical position on bridge).
pub fn nroot_set_height(m: &mut NoseRootMorph, v: f32) {
    m.height = v.clamp(0.0, 1.0);
}

/// Set bridge-to-forehead continuity smoothness.
pub fn nroot_set_bridge_continuity(m: &mut NoseRootMorph, v: f32) {
    m.bridge_continuity = v.clamp(0.0, 1.0);
}

/// Compute aspect ratio of root region.
pub fn nroot_aspect_ratio(m: &NoseRootMorph) -> f32 {
    if m.depth.abs() < 1e-6 {
        return 0.0;
    }
    m.width / m.depth
}

/// Serialize to JSON-like string.
pub fn nose_root_morph_to_json(m: &NoseRootMorph) -> String {
    format!(
        r#"{{"width":{:.4},"depth":{:.4},"height":{:.4},"bridge_continuity":{:.4},"intercanthal_ratio":{:.4}}}"#,
        m.width, m.depth, m.height, m.bridge_continuity, m.intercanthal_ratio
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_nose_root_morph();
        assert!((m.width - 0.5).abs() < 1e-6);
        assert!((m.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_width_clamp_high() {
        let mut m = new_nose_root_morph();
        nroot_set_width(&mut m, 5.0);
        assert_eq!(m.width, 1.0);
    }

    #[test]
    fn test_width_clamp_low() {
        let mut m = new_nose_root_morph();
        nroot_set_width(&mut m, -1.0);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_depth_set() {
        let mut m = new_nose_root_morph();
        nroot_set_depth(&mut m, 0.7);
        assert!((m.depth - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_height_set() {
        let mut m = new_nose_root_morph();
        nroot_set_height(&mut m, 0.6);
        assert!((m.height - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_bridge_continuity_clamp() {
        let mut m = new_nose_root_morph();
        nroot_set_bridge_continuity(&mut m, 2.0);
        assert_eq!(m.bridge_continuity, 1.0);
    }

    #[test]
    fn test_aspect_ratio_positive() {
        let m = new_nose_root_morph();
        assert!(nroot_aspect_ratio(&m) > 0.0);
    }

    #[test]
    fn test_aspect_ratio_zero_depth() {
        let mut m = new_nose_root_morph();
        m.depth = 0.0;
        assert_eq!(nroot_aspect_ratio(&m), 0.0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_nose_root_morph();
        let s = nose_root_morph_to_json(&m);
        assert!(s.contains("bridge_continuity"));
    }

    #[test]
    fn test_clone() {
        let m = new_nose_root_morph();
        let m2 = m.clone();
        assert!((m2.height - m.height).abs() < 1e-6);
    }
}
