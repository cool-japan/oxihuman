// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Position-based dynamics rope/cable simulation using Verlet integration.

#[allow(dead_code)]
pub struct RopeParticle {
    pub position: [f32; 3],
    pub prev_position: [f32; 3],
    pub mass: f32,
    pub pinned: bool,
}

#[allow(dead_code)]
pub struct RopeSegment {
    pub particle_a: usize,
    pub particle_b: usize,
    pub rest_length: f32,
    pub stiffness: f32,
}

#[allow(dead_code)]
pub struct RopeConfig {
    pub gravity: [f32; 3],
    pub damping: f32,
    pub substeps: u32,
    pub bend_stiffness: f32,
}

#[allow(dead_code)]
pub struct Rope {
    pub particles: Vec<RopeParticle>,
    pub segments: Vec<RopeSegment>,
    pub config: RopeConfig,
}

// ── helpers ───────────────────────────────────────────────────────────────────

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn len3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

// ── public API ────────────────────────────────────────────────────────────────

/// Create a straight vertical rope hanging downward from `start`.
#[allow(dead_code)]
pub fn new_rope(n_particles: usize, spacing: f32, start: [f32; 3], config: RopeConfig) -> Rope {
    assert!(n_particles >= 2, "Rope needs at least 2 particles");
    let mut particles = Vec::with_capacity(n_particles);
    for i in 0..n_particles {
        let pos = [start[0], start[1] - spacing * i as f32, start[2]];
        particles.push(RopeParticle {
            position: pos,
            prev_position: pos,
            mass: 1.0,
            pinned: false,
        });
    }
    // Pin the first particle
    particles[0].pinned = true;

    let mut segments = Vec::with_capacity(n_particles - 1);
    for i in 0..(n_particles - 1) {
        segments.push(RopeSegment {
            particle_a: i,
            particle_b: i + 1,
            rest_length: spacing,
            stiffness: 1.0,
        });
    }

    Rope {
        particles,
        segments,
        config,
    }
}

/// Advance the rope simulation by `dt` seconds using Verlet integration
/// and position-based distance constraint solving.
#[allow(dead_code)]
pub fn step_rope(rope: &mut Rope, dt: f32) {
    let sub_dt = dt / rope.config.substeps as f32;
    let gravity = rope.config.gravity;
    let damping = rope.config.damping;

    for _ in 0..rope.config.substeps {
        // Verlet integrate
        for p in &mut rope.particles {
            if p.pinned {
                continue;
            }
            let vel = sub3(p.position, p.prev_position);
            let vel_damped = scale3(vel, 1.0 - damping);
            let gravity_contrib = scale3(gravity, sub_dt * sub_dt);
            let new_pos = add3(add3(p.position, vel_damped), gravity_contrib);
            p.prev_position = p.position;
            p.position = new_pos;
        }

        // PBD constraint solving
        let n_seg = rope.segments.len();
        for _ in 0..4 {
            for si in 0..n_seg {
                let pa_idx = rope.segments[si].particle_a;
                let pb_idx = rope.segments[si].particle_b;
                let rest = rope.segments[si].rest_length;
                let stiff = rope.segments[si].stiffness;

                let pa = rope.particles[pa_idx].position;
                let pb = rope.particles[pb_idx].position;
                let delta = sub3(pb, pa);
                let cur_len = len3(delta);
                if cur_len < 1e-10 {
                    continue;
                }
                let correction = (cur_len - rest) / cur_len * stiff;
                let half = scale3(delta, correction * 0.5);

                let pinned_a = rope.particles[pa_idx].pinned;
                let pinned_b = rope.particles[pb_idx].pinned;

                match (pinned_a, pinned_b) {
                    (false, false) => {
                        rope.particles[pa_idx].position = add3(pa, half);
                        rope.particles[pb_idx].position = sub3(pb, half);
                    }
                    (false, true) => {
                        rope.particles[pa_idx].position = add3(pa, scale3(delta, correction));
                    }
                    (true, false) => {
                        rope.particles[pb_idx].position = sub3(pb, scale3(delta, correction));
                    }
                    (true, true) => {}
                }
            }
        }
    }
}

