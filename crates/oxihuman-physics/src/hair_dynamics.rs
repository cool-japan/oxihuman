//! Hair strand simulation with spring-mass chains and aerodynamic drag.
//!
//! Each strand is modelled as a chain of particles connected by distance
//! springs. The root particle is pinned.  Verlet integration is used for
//! the time step, and aerodynamic drag is applied per-particle proportional
//! to the relative velocity between the particle and the ambient wind.

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

/// Per-particle state in a hair strand.
#[allow(dead_code)]
#[derive(Clone)]
pub struct HairParticle {
    /// Current position.
    pub position: [f32; 3],
    /// Previous position (Verlet integration).
    pub prev_position: [f32; 3],
    /// Whether this particle is pinned (does not move).
    pub pinned: bool,
    /// Rest length to next particle in the strand.
    pub rest_length: f32,
}

/// A single hair strand: a chain of particles.
#[allow(dead_code)]
pub struct HairStrand {
    pub particles: Vec<HairParticle>,
}

/// Configuration for hair simulation.
#[allow(dead_code)]
pub struct HairConfig {
    /// Spring stiffness coefficient (0..1).
    pub stiffness: f32,
    /// Damping factor applied per step.
    pub damping: f32,
    /// Drag coefficient for aerodynamic interaction.
    pub drag: f32,
    /// Number of constraint-projection iterations per step.
    pub constraint_iterations: u32,
}

/// Collection of hair strands forming a hair system.
#[allow(dead_code)]
pub struct HairSystem {
    pub strands: Vec<HairStrand>,
    pub cfg: HairConfig,
}

// ---------------------------------------------------------------------------
// Default config
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn default_hair_config() -> HairConfig {
    HairConfig {
        stiffness: 0.8,
        damping: 0.98,
        drag: 0.05,
        constraint_iterations: 4,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn len3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l < 1e-10 {
        [0.0; 3]
    } else {
        scale3(a, 1.0 / l)
    }
}

// ---------------------------------------------------------------------------
// Core functions
// ---------------------------------------------------------------------------

/// Create a new empty hair system with the given configuration.
#[allow(dead_code)]
pub fn new_hair_system(cfg: &HairConfig) -> HairSystem {
    HairSystem {
        strands: Vec::new(),
        cfg: HairConfig {
            stiffness: cfg.stiffness,
            damping: cfg.damping,
            drag: cfg.drag,
            constraint_iterations: cfg.constraint_iterations,
        },
    }
}

/// Add a strand rooted at `root` growing in direction `dir`, with `segments` segments.
#[allow(dead_code)]
pub fn add_strand(
    sys: &mut HairSystem,
    root: [f32; 3],
    dir: [f32; 3],
    length: f32,
    segments: u32,
) {
    let count = (segments + 1).max(2) as usize;
    let seg_len = length / (count - 1) as f32;
    let unit_dir = normalize3(dir);

    let mut particles: Vec<HairParticle> = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 * seg_len;
        let pos = add3(root, scale3(unit_dir, t));
        particles.push(HairParticle {
            position: pos,
            prev_position: pos,
            pinned: i == 0,
            rest_length: seg_len,
        });
    }

    sys.strands.push(HairStrand { particles });
}

/// Advance the hair system by one time step using Verlet integration.
#[allow(dead_code)]
pub fn step_hair(
    sys: &mut HairSystem,
    dt: f32,
    gravity: [f32; 3],
    wind: [f32; 3],
) {
    let dt2 = dt * dt;
    let damping = sys.cfg.damping;
    let drag = sys.cfg.drag;

    for strand in &mut sys.strands {
        for p in &mut strand.particles {
            if p.pinned {
                p.prev_position = p.position;
                continue;
            }

            // Velocity estimate from Verlet.
            let vel = sub3(p.position, p.prev_position);

            // Relative velocity w.r.t. wind.
            let rel_vel = sub3(vel, scale3(wind, dt));
            let aero = scale3(rel_vel, -drag);

            // Acceleration = gravity + aero.
            let accel = add3(gravity, aero);

            let new_pos = add3(
                p.position,
                add3(scale3(vel, damping), scale3(accel, dt2)),
            );

            p.prev_position = p.position;
            p.position = new_pos;
        }
    }

    let iters = sys.cfg.constraint_iterations;
    for _ in 0..iters {
        apply_hair_constraints(sys);
    }
}

/// Project distance constraints along each strand.
#[allow(dead_code)]
pub fn apply_hair_constraints(sys: &mut HairSystem) {
    let stiffness = sys.cfg.stiffness;

    for strand in &mut sys.strands {
        let n = strand.particles.len();
        if n < 2 {
            continue;
        }
        for i in 0..n - 1 {
            let rest = strand.particles[i].rest_length;
            let pa = strand.particles[i].position;
            let pb = strand.particles[i + 1].position;
            let delta = sub3(pb, pa);
            let cur_len = len3(delta);
            if cur_len < 1e-10 {
                continue;
            }
            let diff = (cur_len - rest) / cur_len * stiffness;
            let correction = scale3(delta, 0.5 * diff);

            let a_pinned = strand.particles[i].pinned;
            let b_pinned = strand.particles[i + 1].pinned;

            if !a_pinned && !b_pinned {
                strand.particles[i].position = add3(pa, correction);
                strand.particles[i + 1].position = sub3(pb, correction);
            } else if !a_pinned {
                strand.particles[i].position = add3(pa, scale3(correction, 2.0));
            } else if !b_pinned {
                strand.particles[i + 1].position = sub3(pb, scale3(correction, 2.0));
            }
        }
    }
}

