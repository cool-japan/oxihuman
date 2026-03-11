//! Buoyancy simulation using Archimedes' principle.
//!
//! Computes buoyant forces on rigid bodies partially or fully submerged in a
//! fluid. Bodies are treated as uniform-density boxes for the purpose of
//! submerged-volume estimation.

/// Configuration for the buoyancy simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BuoyancyConfig {
    /// Density of the fluid (kg/m³). Water ≈ 1000.
    pub water_density: f32,
    /// Gravitational acceleration (m/s²).
    pub gravity: f32,
}

/// A rigid body subject to buoyancy.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BuoyantBody {
    /// Mass of the body (kg).
    pub mass: f32,
    /// Total volume of the body (m³).
    pub volume: f32,
    /// Linear drag coefficient.
    pub drag: f32,
    /// Vertical velocity (m/s), positive = upward.
    pub velocity_y: f32,
    /// Half-height of the body (used for submerged fraction).
    pub half_height: f32,
}

/// Result of a single buoyancy computation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BuoyancyResult {
    /// Net buoyant force (N), positive = upward.
    pub force: f32,
    /// Fraction of the body currently submerged [0, 1].
    pub submerged_fraction: f32,
    /// Whether the body is floating (net upward force > 0 or partially submerged).
    pub floating: bool,
}

/// Returns the default buoyancy configuration (water, standard gravity).
#[allow(dead_code)]
pub fn default_buoyancy_config() -> BuoyancyConfig {
    BuoyancyConfig {
        water_density: 1000.0,
        gravity: 9.81,
    }
}

/// Creates a new buoyant body with the given mass, volume, and drag coefficient.
///
/// The half-height is estimated from the volume assuming a cube shape.
#[allow(dead_code)]
pub fn new_buoyant_body(mass: f32, volume: f32, drag: f32) -> BuoyantBody {
    let side = volume.cbrt();
    BuoyantBody {
        mass,
        volume,
        drag,
        velocity_y: 0.0,
        half_height: side * 0.5,
    }
}

/// Sets the fluid density in a configuration.
#[allow(dead_code)]
pub fn set_water_density(cfg: &mut BuoyancyConfig, density: f32) {
    cfg.water_density = density.max(0.0);
}

/// Computes the buoyancy result for a body whose center is at `body_center_y`.
///
/// The submerged fraction is derived by comparing the body's vertical extent
/// [center - half_height, center + half_height] to `water_level`.
#[allow(dead_code)]
pub fn compute_buoyancy(
    body: &BuoyantBody,
    water_level: f32,
    body_center_y: f32,
    cfg: &BuoyancyConfig,
) -> BuoyancyResult {
    let top = body_center_y + body.half_height;
    let bottom = body_center_y - body.half_height;
    let height = top - bottom; // = 2 * half_height

    let submerged_fraction = if water_level <= bottom {
        0.0f32
    } else if water_level >= top {
        1.0f32
    } else {
        ((water_level - bottom) / height).clamp(0.0, 1.0)
    };

    let submerged_volume = body.volume * submerged_fraction;
    let buoyant_force = cfg.water_density * cfg.gravity * submerged_volume;
    let gravity_force = body.mass * cfg.gravity;
    let net_force = buoyant_force - gravity_force;

    BuoyancyResult {
        force: net_force,
        submerged_fraction,
        floating: submerged_fraction > 0.0,
    }
}

/// Returns the net buoyant force from a result.
#[allow(dead_code)]
pub fn buoyancy_force(result: &BuoyancyResult) -> f32 {
    result.force
}

/// Returns the submerged fraction from a result.
#[allow(dead_code)]
pub fn buoyancy_submerged_fraction(result: &BuoyancyResult) -> f32 {
    result.submerged_fraction
}

/// Returns whether the body is floating (partially or fully submerged).
#[allow(dead_code)]
pub fn buoyancy_is_floating(result: &BuoyancyResult) -> bool {
    result.floating
}

/// Computes the drag force magnitude on a body given its current velocity.
///
/// Drag = 0.5 * drag_coefficient * water_density * velocity²  (with sign).
#[allow(dead_code)]
pub fn buoyancy_drag_force(body: &BuoyantBody, cfg: &BuoyancyConfig) -> f32 {
    let v = body.velocity_y;
    -0.5 * body.drag * cfg.water_density * v * v.abs()
}

/// Returns the current vertical velocity of the body.
#[allow(dead_code)]
pub fn buoyant_body_velocity(body: &BuoyantBody) -> f32 {
    body.velocity_y
}

