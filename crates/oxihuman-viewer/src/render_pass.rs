// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Render pass descriptors for multi-pass rendering.
//!
//! Covers shadow map, G-buffer, lighting accumulation, post-process, and UI passes.

// ── Attachment format enums ───────────────────────────────────────────────────

/// Pixel format for a colour attachment.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AttachmentFormat {
    Rgba8Unorm,
    Rgba16Float,
    R11G11B10Float,
    Bgra8UnormSrgb,
}

/// Pixel format for a depth (and optional stencil) attachment.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum DepthFormat {
    Depth32Float,
    Depth24Stencil8,
    Depth16Unorm,
}

// ── Load / Store ops ──────────────────────────────────────────────────────────

/// What to do with an attachment's content at the start of a pass.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum LoadOp {
    /// Clear to the given RGBA value.
    Clear([f32; 4]),
    /// Preserve the current content.
    Load,
    /// Don't care — contents are undefined.
    DontCare,
}

/// What to do with an attachment's content at the end of a pass.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum StoreOp {
    /// Write results back to the attachment.
    Store,
    /// Discard the results (e.g. depth not needed after the pass).
    Discard,
}

// ── Attachment descriptors ────────────────────────────────────────────────────

/// Descriptor for a single colour attachment.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorAttachment {
    pub format: AttachmentFormat,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    /// 1 (no MSAA) or 4 (4× MSAA).
    pub sample_count: u32,
}

/// Descriptor for the depth (and optional stencil) attachment.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DepthAttachment {
    pub format: DepthFormat,
    pub depth_load_op: LoadOp,
    pub depth_store_op: StoreOp,
    pub stencil_load_op: LoadOp,
    pub stencil_store_op: StoreOp,
}

// ── RenderPassDescriptor ──────────────────────────────────────────────────────

/// Full descriptor for one render pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPassDescriptor {
    pub label: String,
    pub color_attachments: Vec<ColorAttachment>,
    pub depth_attachment: Option<DepthAttachment>,
    /// Sample count that applies to the whole pass.
    pub sample_count: u32,
}

impl RenderPassDescriptor {
    /// Depth-only shadow-map pass.
    #[allow(dead_code)]
    pub fn shadow_pass() -> Self {
        RenderPassDescriptor {
            label: "shadow_pass".to_string(),
            color_attachments: vec![],
            depth_attachment: Some(DepthAttachment {
                format: DepthFormat::Depth32Float,
                depth_load_op: LoadOp::Clear([1.0, 0.0, 0.0, 0.0]),
                depth_store_op: StoreOp::Store,
                stencil_load_op: LoadOp::DontCare,
                stencil_store_op: StoreOp::Discard,
            }),
            sample_count: 1,
        }
    }

    /// G-buffer pass: albedo + normal + material + depth.
    #[allow(dead_code)]
    pub fn gbuffer_pass() -> Self {
        RenderPassDescriptor {
            label: "gbuffer_pass".to_string(),
            color_attachments: vec![
                ColorAttachment {
                    format: AttachmentFormat::Rgba8Unorm,
                    load_op: LoadOp::Clear([0.0, 0.0, 0.0, 1.0]),
                    store_op: StoreOp::Store,
                    sample_count: 1,
                },
                ColorAttachment {
                    format: AttachmentFormat::Rgba16Float,
                    load_op: LoadOp::Clear([0.0, 0.0, 1.0, 0.0]),
                    store_op: StoreOp::Store,
                    sample_count: 1,
                },
                ColorAttachment {
                    format: AttachmentFormat::Rgba8Unorm,
                    load_op: LoadOp::Clear([0.0, 0.0, 0.0, 0.0]),
                    store_op: StoreOp::Store,
                    sample_count: 1,
                },
            ],
            depth_attachment: Some(DepthAttachment {
                format: DepthFormat::Depth24Stencil8,
                depth_load_op: LoadOp::Clear([1.0, 0.0, 0.0, 0.0]),
                depth_store_op: StoreOp::Store,
                stencil_load_op: LoadOp::Clear([0.0, 0.0, 0.0, 0.0]),
                stencil_store_op: StoreOp::Discard,
            }),
            sample_count: 1,
        }
    }

