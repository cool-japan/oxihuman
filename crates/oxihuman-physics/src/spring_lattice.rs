//! 3D lattice of mass-spring nodes for volumetric soft-body deformation.
//!
//! The lattice is a regular 3D grid of nodes connected by Hookean springs
//! to their 6-connected axis-aligned neighbours. Integration uses a
//! semi-implicit Euler step (mass-spring with damping). Nodes can be
//! pinned (zero velocity and infinite mass) to serve as boundary
//! conditions.

#![allow(dead_code)]

/// Configuration for building a spring lattice.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SpringLatticeConfig {
    /// Number of nodes along the X axis.
    pub nx: usize,
    /// Number of nodes along the Y axis.
    pub ny: usize,
    /// Number of nodes along the Z axis.
    pub nz: usize,
    /// Rest-length spacing between adjacent nodes.
    pub spacing: f32,
    /// Spring stiffness coefficient (N/m).
    pub stiffness: f32,
    /// Damping coefficient.
    pub damping: f32,
    /// Mass of each node (kg).
    pub node_mass: f32,
    /// Gravitational acceleration applied per step.
    pub gravity: [f32; 3],
}

/// A single mass node in the lattice.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LatticeNode {
    /// Current position in world space.
    pub position: [f32; 3],
    /// Current velocity.
    pub velocity: [f32; 3],
    /// Whether this node is pinned (immovable).
    pub pinned: bool,
}

/// A spring bond connecting two nodes.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LatticeBond {
    /// Index of the first node.
    pub node_a: usize,
    /// Index of the second node.
    pub node_b: usize,
    /// Rest length of this spring.
    pub rest_length: f32,
}

/// The full spring lattice state.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SpringLattice {
    /// All nodes in index order (x + nx*(y + ny*z)).
    pub nodes: Vec<LatticeNode>,
    /// All spring bonds.
    pub bonds: Vec<LatticeBond>,
    /// Lattice dimensions.
    pub nx: usize,
    /// Lattice dimensions.
    pub ny: usize,
    /// Lattice dimensions.
    pub nz: usize,
    /// Spring stiffness.
    pub stiffness: f32,
    /// Damping coefficient.
    pub damping: f32,
    /// Node mass.
    pub node_mass: f32,
    /// Gravity vector.
    pub gravity: [f32; 3],
}

/// Return sensible defaults for [`SpringLatticeConfig`].
#[allow(dead_code)]
pub fn default_spring_lattice_config() -> SpringLatticeConfig {
    SpringLatticeConfig {
        nx: 4,
        ny: 4,
        nz: 4,
        spacing: 1.0,
        stiffness: 100.0,
        damping: 5.0,
        node_mass: 1.0,
        gravity: [0.0, -9.81, 0.0],
    }
}

/// Build a spring lattice from the given configuration.
#[allow(dead_code)]
pub fn build_spring_lattice(config: &SpringLatticeConfig) -> SpringLattice {
    let nx = config.nx.max(1);
    let ny = config.ny.max(1);
    let nz = config.nz.max(1);
    let s = config.spacing;

    let mut nodes = Vec::with_capacity(nx * ny * nz);
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                nodes.push(LatticeNode {
                    position: [ix as f32 * s, iy as f32 * s, iz as f32 * s],
                    velocity: [0.0; 3],
                    pinned: false,
                });
            }
        }
    }

    let idx = |x: usize, y: usize, z: usize| x + nx * (y + ny * z);

    let mut bonds = Vec::new();
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let a = idx(ix, iy, iz);
                if ix + 1 < nx {
                    bonds.push(LatticeBond { node_a: a, node_b: idx(ix + 1, iy, iz), rest_length: s });
                }
                if iy + 1 < ny {
                    bonds.push(LatticeBond { node_a: a, node_b: idx(ix, iy + 1, iz), rest_length: s });
                }
                if iz + 1 < nz {
                    bonds.push(LatticeBond { node_a: a, node_b: idx(ix, iy, iz + 1), rest_length: s });
                }
            }
        }
    }

    SpringLattice {
        nodes,
        bonds,
        nx,
        ny,
        nz,
        stiffness: config.stiffness,
        damping: config.damping,
        node_mass: config.node_mass,
        gravity: config.gravity,
    }
}

