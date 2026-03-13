// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Web Animations API JSON export stub.

/// A single keyframe in Web Animations API format.
pub struct WebAnimKeyframe {
    pub offset: f32,
    pub property: String,
    pub value: String,
    pub easing: String,
}

/// Web Animations API animation options.
pub struct WebAnimOptions {
    pub duration: u32,
    pub iterations: f32,
    pub fill: String,
    pub easing: String,
    pub delay: i32,
}

impl Default for WebAnimOptions {
    fn default() -> Self {
        Self {
            duration: 1000,
            iterations: 1.0,
            fill: "none".to_string(),
            easing: "ease".to_string(),
            delay: 0,
        }
    }
}

/// A Web Animations API animation export.
pub struct WebAnimExport {
    pub keyframes: Vec<WebAnimKeyframe>,
    pub options: WebAnimOptions,
    pub target_selector: String,
}

/// Create a new Web Animations API export.
pub fn new_web_anim_export(target: &str, options: WebAnimOptions) -> WebAnimExport {
    WebAnimExport {
        keyframes: Vec::new(),
        options,
        target_selector: target.to_string(),
    }
}

/// Add a keyframe.
pub fn add_web_anim_keyframe(
    exp: &mut WebAnimExport,
    offset: f32,
    property: &str,
    value: &str,
    easing: &str,
) {
    exp.keyframes.push(WebAnimKeyframe {
        offset: offset.clamp(0.0, 1.0),
        property: property.to_string(),
        value: value.to_string(),
        easing: easing.to_string(),
    });
}

/// Keyframe count.
pub fn web_anim_keyframe_count(exp: &WebAnimExport) -> usize {
    exp.keyframes.len()
}

/// Render a simple JSON representation.
pub fn render_web_anim_json(exp: &WebAnimExport) -> String {
    format!(
        r#"{{"target":"{}","duration":{},"keyframes":{}}}"#,
        exp.target_selector,
        exp.options.duration,
        exp.keyframes.len()
    )
}

/// Validate export (target non-empty, duration > 0).
pub fn validate_web_anim(exp: &WebAnimExport) -> bool {
    !exp.target_selector.is_empty() && exp.options.duration > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_web_anim_export("#el", WebAnimOptions::default());
        assert_eq!(web_anim_keyframe_count(&exp), 0 /* empty */);
    }

    #[test]
    fn add_keyframe_increments() {
        let mut exp = new_web_anim_export("#el", WebAnimOptions::default());
        add_web_anim_keyframe(&mut exp, 0.0, "opacity", "0", "ease");
        assert_eq!(web_anim_keyframe_count(&exp), 1 /* one keyframe */);
    }

    #[test]
    fn offset_clamped() {
        let mut exp = new_web_anim_export("#el", WebAnimOptions::default());
        add_web_anim_keyframe(&mut exp, 1.5, "x", "10px", "ease");
        assert!(exp.keyframes[0].offset <= 1.0 /* clamped */);
    }

    #[test]
    fn validate_valid() {
        let exp = new_web_anim_export("#div", WebAnimOptions::default());
        assert!(validate_web_anim(&exp) /* valid */);
    }

    #[test]
    fn validate_empty_target_fails() {
        let exp = new_web_anim_export("", WebAnimOptions::default());
        assert!(!validate_web_anim(&exp) /* invalid */);
    }

    #[test]
    fn render_contains_target() {
        let exp = new_web_anim_export("#my-el", WebAnimOptions::default());
        let json = render_web_anim_json(&exp);
        assert!(json.contains("my-el") /* target in output */);
    }

    #[test]
    fn render_contains_duration() {
        let opt = WebAnimOptions {
            duration: 2500,
            ..Default::default()
        };
        let exp = new_web_anim_export("#x", opt);
        let json = render_web_anim_json(&exp);
        assert!(json.contains("2500") /* duration */);
    }

    #[test]
    fn default_options_reasonable() {
        let opt = WebAnimOptions::default();
        assert!(opt.duration > 0 /* positive duration */);
        assert!(opt.iterations >= 1.0 /* at least once */);
    }

    #[test]
    fn multiple_keyframes() {
        let mut exp = new_web_anim_export("#a", WebAnimOptions::default());
        add_web_anim_keyframe(&mut exp, 0.0, "opacity", "0", "ease");
        add_web_anim_keyframe(&mut exp, 0.5, "opacity", "0.5", "linear");
        add_web_anim_keyframe(&mut exp, 1.0, "opacity", "1", "ease");
        assert_eq!(web_anim_keyframe_count(&exp), 3 /* three keyframes */);
    }
}
