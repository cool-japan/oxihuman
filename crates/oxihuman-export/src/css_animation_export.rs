// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! CSS keyframe animation export stub.

/// A single CSS keyframe stop.
pub struct CssKeyframe {
    pub percent: f32,
    pub property: String,
    pub value: String,
}

/// A CSS @keyframes animation.
pub struct CssAnimation {
    pub name: String,
    pub keyframes: Vec<CssKeyframe>,
    pub duration_ms: u32,
    pub iteration_count: String,
    pub timing_function: String,
}

/// Create a new CSS animation.
pub fn new_css_animation(name: &str, duration_ms: u32) -> CssAnimation {
    CssAnimation {
        name: name.to_string(),
        keyframes: Vec::new(),
        duration_ms,
        iteration_count: "1".to_string(),
        timing_function: "ease".to_string(),
    }
}

/// Add a keyframe stop.
pub fn add_css_keyframe(anim: &mut CssAnimation, percent: f32, property: &str, value: &str) {
    anim.keyframes.push(CssKeyframe {
        percent: percent.clamp(0.0, 100.0),
        property: property.to_string(),
        value: value.to_string(),
    });
}

/// Keyframe count.
pub fn css_keyframe_count(anim: &CssAnimation) -> usize {
    anim.keyframes.len()
}

/// Render to a CSS @keyframes string.
pub fn render_css_keyframes(anim: &CssAnimation) -> String {
    let mut s = format!("@keyframes {} {{\n", anim.name);
    for kf in &anim.keyframes {
        s.push_str(&format!(
            "  {}% {{ {}: {}; }}\n",
            kf.percent, kf.property, kf.value
        ));
    }
    s.push('}');
    s
}

/// Render animation selector rule.
pub fn render_css_animation_rule(selector: &str, anim: &CssAnimation) -> String {
    format!(
        "{} {{ animation: {} {}ms {} {}; }}",
        selector, anim.name, anim.duration_ms, anim.timing_function, anim.iteration_count
    )
}

/// Validate animation (name non-empty, duration > 0).
pub fn validate_css_animation(anim: &CssAnimation) -> bool {
    !anim.name.is_empty() && anim.duration_ms > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_animation_empty() {
        let a = new_css_animation("fade", 500);
        assert_eq!(css_keyframe_count(&a), 0 /* empty */);
    }

    #[test]
    fn add_keyframe_increments() {
        let mut a = new_css_animation("slide", 300);
        add_css_keyframe(&mut a, 0.0, "transform", "translateX(0)");
        assert_eq!(css_keyframe_count(&a), 1 /* one keyframe */);
    }

    #[test]
    fn keyframe_percent_clamped() {
        let mut a = new_css_animation("x", 100);
        add_css_keyframe(&mut a, 150.0, "opacity", "1");
        assert!(a.keyframes[0].percent <= 100.0 /* clamped */);
    }

    #[test]
    fn render_keyframes_contains_name() {
        let a = new_css_animation("pulse", 1000);
        let css = render_css_keyframes(&a);
        assert!(css.contains("pulse") /* name in output */);
    }

    #[test]
    fn render_keyframes_contains_property() {
        let mut a = new_css_animation("anim", 1000);
        add_css_keyframe(&mut a, 50.0, "opacity", "0.5");
        let css = render_css_keyframes(&a);
        assert!(css.contains("opacity") /* property */);
    }

    #[test]
    fn validate_valid_animation() {
        let a = new_css_animation("valid", 200);
        assert!(validate_css_animation(&a) /* valid */);
    }

    #[test]
    fn validate_zero_duration_fails() {
        let a = new_css_animation("zero", 0);
        assert!(!validate_css_animation(&a) /* invalid */);
    }

    #[test]
    fn render_rule_contains_selector() {
        let a = new_css_animation("bounce", 400);
        let rule = render_css_animation_rule(".box", &a);
        assert!(rule.contains(".box") /* selector */);
    }

    #[test]
    fn multiple_keyframes() {
        let mut a = new_css_animation("m", 1000);
        add_css_keyframe(&mut a, 0.0, "opacity", "0");
        add_css_keyframe(&mut a, 50.0, "opacity", "0.5");
        add_css_keyframe(&mut a, 100.0, "opacity", "1");
        assert_eq!(css_keyframe_count(&a), 3 /* three keyframes */);
    }
}
