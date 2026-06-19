// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
#![allow(clippy::needless_range_loop)]

//! Articulated rigid-body tree with spatial-inertia forward dynamics.
//!
//! `forward_dynamics_step` performs real multibody forward dynamics rather than a
//! scalar `τ/mass·dt` integrator:
//!
//! 1. **Forward kinematics** places every link's joint origin, world joint axis,
//!    centre of mass, and orientation from the link geometry
//!    (`joint_axis`, `com_offset`, `link_length`) and the current joint angles.
//! 2. The **joint-space inertia matrix** `H(q)` is assembled by the Composite-
//!    Rigid-Body method: each link's spatial inertia is propagated through the
//!    tree via world-frame geometric Jacobians, so coupling between joints is
//!    captured exactly (it reproduces the parallel-axis theorem for a single
//!    link).
//! 3. The velocity-dependent **bias forces** `C(q,q̇)` (Coriolis + centrifugal)
//!    are computed with the Recursive Newton–Euler algorithm.
//! 4. Forward dynamics solves `H q̈ = τ − C` and integrates with semi-implicit
//!    Euler.

// ---------------------------------------------------------------------------
// data types
// ---------------------------------------------------------------------------

/// A single body (link) in the articulated tree, driven by a 1-DOF revolute
/// inboard joint.
#[derive(Debug, Clone)]
pub struct ArtBody {
    /// Link mass (kg).
    pub mass: f32,
    /// Rotational inertia about the COM, packed as `[Ixx, Iyy, Izz, Ixy, Ixz, Iyz]`.
    pub inertia: [f32; 6],
    /// Parent body index (`None` for a root).
    pub parent: Option<usize>,
    /// Revolute joint axis in the body's local frame (need not be unit length).
    pub joint_axis: [f32; 3],
    /// Centre-of-mass offset from the inboard joint origin, in the body frame.
    pub com_offset: [f32; 3],
    /// Distance from this body's inboard joint to where its children attach
    /// (placed along local +X), used to position child joint origins.
    pub link_length: f32,
}

/// Articulated rigid-body tree. `positions` / `velocities` are per-joint angles
/// and angular velocities (1-DOF revolute joints), indexed by body.
#[derive(Debug, Clone, Default)]
pub struct ArtBodyTree {
    pub bodies: Vec<ArtBody>,
    pub positions: Vec<f32>,
    pub velocities: Vec<f32>,
}

impl ArtBodyTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_body(&mut self, body: ArtBody) -> usize {
        let idx = self.bodies.len();
        self.positions.push(0.0);
        self.velocities.push(0.0);
        self.bodies.push(body);
        idx
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }
}

// ---------------------------------------------------------------------------
// 3-vector / 3×3-matrix helpers (pure Rust, no external math deps)
// ---------------------------------------------------------------------------

type V3 = [f32; 3];
type M3 = [[f32; 3]; 3];

fn v_sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn v_add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn v_scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn v_dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn v_cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn v_norm(a: V3) -> V3 {
    let l = v_dot(a, a).sqrt();
    if l < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        v_scale(a, 1.0 / l)
    }
}

fn m_identity() -> M3 {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

fn m_mul(a: &M3, b: &M3) -> M3 {
    let mut r = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            let mut s = 0.0f32;
            for k in 0..3 {
                s += a[i][k] * b[k][j];
            }
            r[i][j] = s;
        }
    }
    r
}

fn m_transpose(a: &M3) -> M3 {
    let mut r = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = a[j][i];
        }
    }
    r
}

/// Matrix–vector product `M·v`.
fn m_vec(a: &M3, v: V3) -> V3 {
    [v_dot(a[0], v), v_dot(a[1], v), v_dot(a[2], v)]
}

/// Symmetric rotational inertia matrix from the packed `[Ixx,Iyy,Izz,Ixy,Ixz,Iyz]`.
fn inertia_matrix(i: [f32; 6]) -> M3 {
    [[i[0], i[3], i[4]], [i[3], i[1], i[5]], [i[4], i[5], i[2]]]
}

/// Rodrigues rotation matrix for a rotation of `angle` about `axis`.
fn rot_axis_angle(axis: V3, angle: f32) -> M3 {
    let u = v_norm(axis);
    let s = angle.sin();
    let c = angle.cos();
    let t = 1.0 - c;
    let (x, y, z) = (u[0], u[1], u[2]);
    [
        [c + x * x * t, x * y * t - z * s, x * z * t + y * s],
        [y * x * t + z * s, c + y * y * t, y * z * t - x * s],
        [z * x * t - y * s, z * y * t + x * s, c + z * z * t],
    ]
}