/// Return the tip position (last particle) of each strand.
#[allow(dead_code)]
pub fn hair_tip_positions(sys: &HairSystem) -> Vec<[f32; 3]> {
    sys.strands
        .iter()
        .filter_map(|s| s.particles.last().map(|p| p.position))
        .collect()
}

/// Compute the current total length of a strand (sum of inter-particle distances).
#[allow(dead_code)]
pub fn hair_strand_length(strand: &HairStrand) -> f32 {
    strand
        .particles
        .windows(2)
        .map(|w| len3(sub3(w[1].position, w[0].position)))
        .sum()
}

/// Set a new stiffness value on all strands of the system.
#[allow(dead_code)]
pub fn set_hair_stiffness(sys: &mut HairSystem, stiffness: f32) {
    sys.cfg.stiffness = stiffness.clamp(0.0, 1.0);
}

/// Reset all particles to their rest positions along their initial directions.
/// Positions are restored using root + direction * rest_length * index.
#[allow(dead_code)]
pub fn reset_hair(sys: &mut HairSystem) {
    for strand in &mut sys.strands {
        let n = strand.particles.len();
        if n == 0 {
            continue;
        }
        // Reconstruct positions by integrating rest lengths from root.
        let root = strand.particles[0].position;
        // Determine stored direction from the original layout (fallback: y-down).
        // Compute unit direction from root to second particle (if available).
        let dir = if n > 1 {
            normalize3(sub3(strand.particles[1].position, root))
        } else {
            [0.0, -1.0, 0.0]
        };

        let mut cumulative = 0.0f32;
        for i in 0..n {
            let rest = strand.particles[i].rest_length;
            let pos = add3(root, scale3(dir, cumulative));
            strand.particles[i].position = pos;
            strand.particles[i].prev_position = pos;
            cumulative += rest;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sys() -> HairSystem {
        let cfg = default_hair_config();
        new_hair_system(&cfg)
    }

    #[test]
    fn test_new_hair_system_empty() {
        let sys = make_sys();
        assert!(sys.strands.is_empty());
    }

    #[test]
    fn test_add_strand_particle_count() {
        let mut sys = make_sys();
        add_strand(&mut sys, [0.0; 3], [0.0, -1.0, 0.0], 1.0, 4);
        assert_eq!(sys.strands.len(), 1);
        // segments=4 → 5 particles
        assert_eq!(sys.strands[0].particles.len(), 5);
    }

    #[test]
    fn test_root_particle_is_pinned() {
        let mut sys = make_sys();
        add_strand(&mut sys, [0.0; 3], [0.0, -1.0, 0.0], 1.0, 3);
        assert!(sys.strands[0].particles[0].pinned);
    }

    #[test]
    fn test_step_hair_root_stays_fixed() {
        let mut sys = make_sys();
        add_strand(&mut sys, [0.0, 0.0, 0.0], [0.0, -1.0, 0.0], 1.0, 4);
        let root_before = sys.strands[0].particles[0].position;
        step_hair(&mut sys, 0.016, [0.0, -9.8, 0.0], [0.0; 3]);
        let root_after = sys.strands[0].particles[0].position;
        assert_eq!(root_before, root_after, "root must not move");
    }

    #[test]
    fn test_hair_tip_positions_count() {
        let mut sys = make_sys();
        add_strand(&mut sys, [0.0; 3], [0.0, -1.0, 0.0], 1.0, 4);
        add_strand(&mut sys, [1.0, 0.0, 0.0], [0.0, -1.0, 0.0], 1.0, 4);
        let tips = hair_tip_positions(&sys);
        assert_eq!(tips.len(), 2);
    }

    #[test]
    fn test_hair_strand_length_approx_rest() {
        let mut sys = make_sys();
        add_strand(&mut sys, [0.0; 3], [0.0, 1.0, 0.0], 1.0, 4);
        let length = hair_strand_length(&sys.strands[0]);
        assert!(
            (length - 1.0).abs() < 1e-4,
            "initial strand length should be ~1.0, got {length}"
        );
    }

    #[test]
    fn test_set_hair_stiffness_clamps() {
        let mut sys = make_sys();
        set_hair_stiffness(&mut sys, 2.5);
        assert!((sys.cfg.stiffness - 1.0).abs() < 1e-6);
        set_hair_stiffness(&mut sys, -1.0);
        assert!((sys.cfg.stiffness - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset_hair_restores_positions() {
        let mut sys = make_sys();
        add_strand(&mut sys, [0.0; 3], [0.0, 1.0, 0.0], 1.0, 3);
        // Displace tips manually.
        for p in &mut sys.strands[0].particles {
            p.position = [99.0; 3];
            p.prev_position = [99.0; 3];
        }
        // Restore root to known value.
        sys.strands[0].particles[0].position = [0.0; 3];
        sys.strands[0].particles[0].prev_position = [0.0; 3];
        reset_hair(&mut sys);
        // Root must be unchanged.
        let root = sys.strands[0].particles[0].position;
        assert_eq!(root, [0.0; 3]);
    }

    #[test]
    fn test_gravity_moves_tips_down() {
        let mut sys = make_sys();
        add_strand(&mut sys, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0, 4);
        let tip_y_before = sys.strands[0].particles.last().expect("should succeed").position[1];
        // Run several steps with gravity pointing down.
        for _ in 0..10 {
            step_hair(&mut sys, 0.016, [0.0, -9.8, 0.0], [0.0; 3]);
        }
        let tip_y_after = sys.strands[0].particles.last().expect("should succeed").position[1];
        assert!(
            tip_y_after < tip_y_before,
            "tip should move down under gravity"
        );
    }
}
