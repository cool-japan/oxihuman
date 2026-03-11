#![allow(dead_code)]

/// Descriptor for a render pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPassDesc {
    name: String,
    color_attachments: Vec<String>,
    depth_attachment: Option<String>,
    clear_values: [f32; 4],
}

#[allow(dead_code)]
pub fn new_pass_desc(name: &str) -> RenderPassDesc {
    RenderPassDesc { name: name.to_string(), color_attachments: Vec::new(), depth_attachment: None, clear_values: [0.0; 4] }
}

#[allow(dead_code)]
pub fn desc_color_attachment(desc: &mut RenderPassDesc, name: &str) {
    desc.color_attachments.push(name.to_string());
}

#[allow(dead_code)]
pub fn desc_depth_attachment(desc: &mut RenderPassDesc, name: &str) {
    desc.depth_attachment = Some(name.to_string());
}

#[allow(dead_code)]
pub fn desc_clear_values(desc: &RenderPassDesc) -> [f32; 4] { desc.clear_values }

#[allow(dead_code)]
pub fn desc_name(desc: &RenderPassDesc) -> &str { &desc.name }

#[allow(dead_code)]
pub fn desc_to_json(desc: &RenderPassDesc) -> String {
    format!("{{\"name\":\"{}\",\"color_attachments\":{},\"has_depth\":{}}}", desc.name, desc.color_attachments.len(), desc.depth_attachment.is_some())
}

#[allow(dead_code)]
pub fn desc_attachment_count(desc: &RenderPassDesc) -> usize {
    desc.color_attachments.len() + if desc.depth_attachment.is_some() { 1 } else { 0 }
}

#[allow(dead_code)]
pub fn desc_validate(desc: &RenderPassDesc) -> bool {
    !desc.name.is_empty() && !desc.color_attachments.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert_eq!(desc_name(&new_pass_desc("main")), "main"); }
    #[test] fn test_color_attachment() {
        let mut d = new_pass_desc("p");
        desc_color_attachment(&mut d, "color0");
        assert_eq!(desc_attachment_count(&d), 1);
    }
    #[test] fn test_depth_attachment() {
        let mut d = new_pass_desc("p");
        desc_color_attachment(&mut d, "c");
        desc_depth_attachment(&mut d, "depth");
        assert_eq!(desc_attachment_count(&d), 2);
    }
    #[test] fn test_clear_values() { assert!((desc_clear_values(&new_pass_desc("p"))[0]).abs() < 1e-6); }
    #[test] fn test_validate_valid() {
        let mut d = new_pass_desc("p");
        desc_color_attachment(&mut d, "c");
        assert!(desc_validate(&d));
    }
    #[test] fn test_validate_no_color() { assert!(!desc_validate(&new_pass_desc("p"))); }
    #[test] fn test_validate_no_name() {
        let mut d = new_pass_desc("");
        desc_color_attachment(&mut d, "c");
        assert!(!desc_validate(&d));
    }
    #[test] fn test_to_json() { assert!(desc_to_json(&new_pass_desc("p")).contains("name")); }
    #[test] fn test_attachment_count_empty() { assert_eq!(desc_attachment_count(&new_pass_desc("p")), 0); }
    #[test] fn test_multiple_color() {
        let mut d = new_pass_desc("p");
        desc_color_attachment(&mut d, "c0");
        desc_color_attachment(&mut d, "c1");
        assert_eq!(desc_attachment_count(&d), 2);
    }
}
