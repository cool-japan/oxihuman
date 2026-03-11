// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Articulated body / multi-link rigid body system using reduced coordinates.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Global configuration for an articulated body system.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArticulatedConfig {
    /// Gravity acceleration [m/s²].
    pub gravity: [f32; 3],
    /// Velocity damping coefficient (0 = no damping, 1 = full stop).
    pub damping: f32,
    /// Maximum number of links allowed.
    pub max_links: usize,
}

/// A single rigid link in the articulated chain.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyLink {
    /// Descriptive label (e.g. "upper_arm_l").
    pub name: String,
    /// Mass [kg].
    pub mass: f32,
    /// Principal moments of inertia [kg·m²] (Ixx, Iyy, Izz).
    pub inertia: [f32; 3],
    /// Offset from parent joint in the parent's local frame [m].
    pub local_position: [f32; 3],
    /// Index of the parent link, or `None` for the root link.
    pub parent_idx: Option<usize>,
}

/// Dynamic state of an articulated system.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArticulatedState {
    /// World-space position of each link's centre of mass [m].
    pub link_positions: Vec<[f32; 3]>,
    /// World-space velocity of each link [m/s].
    pub link_velocities: Vec<[f32; 3]>,
    /// Joint angle for each link relative to its parent [rad].
    pub joint_angles: Vec<f32>,
}

/// A complete articulated body system.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArticulatedSystem {
    /// All links in topological order (root first).
    pub links: Vec<BodyLink>,
    /// Dynamic state.
    pub state: ArticulatedState,
    /// Configuration parameters.
    pub config: ArticulatedConfig,
}

/// Per-step output of [`step_articulated`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArticulatedResult {
    /// World-space positions after integration.
    pub positions: Vec<[f32; 3]>,
    /// Total kinetic energy ½Σmv².
    pub kinetic_energy: f32,
    /// Total gravitational potential energy Σmgh.
    pub potential_energy: f32,
}

