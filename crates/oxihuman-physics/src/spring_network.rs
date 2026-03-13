// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mass-spring network simulation (structural, shear, bending springs).

#[allow(dead_code)]
pub struct SpringNode {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub pinned: bool,
}

#[allow(dead_code)]
pub struct Spring {
    pub node_a: usize,
    pub node_b: usize,
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
}

#[allow(dead_code)]
pub struct SpringNetwork {
    pub nodes: Vec<SpringNode>,
    pub springs: Vec<Spring>,
    pub gravity: [f32; 3],
}

#[allow(dead_code)]
pub struct SpringNetworkConfig {
    pub time_step: f32,
    pub substeps: usize,
    pub air_damping: f32,
}

/// Create an empty spring network with the given gravity vector.
#[allow(dead_code)]
pub fn new_network(gravity: [f32; 3]) -> SpringNetwork {
    SpringNetwork {
        nodes: Vec::new(),
        springs: Vec::new(),
        gravity,
    }
}

/// Add a node at `pos` with given `mass` and pinned status. Returns node index.
#[allow(dead_code)]
pub fn add_node(net: &mut SpringNetwork, pos: [f32; 3], mass: f32, pinned: bool) -> usize {
    let idx = net.nodes.len();
    net.nodes.push(SpringNode {
        position: pos,
        velocity: [0.0; 3],
        mass,
        pinned,
    });
    idx
}

/// Add a spring between nodes `a` and `b`, auto-computing rest length from current positions.
#[allow(dead_code)]
pub fn add_spring(net: &mut SpringNetwork, a: usize, b: usize, stiffness: f32, damping: f32) {
    let pa = net.nodes[a].position;
    let pb = net.nodes[b].position;
    let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
    let rest_length = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    net.springs.push(Spring {
        node_a: a,
        node_b: b,
        rest_length,
        stiffness,
        damping,
    });
}

/// Compute spring force on node_a from the spring (Hooke's law + velocity damping).
#[allow(dead_code)]
pub fn spring_force(node_a: &SpringNode, node_b: &SpringNode, spring: &Spring) -> [f32; 3] {
    let d = [
        node_b.position[0] - node_a.position[0],
        node_b.position[1] - node_a.position[1],
        node_b.position[2] - node_a.position[2],
    ];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len < 1e-10 {
        return [0.0; 3];
    }
    let dir = [d[0] / len, d[1] / len, d[2] / len];
    let extension = len - spring.rest_length;
    // Relative velocity along spring axis for damping
    let rel_v = [
        node_b.velocity[0] - node_a.velocity[0],
        node_b.velocity[1] - node_a.velocity[1],
        node_b.velocity[2] - node_a.velocity[2],
    ];
    let rel_v_proj = rel_v[0] * dir[0] + rel_v[1] * dir[1] + rel_v[2] * dir[2];
    let force_mag = spring.stiffness * extension + spring.damping * rel_v_proj;
    [dir[0] * force_mag, dir[1] * force_mag, dir[2] * force_mag]
}

/// Perform one time step using semi-implicit Euler integration.
#[allow(dead_code)]
pub fn step_network(net: &mut SpringNetwork, cfg: &SpringNetworkConfig) {
    let dt = cfg.time_step / cfg.substeps.max(1) as f32;
    for _ in 0..cfg.substeps.max(1) {
        // Collect forces
        let n = net.nodes.len();
        let mut forces = vec![[0.0_f32; 3]; n];
        // Gravity
        for (i, node) in net.nodes.iter().enumerate() {
            if !node.pinned {
                forces[i][0] += net.gravity[0] * node.mass;
                forces[i][1] += net.gravity[1] * node.mass;
                forces[i][2] += net.gravity[2] * node.mass;
            }
        }
        // Spring forces — collect indices first to avoid borrow issues
        let spring_data: Vec<(usize, usize, [f32; 3])> = net
            .springs
            .iter()
            .map(|s| {
                let fa = spring_force(&net.nodes[s.node_a], &net.nodes[s.node_b], s);
                (s.node_a, s.node_b, fa)
            })
            .collect();
        for (a, b, fa) in spring_data {
            if !net.nodes[a].pinned {
                forces[a][0] += fa[0];
                forces[a][1] += fa[1];
                forces[a][2] += fa[2];
            }
            if !net.nodes[b].pinned {
                forces[b][0] -= fa[0];
                forces[b][1] -= fa[1];
                forces[b][2] -= fa[2];
            }
        }
        // Integrate
        for (i, node) in net.nodes.iter_mut().enumerate() {
            if node.pinned {
                continue;
            }
            let inv_mass = if node.mass > 1e-10 {
                1.0 / node.mass
            } else {
                0.0
            };
            // Update velocity (semi-implicit)
            node.velocity[0] = (node.velocity[0] + forces[i][0] * inv_mass * dt)
                * (1.0 - cfg.air_damping * dt).max(0.0);
            node.velocity[1] = (node.velocity[1] + forces[i][1] * inv_mass * dt)
                * (1.0 - cfg.air_damping * dt).max(0.0);
            node.velocity[2] = (node.velocity[2] + forces[i][2] * inv_mass * dt)
                * (1.0 - cfg.air_damping * dt).max(0.0);
            // Update position
            node.position[0] += node.velocity[0] * dt;
            node.position[1] += node.velocity[1] * dt;
            node.position[2] += node.velocity[2] * dt;
        }
    }
}

