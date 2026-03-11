//! PBD granular material simulation (sand/grain particles).

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub struct Grain {
    pub id: u32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub radius: f32,
    pub mass: f32,
    pub active: bool,
}

#[allow(dead_code)]
pub struct GranularConfig {
    pub restitution: f32,
    pub friction: f32,
    pub gravity: [f32; 3],
    pub substeps: u32,
    pub damping: f32,
}

#[allow(dead_code)]
pub struct GranularWorld {
    pub grains: Vec<Grain>,
    pub config: GranularConfig,
    pub floor_y: f32,
    pub next_id: u32,
}

// ---------------------------------------------------------------------------
// Defaults & constructors
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn default_granular_config() -> GranularConfig {
    GranularConfig {
        restitution: 0.3,
        friction: 0.5,
        gravity: [0.0, -9.81, 0.0],
        substeps: 4,
        damping: 0.98,
    }
}

#[allow(dead_code)]
pub fn new_granular_world(floor_y: f32) -> GranularWorld {
    GranularWorld {
        grains: Vec::new(),
        config: default_granular_config(),
        floor_y,
        next_id: 0,
    }
}

// ---------------------------------------------------------------------------
// Core functions
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn add_grain(world: &mut GranularWorld, pos: [f32; 3], radius: f32, mass: f32) -> u32 {
    let id = world.next_id;
    world.next_id += 1;
    world.grains.push(Grain {
        id,
        position: pos,
        velocity: [0.0; 3],
        radius,
        mass,
        active: true,
    });
    id
}

/// Integrate all active grains under gravity, apply damping, then resolve collisions.
#[allow(dead_code)]
pub fn simulate_granular(world: &mut GranularWorld, dt: f32) {
    let sub_dt = if world.config.substeps > 0 {
        dt / world.config.substeps as f32
    } else {
        dt
    };
    let substeps = world.config.substeps.max(1);

    for _ in 0..substeps {
        let grav = world.config.gravity;
        let damping = world.config.damping;
        let floor_y = world.floor_y;
        let restitution = world.config.restitution;
        let friction = world.config.friction;

        // Integrate velocity and position.
        for grain in world.grains.iter_mut().filter(|g| g.active) {
            grain.velocity[0] += grav[0] * sub_dt;
            grain.velocity[1] += grav[1] * sub_dt;
            grain.velocity[2] += grav[2] * sub_dt;
            grain.velocity[0] *= damping;
            grain.velocity[1] *= damping;
            grain.velocity[2] *= damping;
            grain.position[0] += grain.velocity[0] * sub_dt;
            grain.position[1] += grain.velocity[1] * sub_dt;
            grain.position[2] += grain.velocity[2] * sub_dt;
        }

        // Floor resolution.
        for grain in world.grains.iter_mut().filter(|g| g.active) {
            resolve_grain_floor(grain, floor_y, restitution);
        }

        // Grain-grain collision (O(n^2) for simplicity).
        let n = world.grains.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let (left, right) = world.grains.split_at_mut(j);
                let a = &mut left[i];
                let b = &mut right[0];
                if a.active && b.active && grains_overlap(a, b) {
                    resolve_grain_grain(a, b, restitution, friction);
                }
            }
        }
    }
}

/// Push grain above floor and reflect vertical velocity.
#[allow(dead_code)]
pub fn resolve_grain_floor(grain: &mut Grain, floor_y: f32, restitution: f32) {
    let min_y = floor_y + grain.radius;
    if grain.position[1] < min_y {
        grain.position[1] = min_y;
        if grain.velocity[1] < 0.0 {
            grain.velocity[1] = -grain.velocity[1] * restitution;
        }
    }
}