/// Pin a particle so it doesn't move.
#[allow(dead_code)]
pub fn pin_particle(rope: &mut Rope, idx: usize) {
    if let Some(p) = rope.particles.get_mut(idx) {
        p.pinned = true;
    }
}

/// Unpin a particle so it can move freely.
#[allow(dead_code)]
pub fn unpin_particle(rope: &mut Rope, idx: usize) {
    if let Some(p) = rope.particles.get_mut(idx) {
        p.pinned = false;
    }
}

/// Apply an instantaneous velocity impulse to a particle.
#[allow(dead_code)]
pub fn apply_impulse_to_rope(rope: &mut Rope, particle_idx: usize, impulse: [f32; 3]) {
    if let Some(p) = rope.particles.get_mut(particle_idx) {
        if !p.pinned && p.mass > 0.0 {
            let dv = scale3(impulse, 1.0 / p.mass);
            p.prev_position = sub3(p.prev_position, dv);
        }
    }
}

/// Sum of all segment rest lengths.
#[allow(dead_code)]
pub fn rope_length(rope: &Rope) -> f32 {
    rope.segments.iter().map(|s| s.rest_length).sum()
}

/// Position of the last particle.
#[allow(dead_code)]
pub fn rope_end_position(rope: &Rope) -> [f32; 3] {
    rope.particles
        .last()
        .map(|p| p.position)
        .unwrap_or([0.0; 3])
}

/// Kinetic energy proxy: sum of |velocity|² for all non-pinned particles.
#[allow(dead_code)]
pub fn rope_energy(rope: &Rope) -> f32 {
    rope.particles
        .iter()
        .filter(|p| !p.pinned)
        .map(|p| {
            let v = sub3(p.position, p.prev_position);
            p.mass * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
        })
        .sum()
}

/// Current stretch ratio for a segment (current_length / rest_length).
#[allow(dead_code)]
pub fn rope_tension_at(rope: &Rope, segment_idx: usize) -> f32 {
    let seg = match rope.segments.get(segment_idx) {
        Some(s) => s,
        None => return 0.0,
    };
    let pa = rope.particles[seg.particle_a].position;
    let pb = rope.particles[seg.particle_b].position;
    let cur = len3(sub3(pb, pa));
    if seg.rest_length < 1e-10 {
        1.0
    } else {
        cur / seg.rest_length
    }
}

/// Return all particle positions in order.
#[allow(dead_code)]
pub fn rope_to_polyline(rope: &Rope) -> Vec<[f32; 3]> {
    rope.particles.iter().map(|p| p.position).collect()
}

/// Attach (and pin) the last particle to a fixed world position.
#[allow(dead_code)]
pub fn attach_rope_end(rope: &mut Rope, pos: [f32; 3]) {
    if let Some(p) = rope.particles.last_mut() {
        p.position = pos;
        p.prev_position = pos;
        p.pinned = true;
    }
}

