// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! HDR rendering pipeline with exposure control and auto-exposure.

// ── HdrConfig ─────────────────────────────────────────────────────────────────

/// Configuration for the HDR pipeline.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HdrConfig {
    /// Enable automatic exposure adjustment.
    pub auto_exposure: bool,
    /// Target average scene luminance for auto-exposure.
    pub target_luminance: f32,
    /// Rate at which exposure adapts per second (EV/s).
    pub exposure_speed: f32,
    /// Minimum allowed exposure value (EV).
    pub min_exposure: f32,
    /// Maximum allowed exposure value (EV).
    pub max_exposure: f32,
}

// ── HdrBuffer ─────────────────────────────────────────────────────────────────

/// A floating-point HDR image buffer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HdrBuffer {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// RGBA pixels in row-major order.
    pub pixels: Vec<[f32; 4]>,
}

// ── HdrPipelineState ──────────────────────────────────────────────────────────

/// Runtime state of the HDR pipeline.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HdrPipelineState {
    /// Pipeline configuration.
    pub config: HdrConfig,
    /// Current exposure multiplier.
    pub current_exposure: f32,
    /// Smoothed average luminance of the scene.
    pub avg_luminance: f32,
}

// ── HdrResult ─────────────────────────────────────────────────────────────────

/// Result of processing one HDR frame.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HdrResult {
    /// Exposure-adjusted output buffer.
    pub output: HdrBuffer,
    /// The exposure value actually applied.
    pub applied_exposure: f32,
    /// Luminance value of the brightest pixel in the output.
    pub histogram_peak: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a sensible default [`HdrConfig`].
#[allow(dead_code)]
pub fn default_hdr_config() -> HdrConfig {
    HdrConfig {
        auto_exposure: true,
        target_luminance: 0.18,
        exposure_speed: 1.0,
        min_exposure: 0.1,
        max_exposure: 10.0,
    }
}

/// Create a new, black [`HdrBuffer`] with the given dimensions.
#[allow(dead_code)]
pub fn new_hdr_buffer(w: u32, h: u32) -> HdrBuffer {
    HdrBuffer {
        width: w,
        height: h,
        pixels: vec![[0.0, 0.0, 0.0, 1.0]; (w * h) as usize],
    }
}

/// Create a new [`HdrPipelineState`] from a config with neutral exposure.
#[allow(dead_code)]
pub fn new_hdr_pipeline(cfg: HdrConfig) -> HdrPipelineState {
    HdrPipelineState {
        config: cfg,
        current_exposure: 1.0,
        avg_luminance: 0.18,
    }
}

/// Compute the average luminance of `buf` using standard luminance coefficients.
#[allow(dead_code)]
pub fn compute_avg_luminance(buf: &HdrBuffer) -> f32 {
    if buf.pixels.is_empty() {
        return 0.0;
    }
    let sum: f32 = buf
        .pixels
        .iter()
        .map(|p| 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2])
        .sum();
    sum / buf.pixels.len() as f32
}

/// Update `state.current_exposure` towards the value required to match
/// `target_luminance`, moving at `exposure_speed` per second.
#[allow(dead_code)]
pub fn update_exposure(state: &mut HdrPipelineState, scene_luminance: f32, dt: f32) {
    // Smooth the scene luminance.
    let alpha = (state.config.exposure_speed * dt).clamp(0.0, 1.0);
    state.avg_luminance = state.avg_luminance + alpha * (scene_luminance - state.avg_luminance);

    if state.config.auto_exposure && state.avg_luminance > 1e-6 {
        let desired = state.config.target_luminance / state.avg_luminance;
        let target_ev = desired.clamp(state.config.min_exposure, state.config.max_exposure);
        state.current_exposure =
            state.current_exposure + alpha * (target_ev - state.current_exposure);
    }
}

/// Return a new [`HdrBuffer`] with every pixel multiplied by `exposure`.
#[allow(dead_code)]
pub fn apply_exposure(buf: &HdrBuffer, exposure: f32) -> HdrBuffer {
    let pixels = buf
        .pixels
        .iter()
        .map(|p| [p[0] * exposure, p[1] * exposure, p[2] * exposure, p[3]])
        .collect();
    HdrBuffer {
        width: buf.width,
        height: buf.height,
        pixels,
    }
}

/// Return the pixel at `(x, y)`, or a transparent black pixel if out of bounds.
#[allow(dead_code)]
pub fn hdr_pixel_at(buf: &HdrBuffer, x: u32, y: u32) -> [f32; 4] {
    if x >= buf.width || y >= buf.height {
        return [0.0, 0.0, 0.0, 0.0];
    }
    buf.pixels[(y * buf.width + x) as usize]
}