/// Simple sphere-sphere collision response.
#[allow(dead_code)]
pub fn resolve_grain_grain(a: &mut Grain, b: &mut Grain, restitution: f32, friction: f32) {
    let dx = b.position[0] - a.position[0];
    let dy = b.position[1] - a.position[1];
    let dz = b.position[2] - a.position[2];
    let dist_sq = dx * dx + dy * dy + dz * dz;
    let min_dist = a.radius + b.radius;
    if dist_sq >= min_dist * min_dist || dist_sq < 1e-12 {
        return;
    }
    let dist = dist_sq.sqrt();
    let nx = dx / dist;
    let ny = dy / dist;
    let nz = dz / dist;

    // Separate.
    let overlap = min_dist - dist;
    let total_mass = a.mass + b.mass;
    let fa = b.mass / total_mass;
    let fb = a.mass / total_mass;
    a.position[0] -= nx * overlap * fa;
    a.position[1] -= ny * overlap * fa;
    a.position[2] -= nz * overlap * fa;
    b.position[0] += nx * overlap * fb;
    b.position[1] += ny * overlap * fb;
    b.position[2] += nz * overlap * fb;

    // Impulse along normal.
    let rv_n = (b.velocity[0] - a.velocity[0]) * nx
        + (b.velocity[1] - a.velocity[1]) * ny
        + (b.velocity[2] - a.velocity[2]) * nz;
    if rv_n > 0.0 {
        return; // already separating
    }
    let j = -(1.0 + restitution) * rv_n / (1.0 / a.mass + 1.0 / b.mass);
    let jx = j * nx;
    let jy = j * ny;
    let jz = j * nz;
    a.velocity[0] -= jx / a.mass;
    a.velocity[1] -= jy / a.mass;
    a.velocity[2] -= jz / a.mass;
    b.velocity[0] += jx / b.mass;
    b.velocity[1] += jy / b.mass;
    b.velocity[2] += jz / b.mass;

    // Simple friction along tangent.
    let tv_x = (b.velocity[0] - a.velocity[0]) - rv_n * nx;
    let tv_y = (b.velocity[1] - a.velocity[1]) - rv_n * ny;
    let tv_z = (b.velocity[2] - a.velocity[2]) - rv_n * nz;
    let tv_len = (tv_x * tv_x + tv_y * tv_y + tv_z * tv_z).sqrt();
    if tv_len > 1e-8 {
        let ft = (friction * j.abs()).min(tv_len / (1.0 / a.mass + 1.0 / b.mass));
        let tx = tv_x / tv_len;
        let ty = tv_y / tv_len;
        let tz = tv_z / tv_len;
        a.velocity[0] += ft * tx / a.mass;
        a.velocity[1] += ft * ty / a.mass;
        a.velocity[2] += ft * tz / a.mass;
        b.velocity[0] -= ft * tx / b.mass;
        b.velocity[1] -= ft * ty / b.mass;
        b.velocity[2] -= ft * tz / b.mass;
    }
}

/// True if two grains are overlapping (sphere-sphere test).
#[allow(dead_code)]
pub fn grains_overlap(a: &Grain, b: &Grain) -> bool {
    let dx = a.position[0] - b.position[0];
    let dy = a.position[1] - b.position[1];
    let dz = a.position[2] - b.position[2];
    let dist_sq = dx * dx + dy * dy + dz * dz;
    let min_dist = a.radius + b.radius;
    dist_sq < min_dist * min_dist
}

/// Total number of grains (active or not).
#[allow(dead_code)]
pub fn grain_count(world: &GranularWorld) -> usize {
    world.grains.len()
}

/// Number of active grains.
#[allow(dead_code)]
pub fn active_grain_count(world: &GranularWorld) -> usize {
    world.grains.iter().filter(|g| g.active).count()
}

/// Sum of kinetic energies: 0.5 * m * v^2.
#[allow(dead_code)]
pub fn total_granular_energy(world: &GranularWorld) -> f32 {
    world
        .grains
        .iter()
        .filter(|g| g.active)
        .map(|g| {
            let v2 = g.velocity[0] * g.velocity[0]
                + g.velocity[1] * g.velocity[1]
                + g.velocity[2] * g.velocity[2];
            0.5 * g.mass * v2
        })
        .sum()
}

/// Maximum Y position of any active grain top (pos.y + radius).
#[allow(dead_code)]
pub fn grain_pile_height(world: &GranularWorld) -> f32 {
    world
        .grains
        .iter()
        .filter(|g| g.active)
        .map(|g| g.position[1] + g.radius)
        .fold(f32::NEG_INFINITY, f32::max)
}

/// Remove a grain by ID.  Returns true if found and removed.
#[allow(dead_code)]
pub fn remove_grain(world: &mut GranularWorld, id: u32) -> bool {
    if let Some(pos) = world.grains.iter().position(|g| g.id == id) {
        world.grains.remove(pos);
        true
    } else {
        false
    }
}

/// Scatter `count` grains around `center` using a deterministic LCG seeded with `seed`.
#[allow(dead_code)]
pub fn pour_grains(
    world: &mut GranularWorld,
    center: [f32; 3],
    count: u32,
    radius: f32,
    seed: u64,
) {
    let mut state = seed ^ 0xBEEF_1234_5678_9ABC;
    for _ in 0..count {
        let rx = lcg_f32(&mut state) * 2.0 - 1.0;
        let ry = lcg_f32(&mut state) * 0.5;
        let rz = lcg_f32(&mut state) * 2.0 - 1.0;
        let pos = [
            center[0] + rx * radius * 4.0,
            center[1] + ry * radius * 4.0 + radius,
            center[2] + rz * radius * 4.0,
        ];
        add_grain(world, pos, radius, 1.0);
    }
}

fn lcg_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

