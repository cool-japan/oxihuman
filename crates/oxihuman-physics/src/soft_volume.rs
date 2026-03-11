//! Pressure-based soft volume (balloon physics) using PBD constraints.
//!
//! Models a closed inflatable surface.  An internal pressure term pushes
//! nodes outward when the enclosed volume drops below its rest volume,
//! creating a balloon-like behaviour.

#![allow(dead_code)]

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
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

// ── Public types ──────────────────────────────────────────────────────────────

/// Configuration for the soft volume simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SoftVolumeConfig {
    /// Target pressure (Pa).  Higher values resist compression more.
    pub pressure: f32,
    /// Stiffness of distance constraints between adjacent nodes [0, 1].
    pub stiffness: f32,
    /// Gravity vector.
    pub gravity: [f32; 3],
    /// Velocity damping factor per step.
    pub damping: f32,
    /// Number of constraint iterations per step.
    pub solver_iterations: u32,
}

/// A single node (vertex) of the soft volume.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SoftVolumeNode {
    /// Current position.
    pub position: [f32; 3],
    /// Predicted position (PBD).
    pub predicted: [f32; 3],
    /// Velocity.
    pub velocity: [f32; 3],
    /// Inverse mass (0 = pinned).
    pub inv_mass: f32,
    /// Rest position.
    pub rest_position: [f32; 3],
}

/// Pressure-based soft volume.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SoftVolume {
    /// Nodes forming the volume surface.
    pub nodes: Vec<SoftVolumeNode>,
    /// Triangle index buffer (triples) over nodes.
    pub indices: Vec<u32>,
    /// Configuration.
    pub config: SoftVolumeConfig,
    /// Rest volume (computed at construction from rest positions).
    pub rest_volume: f32,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a default `SoftVolumeConfig`.
#[allow(dead_code)]
pub fn default_soft_volume_config() -> SoftVolumeConfig {
    SoftVolumeConfig {
        pressure: 1.0,
        stiffness: 0.5,
        gravity: [0.0, -9.81, 0.0],
        damping: 0.02,
        solver_iterations: 10,
    }
}

/// Create a new empty soft volume from a config.
#[allow(dead_code)]
pub fn new_soft_volume(config: SoftVolumeConfig, indices: Vec<u32>) -> SoftVolume {
    SoftVolume { nodes: Vec::new(), indices, config, rest_volume: 0.0 }
}

/// Add a node to the volume and return its index.
#[allow(dead_code)]
pub fn soft_volume_add_node(volume: &mut SoftVolume, position: [f32; 3], mass: f32) -> usize {
    let inv_mass = if mass > 1e-10 { 1.0 / mass } else { 0.0 };
    let idx = volume.nodes.len();
    volume.nodes.push(SoftVolumeNode {
        position,
        predicted: position,
        velocity: [0.0; 3],
        inv_mass,
        rest_position: position,
    });
    // Recompute rest volume
    volume.rest_volume = compute_signed_volume(&volume.nodes, &volume.indices).abs();
    idx
}

