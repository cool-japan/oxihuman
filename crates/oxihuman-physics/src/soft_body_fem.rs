// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Finite Element Method (FEM) soft body simulation stub.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Physical material parameters for an FEM simulation.
pub struct FemConfig {
    /// Young's modulus (stiffness of the material, Pa).
    pub youngs_modulus: f32,
    /// Poisson's ratio (0 = no lateral contraction, 0.5 = incompressible).
    pub poisson_ratio: f32,
    /// Material density (kg/m³).
    pub density: f32,
    /// Rayleigh damping coefficient.
    pub damping: f32,
    /// Number of substeps per frame.
    pub sub_steps: u32,
}

/// A single node (particle) in the FEM mesh.
pub struct FemNode {
    /// World-space position.
    pub position: [f32; 3],
    /// Velocity.
    pub velocity: [f32; 3],
    /// Lumped nodal mass.
    pub mass: f32,
    /// If `true`, this node is pinned and its position will not change.
    pub fixed: bool,
}

/// A tetrahedral element (4 nodes).
pub struct FemElement {
    /// Indices of the 4 nodes forming this tetrahedron.
    pub node_indices: [u32; 4],
    /// Rest-pose volume of the tetrahedron.
    pub rest_volume: f32,
    /// 3×3 element stiffness matrix (simplified).
    pub stiffness_matrix: [[f32; 3]; 3],
}

/// The full FEM soft body system.
pub struct FemSystem {
    /// All nodes in the mesh.
    pub nodes: Vec<FemNode>,
    /// All tetrahedral elements.
    pub elements: Vec<FemElement>,
    /// Simulation configuration.
    pub config: FemConfig,
    /// Accumulated simulation time.
    pub time: f32,
}

