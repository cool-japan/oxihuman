//! Motion blur post-processing using velocity buffer accumulation.

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for the motion blur effect.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MotionBlurConfig {
    pub samples: u32,
    pub shutter_angle: f32,
    pub max_velocity_px: f32,
    pub enabled: bool,
}

/// Per-pixel 2-D velocity buffer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VelocityBuffer {
    pub width: u32,
    pub height: u32,
    pub velocities: Vec<[f32; 2]>,
}

/// Result of applying motion blur to a frame.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MotionBlurResult {
    pub output: Vec<[f32; 4]>,
    pub avg_velocity: f32,
    pub blurred_pixel_count: usize,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Returns a sensible default `MotionBlurConfig`.
#[allow(dead_code)]
pub fn default_motion_blur_config() -> MotionBlurConfig {
    MotionBlurConfig {
        samples: 8,
        shutter_angle: 180.0,
        max_velocity_px: 32.0,
        enabled: true,
    }
}

/// Allocates a new `VelocityBuffer` with all velocities set to zero.
#[allow(dead_code)]
pub fn new_velocity_buffer(w: u32, h: u32) -> VelocityBuffer {
    VelocityBuffer {
        width: w,
        height: h,
        velocities: vec![[0.0f32; 2]; (w * h) as usize],
    }
}

/// Writes the velocity `(vx, vy)` for the pixel at `(x, y)`.
#[allow(dead_code)]
pub fn set_velocity(buf: &mut VelocityBuffer, x: u32, y: u32, vx: f32, vy: f32) {
    let idx = (y * buf.width + x) as usize;
    if idx < buf.velocities.len() {
        buf.velocities[idx] = [vx, vy];
    }
}

/// Returns the velocity at `(x, y)`.
#[allow(dead_code)]
pub fn velocity_at(buf: &VelocityBuffer, x: u32, y: u32) -> [f32; 2] {
    let idx = (y * buf.width + x) as usize;
    buf.velocities.get(idx).copied().unwrap_or([0.0; 2])
}

/// Applies motion blur to `frame` using the velocity buffer and config.
///
/// Each output pixel is the average of samples taken along the velocity vector.
#[allow(dead_code)]
pub fn apply_motion_blur(
    frame: &[[f32; 4]],
    vel: &VelocityBuffer,
    cfg: &MotionBlurConfig,
) -> MotionBlurResult {
    let w = vel.width as usize;
    let h = vel.height as usize;
    let total = w * h;
    let mut output = vec![[0.0f32; 4]; total];
    let mut blurred_pixel_count = 0usize;

    if !cfg.enabled || cfg.samples == 0 {
        // Pass-through
        if frame.len() >= total {
            output.copy_from_slice(&frame[..total]);
        }
        let av = avg_scene_velocity(vel);
        return MotionBlurResult {
            output,
            avg_velocity: av,
            blurred_pixel_count: 0,
        };
    }

    let scale = (cfg.shutter_angle / 360.0).clamp(0.0, 1.0);
    let samples = cfg.samples.max(1) as usize;

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            if idx >= frame.len() {
                continue;
            }
            let [vx, vy] = velocity_at(vel, x as u32, y as u32);
            let mag = velocity_magnitude([vx, vy]).min(cfg.max_velocity_px);
            if mag < 0.5 {
                output[idx] = frame[idx];
                continue;
            }

            blurred_pixel_count += 1;
            let step_vx = vx * scale / (samples as f32);
            let step_vy = vy * scale / (samples as f32);

            let mut acc = [0.0f32; 4];
            for s in 0..samples {
                let t = s as f32;
                let sx = (x as f32 + step_vx * t).clamp(0.0, (w - 1) as f32) as usize;
                let sy = (y as f32 + step_vy * t).clamp(0.0, (h - 1) as f32) as usize;
                let si = sy * w + sx;
                if si < frame.len() {
                    let p = frame[si];
                    acc[0] += p[0];
                    acc[1] += p[1];
                    acc[2] += p[2];
                    acc[3] += p[3];
                }
            }
            let inv = 1.0 / (samples as f32);
            output[idx] = [acc[0] * inv, acc[1] * inv, acc[2] * inv, acc[3] * inv];
        }
    }

    let av = avg_scene_velocity(vel);
    MotionBlurResult {
        output,
        avg_velocity: av,
        blurred_pixel_count,
    }
}

