// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ShaderToy GLSL stub export.

/// A ShaderToy channel input type.
#[derive(Clone, Copy, PartialEq)]
pub enum ShaderToyChannel {
    Texture2D,
    CubeMap,
    Buffer,
    Keyboard,
}

/// A ShaderToy export stub.
pub struct ShaderToyExport {
    pub image_shader: String,
    pub common_shader: String,
    pub channels: Vec<ShaderToyChannel>,
    pub title: String,
}

/// Create a new ShaderToy export with a default shader.
pub fn new_shader_toy_export(title: &str) -> ShaderToyExport {
    ShaderToyExport {
        image_shader:
            "void mainImage(out vec4 fragColor,in vec2 fragCoord){\n  fragColor=vec4(1.0);\n}"
                .to_string(),
        common_shader: String::new(),
        channels: Vec::new(),
        title: title.to_string(),
    }
}

/// Set the image shader source.
pub fn set_image_shader(exp: &mut ShaderToyExport, src: &str) {
    exp.image_shader = src.to_string();
}

/// Set the common shader source.
pub fn set_common_shader(exp: &mut ShaderToyExport, src: &str) {
    exp.common_shader = src.to_string();
}

/// Add a channel input.
pub fn add_shader_toy_channel(exp: &mut ShaderToyExport, ch: ShaderToyChannel) {
    exp.channels.push(ch);
}

/// Channel count.
pub fn shader_toy_channel_count(exp: &ShaderToyExport) -> usize {
    exp.channels.len()
}

/// Validate (image shader non-empty).
pub fn validate_shader_toy(exp: &ShaderToyExport) -> bool {
    !exp.image_shader.is_empty() && !exp.title.is_empty()
}

/// Render a simple text representation.
pub fn render_shader_toy_stub(exp: &ShaderToyExport) -> String {
    format!(
        "// ShaderToy: {}\n// Channels: {}\n// Common:\n{}\n// Image:\n{}",
        exp.title,
        exp.channels.len(),
        exp.common_shader,
        exp.image_shader
    )
}

/// Check if the image shader contains a specific identifier.
pub fn shader_contains(exp: &ShaderToyExport, identifier: &str) -> bool {
    exp.image_shader.contains(identifier) || exp.common_shader.contains(identifier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_has_default_shader() {
        let exp = new_shader_toy_export("test");
        assert!(!exp.image_shader.is_empty() /* has shader */);
    }

    #[test]
    fn validate_valid_export() {
        let exp = new_shader_toy_export("valid");
        assert!(validate_shader_toy(&exp) /* valid */);
    }

    #[test]
    fn validate_empty_title_fails() {
        let exp = new_shader_toy_export("");
        assert!(!validate_shader_toy(&exp) /* invalid */);
    }

    #[test]
    fn set_image_shader_updates() {
        let mut exp = new_shader_toy_export("s");
        set_image_shader(
            &mut exp,
            "void mainImage(out vec4 c,in vec2 u){c=vec4(u,0,1);}",
        );
        assert!(exp.image_shader.contains("mainImage") /* updated */);
    }

    #[test]
    fn add_channel_increments() {
        let mut exp = new_shader_toy_export("ch");
        add_shader_toy_channel(&mut exp, ShaderToyChannel::Texture2D);
        assert_eq!(shader_toy_channel_count(&exp), 1 /* one channel */);
    }

    #[test]
    fn render_contains_title() {
        let exp = new_shader_toy_export("MyShader");
        let out = render_shader_toy_stub(&exp);
        assert!(out.contains("MyShader") /* title */);
    }

    #[test]
    fn shader_contains_identifier() {
        let exp = new_shader_toy_export("x");
        assert!(shader_contains(&exp, "mainImage") /* identifier present */);
    }

    #[test]
    fn shader_contains_not_present() {
        let exp = new_shader_toy_export("x");
        assert!(!shader_contains(&exp, "nothere") /* not found */);
    }

    #[test]
    fn common_shader_empty_by_default() {
        let exp = new_shader_toy_export("c");
        assert!(exp.common_shader.is_empty() /* empty */);
    }
}
