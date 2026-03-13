// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Vertex color export: per-vertex RGBA/grayscale buffers, AO baking, and encoding.

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Supported vertex-color storage formats.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexColorFormat {
    /// Four channels, 8 bits each.
    Rgba8,
    /// Three channels, 32-bit float each.
    RgbF32,
    /// Single luminance channel, 8 bits.
    Grayscale8,
}

// ── Structs ───────────────────────────────────────────────────────────────────

/// A buffer of per-vertex RGBA colors (stored as f32 in [0, 1]).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexColorBuffer {
    /// RGBA values in linear [0.0, 1.0] space, one entry per vertex.
    pub colors: Vec<[f32; 4]>,
    /// Preferred export format.
    pub format: VertexColorFormat,
}

/// Configuration options for vertex-color export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexColorExportConfig {
    /// Target encoding format.
    pub format: VertexColorFormat,
    /// Gamma value applied during export (1.0 = linear, 2.2 = sRGB).
    pub gamma: f32,
    /// Whether to clamp values to [0, 1] before encoding.
    pub clamp_before_encode: bool,
}

// ── Constructor functions ─────────────────────────────────────────────────────

/// Create a default vertex-color export configuration (RGBA8, gamma 2.2).
#[allow(dead_code)]
pub fn default_vertex_color_config() -> VertexColorExportConfig {
    VertexColorExportConfig {
        format: VertexColorFormat::Rgba8,
        gamma: 2.2,
        clamp_before_encode: true,
    }
}

/// Create a new vertex-color buffer with `count` vertices, all set to opaque white.
#[allow(dead_code)]
pub fn new_vertex_color_buffer(count: usize, format: VertexColorFormat) -> VertexColorBuffer {
    VertexColorBuffer {
        colors: vec![[1.0, 1.0, 1.0, 1.0]; count],
        format,
    }
}

// ── Per-vertex accessors ──────────────────────────────────────────────────────

/// Set the color of vertex at `index`.  Does nothing if index is out of range.
#[allow(dead_code)]
pub fn set_vertex_color(buf: &mut VertexColorBuffer, index: usize, color: [f32; 4]) {
    if let Some(c) = buf.colors.get_mut(index) {
        *c = color;
    }
}

/// Get the color of vertex at `index`, or `[0,0,0,0]` if out of range.
#[allow(dead_code)]
pub fn get_vertex_color(buf: &VertexColorBuffer, index: usize) -> [f32; 4] {
    buf.colors.get(index).copied().unwrap_or([0.0; 4])
}

/// Fill every vertex in the buffer with the same color.
#[allow(dead_code)]
pub fn fill_uniform_color(buf: &mut VertexColorBuffer, color: [f32; 4]) {
    for c in buf.colors.iter_mut() {
        *c = color;
    }
}

// ── Encode / decode ───────────────────────────────────────────────────────────

/// Encode the buffer to a byte vector according to its format.
///
/// * `Rgba8`      → 4 bytes per vertex (R G B A)
/// * `RgbF32`     → 12 bytes per vertex (3 × little-endian f32)
/// * `Grayscale8` → 1 byte per vertex (luminance)
#[allow(dead_code)]
pub fn encode_to_bytes(buf: &VertexColorBuffer) -> Vec<u8> {
    match buf.format {
        VertexColorFormat::Rgba8 => {
            let mut out = Vec::with_capacity(buf.colors.len() * 4);
            for &[r, g, b, a] in &buf.colors {
                out.push(float_to_vertex_color(r));
                out.push(float_to_vertex_color(g));
                out.push(float_to_vertex_color(b));
                out.push(float_to_vertex_color(a));
            }
            out
        }
        VertexColorFormat::RgbF32 => {
            let mut out = Vec::with_capacity(buf.colors.len() * 12);
            for &[r, g, b, _a] in &buf.colors {
                out.extend_from_slice(&r.to_le_bytes());
                out.extend_from_slice(&g.to_le_bytes());
                out.extend_from_slice(&b.to_le_bytes());
            }
            out
        }
        VertexColorFormat::Grayscale8 => {
            let mut out = Vec::with_capacity(buf.colors.len());
            for &[r, g, b, _a] in &buf.colors {
                let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                out.push(float_to_vertex_color(lum));
            }
            out
        }
    }
}

