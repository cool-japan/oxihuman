// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Kirchhoff elastic rod simulation for hair, fur, and cable physics.
//!
//! Each rod is discretized into segments with per-segment frames.  The
//! simulation integrates bending and twisting energies at each time step
//! using a simple Verlet-style scheme.

// ── Configuration ─────────────────────────────────────────────────────────────

/// Simulation parameters for an [`ElasticRod`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ElasticRodConfig {
    /// Bending stiffness (N·m²).
    pub bending_stiffness: f32,
    /// Twisting stiffness (N·m²).
    pub twisting_stiffness: f32,
    /// Linear damping coefficient.
    pub damping: f32,
    /// Rest length of each segment (meters).
    pub rest_length: f32,
    /// Gravitational acceleration (m/s²), applied downward (−Y).
    pub gravity: f32,
}

/// Build an [`ElasticRodConfig`] with sensible defaults.
#[allow(dead_code)]
pub fn default_elastic_rod_config() -> ElasticRodConfig {
    ElasticRodConfig {
        bending_stiffness: 0.05,
        twisting_stiffness: 0.02,
        damping: 0.98,
        rest_length: 0.05,
        gravity: 9.81,
    }
}

// ── Segment ───────────────────────────────────────────────────────────────────

/// One segment (edge) of the rod.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RodSegment {
    /// World-space position of the segment's proximal (start) point.
    pub position: [f32; 3],
    /// Current velocity.
    pub velocity: [f32; 3],
    /// Whether this point is pinned (zero velocity, zero force applied).
    pub pinned: bool,
    /// Rest-pose material frame's reference direction.
    pub rest_frame: [f32; 3],
}

// ── Elastic rod ───────────────────────────────────────────────────────────────

/// An elastic rod consisting of a chain of [`RodSegment`]s.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ElasticRod {
    /// Ordered list of segments (first segment is the root).
    pub segments: Vec<RodSegment>,
    /// Simulation configuration.
    pub config: ElasticRodConfig,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Create a new, empty [`ElasticRod`] with the given configuration.
#[allow(dead_code)]
pub fn new_elastic_rod(config: ElasticRodConfig) -> ElasticRod {
    ElasticRod {
        segments: Vec::new(),
        config,
    }
}

/// Append a new segment to the rod at the given position.
#[allow(dead_code)]
pub fn add_rod_segment(rod: &mut ElasticRod, position: [f32; 3]) {
    rod.segments.push(RodSegment {
        position,
        velocity: [0.0; 3],
        pinned: false,
        rest_frame: [0.0, 0.0, 1.0],
    });
}

/// Return the number of segments in the rod.
#[allow(dead_code)]
pub fn rod_segment_count(rod: &ElasticRod) -> usize {
    rod.segments.len()
}

/// Integrate the rod by one time step `dt`, applying bending, twisting, and gravity forces.
#[allow(dead_code)]
pub fn update_elastic_rod(rod: &mut ElasticRod, dt: f32) {
    let n = rod.segments.len();
    if n < 2 {
        return;
    }

    let bending = rod.config.bending_stiffness;
    let twisting = rod.config.twisting_stiffness;
    let damp = rod.config.damping;
    let grav = rod.config.gravity;
    let rest = rod.config.rest_length;

    // Build force accumulator
    let mut forces = vec![[0.0f32; 3]; n];

    // Gravity
    for f in forces.iter_mut() {
        f[1] -= grav;
    }

    // Stretch forces (spring along each edge)
    for i in 0..n - 1 {
        let diff = sub3(rod.segments[i + 1].position, rod.segments[i].position);
        let len = len3(diff);
        if len < 1e-10 {
            continue;
        }
        let dir = scale3(diff, 1.0 / len);
        let stretch = (len - rest) * bending;
        forces[i] = add3(forces[i], scale3(dir, stretch));
        forces[i + 1] = add3(forces[i + 1], scale3(dir, -stretch));
    }

    // Bending forces (second-order curvature between consecutive edges)
    for i in 1..n - 1 {
        let p0 = rod.segments[i - 1].position;
        let p1 = rod.segments[i].position;
        let p2 = rod.segments[i + 1].position;
        let e0 = normalize3(sub3(p1, p0));
        let e1 = normalize3(sub3(p2, p1));
        let kappa = cross3(e0, e1); // discrete curvature vector
        let fb = scale3(kappa, bending);
        forces[i - 1] = add3(forces[i - 1], scale3(fb, -0.5));
        forces[i] = add3(forces[i], fb);
        forces[i + 1] = add3(forces[i + 1], scale3(fb, -0.5));
    }

    // Twisting forces (penalise frame rotation around tangent)
    for (idx, win) in rod.segments.windows(3).enumerate() {
        let i = idx + 1; // win[0]=i-1, win[1]=i, win[2]=i+1
        let t0 = normalize3(sub3(win[1].position, win[0].position));
        let t1 = normalize3(sub3(win[2].position, win[1].position));
        // Twist = sin of angle between frames projected out of tangent
        let twist = cross3(t0, t1);
        let ft = scale3(twist, -twisting);
        forces[i] = add3(forces[i], ft);
    }

    // Integrate velocities and positions
    for (i, seg) in rod.segments.iter_mut().enumerate() {
        if seg.pinned {
            seg.velocity = [0.0; 3];
            continue;
        }
        seg.velocity = add3(seg.velocity, scale3(forces[i], dt));
        seg.velocity = scale3(seg.velocity, damp);
        seg.position = add3(seg.position, scale3(seg.velocity, dt));
    }
}