/// World-frame rotational inertia `R·Ī·Rᵀ`.
fn world_inertia(body: &ArtBody, rw: &M3) -> M3 {
    let ib = inertia_matrix(body.inertia);
    m_mul(&m_mul(rw, &ib), &m_transpose(rw))
}

// ---------------------------------------------------------------------------
// kinematics
// ---------------------------------------------------------------------------

/// Per-body world kinematics in the current configuration.
struct Kin {
    /// Joint origin (world).
    o: Vec<V3>,
    /// Joint axis (world, unit).
    a: Vec<V3>,
    /// Centre of mass (world).
    c: Vec<V3>,
    /// Body orientation (world).
    rw: Vec<M3>,
}

/// Compute world transforms for every body (tolerant of any parent ordering).
fn forward_kinematics(tree: &ArtBodyTree) -> Kin {
    let n = tree.bodies.len();
    let mut rw_opt: Vec<Option<M3>> = vec![None; n];
    let mut o: Vec<V3> = vec![[0.0; 3]; n];

    fn resolve(
        i: usize,
        tree: &ArtBodyTree,
        rw_opt: &mut Vec<Option<M3>>,
        o: &mut Vec<V3>,
    ) -> (M3, V3) {
        if let Some(r) = rw_opt[i] {
            return (r, o[i]);
        }
        let body = &tree.bodies[i];
        let q = tree.positions.get(i).copied().unwrap_or(0.0);
        let rj = rot_axis_angle(body.joint_axis, q);
        let (rwi, oi) = match body.parent {
            Some(p) if p < tree.bodies.len() && p != i => {
                let (rwp, op) = resolve(p, tree, rw_opt, o);
                let t = [tree.bodies[p].link_length, 0.0, 0.0];
                (m_mul(&rwp, &rj), v_add(op, m_vec(&rwp, t)))
            }
            _ => (rj, [0.0; 3]),
        };
        rw_opt[i] = Some(rwi);
        o[i] = oi;
        (rwi, oi)
    }

    for i in 0..n {
        let _ = resolve(i, tree, &mut rw_opt, &mut o);
    }

    let rw: Vec<M3> = rw_opt
        .into_iter()
        .map(|m| m.unwrap_or(m_identity()))
        .collect();
    let mut a = vec![[0.0f32; 3]; n];
    let mut c = vec![[0.0f32; 3]; n];
    for i in 0..n {
        a[i] = v_norm(m_vec(&rw[i], tree.bodies[i].joint_axis));
        c[i] = v_add(o[i], m_vec(&rw[i], tree.bodies[i].com_offset));
    }
    Kin { o, a, c, rw }
}

/// Joints affecting body `b`, from `b` up to its root (inclusive).
fn ancestors_inclusive(tree: &ArtBodyTree, b: usize) -> Vec<usize> {
    let mut chain = Vec::new();
    let mut cur = Some(b);
    while let Some(i) = cur {
        chain.push(i);
        cur = tree.bodies[i]
            .parent
            .filter(|&p| p < tree.bodies.len() && p != i);
    }
    chain
}

/// Topological order with every parent appearing before its children.
fn topo_order(tree: &ArtBodyTree) -> Vec<usize> {
    let n = tree.bodies.len();
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut order: Vec<usize> = Vec::with_capacity(n);
    for i in 0..n {
        match tree.bodies[i].parent {
            Some(p) if p < n && p != i => children[p].push(i),
            _ => order.push(i),
        }
    }
    let mut idx = 0;
    while idx < order.len() {
        let cur = order[idx];
        idx += 1;
        for &ch in &children[cur] {
            order.push(ch);
        }
    }
    order
}

// ---------------------------------------------------------------------------
// dynamics: mass matrix (CRBA-style) and bias forces (RNEA)
// ---------------------------------------------------------------------------

