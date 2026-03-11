//! Debris fragment simulation after fracture/destruction.

/// Simple LCG for debris randomization (no rand dependency).
/// Returns a value in [0.0, 1.0).
fn lcg_rand_f32(seed: &mut u64) -> f32 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let bits = ((*seed >> 33) as u32) | 0x3F80_0000;
    f32::from_bits(bits) - 1.0
}

#[allow(dead_code)]
pub struct DebrisFragment {
    pub id: u32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub rotation: [f32; 4],
    pub mass: f32,
    pub restitution: f32,
    pub friction: f32,
    pub alive: bool,
    pub lifetime: f32,
    pub age: f32,
}

#[allow(dead_code)]
pub struct DebrisSystem {
    pub fragments: Vec<DebrisFragment>,
    pub gravity: [f32; 3],
    pub next_id: u32,
    pub floor_y: Option<f32>,
}

#[allow(dead_code)]
pub struct DebrisConfig {
    pub max_fragments: usize,
    pub lifetime: f32,
    pub gravity: [f32; 3],
    pub restitution: f32,
    pub friction: f32,
}

/// Returns a default debris configuration.
#[allow(dead_code)]
pub fn default_debris_config() -> DebrisConfig {
    DebrisConfig {
        max_fragments: 64,
        lifetime: 5.0,
        gravity: [0.0, -9.81, 0.0],
        restitution: 0.3,
        friction: 0.5,
    }
}

/// Create a new debris system.
#[allow(dead_code)]
pub fn new_debris_system(gravity: [f32; 3]) -> DebrisSystem {
    DebrisSystem {
        fragments: Vec::new(),
        gravity,
        next_id: 0,
        floor_y: None,
    }
}

/// Spawn a single fragment and return its id.
#[allow(dead_code)]
pub fn spawn_fragment(
    sys: &mut DebrisSystem,
    pos: [f32; 3],
    vel: [f32; 3],
    mass: f32,
    cfg: &DebrisConfig,
) -> u32 {
    let id = sys.next_id;
    sys.next_id += 1;
    sys.fragments.push(DebrisFragment {
        id,
        position: pos,
        velocity: vel,
        angular_velocity: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        mass: mass.max(0.001),
        restitution: cfg.restitution,
        friction: cfg.friction,
        alive: true,
        lifetime: cfg.lifetime,
        age: 0.0,
    });
    id
}

/// Spawn fragments in random directions from an origin.
#[allow(dead_code)]
pub fn spawn_explosion(
    sys: &mut DebrisSystem,
    origin: [f32; 3],
    count: usize,
    force: f32,
    cfg: &DebrisConfig,
) {
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    for _ in 0..count {
        let vx = lcg_rand_f32(&mut seed) * 2.0 - 1.0;
        let vy = lcg_rand_f32(&mut seed) * 2.0 - 1.0;
        let vz = lcg_rand_f32(&mut seed) * 2.0 - 1.0;
        let len = (vx * vx + vy * vy + vz * vz).sqrt().max(1e-8);
        let vel = [vx / len * force, vy / len * force, vz / len * force];
        spawn_fragment(sys, origin, vel, 1.0, cfg);
    }
}

/// Advance the debris simulation by dt seconds.
#[allow(dead_code)]
pub fn step_debris(sys: &mut DebrisSystem, dt: f32) {
    let gx = sys.gravity[0];
    let gy = sys.gravity[1];
    let gz = sys.gravity[2];
    let floor_y = sys.floor_y;

    for frag in sys.fragments.iter_mut() {
        if !frag.alive {
            continue;
        }

        // Euler integration
        frag.velocity[0] += gx * dt;
        frag.velocity[1] += gy * dt;
        frag.velocity[2] += gz * dt;

        frag.position[0] += frag.velocity[0] * dt;
        frag.position[1] += frag.velocity[1] * dt;
        frag.position[2] += frag.velocity[2] * dt;

        // Floor bounce
        if let Some(fy) = floor_y {
            if frag.position[1] < fy && frag.velocity[1] < 0.0 {
                frag.position[1] = fy;
                frag.velocity[1] = -frag.velocity[1] * frag.restitution;
                frag.velocity[0] *= 1.0 - frag.friction * dt;
                frag.velocity[2] *= 1.0 - frag.friction * dt;
            }
        }

        // Lifetime
        frag.age += dt;
        if frag.age >= frag.lifetime {
            frag.alive = false;
        }
    }
}