/// Maximum vertical (Y-axis) displacement from the straight line between
/// the first and last particles.
#[allow(dead_code)]
pub fn rope_sag(rope: &Rope) -> f32 {
    if rope.particles.len() < 2 {
        return 0.0;
    }
    let start = rope.particles[0].position;
    let end = rope.particles[rope.particles.len() - 1].position;
    let n = rope.particles.len();

    rope.particles
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let t = i as f32 / (n - 1) as f32;
            let line_y = start[1] + (end[1] - start[1]) * t;
            (p.position[1] - line_y).abs()
        })
        .fold(0.0f32, f32::max)
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> RopeConfig {
        RopeConfig {
            gravity: [0.0, -9.81, 0.0],
            damping: 0.01,
            substeps: 4,
            bend_stiffness: 0.1,
        }
    }

    fn simple_rope(n: usize) -> Rope {
        new_rope(n, 1.0, [0.0, 0.0, 0.0], default_config())
    }

    // 1
    #[test]
    fn new_rope_particle_count() {
        let rope = simple_rope(5);
        assert_eq!(rope.particles.len(), 5);
    }

    // 2
    #[test]
    fn new_rope_segment_count() {
        let rope = simple_rope(5);
        assert_eq!(rope.segments.len(), 4);
    }

    // 3
    #[test]
    fn new_rope_first_pinned() {
        let rope = simple_rope(4);
        assert!(rope.particles[0].pinned);
    }

    // 4
    #[test]
    fn rope_length_correct() {
        let rope = new_rope(5, 1.0, [0.0, 0.0, 0.0], default_config());
        assert!((rope_length(&rope) - 4.0).abs() < 1e-5);
    }

    // 5
    #[test]
    fn rope_end_position_initial() {
        let rope = new_rope(5, 1.0, [0.0, 0.0, 0.0], default_config());
        let end = rope_end_position(&rope);
        assert!((end[1] + 4.0).abs() < 1e-5); // hanging down
    }

    // 6
    #[test]
    fn pin_unpin_particle() {
        let mut rope = simple_rope(4);
        unpin_particle(&mut rope, 0);
        assert!(!rope.particles[0].pinned);
        pin_particle(&mut rope, 0);
        assert!(rope.particles[0].pinned);
    }

    // 7
    #[test]
    fn step_rope_pinned_stays_fixed() {
        let mut rope = simple_rope(5);
        let initial = rope.particles[0].position;
        step_rope(&mut rope, 0.016);
        let after = rope.particles[0].position;
        assert!((after[0] - initial[0]).abs() < 1e-6);
        assert!((after[1] - initial[1]).abs() < 1e-6);
    }

    // 8
    #[test]
    fn step_rope_free_particle_moves() {
        let mut rope = simple_rope(3);
        let initial_y = rope.particles[2].position[1];
        // Run several steps so gravity has effect
        for _ in 0..10 {
            step_rope(&mut rope, 0.016);
        }
        // The free end should sag down (y decreases)
        assert!(rope.particles[2].position[1] <= initial_y);
    }

    // 9
    #[test]
    fn apply_impulse_changes_velocity() {
        let mut rope = simple_rope(4);
        let prev = rope.particles[3].prev_position;
        apply_impulse_to_rope(&mut rope, 3, [1.0, 0.0, 0.0]);
        // prev_position shifts, which effectively changes velocity
        assert!((rope.particles[3].prev_position[0] - prev[0]).abs() > 1e-6);
    }

    // 10
    #[test]
    fn rope_tension_at_rest() {
        let rope = simple_rope(4);
        let t = rope_tension_at(&rope, 0);
        // At rest the ratio should be ~1.0
        assert!((t - 1.0).abs() < 1e-4);
    }

    // 11
    #[test]
    fn rope_to_polyline_length() {
        let rope = simple_rope(5);
        let poly = rope_to_polyline(&rope);
        assert_eq!(poly.len(), 5);
    }

    // 12
    #[test]
    fn attach_rope_end_pins_last() {
        let mut rope = simple_rope(4);
        attach_rope_end(&mut rope, [5.0, 5.0, 5.0]);
        let last = rope.particles.last().unwrap();
        assert!(last.pinned);
        assert!((last.position[0] - 5.0).abs() < 1e-6);
    }

    // 13
    #[test]
    fn rope_sag_straight_rope_is_zero() {
        let rope = simple_rope(5);
        // All particles are on a straight vertical line → sag = 0
        let sag = rope_sag(&rope);
        assert!(sag < 1e-5);
    }

    // 14
    #[test]
    fn rope_energy_pinned_only_is_zero() {
        let mut rope = simple_rope(3);
        // Pin all particles
        for p in &mut rope.particles {
            p.pinned = true;
        }
        assert!(rope_energy(&rope) < 1e-6);
    }

    // 15
    #[test]
    fn rope_sag_after_steps_increases() {
        let mut rope = simple_rope(10);
        attach_rope_end(&mut rope, [0.0, -9.0, 0.0]);
        for _ in 0..100 {
            step_rope(&mut rope, 0.016);
        }
        // When both ends are pinned and gravity acts, the rope should sag
        let sag = rope_sag(&rope);
        assert!(sag > 0.0, "expected sag > 0, got {sag}");
    }
}