// ── Internal helpers ──────────────────────────────────────────────────────────

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Rotate a 2D vector (x, y) by `angle` radians around the Z axis.
#[inline]
fn rotate_xy(v: [f32; 3], angle: f32) -> [f32; 3] {
    let (s, c) = angle.sin_cos();
    [v[0] * c - v[1] * s, v[0] * s + v[1] * c, v[2]]
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a sensible default [`ArticulatedConfig`].
#[allow(dead_code)]
pub fn default_articulated_config() -> ArticulatedConfig {
    ArticulatedConfig {
        gravity: [0.0, -9.81, 0.0],
        damping: 0.02,
        max_links: 64,
    }
}

/// Create a new [`BodyLink`].
///
/// Default inertia is `[0.01; 3]` and local_position is the origin.
#[allow(dead_code)]
pub fn new_body_link(name: &str, mass: f32, parent: Option<usize>) -> BodyLink {
    BodyLink {
        name: name.to_string(),
        mass,
        inertia: [0.01; 3],
        local_position: [0.0; 3],
        parent_idx: parent,
    }
}

/// Create a new empty articulated system.
#[allow(dead_code)]
pub fn new_articulated_system(cfg: ArticulatedConfig) -> ArticulatedSystem {
    ArticulatedSystem {
        links: Vec::new(),
        state: ArticulatedState {
            link_positions: Vec::new(),
            link_velocities: Vec::new(),
            joint_angles: Vec::new(),
        },
        config: cfg,
    }
}

/// Add a link to the system (position / velocity / joint angle initialised to zero).
#[allow(dead_code)]
pub fn add_link(sys: &mut ArticulatedSystem, link: BodyLink) {
    if sys.links.len() < sys.config.max_links {
        sys.links.push(link);
        sys.state.link_positions.push([0.0; 3]);
        sys.state.link_velocities.push([0.0; 3]);
        sys.state.joint_angles.push(0.0);
    }
}

/// Advance the simulation by `dt` seconds.
///
/// Algorithm (simplified forward dynamics):
/// 1. Forward kinematics: propagate parent positions through joint transforms.
/// 2. Apply gravitational acceleration.
/// 3. Semi-implicit Euler: integrate velocities then positions.
/// 4. Apply velocity damping.
#[allow(dead_code)]
pub fn step_articulated(sys: &mut ArticulatedSystem, dt: f32) -> ArticulatedResult {
    let n = sys.links.len();
    let g = sys.config.gravity;
    let damp = 1.0 - sys.config.damping.clamp(0.0, 1.0);

    // ── Forward kinematics (root may have no parent) ──────────────────────────
    for i in 0..n {
        let parent_pos = match sys.links[i].parent_idx {
            None => [0.0_f32; 3],
            Some(pi) => sys.state.link_positions[pi],
        };
        let angle = sys.state.joint_angles[i];
        let local = rotate_xy(sys.links[i].local_position, angle);
        sys.state.link_positions[i] = [
            parent_pos[0] + local[0],
            parent_pos[1] + local[1],
            parent_pos[2] + local[2],
        ];
    }

    // ── Gravity + integration ─────────────────────────────────────────────────
    for i in 0..n {
        sys.state.link_velocities[i][0] = (sys.state.link_velocities[i][0] + g[0] * dt) * damp;
        sys.state.link_velocities[i][1] = (sys.state.link_velocities[i][1] + g[1] * dt) * damp;
        sys.state.link_velocities[i][2] = (sys.state.link_velocities[i][2] + g[2] * dt) * damp;
        sys.state.link_positions[i][0] += sys.state.link_velocities[i][0] * dt;
        sys.state.link_positions[i][1] += sys.state.link_velocities[i][1] * dt;
        sys.state.link_positions[i][2] += sys.state.link_velocities[i][2] * dt;
    }

    // ── Diagnostics ───────────────────────────────────────────────────────────
    let mut kinetic_energy = 0.0_f32;
    let mut potential_energy = 0.0_f32;

    for i in 0..n {
        let v = sys.state.link_velocities[i];
        kinetic_energy += 0.5 * sys.links[i].mass * dot3(v, v);
        // PE = m·g·h  (h = y component)
        potential_energy += sys.links[i].mass * (-g[1]) * sys.state.link_positions[i][1];
    }

    ArticulatedResult {
        positions: sys.state.link_positions.clone(),
        kinetic_energy,
        potential_energy,
    }
}

/// Number of links in the system.
#[allow(dead_code)]
#[inline]
pub fn link_count(sys: &ArticulatedSystem) -> usize {
    sys.links.len()
}

/// Set joint angle for link `idx` [radians].
#[allow(dead_code)]
pub fn set_joint_angle(sys: &mut ArticulatedSystem, idx: usize, angle: f32) {
    if idx < sys.state.joint_angles.len() {
        sys.state.joint_angles[idx] = angle;
    }
}

/// Get joint angle for link `idx` [radians].
#[allow(dead_code)]
pub fn joint_angle(sys: &ArticulatedSystem, idx: usize) -> f32 {
    sys.state.joint_angles.get(idx).copied().unwrap_or(0.0)
}

/// Serialize the articulated system summary to JSON.
#[allow(dead_code)]
pub fn articulated_system_to_json(sys: &ArticulatedSystem) -> String {
    format!(
        "{{\"link_count\":{},\"damping\":{}}}",
        sys.links.len(),
        sys.config.damping
    )
}

/// Serialize an [`ArticulatedResult`] to JSON.
#[allow(dead_code)]
pub fn articulated_result_to_json(r: &ArticulatedResult) -> String {
    format!(
        "{{\"link_count\":{},\"kinetic_energy\":{},\"potential_energy\":{}}}",
        r.positions.len(),
        r.kinetic_energy,
        r.potential_energy
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn two_link_system() -> ArticulatedSystem {
        let cfg = default_articulated_config();
        let mut sys = new_articulated_system(cfg);
        let root = new_body_link("root", 1.0, None);
        let child = BodyLink {
            name: "child".to_string(),
            mass: 0.5,
            inertia: [0.01; 3],
            local_position: [0.0, 1.0, 0.0],
            parent_idx: Some(0),
        };
        add_link(&mut sys, root);
        add_link(&mut sys, child);
        sys
    }

    #[test]
    fn default_config_smoke() {
        let cfg = default_articulated_config();
        assert!(cfg.max_links > 0);
        assert!(cfg.damping >= 0.0 && cfg.damping <= 1.0);
    }

    #[test]
    fn add_link_increases_count() {
        let mut sys = new_articulated_system(default_articulated_config());
        assert_eq!(link_count(&sys), 0);
        add_link(&mut sys, new_body_link("root", 1.0, None));
        assert_eq!(link_count(&sys), 1);
    }

    #[test]
    fn set_and_get_joint_angle() {
        let mut sys = two_link_system();
        set_joint_angle(&mut sys, 1, 0.5);
        assert!((joint_angle(&sys, 1) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn step_gravity_accelerates_downward() {
        let mut sys = two_link_system();
        let _r1 = step_articulated(&mut sys, 0.1);
        let _r2 = step_articulated(&mut sys, 0.1);
        // velocity should be negative (downward) after gravity steps
        assert!(sys.state.link_velocities[0][1] < 0.0);
    }

    #[test]
    fn kinetic_energy_non_negative() {
        let mut sys = two_link_system();
        let result = step_articulated(&mut sys, 0.01);
        assert!(result.kinetic_energy >= 0.0);
    }

    #[test]
    fn json_contains_link_count() {
        let sys = two_link_system();
        let json = articulated_system_to_json(&sys);
        assert!(json.contains("link_count"));
        assert!(json.contains("damping"));
    }

    #[test]
    fn result_json_contains_energy_fields() {
        let mut sys = two_link_system();
        let result = step_articulated(&mut sys, 0.01);
        let json = articulated_result_to_json(&result);
        assert!(json.contains("kinetic_energy"));
        assert!(json.contains("potential_energy"));
    }

    #[test]
    fn max_links_respected() {
        let cfg = ArticulatedConfig {
            gravity: [0.0, -9.81, 0.0],
            damping: 0.0,
            max_links: 2,
        };
        let mut sys = new_articulated_system(cfg);
        for _ in 0..5 {
            add_link(&mut sys, new_body_link("x", 1.0, None));
        }
        assert_eq!(link_count(&sys), 2);
    }
}
