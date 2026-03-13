//! Raycast-based wheel physics collider for vehicle simulation.
//!
//! Models a single wheel as a spring-damper suspension system with a simplified
//! friction model.  The "raycast" is a vertical downward ray from the wheel hub.

// ── public structs ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Static configuration for a wheel collider.
pub struct WheelConfig {
    /// Wheel radius (meters).
    pub radius: f32,
    /// Suspension rest length (meters).
    pub suspension_length: f32,
    /// Spring stiffness (N/m).
    pub spring_stiffness: f32,
    /// Damping coefficient (N·s/m).
    pub damping: f32,
    /// Peak friction coefficient (Pacejka-like, simplified).
    pub friction_coeff: f32,
    /// Mass of the wheel assembly (kg).
    pub mass: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Information about the current wheel-ground contact.
pub struct WheelContactInfo {
    /// World-space contact point.
    pub point: [f32; 3],
    /// Ground normal at the contact point.
    pub normal: [f32; 3],
    /// Distance from wheel hub to contact point.
    pub distance: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Runtime state for a single wheel collider.
pub struct WheelCollider {
    /// Static configuration.
    pub config: WheelConfig,
    /// Current contact information, if grounded.
    pub contact: Option<WheelContactInfo>,
    /// Current angular velocity (rad/s, positive = forward roll).
    pub angular_velocity: f32,
    /// Current steering angle (degrees).
    pub steering_angle: f32,
    /// Current compression of the suspension spring (meters).
    pub suspension_compression: f32,
    /// Whether the collider is enabled.
    pub enabled: bool,
}

// ── public functions ──────────────────────────────────────────────────────────

#[allow(dead_code)]
/// Returns a [`WheelConfig`] with sensible defaults.
pub fn default_wheel_config() -> WheelConfig {
    WheelConfig {
        radius: 0.35,
        suspension_length: 0.3,
        spring_stiffness: 20_000.0,
        damping: 2_000.0,
        friction_coeff: 1.0,
        mass: 20.0,
    }
}

#[allow(dead_code)]
/// Creates a new [`WheelCollider`] from the given configuration.
pub fn new_wheel_collider(cfg: &WheelConfig) -> WheelCollider {
    WheelCollider {
        config: cfg.clone(),
        contact: None,
        angular_velocity: 0.0,
        steering_angle: 0.0,
        suspension_compression: 0.0,
        enabled: true,
    }
}

#[allow(dead_code)]
/// Performs a simplified "raycast" downward from `ray_origin` along `ray_dir`.
///
/// The ground is modelled as an infinite plane at y = 0.  Returns contact info
/// if the ray hits within `max_dist`.
pub fn wheel_raycast(
    wheel: &WheelCollider,
    ray_origin: [f32; 3],
    ray_dir: [f32; 3],
    max_dist: f32,
) -> Option<WheelContactInfo> {
    if !wheel.enabled {
        return None;
    }
    // Simple plane-at-y=0 intersection.
    let dir_y = ray_dir[1];
    if dir_y.abs() < f32::EPSILON {
        return None;
    }
    let t = -ray_origin[1] / dir_y;
    if t < 0.0 || t > max_dist {
        return None;
    }
    let point = [
        ray_origin[0] + ray_dir[0] * t,
        0.0,
        ray_origin[2] + ray_dir[2] * t,
    ];
    Some(WheelContactInfo {
        point,
        normal: [0.0, 1.0, 0.0],
        distance: t,
    })
}

#[allow(dead_code)]
/// Returns `true` if the wheel currently has ground contact.
pub fn wheel_is_grounded(wheel: &WheelCollider) -> bool {
    wheel.contact.is_some()
}

#[allow(dead_code)]
/// Computes the suspension spring force (Newtons) at the current compression.
pub fn wheel_suspension_force(wheel: &WheelCollider) -> f32 {
    wheel.config.spring_stiffness * wheel.suspension_compression
}

#[allow(dead_code)]
/// Returns the friction force given a longitudinal slip value.
///
/// Uses a very simplified Pacejka-inspired formula: F = μ · slip · mass · g.
pub fn wheel_friction_force(wheel: &WheelCollider, slip: f32) -> f32 {
    const GRAVITY: f32 = 9.81;
    let normal_force = wheel.config.mass * GRAVITY;
    wheel.config.friction_coeff * slip.clamp(-1.0, 1.0) * normal_force
}

#[allow(dead_code)]
/// Steps the wheel simulation by `dt` seconds, applying `throttle` [0,1] and
/// `brake` [0,1].
pub fn step_wheel(wheel: &mut WheelCollider, dt: f32, throttle: f32, brake: f32) {
    if !wheel.enabled {
        return;
    }

    // Drive torque increases angular velocity; brake decelerates it.
    const MAX_TORQUE: f32 = 500.0; // N·m
    const BRAKE_TORQUE: f32 = 800.0;

    let drive = throttle.clamp(0.0, 1.0) * MAX_TORQUE;
    let braking = brake.clamp(0.0, 1.0) * BRAKE_TORQUE;

    let r = wheel.config.radius.max(f32::EPSILON);
    let alpha = (drive - wheel.angular_velocity.signum() * braking) / (wheel.config.mass * r * r);
    wheel.angular_velocity += alpha * dt;

    // Rolling resistance
    wheel.angular_velocity *= 1.0 - 0.02 * dt;

    // Update suspension: raycast downward from a hub position 1 m above ground.
    let hub = [0.0_f32, wheel.config.suspension_length + wheel.config.radius, 0.0];
    let ray_dir = [0.0_f32, -1.0, 0.0];
    let max_ray = wheel.config.suspension_length + wheel.config.radius + 0.5;

    if let Some(contact) = wheel_raycast(wheel, hub, ray_dir, max_ray) {
        let compress = (wheel.config.suspension_length + wheel.config.radius - contact.distance)
            .clamp(0.0, wheel.config.suspension_length);
        wheel.suspension_compression = compress;
        wheel.contact = Some(contact);
    } else {
        wheel.suspension_compression = 0.0;
        wheel.contact = None;
    }
}

#[allow(dead_code)]
/// Returns the current angular velocity of the wheel (rad/s).
pub fn wheel_angular_velocity(wheel: &WheelCollider) -> f32 {
    wheel.angular_velocity
}

#[allow(dead_code)]
/// Returns the current contact point in world space, if grounded.
pub fn wheel_contact_point(wheel: &WheelCollider) -> Option<[f32; 3]> {
    wheel.contact.as_ref().map(|c| c.point)
}

#[allow(dead_code)]
/// Sets the steering angle of the wheel (degrees).
pub fn set_wheel_steering_angle(wheel: &mut WheelCollider, angle_deg: f32) {
    wheel.steering_angle = angle_deg;
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_wheel() -> WheelCollider {
        new_wheel_collider(&default_wheel_config())
    }

    #[test]
    fn test_default_config_radius() {
        let cfg = default_wheel_config();
        assert!(cfg.radius > 0.0);
    }

    #[test]
    fn test_new_wheel_collider_not_grounded() {
        let w = make_wheel();
        assert!(!wheel_is_grounded(&w));
    }

    #[test]
    fn test_wheel_raycast_hits_ground() {
        let w = make_wheel();
        // Ray from above, pointing down
        let origin = [0.0, 1.0, 0.0];
        let dir = [0.0, -1.0, 0.0];
        let contact = wheel_raycast(&w, origin, dir, 5.0);
        assert!(contact.is_some());
        let c = contact.expect("should succeed");
        assert!((c.point[1]).abs() < 1e-5);
        assert!((c.distance - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_wheel_raycast_misses_when_pointing_up() {
        let w = make_wheel();
        let contact = wheel_raycast(&w, [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], 10.0);
        assert!(contact.is_none());
    }

    #[test]
    fn test_step_wheel_increases_angular_velocity() {
        let mut w = make_wheel();
        step_wheel(&mut w, 0.016, 1.0, 0.0);
        assert!(wheel_angular_velocity(&w) > 0.0);
    }

    #[test]
    fn test_wheel_suspension_force_zero_when_uncompressed() {
        let w = make_wheel();
        assert_eq!(wheel_suspension_force(&w), 0.0);
    }

    #[test]
    fn test_wheel_friction_force_positive_slip() {
        let w = make_wheel();
        let f = wheel_friction_force(&w, 0.5);
        assert!(f > 0.0);
    }

    #[test]
    fn test_set_steering_angle() {
        let mut w = make_wheel();
        set_wheel_steering_angle(&mut w, 25.0);
        assert!((w.steering_angle - 25.0).abs() < 1e-5);
    }

    #[test]
    fn test_step_wheel_grounded_after_step() {
        let mut w = make_wheel();
        step_wheel(&mut w, 0.016, 0.0, 0.0);
        // The internal raycast from hub (y = suspension+radius) should hit y=0.
        assert!(wheel_is_grounded(&w), "wheel should be grounded after step");
    }
}
