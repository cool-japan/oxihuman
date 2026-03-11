#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphWeightAnimator {
    current: f32,
    target: f32,
    speed: f32,
    done: bool,
}

#[allow(dead_code)]
pub fn new_weight_animator(initial: f32) -> MorphWeightAnimator {
    MorphWeightAnimator { current: initial, target: initial, speed: 1.0, done: true }
}

#[allow(dead_code)]
pub fn animate_weight(a: &mut MorphWeightAnimator, target: f32, speed: f32) {
    a.target = target;
    a.speed = speed.max(0.01);
    a.done = false;
}

#[allow(dead_code)]
pub fn animator_update(a: &mut MorphWeightAnimator, dt: f32) -> f32 {
    if !a.done {
        let diff = a.target - a.current;
        let step = a.speed * dt;
        if diff.abs() <= step { a.current = a.target; a.done = true; }
        else { a.current += diff.signum() * step; }
    }
    a.current
}

#[allow(dead_code)]
pub fn animator_is_done(a: &MorphWeightAnimator) -> bool { a.done }

#[allow(dead_code)]
pub fn animator_current(a: &MorphWeightAnimator) -> f32 { a.current }

#[allow(dead_code)]
pub fn animator_target_wa(a: &MorphWeightAnimator) -> f32 { a.target }

#[allow(dead_code)]
pub fn animator_to_json(a: &MorphWeightAnimator) -> String {
    format!("{{\"current\":{:.4},\"target\":{:.4},\"done\":{}}}", a.current, a.target, a.done)
}

#[allow(dead_code)]
pub fn animator_reset(a: &mut MorphWeightAnimator) {
    a.current = 0.0; a.target = 0.0; a.done = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let a = new_weight_animator(0.0); assert!(animator_is_done(&a)); }
    #[test] fn test_animate() { let mut a = new_weight_animator(0.0); animate_weight(&mut a, 1.0, 2.0); assert!(!animator_is_done(&a)); }
    #[test] fn test_update() { let mut a = new_weight_animator(0.0); animate_weight(&mut a, 1.0, 10.0); animator_update(&mut a, 0.05); assert!(animator_current(&a) > 0.0); }
    #[test] fn test_done_after() { let mut a = new_weight_animator(0.0); animate_weight(&mut a, 0.1, 10.0); animator_update(&mut a, 1.0); assert!(animator_is_done(&a)); }
    #[test] fn test_current() { let a = new_weight_animator(0.5); assert!((animator_current(&a) - 0.5).abs() < 1e-6); }
    #[test] fn test_target() { let mut a = new_weight_animator(0.0); animate_weight(&mut a, 0.8, 1.0); assert!((animator_target_wa(&a) - 0.8).abs() < 1e-6); }
    #[test] fn test_json() { let a = new_weight_animator(0.0); assert!(animator_to_json(&a).contains("current")); }
    #[test] fn test_reset() { let mut a = new_weight_animator(1.0); animator_reset(&mut a); assert!((animator_current(&a)).abs() < 1e-6); }
    #[test] fn test_no_move_when_done() { let mut a = new_weight_animator(0.5); let v = animator_update(&mut a, 0.1); assert!((v - 0.5).abs() < 1e-6); }
    #[test] fn test_negative_target() { let mut a = new_weight_animator(0.0); animate_weight(&mut a, -1.0, 5.0); animator_update(&mut a, 0.1); assert!(animator_current(&a) < 0.0); }
}