    /// HDR lighting accumulation pass.
    #[allow(dead_code)]
    pub fn lighting_pass() -> Self {
        RenderPassDescriptor {
            label: "lighting_pass".to_string(),
            color_attachments: vec![ColorAttachment {
                format: AttachmentFormat::Rgba16Float,
                load_op: LoadOp::Clear([0.0, 0.0, 0.0, 1.0]),
                store_op: StoreOp::Store,
                sample_count: 1,
            }],
            depth_attachment: Some(DepthAttachment {
                format: DepthFormat::Depth32Float,
                depth_load_op: LoadOp::Load,
                depth_store_op: StoreOp::Discard,
                stencil_load_op: LoadOp::DontCare,
                stencil_store_op: StoreOp::Discard,
            }),
            sample_count: 1,
        }
    }

    /// Tone-mapping / bloom post-process pass — no depth attachment.
    #[allow(dead_code)]
    pub fn post_process_pass() -> Self {
        RenderPassDescriptor {
            label: "post_process_pass".to_string(),
            color_attachments: vec![ColorAttachment {
                format: AttachmentFormat::Bgra8UnormSrgb,
                load_op: LoadOp::DontCare,
                store_op: StoreOp::Store,
                sample_count: 1,
            }],
            depth_attachment: None,
            sample_count: 1,
        }
    }

    /// Forward UI pass — no depth write.
    #[allow(dead_code)]
    pub fn ui_pass() -> Self {
        RenderPassDescriptor {
            label: "ui_pass".to_string(),
            color_attachments: vec![ColorAttachment {
                format: AttachmentFormat::Bgra8UnormSrgb,
                load_op: LoadOp::Load,
                store_op: StoreOp::Store,
                sample_count: 1,
            }],
            depth_attachment: None,
            sample_count: 1,
        }
    }
}

// ── Free functions ────────────────────────────────────────────────────────────

/// Count of colour attachments plus one if a depth attachment is present.
#[allow(dead_code)]
pub fn total_attachment_count(desc: &RenderPassDescriptor) -> usize {
    desc.color_attachments.len()
        + if desc.depth_attachment.is_some() {
            1
        } else {
            0
        }
}

/// `true` if the pass has no colour attachments (depth-only).
#[allow(dead_code)]
pub fn is_depth_only(desc: &RenderPassDescriptor) -> bool {
    desc.color_attachments.is_empty() && desc.depth_attachment.is_some()
}

/// Bytes per pixel for a colour attachment format.
#[allow(dead_code)]
pub fn attachment_format_bytes(fmt: &AttachmentFormat) -> u32 {
    match fmt {
        AttachmentFormat::Rgba8Unorm => 4,
        AttachmentFormat::Rgba16Float => 8,
        AttachmentFormat::R11G11B10Float => 4,
        AttachmentFormat::Bgra8UnormSrgb => 4,
    }
}

/// Bytes per pixel for a depth format.
#[allow(dead_code)]
pub fn depth_format_bytes(fmt: &DepthFormat) -> u32 {
    match fmt {
        DepthFormat::Depth32Float => 4,
        DepthFormat::Depth24Stencil8 => 4,
        DepthFormat::Depth16Unorm => 2,
    }
}