/// Decode a byte slice back into a `VertexColorBuffer`.
///
/// Returns `None` if the byte count is inconsistent with `format`.
#[allow(dead_code)]
pub fn decode_from_bytes(bytes: &[u8], format: VertexColorFormat) -> Option<VertexColorBuffer> {
    match format {
        VertexColorFormat::Rgba8 => {
            if !bytes.len().is_multiple_of(4) {
                return None;
            }
            let colors = bytes
                .chunks_exact(4)
                .map(|c| {
                    [
                        vertex_color_to_float(c[0]),
                        vertex_color_to_float(c[1]),
                        vertex_color_to_float(c[2]),
                        vertex_color_to_float(c[3]),
                    ]
                })
                .collect();
            Some(VertexColorBuffer { colors, format })
        }
        VertexColorFormat::RgbF32 => {
            if !bytes.len().is_multiple_of(12) {
                return None;
            }
            let colors = bytes
                .chunks_exact(12)
                .map(|c| {
                    let r = f32::from_le_bytes(c[0..4].try_into().unwrap_or_default());
                    let g = f32::from_le_bytes(c[4..8].try_into().unwrap_or_default());
                    let b = f32::from_le_bytes(c[8..12].try_into().unwrap_or_default());
                    [r, g, b, 1.0]
                })
                .collect();
            Some(VertexColorBuffer { colors, format })
        }
        VertexColorFormat::Grayscale8 => {
            let colors = bytes
                .iter()
                .map(|&v| {
                    let f = vertex_color_to_float(v);
                    [f, f, f, 1.0]
                })
                .collect();
            Some(VertexColorBuffer { colors, format })
        }
    }
}

// ── Conversion helpers ────────────────────────────────────────────────────────

/// Convert an 8-bit color component to a linear f32 in [0, 1].
#[allow(dead_code)]
#[inline]
pub fn vertex_color_to_float(v: u8) -> f32 {
    v as f32 / 255.0
}

