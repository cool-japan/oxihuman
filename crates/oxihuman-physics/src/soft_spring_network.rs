// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Network of springs for soft-body simulation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpringEdge {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub stiffness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpringNetwork {
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
    pub edges: Vec<SpringEdge>,
    pub inv_masses: Vec<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpringNetworkConfig {
    pub damping: f32,
    pub gravity: [f32; 3],
}

#[allow(dead_code)]
pub fn default_spring_network_config() -> SpringNetworkConfig {
    SpringNetworkConfig { damping: 0.98, gravity: [0.0, -9.81, 0.0] }
}

#[allow(dead_code)]
pub fn new_spring_network() -> SpringNetwork {
    SpringNetwork {
        positions: Vec::new(),
        velocities: Vec::new(),
        edges: Vec::new(),
        inv_masses: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn sn_add_particle(net: &mut SpringNetwork, pos: [f32; 3], inv_mass: f32) {
    net.positions.push(pos);
    net.velocities.push([0.0; 3]);
    net.inv_masses.push(inv_mass);
}

#[allow(dead_code)]
pub fn sn_add_edge(net: &mut SpringNetwork, a: usize, b: usize, stiffness: f32) {
    let pa = net.positions[a];
    let pb = net.positions[b];
    let dx = pb[0] - pa[0];
    let dy = pb[1] - pa[1];
    let dz = pb[2] - pa[2];
    let rest_len = (dx * dx + dy * dy + dz * dz).sqrt();
    net.edges.push(SpringEdge { a, b, rest_len, stiffness });
}

#[allow(dead_code)]
pub fn sn_step(net: &mut SpringNetwork, dt: f32, config: &SpringNetworkConfig) {
    let n = net.positions.len();

    // Apply gravity
    for i in 0..n {
        if net.inv_masses[i] <= 0.0 {
            continue;
        }
        for axis in 0..3 {
            net.velocities[i][axis] = (net.velocities[i][axis] + config.gravity[axis] * dt) * config.damping;
        }
    }

    // Apply spring forces
    for edge in &net.edges {
        let pa = net.positions[edge.a];
        let pb = net.positions[edge.b];
        let dx = pb[0] - pa[0];
        let dy = pb[1] - pa[1];
        let dz = pb[2] - pa[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist < 1e-10 {
            continue;
        }
        let stretch = dist - edge.rest_len;
        let force_mag = edge.stiffness * stretch;
        let nx = dx / dist;
        let ny = dy / dist;
        let nz = dz / dist;

        let ima = net.inv_masses[edge.a];
        let imb = net.inv_masses[edge.b];

        if ima > 0.0 {
            net.velocities[edge.a][0] += force_mag * nx * ima * dt;
            net.velocities[edge.a][1] += force_mag * ny * ima * dt;
            net.velocities[edge.a][2] += force_mag * nz * ima * dt;
        }
        if imb > 0.0 {
            net.velocities[edge.b][0] -= force_mag * nx * imb * dt;
            net.velocities[edge.b][1] -= force_mag * ny * imb * dt;
            net.velocities[edge.b][2] -= force_mag * nz * imb * dt;
        }
    }

    // Integrate positions
    for i in 0..n {
        if net.inv_masses[i] <= 0.0 {
            continue;
        }
        for axis in 0..3 {
            net.positions[i][axis] += net.velocities[i][axis] * dt;
        }
    }
}

#[allow(dead_code)]
pub fn sn_particle_count(net: &SpringNetwork) -> usize {
    net.positions.len()
}

#[allow(dead_code)]
pub fn sn_edge_count(net: &SpringNetwork) -> usize {
    net.edges.len()
}

#[allow(dead_code)]
pub fn sn_total_energy(net: &SpringNetwork) -> f32 {
    let mut ke = 0.0f32;
    for i in 0..net.positions.len() {
        if net.inv_masses[i] <= 0.0 {
            continue;
        }
        let mass = 1.0 / net.inv_masses[i];
        let v = &net.velocities[i];
        ke += 0.5 * mass * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
    }
    let mut pe = 0.0f32;
    for edge in &net.edges {
        let pa = net.positions[edge.a];
        let pb = net.positions[edge.b];
        let dx = pb[0] - pa[0];
        let dy = pb[1] - pa[1];
        let dz = pb[2] - pa[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        let stretch = dist - edge.rest_len;
        pe += 0.5 * edge.stiffness * stretch * stretch;
    }
    ke + pe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_spring_network_config();
        assert!((cfg.damping - 0.98).abs() < 1e-6);
    }

    #[test]
    fn test_new_empty() {
        let net = new_spring_network();
        assert_eq!(sn_particle_count(&net), 0);
        assert_eq!(sn_edge_count(&net), 0);
    }

    #[test]
    fn test_add_particle() {
        let mut net = new_spring_network();
        sn_add_particle(&mut net, [0.0; 3], 1.0);
        assert_eq!(sn_particle_count(&net), 1);
    }

    #[test]
    fn test_add_edge() {
        let mut net = new_spring_network();
        sn_add_particle(&mut net, [0.0, 0.0, 0.0], 1.0);
        sn_add_particle(&mut net, [1.0, 0.0, 0.0], 1.0);
        sn_add_edge(&mut net, 0, 1, 100.0);
        assert_eq!(sn_edge_count(&net), 1);
        assert!((net.edges[0].rest_len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_step_gravity() {
        let cfg = default_spring_network_config();
        let mut net = new_spring_network();
        sn_add_particle(&mut net, [0.0, 10.0, 0.0], 1.0);
        let before_y = net.positions[0][1];
        sn_step(&mut net, 0.1, &cfg);
        assert!(net.positions[0][1] < before_y);
    }

    #[test]
    fn test_static_particle_no_move() {
        let cfg = default_spring_network_config();
        let mut net = new_spring_network();
        sn_add_particle(&mut net, [0.0; 3], 0.0); // static
        sn_step(&mut net, 0.1, &cfg);
        assert_eq!(net.positions[0], [0.0; 3]);
    }

    #[test]
    fn test_total_energy_zero_vel() {
        let mut net = new_spring_network();
        sn_add_particle(&mut net, [0.0, 0.0, 0.0], 1.0);
        sn_add_particle(&mut net, [1.0, 0.0, 0.0], 1.0);
        sn_add_edge(&mut net, 0, 1, 10.0);
        // At rest length, spring PE = 0, KE = 0
        let e = sn_total_energy(&net);
        assert!(e < 1e-6);
    }

    #[test]
    fn test_particle_and_edge_counts() {
        let mut net = new_spring_network();
        sn_add_particle(&mut net, [0.0; 3], 1.0);
        sn_add_particle(&mut net, [1.0; 3], 1.0);
        sn_add_particle(&mut net, [2.0; 3], 1.0);
        sn_add_edge(&mut net, 0, 1, 10.0);
        sn_add_edge(&mut net, 1, 2, 10.0);
        assert_eq!(sn_particle_count(&net), 3);
        assert_eq!(sn_edge_count(&net), 2);
    }

    #[test]
    fn test_spring_restoring_force() {
        let cfg = SpringNetworkConfig { damping: 1.0, gravity: [0.0; 3] };
        let mut net = new_spring_network();
        sn_add_particle(&mut net, [0.0, 0.0, 0.0], 0.0); // static anchor
        sn_add_particle(&mut net, [2.0, 0.0, 0.0], 1.0); // stretched spring
        sn_add_edge(&mut net, 0, 1, 100.0); // rest_len = 2.0, no stretch
        // Manually set rest_len shorter to create tension
        net.edges[0].rest_len = 1.0;
        let before_x = net.positions[1][0];
        sn_step(&mut net, 0.01, &cfg);
        // Spring pulls particle 1 towards anchor (x decreases)
        assert!(net.positions[1][0] < before_x);
    }
}
