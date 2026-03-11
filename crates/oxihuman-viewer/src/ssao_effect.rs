//! Screen-Space Ambient Occlusion (SSAO) effect.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration parameters for the SSAO effect.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SsaoEffectConfig {
    pub radius: f32,
    pub bias: f32,
    pub intensity: f32,
    pub sample_count: u32,
    pub blur_passes: u32,
}

/// Hemisphere sample kernel and noise texture size for SSAO.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SsaoKernel {
    pub samples: Vec<[f32; 3]>,
    pub noise_size: u32,
}

/// CPU-side buffer storing per-pixel ambient occlusion values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SsaoBuffer {
    pub width: u32,
    pub height: u32,
    pub ao_data: Vec<f32>,
}

/// Result of an SSAO computation pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SsaoResult {
    pub ao_buffer: SsaoBuffer,
    pub avg_occlusion: f32,
    pub dark_pixel_count: usize,
}

// ── Functions ──────────────────────────────────────────────────────────────────

/// Return a default [`SsaoEffectConfig`] with sensible defaults.
#[allow(dead_code)]
pub fn default_ssao_config() -> SsaoEffectConfig {
    SsaoEffectConfig {
        radius: 0.5,
        bias: 0.025,
        intensity: 1.0,
        sample_count: 32,
        blur_passes: 2,
    }
}

/// Generate a hemisphere sample kernel with `sample_count` samples.
/// Samples are distributed over the unit hemisphere and biased toward the
/// centre using a simple accelerating interpolation.
#[allow(dead_code)]
pub fn generate_ssao_kernel(sample_count: u32) -> SsaoKernel {
    let n = sample_count.max(1) as usize;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let theta = 2.0 * std::f32::consts::PI * (i as f32 / n as f32);
        let phi = (i as f32 / n as f32) * std::f32::consts::FRAC_PI_2;
        let scale = (i as f32 / n as f32).clamp(0.0, 1.0);
        let scale = 0.1_f32 + scale * scale * 0.9;
        samples.push([
            theta.cos() * phi.cos() * scale,
            theta.sin() * phi.cos() * scale,
            phi.sin().abs() * scale,
        ]);
    }
    SsaoKernel {
        samples,
        noise_size: 4,
    }
}

/// Allocate an [`SsaoBuffer`] of given dimensions filled with 1.0 (no occlusion).
#[allow(dead_code)]
pub fn new_ssao_buffer(w: u32, h: u32) -> SsaoBuffer {
    let size = (w as usize) * (h as usize);
    SsaoBuffer {
        width: w,
        height: h,
        ao_data: vec![1.0_f32; size],
    }
}

/// Compute SSAO using depth and normal buffers.  This is a CPU stub that
/// produces a plausible occlusion buffer based on depth variance.
#[allow(dead_code)]
pub fn compute_ssao(
    depth: &SsaoBuffer,
    _normals: &SsaoBuffer,
    kernel: &SsaoKernel,
    cfg: &SsaoEffectConfig,
) -> SsaoResult {
    let mut out = new_ssao_buffer(depth.width, depth.height);
    let w = depth.width as usize;
    let h = depth.height as usize;
    let radius = cfg.radius;
    let bias = cfg.bias;
    let intensity = cfg.intensity;
    let n_samples = kernel.samples.len();

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let base_depth = depth.ao_data.get(idx).copied().unwrap_or(1.0);
            let mut occlusion = 0.0_f32;
            for s in &kernel.samples {
                let sx = (x as f32 + s[0] * radius * w as f32).clamp(0.0, (w - 1) as f32) as usize;
                let sy = (y as f32 + s[1] * radius * h as f32).clamp(0.0, (h - 1) as f32) as usize;
                let sample_depth = depth.ao_data.get(sy * w + sx).copied().unwrap_or(1.0);
                if sample_depth < base_depth - bias {
                    let range_check = 1.0 - (base_depth - sample_depth).abs() / radius;
                    occlusion += range_check.clamp(0.0, 1.0);
                }
            }
            let ao = if !kernel.samples.is_empty() {
                (1.0 - occlusion / n_samples as f32 * intensity).clamp(0.0, 1.0)
            } else {
                1.0
            };
            out.ao_data[idx] = ao;
        }
    }

    let out = blur_ao_buffer(&out, cfg.blur_passes);
    let avg = ssao_avg_occlusion(&out);
    let dark_pixel_count = out.ao_data.iter().filter(|&&v| v < 0.5).count();

    SsaoResult {
        ao_buffer: out,
        avg_occlusion: avg,
        dark_pixel_count,
    }
}

