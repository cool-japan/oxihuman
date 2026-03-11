//! Spring bone system for secondary motion (hair, tails, accessories).
//!
//! Implements a Verlet-integration based spring bone chain with stiffness,
//! damping, wind influence, and constraint solving.

/// Configuration for spring bone physics.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpringBoneConfig {
    /// Spring stiffness coefficient (higher = stiffer).
    pub stiffness: f32,
    /// Damping coefficient (0 = no damping, 1 = critically damped).
    pub damping: f32,
    /// Gravity acceleration applied to each bone.
    pub gravity: f32,
    /// Maximum bone length stretch factor before constraint kicks in.
    pub length_tolerance: f32,
}

/// A single spring bone in a chain.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpringBone {
    /// Current world-space position.
    pub position: [f32; 3],
    /// Position from the previous simulation step (Verlet).
    pub prev_position: [f32; 3],
    /// Rest length to the parent bone.
    pub rest_length: f32,
    /// Per-bone stiffness override (if `None`, use chain config).
    pub stiffness_override: Option<f32>,
    /// Per-bone damping override (if `None`, use chain config).
    pub damping_override: Option<f32>,
    /// Whether this bone is pinned (does not move).
    pub pinned: bool,
}

/// A chain of spring bones (e.g. a hair strand or tail).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpringBoneChain {
    /// Ordered list of bones from root to tip.
    pub bones: Vec<SpringBone>,
    /// Physics configuration for this chain.
    pub config: SpringBoneConfig,
}

/// Returns a default `SpringBoneConfig`.
#[allow(dead_code)]
pub fn default_spring_bone_config() -> SpringBoneConfig {
    SpringBoneConfig {
        stiffness: 0.8,
        damping: 0.05,
        gravity: 9.81,
        length_tolerance: 0.01,
    }
}

/// Creates a new `SpringBoneChain` with no bones.
#[allow(dead_code)]
pub fn new_spring_bone_chain(config: SpringBoneConfig) -> SpringBoneChain {
    SpringBoneChain { bones: Vec::new(), config }
}

/// Appends a bone to the end of the chain at `position` with `rest_length`.
#[allow(dead_code)]
pub fn add_bone_to_chain(chain: &mut SpringBoneChain, position: [f32; 3], rest_length: f32) {
    chain.bones.push(SpringBone {
        position,
        prev_position: position,
        rest_length,
        stiffness_override: None,
        damping_override: None,
        pinned: false,
    });
}

