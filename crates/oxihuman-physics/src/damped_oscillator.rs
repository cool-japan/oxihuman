//! Spring-mass damped oscillator for secondary motion and UI animation.

/// Configuration parameters for a damped oscillator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OscillatorConfig {
    pub spring_k: f32,
    pub damping: f32,
    pub mass: f32,
    pub rest_position: f32,
}

/// Dynamic state of a damped oscillator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OscillatorState {
    pub position: f32,
    pub velocity: f32,
    pub time: f32,
}

/// Result snapshot after one integration step.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OscillatorResult {
    pub position: f32,
    pub velocity: f32,
    pub energy: f32,
    pub is_settled: bool,
}

/// Return a default `OscillatorConfig`.
#[allow(dead_code)]
pub fn default_oscillator_config() -> OscillatorConfig {
    OscillatorConfig {
        spring_k: 50.0,
        damping: 5.0,
        mass: 1.0,
        rest_position: 0.0,
    }
}

/// Construct a new `OscillatorState` with the given initial position.
#[allow(dead_code)]
pub fn new_oscillator_state(initial_pos: f32) -> OscillatorState {
    OscillatorState {
        position: initial_pos,
        velocity: 0.0,
        time: 0.0,
    }
}

/// Advance the oscillator by `dt` seconds using semi-implicit Euler integration.
///
/// Returns an `OscillatorResult` with the updated state and energy.
#[allow(dead_code)]
pub fn step_oscillator(
    state: &mut OscillatorState,
    cfg: &OscillatorConfig,
    dt: f32,
) -> OscillatorResult {
    let dt = dt.clamp(1e-6, 0.1);
    let disp = state.position - cfg.rest_position;
    let spring_force = -cfg.spring_k * disp;
    let damp_force = -cfg.damping * state.velocity;
    let accel = (spring_force + damp_force) / cfg.mass.max(1e-10);

    state.velocity += accel * dt;
    state.position += state.velocity * dt;
    state.time += dt;

    let energy = oscillator_energy(state, cfg);
    let is_settled = energy < 1e-6 && state.velocity.abs() < 1e-4;

    OscillatorResult {
        position: state.position,
        velocity: state.velocity,
        energy,
        is_settled,
    }
}

/// Compute total mechanical energy (kinetic + potential) of the oscillator.
#[allow(dead_code)]
pub fn oscillator_energy(state: &OscillatorState, cfg: &OscillatorConfig) -> f32 {
    let disp = state.position - cfg.rest_position;
    let ke = 0.5 * cfg.mass * state.velocity * state.velocity;
    let pe = 0.5 * cfg.spring_k * disp * disp;
    ke + pe
}

/// Compute the critical damping coefficient for a given spring and mass.
#[allow(dead_code)]
pub fn critical_damping(spring_k: f32, mass: f32) -> f32 {
    2.0 * (spring_k * mass.max(1e-10)).sqrt()
}

/// Return `true` if the oscillator is overdamped (damping > critical damping).
#[allow(dead_code)]
pub fn is_overdamped(cfg: &OscillatorConfig) -> bool {
    let c_crit = critical_damping(cfg.spring_k, cfg.mass);
    cfg.damping > c_crit
}

/// Return `true` if the oscillator is underdamped (damping < critical damping).
#[allow(dead_code)]
pub fn is_underdamped(cfg: &OscillatorConfig) -> bool {
    let c_crit = critical_damping(cfg.spring_k, cfg.mass);
    cfg.damping < c_crit
}

/// Compute the natural angular frequency ω₀ = √(k/m) in rad/s.
#[allow(dead_code)]
pub fn oscillator_frequency(cfg: &OscillatorConfig) -> f32 {
    (cfg.spring_k / cfg.mass.max(1e-10)).sqrt()
}

/// Serialize an `OscillatorState` to a JSON string.
#[allow(dead_code)]
pub fn oscillator_to_json(state: &OscillatorState) -> String {
    format!(
        "{{\"position\":{},\"velocity\":{},\"time\":{}}}",
        state.position, state.velocity, state.time
    )
}

/// Reset the oscillator to `pos` with zero velocity.
#[allow(dead_code)]
pub fn reset_oscillator(state: &mut OscillatorState, pos: f32) {
    state.position = pos;
    state.velocity = 0.0;
    state.time = 0.0;
}

/// Serialize an `OscillatorResult` to a JSON string.
#[allow(dead_code)]
pub fn oscillator_result_to_json(r: &OscillatorResult) -> String {
    format!(
        "{{\"position\":{},\"velocity\":{},\"energy\":{},\"is_settled\":{}}}",
        r.position, r.velocity, r.energy, r.is_settled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_sensible() {
        let cfg = default_oscillator_config();
        assert!(cfg.spring_k > 0.0);
        assert!(cfg.mass > 0.0);
        assert!(cfg.damping >= 0.0);
    }

    #[test]
    fn new_state_zero_velocity() {
        let state = new_oscillator_state(1.0);
        assert!((state.position - 1.0).abs() < 1e-6);
        assert!(state.velocity.abs() < 1e-6);
    }

    #[test]
    fn step_oscillator_moves_toward_rest() {
        let cfg = default_oscillator_config();
        let mut state = new_oscillator_state(1.0);
        // After many steps with strong damping, position should approach rest
        for _ in 0..1000 {
            step_oscillator(&mut state, &cfg, 0.01);
        }
        assert!(
            (state.position - cfg.rest_position).abs() < 0.1,
            "oscillator did not settle, pos={}",
            state.position
        );
    }

    #[test]
    fn energy_decreases_with_damping() {
        let cfg = default_oscillator_config();
        let mut state = new_oscillator_state(1.0);
        let e0 = oscillator_energy(&state, &cfg);
        for _ in 0..100 {
            step_oscillator(&mut state, &cfg, 0.01);
        }
        let e1 = oscillator_energy(&state, &cfg);
        assert!(e1 < e0, "energy should decrease with damping");
    }

    #[test]
    fn critical_damping_formula() {
        let c = critical_damping(100.0, 1.0);
        assert!((c - 20.0).abs() < 1e-3, "expected 20, got {c}");
    }

    #[test]
    fn overdamped_detection() {
        let cfg = OscillatorConfig {
            spring_k: 1.0,
            damping: 100.0,
            mass: 1.0,
            rest_position: 0.0,
        };
        assert!(is_overdamped(&cfg));
        assert!(!is_underdamped(&cfg));
    }

    #[test]
    fn oscillator_to_json_contains_fields() {
        let state = new_oscillator_state(0.5);
        let json = oscillator_to_json(&state);
        assert!(json.contains("position"));
        assert!(json.contains("velocity"));
        assert!(json.contains("time"));
    }

    #[test]
    fn oscillator_result_to_json_contains_energy() {
        let r = OscillatorResult {
            position: 0.1,
            velocity: 0.2,
            energy: 0.05,
            is_settled: false,
        };
        let json = oscillator_result_to_json(&r);
        assert!(json.contains("energy"));
        assert!(json.contains("is_settled"));
    }

    #[test]
    fn reset_oscillator_clears_state() {
        let mut state = new_oscillator_state(1.0);
        state.velocity = 5.0;
        state.time = 10.0;
        reset_oscillator(&mut state, 0.0);
        assert!(state.position.abs() < 1e-6);
        assert!(state.velocity.abs() < 1e-6);
        assert!(state.time.abs() < 1e-6);
    }
}
