//! Subsurface scattering (SSS) approximation for skin rendering.

// ── Types ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SssConfig {
    pub scatter_radius: f32,
    pub scatter_color: [f32; 3],
    pub albedo: [f32; 3],
    pub translucency: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SssBuffer {
    pub width: u32,
    pub height: u32,
    pub irradiance: Vec<[f32; 3]>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SssResult {
    pub output: SssBuffer,
    pub avg_translucency: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_sss_config() -> SssConfig {
    SssConfig {
        scatter_radius: 0.05,
        scatter_color: [0.9, 0.6, 0.5],
        albedo: [0.85, 0.70, 0.65],
        translucency: 0.3,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn new_sss_buffer(w: u32, h: u32) -> SssBuffer {
    let n = (w as usize) * (h as usize);
    SssBuffer {
        width: w,
        height: h,
        irradiance: vec![[0.0, 0.0, 0.0]; n],
    }
}

#[allow(dead_code)]
pub fn apply_sss(input: &SssBuffer, cfg: &SssConfig) -> SssResult {
    if !cfg.enabled {
        return SssResult {
            output: input.clone(),
            avg_translucency: 0.0,
        };
    }
    let blurred = blur_irradiance(input, cfg.scatter_radius);
    let avg_t = sss_avg_irradiance(&blurred) * cfg.translucency;
    SssResult {
        output: blurred,
        avg_translucency: avg_t,
    }
}

#[allow(dead_code)]
pub fn sss_diffusion_profile(r: f32, cfg: &SssConfig) -> f32 {
    let sigma = cfg.scatter_radius.max(1e-6);
    let rr = r / sigma;
    // Dipole-like approximation: sum of two Gaussians
    let g1 = (-rr * rr * 0.5).exp();
    let g2 = (-rr * rr * 0.125).exp() * 0.5;
    ((g1 + g2) / (2.0 * std::f32::consts::PI * sigma * sigma)).max(0.0)
}

#[allow(dead_code)]
pub fn blur_irradiance(buf: &SssBuffer, radius: f32) -> SssBuffer {
    let w = buf.width as usize;
    let h = buf.height as usize;
    let r = (radius * (w as f32)).round() as usize;
    let r = r.clamp(1, w.min(h) / 2).max(1);

    let mut out = new_sss_buffer(buf.width, buf.height);
    for y in 0..h {
        for x in 0..w {
            let mut sum = [0.0f32; 3];
            let mut weight = 0.0f32;
            let x0 = x.saturating_sub(r);
            let x1 = (x + r).min(w - 1);
            let y0 = y.saturating_sub(r);
            let y1 = (y + r).min(h - 1);
            for ny in y0..=y1 {
                for nx in x0..=x1 {
                    let idx = ny * w + nx;
                    let px = buf.irradiance[idx];
                    sum[0] += px[0];
                    sum[1] += px[1];
                    sum[2] += px[2];
                    weight += 1.0;
                }
            }
            if weight > 0.0 {
                out.irradiance[y * w + x] = [
                    sum[0] / weight,
                    sum[1] / weight,
                    sum[2] / weight,
                ];
            }
        }
    }
    out
}

#[allow(dead_code)]
pub fn sss_pixel_at(buf: &SssBuffer, x: u32, y: u32) -> [f32; 3] {
    let w = buf.width as usize;
    let idx = y as usize * w + x as usize;
    if idx < buf.irradiance.len() {
        buf.irradiance[idx]
    } else {
        [0.0, 0.0, 0.0]
    }
}

#[allow(dead_code)]
pub fn write_sss_pixel(buf: &mut SssBuffer, x: u32, y: u32, v: [f32; 3]) {
    let w = buf.width as usize;
    let idx = y as usize * w + x as usize;
    if idx < buf.irradiance.len() {
        buf.irradiance[idx] = v;
    }
}

#[allow(dead_code)]
pub fn sss_pixel_count(buf: &SssBuffer) -> usize {
    buf.irradiance.len()
}

#[allow(dead_code)]
pub fn sss_config_to_json(cfg: &SssConfig) -> String {
    format!(
        "{{\"scatter_radius\":{},\"translucency\":{},\"enabled\":{}}}",
        cfg.scatter_radius, cfg.translucency, cfg.enabled
    )
}

#[allow(dead_code)]
pub fn sss_avg_irradiance(buf: &SssBuffer) -> f32 {
    if buf.irradiance.is_empty() {
        return 0.0;
    }
    let total: f32 = buf.irradiance.iter().map(|p| p[0] + p[1] + p[2]).sum();
    total / (buf.irradiance.len() as f32 * 3.0)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_enabled() {
        let cfg = default_sss_config();
        assert!(cfg.enabled);
        assert!(cfg.scatter_radius > 0.0);
    }

    #[test]
    fn new_buffer_correct_size() {
        let buf = new_sss_buffer(4, 4);
        assert_eq!(sss_pixel_count(&buf), 16);
    }

    #[test]
    fn write_and_read_pixel() {
        let mut buf = new_sss_buffer(8, 8);
        write_sss_pixel(&mut buf, 2, 3, [0.5, 0.6, 0.7]);
        let px = sss_pixel_at(&buf, 2, 3);
        assert!((px[0] - 0.5).abs() < 1e-6);
        assert!((px[1] - 0.6).abs() < 1e-6);
    }

    #[test]
    fn apply_sss_disabled_returns_input() {
        let mut cfg = default_sss_config();
        cfg.enabled = false;
        let buf = new_sss_buffer(4, 4);
        let result = apply_sss(&buf, &cfg);
        assert_eq!(result.avg_translucency, 0.0);
        assert_eq!(result.output.width, 4);
    }

    #[test]
    fn diffusion_profile_decays() {
        let cfg = default_sss_config();
        let d0 = sss_diffusion_profile(0.0, &cfg);
        let d1 = sss_diffusion_profile(1.0, &cfg);
        assert!(d0 > d1, "profile should decay with distance");
    }

    #[test]
    fn blur_preserves_dimensions() {
        let buf = new_sss_buffer(8, 8);
        let blurred = blur_irradiance(&buf, 0.1);
        assert_eq!(blurred.width, 8);
        assert_eq!(blurred.height, 8);
    }

    #[test]
    fn avg_irradiance_empty_buf() {
        let buf = new_sss_buffer(0, 0);
        assert_eq!(sss_avg_irradiance(&buf), 0.0);
    }

    #[test]
    fn config_to_json_contains_radius() {
        let cfg = default_sss_config();
        let json = sss_config_to_json(&cfg);
        assert!(json.contains("scatter_radius"));
    }
}
