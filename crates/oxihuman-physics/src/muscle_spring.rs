//! Muscle-tendon spring model: active contractile element + passive elastic element.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MuscleSpringConfig {
    pub rest_length: f32,
    pub passive_stiffness: f32,
    pub active_stiffness: f32,
    pub max_activation: f32,
    pub damping: f32,
    pub tendon_stiffness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MuscleSpring {
    config: MuscleSpringConfig,
    activation: f32,
    current_length: f32,
    velocity: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MuscleSpringResult {
    pub total_force: f32,
    pub passive_force: f32,
    pub active_force: f32,
    pub tendon_force: f32,
    pub energy: f32,
}

#[allow(dead_code)]
pub fn default_muscle_spring_config() -> MuscleSpringConfig {
    MuscleSpringConfig {
        rest_length: 1.0,
        passive_stiffness: 100.0,
        active_stiffness: 500.0,
        max_activation: 1.0,
        damping: 10.0,
        tendon_stiffness: 1000.0,
    }
}

#[allow(dead_code)]
pub fn new_muscle_spring(config: MuscleSpringConfig) -> MuscleSpring {
    let rest = config.rest_length;
    MuscleSpring {
        config,
        activation: 0.0,
        current_length: rest,
        velocity: 0.0,
    }
}

#[allow(dead_code)]
pub fn muscle_spring_force(spring: &MuscleSpring) -> MuscleSpringResult {
    let stretch = spring.current_length - spring.config.rest_length;

    // Passive element: linear spring (only resists elongation)
    let passive_force = if stretch > 0.0 {
        spring.config.passive_stiffness * stretch
    } else {
        spring.config.passive_stiffness * stretch * 0.1  // small compression stiffness
    };

    // Active contractile element: pulls toward rest length proportional to activation
    let act = spring.activation.clamp(0.0, spring.config.max_activation);
    let active_force = act * spring.config.active_stiffness * (-stretch);  // contracts

    // Tendon element: high-stiffness series spring (simple model: same stretch)
    let tendon_force = spring.config.tendon_stiffness * stretch.max(0.0) * 0.01;

    // Damping
    let damping_force = -spring.config.damping * spring.velocity;

    let total_force = passive_force + active_force + tendon_force + damping_force;

    // Energy: elastic potential
    let passive_energy = 0.5 * spring.config.passive_stiffness * stretch.powi(2);
    let active_energy  = 0.5 * spring.config.active_stiffness * act * stretch.powi(2);
    let energy = passive_energy + active_energy;

    MuscleSpringResult { total_force, passive_force, active_force, tendon_force, energy }
}

#[allow(dead_code)]
pub fn muscle_spring_activate(spring: &mut MuscleSpring, level: f32) {
    spring.activation = level.clamp(0.0, spring.config.max_activation);
}

#[allow(dead_code)]
pub fn muscle_spring_deactivate(spring: &mut MuscleSpring) {
    spring.activation = 0.0;
}

#[allow(dead_code)]
pub fn muscle_spring_length(spring: &MuscleSpring) -> f32 {
    spring.current_length
}

#[allow(dead_code)]
pub fn muscle_spring_stiffness(spring: &MuscleSpring) -> f32 {
    spring.config.passive_stiffness + spring.activation * spring.config.active_stiffness
}

#[allow(dead_code)]
pub fn muscle_spring_energy(spring: &MuscleSpring) -> f32 {
    muscle_spring_force(spring).energy
}

#[allow(dead_code)]
pub fn muscle_spring_to_json(spring: &MuscleSpring) -> String {
    format!(
        "{{\"rest_length\":{:.4},\"current_length\":{:.4},\"activation\":{:.4},\
         \"passive_stiffness\":{:.4},\"active_stiffness\":{:.4},\"damping\":{:.4}}}",
        spring.config.rest_length, spring.current_length, spring.activation,
        spring.config.passive_stiffness, spring.config.active_stiffness, spring.config.damping
    )
}

#[allow(dead_code)]
pub fn muscle_spring_reset(spring: &mut MuscleSpring) {
    spring.activation = 0.0;
    spring.current_length = spring.config.rest_length;
    spring.velocity = 0.0;
}

/// Set the current length of the muscle (called from simulation step).
#[allow(dead_code)]
pub fn muscle_spring_set_length(spring: &mut MuscleSpring, length: f32, velocity: f32) {
    spring.current_length = length.max(0.0);
    spring.velocity = velocity;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_muscle_spring_config();
        assert!((cfg.rest_length - 1.0).abs() < 1e-6);
        assert!(cfg.passive_stiffness > 0.0);
    }

    #[test]
    fn test_new_spring_at_rest() {
        let s = new_muscle_spring(default_muscle_spring_config());
        assert!((muscle_spring_length(&s) - 1.0).abs() < 1e-6);
        assert!((s.activation - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_force_at_rest_zero() {
        let s = new_muscle_spring(default_muscle_spring_config());
        let res = muscle_spring_force(&s);
        // At rest length, stretch=0 → passive=0, active=0 (no activation), tendon=0
        assert!((res.passive_force - 0.0).abs() < 1e-5);
        assert!((res.active_force - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_passive_force_positive_when_stretched() {
        let mut s = new_muscle_spring(default_muscle_spring_config());
        muscle_spring_set_length(&mut s, 1.5, 0.0);
        let res = muscle_spring_force(&s);
        assert!(res.passive_force > 0.0);
    }

    #[test]
    fn test_active_force_contracts() {
        let mut s = new_muscle_spring(default_muscle_spring_config());
        muscle_spring_set_length(&mut s, 1.2, 0.0);
        muscle_spring_activate(&mut s, 1.0);
        let res = muscle_spring_force(&s);
        // Active force should oppose stretch (negative or reduce total)
        assert!(res.active_force < 0.0);
    }

    #[test]
    fn test_activate_clamp() {
        let mut s = new_muscle_spring(default_muscle_spring_config());
        muscle_spring_activate(&mut s, 5.0);
        assert!((s.activation - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_deactivate() {
        let mut s = new_muscle_spring(default_muscle_spring_config());
        muscle_spring_activate(&mut s, 0.8);
        muscle_spring_deactivate(&mut s);
        assert!((s.activation - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_stiffness_increases_with_activation() {
        let mut s = new_muscle_spring(default_muscle_spring_config());
        let k0 = muscle_spring_stiffness(&s);
        muscle_spring_activate(&mut s, 1.0);
        let k1 = muscle_spring_stiffness(&s);
        assert!(k1 > k0);
    }

    #[test]
    fn test_energy_nonneg_at_rest() {
        let s = new_muscle_spring(default_muscle_spring_config());
        assert!(muscle_spring_energy(&s) >= 0.0);
    }

    #[test]
    fn test_reset() {
        let mut s = new_muscle_spring(default_muscle_spring_config());
        muscle_spring_set_length(&mut s, 1.5, 1.0);
        muscle_spring_activate(&mut s, 0.8);
        muscle_spring_reset(&mut s);
        assert!((muscle_spring_length(&s) - 1.0).abs() < 1e-6);
        assert!((s.activation - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json_fields() {
        let s = new_muscle_spring(default_muscle_spring_config());
        let json = muscle_spring_to_json(&s);
        assert!(json.contains("rest_length"));
        assert!(json.contains("activation"));
        assert!(json.contains("damping"));
    }
}