/// Advance the soft volume simulation by one time step `dt`.
#[allow(dead_code)]
pub fn soft_volume_step(volume: &mut SoftVolume, dt: f32) {
    let n = volume.nodes.len();
    if n == 0 { return; }

    let g = volume.config.gravity;
    let damp = (1.0 - volume.config.damping).clamp(0.0, 1.0);

    // Integrate velocities
    for node in volume.nodes.iter_mut() {
        if node.inv_mass < 1e-10 { node.velocity = [0.0; 3]; continue; }
        node.velocity = add3(node.velocity, scale3(g, dt));
        node.velocity = scale3(node.velocity, damp);
        node.predicted = add3(node.position, scale3(node.velocity, dt));
    }

    // Pressure constraint: push nodes outward when volume is below rest
    let cur_vol = compute_signed_volume(&volume.nodes, &volume.indices).abs();
    let vol_ratio = if volume.rest_volume > 1e-10 {
        volume.config.pressure * (1.0 - cur_vol / volume.rest_volume)
    } else {
        0.0
    };

    let tri_count = volume.indices.len() / 3;
    for _ in 0..volume.config.solver_iterations {
        for t in 0..tri_count {
            let ia = volume.indices[t * 3] as usize;
            let ib = volume.indices[t * 3 + 1] as usize;
            let ic = volume.indices[t * 3 + 2] as usize;
            if ia >= n || ib >= n || ic >= n { continue; }

            let pa = volume.nodes[ia].predicted;
            let pb = volume.nodes[ib].predicted;
            let pc = volume.nodes[ic].predicted;
            let normal = cross3(sub3(pb, pa), sub3(pc, pa));
            let normal_len = len3(normal);
            if normal_len < 1e-10 { continue; }
            let n_unit = scale3(normal, 1.0 / normal_len);

            let delta = scale3(n_unit, vol_ratio * 0.1 * dt);

            let wa = volume.nodes[ia].inv_mass;
            let wb = volume.nodes[ib].inv_mass;
            let wc = volume.nodes[ic].inv_mass;

            if wa > 1e-10 {
                volume.nodes[ia].predicted = add3(pa, scale3(delta, wa));
            }
            if wb > 1e-10 {
                volume.nodes[ib].predicted = add3(pb, scale3(delta, wb));
            }
            if wc > 1e-10 {
                volume.nodes[ic].predicted = add3(pc, scale3(delta, wc));
            }
        }
    }

    // Commit positions
    let inv_dt = 1.0 / dt.max(1e-6);
    for node in volume.nodes.iter_mut() {
        if node.inv_mass < 1e-10 { continue; }
        node.velocity = scale3(sub3(node.predicted, node.position), inv_dt);
        node.position = node.predicted;
    }
}

/// Return the configured pressure.
#[allow(dead_code)]
pub fn soft_volume_pressure(volume: &SoftVolume) -> f32 {
    volume.config.pressure
}

/// Return the rest volume.
#[allow(dead_code)]
pub fn soft_volume_rest_volume(volume: &SoftVolume) -> f32 {
    volume.rest_volume
}

/// Compute the current signed volume enclosed by the surface mesh.
#[allow(dead_code)]
pub fn soft_volume_current_volume(volume: &SoftVolume) -> f32 {
    compute_signed_volume(&volume.nodes, &volume.indices).abs()
}

/// Return the number of nodes.
#[allow(dead_code)]
pub fn soft_volume_node_count(volume: &SoftVolume) -> usize {
    volume.nodes.len()
}

/// Serialise the volume to a JSON string.
#[allow(dead_code)]
pub fn soft_volume_to_json(volume: &SoftVolume) -> String {
    format!(
        "{{\"node_count\":{},\"pressure\":{},\"rest_volume\":{}}}",
        volume.nodes.len(),
        volume.config.pressure,
        volume.rest_volume,
    )
}

/// Reset all node velocities to zero.
#[allow(dead_code)]
pub fn soft_volume_reset(volume: &mut SoftVolume) {
    for node in volume.nodes.iter_mut() {
        node.velocity = [0.0; 3];
        node.predicted = node.position;
    }
}

// ── Internal ──────────────────────────────────────────────────────────────────