/// Count fragments that are still alive.
#[allow(dead_code)]
pub fn living_fragment_count(sys: &DebrisSystem) -> usize {
    sys.fragments.iter().filter(|f| f.alive).count()
}

/// Remove dead fragments from the system.
#[allow(dead_code)]
pub fn remove_dead(sys: &mut DebrisSystem) {
    sys.fragments.retain(|f| f.alive);
}

/// Apply wind drag to all living fragments.
#[allow(dead_code)]
pub fn apply_wind_to_debris(sys: &mut DebrisSystem, wind: [f32; 3], drag: f32) {
    for frag in sys.fragments.iter_mut() {
        if !frag.alive {
            continue;
        }
        frag.velocity[0] += (wind[0] - frag.velocity[0]) * drag;
        frag.velocity[1] += (wind[1] - frag.velocity[1]) * drag;
        frag.velocity[2] += (wind[2] - frag.velocity[2]) * drag;
    }
}

/// Kinetic energy of a single fragment: 0.5 * m * v^2.
#[allow(dead_code)]
pub fn fragment_kinetic_energy(frag: &DebrisFragment) -> f32 {
    let v2 = frag.velocity[0].powi(2) + frag.velocity[1].powi(2) + frag.velocity[2].powi(2);
    0.5 * frag.mass * v2
}

/// Sum of kinetic energies of all living fragments.
#[allow(dead_code)]
pub fn total_kinetic_energy(sys: &DebrisSystem) -> f32 {
    sys.fragments
        .iter()
        .filter(|f| f.alive)
        .map(fragment_kinetic_energy)
        .sum()
}

/// Set a floor y-plane for bounce collisions.
#[allow(dead_code)]
pub fn set_floor(sys: &mut DebrisSystem, y: f32) {
    sys.floor_y = Some(y);
}

