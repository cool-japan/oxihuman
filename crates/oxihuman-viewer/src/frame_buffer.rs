// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Frame buffer management utilities.

/// Attachment type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttachmentType {
    Color,
    Depth,
    Stencil,
    DepthStencil,
}

/// Frame buffer attachment descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FrameBufferAttachment {
    pub attachment_type: AttachmentType,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

/// Frame buffer descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FrameBufferDesc {
    pub attachments: Vec<FrameBufferAttachment>,
    pub width: u32,
    pub height: u32,
    pub label: String,
}

/// Create a color attachment.
#[allow(dead_code)]
pub fn color_attachment(width: u32, height: u32) -> FrameBufferAttachment {
    FrameBufferAttachment {
        attachment_type: AttachmentType::Color,
        width,
        height,
        format: "RGBA8".to_string(),
    }
}

/// Create a depth attachment.
#[allow(dead_code)]
pub fn depth_attachment(width: u32, height: u32) -> FrameBufferAttachment {
    FrameBufferAttachment {
        attachment_type: AttachmentType::Depth,
        width,
        height,
        format: "D32F".to_string(),
    }
}

/// Create a frame buffer descriptor.
#[allow(dead_code)]
pub fn new_frame_buffer(width: u32, height: u32) -> FrameBufferDesc {
    FrameBufferDesc {
        attachments: vec![
            color_attachment(width, height),
            depth_attachment(width, height),
        ],
        width,
        height,
        label: "default_fb".to_string(),
    }
}

/// Add attachment.
#[allow(dead_code)]
pub fn add_attachment(fb: &mut FrameBufferDesc, att: FrameBufferAttachment) {
    fb.attachments.push(att);
}

/// Attachment count.
#[allow(dead_code)]
pub fn attachment_count(fb: &FrameBufferDesc) -> usize {
    fb.attachments.len()
}

/// Estimate memory in bytes.
#[allow(dead_code)]
pub fn frame_buffer_memory(fb: &FrameBufferDesc) -> u64 {
    fb.attachments.iter().map(|a| {
        let bpp: u64 = match a.attachment_type {
            AttachmentType::Color => 4,
            AttachmentType::Depth => 4,
            AttachmentType::Stencil => 1,
            AttachmentType::DepthStencil => 5,
        };
        a.width as u64 * a.height as u64 * bpp
    }).sum()
}

/// Resize frame buffer.
#[allow(dead_code)]
pub fn resize_frame_buffer(fb: &mut FrameBufferDesc, width: u32, height: u32) {
    fb.width = width;
    fb.height = height;
    for att in &mut fb.attachments {
        att.width = width;
        att.height = height;
    }
}

/// Has depth attachment?
#[allow(dead_code)]
pub fn has_depth(fb: &FrameBufferDesc) -> bool {
    fb.attachments.iter().any(|a| matches!(a.attachment_type, AttachmentType::Depth | AttachmentType::DepthStencil))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_frame_buffer() {
        let fb = new_frame_buffer(1920, 1080);
        assert_eq!(fb.width, 1920);
    }

    #[test]
    fn test_default_attachments() {
        let fb = new_frame_buffer(800, 600);
        assert_eq!(attachment_count(&fb), 2);
    }

    #[test]
    fn test_add_attachment() {
        let mut fb = new_frame_buffer(800, 600);
        add_attachment(&mut fb, color_attachment(800, 600));
        assert_eq!(attachment_count(&fb), 3);
    }

    #[test]
    fn test_memory() {
        let fb = new_frame_buffer(100, 100);
        let mem = frame_buffer_memory(&fb);
        assert_eq!(mem, 100 * 100 * 4 + 100 * 100 * 4);
    }

    #[test]
    fn test_resize() {
        let mut fb = new_frame_buffer(800, 600);
        resize_frame_buffer(&mut fb, 1920, 1080);
        assert_eq!(fb.width, 1920);
        assert_eq!(fb.attachments[0].width, 1920);
    }

    #[test]
    fn test_has_depth() {
        let fb = new_frame_buffer(800, 600);
        assert!(has_depth(&fb));
    }

    #[test]
    fn test_color_only() {
        let mut fb = FrameBufferDesc {
            attachments: vec![color_attachment(800, 600)],
            width: 800,
            height: 600,
            label: "test".to_string(),
        };
        assert!(!has_depth(&fb));
        add_attachment(&mut fb, depth_attachment(800, 600));
        assert!(has_depth(&fb));
    }

    #[test]
    fn test_color_attachment_format() {
        let a = color_attachment(100, 100);
        assert_eq!(a.format, "RGBA8");
    }

    #[test]
    fn test_depth_attachment_format() {
        let a = depth_attachment(100, 100);
        assert_eq!(a.format, "D32F");
    }

    #[test]
    fn test_label() {
        let fb = new_frame_buffer(800, 600);
        assert_eq!(fb.label, "default_fb");
    }
}
