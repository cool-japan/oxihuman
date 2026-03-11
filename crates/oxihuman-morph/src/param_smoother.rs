#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParamSmoother {
    current: f32,
    target: f32,
    speed: f32,
}

#[allow(dead_code)]
pub fn new_param_smoother(initial: f32, speed: f32) -> ParamSmoother {
    ParamSmoother { current: initial, target: initial, speed: speed.max(0.01) }
}

#[allow(dead_code)]
pub fn smooth_param(s: &mut ParamSmoother, dt: f32) -> f32 {
    let diff = s.target - s.current;
    s.current += diff * (s.speed * dt).min(1.0);
    s.current
}

#[allow(dead_code)]
pub fn smoother_target(s: &ParamSmoother) -> f32 { s.target }

#[allow(dead_code)]
pub fn smoother_current(s: &ParamSmoother) -> f32 { s.current }

#[allow(dead_code)]
pub fn smoother_speed(s: &ParamSmoother) -> f32 { s.speed }

#[allow(dead_code)]
pub fn smoother_is_settled(s: &ParamSmoother) -> bool { (s.target - s.current).abs() < 1e-5 }

#[allow(dead_code)]
pub fn smoother_to_json(s: &ParamSmoother) -> String {
    format!("{{\"current\":{:.4},\"target\":{:.4},\"speed\":{:.4}}}", s.current, s.target, s.speed)
}

#[allow(dead_code)]
pub fn smoother_reset(s: &mut ParamSmoother) { s.current = 0.0; s.target = 0.0; }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let s = new_param_smoother(0.0, 5.0); assert!(smoother_is_settled(&s)); }
    #[test] fn test_target() { let s = new_param_smoother(1.0, 1.0); assert!((smoother_target(&s) - 1.0).abs() < 1e-6); }
    #[test] fn test_current() { let s = new_param_smoother(0.5, 1.0); assert!((smoother_current(&s) - 0.5).abs() < 1e-6); }
    #[test] fn test_speed() { let s = new_param_smoother(0.0, 3.0); assert!((smoother_speed(&s) - 3.0).abs() < 1e-6); }
    #[test] fn test_smooth() { let mut s = new_param_smoother(0.0, 10.0); s.target = 1.0; smooth_param(&mut s, 0.1); assert!(smoother_current(&s) > 0.0); }
    #[test] fn test_settled() { let s = new_param_smoother(0.5, 1.0); assert!(smoother_is_settled(&s)); }
    #[test] fn test_not_settled() { let mut s = new_param_smoother(0.0, 1.0); s.target = 1.0; assert!(!smoother_is_settled(&s)); }
    #[test] fn test_json() { let s = new_param_smoother(0.0, 1.0); assert!(smoother_to_json(&s).contains("current")); }
    #[test] fn test_reset() { let mut s = new_param_smoother(5.0, 1.0); smoother_reset(&mut s); assert!((smoother_current(&s)).abs() < 1e-6); }
    #[test] fn test_min_speed() { let s = new_param_smoother(0.0, -1.0); assert!(smoother_speed(&s) >= 0.01); }
}