/// Return a human-readable one-line summary of a render pass.
#[allow(dead_code)]
pub fn render_pass_summary(desc: &RenderPassDescriptor) -> String {
    format!(
        "RenderPass '{}': {} color, depth={}, samples={}",
        desc.label,
        desc.color_attachments.len(),
        desc.depth_attachment.is_some(),
        desc.sample_count
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // 1. shadow_pass is depth-only
    #[test]
    fn shadow_pass_is_depth_only() {
        let p = RenderPassDescriptor::shadow_pass();
        assert!(is_depth_only(&p));
    }

    // 2. gbuffer_pass has exactly 3 colour attachments
    #[test]
    fn gbuffer_pass_three_color_attachments() {
        let p = RenderPassDescriptor::gbuffer_pass();
        assert_eq!(p.color_attachments.len(), 3);
    }

    // 3. gbuffer_pass has a depth attachment
    #[test]
    fn gbuffer_pass_has_depth() {
        let p = RenderPassDescriptor::gbuffer_pass();
        assert!(p.depth_attachment.is_some());
    }

    // 4. lighting_pass uses HDR (Rgba16Float) format
    #[test]
    fn lighting_pass_hdr_format() {
        let p = RenderPassDescriptor::lighting_pass();
        assert_eq!(p.color_attachments.len(), 1);
        assert_eq!(p.color_attachments[0].format, AttachmentFormat::Rgba16Float);
    }

    // 5. post_process_pass has no depth
    #[test]
    fn post_process_pass_no_depth() {
        let p = RenderPassDescriptor::post_process_pass();
        assert!(p.depth_attachment.is_none());
    }

    // 6. attachment_format_bytes Rgba8 = 4
    #[test]
    fn format_bytes_rgba8() {
        assert_eq!(attachment_format_bytes(&AttachmentFormat::Rgba8Unorm), 4);
    }

    // 7. attachment_format_bytes Rgba16Float = 8
    #[test]
    fn format_bytes_rgba16float() {
        assert_eq!(attachment_format_bytes(&AttachmentFormat::Rgba16Float), 8);
    }

    // 8. depth_format_bytes Depth32Float = 4
    #[test]
    fn depth_bytes_depth32() {
        assert_eq!(depth_format_bytes(&DepthFormat::Depth32Float), 4);
    }

    // 9. depth_format_bytes Depth16Unorm = 2
    #[test]
    fn depth_bytes_depth16() {
        assert_eq!(depth_format_bytes(&DepthFormat::Depth16Unorm), 2);
    }

    // 10. total_attachment_count shadow = 1 (depth only)
    #[test]
    fn total_count_shadow() {
        let p = RenderPassDescriptor::shadow_pass();
        assert_eq!(total_attachment_count(&p), 1);
    }

    // 11. total_attachment_count gbuffer = 4 (3 color + depth)
    #[test]
    fn total_count_gbuffer() {
        let p = RenderPassDescriptor::gbuffer_pass();
        assert_eq!(total_attachment_count(&p), 4);
    }

    // 12. render_pass_summary is non-empty
    #[test]
    fn summary_non_empty() {
        let p = RenderPassDescriptor::lighting_pass();
        let s = render_pass_summary(&p);
        assert!(!s.is_empty());
        assert!(s.contains("lighting_pass"));
    }

    // 13. all preset passes have valid sample_count (1 or 4)
    #[test]
    fn all_passes_valid_sample_count() {
        let passes = vec![
            RenderPassDescriptor::shadow_pass(),
            RenderPassDescriptor::gbuffer_pass(),
            RenderPassDescriptor::lighting_pass(),
            RenderPassDescriptor::post_process_pass(),
            RenderPassDescriptor::ui_pass(),
        ];
        for p in &passes {
            assert!(
                p.sample_count == 1 || p.sample_count == 4,
                "invalid sample_count in {}",
                p.label
            );
        }
    }

    // 14. ui_pass has no depth
    #[test]
    fn ui_pass_no_depth() {
        let p = RenderPassDescriptor::ui_pass();
        assert!(p.depth_attachment.is_none());
    }

    // 15. R11G11B10Float bytes = 4
    #[test]
    fn format_bytes_r11g11b10() {
        assert_eq!(
            attachment_format_bytes(&AttachmentFormat::R11G11B10Float),
            4
        );
    }
}

// ── RenderPassStage / RenderPassConfig / RenderPassList ───────────────────────

/// High-level render pass stage classification.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum RenderPassStage {
    Depth,
    Opaque,
    Transparent,
    PostProcess,
    UI,
}

/// Simple per-pass configuration toggle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPassConfig {
    pub stage: RenderPassStage,
    pub enabled: bool,
    pub clear_depth: bool,
    pub clear_color: bool,
}

/// Ordered list of render pass configs.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct RenderPassList {
    passes: Vec<RenderPassConfig>,
}