/// Advance the lattice by one time step `dt` (seconds).
#[allow(dead_code)]
pub fn lattice_step(lattice: &mut SpringLattice, dt: f32) {
    let n = lattice.nodes.len();
    let mut forces = vec![[0.0_f32; 3]; n];

    // Gravity
    for (i, f) in forces.iter_mut().enumerate() {
        if !lattice.nodes[i].pinned {
            f[0] += lattice.node_mass * lattice.gravity[0];
            f[1] += lattice.node_mass * lattice.gravity[1];
            f[2] += lattice.node_mass * lattice.gravity[2];
        }
    }

    // Spring forces
    let bonds = lattice.bonds.clone();
    for bond in &bonds {
        let pa = lattice.nodes[bond.node_a].position;
        let pb = lattice.nodes[bond.node_b].position;
        let dx = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let len = (dx[0] * dx[0] + dx[1] * dx[1] + dx[2] * dx[2]).sqrt();
        if len < 1e-9 {
            continue;
        }
        let ext = len - bond.rest_length;
        let mag = lattice.stiffness * ext;
        let dir = [dx[0] / len, dx[1] / len, dx[2] / len];

        // Damping: relative velocity along bond
        let va = lattice.nodes[bond.node_a].velocity;
        let vb = lattice.nodes[bond.node_b].velocity;
        let rel_v = (vb[0] - va[0]) * dir[0] + (vb[1] - va[1]) * dir[1] + (vb[2] - va[2]) * dir[2];
        let damp_mag = lattice.damping * rel_v;

        let total = mag + damp_mag;
        forces[bond.node_a][0] += total * dir[0];
        forces[bond.node_a][1] += total * dir[1];
        forces[bond.node_a][2] += total * dir[2];
        forces[bond.node_b][0] -= total * dir[0];
        forces[bond.node_b][1] -= total * dir[1];
        forces[bond.node_b][2] -= total * dir[2];
    }

    // Integrate
    let inv_mass = if lattice.node_mass > 1e-12 { 1.0 / lattice.node_mass } else { 0.0 };
    for (i, node) in lattice.nodes.iter_mut().enumerate() {
        if node.pinned {
            continue;
        }
        node.velocity[0] += forces[i][0] * inv_mass * dt;
        node.velocity[1] += forces[i][1] * inv_mass * dt;
        node.velocity[2] += forces[i][2] * inv_mass * dt;
        node.position[0] += node.velocity[0] * dt;
        node.position[1] += node.velocity[1] * dt;
        node.position[2] += node.velocity[2] * dt;
    }
}

/// Return the number of lattice nodes.
#[allow(dead_code)]
pub fn lattice_node_count(lattice: &SpringLattice) -> usize {
    lattice.nodes.len()
}

/// Return the number of spring bonds.
#[allow(dead_code)]
pub fn lattice_bond_count(lattice: &SpringLattice) -> usize {
    lattice.bonds.len()
}

/// Return the position of the node at `index`.
#[allow(dead_code)]
pub fn lattice_node_position(lattice: &SpringLattice, index: usize) -> [f32; 3] {
    lattice.nodes[index].position
}

/// Compute the total kinetic energy of the lattice.
#[allow(dead_code)]
pub fn lattice_kinetic_energy(lattice: &SpringLattice) -> f32 {
    let half_m = 0.5 * lattice.node_mass;
    lattice
        .nodes
        .iter()
        .filter(|n| !n.pinned)
        .map(|n| {
            let v2 = n.velocity[0] * n.velocity[0]
                + n.velocity[1] * n.velocity[1]
                + n.velocity[2] * n.velocity[2];
            half_m * v2
        })
        .sum()
}