/// Advances the spring bone chain by one physics step (`dt` seconds).
///
/// Uses Verlet integration with length constraints.
#[allow(dead_code)]
pub fn update_spring_bones(chain: &mut SpringBoneChain, dt: f32) {
    let cfg = chain.config.clone();
    let bone_count = chain.bones.len();

    // Verlet integration for each non-pinned bone
    for i in 0..bone_count {
        if chain.bones[i].pinned {
            continue;
        }
        let stiffness = chain.bones[i].stiffness_override.unwrap_or(cfg.stiffness);
        let damping = chain.bones[i].damping_override.unwrap_or(cfg.damping);

        let pos = chain.bones[i].position;
        let prev = chain.bones[i].prev_position;

        // Velocity estimate
        let vx = (pos[0] - prev[0]) * (1.0 - damping);
        let vy = (pos[1] - prev[1]) * (1.0 - damping);
        let vz = (pos[2] - prev[2]) * (1.0 - damping);

        // Compute rest position (for stiffness spring toward parent)
        let rest_pos = if i > 0 {
            chain.bones[i - 1].position
        } else {
            pos
        };

        // Spring force toward rest position
        let spring_x = (rest_pos[0] - pos[0]) * stiffness;
        let spring_y = (rest_pos[1] - pos[1]) * stiffness;
        let spring_z = (rest_pos[2] - pos[2]) * stiffness;

        // Gravity
        let gravity_y = -cfg.gravity * dt * dt;

        let new_x = pos[0] + vx + (spring_x) * dt * dt;
        let new_y = pos[1] + vy + (spring_y + gravity_y) * dt * dt;
        let new_z = pos[2] + vz + (spring_z) * dt * dt;

        chain.bones[i].prev_position = pos;
        chain.bones[i].position = [new_x, new_y, new_z];
    }

    // Length constraint: walk from root to tip
    for i in 1..bone_count {
        let parent_pos = chain.bones[i - 1].position;
        let pos = chain.bones[i].position;
        let rest_len = chain.bones[i].rest_length;

        let dx = pos[0] - parent_pos[0];
        let dy = pos[1] - parent_pos[1];
        let dz = pos[2] - parent_pos[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist > 1e-12 {
            let scale = rest_len / dist;
            chain.bones[i].position = [
                parent_pos[0] + dx * scale,
                parent_pos[1] + dy * scale,
                parent_pos[2] + dz * scale,
            ];
        }
    }
}

/// Returns the number of bones in the chain.
#[allow(dead_code)]
pub fn spring_bone_count(chain: &SpringBoneChain) -> usize {
    chain.bones.len()
}

/// Returns the world-space position of the tip bone (last in chain).
///
/// Returns `None` if the chain is empty.
#[allow(dead_code)]
pub fn chain_tip_position(chain: &SpringBoneChain) -> Option<[f32; 3]> {
    chain.bones.last().map(|b| b.position)
}

/// Sets the stiffness override on bone `index`.
#[allow(dead_code)]
pub fn set_bone_stiffness(chain: &mut SpringBoneChain, index: usize, stiffness: f32) {
    if index < chain.bones.len() {
        chain.bones[index].stiffness_override = Some(stiffness);
    }
}

/// Sets the damping override on bone `index`.
#[allow(dead_code)]
pub fn set_bone_damping(chain: &mut SpringBoneChain, index: usize, damping: f32) {
    if index < chain.bones.len() {
        chain.bones[index].damping_override = Some(damping);
    }
}

/// Applies a wind force vector to every non-pinned bone in the chain.
///
/// `wind` is `[wx, wy, wz]` in world space.
#[allow(dead_code)]
pub fn apply_wind_to_chain(chain: &mut SpringBoneChain, wind: [f32; 3], dt: f32) {
    for bone in chain.bones.iter_mut() {
        if bone.pinned {
            continue;
        }
        let fx = wind[0] * dt * dt;
        let fy = wind[1] * dt * dt;
        let fz = wind[2] * dt * dt;
        bone.position[0] += fx;
        bone.position[1] += fy;
        bone.position[2] += fz;
    }
}

/// Pins the root bone of the chain (index 0) at its current position.
///
/// The root will not move during `update_spring_bones`.
#[allow(dead_code)]
pub fn pin_chain_root(chain: &mut SpringBoneChain) {
    if !chain.bones.is_empty() {
        chain.bones[0].pinned = true;
    }
}

/// Resets all bones to their rest positions (prev = current).
///
/// Call this when the character teleports to prevent velocity artifacts.
#[allow(dead_code)]
pub fn reset_spring_chain(chain: &mut SpringBoneChain) {
    for bone in chain.bones.iter_mut() {
        bone.prev_position = bone.position;
    }
}

/// Computes the total arc length of the chain from root to tip.
#[allow(dead_code)]
pub fn chain_length(chain: &SpringBoneChain) -> f32 {
    if chain.bones.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for i in 1..chain.bones.len() {
        let a = chain.bones[i - 1].position;
        let b = chain.bones[i].position;
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

/// Returns the world-space position of bone at `index`.
///
/// Returns `None` if `index` is out of range.
#[allow(dead_code)]
pub fn bone_world_position(chain: &SpringBoneChain, index: usize) -> Option<[f32; 3]> {
    chain.bones.get(index).map(|b| b.position)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_chain(n: usize) -> SpringBoneChain {
        let cfg = default_spring_bone_config();
        let mut chain = new_spring_bone_chain(cfg);
        for i in 0..n {
            add_bone_to_chain(&mut chain, [0.0, -(i as f32), 0.0], 1.0);
        }
        chain
    }

    #[test]
    fn test_default_spring_bone_config() {
        let cfg = default_spring_bone_config();
        assert!(cfg.stiffness > 0.0);
        assert!(cfg.damping >= 0.0);
    }

    #[test]
    fn test_new_spring_bone_chain_empty() {
        let cfg = default_spring_bone_config();
        let chain = new_spring_bone_chain(cfg);
        assert_eq!(spring_bone_count(&chain), 0);
    }

    #[test]
    fn test_add_bone_to_chain() {
        let mut chain = simple_chain(0);
        add_bone_to_chain(&mut chain, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(spring_bone_count(&chain), 1);
    }

    #[test]
    fn test_chain_tip_position_empty_returns_none() {
        let chain = simple_chain(0);
        assert!(chain_tip_position(&chain).is_none());
    }

    #[test]
    fn test_chain_tip_position_nonempty() {
        let chain = simple_chain(3);
        let tip = chain_tip_position(&chain);
        assert!(tip.is_some());
    }

    #[test]
    fn test_spring_bone_count() {
        let chain = simple_chain(5);
        assert_eq!(spring_bone_count(&chain), 5);
    }

    #[test]
    fn test_pin_chain_root() {
        let mut chain = simple_chain(3);
        pin_chain_root(&mut chain);
        assert!(chain.bones[0].pinned);
    }

    #[test]
    fn test_pin_chain_root_does_not_move() {
        let mut chain = simple_chain(3);
        pin_chain_root(&mut chain);
        let root_pos = chain.bones[0].position;
        update_spring_bones(&mut chain, 0.016);
        assert_eq!(chain.bones[0].position, root_pos);
    }

    #[test]
    fn test_update_spring_bones_does_not_panic() {
        let mut chain = simple_chain(5);
        pin_chain_root(&mut chain);
        update_spring_bones(&mut chain, 0.016);
    }

    #[test]
    fn test_chain_length_two_bones() {
        let mut chain = simple_chain(0);
        add_bone_to_chain(&mut chain, [0.0, 0.0, 0.0], 1.0);
        add_bone_to_chain(&mut chain, [0.0, -1.0, 0.0], 1.0);
        let len = chain_length(&chain);
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_chain_length_single_bone() {
        let chain = simple_chain(1);
        assert!((chain_length(&chain)).abs() < 1e-5);
    }

    #[test]
    fn test_bone_world_position_valid_index() {
        let chain = simple_chain(3);
        let pos = bone_world_position(&chain, 2);
        assert!(pos.is_some());
    }

    #[test]
    fn test_bone_world_position_invalid_index() {
        let chain = simple_chain(3);
        assert!(bone_world_position(&chain, 10).is_none());
    }

    #[test]
    fn test_set_bone_stiffness() {
        let mut chain = simple_chain(3);
        set_bone_stiffness(&mut chain, 1, 0.5);
        assert_eq!(chain.bones[1].stiffness_override, Some(0.5));
    }

    #[test]
    fn test_set_bone_damping() {
        let mut chain = simple_chain(3);
        set_bone_damping(&mut chain, 1, 0.1);
        assert_eq!(chain.bones[1].damping_override, Some(0.1));
    }

    #[test]
    fn test_reset_spring_chain() {
        let mut chain = simple_chain(3);
        // Update a few times to move positions
        pin_chain_root(&mut chain);
        for _ in 0..10 {
            update_spring_bones(&mut chain, 0.016);
        }
        reset_spring_chain(&mut chain);
        // After reset, prev_position should match current position
        for bone in &chain.bones {
            assert_eq!(bone.prev_position, bone.position);
        }
    }

    #[test]
    fn test_apply_wind_to_chain() {
        let mut chain = simple_chain(3);
        pin_chain_root(&mut chain);
        let orig_tip = chain_tip_position(&chain).unwrap();
        apply_wind_to_chain(&mut chain, [1.0, 0.0, 0.0], 0.016);
        let new_tip = chain_tip_position(&chain).unwrap();
        // Tip (non-pinned) should move in wind direction
        assert!(new_tip[0] > orig_tip[0]);
    }

    #[test]
    fn test_apply_wind_does_not_move_pinned_root() {
        let mut chain = simple_chain(3);
        pin_chain_root(&mut chain);
        let root_pos = chain.bones[0].position;
        apply_wind_to_chain(&mut chain, [1.0, 0.0, 0.0], 0.016);
        assert_eq!(chain.bones[0].position, root_pos);
    }
}
