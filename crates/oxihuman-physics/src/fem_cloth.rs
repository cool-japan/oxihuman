// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! FEM-based cloth simulation using membrane elements.

// ── Structs ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FemClothConfig {
    pub thickness: f32,
    pub youngs_modulus: f32,
    pub poisson_ratio: f32,
    pub damping: f32,
    pub gravity: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FemClothNode {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub pinned: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FemClothElement {
    pub node_a: u32,
    pub node_b: u32,
    pub node_c: u32,
    pub rest_area: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FemClothSystem {
    pub nodes: Vec<FemClothNode>,
    pub elements: Vec<FemClothElement>,
    pub config: FemClothConfig,
    pub time: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FemClothResult {
    pub node_positions: Vec<[f32; 3]>,
    pub strain_energy: f32,
    pub kinetic_energy: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_fem_cloth_config() -> FemClothConfig {
    FemClothConfig {
        thickness: 0.001,
        youngs_modulus: 1e5,
        poisson_ratio: 0.3,
        damping: 0.01,
        gravity: [0.0, -9.81, 0.0],
    }
}

#[allow(dead_code)]
pub fn new_fem_cloth_node(pos: [f32; 3], mass: f32) -> FemClothNode {
    FemClothNode {
        position: pos,
        velocity: [0.0; 3],
        mass,
        pinned: false,
    }
}

#[allow(dead_code)]
pub fn new_fem_cloth_element(a: u32, b: u32, c: u32, rest_area: f32) -> FemClothElement {
    FemClothElement {
        node_a: a,
        node_b: b,
        node_c: c,
        rest_area,
    }
}

#[allow(dead_code)]
pub fn new_fem_cloth_system(cfg: FemClothConfig) -> FemClothSystem {
    FemClothSystem {
        nodes: Vec::new(),
        elements: Vec::new(),
        config: cfg,
        time: 0.0,
    }
}

#[allow(dead_code)]
pub fn add_cloth_node(sys: &mut FemClothSystem, node: FemClothNode) {
    sys.nodes.push(node);
}

#[allow(dead_code)]
pub fn add_cloth_element(sys: &mut FemClothSystem, elem: FemClothElement) {
    sys.elements.push(elem);
}

fn triangle_area(pa: [f32; 3], pb: [f32; 3], pc: [f32; 3]) -> f32 {
    let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
    let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt()
}

#[allow(dead_code)]
pub fn step_fem_cloth(sys: &mut FemClothSystem, dt: f32) -> FemClothResult {
    let g = sys.config.gravity;
    let damping = sys.config.damping;

    // Compute forces and integrate
    let n = sys.nodes.len();
    let mut forces = vec![[0.0f32; 3]; n];

    // Gravity
    for (i, node) in sys.nodes.iter().enumerate() {
        if !node.pinned {
            forces[i][0] += node.mass * g[0];
            forces[i][1] += node.mass * g[1];
            forces[i][2] += node.mass * g[2];
        }
    }

    // Simple membrane spring forces from elements
    let k = sys.config.youngs_modulus * sys.config.thickness;
    let elements = sys.elements.clone();
    for elem in &elements {
        let ia = elem.node_a as usize;
        let ib = elem.node_b as usize;
        let ic = elem.node_c as usize;
        if ia >= n || ib >= n || ic >= n {
            continue;
        }
        let pa = sys.nodes[ia].position;
        let pb = sys.nodes[ib].position;
        let pc = sys.nodes[ic].position;
        let area = triangle_area(pa, pb, pc);
        let strain = ((area - elem.rest_area) / elem.rest_area.max(1e-10)).clamp(-1.0, 1.0);

        // Simple equal distribution of restoring force to each vertex
        let f = -k * strain * elem.rest_area * (1.0 / 3.0);
        if !sys.nodes[ia].pinned {
            for v in &mut forces[ia] { *v += f; }
        }
        if !sys.nodes[ib].pinned {
            for v in &mut forces[ib] { *v += f; }
        }
        if !sys.nodes[ic].pinned {
            for v in &mut forces[ic] { *v += f; }
        }
    }

    // Integrate
    for (i, node) in sys.nodes.iter_mut().enumerate() {
        if node.pinned {
            continue;
        }
        let inv_mass = if node.mass > 1e-10 { 1.0 / node.mass } else { 0.0 };
        for ((f, vel), pos) in forces[i]
            .iter()
            .zip(node.velocity.iter_mut())
            .zip(node.position.iter_mut())
        {
            let accel = f * inv_mass;
            *vel = *vel * (1.0 - damping) + accel * dt;
            *pos += *vel * dt;
        }
    }

    sys.time += dt;

    // Compute energies
    let kinetic_energy: f32 = sys.nodes.iter().map(|node| {
        let v2: f32 = node.velocity.iter().map(|&v| v * v).sum();
        0.5 * node.mass * v2
    }).sum();

    let strain_energy: f32 = sys.elements.iter().map(|elem| {
        let ia = elem.node_a as usize;
        let ib = elem.node_b as usize;
        let ic = elem.node_c as usize;
        if ia >= sys.nodes.len() || ib >= sys.nodes.len() || ic >= sys.nodes.len() {
            return 0.0;
        }
        let pa = sys.nodes[ia].position;
        let pb = sys.nodes[ib].position;
        let pc = sys.nodes[ic].position;
        let area = triangle_area(pa, pb, pc);
        let strain = (area - elem.rest_area) / elem.rest_area.max(1e-10);
        0.5 * k * strain * strain * elem.rest_area
    }).sum();

    let node_positions: Vec<[f32; 3]> = sys.nodes.iter().map(|n| n.position).collect();

    FemClothResult {
        node_positions,
        strain_energy,
        kinetic_energy,
    }
}

#[allow(dead_code)]
pub fn cloth_node_count(sys: &FemClothSystem) -> usize {
    sys.nodes.len()
}

#[allow(dead_code)]
pub fn fem_cloth_system_to_json(sys: &FemClothSystem) -> String {
    format!(
        "{{\"node_count\":{},\"element_count\":{},\"time\":{}}}",
        sys.nodes.len(),
        sys.elements.len(),
        sys.time
    )
}

#[allow(dead_code)]
pub fn fem_cloth_result_to_json(r: &FemClothResult) -> String {
    format!(
        "{{\"node_count\":{},\"strain_energy\":{},\"kinetic_energy\":{}}}",
        r.node_positions.len(),
        r.strain_energy,
        r.kinetic_energy
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_cloth_system() -> FemClothSystem {
        let cfg = default_fem_cloth_config();
        let mut sys = new_fem_cloth_system(cfg);
        add_cloth_node(&mut sys, new_fem_cloth_node([0.0, 1.0, 0.0], 1.0));
        add_cloth_node(&mut sys, new_fem_cloth_node([1.0, 1.0, 0.0], 1.0));
        add_cloth_node(&mut sys, new_fem_cloth_node([0.5, 1.0, 1.0], 1.0));
        let area = triangle_area(
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.5, 1.0, 1.0],
        );
        add_cloth_element(&mut sys, new_fem_cloth_element(0, 1, 2, area));
        sys
    }

    #[test]
    fn test_default_config() {
        let cfg = default_fem_cloth_config();
        assert!(cfg.thickness > 0.0);
        assert!(cfg.youngs_modulus > 0.0);
    }

    #[test]
    fn test_cloth_node_count() {
        let sys = simple_cloth_system();
        assert_eq!(cloth_node_count(&sys), 3);
    }

    #[test]
    fn test_step_produces_result() {
        let mut sys = simple_cloth_system();
        let result = step_fem_cloth(&mut sys, 0.01);
        assert_eq!(result.node_positions.len(), 3);
    }

    #[test]
    fn test_gravity_moves_nodes() {
        let mut sys = simple_cloth_system();
        let y0 = sys.nodes[0].position[1];
        step_fem_cloth(&mut sys, 0.1);
        let y1 = sys.nodes[0].position[1];
        // With gravity pointing down, y should decrease
        assert!(y1 < y0, "gravity should move nodes downward");
    }

    #[test]
    fn test_pinned_node_does_not_move() {
        let cfg = default_fem_cloth_config();
        let mut sys = new_fem_cloth_system(cfg);
        let mut pinned = new_fem_cloth_node([0.0, 0.0, 0.0], 1.0);
        pinned.pinned = true;
        add_cloth_node(&mut sys, pinned);
        add_cloth_node(&mut sys, new_fem_cloth_node([1.0, 0.0, 0.0], 1.0));
        add_cloth_node(&mut sys, new_fem_cloth_node([0.5, 1.0, 0.0], 1.0));
        let area = triangle_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]);
        add_cloth_element(&mut sys, new_fem_cloth_element(0, 1, 2, area));
        step_fem_cloth(&mut sys, 0.1);
        let pos = sys.nodes[0].position;
        assert!((pos[0]).abs() < 1e-6);
        assert!((pos[1]).abs() < 1e-6);
        assert!((pos[2]).abs() < 1e-6);
    }

    #[test]
    fn test_system_to_json() {
        let sys = simple_cloth_system();
        let json = fem_cloth_system_to_json(&sys);
        assert!(json.contains("\"node_count\":3"));
        assert!(json.contains("\"element_count\":1"));
    }

    #[test]
    fn test_result_to_json() {
        let mut sys = simple_cloth_system();
        let result = step_fem_cloth(&mut sys, 0.01);
        let json = fem_cloth_result_to_json(&result);
        assert!(json.contains("\"node_count\":3"));
    }

    #[test]
    fn test_kinetic_energy_nonnegative() {
        let mut sys = simple_cloth_system();
        let result = step_fem_cloth(&mut sys, 0.01);
        assert!(result.kinetic_energy >= 0.0);
    }
}
