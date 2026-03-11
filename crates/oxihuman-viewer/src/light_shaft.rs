// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Volumetric light shaft (god ray) post-processing effect.

// ── Types ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightShaftConfig {
    pub decay: f32,
    pub density: f32,
    pub weight: f32,
    pub exposure: f32,
    pub sample_count: u32,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightShaftBuffer {
    pub width: u32,
    pub height: u32,
    pub occlusion: Vec<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightShaftResult {
    pub output: Vec<f32>,
    pub light_screen_pos: [f32; 2],
    pub total_energy: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_light_shaft_config() -> LightShaftConfig {
    LightShaftConfig {
        decay: 0.96,
        density: 0.5,
        weight: 0.4,
        exposure: 0.2,
        sample_count: 100,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn new_light_shaft_buffer(w: u32, h: u32) -> LightShaftBuffer {
    LightShaftBuffer {
        width: w,
        height: h,
        occlusion: vec![1.0f32; (w * h) as usize],
    }
}

#[allow(dead_code)]
pub fn compute_light_shafts(
    occlusion: &LightShaftBuffer,
    light_pos: [f32; 2],
    cfg: &LightShaftConfig,
) -> LightShaftResult {
    if !cfg.enabled {
        let pixel_count = (occlusion.width * occlusion.height) as usize;
        return LightShaftResult {
            output: vec![0.0f32; pixel_count],
            light_screen_pos: light_pos,
            total_energy: 0.0,
        };
    }
    let output = radial_blur_toward(
        &occlusion.occlusion,
        occlusion.width,
        occlusion.height,
        light_pos,
        cfg.sample_count,
        cfg.decay,
    );
    // Scale by weight and exposure
    let output: Vec<f32> = output
        .iter()
        .map(|&v| v * cfg.weight * cfg.exposure)
        .collect();
    let total_energy = output.iter().sum::<f32>();
    LightShaftResult {
        output,
        light_screen_pos: light_pos,
        total_energy,
    }
}

/// Perform radial blur from each pixel toward `center`.
#[allow(dead_code)]
pub fn radial_blur_toward(
    data: &[f32],
    w: u32,
    h: u32,
    center: [f32; 2],
    samples: u32,
    decay: f32,
) -> Vec<f32> {
    let pixel_count = (w * h) as usize;
    let mut out = vec![0.0f32; pixel_count];
    let samples = samples.max(1);
    for py in 0..h {
        for px in 0..w {
            let mut illumination_decay = 1.0f32;
            let mut color = 0.0f32;
            // Step from pixel toward light center
            let dx = (center[0] - px as f32) / samples as f32;
            let dy = (center[1] - py as f32) / samples as f32;
            let mut sx = px as f32;
            let mut sy = py as f32;
            for _ in 0..samples {
                let ix = (sx.round() as i32).clamp(0, w as i32 - 1) as u32;
                let iy = (sy.round() as i32).clamp(0, h as i32 - 1) as u32;
                let sample_val = data[(iy * w + ix) as usize];
                color += sample_val * illumination_decay;
                illumination_decay *= decay;
                sx += dx;
                sy += dy;
            }
            out[(py * w + px) as usize] = color / samples as f32;
        }
    }
    out
}

#[allow(dead_code)]
pub fn occlusion_at(buf: &LightShaftBuffer, x: u32, y: u32) -> f32 {
    let idx = (y * buf.width + x) as usize;
    buf.occlusion.get(idx).copied().unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn write_occlusion(buf: &mut LightShaftBuffer, x: u32, y: u32, val: f32) {
    let idx = (y * buf.width + x) as usize;
    if let Some(slot) = buf.occlusion.get_mut(idx) {
        *slot = val.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn light_shaft_pixel_count(buf: &LightShaftBuffer) -> usize {
    (buf.width * buf.height) as usize
}

#[allow(dead_code)]
pub fn light_shaft_config_to_json(cfg: &LightShaftConfig) -> String {
    format!(
        r#"{{"decay":{:.4},"density":{:.4},"weight":{:.4},"exposure":{:.4},"sample_count":{},"enabled":{}}}"#,
        cfg.decay, cfg.density, cfg.weight, cfg.exposure, cfg.sample_count, cfg.enabled
    )
}

#[allow(dead_code)]
pub fn light_shaft_result_to_json(r: &LightShaftResult) -> String {
    format!(
        r#"{{"light_screen_pos":[{:.4},{:.4}],"total_energy":{:.6},"pixel_count":{}}}"#,
        r.light_screen_pos[0],
        r.light_screen_pos[1],
        r.total_energy,
        r.output.len()
    )
}

#[allow(dead_code)]
pub fn shaft_total_energy(result: &LightShaftResult) -> f32 {
    result.total_energy
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_enabled_and_decay() {
        let cfg = default_light_shaft_config();
        assert!(cfg.enabled);
        assert!((cfg.decay - 0.96).abs() < 1e-6);
    }

    #[test]
    fn new_buffer_correct_size() {
        let buf = new_light_shaft_buffer(4, 4);
        assert_eq!(buf.occlusion.len(), 16);
        assert_eq!(light_shaft_pixel_count(&buf), 16);
    }

    #[test]
    fn write_and_read_occlusion() {
        let mut buf = new_light_shaft_buffer(8, 8);
        write_occlusion(&mut buf, 2, 3, 0.5);
        assert!((occlusion_at(&buf, 2, 3) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn write_occlusion_clamps() {
        let mut buf = new_light_shaft_buffer(4, 4);
        write_occlusion(&mut buf, 0, 0, 2.5);
        assert!((occlusion_at(&buf, 0, 0) - 1.0).abs() < 1e-6);
        write_occlusion(&mut buf, 0, 0, -1.0);
        assert!((occlusion_at(&buf, 0, 0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn compute_light_shafts_disabled_returns_zeros() {
        let buf = new_light_shaft_buffer(4, 4);
        let mut cfg = default_light_shaft_config();
        cfg.enabled = false;
        let result = compute_light_shafts(&buf, [2.0, 2.0], &cfg);
        assert_eq!(result.output.len(), 16);
        assert!(result.output.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn compute_light_shafts_enabled_produces_output() {
        let buf = new_light_shaft_buffer(8, 8);
        let cfg = default_light_shaft_config();
        let result = compute_light_shafts(&buf, [4.0, 4.0], &cfg);
        assert_eq!(result.output.len(), 64);
    }

    #[test]
    fn shaft_total_energy_accessor() {
        let r = LightShaftResult {
            output: vec![0.5, 0.5],
            light_screen_pos: [0.0, 0.0],
            total_energy: 1.0,
        };
        assert!((shaft_total_energy(&r) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn config_to_json_contains_decay() {
        let cfg = default_light_shaft_config();
        let json = light_shaft_config_to_json(&cfg);
        assert!(json.contains("decay"));
        assert!(json.contains("sample_count"));
    }

    #[test]
    fn result_to_json_contains_total_energy() {
        let r = LightShaftResult {
            output: vec![],
            light_screen_pos: [10.0, 20.0],
            total_energy: std::f32::consts::PI,
        };
        let json = light_shaft_result_to_json(&r);
        assert!(json.contains("total_energy"));
        assert!(json.contains("10.0000"));
    }
}
