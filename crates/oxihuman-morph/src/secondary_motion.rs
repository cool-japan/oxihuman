//! Secondary motion — lag/follow-through for hair, clothing, and soft attachments.

#[allow(dead_code)]
pub struct SecondaryBone {
    pub id: u32,
    pub name: String,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub target_position: [f32; 3],
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

#[allow(dead_code)]
pub struct SecondaryChain {
    pub name: String,
    pub bones: Vec<SecondaryBone>,
    pub gravity: [f32; 3],
    pub wind: [f32; 3],
}

#[allow(dead_code)]
pub struct SecondaryMotionConfig {
    pub gravity: [f32; 3],
    pub default_stiffness: f32,
    pub default_damping: f32,
    pub default_mass: f32,
}

#[allow(dead_code)]
pub fn default_secondary_config() -> SecondaryMotionConfig {
    SecondaryMotionConfig {
        gravity: [0.0, -9.81, 0.0],
        default_stiffness: 50.0,
        default_damping: 5.0,
        default_mass: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_secondary_bone(
    id: u32,
    name: &str,
    pos: [f32; 3],
    stiffness: f32,
    damping: f32,
) -> SecondaryBone {
    SecondaryBone {
        id,
        name: name.to_string(),
        position: pos,
        velocity: [0.0, 0.0, 0.0],
        target_position: pos,
        stiffness,
        damping,
        mass: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_secondary_chain(name: &str, gravity: [f32; 3]) -> SecondaryChain {
    SecondaryChain {
        name: name.to_string(),
        bones: Vec::new(),
        gravity,
        wind: [0.0, 0.0, 0.0],
    }
}

#[allow(dead_code)]
pub fn add_secondary_bone(chain: &mut SecondaryChain, bone: SecondaryBone) {
    chain.bones.push(bone);
}

#[allow(dead_code)]
pub fn update_secondary_bone(bone: &mut SecondaryBone, dt: f32, external_force: [f32; 3]) {
    // Spring-damper: F = k*(target-pos) - d*vel + ext
    let spring = [
        bone.stiffness * (bone.target_position[0] - bone.position[0]),
        bone.stiffness * (bone.target_position[1] - bone.position[1]),
        bone.stiffness * (bone.target_position[2] - bone.position[2]),
    ];
    let damp = [
        bone.damping * bone.velocity[0],
        bone.damping * bone.velocity[1],
        bone.damping * bone.velocity[2],
    ];
    let force = [
        spring[0] - damp[0] + external_force[0],
        spring[1] - damp[1] + external_force[1],
        spring[2] - damp[2] + external_force[2],
    ];
    let inv_mass = if bone.mass > 0.0 {
        1.0 / bone.mass
    } else {
        1.0
    };
    bone.velocity[0] += force[0] * inv_mass * dt;
    bone.velocity[1] += force[1] * inv_mass * dt;
    bone.velocity[2] += force[2] * inv_mass * dt;
    bone.position[0] += bone.velocity[0] * dt;
    bone.position[1] += bone.velocity[1] * dt;
    bone.position[2] += bone.velocity[2] * dt;
}

#[allow(dead_code)]
pub fn update_secondary_chain(chain: &mut SecondaryChain, dt: f32, target_positions: &[[f32; 3]]) {
    let gravity = chain.gravity;
    let wind = chain.wind;
    let external = [
        gravity[0] + wind[0],
        gravity[1] + wind[1],
        gravity[2] + wind[2],
    ];
    for (i, bone) in chain.bones.iter_mut().enumerate() {
        if i < target_positions.len() {
            bone.target_position = target_positions[i];
        }
        update_secondary_bone(bone, dt, external);
    }
}

#[allow(dead_code)]
pub fn set_chain_wind(chain: &mut SecondaryChain, wind: [f32; 3]) {
    chain.wind = wind;
}

#[allow(dead_code)]
pub fn secondary_bone_count(chain: &SecondaryChain) -> usize {
    chain.bones.len()
}

#[allow(dead_code)]
pub fn chain_kinetic_energy(chain: &SecondaryChain) -> f32 {
    chain.bones.iter().fold(0.0_f32, |acc, bone| {
        let v2 = bone.velocity[0] * bone.velocity[0]
            + bone.velocity[1] * bone.velocity[1]
            + bone.velocity[2] * bone.velocity[2];
        acc + 0.5 * bone.mass * v2
    })
}

#[allow(dead_code)]
pub fn reset_secondary_chain(chain: &mut SecondaryChain) {
    for bone in chain.bones.iter_mut() {
        bone.velocity = [0.0, 0.0, 0.0];
    }
}

#[allow(dead_code)]
pub fn secondary_chain_positions(chain: &SecondaryChain) -> Vec<[f32; 3]> {
    chain.bones.iter().map(|b| b.position).collect()
}

#[allow(dead_code)]
pub fn secondary_bone_lag(bone: &SecondaryBone) -> f32 {
    let dx = bone.position[0] - bone.target_position[0];
    let dy = bone.position[1] - bone.target_position[1];
    let dz = bone.position[2] - bone.target_position[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn blend_secondary_to_target(chain: &mut SecondaryChain, alpha: f32) {
    let a = alpha.clamp(0.0, 1.0);
    for bone in chain.bones.iter_mut() {
        bone.position[0] = bone.position[0] + a * (bone.target_position[0] - bone.position[0]);
        bone.position[1] = bone.position[1] + a * (bone.target_position[1] - bone.position[1]);
        bone.position[2] = bone.position[2] + a * (bone.target_position[2] - bone.position[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_secondary_bone_fields() {
        let bone = new_secondary_bone(1, "hair_tip", [1.0, 2.0, 3.0], 40.0, 4.0);
        assert_eq!(bone.id, 1);
        assert_eq!(bone.name, "hair_tip");
        assert_eq!(bone.position, [1.0, 2.0, 3.0]);
        assert_eq!(bone.velocity, [0.0, 0.0, 0.0]);
        assert_eq!(bone.stiffness, 40.0);
        assert_eq!(bone.damping, 4.0);
    }

    #[test]
    fn test_new_secondary_chain_empty() {
        let chain = new_secondary_chain("hair_chain", [0.0, -9.81, 0.0]);
        assert_eq!(chain.name, "hair_chain");
        assert!(chain.bones.is_empty());
    }

    #[test]
    fn test_add_secondary_bone() {
        let mut chain = new_secondary_chain("chain", [0.0, -9.81, 0.0]);
        let bone = new_secondary_bone(0, "b0", [0.0, 0.0, 0.0], 50.0, 5.0);
        add_secondary_bone(&mut chain, bone);
        assert_eq!(chain.bones.len(), 1);
    }

    #[test]
    fn test_secondary_bone_count() {
        let mut chain = new_secondary_chain("c", [0.0, 0.0, 0.0]);
        add_secondary_bone(&mut chain, new_secondary_bone(0, "b0", [0.0; 3], 1.0, 1.0));
        add_secondary_bone(&mut chain, new_secondary_bone(1, "b1", [1.0; 3], 1.0, 1.0));
        assert_eq!(secondary_bone_count(&chain), 2);
    }

    #[test]
    fn test_update_secondary_bone_spring_force() {
        let mut bone = new_secondary_bone(0, "b", [0.0, 0.0, 0.0], 100.0, 0.0);
        bone.target_position = [1.0, 0.0, 0.0];
        // F = k*(target-pos) - d*vel + ext = 100*(1-0) - 0 + 0 = 100
        // a = F/m = 100
        // vel += a*dt => vel = 100 * 0.01 = 1.0
        // pos += vel*dt => pos = 1.0 * 0.01 = 0.01
        update_secondary_bone(&mut bone, 0.01, [0.0, 0.0, 0.0]);
        assert!((bone.velocity[0] - 1.0).abs() < 1e-4);
        assert!((bone.position[0] - 0.01).abs() < 1e-4);
    }

    #[test]
    fn test_update_secondary_bone_external_force() {
        let mut bone = new_secondary_bone(0, "b", [0.0; 3], 0.0, 0.0);
        // external force of 10 in Y, no spring or damping
        update_secondary_bone(&mut bone, 0.1, [0.0, 10.0, 0.0]);
        assert!((bone.velocity[1] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_chain_kinetic_energy_zero_velocity() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        add_secondary_bone(&mut chain, new_secondary_bone(0, "b0", [0.0; 3], 1.0, 1.0));
        assert_eq!(chain_kinetic_energy(&chain), 0.0);
    }

    #[test]
    fn test_chain_kinetic_energy_nonzero() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        let mut bone = new_secondary_bone(0, "b0", [0.0; 3], 1.0, 1.0);
        bone.velocity = [2.0, 0.0, 0.0];
        bone.mass = 1.0;
        add_secondary_bone(&mut chain, bone);
        // KE = 0.5 * 1.0 * 4.0 = 2.0
        assert!((chain_kinetic_energy(&chain) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_reset_secondary_chain_zeros_velocity() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        let mut bone = new_secondary_bone(0, "b0", [0.0; 3], 1.0, 1.0);
        bone.velocity = [5.0, 3.0, 1.0];
        add_secondary_bone(&mut chain, bone);
        reset_secondary_chain(&mut chain);
        assert_eq!(chain.bones[0].velocity, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_secondary_bone_lag() {
        let mut bone = new_secondary_bone(0, "b", [0.0; 3], 1.0, 1.0);
        bone.target_position = [3.0, 4.0, 0.0];
        let lag = secondary_bone_lag(&bone);
        assert!((lag - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_secondary_bone_lag_zero() {
        let bone = new_secondary_bone(0, "b", [1.0, 2.0, 3.0], 1.0, 1.0);
        assert_eq!(secondary_bone_lag(&bone), 0.0);
    }

    #[test]
    fn test_blend_secondary_to_target_full() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        let mut bone = new_secondary_bone(0, "b0", [0.0; 3], 1.0, 1.0);
        bone.target_position = [10.0, 0.0, 0.0];
        add_secondary_bone(&mut chain, bone);
        blend_secondary_to_target(&mut chain, 1.0);
        assert!((chain.bones[0].position[0] - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_secondary_to_target_half() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        let mut bone = new_secondary_bone(0, "b0", [0.0; 3], 1.0, 1.0);
        bone.target_position = [10.0, 0.0, 0.0];
        add_secondary_bone(&mut chain, bone);
        blend_secondary_to_target(&mut chain, 0.5);
        assert!((chain.bones[0].position[0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_secondary_chain_positions() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        add_secondary_bone(
            &mut chain,
            new_secondary_bone(0, "b0", [1.0, 2.0, 3.0], 1.0, 1.0),
        );
        add_secondary_bone(
            &mut chain,
            new_secondary_bone(1, "b1", [4.0, 5.0, 6.0], 1.0, 1.0),
        );
        let positions = secondary_chain_positions(&chain);
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0], [1.0, 2.0, 3.0]);
        assert_eq!(positions[1], [4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_set_chain_wind() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        set_chain_wind(&mut chain, [1.0, 0.0, 2.0]);
        assert_eq!(chain.wind, [1.0, 0.0, 2.0]);
    }

    #[test]
    fn test_update_secondary_chain_targets() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        add_secondary_bone(&mut chain, new_secondary_bone(0, "b0", [0.0; 3], 10.0, 0.0));
        let targets = [[1.0, 0.0, 0.0]];
        update_secondary_chain(&mut chain, 0.016, &targets);
        assert_eq!(chain.bones[0].target_position, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_default_secondary_config() {
        let cfg = default_secondary_config();
        assert!(cfg.default_stiffness > 0.0);
        assert!(cfg.default_damping > 0.0);
        assert!(cfg.default_mass > 0.0);
        assert!(cfg.gravity[1] < 0.0);
    }
}

// ---------------------------------------------------------------------------
// XPBD Secondary-Motion Constraints
// ---------------------------------------------------------------------------

/// A constraint applied during the XPBD projection pass.
pub enum SecondaryConstraint {
    /// Fix a single particle to a world-space target position.
    Pin { vertex_idx: usize, target: [f32; 3] },
    /// Maintain a rest distance between two particles.
    Length { a: usize, b: usize, rest_len: f32 },
    /// Maintain an approximate volume for a group of particles via radial scaling.
    Volume { vertices: Vec<usize>, rest_vol: f32 },
}

/// A single simulated point mass in the XPBD system.
pub struct XpbdParticle {
    pub pos: [f32; 3],
    pub prev_pos: [f32; 3],
    /// Inverse mass.  `0.0` means the particle is static / pinned by mass.
    pub inv_mass: f32,
}

/// Full XPBD secondary-motion system containing particles and constraints.
pub struct SecondaryMotionSystem {
    pub particles: Vec<XpbdParticle>,
    pub constraints: Vec<SecondaryConstraint>,
    pub gravity: [f32; 3],
    pub xpbd_iterations: u32,
}

impl SecondaryMotionSystem {
    /// Create a new empty system with the given gravity vector.
    pub fn new(gravity: [f32; 3]) -> Self {
        SecondaryMotionSystem {
            particles: Vec::new(),
            constraints: Vec::new(),
            gravity,
            xpbd_iterations: 4,
        }
    }

    /// Add a particle at `pos` with `inv_mass` (use `0.0` for a static particle).
    pub fn add_particle(&mut self, pos: [f32; 3], inv_mass: f32) {
        self.particles.push(XpbdParticle {
            pos,
            prev_pos: pos,
            inv_mass,
        });
    }

    /// Append a constraint to the system.
    pub fn add_constraint(&mut self, c: SecondaryConstraint) {
        self.constraints.push(c);
    }

    /// Advance the simulation by one time step `dt`.
    ///
    /// 1. Semi-implicit Euler (gravity + Verlet velocity integration).
    /// 2. XPBD projection (`xpbd_iterations` passes): Pin, Length, Volume.
    /// 3. `prev_pos` is already updated in step 1.
    pub fn update(&mut self, dt: f32) {
        // --- Step 1: Semi-implicit Euler ---------------------------------
        for p in self.particles.iter_mut() {
            if p.inv_mass <= 0.0 {
                continue;
            }
            let velocity = vec3_sub(p.pos, p.prev_pos);
            p.prev_pos = p.pos;
            let grav_dt2 = [
                self.gravity[0] * dt * dt,
                self.gravity[1] * dt * dt,
                self.gravity[2] * dt * dt,
            ];
            p.pos = vec3_add(vec3_add(p.pos, velocity), grav_dt2);
        }

        // --- Step 2: XPBD projection -------------------------------------
        for _ in 0..self.xpbd_iterations {
            // We need index-based access; collect indices to avoid borrow issues
            let n_constraints = self.constraints.len();
            for ci in 0..n_constraints {
                // Safety: we use raw pointer arithmetic only inside the unsafe
                // block to satisfy the borrow checker.  The indices are
                // validated before use.
                match &self.constraints[ci] {
                    SecondaryConstraint::Pin { vertex_idx, target } => {
                        let idx = *vertex_idx;
                        let tgt = *target;
                        if idx < self.particles.len() {
                            let p = &mut self.particles[idx];
                            if p.inv_mass > 0.0 {
                                p.pos = tgt;
                            }
                        }
                    }
                    SecondaryConstraint::Length { a, b, rest_len } => {
                        let (ia, ib, rl) = (*a, *b, *rest_len);
                        if ia >= self.particles.len() || ib >= self.particles.len() {
                            continue;
                        }
                        // Read current positions
                        let pos_a = self.particles[ia].pos;
                        let pos_b = self.particles[ib].pos;
                        let inv_a = self.particles[ia].inv_mass;
                        let inv_b = self.particles[ib].inv_mass;

                        let delta = vec3_sub(pos_b, pos_a);
                        let dist = vec3_len(delta);
                        if dist < 1e-10 {
                            continue;
                        }
                        let w_sum = inv_a + inv_b;
                        if w_sum == 0.0 {
                            continue;
                        }
                        let scale = (dist - rl) / dist;
                        let correction = [delta[0] * scale, delta[1] * scale, delta[2] * scale];
                        self.particles[ia].pos = vec3_add(
                            self.particles[ia].pos,
                            vec3_scale(correction, inv_a / w_sum),
                        );
                        self.particles[ib].pos = vec3_sub(
                            self.particles[ib].pos,
                            vec3_scale(correction, inv_b / w_sum),
                        );
                    }
                    SecondaryConstraint::Volume { vertices, rest_vol } => {
                        let indices: Vec<usize> = vertices.clone();
                        let rv = *rest_vol;
                        let n = indices.len();
                        if n == 0 {
                            continue;
                        }

                        // Compute centroid
                        let mut centroid = [0.0_f32; 3];
                        let mut valid_count = 0usize;
                        for &vi in &indices {
                            if vi < self.particles.len() {
                                centroid = vec3_add(centroid, self.particles[vi].pos);
                                valid_count += 1;
                            }
                        }
                        if valid_count == 0 {
                            continue;
                        }
                        let inv_n = 1.0 / valid_count as f32;
                        centroid = vec3_scale(centroid, inv_n);

                        // Approximate current "volume" as mean squared distance from centroid
                        let mut mean_r2 = 0.0_f32;
                        for &vi in &indices {
                            if vi >= self.particles.len() {
                                continue;
                            }
                            let d = vec3_sub(self.particles[vi].pos, centroid);
                            mean_r2 += vec3_dot(d, d);
                        }
                        mean_r2 *= inv_n;

                        if mean_r2 < 1e-12 {
                            continue;
                        }

                        // rest_vol is treated as the rest mean-squared radius
                        let ratio = (rv / mean_r2).sqrt();

                        for &vi in &indices {
                            if vi >= self.particles.len() {
                                continue;
                            }
                            if self.particles[vi].inv_mass <= 0.0 {
                                continue;
                            }
                            let offset = vec3_sub(self.particles[vi].pos, centroid);
                            let new_offset = vec3_scale(offset, ratio);
                            self.particles[vi].pos = vec3_add(centroid, new_offset);
                        }
                    }
                }
            }
        }
    }

    /// Return a snapshot of all current particle positions.
    pub fn particle_positions(&self) -> Vec<[f32; 3]> {
        self.particles.iter().map(|p| p.pos).collect()
    }

    /// Detect particle pairs that are closer than `2 * collision_radius`.
    ///
    /// Uses O(n²) brute-force for n < 32, and a spatial hash grid for n ≥ 32.
    pub fn detect_self_collisions(&self, collision_radius: f32) -> Vec<(usize, usize)> {
        let n = self.particles.len();
        let threshold = 2.0 * collision_radius;

        if n < 32 {
            // O(n²) brute-force
            let mut pairs = Vec::new();
            for i in 0..n {
                for j in (i + 1)..n {
                    let d = vec3_sub(self.particles[i].pos, self.particles[j].pos);
                    if vec3_len(d) < threshold {
                        pairs.push((i, j));
                    }
                }
            }
            pairs
        } else {
            // Spatial hash grid
            spatial_hash_collisions(&self.particles, threshold)
        }
    }
}

// ---------------------------------------------------------------------------
// Private spatial hash collision detection (used when n >= 32)
// ---------------------------------------------------------------------------

fn spatial_hash_collisions(particles: &[XpbdParticle], threshold: f32) -> Vec<(usize, usize)> {
    use std::collections::HashMap;

    let cell_size = threshold.max(1e-6);
    let inv_cell = 1.0 / cell_size;

    let cell_key = |pos: [f32; 3]| -> (i64, i64, i64) {
        (
            (pos[0] * inv_cell).floor() as i64,
            (pos[1] * inv_cell).floor() as i64,
            (pos[2] * inv_cell).floor() as i64,
        )
    };

    // Build grid: cell -> list of particle indices
    let mut grid: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();
    for (i, p) in particles.iter().enumerate() {
        let key = cell_key(p.pos);
        grid.entry(key).or_default().push(i);
    }

    let mut pairs: Vec<(usize, usize)> = Vec::new();
    // For each particle, check its own cell and the 26 neighbouring cells
    let offsets: &[(i64, i64, i64)] = &[
        (-1, -1, -1),
        (-1, -1, 0),
        (-1, -1, 1),
        (-1, 0, -1),
        (-1, 0, 0),
        (-1, 0, 1),
        (-1, 1, -1),
        (-1, 1, 0),
        (-1, 1, 1),
        (0, -1, -1),
        (0, -1, 0),
        (0, -1, 1),
        (0, 0, -1),
        (0, 0, 0),
        (0, 0, 1),
        (0, 1, -1),
        (0, 1, 0),
        (0, 1, 1),
        (1, -1, -1),
        (1, -1, 0),
        (1, -1, 1),
        (1, 0, -1),
        (1, 0, 0),
        (1, 0, 1),
        (1, 1, -1),
        (1, 1, 0),
        (1, 1, 1),
    ];

    // Use a HashSet to deduplicate pairs
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();

    for (i, p) in particles.iter().enumerate() {
        let (cx, cy, cz) = cell_key(p.pos);
        for &(ox, oy, oz) in offsets {
            let neighbour_key = (cx + ox, cy + oy, cz + oz);
            if let Some(bucket) = grid.get(&neighbour_key) {
                for &j in bucket {
                    if i == j {
                        continue;
                    }
                    let (lo, hi) = if i < j { (i, j) } else { (j, i) };
                    if seen.contains(&(lo, hi)) {
                        continue;
                    }
                    let d = vec3_sub(particles[i].pos, particles[j].pos);
                    if vec3_len(d) < threshold {
                        seen.insert((lo, hi));
                        pairs.push((lo, hi));
                    }
                }
            }
        }
    }

    pairs.sort_unstable();
    pairs
}

// ---------------------------------------------------------------------------
// Inline vec3 helpers (private)
// ---------------------------------------------------------------------------

#[inline]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn vec3_len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

#[inline]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

// ---------------------------------------------------------------------------
// Tests for XPBD system
// ---------------------------------------------------------------------------

#[cfg(test)]
mod xpbd_tests {
    use super::*;

    /// Pin constraint holds the target position after many steps.
    #[test]
    fn test_pin_holds_fixed() {
        let mut sys = SecondaryMotionSystem::new([0.0, -9.8, 0.0]);
        sys.add_particle([1.0, 2.0, 3.0], 1.0); // particle 0 — will be pinned
        sys.add_particle([0.0, 0.0, 0.0], 0.0); // particle 1 — static mass
        sys.add_constraint(SecondaryConstraint::Pin {
            vertex_idx: 0,
            target: [1.0, 2.0, 3.0],
        });

        for _ in 0..10 {
            sys.update(0.016);
        }

        let pos = sys.particles[0].pos;
        assert!(
            (pos[0] - 1.0).abs() < 1e-5
                && (pos[1] - 2.0).abs() < 1e-5
                && (pos[2] - 3.0).abs() < 1e-5,
            "Pinned particle drifted: {pos:?}"
        );
    }

    /// Length constraint keeps rest distance within 5% after 30 steps.
    #[test]
    fn test_length_constraint_preserves_distance() {
        let mut sys = SecondaryMotionSystem::new([0.0, 0.0, 0.0]);
        sys.add_particle([0.0, 0.0, 0.0], 1.0);
        sys.add_particle([2.0, 0.0, 0.0], 1.0); // initially 2.0 apart; rest is 1.0
        sys.add_constraint(SecondaryConstraint::Length {
            a: 0,
            b: 1,
            rest_len: 1.0,
        });

        for _ in 0..30 {
            sys.update(0.016);
        }

        let p0 = sys.particles[0].pos;
        let p1 = sys.particles[1].pos;
        let d = vec3_len(vec3_sub(p1, p0));
        assert!(
            (d - 1.0).abs() < 0.05,
            "Length {d} deviates more than 5% from rest_len=1.0"
        );
    }

    /// Under gravity both particles drift down, but their mutual distance stays ≈1.0.
    #[test]
    fn test_no_stretch_under_gravity() {
        let mut sys = SecondaryMotionSystem::new([0.0, -9.8, 0.0]);
        sys.add_particle([0.0, 0.0, 0.0], 1.0);
        sys.add_particle([0.0, 1.0, 0.0], 1.0);
        sys.add_constraint(SecondaryConstraint::Length {
            a: 0,
            b: 1,
            rest_len: 1.0,
        });
        // Raise iteration count so the constraint is well-satisfied under gravity
        sys.xpbd_iterations = 20;

        for _ in 0..30 {
            sys.update(0.016);
        }

        let p0 = sys.particles[0].pos;
        let p1 = sys.particles[1].pos;
        let d = vec3_len(vec3_sub(p1, p0));
        assert!(
            (d - 1.0).abs() < 0.05,
            "Distance under gravity {d} deviates more than 5% from 1.0"
        );
    }

    /// Two nearby particles are detected as a collision pair.
    #[test]
    fn test_self_collision_detection() {
        let mut sys = SecondaryMotionSystem::new([0.0, 0.0, 0.0]);
        sys.add_particle([0.0, 0.0, 0.0], 1.0);
        sys.add_particle([0.1, 0.0, 0.0], 1.0);

        let pairs = sys.detect_self_collisions(0.15);
        assert_eq!(pairs.len(), 1, "Expected 1 collision pair, got {:?}", pairs);
        assert_eq!(pairs[0], (0, 1));
    }
}
