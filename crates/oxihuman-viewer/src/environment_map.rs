//! Environment map (lat-long / equirectangular) sampling and management.

// ── Types ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum EnvMapFormat {
    LatLong,
    Cubemap,
    HStrip,
    VStrip,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvMapConfig {
    pub format: EnvMapFormat,
    pub width: u32,
    pub height: u32,
    pub hdr: bool,
    pub intensity: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvMap {
    pub config: EnvMapConfig,
    pub data: Vec<[f32; 3]>,
    pub is_loaded: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvSampleResult {
    pub color: [f32; 3],
    pub mip_level: u32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_env_map_config(w: u32, h: u32) -> EnvMapConfig {
    EnvMapConfig {
        format: EnvMapFormat::LatLong,
        width: w,
        height: h,
        hdr: true,
        intensity: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_env_map(cfg: EnvMapConfig) -> EnvMap {
    let n = (cfg.width as usize) * (cfg.height as usize);
    EnvMap {
        data: vec![[0.0, 0.0, 0.0]; n],
        config: cfg,
        is_loaded: false,
    }
}

#[allow(dead_code)]
pub fn sample_env_map(map: &EnvMap, dir: [f32; 3]) -> EnvSampleResult {
    let uv = direction_to_uv_latlong(dir);
    let px = uv_to_pixel(uv, map.config.width, map.config.height);
    let color = env_map_pixel_at(map, px[0], px[1]);
    let scaled = [
        color[0] * map.config.intensity,
        color[1] * map.config.intensity,
        color[2] * map.config.intensity,
    ];
    EnvSampleResult {
        color: scaled,
        mip_level: 0,
    }
}

#[allow(dead_code)]
pub fn direction_to_uv_latlong(dir: [f32; 3]) -> [f32; 2] {
    // Normalize the direction
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    let d = if len > 1e-9 {
        [dir[0] / len, dir[1] / len, dir[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    };

    let u = (d[0].atan2(d[2]) / (2.0 * std::f32::consts::PI)) + 0.5;
    let v = 0.5 - d[1].asin() / std::f32::consts::PI;
    [u.clamp(0.0, 1.0), v.clamp(0.0, 1.0)]
}

#[allow(dead_code)]
pub fn uv_to_pixel(uv: [f32; 2], w: u32, h: u32) -> [u32; 2] {
    let px = ((uv[0] * w as f32) as u32).min(w.saturating_sub(1));
    let py = ((uv[1] * h as f32) as u32).min(h.saturating_sub(1));
    [px, py]
}

#[allow(dead_code)]
pub fn env_map_pixel_at(map: &EnvMap, x: u32, y: u32) -> [f32; 3] {
    let w = map.config.width as usize;
    let idx = y as usize * w + x as usize;
    if idx < map.data.len() {
        map.data[idx]
    } else {
        [0.0, 0.0, 0.0]
    }
}

#[allow(dead_code)]
pub fn write_env_pixel(map: &mut EnvMap, x: u32, y: u32, color: [f32; 3]) {
    let w = map.config.width as usize;
    let idx = y as usize * w + x as usize;
    if idx < map.data.len() {
        map.data[idx] = color;
    }
}

#[allow(dead_code)]
pub fn env_map_pixel_count(map: &EnvMap) -> usize {
    map.data.len()
}

#[allow(dead_code)]
pub fn env_map_format_name(map: &EnvMap) -> &'static str {
    match map.config.format {
        EnvMapFormat::LatLong => "latlong",
        EnvMapFormat::Cubemap => "cubemap",
        EnvMapFormat::HStrip => "hstrip",
        EnvMapFormat::VStrip => "vstrip",
    }
}

#[allow(dead_code)]
pub fn env_map_to_json(map: &EnvMap) -> String {
    format!(
        "{{\"format\":\"{}\",\"width\":{},\"height\":{},\"hdr\":{},\"intensity\":{}}}",
        env_map_format_name(map),
        map.config.width,
        map.config.height,
        map.config.hdr,
        map.config.intensity
    )
}

#[allow(dead_code)]
pub fn env_map_avg_luminance(map: &EnvMap) -> f32 {
    if map.data.is_empty() {
        return 0.0;
    }
    let sum: f32 = map
        .data
        .iter()
        .map(|p| 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2])
        .sum();
    sum / map.data.len() as f32
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_format_is_latlong() {
        let cfg = default_env_map_config(512, 256);
        assert_eq!(cfg.format, EnvMapFormat::LatLong);
        assert_eq!(cfg.width, 512);
        assert_eq!(cfg.height, 256);
    }

    #[test]
    fn new_env_map_correct_pixel_count() {
        let cfg = default_env_map_config(8, 4);
        let map = new_env_map(cfg);
        assert_eq!(env_map_pixel_count(&map), 32);
    }

    #[test]
    fn write_and_read_pixel() {
        let cfg = default_env_map_config(4, 4);
        let mut map = new_env_map(cfg);
        write_env_pixel(&mut map, 1, 2, [0.1, 0.2, 0.3]);
        let px = env_map_pixel_at(&map, 1, 2);
        assert!((px[0] - 0.1).abs() < 1e-6);
        assert!((px[2] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn direction_to_uv_north_pole() {
        // Up direction → v should be near 0
        let uv = direction_to_uv_latlong([0.0, 1.0, 0.0]);
        assert!(uv[1] < 0.1, "north pole should map to top of image");
    }

    #[test]
    fn direction_to_uv_south_pole() {
        let uv = direction_to_uv_latlong([0.0, -1.0, 0.0]);
        assert!(uv[1] > 0.9, "south pole should map to bottom of image");
    }

    #[test]
    fn uv_to_pixel_clamps() {
        let px = uv_to_pixel([1.1, -0.1], 64, 32);
        assert_eq!(px[0], 63);
        assert_eq!(px[1], 0);
    }

    #[test]
    fn format_names_are_correct() {
        let mut cfg = default_env_map_config(4, 4);
        cfg.format = EnvMapFormat::Cubemap;
        let map = new_env_map(cfg);
        assert_eq!(env_map_format_name(&map), "cubemap");
    }

    #[test]
    fn avg_luminance_empty() {
        let cfg = default_env_map_config(0, 0);
        let map = new_env_map(cfg);
        assert_eq!(env_map_avg_luminance(&map), 0.0);
    }

    #[test]
    fn avg_luminance_uniform_white() {
        let cfg = default_env_map_config(2, 2);
        let mut map = new_env_map(cfg);
        for px in map.data.iter_mut() {
            *px = [1.0, 1.0, 1.0];
        }
        let lum = env_map_avg_luminance(&map);
        assert!((lum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn to_json_contains_width() {
        let cfg = default_env_map_config(128, 64);
        let map = new_env_map(cfg);
        let json = env_map_to_json(&map);
        assert!(json.contains("128"));
        assert!(json.contains("64"));
    }
}
