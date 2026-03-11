#![allow(dead_code)]

/// Smooth interpolator between morph states.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphInterpolator {
    current: Vec<f32>,
    target: Vec<f32>,
    speed: f32,
    progress: f32,
}

#[allow(dead_code)]
pub fn new_morph_interpolator(size: usize) -> MorphInterpolator {
    MorphInterpolator {
        current: vec![0.0; size],
        target: vec![0.0; size],
        speed: 1.0,
        progress: 1.0,
    }
}

#[allow(dead_code)]
pub fn interpolate_morph(interp: &mut MorphInterpolator, dt: f32) -> &[f32] {
    if interp.progress < 1.0 {
        interp.progress = (interp.progress + dt * interp.speed).min(1.0);
        let t = interp.progress;
        for i in 0..interp.current.len() {
            let a = interp.current[i];
            let b = if i < interp.target.len() { interp.target[i] } else { 0.0 };
            interp.current[i] = a + (b - a) * t;
        }
    }
    &interp.current
}

#[allow(dead_code)]
pub fn set_interpolation_speed(interp: &mut MorphInterpolator, speed: f32) {
    interp.speed = speed.max(0.0);
}

#[allow(dead_code)]
pub fn interpolation_progress(interp: &MorphInterpolator) -> f32 { interp.progress }

#[allow(dead_code)]
pub fn interpolator_is_done(interp: &MorphInterpolator) -> bool { interp.progress >= 1.0 }

#[allow(dead_code)]
pub fn interpolator_target(interp: &MorphInterpolator) -> &[f32] { &interp.target }

#[allow(dead_code)]
pub fn interpolator_to_json(interp: &MorphInterpolator) -> String {
    format!("{{\"size\":{},\"speed\":{:.4},\"progress\":{:.4}}}", interp.current.len(), interp.speed, interp.progress)
}

#[allow(dead_code)]
pub fn interpolator_reset(interp: &mut MorphInterpolator) {
    for v in interp.current.iter_mut() { *v = 0.0; }
    for v in interp.target.iter_mut() { *v = 0.0; }
    interp.progress = 1.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() {
        let i = new_morph_interpolator(3);
        assert!(interpolator_is_done(&i));
    }
    #[test] fn test_set_speed() {
        let mut i = new_morph_interpolator(1);
        set_interpolation_speed(&mut i, 2.0);
        assert!((i.speed - 2.0).abs() < 1e-6);
    }
    #[test] fn test_progress() {
        let i = new_morph_interpolator(1);
        assert!((interpolation_progress(&i) - 1.0).abs() < 1e-6);
    }
    #[test] fn test_interpolate_done() {
        let mut i = new_morph_interpolator(1);
        let result = interpolate_morph(&mut i, 1.0);
        assert_eq!(result.len(), 1);
    }
    #[test] fn test_interpolate_active() {
        let mut i = new_morph_interpolator(1);
        i.target = vec![1.0];
        i.progress = 0.0;
        let _ = interpolate_morph(&mut i, 0.5);
        assert!(i.progress > 0.0);
    }
    #[test] fn test_target() {
        let mut i = new_morph_interpolator(2);
        i.target = vec![1.0, 2.0];
        assert_eq!(interpolator_target(&i).len(), 2);
    }
    #[test] fn test_to_json() {
        let i = new_morph_interpolator(3);
        assert!(interpolator_to_json(&i).contains("speed"));
    }
    #[test] fn test_reset() {
        let mut i = new_morph_interpolator(2);
        i.target = vec![1.0, 1.0];
        i.current = vec![0.5, 0.5];
        interpolator_reset(&mut i);
        assert!((i.current[0]).abs() < 1e-6);
    }
    #[test] fn test_is_done() {
        let mut i = new_morph_interpolator(1);
        i.progress = 0.5;
        assert!(!interpolator_is_done(&i));
    }
    #[test] fn test_speed_clamped() {
        let mut i = new_morph_interpolator(1);
        set_interpolation_speed(&mut i, -5.0);
        assert!((i.speed).abs() < 1e-6);
    }
}