fn lcg_f32(state: &mut u64) -> f32 {
    let v = lcg_next(state);
    (v >> 11) as f32 / (1u64 << 53) as f32
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_grain_count() {
        let mut world = new_granular_world(0.0);
        add_grain(&mut world, [0.0, 5.0, 0.0], 0.1, 1.0);
        add_grain(&mut world, [1.0, 5.0, 0.0], 0.1, 1.0);
        assert_eq!(grain_count(&world), 2);
    }

    #[test]
    fn test_active_grain_count() {
        let mut world = new_granular_world(0.0);
        add_grain(&mut world, [0.0, 5.0, 0.0], 0.1, 1.0);
        world.grains[0].active = false;
        add_grain(&mut world, [1.0, 5.0, 0.0], 0.1, 1.0);
        assert_eq!(active_grain_count(&world), 1);
    }

    #[test]
    fn test_grains_overlap_true() {
        let a = Grain {
            id: 0,
            position: [0.0, 0.0, 0.0],
            velocity: [0.0; 3],
            radius: 0.5,
            mass: 1.0,
            active: true,
        };
        let b = Grain {
            id: 1,
            position: [0.5, 0.0, 0.0],
            velocity: [0.0; 3],
            radius: 0.5,
            mass: 1.0,
            active: true,
        };
        assert!(grains_overlap(&a, &b));
    }

    #[test]
    fn test_grains_overlap_false() {
        let a = Grain {
            id: 0,
            position: [0.0, 0.0, 0.0],
            velocity: [0.0; 3],
            radius: 0.4,
            mass: 1.0,
            active: true,
        };
        let b = Grain {
            id: 1,
            position: [2.0, 0.0, 0.0],
            velocity: [0.0; 3],
            radius: 0.4,
            mass: 1.0,
            active: true,
        };
        assert!(!grains_overlap(&a, &b));
    }

    #[test]
    fn test_simulate_grain_falls_to_floor() {
        let mut world = new_granular_world(0.0);
        add_grain(&mut world, [0.0, 10.0, 0.0], 0.1, 1.0);
        // Simulate many steps.
        for _ in 0..200 {
            simulate_granular(&mut world, 0.05);
        }
        // Grain should be near the floor.
        let y = world.grains[0].position[1];
        assert!(y < 5.0, "grain should have fallen, y={y}");
    }

    #[test]
    fn test_resolve_grain_floor() {
        let mut g = Grain {
            id: 0,
            position: [0.0, -0.5, 0.0],
            velocity: [0.0, -2.0, 0.0],
            radius: 0.1,
            mass: 1.0,
            active: true,
        };
        resolve_grain_floor(&mut g, 0.0, 0.5);
        assert!(g.position[1] >= 0.1 - 1e-5);
        assert!(g.velocity[1] >= 0.0);
    }

    #[test]
    fn test_grain_pile_height() {
        let mut world = new_granular_world(0.0);
        add_grain(&mut world, [0.0, 1.0, 0.0], 0.2, 1.0);
        add_grain(&mut world, [0.0, 3.0, 0.0], 0.5, 1.0);
        let h = grain_pile_height(&world);
        assert!((h - 3.5).abs() < 1e-5, "expected 3.5 got {h}");
    }

    #[test]
    fn test_remove_grain_found() {
        let mut world = new_granular_world(0.0);
        let id = add_grain(&mut world, [0.0, 5.0, 0.0], 0.1, 1.0);
        assert!(remove_grain(&mut world, id));
        assert_eq!(grain_count(&world), 0);
    }

    #[test]
    fn test_remove_grain_not_found() {
        let mut world = new_granular_world(0.0);
        assert!(!remove_grain(&mut world, 999));
    }

    #[test]
    fn test_pour_grains_count() {
        let mut world = new_granular_world(0.0);
        pour_grains(&mut world, [0.0, 5.0, 0.0], 10, 0.1, 42);
        assert_eq!(grain_count(&world), 10);
    }

    #[test]
    fn test_pour_grains_around_center() {
        let mut world = new_granular_world(0.0);
        pour_grains(&mut world, [5.0, 5.0, 5.0], 20, 0.1, 7);
        for g in &world.grains {
            // Grains should be within a reasonable distance of center.
            let dx = g.position[0] - 5.0;
            let dz = g.position[2] - 5.0;
            assert!(dx.hypot(dz) < 10.0, "grain too far from center");
        }
    }

    #[test]
    fn test_total_granular_energy_at_rest() {
        let mut world = new_granular_world(0.0);
        add_grain(&mut world, [0.0, 0.0, 0.0], 0.1, 1.0);
        assert!((total_granular_energy(&world)).abs() < 1e-9);
    }

    #[test]
    fn test_resolve_grain_grain_separates() {
        let mut a = Grain {
            id: 0,
            position: [0.0, 0.0, 0.0],
            velocity: [1.0, 0.0, 0.0],
            radius: 0.5,
            mass: 1.0,
            active: true,
        };
        let mut b = Grain {
            id: 1,
            position: [0.6, 0.0, 0.0],
            velocity: [-1.0, 0.0, 0.0],
            radius: 0.5,
            mass: 1.0,
            active: true,
        };
        resolve_grain_grain(&mut a, &mut b, 0.5, 0.3);
        // After resolution grains should not overlap (or less).
        let dx = a.position[0] - b.position[0];
        let dist = dx.abs();
        assert!(dist >= 0.9, "grains should be separated, dist={dist}");
    }

    #[test]
    fn test_default_config_fields() {
        let cfg = default_granular_config();
        assert!(cfg.restitution >= 0.0 && cfg.restitution <= 1.0);
        assert!(cfg.gravity[1] < 0.0);
        assert!(cfg.substeps > 0);
    }
}