/// Compute total kinetic + potential energy of the network.
#[allow(dead_code)]
pub fn network_energy(net: &SpringNetwork) -> f32 {
    let mut energy = 0.0_f32;
    // Kinetic
    for node in &net.nodes {
        let v2 = node.velocity[0].powi(2) + node.velocity[1].powi(2) + node.velocity[2].powi(2);
        energy += 0.5 * node.mass * v2;
    }
    // Potential (spring)
    for s in &net.springs {
        let pa = net.nodes[s.node_a].position;
        let pb = net.nodes[s.node_b].position;
        let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let ext = len - s.rest_length;
        energy += 0.5 * s.stiffness * ext * ext;
    }
    energy
}

/// Compute the axis-aligned bounding box of all nodes.
#[allow(dead_code)]
pub fn network_bounding_box(net: &SpringNetwork) -> ([f32; 3], [f32; 3]) {
    if net.nodes.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = net.nodes[0].position;
    let mut mx = net.nodes[0].position;
    for node in net.nodes.iter().skip(1) {
        for i in 0..3 {
            if node.position[i] < mn[i] {
                mn[i] = node.position[i];
            }
            if node.position[i] > mx[i] {
                mx[i] = node.position[i];
            }
        }
    }
    (mn, mx)
}

/// Compute current length - rest length for a spring.
#[allow(dead_code)]
pub fn spring_extension(net: &SpringNetwork, spring_idx: usize) -> f32 {
    let s = &net.springs[spring_idx];
    let pa = net.nodes[s.node_a].position;
    let pb = net.nodes[s.node_b].position;
    let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    len - s.rest_length
}

/// Count pinned nodes.
#[allow(dead_code)]
pub fn count_pinned(net: &SpringNetwork) -> usize {
    net.nodes.iter().filter(|n| n.pinned).count()
}

/// Build a 2D grid of mass-spring nodes. Nodes are in the XZ plane.
#[allow(dead_code)]
pub fn build_grid_network(rows: usize, cols: usize, spacing: f32, stiffness: f32) -> SpringNetwork {
    let mut net = new_network([0.0, -9.81, 0.0]);
    // Add nodes
    for r in 0..rows {
        for c in 0..cols {
            let pinned = r == 0;
            add_node(
                &mut net,
                [c as f32 * spacing, 0.0, r as f32 * spacing],
                1.0,
                pinned,
            );
        }
    }
    // Structural springs (horizontal and vertical)
    for r in 0..rows {
        for c in 0..cols {
            let idx = r * cols + c;
            if c + 1 < cols {
                add_spring(&mut net, idx, idx + 1, stiffness, 0.1);
            }
            if r + 1 < rows {
                add_spring(&mut net, idx, idx + cols, stiffness, 0.1);
            }
        }
    }
    // Shear springs
    for r in 0..rows.saturating_sub(1) {
        for c in 0..cols.saturating_sub(1) {
            let idx = r * cols + c;
            add_spring(&mut net, idx, idx + cols + 1, stiffness * 0.5, 0.1);
            add_spring(&mut net, idx + 1, idx + cols, stiffness * 0.5, 0.1);
        }
    }
    net
}

/// Apply an instantaneous velocity impulse to a node.
#[allow(dead_code)]
pub fn apply_impulse(net: &mut SpringNetwork, node_idx: usize, impulse: [f32; 3]) {
    let node = &mut net.nodes[node_idx];
    if node.pinned {
        return;
    }
    let inv_mass = if node.mass > 1e-10 {
        1.0 / node.mass
    } else {
        0.0
    };
    node.velocity[0] += impulse[0] * inv_mass;
    node.velocity[1] += impulse[1] * inv_mass;
    node.velocity[2] += impulse[2] * inv_mass;
}