/// Blur the AO buffer by running `passes` box-blur passes.
#[allow(dead_code)]
pub fn blur_ao_buffer(buf: &SsaoBuffer, passes: u32) -> SsaoBuffer {
    let mut current = buf.clone();
    let w = buf.width as usize;
    let h = buf.height as usize;
    for _ in 0..passes {
        let src = current.ao_data.clone();
        for y in 0..h {
            for x in 0..w {
                let mut sum = 0.0_f32;
                let mut count = 0u32;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                            sum += src[(ny as usize) * w + (nx as usize)];
                            count += 1;
                        }
                    }
                }
                current.ao_data[y * w + x] = if count > 0 { sum / count as f32 } else { 0.0 };
            }
        }
    }
    current
}

/// Read the AO value at pixel `(x, y)`, returning 1.0 if out of bounds.
#[allow(dead_code)]
pub fn ssao_pixel_at(buf: &SsaoBuffer, x: u32, y: u32) -> f32 {
    let idx = (y as usize) * (buf.width as usize) + (x as usize);
    buf.ao_data.get(idx).copied().unwrap_or(1.0)
}

/// Write an AO value at pixel `(x, y)`, clamped to `[0, 1]`.
#[allow(dead_code)]
pub fn write_ao_pixel(buf: &mut SsaoBuffer, x: u32, y: u32, val: f32) {
    let idx = (y as usize) * (buf.width as usize) + (x as usize);
    if let Some(slot) = buf.ao_data.get_mut(idx) {
        *slot = val.clamp(0.0, 1.0);
    }
}

/// Compute the average AO value across the entire buffer.
#[allow(dead_code)]
pub fn ssao_avg_occlusion(buf: &SsaoBuffer) -> f32 {
    if buf.ao_data.is_empty() {
        return 1.0;
    }
    buf.ao_data.iter().sum::<f32>() / buf.ao_data.len() as f32
}

/// Serialise the SSAO config to a JSON string.
#[allow(dead_code)]
pub fn ssao_config_to_json(cfg: &SsaoEffectConfig) -> String {
    format!(
        r#"{{"radius":{:.4},"bias":{:.4},"intensity":{:.4},"sample_count":{},"blur_passes":{}}}"#,
        cfg.radius, cfg.bias, cfg.intensity, cfg.sample_count, cfg.blur_passes
    )
}

/// Serialise the SSAO kernel to a JSON string.
#[allow(dead_code)]
pub fn ssao_kernel_to_json(k: &SsaoKernel) -> String {
    let samples: Vec<String> = k
        .samples
        .iter()
        .map(|s| format!("[{:.4},{:.4},{:.4}]", s[0], s[1], s[2]))
        .collect();
    format!(
        r#"{{"noise_size":{},"samples":[{}]}}"#,
        k.noise_size,
        samples.join(",")
    )
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_ssao_config();
        assert!((cfg.radius - 0.5).abs() < 1e-6);
        assert_eq!(cfg.sample_count, 32);
        assert_eq!(cfg.blur_passes, 2);
    }

    #[test]
    fn new_ssao_buffer_all_ones() {
        let buf = new_ssao_buffer(4, 4);
        assert_eq!(buf.ao_data.len(), 16);
        assert!(buf.ao_data.iter().all(|&v| (v - 1.0).abs() < 1e-6));
    }

    #[test]
    fn write_and_read_pixel() {
        let mut buf = new_ssao_buffer(8, 8);
        write_ao_pixel(&mut buf, 3, 2, 0.5);
        assert!((ssao_pixel_at(&buf, 3, 2) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn avg_occlusion_full_buffer() {
        let buf = new_ssao_buffer(4, 4);
        assert!((ssao_avg_occlusion(&buf) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn generate_kernel_count() {
        let kernel = generate_ssao_kernel(16);
        assert_eq!(kernel.samples.len(), 16);
    }

    #[test]
    fn blur_does_not_change_size() {
        let buf = new_ssao_buffer(8, 8);
        let blurred = blur_ao_buffer(&buf, 2);
        assert_eq!(blurred.width, 8);
        assert_eq!(blurred.height, 8);
        assert_eq!(blurred.ao_data.len(), 64);
    }

    #[test]
    fn config_to_json_contains_fields() {
        let cfg = default_ssao_config();
        let json = ssao_config_to_json(&cfg);
        assert!(json.contains("radius"));
        assert!(json.contains("sample_count"));
    }
}