/// Serialise a brief description of the lattice to JSON.
#[allow(dead_code)]
pub fn lattice_to_json(lattice: &SpringLattice) -> String {
    format!(
        r#"{{"nx":{},"ny":{},"nz":{},"nodes":{},"bonds":{},"stiffness":{:.3},"damping":{:.3}}}"#,
        lattice.nx,
        lattice.ny,
        lattice.nz,
        lattice.nodes.len(),
        lattice.bonds.len(),
        lattice.stiffness,
        lattice.damping,
    )
}

/// Reset all node velocities to zero (positions unchanged).
#[allow(dead_code)]
pub fn lattice_reset(lattice: &mut SpringLattice) {
    for node in &mut lattice.nodes {
        node.velocity = [0.0; 3];
    }
}

/// Pin (or unpin) the node at `index` so it does not move.
#[allow(dead_code)]
pub fn lattice_pin_node(lattice: &mut SpringLattice, index: usize, pinned: bool) {
    lattice.nodes[index].pinned = pinned;
    if pinned {
        lattice.nodes[index].velocity = [0.0; 3];
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_lattice() -> SpringLattice {
        let cfg = SpringLatticeConfig {
            nx: 2, ny: 2, nz: 2,
            spacing: 1.0,
            stiffness: 100.0,
            damping: 0.0,
            node_mass: 1.0,
            gravity: [0.0; 3],
        };
        build_spring_lattice(&cfg)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_spring_lattice_config();
        assert_eq!(cfg.nx, 4);
        assert_eq!(cfg.stiffness, 100.0);
    }

    #[test]
    fn test_node_count() {
        let lat = tiny_lattice();
        assert_eq!(lattice_node_count(&lat), 8);
    }

    #[test]
    fn test_bond_count() {
        let lat = tiny_lattice();
        // 2x2x2: 1+1+1 bonds per corner corner = 12 bonds in 2x2x2
        assert_eq!(lattice_bond_count(&lat), 12);
    }

    #[test]
    fn test_initial_positions() {
        let lat = tiny_lattice();
        let p = lattice_node_position(&lat, 0);
        assert_eq!(p, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_lattice_step_no_gravity_no_displacement() {
        let mut lat = tiny_lattice();
        // Pin all nodes → no movement expected
        for i in 0..lattice_node_count(&lat) {
            lattice_pin_node(&mut lat, i, true);
        }
        lattice_step(&mut lat, 0.016);
        let p = lattice_node_position(&lat, 0);
        assert_eq!(p, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_kinetic_energy_zero_at_rest() {
        let lat = tiny_lattice();
        assert_eq!(lattice_kinetic_energy(&lat), 0.0);
    }

    #[test]
    fn test_to_json_contains_nodes() {
        let lat = tiny_lattice();
        let json = lattice_to_json(&lat);
        assert!(json.contains("nodes"));
        assert!(json.contains("bonds"));
    }

    #[test]
    fn test_reset_clears_velocity() {
        let mut lat = tiny_lattice();
        lat.nodes[0].velocity = [1.0, 2.0, 3.0];
        lattice_reset(&mut lat);
        assert_eq!(lat.nodes[0].velocity, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_pin_node() {
        let mut lat = tiny_lattice();
        lattice_pin_node(&mut lat, 0, true);
        assert!(lat.nodes[0].pinned);
        assert_eq!(lat.nodes[0].velocity, [0.0, 0.0, 0.0]);
        lattice_pin_node(&mut lat, 0, false);
        assert!(!lat.nodes[0].pinned);
    }

    #[test]
    fn test_gravity_moves_free_node() {
        let cfg = SpringLatticeConfig {
            nx: 1, ny: 1, nz: 1,
            spacing: 1.0,
            stiffness: 0.0,
            damping: 0.0,
            node_mass: 1.0,
            gravity: [0.0, -9.81, 0.0],
        };
        let mut lat = build_spring_lattice(&cfg);
        let y0 = lattice_node_position(&lat, 0)[1];
        lattice_step(&mut lat, 0.1);
        let y1 = lattice_node_position(&lat, 0)[1];
        assert!(y1 < y0, "node should fall: {} < {}", y1, y0);
    }
}