/// Compute the bending energy of the rod (sum of squared curvatures × stiffness).
#[allow(dead_code)]
pub fn rod_bending_energy(rod: &ElasticRod) -> f32 {
    let n = rod.segments.len();
    if n < 3 {
        return 0.0;
    }
    let mut energy = 0.0f32;
    for i in 1..n - 1 {
        let p0 = rod.segments[i - 1].position;
        let p1 = rod.segments[i].position;
        let p2 = rod.segments[i + 1].position;
        let e0 = normalize3(sub3(p1, p0));
        let e1 = normalize3(sub3(p2, p1));
        let kappa = cross3(e0, e1);
        energy += dot3(kappa, kappa) * rod.config.bending_stiffness * 0.5;
    }
    energy
}

/// Compute the twisting energy of the rod.
#[allow(dead_code)]
pub fn rod_twisting_energy(rod: &ElasticRod) -> f32 {
    let n = rod.segments.len();
    if n < 3 {
        return 0.0;
    }
    let mut energy = 0.0f32;
    for i in 1..n - 1 {
        let p0 = rod.segments[i - 1].position;
        let p1 = rod.segments[i].position;
        let p2 = rod.segments[i + 1].position;
        let t0 = normalize3(sub3(p1, p0));
        let t1 = normalize3(sub3(p2, p1));
        let twist = cross3(t0, t1);
        energy += dot3(twist, twist) * rod.config.twisting_stiffness * 0.5;
    }
    energy
}

/// Return the total mechanical energy (bending + twisting + kinetic).
#[allow(dead_code)]
pub fn rod_total_energy(rod: &ElasticRod) -> f32 {
    let kinetic: f32 = rod
        .segments
        .iter()
        .map(|s| dot3(s.velocity, s.velocity) * 0.5)
        .sum();
    rod_bending_energy(rod) + rod_twisting_energy(rod) + kinetic
}

/// Apply a gravity impulse to all unpinned segments.
#[allow(dead_code)]
pub fn apply_gravity_to_rod(rod: &mut ElasticRod, dt: f32) {
    let g = rod.config.gravity;
    for seg in rod.segments.iter_mut() {
        if !seg.pinned {
            seg.velocity[1] -= g * dt;
        }
    }
}

/// Pin (fix) one end of the rod by index.
#[allow(dead_code)]
pub fn pin_rod_end(rod: &mut ElasticRod, index: usize) {
    if let Some(seg) = rod.segments.get_mut(index) {
        seg.pinned = true;
        seg.velocity = [0.0; 3];
    }
}

/// Compute the total arc length of the rod.
#[allow(dead_code)]
pub fn rod_length(rod: &ElasticRod) -> f32 {
    rod.segments
        .windows(2)
        .map(|w| len3(sub3(w[1].position, w[0].position)))
        .sum()
}

/// Return the unit tangent vector at segment `i` (from segment i to i+1).
///
/// Returns `[0,1,0]` for out-of-range or degenerate indices.
#[allow(dead_code)]
pub fn rod_centerline_tangent(rod: &ElasticRod, i: usize) -> [f32; 3] {
    let n = rod.segments.len();
    if n < 2 || i + 1 >= n {
        return [0.0, 1.0, 0.0];
    }
    let diff = sub3(rod.segments[i + 1].position, rod.segments[i].position);
    normalize3(diff)
}

/// Compute the discrete curvature vector at segment `i` (requires i ≥ 1 and i+1 < n).
#[allow(dead_code)]
pub fn rod_curvature_at(rod: &ElasticRod, i: usize) -> [f32; 3] {
    let n = rod.segments.len();
    if i == 0 || i + 1 >= n {
        return [0.0; 3];
    }
    let p0 = rod.segments[i - 1].position;
    let p1 = rod.segments[i].position;
    let p2 = rod.segments[i + 1].position;
    let e0 = normalize3(sub3(p1, p0));
    let e1 = normalize3(sub3(p2, p1));
    cross3(e0, e1)
}

/// Reset all segment velocities to zero.
#[allow(dead_code)]
pub fn reset_rod(rod: &mut ElasticRod) {
    for seg in rod.segments.iter_mut() {
        seg.velocity = [0.0; 3];
    }
}