/// Result of one FEM timestep.
pub struct FemResult {
    /// Node positions after the step.
    pub node_positions: Vec<[f32; 3]>,
    /// Total elastic strain energy across all elements.
    pub strain_energy: f32,
    /// Maximum displacement of any node from its previous position.
    pub max_displacement: f32,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Volume of a tetrahedron given its four vertex positions.
fn tet_volume_from_positions(
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
    p3: [f32; 3],
) -> f32 {
    let a = sub3(p1, p0);
    let b = sub3(p2, p0);
    let c = sub3(p3, p0);
    // scalar triple product / 6
    let cross = [
        b[1] * c[2] - b[2] * c[1],
        b[2] * c[0] - b[0] * c[2],
        b[0] * c[1] - b[1] * c[0],
    ];
    (dot3(a, cross) / 6.0).abs()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build a default `FemConfig` with physically plausible values for soft tissue.
#[allow(dead_code)]
pub fn default_fem_config() -> FemConfig {
    FemConfig {
        youngs_modulus: 10_000.0, // 10 kPa (soft tissue)
        poisson_ratio: 0.45,
        density: 1000.0,   // kg/m³
        damping: 0.01,
        sub_steps: 4,
    }
}

/// Construct a free FEM node at the given position with zero velocity.
#[allow(dead_code)]
pub fn new_fem_node(pos: [f32; 3], mass: f32) -> FemNode {
    FemNode {
        position: pos,
        velocity: [0.0; 3],
        mass,
        fixed: false,
    }
}

/// Construct a tetrahedral element with a given rest volume.
/// The stiffness matrix is initialised to an identity-like diagonal.
#[allow(dead_code)]
pub fn new_fem_element(indices: [u32; 4], rest_vol: f32) -> FemElement {
    FemElement {
        node_indices: indices,
        rest_volume: rest_vol,
        stiffness_matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    }
}

/// Create an empty `FemSystem` with the supplied config.
#[allow(dead_code)]
pub fn new_fem_system(cfg: FemConfig) -> FemSystem {
    FemSystem {
        nodes: Vec::new(),
        elements: Vec::new(),
        config: cfg,
        time: 0.0,
    }
}

/// Add a node to an existing `FemSystem`.
#[allow(dead_code)]
pub fn add_fem_node(sys: &mut FemSystem, node: FemNode) {
    sys.nodes.push(node);
}

/// Add a tetrahedral element to an existing `FemSystem`.
#[allow(dead_code)]
pub fn add_fem_element(sys: &mut FemSystem, elem: FemElement) {
    sys.elements.push(elem);
}

/// Advance the FEM simulation by one timestep `dt`.
///
/// This is a simplified semi-implicit Euler integration.
/// Internal forces are computed per-element and applied to nodes.
#[allow(dead_code)]
pub fn step_fem(sys: &mut FemSystem, dt: f32) -> FemResult {
    let sub_dt = if sys.config.sub_steps > 0 {
        dt / sys.config.sub_steps as f32
    } else {
        dt
    };

    let prev_positions: Vec<[f32; 3]> = sys.nodes.iter().map(|n| n.position).collect();

    for _ in 0..sys.config.sub_steps.max(1) {
        // For each element apply spring-like forces from volume change.
        let mut forces: Vec<[f32; 3]> = vec![[0.0; 3]; sys.nodes.len()];

        for elem in &sys.elements {
            let idx = elem.node_indices;
            if idx.iter().any(|&i| i as usize >= sys.nodes.len()) {
                continue;
            }
            let p: [[f32; 3]; 4] = [
                sys.nodes[idx[0] as usize].position,
                sys.nodes[idx[1] as usize].position,
                sys.nodes[idx[2] as usize].position,
                sys.nodes[idx[3] as usize].position,
            ];
            let vol = tet_volume_from_positions(p[0], p[1], p[2], p[3]);
            let vol_strain = vol - elem.rest_volume;

            // Simple volumetric restoring force: push nodes away from centroid if
            // compressed, toward centroid if over-expanded.
            let centroid = [
                (p[0][0] + p[1][0] + p[2][0] + p[3][0]) * 0.25,
                (p[0][1] + p[1][1] + p[2][1] + p[3][1]) * 0.25,
                (p[0][2] + p[1][2] + p[2][2] + p[3][2]) * 0.25,
            ];

            let k = sys.config.youngs_modulus * elem.rest_volume.max(1e-10);
            for &ni in &idx {
                let ni = ni as usize;
                let dir = sub3(p[ni % 4], centroid);
                let l = len3(dir);
                if l > 1e-12 {
                    let f_scalar = k * vol_strain / l;
                    forces[ni][0] += dir[0] / l * f_scalar;
                    forces[ni][1] += dir[1] / l * f_scalar;
                    forces[ni][2] += dir[2] / l * f_scalar;
                }
            }
        }

        // Integrate nodes.
        for (i, node) in sys.nodes.iter_mut().enumerate() {
            if node.fixed || node.mass <= 0.0 {
                continue;
            }
            let accel = scale3(forces[i], 1.0 / node.mass);
            // Apply damping.
            let damp = sys.config.damping;
            node.velocity[0] = node.velocity[0] * (1.0 - damp) + accel[0] * sub_dt;
            node.velocity[1] = node.velocity[1] * (1.0 - damp) + accel[1] * sub_dt;
            node.velocity[2] = node.velocity[2] * (1.0 - damp) + accel[2] * sub_dt;
            node.position[0] += node.velocity[0] * sub_dt;
            node.position[1] += node.velocity[1] * sub_dt;
            node.position[2] += node.velocity[2] * sub_dt;
        }
    }

    sys.time += dt;

    let strain_energy = fem_strain_energy(sys);

    let max_displacement = sys
        .nodes
        .iter()
        .zip(prev_positions.iter())
        .map(|(n, &prev)| len3(sub3(n.position, prev)))
        .fold(0.0_f32, f32::max);

    FemResult {
        node_positions: sys.nodes.iter().map(|n| n.position).collect(),
        strain_energy,
        max_displacement,
    }
}

/// Compute total elastic strain energy stored in the system.
#[allow(dead_code)]
pub fn fem_strain_energy(sys: &FemSystem) -> f32 {
    let mut total = 0.0f32;
    for elem in &sys.elements {
        let idx = elem.node_indices;
        if idx.iter().any(|&i| i as usize >= sys.nodes.len()) {
            continue;
        }
        let p: [[f32; 3]; 4] = [
            sys.nodes[idx[0] as usize].position,
            sys.nodes[idx[1] as usize].position,
            sys.nodes[idx[2] as usize].position,
            sys.nodes[idx[3] as usize].position,
        ];
        let vol = tet_volume_from_positions(p[0], p[1], p[2], p[3]);
        let vol_strain = vol - elem.rest_volume;
        total += 0.5 * sys.config.youngs_modulus * vol_strain * vol_strain;
    }
    total
}

/// Return the number of nodes in the system.
#[allow(dead_code)]
pub fn fem_node_count(sys: &FemSystem) -> usize {
    sys.nodes.len()
}

/// Serialize the FEM system state to a compact JSON string.
#[allow(dead_code)]
pub fn fem_system_to_json(sys: &FemSystem) -> String {
    format!(
        "{{\"nodes\":{},\"elements\":{},\"time\":{},\"youngs_modulus\":{}}}",
        sys.nodes.len(),
        sys.elements.len(),
        sys.time,
        sys.config.youngs_modulus,
    )
}

/// Serialize a `FemResult` to a compact JSON string.
#[allow(dead_code)]
pub fn fem_result_to_json(r: &FemResult) -> String {
    format!(
        "{{\"node_count\":{},\"strain_energy\":{},\"max_displacement\":{}}}",
        r.node_positions.len(),
        r.strain_energy,
        r.max_displacement,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_fem_system() -> FemSystem {
        let cfg = default_fem_config();
        let mut sys = new_fem_system(cfg);
        // Four nodes forming a unit tetrahedron.
        add_fem_node(&mut sys, new_fem_node([0.0, 0.0, 0.0], 1.0));
        add_fem_node(&mut sys, new_fem_node([1.0, 0.0, 0.0], 1.0));
        add_fem_node(&mut sys, new_fem_node([0.0, 1.0, 0.0], 1.0));
        add_fem_node(&mut sys, new_fem_node([0.0, 0.0, 1.0], 1.0));
        let rest_vol = 1.0 / 6.0; // volume of the unit tet
        add_fem_element(&mut sys, new_fem_element([0, 1, 2, 3], rest_vol));
        sys
    }

    #[test]
    fn fem_node_count_correct() {
        let sys = simple_fem_system();
        assert_eq!(fem_node_count(&sys), 4);
    }

    #[test]
    fn fem_strain_energy_at_rest_near_zero() {
        let sys = simple_fem_system();
        // At rest shape, strain should be near zero.
        let e = fem_strain_energy(&sys);
        assert!(e < 1e-3, "strain energy at rest = {}", e);
    }

    #[test]
    fn step_fem_returns_node_positions() {
        let mut sys = simple_fem_system();
        let res = step_fem(&mut sys, 0.01);
        assert_eq!(res.node_positions.len(), 4);
    }

    #[test]
    fn step_fem_advances_time() {
        let mut sys = simple_fem_system();
        step_fem(&mut sys, 0.01);
        assert!((sys.time - 0.01).abs() < 1e-7);
    }

    #[test]
    fn fem_system_to_json_contains_fields() {
        let sys = simple_fem_system();
        let json = fem_system_to_json(&sys);
        assert!(json.contains("nodes"));
        assert!(json.contains("elements"));
    }

    #[test]
    fn fem_result_to_json_contains_fields() {
        let mut sys = simple_fem_system();
        let res = step_fem(&mut sys, 0.01);
        let json = fem_result_to_json(&res);
        assert!(json.contains("strain_energy"));
        assert!(json.contains("max_displacement"));
    }

    #[test]
    fn fixed_node_does_not_move() {
        let cfg = default_fem_config();
        let mut sys = new_fem_system(cfg);
        let mut n0 = new_fem_node([0.0, 0.0, 0.0], 1.0);
        n0.fixed = true;
        add_fem_node(&mut sys, n0);
        add_fem_node(&mut sys, new_fem_node([1.0, 0.0, 0.0], 1.0));
        add_fem_node(&mut sys, new_fem_node([0.0, 1.0, 0.0], 1.0));
        add_fem_node(&mut sys, new_fem_node([0.0, 0.0, 1.0], 1.0));
        add_fem_element(&mut sys, new_fem_element([0, 1, 2, 3], 1.0 / 6.0));
        step_fem(&mut sys, 0.01);
        let pos = sys.nodes[0].position;
        assert!((pos[0]).abs() < 1e-7);
        assert!((pos[1]).abs() < 1e-7);
        assert!((pos[2]).abs() < 1e-7);
    }

    #[test]
    fn new_fem_element_identity_stiffness() {
        let elem = new_fem_element([0, 1, 2, 3], 0.5);
        assert!((elem.stiffness_matrix[0][0] - 1.0).abs() < 1e-7);
        assert!((elem.stiffness_matrix[1][1] - 1.0).abs() < 1e-7);
        assert!((elem.stiffness_matrix[2][2] - 1.0).abs() < 1e-7);
    }
}
