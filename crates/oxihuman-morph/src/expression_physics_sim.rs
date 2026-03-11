#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionPhysicsSim {
    jiggle: f32,
    spring: f32,
    damping: f32,
    velocity: f32,
    position: f32,
}

#[allow(dead_code)]
pub fn new_expression_physics(jiggle: f32, spring: f32, damping: f32) -> ExpressionPhysicsSim {
    ExpressionPhysicsSim { jiggle, spring, damping, velocity: 0.0, position: 0.0 }
}

#[allow(dead_code)]
pub fn physics_step_expr(sim: &mut ExpressionPhysicsSim, target: f32, dt: f32) -> f32 {
    let force = -sim.spring * (sim.position - target) - sim.damping * sim.velocity;
    sim.velocity += force * dt;
    sim.position += sim.velocity * dt;
    sim.position += sim.jiggle * sim.velocity.abs() * 0.01;
    sim.position
}

#[allow(dead_code)]
pub fn jiggle_weight(sim: &ExpressionPhysicsSim) -> f32 { sim.jiggle }

#[allow(dead_code)]
pub fn spring_weight(sim: &ExpressionPhysicsSim) -> f32 { sim.spring }

#[allow(dead_code)]
pub fn damping_weight(sim: &ExpressionPhysicsSim) -> f32 { sim.damping }

#[allow(dead_code)]
pub fn physics_reset_expr(sim: &mut ExpressionPhysicsSim) {
    sim.velocity = 0.0; sim.position = 0.0;
}

#[allow(dead_code)]
pub fn physics_to_json_eps(sim: &ExpressionPhysicsSim) -> String {
    format!("{{\"position\":{:.4},\"velocity\":{:.4}}}", sim.position, sim.velocity)
}

#[allow(dead_code)]
pub fn physics_energy(sim: &ExpressionPhysicsSim) -> f32 {
    0.5 * sim.spring * sim.position * sim.position + 0.5 * sim.velocity * sim.velocity
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let s = new_expression_physics(0.1, 10.0, 1.0); assert!((s.position).abs() < 1e-6); }
    #[test] fn test_step() { let mut s = new_expression_physics(0.0, 10.0, 0.5); physics_step_expr(&mut s, 1.0, 0.01); assert!(s.position != 0.0 || s.velocity != 0.0); }
    #[test] fn test_jiggle() { let s = new_expression_physics(0.5, 1.0, 1.0); assert!((jiggle_weight(&s) - 0.5).abs() < 1e-6); }
    #[test] fn test_spring() { let s = new_expression_physics(0.0, 5.0, 1.0); assert!((spring_weight(&s) - 5.0).abs() < 1e-6); }
    #[test] fn test_damping() { let s = new_expression_physics(0.0, 1.0, 3.0); assert!((damping_weight(&s) - 3.0).abs() < 1e-6); }
    #[test] fn test_reset() { let mut s = new_expression_physics(0.0, 10.0, 1.0); physics_step_expr(&mut s, 1.0, 0.1); physics_reset_expr(&mut s); assert!((s.position).abs() < 1e-6); }
    #[test] fn test_json() { let s = new_expression_physics(0.0, 1.0, 1.0); assert!(physics_to_json_eps(&s).contains("position")); }
    #[test] fn test_energy_zero() { let s = new_expression_physics(0.0, 1.0, 1.0); assert!(physics_energy(&s) < 1e-6); }
    #[test] fn test_energy_nonzero() { let mut s = new_expression_physics(0.0, 10.0, 0.1); physics_step_expr(&mut s, 1.0, 0.1); assert!(physics_energy(&s) > 0.0); }
    #[test] fn test_converge() { let mut s = new_expression_physics(0.0, 10.0, 5.0); for _ in 0..1000 { physics_step_expr(&mut s, 0.5, 0.01); } assert!((s.position - 0.5).abs() < 0.5); }
}