// ── Math helpers ──────────────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v).max(1e-10);
    [v[0] / l, v[1] / l, v[2] / l]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn straight_rod(n: usize) -> ElasticRod {
        let mut rod = new_elastic_rod(default_elastic_rod_config());
        for i in 0..n {
            add_rod_segment(&mut rod, [0.0, i as f32 * 0.05, 0.0]);
        }
        pin_rod_end(&mut rod, 0);
        rod
    }

    #[test]
    fn default_config_positive_stiffness() {
        let c = default_elastic_rod_config();
        assert!(c.bending_stiffness > 0.0);
        assert!(c.twisting_stiffness > 0.0);
    }

    #[test]
    fn new_elastic_rod_empty() {
        let rod = new_elastic_rod(default_elastic_rod_config());
        assert_eq!(rod_segment_count(&rod), 0);
    }

    #[test]
    fn add_rod_segment_increments_count() {
        let mut rod = new_elastic_rod(default_elastic_rod_config());
        add_rod_segment(&mut rod, [0.0, 0.0, 0.0]);
        assert_eq!(rod_segment_count(&rod), 1);
        add_rod_segment(&mut rod, [0.0, 0.05, 0.0]);
        assert_eq!(rod_segment_count(&rod), 2);
    }

    #[test]
    fn rod_segment_count_correct() {
        let rod = straight_rod(5);
        assert_eq!(rod_segment_count(&rod), 5);
    }

    #[test]
    fn rod_length_correct_for_straight_rod() {
        let rod = straight_rod(6); // 5 segments of 0.05 m
        let l = rod_length(&rod);
        assert!((l - 0.25).abs() < 1e-4, "length={l}");
    }

    #[test]
    fn pin_rod_end_marks_pinned() {
        let mut rod = straight_rod(4);
        pin_rod_end(&mut rod, 0);
        assert!(rod.segments[0].pinned);
    }

    #[test]
    fn update_elastic_rod_pinned_segment_stays() {
        let mut rod = straight_rod(4);
        let pos_before = rod.segments[0].position;
        update_elastic_rod(&mut rod, 0.01);
        let pos_after = rod.segments[0].position;
        assert_eq!(pos_before, pos_after);
    }

    #[test]
    fn update_elastic_rod_free_segment_moves() {
        let mut rod = straight_rod(4);
        let pos_before = rod.segments[3].position;
        update_elastic_rod(&mut rod, 0.01);
        let pos_after = rod.segments[3].position;
        // Gravity should pull the tip down
        assert_ne!(
            pos_before, pos_after,
            "free segment should move under gravity"
        );
    }

    #[test]
    fn rod_bending_energy_zero_for_short_rod() {
        let rod = straight_rod(2);
        assert_eq!(rod_bending_energy(&rod), 0.0);
    }

    #[test]
    fn rod_bending_energy_nonneg_for_long_rod() {
        let rod = straight_rod(6);
        assert!(rod_bending_energy(&rod) >= 0.0);
    }

    #[test]
    fn rod_twisting_energy_nonneg() {
        let rod = straight_rod(6);
        assert!(rod_twisting_energy(&rod) >= 0.0);
    }

    #[test]
    fn rod_total_energy_equals_sum() {
        let rod = straight_rod(6);
        let e = rod_total_energy(&rod);
        let manual = rod_bending_energy(&rod) + rod_twisting_energy(&rod);
        // kinetic energy is zero for a fresh rod
        assert!((e - manual).abs() < 1e-5, "total={e} manual={manual}");
    }

    #[test]
    fn apply_gravity_to_rod_changes_velocity() {
        let mut rod = straight_rod(4);
        // unpin all
        for seg in rod.segments.iter_mut() {
            seg.pinned = false;
        }
        let v_before = rod.segments[2].velocity[1];
        apply_gravity_to_rod(&mut rod, 0.01);
        let v_after = rod.segments[2].velocity[1];
        assert!(v_after < v_before, "gravity should decrease vy");
    }

    #[test]
    fn rod_centerline_tangent_unit_length() {
        let rod = straight_rod(4);
        let t = rod_centerline_tangent(&rod, 1);
        let l = len3(t);
        assert!((l - 1.0).abs() < 1e-5, "tangent length={l}");
    }

    #[test]
    fn rod_curvature_at_zero_for_straight_rod() {
        let rod = straight_rod(5);
        let kappa = rod_curvature_at(&rod, 2);
        let k = len3(kappa);
        assert!(k < 1e-4, "straight rod curvature should be ~0, got {k}");
    }

    #[test]
    fn reset_rod_zeroes_velocities() {
        let mut rod = straight_rod(4);
        update_elastic_rod(&mut rod, 0.1);
        reset_rod(&mut rod);
        for seg in &rod.segments {
            assert_eq!(seg.velocity, [0.0; 3]);
        }
    }
}