/// Write a pixel at `(x, y)`.  Does nothing if out of bounds.
#[allow(dead_code)]
pub fn write_hdr_pixel(buf: &mut HdrBuffer, x: u32, y: u32, p: [f32; 4]) {
    if x < buf.width && y < buf.height {
        buf.pixels[(y * buf.width + x) as usize] = p;
    }
}

/// Run the full HDR pipeline: update exposure, apply it, measure histogram peak.
#[allow(dead_code)]
pub fn process_hdr(
    state: &mut HdrPipelineState,
    input: &HdrBuffer,
    dt: f32,
) -> HdrResult {
    let scene_lum = compute_avg_luminance(input);
    update_exposure(state, scene_lum, dt);
    let output = apply_exposure(input, state.current_exposure);
    let histogram_peak = output
        .pixels
        .iter()
        .map(|p| 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2])
        .fold(0.0_f32, f32::max);
    HdrResult {
        applied_exposure: state.current_exposure,
        histogram_peak,
        output,
    }
}

/// Serialise a [`HdrConfig`] to a JSON string.
#[allow(dead_code)]
pub fn hdr_config_to_json(cfg: &HdrConfig) -> String {
    format!(
        r#"{{"auto_exposure":{},"target_luminance":{:.4},"exposure_speed":{:.4},"min_exposure":{:.4},"max_exposure":{:.4}}}"#,
        cfg.auto_exposure,
        cfg.target_luminance,
        cfg.exposure_speed,
        cfg.min_exposure,
        cfg.max_exposure
    )
}

/// Serialise a [`HdrPipelineState`] to a JSON string.
#[allow(dead_code)]
pub fn hdr_state_to_json(state: &HdrPipelineState) -> String {
    format!(
        r#"{{"current_exposure":{:.4},"avg_luminance":{:.4},"config":{}}}"#,
        state.current_exposure,
        state.avg_luminance,
        hdr_config_to_json(&state.config)
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn white_buf(w: u32, h: u32) -> HdrBuffer {
        HdrBuffer {
            width: w,
            height: h,
            pixels: vec![[1.0, 1.0, 1.0, 1.0]; (w * h) as usize],
        }
    }

    #[test]
    fn default_config_auto_exposure_enabled() {
        let cfg = default_hdr_config();
        assert!(cfg.auto_exposure);
        assert!((cfg.target_luminance - 0.18).abs() < 1e-5);
    }

    #[test]
    fn new_hdr_buffer_correct_size() {
        let buf = new_hdr_buffer(4, 4);
        assert_eq!(buf.pixels.len(), 16);
        assert_eq!(buf.width, 4);
        assert_eq!(buf.height, 4);
    }

    #[test]
    fn compute_avg_luminance_white_image() {
        let buf = white_buf(2, 2);
        let lum = compute_avg_luminance(&buf);
        assert!((lum - 1.0).abs() < 1e-4, "white image luminance ≈ 1.0, got {}", lum);
    }

    #[test]
    fn compute_avg_luminance_black_image() {
        let buf = new_hdr_buffer(2, 2);
        let lum = compute_avg_luminance(&buf);
        assert!((lum).abs() < 1e-6);
    }

    #[test]
    fn apply_exposure_scales_rgb() {
        let buf = white_buf(1, 1);
        let out = apply_exposure(&buf, 2.0);
        assert!((out.pixels[0][0] - 2.0).abs() < 1e-5);
        assert!((out.pixels[0][3] - 1.0).abs() < 1e-5); // alpha unchanged
    }

    #[test]
    fn hdr_pixel_at_bounds() {
        let buf = new_hdr_buffer(4, 4);
        let p = hdr_pixel_at(&buf, 10, 10); // out of bounds
        assert_eq!(p, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn write_hdr_pixel_round_trip() {
        let mut buf = new_hdr_buffer(4, 4);
        write_hdr_pixel(&mut buf, 1, 2, [0.5, 0.3, 0.1, 1.0]);
        let p = hdr_pixel_at(&buf, 1, 2);
        assert!((p[0] - 0.5).abs() < 1e-5);
        assert!((p[1] - 0.3).abs() < 1e-5);
    }

    #[test]
    fn process_hdr_returns_result_with_exposure() {
        let cfg = default_hdr_config();
        let mut state = new_hdr_pipeline(cfg);
        let input = white_buf(2, 2);
        let result = process_hdr(&mut state, &input, 0.016);
        assert!(result.applied_exposure > 0.0);
        assert_eq!(result.output.pixels.len(), 4);
    }

    #[test]
    fn hdr_config_to_json_contains_auto_exposure() {
        let cfg = default_hdr_config();
        let json = hdr_config_to_json(&cfg);
        assert!(json.contains("\"auto_exposure\":true"));
    }

    #[test]
    fn hdr_state_to_json_contains_current_exposure() {
        let cfg = default_hdr_config();
        let state = new_hdr_pipeline(cfg);
        let json = hdr_state_to_json(&state);
        assert!(json.contains("\"current_exposure\""));
    }
}
