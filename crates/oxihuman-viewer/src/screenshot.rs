// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Screenshot capture with CPU-side software rasterization.
//!
//! Implements basic triangle rasterization with z-buffering and Phong shading,
//! producing an RGBA pixel buffer that can be exported to PPM or TGA format.
//! No GPU dependency — pure Rust.

use crate::lighting_presets::LightingPreset;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Captures a rendered frame to an image buffer using software rasterization.
pub struct ScreenshotCapture {
    width: u32,
    height: u32,
}

/// Raw RGBA image buffer.
#[derive(Debug, Clone)]
pub struct ImageBuffer {
    /// Pixel data in row-major RGBA order.
    pub data: Vec<u8>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Channels per pixel (always 4 for RGBA).
    pub channels: u8,
}

// ── Matrix / vector helpers (f64) ─────────────────────────────────────────────

fn mat4_mul_vec4(m: &[f64; 16], v: [f64; 4]) -> [f64; 4] {
    // Column-major: m[col*4 + row]
    [
        m[0] * v[0] + m[4] * v[1] + m[8] * v[2] + m[12] * v[3],
        m[1] * v[0] + m[5] * v[1] + m[9] * v[2] + m[13] * v[3],
        m[2] * v[0] + m[6] * v[1] + m[10] * v[2] + m[14] * v[3],
        m[3] * v[0] + m[7] * v[1] + m[11] * v[2] + m[15] * v[3],
    ]
}

fn mat4_multiply(a: &[f64; 16], b: &[f64; 16]) -> [f64; 16] {
    let mut out = [0.0f64; 16];
    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[k * 4 + row] * b[col * 4 + k];
            }
            out[col * 4 + row] = sum;
        }
    }
    out
}