/// Steps the body forward by `dt` seconds under buoyancy and drag forces.
///
/// The body's position is tracked implicitly through its `velocity_y`; callers
/// should maintain `body_center_y` externally.
#[allow(dead_code)]
pub fn step_buoyant_body(
    body: &mut BuoyantBody,
    water_level: f32,
    dt: f32,
    cfg: &BuoyancyConfig,
) {
    // We need an external center_y; approximate using velocity integration.
    // Since we don't store position here, use 0 as a base (caller manages it).
    // We expose velocity so callers can integrate position separately.
    // Compute forces at current velocity.
    let drag = buoyancy_drag_force(body, cfg);
    let gravity_force = -body.mass * cfg.gravity;
    // Buoyancy with center at y=0 relative reference.
    let result = compute_buoyancy(body, water_level, 0.0, cfg);
    let buoyant = result.force - gravity_force; // net = buoyant_upward - gravity (already included in result.force)
    // result.force = buoyant_upward - gravity, so total = result.force + drag
    let total_force = result.force + drag;
    let accel = if body.mass > 0.0 { total_force / body.mass } else { 0.0 };
    body.velocity_y += accel * dt;
    let _ = buoyant; // suppress unused warning
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_buoyancy_config();
        assert!((cfg.water_density - 1000.0).abs() < 1.0);
        assert!((cfg.gravity - 9.81).abs() < 0.01);
    }

    #[test]
    fn test_fully_submerged() {
        let cfg = default_buoyancy_config();
        // 1 m³ body, mass=500 kg → should float (buoyant > gravity).
        let body = new_buoyant_body(500.0, 1.0, 0.0);
        let result = compute_buoyancy(&body, 100.0, 0.0, &cfg);
        assert!((result.submerged_fraction - 1.0).abs() < 1e-5);
        assert!(result.force > 0.0, "net force should be upward: {}", result.force);
        assert!(buoyancy_is_floating(&result));
    }

    #[test]
    fn test_not_submerged() {
        let cfg = default_buoyancy_config();
        let body = new_buoyant_body(100.0, 1.0, 0.0);
        // Water is far below.
        let result = compute_buoyancy(&body, -10.0, 0.0, &cfg);
        assert_eq!(result.submerged_fraction, 0.0);
        assert!(!buoyancy_is_floating(&result));
    }

    #[test]
    fn test_partial_submersion() {
        let cfg = default_buoyancy_config();
        let body = new_buoyant_body(100.0, 1.0, 0.0);
        // Center at 0, half_height ≈ 0.5. Water at 0 → 50 % submerged.
        let result = compute_buoyancy(&body, 0.0, 0.0, &cfg);
        assert!((result.submerged_fraction - 0.5).abs() < 0.02);
    }

    #[test]
    fn test_heavy_body_sinks() {
        let cfg = default_buoyancy_config();
        // 2000 kg in 1 m³ → denser than water → net force negative.
        let body = new_buoyant_body(2000.0, 1.0, 0.0);
        let result = compute_buoyancy(&body, 100.0, 0.0, &cfg);
        assert!(result.force < 0.0, "heavy body should have net downward force");
    }

    #[test]
    fn test_set_water_density() {
        let mut cfg = default_buoyancy_config();
        set_water_density(&mut cfg, 800.0);
        assert!((cfg.water_density - 800.0).abs() < 1.0);
    }

    #[test]
    fn test_drag_force_direction() {
        let cfg = default_buoyancy_config();
        let mut body = new_buoyant_body(100.0, 1.0, 0.1);
        body.velocity_y = 2.0; // moving up
        let drag = buoyancy_drag_force(&body, &cfg);
        assert!(drag < 0.0, "drag on upward motion should be downward: {}", drag);
    }

    #[test]
    fn test_step_applies_gravity() {
        let cfg = default_buoyancy_config();
        let mut body = new_buoyant_body(10000.0, 0.001, 0.0);
        body.velocity_y = 0.0;
        // Very heavy, tiny volume → gravity dominates → velocity becomes negative.
        step_buoyant_body(&mut body, -100.0, 0.1, &cfg);
        assert!(body.velocity_y < 0.0, "should fall: v={}", body.velocity_y);
    }

    #[test]
    fn test_buoyancy_force_accessor() {
        let cfg = default_buoyancy_config();
        let body = new_buoyant_body(500.0, 1.0, 0.0);
        let result = compute_buoyancy(&body, 100.0, 0.0, &cfg);
        assert_eq!(buoyancy_force(&result), result.force);
    }

    #[test]
    fn test_submerged_fraction_clamped() {
        let cfg = default_buoyancy_config();
        let body = new_buoyant_body(100.0, 1.0, 0.0);
        let r1 = compute_buoyancy(&body, 1000.0, 0.0, &cfg);
        let r2 = compute_buoyancy(&body, -1000.0, 0.0, &cfg);
        assert!(buoyancy_submerged_fraction(&r1) <= 1.0);
        assert!(buoyancy_submerged_fraction(&r2) >= 0.0);
    }
}
