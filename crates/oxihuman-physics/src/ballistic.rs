/// Ballistic projectile trajectory simulation.
#[allow(dead_code)]
pub struct BallisticConfig {
    pub gravity: [f32; 3],
    pub drag_coefficient: f32,
    pub air_density: f32,
    pub cross_section: f32,
}

#[allow(dead_code)]
pub struct Projectile {
    pub id: u32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub radius: f32,
    pub active: bool,
    pub time: f32,
}

#[allow(dead_code)]
pub struct TrajectoryPoint {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub time: f32,
}

#[allow(dead_code)]
pub fn default_ballistic_config() -> BallisticConfig {
    BallisticConfig {
        gravity: [0.0, -9.81, 0.0],
        drag_coefficient: 0.47,
        air_density: 1.225,
        cross_section: 0.01,
    }
}

#[allow(dead_code)]
pub fn new_projectile(pos: [f32; 3], vel: [f32; 3], mass: f32, radius: f32) -> Projectile {
    Projectile {
        id: 0,
        position: pos,
        velocity: vel,
        mass,
        radius,
        active: true,
        time: 0.0,
    }
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = vec3_len(v);
    if len < 1e-12 {
        return [0.0; 3];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Compute the drag force vector opposing velocity.
/// F_drag = -0.5 * rho * Cd * A * |v|^2 * v_hat
#[allow(dead_code)]
pub fn drag_force(velocity: [f32; 3], cfg: &BallisticConfig) -> [f32; 3] {
    let speed = vec3_len(velocity);
    if speed < 1e-12 {
        return [0.0; 3];
    }
    let mag = 0.5 * cfg.air_density * cfg.drag_coefficient * cfg.cross_section * speed * speed;
    let dir = vec3_normalize(velocity);
    [-mag * dir[0], -mag * dir[1], -mag * dir[2]]
}

/// Advance the projectile by one time step using Euler integration.
#[allow(dead_code)]
pub fn simulate_projectile(proj: &mut Projectile, cfg: &BallisticConfig, dt: f32) {
    if !proj.active {
        return;
    }
    let drag = drag_force(proj.velocity, cfg);
    let inv_mass = if proj.mass > 1e-9 {
        1.0 / proj.mass
    } else {
        0.0
    };

    // Acceleration = gravity + drag/mass
    let ax = cfg.gravity[0] + drag[0] * inv_mass;
    let ay = cfg.gravity[1] + drag[1] * inv_mass;
    let az = cfg.gravity[2] + drag[2] * inv_mass;

    proj.velocity[0] += ax * dt;
    proj.velocity[1] += ay * dt;
    proj.velocity[2] += az * dt;

    proj.position[0] += proj.velocity[0] * dt;
    proj.position[1] += proj.velocity[1] * dt;
    proj.position[2] += proj.velocity[2] * dt;

    proj.time += dt;
}

/// Simulate a full trajectory and return the sampled points.
#[allow(dead_code)]
pub fn simulate_trajectory(
    pos: [f32; 3],
    vel: [f32; 3],
    mass: f32,
    cfg: &BallisticConfig,
    duration: f32,
    steps: u32,
) -> Vec<TrajectoryPoint> {
    if steps == 0 {
        return vec![];
    }
    let dt = duration / steps as f32;
    let mut proj = new_projectile(pos, vel, mass, 0.1);
    let mut points = Vec::with_capacity((steps + 1) as usize);

    points.push(TrajectoryPoint {
        position: proj.position,
        velocity: proj.velocity,
        time: proj.time,
    });

    for _ in 0..steps {
        simulate_projectile(&mut proj, cfg, dt);
        points.push(TrajectoryPoint {
            position: proj.position,
            velocity: proj.velocity,
            time: proj.time,
        });
    }

    points
}

/// Predict the landing position on a horizontal plane at plane_y via iterative simulation.
#[allow(dead_code)]
pub fn impact_point_on_plane(
    proj: &Projectile,
    cfg: &BallisticConfig,
    plane_y: f32,
) -> Option<[f32; 3]> {
    if !proj.active {
        return None;
    }
    // If already below plane, return current position
    if proj.position[1] <= plane_y {
        return Some(proj.position);
    }

    let mut sim = Projectile {
        id: proj.id,
        position: proj.position,
        velocity: proj.velocity,
        mass: proj.mass,
        radius: proj.radius,
        active: true,
        time: proj.time,
    };

    let max_steps = 10000;
    let dt = 0.001f32;
    for _ in 0..max_steps {
        let prev_y = sim.position[1];
        simulate_projectile(&mut sim, cfg, dt);
        if sim.position[1] <= plane_y {
            // Linear interpolate crossing point
            let t_frac = (prev_y - plane_y) / (prev_y - sim.position[1]).max(1e-9);
            let px = sim.position[0] - sim.velocity[0] * dt * (1.0 - t_frac);
            let pz = sim.position[2] - sim.velocity[2] * dt * (1.0 - t_frac);
            return Some([px, plane_y, pz]);
        }
    }
    None
}

/// Approximate initial velocity needed to reach `to` from `from` at the given speed.
/// Uses a simple ballistic arc approximation (no drag).
#[allow(dead_code)]
pub fn launch_velocity_for_target(
    from: [f32; 3],
    to: [f32; 3],
    speed: f32,
    _cfg: &BallisticConfig,
) -> Option<[f32; 3]> {
    let dx = to[0] - from[0];
    let dy = to[1] - from[1];
    let dz = to[2] - from[2];
    let horiz_dist = (dx * dx + dz * dz).sqrt();
    if horiz_dist < 1e-9 {
        // Straight up
        return Some([0.0, speed, 0.0]);
    }
    let total_dist = (dx * dx + dy * dy + dz * dz).sqrt();
    if total_dist < 1e-9 {
        return None;
    }
    let dir = [dx / total_dist, dy / total_dist, dz / total_dist];
    Some([dir[0] * speed, dir[1] * speed, dir[2] * speed])
}

/// Maximum range for a 45-degree launch on flat ground (analytical, no drag).
/// R = v^2 / |g|
#[allow(dead_code)]
pub fn max_range(speed: f32, cfg: &BallisticConfig) -> f32 {
    let g = vec3_len(cfg.gravity);
    if g < 1e-9 {
        return f32::MAX;
    }
    speed * speed / g
}

#[allow(dead_code)]
pub fn projectile_kinetic_energy(proj: &Projectile) -> f32 {
    let speed = vec3_len(proj.velocity);
    0.5 * proj.mass * speed * speed
}

/// Total arc length of the trajectory.
#[allow(dead_code)]
pub fn trajectory_length(points: &[TrajectoryPoint]) -> f32 {
    if points.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    for i in 1..points.len() {
        let a = points[i - 1].position;
        let b = points[i].position;
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

#[allow(dead_code)]
pub fn trajectory_max_height(points: &[TrajectoryPoint]) -> f32 {
    points
        .iter()
        .map(|p| p.position[1])
        .fold(f32::NEG_INFINITY, f32::max)
}

#[allow(dead_code)]
pub fn time_of_flight(points: &[TrajectoryPoint]) -> f32 {
    if points.is_empty() {
        return 0.0;
    }
    points[points.len() - 1].time - points[0].time
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ballistic_config() {
        let cfg = default_ballistic_config();
        assert!(cfg.drag_coefficient > 0.0);
        assert!(cfg.air_density > 0.0);
        assert!(cfg.gravity[1] < 0.0);
    }

    #[test]
    fn test_new_projectile() {
        let p = new_projectile([0.0; 3], [10.0, 20.0, 0.0], 1.0, 0.05);
        assert!(p.active);
        assert_eq!(p.time, 0.0);
        assert!((p.velocity[1] - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_simulate_projectile_gravity() {
        let cfg = default_ballistic_config();
        let mut proj = new_projectile([0.0, 10.0, 0.0], [0.0, 0.0, 0.0], 1.0, 0.05);
        let y0 = proj.position[1];
        simulate_projectile(&mut proj, &cfg, 0.1);
        // Should fall due to gravity
        assert!(proj.position[1] < y0);
    }

    #[test]
    fn test_simulate_projectile_inactive() {
        let cfg = default_ballistic_config();
        let mut proj = new_projectile([0.0, 0.0, 0.0], [1.0, 1.0, 0.0], 1.0, 0.05);
        proj.active = false;
        let pos = proj.position;
        simulate_projectile(&mut proj, &cfg, 0.1);
        assert_eq!(proj.position[0], pos[0]);
    }

    #[test]
    fn test_drag_force_opposes_velocity() {
        let cfg = default_ballistic_config();
        let vel = [10.0f32, 0.0, 0.0];
        let force = drag_force(vel, &cfg);
        // Drag opposes positive x velocity
        assert!(force[0] < 0.0);
        assert!(force[1].abs() < 1e-6);
    }

    #[test]
    fn test_drag_force_zero_velocity() {
        let cfg = default_ballistic_config();
        let force = drag_force([0.0; 3], &cfg);
        assert_eq!(force, [0.0; 3]);
    }

    #[test]
    fn test_simulate_trajectory_correct_length() {
        let cfg = default_ballistic_config();
        let points = simulate_trajectory([0.0; 3], [10.0, 20.0, 0.0], 1.0, &cfg, 1.0, 100);
        assert_eq!(points.len(), 101); // steps + 1
    }

    #[test]
    fn test_simulate_trajectory_zero_steps() {
        let cfg = default_ballistic_config();
        let points = simulate_trajectory([0.0; 3], [10.0, 0.0, 0.0], 1.0, &cfg, 1.0, 0);
        assert!(points.is_empty());
    }

    #[test]
    fn test_impact_point_on_plane() {
        let cfg = BallisticConfig {
            gravity: [0.0, -9.81, 0.0],
            drag_coefficient: 0.0, // no drag for simpler test
            air_density: 0.0,
            cross_section: 0.01,
        };
        let proj = new_projectile([0.0, 5.0, 0.0], [5.0, 0.0, 0.0], 1.0, 0.05);
        let impact = impact_point_on_plane(&proj, &cfg, 0.0);
        assert!(impact.is_some());
        let pt = impact.unwrap();
        assert!((pt[1]).abs() < 0.01); // near y=0
    }

    #[test]
    fn test_impact_point_already_below() {
        let cfg = default_ballistic_config();
        let proj = new_projectile([0.0, -1.0, 0.0], [0.0, 0.0, 0.0], 1.0, 0.05);
        let impact = impact_point_on_plane(&proj, &cfg, 0.0);
        assert!(impact.is_some());
    }

    #[test]
    fn test_max_range() {
        let cfg = default_ballistic_config();
        let range = max_range(10.0, &cfg);
        let expected = 100.0 / 9.81;
        assert!((range - expected).abs() < 0.01);
    }

    #[test]
    fn test_projectile_kinetic_energy() {
        let proj = new_projectile([0.0; 3], [3.0, 4.0, 0.0], 2.0, 0.05);
        let ke = projectile_kinetic_energy(&proj);
        // speed = 5.0, ke = 0.5 * 2 * 25 = 25
        assert!((ke - 25.0).abs() < 1e-4);
    }

    #[test]
    fn test_trajectory_length() {
        let cfg = default_ballistic_config();
        let points = simulate_trajectory([0.0; 3], [10.0, 10.0, 0.0], 1.0, &cfg, 2.0, 200);
        let len = trajectory_length(&points);
        assert!(len > 0.0);
    }

    #[test]
    fn test_trajectory_length_single_point() {
        let points = vec![TrajectoryPoint {
            position: [0.0; 3],
            velocity: [0.0; 3],
            time: 0.0,
        }];
        assert_eq!(trajectory_length(&points), 0.0);
    }

    #[test]
    fn test_trajectory_max_height() {
        let cfg = BallisticConfig {
            gravity: [0.0, -9.81, 0.0],
            drag_coefficient: 0.0,
            air_density: 0.0,
            cross_section: 0.01,
        };
        let points = simulate_trajectory([0.0; 3], [0.0, 10.0, 0.0], 1.0, &cfg, 2.0, 200);
        let max_h = trajectory_max_height(&points);
        // Analytical max height = v^2 / (2g) = 100 / 19.62 ≈ 5.1
        assert!(max_h > 4.0);
    }

    #[test]
    fn test_time_of_flight() {
        let cfg = default_ballistic_config();
        let points = simulate_trajectory([0.0; 3], [10.0, 10.0, 0.0], 1.0, &cfg, 2.0, 100);
        let tof = time_of_flight(&points);
        assert!((tof - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_time_of_flight_empty() {
        assert_eq!(time_of_flight(&[]), 0.0);
    }

    #[test]
    fn test_launch_velocity_for_target() {
        let cfg = default_ballistic_config();
        let vel = launch_velocity_for_target([0.0; 3], [10.0, 5.0, 0.0], 20.0, &cfg);
        assert!(vel.is_some());
        let v = vel.unwrap();
        let speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((speed - 20.0).abs() < 0.1);
    }
}