/// Signed volume of a closed triangle mesh using the divergence theorem.
fn compute_signed_volume(nodes: &[SoftVolumeNode], indices: &[u32]) -> f32 {
    let mut vol = 0.0f32;
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        if ia >= nodes.len() || ib >= nodes.len() || ic >= nodes.len() { continue; }
        let a = nodes[ia].position;
        let b = nodes[ib].position;
        let c = nodes[ic].position;
        vol += dot3(a, cross3(b, c));
    }
    vol / 6.0
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tetrahedron_volume() -> SoftVolume {
        let cfg = default_soft_volume_config();
        // Simple tetrahedron indices (4 faces)
        let indices = vec![
            0u32, 1, 2,
            0, 3, 1,
            0, 2, 3,
            1, 3, 2,
        ];
        let mut vol = new_soft_volume(cfg, indices);
        soft_volume_add_node(&mut vol, [0.0, 0.0, 0.0], 1.0);
        soft_volume_add_node(&mut vol, [1.0, 0.0, 0.0], 1.0);
        soft_volume_add_node(&mut vol, [0.0, 1.0, 0.0], 1.0);
        soft_volume_add_node(&mut vol, [0.0, 0.0, 1.0], 1.0);
        vol
    }

    #[test]
    fn test_default_config_pressure_positive() {
        let cfg = default_soft_volume_config();
        assert!(cfg.pressure > 0.0);
    }

    #[test]
    fn test_new_volume_starts_empty() {
        let cfg = default_soft_volume_config();
        let vol = new_soft_volume(cfg, vec![]);
        assert_eq!(soft_volume_node_count(&vol), 0);
    }

    #[test]
    fn test_add_node_increases_count() {
        let cfg = default_soft_volume_config();
        let mut vol = new_soft_volume(cfg, vec![]);
        soft_volume_add_node(&mut vol, [0.0; 3], 1.0);
        assert_eq!(soft_volume_node_count(&vol), 1);
    }

    #[test]
    fn test_rest_volume_positive_for_tetrahedron() {
        let vol = tetrahedron_volume();
        assert!(soft_volume_rest_volume(&vol) > 0.0);
    }

    #[test]
    fn test_current_volume_finite() {
        let vol = tetrahedron_volume();
        let v = soft_volume_current_volume(&vol);
        assert!(v.is_finite());
    }

    #[test]
    fn test_pressure_accessor() {
        let cfg = SoftVolumeConfig { pressure: 3.0, ..default_soft_volume_config() };
        let vol = new_soft_volume(cfg, vec![]);
        assert!((soft_volume_pressure(&vol) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_step_does_not_panic() {
        let mut vol = tetrahedron_volume();
        soft_volume_step(&mut vol, 0.01);
    }

    #[test]
    fn test_reset_zeros_velocity() {
        let mut vol = tetrahedron_volume();
        vol.nodes[0].velocity = [1.0, 2.0, 3.0];
        soft_volume_reset(&mut vol);
        for node in &vol.nodes {
            let v2 = dot3(node.velocity, node.velocity);
            assert!(v2 < 1e-10);
        }
    }

    #[test]
    fn test_to_json_contains_node_count() {
        let vol = tetrahedron_volume();
        let json = soft_volume_to_json(&vol);
        assert!(json.contains("node_count"));
        assert!(json.contains("pressure"));
    }

    #[test]
    fn test_step_moves_free_nodes() {
        let mut vol = tetrahedron_volume();
        let pos_before = vol.nodes[0].position;
        for _ in 0..5 {
            soft_volume_step(&mut vol, 0.01);
        }
        let pos_after = vol.nodes[0].position;
        let diff = sub3(pos_after, pos_before);
        let moved = (diff[0]*diff[0]+diff[1]*diff[1]+diff[2]*diff[2]).sqrt();
        assert!(moved > 0.0);
    }

    #[test]
    fn test_pinned_node_stays_fixed() {
        let mut vol = tetrahedron_volume();
        vol.nodes[0].inv_mass = 0.0; // pin it
        let pos_before = vol.nodes[0].position;
        for _ in 0..10 {
            soft_volume_step(&mut vol, 0.01);
        }
        let pos_after = vol.nodes[0].position;
        let diff = sub3(pos_after, pos_before);
        let moved = (diff[0]*diff[0]+diff[1]*diff[1]+diff[2]*diff[2]).sqrt();
        assert!(moved < 1e-6);
    }
}
