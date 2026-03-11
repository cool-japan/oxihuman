#![allow(dead_code)]
//! Camera shake: applies shake effects to camera position.

/// Configuration for camera shake.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShakeConfig {
    pub intensity: f32,
    pub decay: f32,
    pub frequency: f32,
}

/// A camera shake state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraShake {
    config: ShakeConfig,
    current_intensity: f32,
    time: f32,
    active: bool,
}

/// Create a new camera shake with the given config.
#[allow(dead_code)]
pub fn new_camera_shake(config: ShakeConfig) -> CameraShake {
    let intensity = config.intensity;
    CameraShake {
        config,
        current_intensity: intensity,
        time: 0.0,
        active: true,
    }
}

/// Return a default shake config.
#[allow(dead_code)]
pub fn default_shake_config() -> ShakeConfig {
    ShakeConfig {
        intensity: 0.1,
        decay: 2.0,
        frequency: 15.0,
    }
}

/// Apply shake for the given delta time, returning the offset.
#[allow(dead_code)]
pub fn apply_shake(shake: &mut CameraShake, dt: f32) -> [f32; 3] {
    if !shake.active || shake.current_intensity < 1e-6 {
        shake.active = false;
        return [0.0; 3];
    }
    shake.time += dt;
    shake.current_intensity *= (-shake.config.decay * dt).exp();
    let phase = shake.time * shake.config.frequency;
    let x = (phase * 1.0).sin() * shake.current_intensity;
    let y = (phase * 1.7).cos() * shake.current_intensity;
    let z = (phase * 0.3).sin() * shake.current_intensity * 0.5;
    [x, y, z]
}

/// Return the current intensity.
#[allow(dead_code)]
pub fn shake_intensity(shake: &CameraShake) -> f32 {
    shake.current_intensity
}

/// Return the decay rate.
#[allow(dead_code)]
pub fn shake_decay(shake: &CameraShake) -> f32 {
    shake.config.decay
}

/// Check if the shake is still active.
#[allow(dead_code)]
pub fn shake_is_active(shake: &CameraShake) -> bool {
    shake.active
}

/// Reset the shake to its initial intensity.
#[allow(dead_code)]
pub fn shake_reset(shake: &mut CameraShake) {
    shake.current_intensity = shake.config.intensity;
    shake.time = 0.0;
    shake.active = true;
}

/// Return the current offset without advancing time.
#[allow(dead_code)]
pub fn shake_offset(shake: &CameraShake) -> [f32; 3] {
    if !shake.active {
        return [0.0; 3];
    }
    let phase = shake.time * shake.config.frequency;
    let x = (phase * 1.0).sin() * shake.current_intensity;
    let y = (phase * 1.7).cos() * shake.current_intensity;
    let z = (phase * 0.3).sin() * shake.current_intensity * 0.5;
    [x, y, z]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_shake_config();
        assert!((c.intensity - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_new_shake() {
        let s = new_camera_shake(default_shake_config());
        assert!(shake_is_active(&s));
    }

    #[test]
    fn test_apply_shake() {
        let mut s = new_camera_shake(default_shake_config());
        let offset = apply_shake(&mut s, 0.016);
        assert!(offset[0].abs() > 0.0 || offset[1].abs() > 0.0);
    }

    #[test]
    fn test_shake_decays() {
        let mut s = new_camera_shake(default_shake_config());
        let initial = shake_intensity(&s);
        apply_shake(&mut s, 1.0);
        assert!(shake_intensity(&s) < initial);
    }

    #[test]
    fn test_shake_decay_rate() {
        let s = new_camera_shake(default_shake_config());
        assert!((shake_decay(&s) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_shake_reset() {
        let mut s = new_camera_shake(default_shake_config());
        apply_shake(&mut s, 5.0);
        shake_reset(&mut s);
        assert!(shake_is_active(&s));
        assert!((shake_intensity(&s) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_shake_offset() {
        let s = new_camera_shake(default_shake_config());
        let o = shake_offset(&s);
        // at time=0, sin(0)=0, cos(0)=1
        assert!((o[0]).abs() < 1e-3);
    }

    #[test]
    fn test_inactive_shake() {
        let mut s = new_camera_shake(ShakeConfig {
            intensity: 0.0,
            decay: 1.0,
            frequency: 1.0,
        });
        let offset = apply_shake(&mut s, 0.1);
        assert!((offset[0]).abs() < 1e-6);
    }

    #[test]
    fn test_high_frequency() {
        let mut s = new_camera_shake(ShakeConfig {
            intensity: 1.0,
            decay: 0.0,
            frequency: 100.0,
        });
        let _ = apply_shake(&mut s, 0.01);
        assert!(shake_is_active(&s));
    }

    #[test]
    fn test_zero_dt() {
        let mut s = new_camera_shake(default_shake_config());
        let _ = apply_shake(&mut s, 0.0);
        assert!(shake_is_active(&s));
    }
}
