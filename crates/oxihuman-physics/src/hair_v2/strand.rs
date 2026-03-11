// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Hair strand representation as a discrete Cosserat rod.
//!
//! Each strand consists of nodes linked by edges. Every node stores a position
//! (center-line sample) and a material frame orientation encoded as a unit
//! quaternion. The orientation couples twist and bending along the strand.

// ── Quaternion helpers (f64, unit) ──────────────────────────────────────────

/// Identity quaternion `[x, y, z, w]`.
pub(crate) const QUAT_IDENTITY: [f64; 4] = [0.0, 0.0, 0.0, 1.0];

/// Quaternion multiply `a * b` (Hamilton product), layout `[x, y, z, w]`.
#[inline]
pub(crate) fn quat_mul(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

/// Quaternion conjugate (inverse for unit quaternions).
#[inline]
pub(crate) fn quat_conj(q: [f64; 4]) -> [f64; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

/// Normalize quaternion, returning identity if magnitude is near zero.
#[inline]
pub(crate) fn quat_normalize(q: [f64; 4]) -> [f64; 4] {
    let len_sq = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
    if len_sq < 1e-30 {
        return QUAT_IDENTITY;
    }
    let inv = 1.0 / len_sq.sqrt();
    [q[0] * inv, q[1] * inv, q[2] * inv, q[3] * inv]
}

/// Rotate a vector `v` by unit quaternion `q`: `q * (0,v) * q*`.
#[inline]
pub(crate) fn quat_rotate(q: [f64; 4], v: [f64; 3]) -> [f64; 3] {
    let qv = [v[0], v[1], v[2], 0.0];
    let r = quat_mul(quat_mul(q, qv), quat_conj(q));
    [r[0], r[1], r[2]]
}

/// Build a quaternion that rotates `from` direction to `to` direction.
/// Both should be roughly unit-length. Falls back to identity on degenerate
/// inputs.
pub(crate) fn quat_from_two_vectors(from: [f64; 3], to: [f64; 3]) -> [f64; 4] {
    let dot = from[0] * to[0] + from[1] * to[1] + from[2] * to[2];
    let cross = [
        from[1] * to[2] - from[2] * to[1],
        from[2] * to[0] - from[0] * to[2],
        from[0] * to[1] - from[1] * to[0],
    ];
    let w = 1.0 + dot;
    if w < 1e-12 {
        // Nearly opposite vectors — pick an arbitrary perpendicular axis.
        let perp = if from[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let axis = [
            from[1] * perp[2] - from[2] * perp[1],
            from[2] * perp[0] - from[0] * perp[2],
            from[0] * perp[1] - from[1] * perp[0],
        ];
        return quat_normalize([axis[0], axis[1], axis[2], 0.0]);
    }
    quat_normalize([cross[0], cross[1], cross[2], w])
}

/// Extract the "twist" angle around the local tangent (z-axis of the frame)
/// from a relative quaternion `dq = q_conj(q_a) * q_b`.
/// Returns the signed twist angle in radians.
pub(crate) fn extract_twist_angle(dq: [f64; 4]) -> f64 {
    // Decompose dq into swing * twist around z-axis.
    // twist component: keep only z and w, renormalize.
    let tz = dq[2];
    let tw = dq[3];
    let len = (tz * tz + tw * tw).sqrt();
    if len < 1e-15 {
        return 0.0;
    }
    2.0 * (tz / len).asin()
}

/// Extract the bend (swing) axis-angle from a relative quaternion.
/// Returns `[bend_x, bend_y, bend_z]` where `bend_z` is negligible
/// (the twist component has been factored out).
pub(crate) fn extract_bend_angles(dq: [f64; 4]) -> [f64; 3] {
    // Factor out twist around z to get swing.
    let tz = dq[2];
    let tw = dq[3];
    let len = (tz * tz + tw * tw).sqrt();
    let (twist_z, twist_w) = if len > 1e-15 {
        (tz / len, tw / len)
    } else {
        (0.0, 1.0)
    };
    // swing = dq * conj(twist)
    let twist_conj = [-twist_z, 0.0, 0.0, twist_w]; // twist is around z only
    // Actually twist quat is [0, 0, twist_z, twist_w], its conjugate:
    let tc = [0.0, 0.0, -twist_z, twist_w];
    let swing = quat_mul(dq, tc);
    // Convert swing to axis-angle vector (small-angle approx for swing).
    let half_angle = (swing[0] * swing[0] + swing[1] * swing[1] + swing[2] * swing[2]).sqrt();
    if half_angle < 1e-15 {
        return [0.0, 0.0, 0.0];
    }
    let angle = 2.0 * half_angle.atan2(swing[3]);
    let inv_ha = angle / half_angle;
    [swing[0] * inv_ha, swing[1] * inv_ha, swing[2] * inv_ha]
}

// ── Vector helpers (f64) ────────────────────────────────────────────────────

#[inline]
pub(crate) fn v3_add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
pub(crate) fn v3_sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
pub(crate) fn v3_scale(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
pub(crate) fn v3_dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
pub(crate) fn v3_cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
pub(crate) fn v3_length(a: [f64; 3]) -> f64 {
    v3_dot(a, a).sqrt()
}

#[inline]
pub(crate) fn v3_normalize(a: [f64; 3]) -> [f64; 3] {
    let l = v3_length(a);
    if l < 1e-30 {
        return [0.0, 0.0, 0.0];
    }
    v3_scale(a, 1.0 / l)
}

#[inline]
pub(crate) fn v3_lerp(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

// ── Core types ──────────────────────────────────────────────────────────────

/// A single node on the Cosserat rod center-line.
#[derive(Debug, Clone)]
pub struct HairNode {
    /// World-space position of this node.
    pub position: [f64; 3],
    /// Material-frame orientation as unit quaternion `[x, y, z, w]`.
    pub orientation: [f64; 4],
    /// Linear velocity.
    pub velocity: [f64; 3],
    /// Angular velocity in body frame.
    pub angular_velocity: [f64; 3],
    /// Inverse mass (0.0 for pinned / root nodes).
    pub inv_mass: f64,
    /// Inverse rotational inertia (scalar approximation for thin rod).
    pub inv_inertia: f64,
}

impl HairNode {
    /// Create a new node at the given position with identity orientation.
    pub fn new(position: [f64; 3], mass: f64) -> Self {
        let inv_mass = if mass > 1e-30 { 1.0 / mass } else { 0.0 };
        // Thin rod inertia approximation: I ~ m * L^2 / 12, but per-node we
        // just store a scalar. Caller can override.
        let inv_inertia = inv_mass; // simplified
        Self {
            position,
            orientation: QUAT_IDENTITY,
            velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            inv_mass,
            inv_inertia,
        }
    }

    /// Create a pinned (fixed) node — zero inverse mass.
    pub fn new_pinned(position: [f64; 3]) -> Self {
        Self {
            position,
            orientation: QUAT_IDENTITY,
            velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            inv_mass: 0.0,
            inv_inertia: 0.0,
        }
    }
}

/// A single hair strand modeled as a discrete Cosserat rod.
#[derive(Debug, Clone)]
pub struct HairStrand {
    /// Nodes along the center-line (index 0 = root, pinned).
    pub nodes: Vec<HairNode>,
    /// Rest lengths between consecutive nodes.
    pub rest_lengths: Vec<f64>,
    /// Rest curvature vectors (Darboux vector) per edge.
    /// `rest_curvatures[i]` encodes the rest bend-twist between node i and i+1.
    pub rest_curvatures: Vec<[f64; 3]>,
    /// Rest orientations — stored for shape matching.
    rest_orientations: Vec<[f64; 4]>,
    /// Rest positions — stored for shape matching.
    rest_positions: Vec<[f64; 3]>,
}

impl HairStrand {
    /// Build a strand from root along a direction.
    ///
    /// `root` — pinned root position.
    /// `direction` — approximate tangent direction (will be normalized).
    /// `total_length` — total strand length in world units.
    /// `segments` — number of segments (nodes = segments + 1).
    /// `mass_per_unit_length` — linear density.
    pub fn new(
        root: [f64; 3],
        direction: [f64; 3],
        total_length: f64,
        segments: usize,
        mass_per_unit_length: f64,
    ) -> Self {
        let segments = segments.max(1);
        let seg_len = total_length / segments as f64;
        let dir = v3_normalize(direction);
        let seg_mass = mass_per_unit_length * seg_len;

        // Build reference frame from direction.
        let ref_quat = orientation_from_tangent(dir);

        let mut nodes = Vec::with_capacity(segments + 1);
        // Root is pinned.
        let mut root_node = HairNode::new_pinned(root);
        root_node.orientation = ref_quat;
        nodes.push(root_node);

        for i in 1..=segments {
            let pos = v3_add(root, v3_scale(dir, seg_len * i as f64));
            let mut node = HairNode::new(pos, seg_mass);
            node.orientation = ref_quat;
            nodes.push(node);
        }

        let rest_lengths = vec![seg_len; segments];
        // Straight rest pose => zero Darboux vector.
        let rest_curvatures = vec![[0.0; 3]; segments];
        let rest_orientations: Vec<_> = nodes.iter().map(|n| n.orientation).collect();
        let rest_positions: Vec<_> = nodes.iter().map(|n| n.position).collect();

        Self {
            nodes,
            rest_lengths,
            rest_curvatures,
            rest_orientations,
            rest_positions,
        }
    }

    /// Build a strand from an explicit chain of positions.
    ///
    /// The first node is pinned (root). Orientations are computed from
    /// consecutive tangent vectors. `mass_per_unit_length` sets the mass
    /// of each node proportionally to the adjacent segment lengths.
    pub fn from_positions(positions: &[[f64; 3]], mass_per_unit_length: f64) -> anyhow::Result<Self> {
        if positions.len() < 2 {
            anyhow::bail!("HairStrand requires at least 2 positions");
        }

        let n = positions.len();
        let segments = n - 1;
        let mut rest_lengths = Vec::with_capacity(segments);
        let mut nodes = Vec::with_capacity(n);

        // Root (pinned).
        let mut root_node = HairNode::new_pinned(positions[0]);
        // Compute tangent for root orientation.
        let first_tangent = v3_normalize(v3_sub(positions[1], positions[0]));
        root_node.orientation = orientation_from_tangent(first_tangent);
        nodes.push(root_node);

        for i in 1..n {
            let seg_vec = v3_sub(positions[i], positions[i - 1]);
            let seg_len = v3_length(seg_vec);
            rest_lengths.push(seg_len);

            let tangent = if seg_len > 1e-15 {
                v3_scale(seg_vec, 1.0 / seg_len)
            } else {
                first_tangent
            };
            let mass = mass_per_unit_length * seg_len;
            let mut node = HairNode::new(positions[i], mass);
            node.orientation = orientation_from_tangent(tangent);
            nodes.push(node);
        }

        // Compute rest curvatures (Darboux vectors) from consecutive orientations.
        let mut rest_curvatures = Vec::with_capacity(segments);
        for i in 0..segments {
            let dq = quat_mul(quat_conj(nodes[i].orientation), nodes[i + 1].orientation);
            let bend = extract_bend_angles(dq);
            rest_curvatures.push(bend);
        }

        let rest_orientations: Vec<_> = nodes.iter().map(|n| n.orientation).collect();
        let rest_positions: Vec<_> = nodes.iter().map(|n| n.position).collect();

        Ok(Self {
            nodes,
            rest_lengths,
            rest_curvatures,
            rest_orientations,
            rest_positions,
        })
    }

    /// Number of nodes (particles).
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges (segments).
    #[inline]
    pub fn segment_count(&self) -> usize {
        if self.nodes.len() > 1 {
            self.nodes.len() - 1
        } else {
            0
        }
    }

    /// Position of the tip node.
    pub fn tip_position(&self) -> [f64; 3] {
        self.nodes
            .last()
            .map(|n| n.position)
            .unwrap_or([0.0; 3])
    }

    /// Position of the root node.
    pub fn root_position(&self) -> [f64; 3] {
        self.nodes
            .first()
            .map(|n| n.position)
            .unwrap_or([0.0; 3])
    }

    /// Compute the center of mass of the strand.
    pub fn center_of_mass(&self) -> [f64; 3] {
        if self.nodes.is_empty() {
            return [0.0; 3];
        }
        let mut com = [0.0; 3];
        let mut total_mass = 0.0_f64;
        for node in &self.nodes {
            let m = if node.inv_mass > 1e-30 {
                1.0 / node.inv_mass
            } else {
                1e6 // treat pinned nodes as very heavy for COM purposes
            };
            com = v3_add(com, v3_scale(node.position, m));
            total_mass += m;
        }
        if total_mass > 1e-30 {
            v3_scale(com, 1.0 / total_mass)
        } else {
            com
        }
    }

    /// Get rest position for node `i`.
    pub fn rest_position(&self, i: usize) -> Option<[f64; 3]> {
        self.rest_positions.get(i).copied()
    }

    /// Get rest orientation for node `i`.
    pub fn rest_orientation(&self, i: usize) -> Option<[f64; 4]> {
        self.rest_orientations.get(i).copied()
    }

    /// Total strand length (sum of rest lengths).
    pub fn total_rest_length(&self) -> f64 {
        self.rest_lengths.iter().sum()
    }

    /// Update rest pose from current configuration (e.g. after grooming).
    pub fn snapshot_rest_pose(&mut self) {
        self.rest_positions = self.nodes.iter().map(|n| n.position).collect();
        self.rest_orientations = self.nodes.iter().map(|n| n.orientation).collect();
        // Recompute rest curvatures.
        for i in 0..self.segment_count() {
            let dq = quat_mul(
                quat_conj(self.nodes[i].orientation),
                self.nodes[i + 1].orientation,
            );
            self.rest_curvatures[i] = extract_bend_angles(dq);
        }
    }
}

/// Build an orientation quaternion whose local +Z axis aligns with `tangent`.
fn orientation_from_tangent(tangent: [f64; 3]) -> [f64; 4] {
    // Default forward is +Z = [0,0,1].
    quat_from_two_vectors([0.0, 0.0, 1.0], tangent)
}

// ── Configuration ───────────────────────────────────────────────────────────

/// Configuration parameters for the v2 hair system.
#[derive(Debug, Clone)]
pub struct HairConfigV2 {
    /// Time step in seconds.
    pub dt: f64,
    /// Gravity vector (m/s^2).
    pub gravity: [f64; 3],
    /// Number of XPBD solver iterations per step.
    pub iterations: usize,
    /// Stretch-twist constraint stiffness (0..1, XPBD compliance inverse).
    pub stretch_stiffness: f64,
    /// Bend-twist constraint stiffness.
    pub bend_stiffness: f64,
    /// Twist constraint stiffness.
    pub twist_stiffness: f64,
    /// Velocity damping factor (0 = no damping, 1 = full damping).
    pub damping: f64,
    /// Shape-matching stiffness (0 = off, 1 = full).
    pub shape_matching_stiffness: f64,
}

impl Default for HairConfigV2 {
    fn default() -> Self {
        Self {
            dt: 1.0 / 60.0,
            gravity: [0.0, -9.81, 0.0],
            iterations: 10,
            stretch_stiffness: 1.0,
            bend_stiffness: 0.5,
            twist_stiffness: 0.3,
            damping: 0.02,
            shape_matching_stiffness: 0.1,
        }
    }
}

// ── Hair system ─────────────────────────────────────────────────────────────

/// Container for multiple hair strands sharing a common configuration.
#[derive(Debug, Clone)]
pub struct HairSystemV2 {
    /// All strands in the system.
    pub strands: Vec<HairStrand>,
    /// Shared simulation configuration.
    pub config: HairConfigV2,
}

impl HairSystemV2 {
    /// Create a new hair system with default config.
    pub fn new() -> Self {
        Self {
            strands: Vec::new(),
            config: HairConfigV2::default(),
        }
    }

    /// Create with a specific configuration.
    pub fn with_config(config: HairConfigV2) -> Self {
        Self {
            strands: Vec::new(),
            config,
        }
    }

    /// Add a strand to the system.
    pub fn add_strand(&mut self, strand: HairStrand) {
        self.strands.push(strand);
    }

    /// Total number of nodes across all strands.
    pub fn total_node_count(&self) -> usize {
        self.strands.iter().map(|s| s.node_count()).sum()
    }

    /// Total number of segments across all strands.
    pub fn total_segment_count(&self) -> usize {
        self.strands.iter().map(|s| s.segment_count()).sum()
    }
}

impl Default for HairSystemV2 {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strand_creation() {
        let strand = HairStrand::new(
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            0.3,
            10,
            0.001,
        );
        assert_eq!(strand.node_count(), 11);
        assert_eq!(strand.segment_count(), 10);
        assert!((strand.total_rest_length() - 0.3).abs() < 1e-10);
        // Root is pinned.
        assert!(strand.nodes[0].inv_mass < 1e-20);
        // Other nodes have mass.
        assert!(strand.nodes[1].inv_mass > 0.0);
    }

    #[test]
    fn test_strand_from_positions() {
        let positions = vec![
            [0.0, 1.0, 0.0],
            [0.0, 0.8, 0.0],
            [0.0, 0.6, 0.0],
            [0.0, 0.4, 0.0],
        ];
        let strand = HairStrand::from_positions(&positions, 0.001).unwrap();
        assert_eq!(strand.node_count(), 4);
        assert_eq!(strand.segment_count(), 3);
    }

    #[test]
    fn test_quat_normalize() {
        let q = quat_normalize([1.0, 2.0, 3.0, 4.0]);
        let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        assert!((len - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_quat_rotate_identity() {
        let v = [1.0, 2.0, 3.0];
        let r = quat_rotate(QUAT_IDENTITY, v);
        for i in 0..3 {
            assert!((r[i] - v[i]).abs() < 1e-12);
        }
    }

    #[test]
    fn test_system_default() {
        let sys = HairSystemV2::new();
        assert_eq!(sys.total_node_count(), 0);
    }

    #[test]
    fn test_snapshot_rest_pose() {
        let mut strand = HairStrand::new(
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            0.3,
            5,
            0.001,
        );
        // Move a node.
        strand.nodes[3].position = [0.1, 0.5, 0.0];
        strand.snapshot_rest_pose();
        let rp = strand.rest_position(3).unwrap();
        assert!((rp[0] - 0.1).abs() < 1e-12);
    }
}