/// Assemble the joint-space inertia matrix `H(q)` via world-frame geometric
/// Jacobians. For each body `b`, the columns of its COM Jacobian are
/// `Jw_j = a_j`, `Jv_j = a_j × (c_b − o_j)` for every joint `j` on the path to
/// the root, and `H[j][k] += m_b (Jv_j·Jv_k) + Jw_j·(Iw_b Jw_k)`.
fn joint_space_inertia(tree: &ArtBodyTree, kin: &Kin) -> Vec<Vec<f32>> {
    let n = tree.bodies.len();
    let mut h = vec![vec![0.0f32; n]; n];
    for b in 0..n {
        let m = tree.bodies[b].mass;
        let iw = world_inertia(&tree.bodies[b], &kin.rw[b]);
        let chain = ancestors_inclusive(tree, b);
        for &j in &chain {
            let jw_j = kin.a[j];
            let jv_j = v_cross(kin.a[j], v_sub(kin.c[b], kin.o[j]));
            for &k in &chain {
                let jw_k = kin.a[k];
                let jv_k = v_cross(kin.a[k], v_sub(kin.c[b], kin.o[k]));
                h[j][k] += m * v_dot(jv_j, jv_k) + v_dot(jw_j, m_vec(&iw, jw_k));
            }
        }
    }
    h
}

/// Velocity-dependent bias forces `C(q,q̇)` (Coriolis + centrifugal) via the
/// Recursive Newton–Euler algorithm with `q̈ = 0` and gravity disabled.
fn bias_forces(tree: &ArtBodyTree, kin: &Kin) -> Vec<f32> {
    let n = tree.bodies.len();
    let order = topo_order(tree);

    let mut omega = vec![[0.0f32; 3]; n]; // angular velocity
    let mut alpha = vec![[0.0f32; 3]; n]; // angular acceleration
    let mut a_o = vec![[0.0f32; 3]; n]; // linear acceleration of joint origin

    // Forward pass (base → tip).
    for &i in &order {
        let qd = tree.velocities.get(i).copied().unwrap_or(0.0);
        let axis_rate = v_scale(kin.a[i], qd);
        match tree.bodies[i].parent {
            Some(p) if p < n && p != i => {
                omega[i] = v_add(omega[p], axis_rate);
                // q̈ = 0  ⇒  α_i = α_p + ω_p × (a_i q̇_i)
                alpha[i] = v_add(alpha[p], v_cross(omega[p], axis_rate));
                let r = v_sub(kin.o[i], kin.o[p]);
                // a_o_i = a_o_p + α_p × r + ω_p × (ω_p × r)
                a_o[i] = v_add(
                    a_o[p],
                    v_add(
                        v_cross(alpha[p], r),
                        v_cross(omega[p], v_cross(omega[p], r)),
                    ),
                );
            }
            _ => {
                omega[i] = axis_rate;
                alpha[i] = [0.0; 3];
                a_o[i] = [0.0; 3];
            }
        }
    }

    // Per-body net force / moment about the joint origin.
    let mut joint_force = vec![[0.0f32; 3]; n];
    let mut joint_moment = vec![[0.0f32; 3]; n];
    for &i in &order {
        let rc = v_sub(kin.c[i], kin.o[i]);
        // a_c = a_o + α × rc + ω × (ω × rc)
        let a_c = v_add(
            a_o[i],
            v_add(
                v_cross(alpha[i], rc),
                v_cross(omega[i], v_cross(omega[i], rc)),
            ),
        );
        let m = tree.bodies[i].mass;
        let f = v_scale(a_c, m);
        let iw = world_inertia(&tree.bodies[i], &kin.rw[i]);
        // Net moment about COM: Iw α + ω × (Iw ω).
        let tau_c = v_add(
            m_vec(&iw, alpha[i]),
            v_cross(omega[i], m_vec(&iw, omega[i])),
        );
        joint_force[i] = f;
        joint_moment[i] = v_add(tau_c, v_cross(rc, f));
    }

    // Backward pass (tip → base): accumulate subtree wrenches.
    for &i in order.iter().rev() {
        if let Some(p) = tree.bodies[i].parent {
            if p < n && p != i {
                let r = v_sub(kin.o[i], kin.o[p]);
                joint_force[p] = v_add(joint_force[p], joint_force[i]);
                joint_moment[p] = v_add(
                    joint_moment[p],
                    v_add(joint_moment[i], v_cross(r, joint_force[i])),
                );
            }
        }
    }

    // Generalised force for each revolute joint = axis · transmitted moment.
    (0..n).map(|i| v_dot(kin.a[i], joint_moment[i])).collect()
}

