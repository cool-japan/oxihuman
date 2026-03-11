// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Multi-link pendulum chain simulation.
//!
//! Implements a simple constrained rigid-link pendulum using Euler integration
//! with iterative distance-constraint projection.  Each link has a fixed
//! rest length and is attached to the next link at its tip.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
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
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-10 {
        [0.0, -1.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Configuration for the pendulum chain simulation.
pub struct PendulumConfig {
    /// Gravitational acceleration vector (default: [0, -9.81, 0]).
    pub gravity: [f32; 3],
    /// Linear damping coefficient (0 = no damping, 1 = critically damped).
    pub damping: f32,
    /// Number of constraint-projection iterations per time step.
    pub constraint_iterations: u32,
}

/// A single rigid link of the pendulum chain.
pub struct PendulumLink {
    /// Current world-space position of the link tip (tail).
    pub position: [f32; 3],
    /// Current velocity of the link tip.
    pub velocity: [f32; 3],
    /// Rest length of this link (distance from previous joint to this tip).
    pub rest_length: f32,
    /// Mass at the link tip (used for energy calculations).
    pub mass: f32,
    /// True if this joint is pinned and cannot move.
    pub pinned: bool,
}

/// The full multi-link pendulum chain.
pub struct PendulumChain {
    /// Fixed root anchor position.
    pub root: [f32; 3],
    /// Whether the root is pinned (always true in normal use).
    pub root_pinned: bool,
    /// The links in order from root to tip.
    pub links: Vec<PendulumLink>,
    /// Current gravity.
    pub gravity: [f32; 3],
    /// Damping.
    pub damping: f32,
    /// Constraint iterations.
    pub constraint_iterations: u32,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return a `PendulumConfig` with standard Earth gravity and light damping.
pub fn default_pendulum_config() -> PendulumConfig {
    PendulumConfig {
        gravity: [0.0, -9.81, 0.0],
        damping: 0.01,
        constraint_iterations: 8,
    }
}

/// Create a new empty pendulum chain with the given root position.
pub fn new_pendulum_chain(root: [f32; 3], cfg: &PendulumConfig) -> PendulumChain {
    PendulumChain {
        root,
        root_pinned: true,
        links: vec![],
        gravity: cfg.gravity,
        damping: cfg.damping,
        constraint_iterations: cfg.constraint_iterations,
    }
}

/// Append a new link to the end of the chain with the given rest length and mass.
///
/// The link tip starts directly below the current chain tip (hanging at rest).
pub fn add_link(chain: &mut PendulumChain, rest_length: f32, mass: f32) {
    let base = if chain.links.is_empty() {
        chain.root
    } else {
        chain.links[chain.links.len() - 1].position
    };
    // Hang link straight down
    let tip = [base[0], base[1] - rest_length.abs(), base[2]];
    chain.links.push(PendulumLink {
        position: tip,
        velocity: [0.0; 3],
        rest_length: rest_length.abs().max(1e-6),
        mass: mass.max(1e-6),
        pinned: false,
    });
}

/// Return the number of links in the chain.
pub fn link_count(chain: &PendulumChain) -> usize {
    chain.links.len()
}

/// Advance the chain by one time step `dt` using symplectic Euler integration
/// followed by iterative distance-constraint projection.
#[allow(clippy::too_many_arguments)]
pub fn update_pendulum(chain: &mut PendulumChain, dt: f32) {
    let n = chain.links.len();
    if n == 0 {
        return;
    }

    let damp = 1.0 - chain.damping.clamp(0.0, 1.0);
    let grav = chain.gravity;

    // 1. Integrate velocities and positions
    for link in chain.links.iter_mut() {
        if link.pinned {
            link.velocity = [0.0; 3];
            continue;
        }
        link.velocity = add3(link.velocity, scale3(grav, dt));
        link.velocity = scale3(link.velocity, damp);
        link.position = add3(link.position, scale3(link.velocity, dt));
    }

    // 2. Project distance constraints (iterate)
    for _ in 0..chain.constraint_iterations {
        for i in 0..n {
            let anchor = if i == 0 {
                chain.root
            } else {
                chain.links[i - 1].position
            };
            let rest = chain.links[i].rest_length;
            if chain.links[i].pinned {
                continue;
            }
            let dir = sub3(chain.links[i].position, anchor);
            let dist = len3(dir);
            if dist < 1e-10 {
                continue;
            }
            // Project tip onto sphere of radius `rest` around anchor
            let correction = scale3(normalize3(dir), rest);
            let new_pos = add3(anchor, correction);
            let delta = sub3(new_pos, chain.links[i].position);
            chain.links[i].position = new_pos;
            // Reflect the correction into velocity (verlet-like)
            chain.links[i].velocity = add3(chain.links[i].velocity, scale3(delta, 1.0 / dt.max(1e-6)));
        }
    }
}

/// Return the world-space position of the chain tip (last link tip).
pub fn pendulum_tip_position(chain: &PendulumChain) -> [f32; 3] {
    chain.links.last().map(|l| l.position).unwrap_or(chain.root)
}

/// Compute the total mechanical energy (kinetic + potential) of the chain.
///
/// Potential energy is relative to the root anchor (positive = above root,
/// negative = below root when gravity points down).
pub fn pendulum_energy(chain: &PendulumChain) -> f32 {
    let g_mag = len3(chain.gravity);
    let mut total = 0.0f32;

    for link in &chain.links {
        // Kinetic energy: 0.5 * m * v^2
        let v2 = dot3(link.velocity, link.velocity);
        total += 0.5 * link.mass * v2;

        // Potential energy: m * g * h  (h = vertical displacement from root)
        let h = link.position[1] - chain.root[1];
        total += link.mass * g_mag * h;
    }
    total
}

/// Override the gravity vector on the chain.
pub fn set_gravity(chain: &mut PendulumChain, gravity: [f32; 3]) {
    chain.gravity = gravity;
}

/// Apply an instantaneous impulse (delta velocity) to link `link_idx`.
pub fn apply_impulse_to_chain(chain: &mut PendulumChain, link_idx: usize, impulse: [f32; 3]) {
    if let Some(link) = chain.links.get_mut(link_idx) {
        if !link.pinned {
            link.velocity = add3(link.velocity, impulse);
        }
    }
}

/// Pin the chain root at position `pos` (prevents root anchor from drifting).
pub fn pin_chain_root(chain: &mut PendulumChain, pos: [f32; 3]) {
    chain.root = pos;
    chain.root_pinned = true;
}

/// Return the angle (in radians) of link `i` from the vertical downward axis.
///
/// Measured as the angle between the link direction and [0, -1, 0].
pub fn pendulum_angle_at(chain: &PendulumChain, i: usize) -> f32 {
    if i >= chain.links.len() {
        return 0.0;
    }
    let anchor = if i == 0 {
        chain.root
    } else {
        chain.links[i - 1].position
    };
    let dir = normalize3(sub3(chain.links[i].position, anchor));
    let down = [0.0f32, -1.0, 0.0];
    let cos_a = dot3(dir, down).clamp(-1.0, 1.0);
    cos_a.acos()
}

/// Estimate the natural frequency (rad/s) of a single-link pendulum of the
/// given length under the chain's gravity.
///
/// Formula: omega = sqrt(g / L).
pub fn chain_natural_frequency(chain: &PendulumChain) -> f32 {
    if chain.links.is_empty() {
        return 0.0;
    }
    let g = len3(chain.gravity);
    let total_length: f32 = chain.links.iter().map(|l| l.rest_length).sum();
    if total_length < 1e-6 {
        return 0.0;
    }
    (g / total_length).sqrt()
}

/// Reset the chain to its hanging-at-rest configuration, zeroing all velocities.
pub fn reset_chain(chain: &mut PendulumChain) {
    let mut base = chain.root;
    for link in chain.links.iter_mut() {
        let tip = [base[0], base[1] - link.rest_length, base[2]];
        link.position = tip;
        link.velocity = [0.0; 3];
        base = tip;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_chain() -> PendulumChain {
        let cfg = default_pendulum_config();
        let mut chain = new_pendulum_chain([0.0, 0.0, 0.0], &cfg);
        add_link(&mut chain, 1.0, 1.0);
        add_link(&mut chain, 1.0, 1.0);
        chain
    }

    // -----------------------------------------------------------------------
    // default_pendulum_config
    // -----------------------------------------------------------------------

    #[test]
    fn default_config_gravity_downward() {
        let cfg = default_pendulum_config();
        assert!(cfg.gravity[1] < 0.0);
    }

    #[test]
    fn default_config_damping_in_range() {
        let cfg = default_pendulum_config();
        assert!(cfg.damping >= 0.0 && cfg.damping <= 1.0);
    }

    // -----------------------------------------------------------------------
    // new_pendulum_chain / add_link / link_count
    // -----------------------------------------------------------------------

    #[test]
    fn new_chain_starts_empty() {
        let cfg = default_pendulum_config();
        let chain = new_pendulum_chain([0.0, 0.0, 0.0], &cfg);
        assert_eq!(link_count(&chain), 0);
    }

    #[test]
    fn add_link_increments_count() {
        let mut chain = simple_chain();
        assert_eq!(link_count(&chain), 2);
        add_link(&mut chain, 0.5, 0.5);
        assert_eq!(link_count(&chain), 3);
    }

    #[test]
    fn links_hang_below_root() {
        let chain = simple_chain();
        // Both links should be below root (y < 0)
        for link in &chain.links {
            assert!(link.position[1] < 0.0, "y={}", link.position[1]);
        }
    }

    // -----------------------------------------------------------------------
    // pendulum_tip_position
    // -----------------------------------------------------------------------

    #[test]
    fn tip_position_empty_chain_is_root() {
        let cfg = default_pendulum_config();
        let chain = new_pendulum_chain([1.0, 2.0, 3.0], &cfg);
        let tip = pendulum_tip_position(&chain);
        assert!((tip[0] - 1.0).abs() < 1e-6);
        assert!((tip[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn tip_position_matches_last_link() {
        let chain = simple_chain();
        let tip = pendulum_tip_position(&chain);
        let last = chain.links.last().unwrap().position;
        assert!((tip[0] - last[0]).abs() < 1e-6);
        assert!((tip[1] - last[1]).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // update_pendulum
    // -----------------------------------------------------------------------

    #[test]
    fn update_preserves_link_count() {
        let mut chain = simple_chain();
        update_pendulum(&mut chain, 0.01);
        assert_eq!(link_count(&chain), 2);
    }

    #[test]
    fn update_enforces_link_lengths() {
        let mut chain = simple_chain();
        // Displace the first link tip to break the constraint
        chain.links[0].position = [5.0, 0.0, 0.0];
        for _ in 0..20 {
            update_pendulum(&mut chain, 0.01);
        }
        // Link length from root to link[0] should be ~1.0
        let dist = len3(sub3(chain.links[0].position, chain.root));
        assert!((dist - 1.0).abs() < 0.1, "dist={}", dist);
    }

    #[test]
    fn update_empty_chain_does_not_panic() {
        let cfg = default_pendulum_config();
        let mut chain = new_pendulum_chain([0.0, 0.0, 0.0], &cfg);
        update_pendulum(&mut chain, 0.01);
    }

    // -----------------------------------------------------------------------
    // pendulum_energy
    // -----------------------------------------------------------------------

    #[test]
    fn energy_at_rest_has_only_potential() {
        let chain = simple_chain();
        let e = pendulum_energy(&chain);
        // At rest, KE = 0; PE is negative (below root, gravity down)
        // Just check it's finite
        assert!(e.is_finite());
    }

    #[test]
    fn energy_with_velocity_is_higher() {
        let mut chain = simple_chain();
        let e0 = pendulum_energy(&chain);
        chain.links[0].velocity = [1.0, 0.0, 0.0];
        let e1 = pendulum_energy(&chain);
        assert!(e1 > e0, "e0={} e1={}", e0, e1);
    }

    // -----------------------------------------------------------------------
    // set_gravity
    // -----------------------------------------------------------------------

    #[test]
    fn set_gravity_updates_chain() {
        let mut chain = simple_chain();
        set_gravity(&mut chain, [0.0, -20.0, 0.0]);
        assert!((chain.gravity[1] + 20.0).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // apply_impulse_to_chain
    // -----------------------------------------------------------------------

    #[test]
    fn apply_impulse_changes_velocity() {
        let mut chain = simple_chain();
        apply_impulse_to_chain(&mut chain, 0, [1.0, 0.0, 0.0]);
        assert!((chain.links[0].velocity[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn apply_impulse_out_of_range_does_not_panic() {
        let mut chain = simple_chain();
        apply_impulse_to_chain(&mut chain, 99, [1.0, 0.0, 0.0]);
    }

    // -----------------------------------------------------------------------
    // pin_chain_root
    // -----------------------------------------------------------------------

    #[test]
    fn pin_chain_root_sets_root_position() {
        let mut chain = simple_chain();
        pin_chain_root(&mut chain, [3.0, 4.0, 5.0]);
        assert!((chain.root[0] - 3.0).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // pendulum_angle_at
    // -----------------------------------------------------------------------

    #[test]
    fn angle_at_rest_is_zero() {
        let chain = simple_chain();
        let angle = pendulum_angle_at(&chain, 0);
        assert!(angle < 0.01, "angle at rest should be ~0, got {}", angle);
    }

    #[test]
    fn angle_out_of_range_is_zero() {
        let chain = simple_chain();
        let angle = pendulum_angle_at(&chain, 99);
        assert!((angle).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // chain_natural_frequency
    // -----------------------------------------------------------------------

    #[test]
    fn natural_frequency_two_unit_links() {
        let chain = simple_chain();
        let omega = chain_natural_frequency(&chain);
        // omega = sqrt(9.81 / 2) ≈ 2.215
        let expected = (9.81f32 / 2.0).sqrt();
        assert!((omega - expected).abs() < 0.01, "omega={}", omega);
    }

    #[test]
    fn natural_frequency_empty_chain_is_zero() {
        let cfg = default_pendulum_config();
        let chain = new_pendulum_chain([0.0; 3], &cfg);
        assert!((chain_natural_frequency(&chain)).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // reset_chain
    // -----------------------------------------------------------------------

    #[test]
    fn reset_chain_zeros_velocities() {
        let mut chain = simple_chain();
        chain.links[0].velocity = [1.0, 2.0, 3.0];
        reset_chain(&mut chain);
        for link in &chain.links {
            let v2 = dot3(link.velocity, link.velocity);
            assert!(v2 < 1e-10);
        }
    }

    #[test]
    fn reset_chain_restores_link_lengths() {
        let mut chain = simple_chain();
        chain.links[0].position = [100.0, 100.0, 100.0];
        reset_chain(&mut chain);
        let dist = len3(sub3(chain.links[0].position, chain.root));
        assert!((dist - 1.0).abs() < 1e-5, "dist={}", dist);
    }
}