fn normalize3(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-15 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn reflect3(incident: [f64; 3], normal: [f64; 3]) -> [f64; 3] {
    let d = 2.0 * dot3(incident, normal);
    [
        incident[0] - d * normal[0],
        incident[1] - d * normal[1],
        incident[2] - d * normal[2],
    ]
}

#[allow(dead_code)]
fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

// ── Rasterizer internals ──────────────────────────────────────────────────────

/// A vertex after projection to clip space, plus world-space data for shading.
#[derive(Clone, Copy)]
struct ProjectedVertex {
    /// Screen-space x, y and clip-space z (depth).
    screen_x: f64,
    screen_y: f64,
    depth: f64,
    /// World-space position for lighting calculations.
    world_pos: [f64; 3],
    /// World-space normal.
    normal: [f64; 3],
}

/// Compute barycentric coordinates for point (px, py) w.r.t. triangle (v0, v1, v2).
/// Returns `None` if the triangle is degenerate or the point is outside.
fn barycentric(
    px: f64,
    py: f64,
    v0: (f64, f64),
    v1: (f64, f64),
    v2: (f64, f64),
) -> Option<(f64, f64, f64)> {
    let denom = (v1.1 - v2.1) * (v0.0 - v2.0) + (v2.0 - v1.0) * (v0.1 - v2.1);
    if denom.abs() < 1e-15 {
        return None;
    }
    let inv_denom = 1.0 / denom;
    let w0 = ((v1.1 - v2.1) * (px - v2.0) + (v2.0 - v1.0) * (py - v2.1)) * inv_denom;
    let w1 = ((v2.1 - v0.1) * (px - v2.0) + (v0.0 - v2.0) * (py - v2.1)) * inv_denom;
    let w2 = 1.0 - w0 - w1;

    // Small epsilon for edge inclusion
    const EPS: f64 = -1e-4;
    if w0 >= EPS && w1 >= EPS && w2 >= EPS {
        Some((w0, w1, w2))
    } else {
        None
    }
}

/// Interpolate a [f64; 3] attribute using barycentric weights.
fn interp3(a: [f64; 3], b: [f64; 3], c: [f64; 3], w0: f64, w1: f64, w2: f64) -> [f64; 3] {
    [
        a[0] * w0 + b[0] * w1 + c[0] * w2,
        a[1] * w0 + b[1] * w1 + c[1] * w2,
        a[2] * w0 + b[2] * w1 + c[2] * w2,
    ]
}

/// Phong shade a fragment using the lighting preset.
fn phong_shade(
    world_pos: [f64; 3],
    normal: [f64; 3],
    camera_pos: [f64; 3],
    lighting: &LightingPreset,
) -> [f64; 3] {
    let n = normalize3(normal);
    let view_dir = normalize3(sub3(camera_pos, world_pos));

    // Material properties (neutral grey diffuse + white specular)
    let kd = [0.7, 0.7, 0.7]; // diffuse reflectance
    let ks = [0.3, 0.3, 0.3]; // specular reflectance
    let shininess = 32.0;

    // Start with ambient
    let mut color = [
        lighting.ambient[0] * kd[0],
        lighting.ambient[1] * kd[1],
        lighting.ambient[2] * kd[2],
    ];

    for light in &lighting.lights {
        let (to_light, light_color) = light.evaluate_at(world_pos);

        // Diffuse (Lambertian)
        let n_dot_l = dot3(n, to_light).max(0.0);
        let diffuse = [
            light_color[0] * kd[0] * n_dot_l,
            light_color[1] * kd[1] * n_dot_l,
            light_color[2] * kd[2] * n_dot_l,
        ];

        // Specular (Blinn-Phong)
        let half_vec = normalize3([
            to_light[0] + view_dir[0],
            to_light[1] + view_dir[1],
            to_light[2] + view_dir[2],
        ]);
        let n_dot_h = dot3(n, half_vec).max(0.0);
        let spec_factor = n_dot_h.powf(shininess);
        let specular = [
            light_color[0] * ks[0] * spec_factor,
            light_color[1] * ks[1] * spec_factor,
            light_color[2] * ks[2] * spec_factor,
        ];

        color[0] += diffuse[0] + specular[0];
        color[1] += diffuse[1] + specular[1];
        color[2] += diffuse[2] + specular[2];
    }

    // Apply exposure
    let exp = lighting.exposure;
    [
        (color[0] * exp).clamp(0.0, 1.0),
        (color[1] * exp).clamp(0.0, 1.0),
        (color[2] * exp).clamp(0.0, 1.0),
    ]
}

/// Extract camera world position from the view matrix (inverse translation).
///
/// For a standard look-at view matrix, the camera position can be recovered
/// from the last column of the inverse. We approximate by using the transpose
/// of the rotation part and the translation entries.
fn camera_pos_from_view(view: &[f64; 16]) -> [f64; 3] {
    // view = [R | t] column-major => R^T * (-t) gives camera position
    // R columns are at indices: col0=[0,1,2], col1=[4,5,6], col2=[8,9,10]
    // t is at indices [12,13,14]
    let tx = view[12];
    let ty = view[13];
    let tz = view[14];
    [
        -(view[0] * tx + view[1] * ty + view[2] * tz),
        -(view[4] * tx + view[5] * ty + view[6] * tz),
        -(view[8] * tx + view[9] * ty + view[10] * tz),
    ]
}

// ── ScreenshotCapture ─────────────────────────────────────────────────────────

impl ScreenshotCapture {
    /// Create a new capture context for the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Resize the capture target.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Render a mesh with Phong shading into an RGBA image buffer.
    ///
    /// # Arguments
    /// * `positions` — world-space vertex positions
    /// * `triangles` — index triples into `positions` / `normals`
    /// * `normals` — per-vertex normals (same length as `positions`)
    /// * `camera_view` — 4x4 column-major view matrix
    /// * `camera_proj` — 4x4 column-major projection matrix
    /// * `lighting` — lighting configuration
    pub fn capture_software_render(
        &self,
        positions: &[[f64; 3]],
        triangles: &[[usize; 3]],
        normals: &[[f64; 3]],
        camera_view: &[f64; 16],
        camera_proj: &[f64; 16],
        lighting: &LightingPreset,
    ) -> anyhow::Result<ImageBuffer> {
        let w = self.width as usize;
        let h = self.height as usize;

        if w == 0 || h == 0 {
            anyhow::bail!("Screenshot dimensions must be non-zero");
        }

        // Allocate framebuffer and z-buffer
        let pixel_count = w * h;
        let mut pixels = vec![0u8; pixel_count * 4]; // RGBA
        let mut z_buffer = vec![f64::INFINITY; pixel_count];

        // Fill background with a dark grey
        for i in 0..pixel_count {
            let base = i * 4;
            pixels[base] = 38; // R
            pixels[base + 1] = 38; // G
            pixels[base + 2] = 46; // B
            pixels[base + 3] = 255; // A
        }

        // Combined view-projection matrix
        let mvp = mat4_multiply(camera_proj, camera_view);

        // Camera world position for specular calculation
        let camera_pos = camera_pos_from_view(camera_view);

        // Project all vertices
        let projected: Vec<Option<ProjectedVertex>> = positions
            .iter()
            .zip(normals.iter())
            .map(|(pos, nrm)| {
                let clip = mat4_mul_vec4(&mvp, [pos[0], pos[1], pos[2], 1.0]);
                // Perspective divide
                if clip[3].abs() < 1e-15 {
                    return None;
                }
                let inv_w = 1.0 / clip[3];
                let ndc_x = clip[0] * inv_w;
                let ndc_y = clip[1] * inv_w;
                let ndc_z = clip[2] * inv_w;

                // NDC to screen space
                let sx = (ndc_x * 0.5 + 0.5) * w as f64;
                let sy = (1.0 - (ndc_y * 0.5 + 0.5)) * h as f64; // flip Y

                Some(ProjectedVertex {
                    screen_x: sx,
                    screen_y: sy,
                    depth: ndc_z,
                    world_pos: *pos,
                    normal: *nrm,
                })
            })
            .collect();

        // Rasterize each triangle
        for tri in triangles {
            let v0_opt = if tri[0] < projected.len() {
                projected[tri[0]]
            } else {
                None
            };
            let v1_opt = if tri[1] < projected.len() {
                projected[tri[1]]
            } else {
                None
            };
            let v2_opt = if tri[2] < projected.len() {
                projected[tri[2]]
            } else {
                None
            };

            let (v0, v1, v2) = match (v0_opt, v1_opt, v2_opt) {
                (Some(a), Some(b), Some(c)) => (a, b, c),
                _ => continue, // Skip triangles with invalid vertices
            };

            // Near-plane clip: skip if any vertex is behind camera
            if v0.depth < -1.0 || v1.depth < -1.0 || v2.depth < -1.0 {
                continue;
            }
            if v0.depth > 1.0 && v1.depth > 1.0 && v2.depth > 1.0 {
                continue;
            }

            // Back-face culling using screen-space cross product
            let e01 = [v1.screen_x - v0.screen_x, v1.screen_y - v0.screen_y];
            let e02 = [v2.screen_x - v0.screen_x, v2.screen_y - v0.screen_y];
            let cross_z = e01[0] * e02[1] - e01[1] * e02[0];
            if cross_z < 0.0 {
                continue; // Back-facing
            }

            // Bounding box of the triangle in screen space
            let min_x = v0
                .screen_x
                .min(v1.screen_x)
                .min(v2.screen_x)
                .floor()
                .max(0.0) as usize;
            let max_x = v0
                .screen_x
                .max(v1.screen_x)
                .max(v2.screen_x)
                .ceil()
                .min(w as f64 - 1.0) as usize;
            let min_y = v0
                .screen_y
                .min(v1.screen_y)
                .min(v2.screen_y)
                .floor()
                .max(0.0) as usize;
            let max_y = v0
                .screen_y
                .max(v1.screen_y)
                .max(v2.screen_y)
                .ceil()
                .min(h as f64 - 1.0) as usize;

            for py in min_y..=max_y {
                for px in min_x..=max_x {
                    let fx = px as f64 + 0.5;
                    let fy = py as f64 + 0.5;

                    let bary = match barycentric(
                        fx,
                        fy,
                        (v0.screen_x, v0.screen_y),
                        (v1.screen_x, v1.screen_y),
                        (v2.screen_x, v2.screen_y),
                    ) {
                        Some(b) => b,
                        None => continue,
                    };

                    let (w0, w1, w2) = bary;

                    // Interpolate depth
                    let depth = v0.depth * w0 + v1.depth * w1 + v2.depth * w2;

                    // Z-buffer test
                    let idx = py * w + px;
                    if depth >= z_buffer[idx] {
                        continue;
                    }
                    z_buffer[idx] = depth;

                    // Interpolate world position and normal
                    let frag_pos = interp3(v0.world_pos, v1.world_pos, v2.world_pos, w0, w1, w2);
                    let frag_normal =
                        normalize3(interp3(v0.normal, v1.normal, v2.normal, w0, w1, w2));

                    // Phong shading
                    let color = phong_shade(frag_pos, frag_normal, camera_pos, lighting);

                    // Write pixel (linear to sRGB approximation via gamma 2.2)
                    let base = idx * 4;
                    pixels[base] = linear_to_srgb_byte(color[0]);
                    pixels[base + 1] = linear_to_srgb_byte(color[1]);
                    pixels[base + 2] = linear_to_srgb_byte(color[2]);
                    pixels[base + 3] = 255;
                }
            }
        }

        Ok(ImageBuffer {
            data: pixels,
            width: self.width,
            height: self.height,
            channels: 4,
        })
    }
}

/// Convert linear [0,1] to sRGB byte [0,255].
fn linear_to_srgb_byte(v: f64) -> u8 {
    let srgb = if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (srgb * 255.0).clamp(0.0, 255.0).round() as u8
}

// ── ImageBuffer ───────────────────────────────────────────────────────────────

impl ImageBuffer {
    /// Export the image in PPM (P6 binary) format.
    ///
    /// PPM is a simple uncompressed format supported by many viewers.
    /// Alpha channel is discarded.
    pub fn to_ppm(&self) -> anyhow::Result<Vec<u8>> {
        if self.width == 0 || self.height == 0 {
            anyhow::bail!("Cannot export zero-dimension image to PPM");
        }
        let header = format!("P6\n{} {}\n255\n", self.width, self.height);
        let pixel_count = self.width as usize * self.height as usize;
        let mut out = Vec::with_capacity(header.len() + pixel_count * 3);
        out.extend_from_slice(header.as_bytes());

        let ch = self.channels as usize;
        for i in 0..pixel_count {
            let base = i * ch;
            let r = self.data.get(base).copied().unwrap_or(0);
            let g = self.data.get(base + 1).copied().unwrap_or(0);
            let b = self.data.get(base + 2).copied().unwrap_or(0);
            out.push(r);
            out.push(g);
            out.push(b);
        }

        Ok(out)
    }

    /// Export the image in TGA (Targa) format — uncompressed, 32-bit RGBA.
    pub fn to_tga(&self) -> anyhow::Result<Vec<u8>> {
        if self.width == 0 || self.height == 0 {
            anyhow::bail!("Cannot export zero-dimension image to TGA");
        }
        if self.width > 65535 || self.height > 65535 {
            anyhow::bail!("TGA dimensions exceed 16-bit limit");
        }

        let pixel_count = self.width as usize * self.height as usize;
        let ch = self.channels as usize;

        // TGA header (18 bytes)
        let mut out = Vec::with_capacity(18 + pixel_count * 4);

        // ID length
        out.push(0);
        // Color map type (none)
        out.push(0);
        // Image type (2 = uncompressed true-color)
        out.push(2);
        // Color map specification (5 bytes of zeros)
        out.extend_from_slice(&[0, 0, 0, 0, 0]);
        // X origin (2 bytes)
        out.extend_from_slice(&[0, 0]);
        // Y origin (2 bytes)
        out.extend_from_slice(&[0, 0]);
        // Width (little-endian u16)
        out.push((self.width & 0xFF) as u8);
        out.push(((self.width >> 8) & 0xFF) as u8);
        // Height (little-endian u16)
        out.push((self.height & 0xFF) as u8);
        out.push(((self.height >> 8) & 0xFF) as u8);
        // Pixel depth (32 bits)
        out.push(32);
        // Image descriptor (bit 5 = top-to-bottom, bits 0-3 = alpha bits = 8)
        out.push(0x28);

        // Pixel data — TGA stores BGRA, top-to-bottom
        for i in 0..pixel_count {
            let base = i * ch;
            let r = self.data.get(base).copied().unwrap_or(0);
            let g = self.data.get(base + 1).copied().unwrap_or(0);
            let b = self.data.get(base + 2).copied().unwrap_or(0);
            let a = if ch >= 4 {
                self.data.get(base + 3).copied().unwrap_or(255)
            } else {
                255
            };
            out.push(b); // B
            out.push(g); // G
            out.push(r); // R
            out.push(a); // A
        }

        Ok(out)
    }

    /// Get the RGBA pixel at (x, y). Returns `None` if out of bounds.
    pub fn pixel_at(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let ch = self.channels as usize;
        let idx = (y as usize * self.width as usize + x as usize) * ch;
        let r = self.data.get(idx).copied()?;
        let g = self.data.get(idx + 1).copied()?;
        let b = self.data.get(idx + 2).copied()?;
        let a = if ch >= 4 {
            self.data.get(idx + 3).copied().unwrap_or(255)
        } else {
            255
        };
        Some([r, g, b, a])
    }

    /// Total byte size of the pixel data.
    pub fn byte_size(&self) -> usize {
        self.data.len()
    }

    /// Create a new empty image buffer filled with a solid colour.
    pub fn solid_color(width: u32, height: u32, color: [u8; 4]) -> Self {
        let pixel_count = width as usize * height as usize;
        let mut data = Vec::with_capacity(pixel_count * 4);
        for _ in 0..pixel_count {
            data.push(color[0]);
            data.push(color[1]);
            data.push(color[2]);
            data.push(color[3]);
        }
        Self {
            data,
            width,
            height,
            channels: 4,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lighting_presets::LightingPreset;

    fn identity_matrix() -> [f64; 16] {
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
    }

    /// Simple orthographic-like projection for testing.
    fn test_proj() -> [f64; 16] {
        // Scale x,y by 1.0, z maps [-1,1] to [0,1]
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
        ]
    }

    #[test]
    fn screenshot_capture_new_dimensions() {
        let cap = ScreenshotCapture::new(800, 600);
        assert_eq!(cap.width, 800);
        assert_eq!(cap.height, 600);
    }

    #[test]
    fn screenshot_capture_resize() {
        let mut cap = ScreenshotCapture::new(100, 100);
        cap.resize(200, 150);
        assert_eq!(cap.width, 200);
        assert_eq!(cap.height, 150);
    }

    #[test]
    fn capture_zero_dimension_error() {
        let cap = ScreenshotCapture::new(0, 100);
        let result = cap.capture_software_render(
            &[],
            &[],
            &[],
            &identity_matrix(),
            &test_proj(),
            &LightingPreset::studio(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn capture_empty_mesh_produces_background() {
        let cap = ScreenshotCapture::new(4, 4);
        let buf = cap
            .capture_software_render(
                &[],
                &[],
                &[],
                &identity_matrix(),
                &test_proj(),
                &LightingPreset::studio(),
            )
            .expect("should succeed with empty mesh");
        assert_eq!(buf.width, 4);
        assert_eq!(buf.height, 4);
        assert_eq!(buf.channels, 4);
        assert_eq!(buf.data.len(), 4 * 4 * 4);
        // All pixels should be the background colour
        let px = buf.pixel_at(0, 0).expect("pixel should exist");
        assert_eq!(px[3], 255); // alpha
    }

    #[test]
    fn capture_single_triangle() {
        // A triangle covering part of the viewport
        let positions = vec![[0.0, 0.5, 0.0], [-0.5, -0.5, 0.0], [0.5, -0.5, 0.0]];
        let normals = vec![[0.0, 0.0, 1.0]; 3];
        let triangles = vec![[0, 1, 2]];

        let cap = ScreenshotCapture::new(16, 16);
        let buf = cap
            .capture_software_render(
                &positions,
                &triangles,
                &normals,
                &identity_matrix(),
                &test_proj(),
                &LightingPreset::medical(), // Even lighting
            )
            .expect("should render triangle");

        // The center pixel should be shaded (not background)
        let center = buf.pixel_at(8, 8);
        assert!(center.is_some());
    }

    #[test]
    fn image_buffer_pixel_at_out_of_bounds() {
        let buf = ImageBuffer::solid_color(4, 4, [128, 128, 128, 255]);
        assert!(buf.pixel_at(5, 0).is_none());
        assert!(buf.pixel_at(0, 5).is_none());
    }

    #[test]
    fn image_buffer_pixel_at_valid() {
        let buf = ImageBuffer::solid_color(2, 2, [10, 20, 30, 40]);
        let px = buf.pixel_at(0, 0).expect("should return pixel");
        assert_eq!(px, [10, 20, 30, 40]);
    }

    #[test]
    fn image_buffer_solid_color_size() {
        let buf = ImageBuffer::solid_color(8, 8, [0, 0, 0, 255]);
        assert_eq!(buf.byte_size(), 8 * 8 * 4);
    }

    #[test]
    fn to_ppm_header_format() {
        let buf = ImageBuffer::solid_color(2, 3, [255, 0, 0, 255]);
        let ppm = buf.to_ppm().expect("PPM export should succeed");
        let header = String::from_utf8_lossy(&ppm[..10]);
        assert!(header.starts_with("P6\n"));
        // Total size: header + 2*3*3 RGB bytes
        let expected_pixel_bytes = 2 * 3 * 3;
        let header_end = ppm
            .windows(1)
            .enumerate()
            .filter(|(_, b)| b[0] == b'\n')
            .nth(2)
            .map(|(i, _)| i + 1)
            .unwrap_or(0);
        assert_eq!(ppm.len() - header_end, expected_pixel_bytes);
    }

    #[test]
    fn to_ppm_zero_dimension_error() {
        let buf = ImageBuffer {
            data: vec![],
            width: 0,
            height: 0,
            channels: 4,
        };
        assert!(buf.to_ppm().is_err());
    }

    #[test]
    fn to_tga_header_size() {
        let buf = ImageBuffer::solid_color(4, 4, [100, 200, 50, 255]);
        let tga = buf.to_tga().expect("TGA export should succeed");
        // TGA header is 18 bytes + 4*4*4 pixel bytes = 18 + 64 = 82
        assert_eq!(tga.len(), 18 + 4 * 4 * 4);
    }

    #[test]
    fn to_tga_header_fields() {
        let buf = ImageBuffer::solid_color(10, 20, [0, 0, 0, 255]);
        let tga = buf.to_tga().expect("TGA export should succeed");
        // Image type at offset 2 should be 2 (uncompressed true-color)
        assert_eq!(tga[2], 2);
        // Width at offset 12-13 (little-endian)
        assert_eq!(tga[12], 10);
        assert_eq!(tga[13], 0);
        // Height at offset 14-15
        assert_eq!(tga[14], 20);
        assert_eq!(tga[15], 0);
        // Pixel depth at offset 16
        assert_eq!(tga[16], 32);
    }

    #[test]
    fn to_tga_bgra_order() {
        let buf = ImageBuffer::solid_color(1, 1, [10, 20, 30, 40]);
        let tga = buf.to_tga().expect("TGA export should succeed");
        // Pixel data starts at offset 18, in BGRA order
        assert_eq!(tga[18], 30); // B
        assert_eq!(tga[19], 20); // G
        assert_eq!(tga[20], 10); // R
        assert_eq!(tga[21], 40); // A
    }

    #[test]
    fn to_tga_zero_dimension_error() {
        let buf = ImageBuffer {
            data: vec![],
            width: 0,
            height: 0,
            channels: 4,
        };
        assert!(buf.to_tga().is_err());
    }

    #[test]
    fn linear_to_srgb_byte_black() {
        assert_eq!(linear_to_srgb_byte(0.0), 0);
    }

    #[test]
    fn linear_to_srgb_byte_white() {
        assert_eq!(linear_to_srgb_byte(1.0), 255);
    }

    #[test]
    fn linear_to_srgb_byte_mid() {
        let val = linear_to_srgb_byte(0.5);
        // sRGB of linear 0.5 should be roughly 188
        assert!(val > 150 && val < 220, "got {val}");
    }

    #[test]
    fn mat4_multiply_identity() {
        let id = identity_matrix();
        let result = mat4_multiply(&id, &id);
        for (i, &val) in result.iter().enumerate() {
            let expected = if i % 5 == 0 { 1.0 } else { 0.0 };
            assert!(
                (val - expected).abs() < 1e-12,
                "index {i}: expected {expected}, got {}",
                val
            );
        }
    }

    #[test]
    fn mat4_mul_vec4_identity() {
        let id = identity_matrix();
        let v = [1.0, 2.0, 3.0, 1.0];
        let result = mat4_mul_vec4(&id, v);
        assert!((result[0] - 1.0).abs() < 1e-12);
        assert!((result[1] - 2.0).abs() < 1e-12);
        assert!((result[2] - 3.0).abs() < 1e-12);
        assert!((result[3] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn barycentric_center_of_triangle() {
        // Equilateral-ish triangle
        let result = barycentric(0.33, 0.33, (0.0, 0.0), (1.0, 0.0), (0.0, 1.0));
        assert!(result.is_some());
        let (w0, w1, w2) = result.expect("should be inside");
        assert!(w0 > 0.0 && w1 > 0.0 && w2 > 0.0);
        assert!(((w0 + w1 + w2) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn barycentric_outside_triangle() {
        let result = barycentric(2.0, 2.0, (0.0, 0.0), (1.0, 0.0), (0.0, 1.0));
        assert!(result.is_none());
    }

    #[test]
    fn camera_pos_from_identity_view() {
        let view = identity_matrix();
        let pos = camera_pos_from_view(&view);
        // Identity view matrix means camera at origin
        assert!((pos[0]).abs() < 1e-10);
        assert!((pos[1]).abs() < 1e-10);
        assert!((pos[2]).abs() < 1e-10);
    }
}