/// Solve `A x = b` by Gauss–Jordan elimination with partial pivoting.
/// Returns `None` if `A` is (numerically) singular.
fn solve_linear(mat: &[Vec<f32>], rhs: &[f32]) -> Option<Vec<f32>> {
    let n = rhs.len();
    if n == 0 {
        return Some(Vec::new());
    }
    let mut a: Vec<Vec<f32>> = mat.to_vec();
    let mut b = rhs.to_vec();
    for col in 0..n {
        let mut piv = col;
        let mut best = a[col][col].abs();
        for r in (col + 1)..n {
            if a[r][col].abs() > best {
                best = a[r][col].abs();
                piv = r;
            }
        }
        if best < 1e-12 {
            return None;
        }
        a.swap(col, piv);
        b.swap(col, piv);
        let d = a[col][col];
        for k in col..n {
            a[col][k] /= d;
        }
        b[col] /= d;
        for r in 0..n {
            if r != col {
                let factor = a[r][col];
                for k in col..n {
                    a[r][k] -= factor * a[col][k];
                }
                b[r] -= factor * b[col];
            }
        }
    }
    Some(b)
}

// ---------------------------------------------------------------------------
// public API
// ---------------------------------------------------------------------------

/// Perform one step of articulated-body forward dynamics.
///
/// Builds `H(q)` and `C(q,q̇)`, solves `H q̈ = τ − C`, and advances the joint
/// states with semi-implicit Euler (`q̇ += q̈·dt`, then `q += q̇·dt`). `tau`
/// supplies the actuator torque per joint (missing entries default to 0).
pub fn forward_dynamics_step(tree: &mut ArtBodyTree, tau: &[f32], dt: f32) {
    let n = tree.bodies.len();
    if n == 0 {
        return;
    }

    let kin = forward_kinematics(tree);
    let mut h = joint_space_inertia(tree, &kin);
    // Small Tikhonov term keeps degenerate inertias invertible.
    for i in 0..n {
        h[i][i] += 1e-9;
    }
    let bias = bias_forces(tree, &kin);

    let rhs: Vec<f32> = (0..n)
        .map(|i| tau.get(i).copied().unwrap_or(0.0) - bias[i])
        .collect();

    let qdd = solve_linear(&h, &rhs).unwrap_or_else(|| {
        // Degenerate inertia: fall back to decoupled per-joint acceleration.
        (0..n)
            .map(|i| (tau.get(i).copied().unwrap_or(0.0) - bias[i]) / h[i][i].max(1e-6))
            .collect()
    });

    for i in 0..n {
        tree.velocities[i] += qdd[i] * dt;
        tree.positions[i] += tree.velocities[i] * dt;
    }
}

/// Total kinetic energy `½ q̇ᵀ H(q) q̇` using the joint-space inertia matrix.
pub fn total_kinetic_energy(tree: &ArtBodyTree) -> f32 {
    let n = tree.bodies.len();
    if n == 0 {
        return 0.0;
    }
    let kin = forward_kinematics(tree);
    let h = joint_space_inertia(tree, &kin);
    let mut energy = 0.0f32;
    for i in 0..n {
        let qi = tree.velocities.get(i).copied().unwrap_or(0.0);
        for j in 0..n {
            let qj = tree.velocities.get(j).copied().unwrap_or(0.0);
            energy += 0.5 * qi * h[i][j] * qj;
        }
    }
    energy
}

/// Return the index of the root body (the first body with no parent).
pub fn find_root(tree: &ArtBodyTree) -> Option<usize> {
    tree.bodies.iter().position(|b| b.parent.is_none())
}

