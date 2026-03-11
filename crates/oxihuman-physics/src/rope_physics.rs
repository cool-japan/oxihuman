//! Rope/chain simulation with position-based dynamics and length constraints.
//!
//! A rope is represented as a chain of nodes.  The root node can be pinned.
//! Each step applies gravity (Verlet integration) followed by iterative
//! distance-constraint projection to maintain segment lengths.

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

/// Configuration for a rope simulation.
#[allow(dead_code)]
pub struct RopeConfig {
    /// Damping factor applied per time step (0..1).
    pub damping: f32,
    /// Gravity scale multiplier.
    pub gravity_scale: f32,
    /// Whether the root node is fixed (pinned).
    pub fixed_root: bool,
}

/// A single node in the rope chain.
#[allow(dead_code)]
#[derive(Clone)]
pub struct RopeNode {
    /// Current world position.
    pub position: [f32; 3],
    /// Previous position used for Verlet integration.
    pub prev_position: [f32; 3],
    /// Rest length from this node to the next.
    pub rest_length: f32,
    /// If true, this node does not move.
    pub pinned: bool,
}

/// A full rope system.
#[allow(dead_code)]
pub struct RopeSystem {
    pub nodes: Vec<RopeNode>,
    pub cfg: RopeConfig,
}

// ---------------------------------------------------------------------------
// Default config
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn default_rope_config() -> RopeConfig {
    RopeConfig {
        damping: 0.99,
        gravity_scale: 1.0,
        fixed_root: true,
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

// ---------------------------------------------------------------------------
// Core functions
// ---------------------------------------------------------------------------

/// Create a new rope starting at `root`, hanging downward, with `segments` segments.
#[allow(dead_code)]
pub fn new_rope(
    root: [f32; 3],
    length: f32,
    segments: u32,
    cfg: &RopeConfig,
) -> RopeSystem {
    let count = (segments + 1).max(2) as usize;
    let seg_len = length / (count - 1) as f32;

    let mut nodes: Vec<RopeNode> = Vec::with_capacity(count);
    for i in 0..count {
        let y_offset = -(i as f32 * seg_len);
        let pos = [root[0], root[1] + y_offset, root[2]];
        nodes.push(RopeNode {
            position: pos,
            prev_position: pos,
            rest_length: seg_len,
            pinned: i == 0 && cfg.fixed_root,
        });
    }

    RopeSystem {
        nodes,
        cfg: RopeConfig {
            damping: cfg.damping,
            gravity_scale: cfg.gravity_scale,
            fixed_root: cfg.fixed_root,
        },
    }
}

/// Advance the rope simulation by one time step using Verlet integration.
#[allow(dead_code)]
pub fn step_rope(rope: &mut RopeSystem, dt: f32, gravity: [f32; 3]) {
    let dt2 = dt * dt;
    let damping = rope.cfg.damping;
    let grav_scale = rope.cfg.gravity_scale;
    let gravity_scaled = scale3(gravity, grav_scale);

    for node in &mut rope.nodes {
        if node.pinned {
            node.prev_position = node.position;
            continue;
        }
        let vel = sub3(node.position, node.prev_position);
        let vel_damped = scale3(vel, damping);
        let accel = scale3(gravity_scaled, dt2);
        let new_pos = add3(node.position, add3(vel_damped, accel));
        node.prev_position = node.position;
        node.position = new_pos;
    }
}

/// Project distance constraints to maintain segment lengths.
/// Call multiple times per step (typically 4-16 iterations for stability).
#[allow(dead_code)]
pub fn apply_rope_constraints(rope: &mut RopeSystem, iterations: u32) {
    let n = rope.nodes.len();
    if n < 2 {
        return;
    }
    for _ in 0..iterations {
        for i in 0..n - 1 {
            let rest = rope.nodes[i].rest_length;
            let pa = rope.nodes[i].position;
            let pb = rope.nodes[i + 1].position;
            let delta = sub3(pb, pa);
            let cur_len = len3(delta);
            if cur_len < 1e-10 {
                continue;
            }
            let diff = (cur_len - rest) / cur_len;
            let correction = scale3(delta, 0.5 * diff);

            let a_pinned = rope.nodes[i].pinned;
            let b_pinned = rope.nodes[i + 1].pinned;

            if !a_pinned && !b_pinned {
                rope.nodes[i].position = add3(pa, correction);
                rope.nodes[i + 1].position = sub3(pb, correction);
            } else if !a_pinned {
                rope.nodes[i].position = add3(pa, scale3(correction, 2.0));
            } else if !b_pinned {
                rope.nodes[i + 1].position = sub3(pb, scale3(correction, 2.0));
            }
        }
    }
}

/// Return the position of the last (tip) node.
#[allow(dead_code)]
pub fn rope_tip(rope: &RopeSystem) -> [f32; 3] {
    rope.nodes
        .last()
        .map(|n| n.position)
        .unwrap_or([0.0; 3])
}

/// Compute the total current length (sum of inter-node distances).
#[allow(dead_code)]
pub fn rope_total_length(rope: &RopeSystem) -> f32 {
    rope.nodes
        .windows(2)
        .map(|w| len3(sub3(w[1].position, w[0].position)))
        .sum()
}

/// Reposition (and re-pin) the first node of the rope.
#[allow(dead_code)]
pub fn set_rope_fixed_end(rope: &mut RopeSystem, pos: [f32; 3]) {
    if let Some(first) = rope.nodes.first_mut() {
        first.position = pos;
        first.prev_position = pos;
        first.pinned = true;
    }
}

/// Return the number of segments (= node count - 1).
#[allow(dead_code)]
pub fn rope_segment_count(rope: &RopeSystem) -> usize {
    rope.nodes.len().saturating_sub(1)
}

/// Pin a specific node to a fixed world position.
#[allow(dead_code)]
pub fn attach_rope_to_point(rope: &mut RopeSystem, node_idx: usize, pos: [f32; 3]) {
    if let Some(node) = rope.nodes.get_mut(node_idx) {
        node.position = pos;
        node.prev_position = pos;
        node.pinned = true;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rope(segments: u32) -> RopeSystem {
        let cfg = default_rope_config();
        new_rope([0.0, 0.0, 0.0], 1.0, segments, &cfg)
    }

    #[test]
    fn test_new_rope_node_count() {
        let rope = make_rope(4);
        assert_eq!(rope.nodes.len(), 5, "4 segments → 5 nodes");
    }

    #[test]
    fn test_root_node_is_pinned() {
        let rope = make_rope(4);
        assert!(rope.nodes[0].pinned, "root must be pinned");
    }

    #[test]
    fn test_rope_segment_count() {
        let rope = make_rope(5);
        assert_eq!(rope_segment_count(&rope), 5);
    }

    #[test]
    fn test_step_rope_root_stays_fixed() {
        let mut rope = make_rope(4);
        let root_before = rope.nodes[0].position;
        for _ in 0..10 {
            step_rope(&mut rope, 0.016, [0.0, -9.8, 0.0]);
            apply_rope_constraints(&mut rope, 8);
        }
        let root_after = rope.nodes[0].position;
        assert_eq!(root_before, root_after, "pinned root must not move");
    }

    #[test]
    fn test_gravity_drops_tip() {
        let mut rope = make_rope(4);
        let tip_y_before = rope_tip(&rope)[1];
        for _ in 0..20 {
            step_rope(&mut rope, 0.016, [0.0, -9.8, 0.0]);
            apply_rope_constraints(&mut rope, 8);
        }
        let tip_y_after = rope_tip(&rope)[1];
        assert!(tip_y_after < tip_y_before, "tip should fall under gravity");
    }

    #[test]
    fn test_rope_total_length_near_rest() {
        let rope = make_rope(4);
        let total = rope_total_length(&rope);
        assert!(
            (total - 1.0).abs() < 1e-4,
            "initial rope length should be ~1.0, got {total}"
        );
    }

    #[test]
    fn test_set_rope_fixed_end() {
        let mut rope = make_rope(4);
        set_rope_fixed_end(&mut rope, [5.0, 5.0, 5.0]);
        assert_eq!(rope.nodes[0].position, [5.0, 5.0, 5.0]);
        assert!(rope.nodes[0].pinned);
    }

    #[test]
    fn test_attach_rope_to_point_pins_node() {
        let mut rope = make_rope(4);
        attach_rope_to_point(&mut rope, 2, [1.0, 2.0, 3.0]);
        assert_eq!(rope.nodes[2].position, [1.0, 2.0, 3.0]);
        assert!(rope.nodes[2].pinned);
    }

    #[test]
    fn test_rope_tip_is_last_node() {
        let rope = make_rope(3);
        let last_pos = rope.nodes.last().unwrap().position;
        assert_eq!(rope_tip(&rope), last_pos);
    }
}
