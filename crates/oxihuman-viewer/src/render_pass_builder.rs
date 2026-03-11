#![allow(dead_code)]
//! Render pass builder: constructs render pass descriptions.

/// A built render pass description.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct PassDesc {
    color_target: String,
    depth_target: String,
    clear_color: [f32; 4],
}

/// Builder for render passes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPassBuilder {
    desc: PassDesc,
}

/// Create a new pass builder with defaults.
#[allow(dead_code)]
pub fn new_pass_builder() -> RenderPassBuilder {
    RenderPassBuilder {
        desc: PassDesc {
            color_target: String::new(),
            depth_target: String::new(),
            clear_color: [0.0, 0.0, 0.0, 1.0],
        },
    }
}

/// Set the color target name.
#[allow(dead_code)]
pub fn set_color_target(builder: &mut RenderPassBuilder, target: &str) {
    builder.desc.color_target = target.to_string();
}

/// Set the depth target name.
#[allow(dead_code)]
pub fn set_depth_target(builder: &mut RenderPassBuilder, target: &str) {
    builder.desc.depth_target = target.to_string();
}

/// Set the clear color.
#[allow(dead_code)]
pub fn set_clear_color(builder: &mut RenderPassBuilder, color: [f32; 4]) {
    builder.desc.clear_color = color;
}

/// Build the pass, returning a JSON-like description string.
#[allow(dead_code)]
pub fn build_pass(builder: &RenderPassBuilder) -> String {
    format!(
        "{{\"color_target\":\"{}\",\"depth_target\":\"{}\",\"clear_color\":[{},{},{},{}]}}",
        builder.desc.color_target,
        builder.desc.depth_target,
        builder.desc.clear_color[0],
        builder.desc.clear_color[1],
        builder.desc.clear_color[2],
        builder.desc.clear_color[3]
    )
}

/// Return the color target.
#[allow(dead_code)]
pub fn pass_color_target(builder: &RenderPassBuilder) -> &str {
    &builder.desc.color_target
}

/// Return the depth target.
#[allow(dead_code)]
pub fn pass_depth_target(builder: &RenderPassBuilder) -> &str {
    &builder.desc.depth_target
}

/// Serialize builder state to JSON-like string.
#[allow(dead_code)]
pub fn builder_to_json(builder: &RenderPassBuilder) -> String {
    build_pass(builder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_builder() {
        let b = new_pass_builder();
        assert_eq!(pass_color_target(&b), "");
    }

    #[test]
    fn test_set_color_target() {
        let mut b = new_pass_builder();
        set_color_target(&mut b, "main_color");
        assert_eq!(pass_color_target(&b), "main_color");
    }

    #[test]
    fn test_set_depth_target() {
        let mut b = new_pass_builder();
        set_depth_target(&mut b, "depth_buffer");
        assert_eq!(pass_depth_target(&b), "depth_buffer");
    }

    #[test]
    fn test_set_clear_color() {
        let mut b = new_pass_builder();
        set_clear_color(&mut b, [1.0, 0.0, 0.0, 1.0]);
        let json = build_pass(&b);
        assert!(json.contains("1"));
    }

    #[test]
    fn test_build_pass() {
        let mut b = new_pass_builder();
        set_color_target(&mut b, "c");
        set_depth_target(&mut b, "d");
        let json = build_pass(&b);
        assert!(json.contains("\"color_target\":\"c\""));
        assert!(json.contains("\"depth_target\":\"d\""));
    }

    #[test]
    fn test_builder_to_json() {
        let b = new_pass_builder();
        let json = builder_to_json(&b);
        assert!(json.contains("clear_color"));
    }

    #[test]
    fn test_default_clear_color() {
        let b = new_pass_builder();
        let json = build_pass(&b);
        assert!(json.contains("0"));
    }

    #[test]
    fn test_depth_target_empty() {
        let b = new_pass_builder();
        assert_eq!(pass_depth_target(&b), "");
    }

    #[test]
    fn test_override_color_target() {
        let mut b = new_pass_builder();
        set_color_target(&mut b, "first");
        set_color_target(&mut b, "second");
        assert_eq!(pass_color_target(&b), "second");
    }

    #[test]
    fn test_full_setup() {
        let mut b = new_pass_builder();
        set_color_target(&mut b, "hdr_rt");
        set_depth_target(&mut b, "depth_rt");
        set_clear_color(&mut b, [0.1, 0.2, 0.3, 1.0]);
        let json = build_pass(&b);
        assert!(json.contains("hdr_rt"));
        assert!(json.contains("depth_rt"));
    }
}