/// Count the number of leaf bodies (no body has them as a parent).
pub fn count_leaves(tree: &ArtBodyTree) -> usize {
    let parents: std::collections::HashSet<usize> =
        tree.bodies.iter().filter_map(|b| b.parent).collect();
    (0..tree.bodies.len())
        .filter(|i| !parents.contains(i))
        .count()
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_tree() -> ArtBodyTree {
        let mut t = ArtBodyTree::new();
        t.add_body(ArtBody {
            mass: 1.0,
            inertia: [0.1, 0.1, 0.1, 0.0, 0.0, 0.0],
            parent: None,
            joint_axis: [0.0, 1.0, 0.0],
            com_offset: [0.5, 0.0, 0.0],
            link_length: 1.0,
        });
        t.add_body(ArtBody {
            mass: 2.0,
            inertia: [0.1, 0.1, 0.1, 0.0, 0.0, 0.0],
            parent: Some(0),
            joint_axis: [0.0, 1.0, 0.0],
            com_offset: [0.5, 0.0, 0.0],
            link_length: 1.0,
        });
        t
    }

    #[test]
    fn test_body_count() {
        /* two bodies added */
        let t = simple_tree();
        assert_eq!(t.body_count(), 2);
    }

    #[test]
    fn test_find_root() {
        /* root has no parent */
        let t = simple_tree();
        assert_eq!(find_root(&t), Some(0));
    }

    #[test]
    fn test_count_leaves() {
        /* only body 1 is a leaf */
        let t = simple_tree();
        assert_eq!(count_leaves(&t), 1);
    }

    #[test]
    fn test_forward_dynamics_step() {
        /* a torque on the base joint accelerates the base joint positively */
        let mut t = simple_tree();
        forward_dynamics_step(&mut t, &[1.0, 0.0], 0.01);
        assert!(t.velocities[0] > 0.0);
    }

    #[test]
    fn test_kinetic_energy_zero_initially() {
        /* all velocities are zero initially */
        let t = simple_tree();
        assert_eq!(total_kinetic_energy(&t), 0.0);
    }

    #[test]
    fn test_kinetic_energy_nonzero_after_step() {
        /* energy increases after applying force */
        let mut t = simple_tree();
        forward_dynamics_step(&mut t, &[10.0, 10.0], 0.1);
        assert!(total_kinetic_energy(&t) > 0.0);
    }

    #[test]
    fn test_add_body_positions_len() {
        /* positions vector grows with each body */
        let t = simple_tree();
        assert_eq!(t.positions.len(), 2);
    }

    #[test]
    fn test_empty_tree_no_root() {
        /* empty tree has no root */
        let t = ArtBodyTree::new();
        assert!(find_root(&t).is_none());
    }

    #[test]
    fn test_empty_tree_no_leaves() {
        /* empty tree has no leaves */
        let t = ArtBodyTree::new();
        assert_eq!(count_leaves(&t), 0);
    }

    #[test]
    fn test_single_link_parallel_axis() {
        /* The joint-space inertia of a single revolute link about its pivot must
        equal Izz + m·d² (parallel-axis theorem), so a step from rest gives
        q̈ = τ / (Izz + m·d²). */
        let mut t = ArtBodyTree::new();
        let m = 3.0f32;
        let d = 0.5f32;
        let izz = 0.2f32;
        t.add_body(ArtBody {
            mass: m,
            inertia: [0.05, 0.05, izz, 0.0, 0.0, 0.0],
            parent: None,
            joint_axis: [0.0, 0.0, 1.0],
            com_offset: [d, 0.0, 0.0],
            link_length: 1.0,
        });
        let tau = 0.7f32;
        let dt = 0.01f32;
        forward_dynamics_step(&mut t, &[tau], dt);

        let expected_h = izz + m * d * d;
        let expected_qd = (tau / expected_h) * dt;
        assert!(
            (t.velocities[0] - expected_qd).abs() < 1e-4,
            "expected q̇ = {expected_qd}, got {}",
            t.velocities[0]
        );
    }

    #[test]
    fn test_single_link_kinetic_energy_matches_inertia() {
        /* For a single link spinning at q̇, KE = ½ (Izz + m·d²) q̇². */
        let mut t = ArtBodyTree::new();
        let (m, d, izz) = (2.0f32, 0.4f32, 0.15f32);
        t.add_body(ArtBody {
            mass: m,
            inertia: [0.05, 0.05, izz, 0.0, 0.0, 0.0],
            parent: None,
            joint_axis: [0.0, 0.0, 1.0],
            com_offset: [d, 0.0, 0.0],
            link_length: 1.0,
        });
        t.velocities[0] = 1.5;
        let h = izz + m * d * d;
        let expected = 0.5 * h * 1.5 * 1.5;
        assert!(
            (total_kinetic_energy(&t) - expected).abs() < 1e-4,
            "expected KE = {expected}, got {}",
            total_kinetic_energy(&t)
        );
    }

    #[test]
    fn test_positions_advance_after_step() {
        /* semi-implicit Euler advances joint angles once velocity is nonzero */
        let mut t = simple_tree();
        forward_dynamics_step(&mut t, &[1.0, 0.0], 0.05);
        assert!(t.positions[0].abs() > 0.0);
    }
}