/// Clamp all node velocities to a maximum speed.
#[allow(dead_code)]
pub fn clamp_velocities(net: &mut SpringNetwork, max_speed: f32) {
    for node in net.nodes.iter_mut() {
        let speed =
            (node.velocity[0].powi(2) + node.velocity[1].powi(2) + node.velocity[2].powi(2)).sqrt();
        if speed > max_speed && speed > 1e-10 {
            let scale = max_speed / speed;
            node.velocity[0] *= scale;
            node.velocity[1] *= scale;
            node.velocity[2] *= scale;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_network() -> SpringNetwork {
        let mut net = new_network([0.0, -9.81, 0.0]);
        add_node(&mut net, [0.0, 0.0, 0.0], 1.0, true);
        add_node(&mut net, [1.0, 0.0, 0.0], 1.0, false);
        add_spring(&mut net, 0, 1, 100.0, 1.0);
        net
    }

    #[test]
    fn add_node_increments_count() {
        let mut net = new_network([0.0, -9.81, 0.0]);
        add_node(&mut net, [0.0; 3], 1.0, false);
        add_node(&mut net, [1.0, 0.0, 0.0], 1.0, false);
        assert_eq!(net.nodes.len(), 2);
    }

    #[test]
    fn add_spring_increments_count() {
        let net = simple_network();
        assert_eq!(net.springs.len(), 1);
    }

    #[test]
    fn spring_force_zero_at_rest_length() {
        let node_a = SpringNode {
            position: [0.0, 0.0, 0.0],
            velocity: [0.0; 3],
            mass: 1.0,
            pinned: false,
        };
        let node_b = SpringNode {
            position: [1.0, 0.0, 0.0],
            velocity: [0.0; 3],
            mass: 1.0,
            pinned: false,
        };
        let spring = Spring {
            node_a: 0,
            node_b: 1,
            rest_length: 1.0,
            stiffness: 100.0,
            damping: 1.0,
        };
        let f = spring_force(&node_a, &node_b, &spring);
        assert!(f[0].abs() < 1e-5 && f[1].abs() < 1e-5 && f[2].abs() < 1e-5);
    }

    #[test]
    fn spring_force_nonzero_when_extended() {
        let node_a = SpringNode {
            position: [0.0, 0.0, 0.0],
            velocity: [0.0; 3],
            mass: 1.0,
            pinned: false,
        };
        let node_b = SpringNode {
            position: [2.0, 0.0, 0.0], // extended by 1
            velocity: [0.0; 3],
            mass: 1.0,
            pinned: false,
        };
        let spring = Spring {
            node_a: 0,
            node_b: 1,
            rest_length: 1.0,
            stiffness: 100.0,
            damping: 0.0,
        };
        let f = spring_force(&node_a, &node_b, &spring);
        assert!(f[0] > 0.0, "force should pull toward extended node");
    }

    #[test]
    fn step_changes_position_of_free_node() {
        let mut net = simple_network();
        let cfg = SpringNetworkConfig {
            time_step: 0.01,
            substeps: 1,
            air_damping: 0.0,
        };
        let initial_y = net.nodes[1].position[1];
        step_network(&mut net, &cfg);
        // Gravity should pull the free node down
        assert!(net.nodes[1].position[1] < initial_y);
    }

    #[test]
    fn pinned_nodes_dont_move() {
        let mut net = simple_network();
        let cfg = SpringNetworkConfig {
            time_step: 0.01,
            substeps: 10,
            air_damping: 0.0,
        };
        let initial_pos = net.nodes[0].position;
        step_network(&mut net, &cfg);
        assert_eq!(net.nodes[0].position, initial_pos);
    }

    #[test]
    fn network_energy_nonneg() {
        let net = simple_network();
        assert!(network_energy(&net) >= 0.0);
    }

    #[test]
    fn network_energy_increases_after_extension() {
        let mut net = simple_network();
        // Move free node to extend the spring
        net.nodes[1].position = [3.0, 0.0, 0.0];
        let e = network_energy(&net);
        assert!(e > 0.0);
    }

    #[test]
    fn network_bounding_box_correct() {
        let net = simple_network();
        let (mn, mx) = network_bounding_box(&net);
        assert!((mn[0] - 0.0).abs() < 1e-6);
        assert!((mx[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn spring_extension_at_rest_is_zero() {
        let net = simple_network();
        let ext = spring_extension(&net, 0);
        assert!(ext.abs() < 1e-5);
    }

    #[test]
    fn count_pinned_correct() {
        let net = simple_network();
        assert_eq!(count_pinned(&net), 1);
    }

    #[test]
    fn grid_network_node_count() {
        let net = build_grid_network(4, 5, 1.0, 100.0);
        assert_eq!(net.nodes.len(), 20);
    }

    #[test]
    fn apply_impulse_changes_velocity() {
        let mut net = simple_network();
        apply_impulse(&mut net, 1, [0.0, 10.0, 0.0]);
        assert!(net.nodes[1].velocity[1] > 0.0);
    }

    #[test]
    fn clamp_velocities_caps_speed() {
        let mut net = new_network([0.0; 3]);
        add_node(&mut net, [0.0; 3], 1.0, false);
        net.nodes[0].velocity = [100.0, 100.0, 100.0];
        clamp_velocities(&mut net, 1.0);
        let speed = (net.nodes[0].velocity[0].powi(2)
            + net.nodes[0].velocity[1].powi(2)
            + net.nodes[0].velocity[2].powi(2))
        .sqrt();
        assert!(speed <= 1.0 + 1e-5);
    }
}