/// Compute the axis-aligned bounding box of all living fragments.
#[allow(dead_code)]
pub fn debris_bounding_box(sys: &DebrisSystem) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    let mut any = false;
    for frag in sys.fragments.iter().filter(|f| f.alive) {
        for k in 0..3 {
            if frag.position[k] < min[k] {
                min[k] = frag.position[k];
            }
            if frag.position[k] > max[k] {
                max[k] = frag.position[k];
            }
        }
        any = true;
    }
    if !any {
        ([0.0; 3], [0.0; 3])
    } else {
        (min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sys() -> DebrisSystem {
        new_debris_system([0.0, -9.81, 0.0])
    }

    fn make_cfg() -> DebrisConfig {
        default_debris_config()
    }

    #[test]
    fn test_default_config() {
        let cfg = make_cfg();
        assert!(cfg.lifetime > 0.0);
        assert!(cfg.max_fragments > 0);
    }

    #[test]
    fn test_spawn_fragment_id() {
        let mut sys = make_sys();
        let cfg = make_cfg();
        let id = spawn_fragment(&mut sys, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0, &cfg);
        assert_eq!(id, 0);
        assert_eq!(sys.fragments.len(), 1);
    }

    #[test]
    fn test_spawn_multiple_fragments() {
        let mut sys = make_sys();
        let cfg = make_cfg();
        spawn_fragment(&mut sys, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0, &cfg);
        spawn_fragment(&mut sys, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 2.0, &cfg);
        assert_eq!(sys.fragments.len(), 2);
        assert_eq!(sys.next_id, 2);
    }

    #[test]
    fn test_spawn_explosion_count() {
        let mut sys = make_sys();
        let cfg = make_cfg();
        spawn_explosion(&mut sys, [0.0, 0.0, 0.0], 10, 5.0, &cfg);
        assert_eq!(sys.fragments.len(), 10);
    }

    #[test]
    fn test_step_changes_position() {
        let mut sys = make_sys();
        let cfg = make_cfg();
        spawn_fragment(&mut sys, [0.0, 10.0, 0.0], [0.0, 0.0, 0.0], 1.0, &cfg);
        step_debris(&mut sys, 0.1);
        let y = sys.fragments[0].position[1];
        assert!(y < 10.0, "gravity should pull fragment down: y={}", y);
    }

    #[test]
    fn test_living_count() {
        let mut sys = make_sys();
        let cfg = make_cfg();
        spawn_fragment(&mut sys, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0, &cfg);
        assert_eq!(living_fragment_count(&sys), 1);
        sys.fragments[0].alive = false;
        assert_eq!(living_fragment_count(&sys), 0);
    }

    #[test]
    fn test_remove_dead() {
        let mut sys = make_sys();
        let cfg = make_cfg();
        spawn_fragment(&mut sys, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0, &cfg);
        sys.fragments[0].alive = false;
        remove_dead(&mut sys);
        assert!(sys.fragments.is_empty());
    }

    #[test]
    fn test_floor_bounce_reverses_y_velocity() {
        let mut sys = make_sys();
        set_floor(&mut sys, 0.0);
        let cfg = make_cfg();
        spawn_fragment(&mut sys, [0.0, 0.1, 0.0], [0.0, -5.0, 0.0], 1.0, &cfg);
        step_debris(&mut sys, 0.1);
        // Fragment should bounce
        let vy = sys.fragments[0].velocity[1];
        assert!(vy > 0.0, "vy should be positive after bounce: {}", vy);
    }

    #[test]
    fn test_kinetic_energy_positive() {
        let mut sys = make_sys();
        let cfg = make_cfg();
        spawn_fragment(&mut sys, [0.0, 0.0, 0.0], [1.0, 2.0, 3.0], 2.0, &cfg);
        let ke = fragment_kinetic_energy(&sys.fragments[0]);
        assert!(ke > 0.0);
        // 0.5 * 2 * (1+4+9) = 14
        assert!((ke - 14.0).abs() < 1e-4);
    }

    #[test]
    fn test_total_kinetic_energy() {
        let mut sys = make_sys();
        let cfg = make_cfg();
        spawn_fragment(&mut sys, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 2.0, &cfg);
        spawn_fragment(&mut sys, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 2.0, &cfg);
        let tke = total_kinetic_energy(&sys);
        assert!(tke > 0.0);
    }

    #[test]
    fn test_lifetime_kills_fragment() {
        let mut sys = make_sys();
        let cfg = DebrisConfig {
            lifetime: 0.1,
            ..make_cfg()
        };
        spawn_fragment(&mut sys, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0, &cfg);
        step_debris(&mut sys, 0.2);
        assert!(!sys.fragments[0].alive);
    }

    #[test]
    fn test_apply_wind() {
        let mut sys = make_sys();
        let cfg = make_cfg();
        spawn_fragment(&mut sys, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0, &cfg);
        apply_wind_to_debris(&mut sys, [10.0, 0.0, 0.0], 0.5);
        let vx = sys.fragments[0].velocity[0];
        assert!(vx > 0.0);
    }

    #[test]
    fn test_debris_bounding_box_empty() {
        let sys = make_sys();
        let (mn, mx) = debris_bounding_box(&sys);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
    }

    #[test]
    fn test_debris_bounding_box_contains_fragment() {
        let mut sys = make_sys();
        let cfg = make_cfg();
        spawn_fragment(&mut sys, [1.0, 2.0, 3.0], [0.0, 0.0, 0.0], 1.0, &cfg);
        let (mn, mx) = debris_bounding_box(&sys);
        assert!((mn[0] - 1.0).abs() < 1e-5);
        assert!((mx[2] - 3.0).abs() < 1e-5);
    }
}