/// Returns the magnitude of a 2-D velocity vector.
#[allow(dead_code)]
pub fn velocity_magnitude(v: [f32; 2]) -> f32 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

/// Computes the average velocity magnitude across the whole buffer.
#[allow(dead_code)]
pub fn avg_scene_velocity(buf: &VelocityBuffer) -> f32 {
    if buf.velocities.is_empty() {
        return 0.0;
    }
    let sum: f32 = buf
        .velocities
        .iter()
        .map(|&v| velocity_magnitude(v))
        .sum();
    sum / (buf.velocities.len() as f32)
}

/// Serialises the config to a JSON string.
#[allow(dead_code)]
pub fn motion_blur_config_to_json(cfg: &MotionBlurConfig) -> String {
    format!(
        "{{\"samples\":{},\"shutter_angle\":{:.2},\"max_velocity_px\":{:.2},\"enabled\":{}}}",
        cfg.samples, cfg.shutter_angle, cfg.max_velocity_px, cfg.enabled,
    )
}

/// Returns the total pixel count in a velocity buffer.
#[allow(dead_code)]
pub fn pixel_count_vel_buf(buf: &VelocityBuffer) -> usize {
    buf.velocities.len()
}

/// Serialises the velocity buffer to a compact JSON stub.
#[allow(dead_code)]
pub fn velocity_to_json(buf: &VelocityBuffer) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"pixel_count\":{}}}",
        buf.width,
        buf.height,
        buf.velocities.len(),
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_motion_blur_config();
        assert_eq!(cfg.samples, 8);
        assert!(cfg.enabled);
        assert!((cfg.shutter_angle - 180.0).abs() < 1e-6);
    }

    #[test]
    fn new_buffer_all_zero() {
        let buf = new_velocity_buffer(4, 4);
        assert_eq!(buf.velocities.len(), 16);
        for v in &buf.velocities {
            assert_eq!(*v, [0.0; 2]);
        }
    }

    #[test]
    fn set_and_get_velocity() {
        let mut buf = new_velocity_buffer(8, 8);
        set_velocity(&mut buf, 3, 2, 1.5, -0.5);
        let v = velocity_at(&buf, 3, 2);
        assert!((v[0] - 1.5).abs() < 1e-6);
        assert!((v[1] + 0.5).abs() < 1e-6);
    }

    #[test]
    fn velocity_magnitude_zero_vec() {
        assert!((velocity_magnitude([0.0; 2])).abs() < 1e-9);
    }

    #[test]
    fn velocity_magnitude_unit_vec() {
        let m = velocity_magnitude([1.0, 0.0]);
        assert!((m - 1.0).abs() < 1e-6);
    }

    #[test]
    fn pixel_count_correct() {
        let buf = new_velocity_buffer(6, 7);
        assert_eq!(pixel_count_vel_buf(&buf), 42);
    }

    #[test]
    fn apply_motion_blur_disabled_passthrough() {
        let mut cfg = default_motion_blur_config();
        cfg.enabled = false;
        let frame = vec![[1.0f32, 0.5, 0.25, 1.0]; 4];
        let vel = new_velocity_buffer(2, 2);
        let result = apply_motion_blur(&frame, &vel, &cfg);
        assert_eq!(result.blurred_pixel_count, 0);
        assert!((result.output[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn avg_scene_velocity_all_zero() {
        let buf = new_velocity_buffer(3, 3);
        assert!((avg_scene_velocity(&buf)).abs() < 1e-9);
    }

    #[test]
    fn config_to_json_contains_samples() {
        let cfg = default_motion_blur_config();
        let j = motion_blur_config_to_json(&cfg);
        assert!(j.contains("\"samples\":8"));
    }
}