#[allow(dead_code)]
pub fn default_render_pass_config(stage: RenderPassStage) -> RenderPassConfig {
    RenderPassConfig {
        stage,
        enabled: true,
        clear_depth: false,
        clear_color: false,
    }
}

#[allow(dead_code)]
pub fn new_render_pass_list() -> RenderPassList {
    RenderPassList::default()
}

#[allow(dead_code)]
pub fn rpl_add_pass(list: &mut RenderPassList, config: RenderPassConfig) {
    list.passes.push(config);
}

#[allow(dead_code)]
pub fn rpl_remove_pass(list: &mut RenderPassList, index: usize) {
    if index < list.passes.len() {
        list.passes.remove(index);
    }
}

#[allow(dead_code)]
pub fn rpl_count(list: &RenderPassList) -> usize {
    list.passes.len()
}

#[allow(dead_code)]
pub fn rpl_get(list: &RenderPassList, index: usize) -> Option<&RenderPassConfig> {
    list.passes.get(index)
}

#[allow(dead_code)]
pub fn rpl_enabled_count(list: &RenderPassList) -> usize {
    list.passes.iter().filter(|p| p.enabled).count()
}

#[allow(dead_code)]
pub fn rpl_stage_name(stage: &RenderPassStage) -> &'static str {
    match stage {
        RenderPassStage::Depth => "depth",
        RenderPassStage::Opaque => "opaque",
        RenderPassStage::Transparent => "transparent",
        RenderPassStage::PostProcess => "post_process",
        RenderPassStage::UI => "ui",
    }
}

#[allow(dead_code)]
pub fn rpl_to_json(list: &RenderPassList) -> String {
    format!(
        r#"{{"pass_count":{},"enabled_count":{}}}"#,
        list.passes.len(),
        rpl_enabled_count(list)
    )
}

#[cfg(test)]
mod rpl_tests {
    use super::*;

    #[test]
    fn test_new_list_empty() {
        let l = new_render_pass_list();
        assert_eq!(rpl_count(&l), 0);
    }

    #[test]
    fn test_add_pass() {
        let mut l = new_render_pass_list();
        rpl_add_pass(&mut l, default_render_pass_config(RenderPassStage::Opaque));
        assert_eq!(rpl_count(&l), 1);
    }

    #[test]
    fn test_remove_pass() {
        let mut l = new_render_pass_list();
        rpl_add_pass(&mut l, default_render_pass_config(RenderPassStage::Depth));
        rpl_remove_pass(&mut l, 0);
        assert_eq!(rpl_count(&l), 0);
    }

    #[test]
    fn test_get_pass() {
        let mut l = new_render_pass_list();
        rpl_add_pass(&mut l, default_render_pass_config(RenderPassStage::UI));
        let p = rpl_get(&l, 0);
        assert!(p.is_some());
        assert_eq!(p.expect("should succeed").stage, RenderPassStage::UI);
    }

    #[test]
    fn test_enabled_count() {
        let mut l = new_render_pass_list();
        rpl_add_pass(&mut l, default_render_pass_config(RenderPassStage::Opaque));
        let mut cfg = default_render_pass_config(RenderPassStage::Transparent);
        cfg.enabled = false;
        rpl_add_pass(&mut l, cfg);
        assert_eq!(rpl_enabled_count(&l), 1);
    }

    #[test]
    fn test_stage_name() {
        assert_eq!(rpl_stage_name(&RenderPassStage::Depth), "depth");
        assert_eq!(
            rpl_stage_name(&RenderPassStage::PostProcess),
            "post_process"
        );
        assert_eq!(rpl_stage_name(&RenderPassStage::UI), "ui");
    }

    #[test]
    fn test_to_json() {
        let l = new_render_pass_list();
        let j = rpl_to_json(&l);
        assert!(j.contains("pass_count"));
        assert!(j.contains("enabled_count"));
    }

    #[test]
    fn test_default_config_enabled() {
        let cfg = default_render_pass_config(RenderPassStage::Opaque);
        assert!(cfg.enabled);
        assert_eq!(cfg.stage, RenderPassStage::Opaque);
    }
}
