//! Simulation cache export for playback of physics/cloth/fluid simulations.

// ── Types ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SimCacheConfig {
    pub frame_rate: f32,
    pub start_frame: u32,
    pub end_frame: u32,
    pub compress: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SimFrame {
    pub frame: u32,
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SimCache {
    pub frames: Vec<SimFrame>,
    pub config: SimCacheConfig,
    pub object_name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SimCacheExportResult {
    pub frame_count: usize,
    pub total_particles: usize,
    pub data_size_bytes: usize,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_sim_cache_config() -> SimCacheConfig {
    SimCacheConfig {
        frame_rate: 24.0,
        start_frame: 0,
        end_frame: 100,
        compress: false,
    }
}

#[allow(dead_code)]
pub fn new_sim_cache(name: &str, cfg: SimCacheConfig) -> SimCache {
    SimCache {
        frames: Vec::new(),
        config: cfg,
        object_name: name.to_string(),
    }
}

#[allow(dead_code)]
pub fn add_sim_frame(cache: &mut SimCache, frame: SimFrame) {
    cache.frames.push(frame);
}

#[allow(dead_code)]
pub fn new_sim_frame(frame: u32, positions: Vec<[f32; 3]>) -> SimFrame {
    let n = positions.len();
    SimFrame {
        frame,
        positions,
        velocities: vec![[0.0, 0.0, 0.0]; n],
    }
}

#[allow(dead_code)]
pub fn export_sim_cache(cache: &SimCache) -> SimCacheExportResult {
    let frame_count = cache.frames.len();
    let total_particles: usize = cache.frames.iter().map(|f| f.positions.len()).sum();
    // Each position and velocity entry is 3 × f32 = 12 bytes
    let data_size_bytes = total_particles * 2 * 12;
    SimCacheExportResult {
        frame_count,
        total_particles,
        data_size_bytes,
    }
}

#[allow(dead_code)]
pub fn frame_count_cache(cache: &SimCache) -> usize {
    cache.frames.len()
}

#[allow(dead_code)]
pub fn sim_cache_duration_sec(cache: &SimCache) -> f32 {
    if cache.config.frame_rate <= 0.0 {
        return 0.0;
    }
    let frames = cache.config.end_frame.saturating_sub(cache.config.start_frame) as f32;
    frames / cache.config.frame_rate
}

#[allow(dead_code)]
pub fn sim_cache_to_json(cache: &SimCache) -> String {
    format!(
        "{{\"name\":\"{}\",\"frames\":{},\"fps\":{}}}",
        cache.object_name,
        cache.frames.len(),
        cache.config.frame_rate
    )
}

#[allow(dead_code)]
pub fn sim_cache_result_to_json(r: &SimCacheExportResult) -> String {
    format!(
        "{{\"frame_count\":{},\"total_particles\":{},\"data_size_bytes\":{}}}",
        r.frame_count, r.total_particles, r.data_size_bytes
    )
}

#[allow(dead_code)]
pub fn interpolate_frame(cache: &SimCache, time: f32) -> Option<SimFrame> {
    if cache.frames.is_empty() {
        return None;
    }
    if cache.config.frame_rate <= 0.0 {
        return None;
    }

    let target_frame = time * cache.config.frame_rate;

    // Find the two surrounding frames
    let mut before: Option<&SimFrame> = None;
    let mut after: Option<&SimFrame> = None;

    for f in &cache.frames {
        let ff = f.frame as f32;
        if ff <= target_frame {
            before = Some(f);
        }
        if ff >= target_frame && after.is_none() {
            after = Some(f);
        }
    }

    let a = before.or(after)?;
    let b = after.unwrap_or(a);

    if a.frame == b.frame {
        return Some(a.clone());
    }

    let t = (target_frame - a.frame as f32) / (b.frame as f32 - a.frame as f32);
    let t = t.clamp(0.0, 1.0);

    let n = a.positions.len().min(b.positions.len());
    let positions: Vec<[f32; 3]> = (0..n)
        .map(|i| {
            let pa = a.positions[i];
            let pb = b.positions[i];
            [
                pa[0] + (pb[0] - pa[0]) * t,
                pa[1] + (pb[1] - pa[1]) * t,
                pa[2] + (pb[2] - pa[2]) * t,
            ]
        })
        .collect();

    Some(SimFrame {
        frame: a.frame,
        positions,
        velocities: vec![[0.0, 0.0, 0.0]; n],
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_sim_cache_config();
        assert!((cfg.frame_rate - 24.0).abs() < 1e-6);
        assert_eq!(cfg.start_frame, 0);
        assert_eq!(cfg.end_frame, 100);
        assert!(!cfg.compress);
    }

    #[test]
    fn new_frame_creates_matching_velocities() {
        let positions = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let frame = new_sim_frame(0, positions);
        assert_eq!(frame.positions.len(), frame.velocities.len());
    }

    #[test]
    fn add_frame_increments_count() {
        let cfg = default_sim_cache_config();
        let mut cache = new_sim_cache("test", cfg);
        let frame = new_sim_frame(0, vec![[0.0, 0.0, 0.0]]);
        add_sim_frame(&mut cache, frame);
        assert_eq!(frame_count_cache(&cache), 1);
    }

    #[test]
    fn export_result_counts_particles() {
        let cfg = default_sim_cache_config();
        let mut cache = new_sim_cache("test", cfg);
        let frame = new_sim_frame(0, vec![[0.0, 0.0, 0.0]; 10]);
        add_sim_frame(&mut cache, frame);
        let result = export_sim_cache(&cache);
        assert_eq!(result.total_particles, 10);
        assert_eq!(result.data_size_bytes, 10 * 2 * 12);
    }

    #[test]
    fn duration_sec_calculation() {
        let cfg = default_sim_cache_config(); // 24fps, 0..100 = 100 frames
        let cache = new_sim_cache("obj", cfg);
        let dur = sim_cache_duration_sec(&cache);
        assert!((dur - 100.0 / 24.0).abs() < 1e-4);
    }

    #[test]
    fn interpolate_frame_empty_returns_none() {
        let cfg = default_sim_cache_config();
        let cache = new_sim_cache("empty", cfg);
        assert!(interpolate_frame(&cache, 0.5).is_none());
    }

    #[test]
    fn interpolate_frame_midpoint() {
        let cfg = default_sim_cache_config();
        let mut cache = new_sim_cache("obj", cfg);
        add_sim_frame(&mut cache, new_sim_frame(0, vec![[0.0, 0.0, 0.0]]));
        add_sim_frame(&mut cache, new_sim_frame(24, vec![[24.0, 0.0, 0.0]]));
        // At t = 0.5 sec, frame = 12 → midway
        let f = interpolate_frame(&cache, 0.5).expect("should interpolate");
        assert!((f.positions[0][0] - 12.0).abs() < 1e-3);
    }

    #[test]
    fn to_json_contains_name() {
        let cfg = default_sim_cache_config();
        let cache = new_sim_cache("my_sim", cfg);
        let json = sim_cache_to_json(&cache);
        assert!(json.contains("my_sim"));
    }

    #[test]
    fn result_to_json_round_trip() {
        let r = SimCacheExportResult {
            frame_count: 10,
            total_particles: 500,
            data_size_bytes: 12000,
        };
        let json = sim_cache_result_to_json(&r);
        assert!(json.contains("10"));
        assert!(json.contains("500"));
        assert!(json.contains("12000"));
    }
}