/// Convert a linear f32 in [0, 1] to an 8-bit color component, clamping the input.
#[allow(dead_code)]
#[inline]
pub fn float_to_vertex_color(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

// ── AO / blending / utilities ─────────────────────────────────────────────────

/// Compute simple hemisphere ambient occlusion from an array of per-vertex normals.
///
/// The "sky" direction is `[0, 1, 0]`.  AO ∈ [0, 1] where 1 is fully lit.
#[allow(dead_code)]
pub fn compute_ao_vertex_colors(normals: &[[f32; 3]]) -> VertexColorBuffer {
    let sky = [0.0_f32, 1.0, 0.0];
    let colors = normals
        .iter()
        .map(|n| {
            let dot = dot3(*n, sky).clamp(0.0, 1.0);
            // Mix a small ambient term to avoid pure black
            let ao = 0.1 + 0.9 * dot;
            [ao, ao, ao, 1.0]
        })
        .collect();
    VertexColorBuffer {
        colors,
        format: VertexColorFormat::Rgba8,
    }
}

/// Blend two vertex-color buffers by `t` (0 = fully `a`, 1 = fully `b`).
///
/// Panics if the buffers have different lengths.
#[allow(dead_code)]
pub fn blend_vertex_colors(
    a: &VertexColorBuffer,
    b: &VertexColorBuffer,
    t: f32,
) -> VertexColorBuffer {
    assert_eq!(
        a.colors.len(),
        b.colors.len(),
        "buffers must have equal length"
    );
    let t = t.clamp(0.0, 1.0);
    let colors = a
        .colors
        .iter()
        .zip(b.colors.iter())
        .map(|(&ca, &cb)| lerp4(ca, cb, t))
        .collect();
    VertexColorBuffer {
        colors,
        format: a.format,
    }
}

/// Return the number of vertices in the buffer.
#[allow(dead_code)]
pub fn vertex_color_count(buf: &VertexColorBuffer) -> usize {
    buf.colors.len()
}

/// Serialise the buffer to a simple CSV string (one vertex per line, R,G,B,A).
#[allow(dead_code)]
pub fn to_csv_string(buf: &VertexColorBuffer) -> String {
    let mut out = String::from("r,g,b,a\n");
    for &[r, g, b, a] in &buf.colors {
        out.push_str(&format!("{:.6},{:.6},{:.6},{:.6}\n", r, g, b, a));
    }
    out
}

/// Apply power-law gamma correction to all RGB channels in-place.
///
/// Use `gamma = 1.0 / 2.2` to convert linear → sRGB.
#[allow(dead_code)]
pub fn apply_gamma_correction(buf: &mut VertexColorBuffer, gamma: f32) {
    for c in buf.colors.iter_mut() {
        c[0] = c[0].max(0.0).powf(gamma);
        c[1] = c[1].max(0.0).powf(gamma);
        c[2] = c[2].max(0.0).powf(gamma);
        // alpha is unchanged
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn lerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_correct_values() {
        let cfg = default_vertex_color_config();
        assert_eq!(cfg.format, VertexColorFormat::Rgba8);
        assert!((cfg.gamma - 2.2).abs() < 1e-5);
        assert!(cfg.clamp_before_encode);
    }

    #[test]
    fn new_buffer_creates_white_vertices() {
        let buf = new_vertex_color_buffer(5, VertexColorFormat::Rgba8);
        assert_eq!(buf.colors.len(), 5);
        for c in &buf.colors {
            assert_eq!(*c, [1.0, 1.0, 1.0, 1.0]);
        }
    }

    #[test]
    fn set_and_get_vertex_color() {
        let mut buf = new_vertex_color_buffer(4, VertexColorFormat::Rgba8);
        set_vertex_color(&mut buf, 2, [0.5, 0.25, 0.75, 1.0]);
        let c = get_vertex_color(&buf, 2);
        assert!((c[0] - 0.5).abs() < 1e-6);
        assert!((c[1] - 0.25).abs() < 1e-6);
        assert!((c[2] - 0.75).abs() < 1e-6);
    }

    #[test]
    fn set_vertex_color_out_of_range_is_noop() {
        let mut buf = new_vertex_color_buffer(3, VertexColorFormat::Rgba8);
        set_vertex_color(&mut buf, 99, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(buf.colors.len(), 3);
    }

    #[test]
    fn get_vertex_color_out_of_range_returns_zero() {
        let buf = new_vertex_color_buffer(2, VertexColorFormat::Rgba8);
        assert_eq!(get_vertex_color(&buf, 99), [0.0; 4]);
    }

    #[test]
    fn fill_uniform_color_sets_all() {
        let mut buf = new_vertex_color_buffer(6, VertexColorFormat::Rgba8);
        fill_uniform_color(&mut buf, [0.1, 0.2, 0.3, 0.4]);
        for c in &buf.colors {
            assert!((c[0] - 0.1).abs() < 1e-6);
            assert!((c[1] - 0.2).abs() < 1e-6);
        }
    }

    #[test]
    fn encode_decode_rgba8_roundtrip() {
        let mut buf = new_vertex_color_buffer(3, VertexColorFormat::Rgba8);
        set_vertex_color(&mut buf, 0, [1.0, 0.0, 0.0, 1.0]);
        set_vertex_color(&mut buf, 1, [0.0, 1.0, 0.0, 1.0]);
        set_vertex_color(&mut buf, 2, [0.0, 0.0, 1.0, 1.0]);
        let bytes = encode_to_bytes(&buf);
        assert_eq!(bytes.len(), 12);
        let decoded = decode_from_bytes(&bytes, VertexColorFormat::Rgba8).expect("should succeed");
        assert_eq!(decoded.colors.len(), 3);
        // Red channel of first vertex should be ~1.0
        assert!((decoded.colors[0][0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn encode_decode_rgbf32_roundtrip() {
        let mut buf = new_vertex_color_buffer(2, VertexColorFormat::RgbF32);
        set_vertex_color(&mut buf, 0, [0.3, 0.6, 0.9, 1.0]);
        set_vertex_color(&mut buf, 1, [0.1, 0.2, 0.3, 1.0]);
        let bytes = encode_to_bytes(&buf);
        assert_eq!(bytes.len(), 24);
        let decoded = decode_from_bytes(&bytes, VertexColorFormat::RgbF32).expect("should succeed");
        assert!((decoded.colors[0][0] - 0.3).abs() < 1e-5);
        assert!((decoded.colors[1][2] - 0.3).abs() < 1e-5);
    }

    #[test]
    fn encode_decode_grayscale8_roundtrip() {
        let mut buf = new_vertex_color_buffer(2, VertexColorFormat::Grayscale8);
        // Fill with a mid-grey
        fill_uniform_color(&mut buf, [0.5, 0.5, 0.5, 1.0]);
        let bytes = encode_to_bytes(&buf);
        assert_eq!(bytes.len(), 2);
        let decoded = decode_from_bytes(&bytes, VertexColorFormat::Grayscale8).expect("should succeed");
        assert_eq!(decoded.colors.len(), 2);
    }

    #[test]
    fn decode_bad_length_returns_none() {
        // 3 bytes is not divisible by 4 (Rgba8 stride)
        let bytes = vec![0u8; 3];
        assert!(decode_from_bytes(&bytes, VertexColorFormat::Rgba8).is_none());
    }

    #[test]
    fn float_to_vertex_color_clamping() {
        assert_eq!(float_to_vertex_color(-0.5), 0);
        assert_eq!(float_to_vertex_color(1.5), 255);
        assert_eq!(float_to_vertex_color(0.0), 0);
        assert_eq!(float_to_vertex_color(1.0), 255);
    }

    #[test]
    fn vertex_color_to_float_bounds() {
        assert!((vertex_color_to_float(0) - 0.0).abs() < 1e-6);
        assert!((vertex_color_to_float(255) - 1.0).abs() < 0.005);
    }

    #[test]
    fn compute_ao_vertex_colors_up_facing() {
        // Normal pointing straight up → should have high AO value
        let normals = vec![[0.0_f32, 1.0, 0.0]];
        let buf = compute_ao_vertex_colors(&normals);
        assert_eq!(buf.colors.len(), 1);
        assert!(buf.colors[0][0] > 0.9, "up-facing normal should be bright");
    }

    #[test]
    fn compute_ao_vertex_colors_down_facing() {
        // Normal pointing straight down → minimum illumination from sky
        let normals = vec![[0.0_f32, -1.0, 0.0]];
        let buf = compute_ao_vertex_colors(&normals);
        assert!(buf.colors[0][0] < 0.2, "down-facing normal should be dim");
    }

    #[test]
    fn blend_vertex_colors_at_zero() {
        let a = new_vertex_color_buffer(2, VertexColorFormat::Rgba8);
        let mut b = new_vertex_color_buffer(2, VertexColorFormat::Rgba8);
        fill_uniform_color(&mut b, [0.0, 0.0, 0.0, 1.0]);
        let out = blend_vertex_colors(&a, &b, 0.0);
        // All white (from a)
        assert!((out.colors[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn blend_vertex_colors_at_one() {
        let a = new_vertex_color_buffer(2, VertexColorFormat::Rgba8);
        let mut b = new_vertex_color_buffer(2, VertexColorFormat::Rgba8);
        fill_uniform_color(&mut b, [0.0, 0.0, 0.0, 1.0]);
        let out = blend_vertex_colors(&a, &b, 1.0);
        // All black (from b)
        assert!((out.colors[0][0] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn vertex_color_count_correct() {
        let buf = new_vertex_color_buffer(7, VertexColorFormat::Rgba8);
        assert_eq!(vertex_color_count(&buf), 7);
    }

    #[test]
    fn to_csv_string_contains_header() {
        let buf = new_vertex_color_buffer(1, VertexColorFormat::Rgba8);
        let csv = to_csv_string(&buf);
        assert!(csv.starts_with("r,g,b,a"));
        assert!(csv.contains("1.000000"));
    }

    #[test]
    fn apply_gamma_correction_modifies_rgb() {
        let mut buf = new_vertex_color_buffer(1, VertexColorFormat::Rgba8);
        fill_uniform_color(&mut buf, [0.5, 0.5, 0.5, 1.0]);
        let alpha_before = buf.colors[0][3];
        apply_gamma_correction(&mut buf, 2.0);
        // 0.5^2 = 0.25
        assert!((buf.colors[0][0] - 0.25).abs() < 1e-5);
        // alpha unchanged
        assert!((buf.colors[0][3] - alpha_before).abs() < 1e-6);
    }
}
