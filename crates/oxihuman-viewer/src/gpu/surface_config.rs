// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! WebGPU surface configuration, depth texture, and MSAA resolve texture setup.
//!
//! The primary entry points are:
//! * [`configure_surface`] — initial surface setup after adapter negotiation.
//! * [`resize_surface`]    — call whenever the window is resized.
//! * [`DepthTexture`]      — wrapper around a depth/stencil texture and its view.
//! * [`MsaaTexture`]       — 4× MSAA resolve target.

use anyhow::{anyhow, Context, Result};

// ── Constants ─────────────────────────────────────────────────────────────────

/// The MSAA sample count used throughout the render pipeline.
pub const MSAA_SAMPLE_COUNT: u32 = 4;

/// The depth/stencil format used for both scene and shadow depth textures.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

// ── SurfaceConfig ─────────────────────────────────────────────────────────────

/// Resolved surface configuration mirroring the values passed to
/// [`wgpu::Surface::configure`].
#[derive(Debug, Clone)]
pub struct SurfaceConfig {
    /// Framebuffer width in pixels.
    pub width: u32,
    /// Framebuffer height in pixels.
    pub height: u32,
    /// Colour format of the swap-chain images.
    pub format: wgpu::TextureFormat,
    /// Presentation mode (Fifo = vsync, Immediate = uncapped, etc.).
    pub present_mode: wgpu::PresentMode,
    /// Alpha composition mode used by the OS compositor.
    pub alpha_mode: wgpu::CompositeAlphaMode,
}

impl SurfaceConfig {
    /// Aspect ratio `width / height`, clamped to avoid division by zero.
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            1.0
        } else {
            self.width as f32 / self.height as f32
        }
    }
}

// ── configure_surface ─────────────────────────────────────────────────────────

/// Configure a [`wgpu::Surface`] for rendering, selecting the best available
/// texture format and presentation mode.
///
/// # Errors
/// Returns an error if the surface reports no supported formats for the adapter.
pub fn configure_surface(
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
    width: u32,
    height: u32,
) -> Result<SurfaceConfig> {
    let caps = surface.get_capabilities(adapter);

    // Prefer sRGB formats; fall back to the first available format.
    let format = caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .or_else(|| caps.formats.first().copied())
        .ok_or_else(|| anyhow!("surface has no supported texture formats"))?;

    // Prefer Fifo (vsync); fall back to Immediate.
    let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Fifo) {
        wgpu::PresentMode::Fifo
    } else {
        caps.present_modes
            .first()
            .copied()
            .unwrap_or(wgpu::PresentMode::Fifo)
    };

    let alpha_mode = caps
        .alpha_modes
        .first()
        .copied()
        .unwrap_or(wgpu::CompositeAlphaMode::Auto);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: width.max(1),
        height: height.max(1),
        present_mode,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(device, &config);

    Ok(SurfaceConfig {
        width: config.width,
        height: config.height,
        format: config.format,
        present_mode: config.present_mode,
        alpha_mode: config.alpha_mode,
    })
}

// ── resize_surface ────────────────────────────────────────────────────────────

/// Reconfigure the surface after a window resize.
///
/// Updates `config` in-place to reflect the new dimensions.  The depth and
/// MSAA textures must be recreated separately with the new size.
///
/// # Errors
/// Returns an error if `new_w` or `new_h` is zero (degenerate framebuffer).
pub fn resize_surface(
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    config: &mut SurfaceConfig,
    new_w: u32,
    new_h: u32,
) -> Result<()> {
    if new_w == 0 || new_h == 0 {
        return Err(anyhow!(
            "degenerate framebuffer size {}×{} — resize ignored",
            new_w,
            new_h
        ));
    }

    config.width = new_w;
    config.height = new_h;

    let wgpu_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: config.format,
        width: new_w,
        height: new_h,
        present_mode: config.present_mode,
        alpha_mode: config.alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(device, &wgpu_config);
    Ok(())
}

// ── DepthTexture ──────────────────────────────────────────────────────────────

/// Owned depth/stencil texture together with its [`wgpu::TextureView`].
///
/// Use [`DepthTexture::create`] to allocate a new depth texture sized to the
/// current framebuffer.
pub struct DepthTexture {
    /// Underlying GPU texture.
    pub texture: wgpu::Texture,
    /// View used when binding this texture as a depth attachment.
    pub view: wgpu::TextureView,
    /// Texture format (always [`DEPTH_FORMAT`]).
    pub format: wgpu::TextureFormat,
}

impl DepthTexture {
    /// Allocate a new depth texture of the given dimensions.
    ///
    /// The texture is created with:
    /// * format [`DEPTH_FORMAT`] (Depth32Float)
    /// * usage `RENDER_ATTACHMENT | TEXTURE_BINDING`
    /// * sample_count `sample_count` (use 1 for shadow maps, 4 for MSAA scene)
    pub fn create(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        sample_count: u32,
        label: Option<&str>,
    ) -> Result<Self> {
        let label = label.unwrap_or("depth_texture");
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{}_view", label)),
            format: Some(DEPTH_FORMAT),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::DepthOnly,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
            usage: None,
        });

        Ok(Self {
            texture,
            view,
            format: DEPTH_FORMAT,
        })
    }

    /// Convenience: create a scene depth texture (4× MSAA).
    pub fn create_scene(device: &wgpu::Device, width: u32, height: u32) -> Result<Self> {
        Self::create(
            device,
            width,
            height,
            MSAA_SAMPLE_COUNT,
            Some("scene_depth"),
        )
        .context("creating scene depth texture")
    }

    /// Convenience: create a shadow map depth texture (no MSAA, custom size).
    pub fn create_shadow_map(device: &wgpu::Device, size: u32) -> Result<Self> {
        Self::create(device, size, size, 1, Some("shadow_depth"))
            .context("creating shadow map depth texture")
    }
}

// ── MsaaTexture ───────────────────────────────────────────────────────────────

/// 4× MSAA multisampled colour texture used as the render target for the
/// scene pass.  The resolved output is written to the swap-chain image via a
/// resolve attachment.
pub struct MsaaTexture {
    /// The multisampled texture (sample_count = 4).
    pub texture: wgpu::Texture,
    /// View for use as a colour attachment.
    pub view: wgpu::TextureView,
    /// Colour format (matches the swap-chain format).
    pub format: wgpu::TextureFormat,
}

impl MsaaTexture {
    /// Allocate a new 4× MSAA colour texture.
    pub fn create(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Result<Self> {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa_texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: MSAA_SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("msaa_texture_view"),
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            format,
        })
    }

    /// Recreate after a window resize, consuming and replacing `self`.
    pub fn resize(self, device: &wgpu::Device, width: u32, height: u32) -> Result<Self> {
        Self::create(device, width, height, self.format).context("resizing MSAA texture")
    }
}
