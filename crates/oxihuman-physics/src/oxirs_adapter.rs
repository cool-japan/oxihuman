// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! OxiRS simulation backend adapter.
//!
//! Bridges OxiHuman's body proxy types to a pure-Rust rigid body simulation
//! engine implemented here from scratch (no external physics crate required).
//!
//! ## Architecture
//!
//! ```text
//! BodyProxies ──► BodyRigMapper::build_from_proxies ──► OxiRsWorld
//!                                                           │
//!                    step(dt) ◄────── semi-implicit Euler   │
//!                    contact resolution ◄─ seq-impulse      │
//!                    BodyRigMapper::sync_transforms_to_proxies
//!                         │
//!                         ▼
//!                    BodyProxies (positions/transforms updated)
//! ```
//!
//! ## Integration
//!
//! The integrator is **semi-implicit (symplectic) Euler**:
//! ```text
//! v(t+dt) = v(t) + inv_mass * F(t) * dt
//! x(t+dt) = x(t) + v(t+dt) * dt
//! ```
//! with linear & angular damping applied multiplicatively each sub-step.
//!
//! Contacts are resolved with a **sequential impulse** (PGS) solver for
//! `solver_iterations` iterations per sub-step.

#![allow(clippy::too_many_arguments)]

use crate::BodyProxies;
use std::collections::HashMap;

// ── ColliderShape ─────────────────────────────────────────────────────────────

/// Collision shape for a rigid body.
#[derive(Debug, Clone, PartialEq)]
pub enum ColliderShape {
    /// Sphere with given radius.
    Sphere(f64),
    /// Capsule: radius, half-height of the cylindrical section.
    Capsule(f64, f64),
    /// Axis-aligned box half-extents (hx, hy, hz).
    Box(f64, f64, f64),
    /// Convex mesh placeholder (treated as sphere of radius 0.1 for collision).
    Mesh,
}

impl ColliderShape {
    /// Approximate bounding sphere radius for broad-phase culling.
    pub fn bounding_radius(&self) -> f64 {
        match self {
            ColliderShape::Sphere(r) => *r,
            ColliderShape::Capsule(r, h) => r + h,
            ColliderShape::Box(hx, hy, hz) => (hx * hx + hy * hy + hz * hz).sqrt(),
            ColliderShape::Mesh => 0.1,
        }
    }

    /// Compute diagonal inertia tensor components I_x, I_y, I_z for `mass`.
    pub fn inertia_diagonal(&self, mass: f64) -> [f64; 3] {
        match self {
            ColliderShape::Sphere(r) => {
                let i = 0.4 * mass * r * r;
                [i, i, i]
            }
            ColliderShape::Capsule(r, h) => {
                // Approximate as cylinder
                let r2 = r * r;
                let h2 = (2.0 * h) * (2.0 * h);
                let ix = mass * (3.0 * r2 + h2) / 12.0;
                let iy = mass * r2 / 2.0;
                [ix, iy, ix]
            }
            ColliderShape::Box(hx, hy, hz) => {
                let mx = mass * (4.0 * (hy * hy + hz * hz)) / 12.0;
                let my = mass * (4.0 * (hx * hx + hz * hz)) / 12.0;
                let mz = mass * (4.0 * (hx * hx + hy * hy)) / 12.0;
                [mx, my, mz]
            }
            ColliderShape::Mesh => {
                let i = mass * 0.01;
                [i, i, i]
            }
        }
    }
}

// ── RigidBodyDef ──────────────────────────────────────────────────────────────

/// Definition used to spawn a rigid body in [`OxiRsWorld`].
#[derive(Debug, Clone)]
pub struct RigidBodyDef {
    /// Collision shape.
    pub shape: ColliderShape,
    /// Mass in kg.  Use a large value (1e30) for effectively static bodies.
    pub mass: f64,
    /// Initial world position.
    pub position: [f64; 3],
    /// Initial orientation quaternion `[x, y, z, w]`.
    pub orientation: [f64; 4],
    /// If `true` the body is immovable (static ground plane, etc.).
    pub is_static: bool,
    /// Coefficient of restitution [0, 1].
    pub restitution: f64,
    /// Coulomb friction coefficient.
    pub friction: f64,
    /// Linear velocity damping factor per second (0 = no damping).
    pub linear_damping: f64,
    /// Angular velocity damping factor per second.
    pub angular_damping: f64,
}

impl Default for RigidBodyDef {
    fn default() -> Self {
        RigidBodyDef {
            shape: ColliderShape::Sphere(0.5),
            mass: 1.0,
            position: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
            is_static: false,
            restitution: 0.3,
            friction: 0.5,
            linear_damping: 0.01,
            angular_damping: 0.01,
        }
    }
}

// ── OxiRsConfig ───────────────────────────────────────────────────────────────

/// Global configuration for an [`OxiRsWorld`].
#[derive(Debug, Clone)]
pub struct OxiRsConfig {
    /// Gravity vector (m/s²).
    pub gravity: [f64; 3],
    /// Sequential-impulse iterations per sub-step.
    pub solver_iterations: u32,
    /// Physics sub-steps per `step(dt)` call.
    pub substeps: u32,
    /// Global time scale (1.0 = realtime).
    pub time_scale: f64,
    /// Allow bodies to go to sleep when kinetic energy is below threshold.
    pub enable_sleeping: bool,
    /// Kinetic energy threshold for sleeping (J).
    pub sleep_threshold: f64,
}

impl Default for OxiRsConfig {
    fn default() -> Self {
        OxiRsConfig {
            gravity: [0.0, -9.81, 0.0],
            solver_iterations: 10,
            substeps: 4,
            time_scale: 1.0,
            enable_sleeping: true,
            sleep_threshold: 0.01,
        }
    }
}

// ── BodyHandle ────────────────────────────────────────────────────────────────

/// Opaque handle returned by [`OxiRsWorld::add_body`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyHandle(u64);

// ── RayCastHit / ContactPair ──────────────────────────────────────────────────

/// Result of a successful ray cast.
#[derive(Debug, Clone)]
pub struct RayCastHit {
    /// Handle of the body that was hit.
    pub handle: BodyHandle,
    /// Distance from ray origin to hit point.
    pub distance: f64,
    /// World-space hit point.
    pub point: [f64; 3],
    /// World-space surface normal at hit.
    pub normal: [f64; 3],
}

/// A contact pair between two bodies.
#[derive(Debug, Clone)]
pub struct ContactPair {
    pub body_a: BodyHandle,
    pub body_b: BodyHandle,
    /// World-space contact point.
    pub point: [f64; 3],
    /// Contact normal (points from B toward A).
    pub normal: [f64; 3],
    /// Penetration depth (positive = overlapping).
    pub depth: f64,
}

// ── Internal body state ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct BodyState {
    handle: BodyHandle,
    shape: ColliderShape,
    mass: f64,
    inv_mass: f64,
    /// Diagonal inverse inertia tensor.
    inv_inertia: [f64; 3],
    position: [f64; 3],
    orientation: [f64; 4], // [x, y, z, w]
    velocity: [f64; 3],
    angular_velocity: [f64; 3],
    force_accum: [f64; 3],
    torque_accum: [f64; 3],
    is_static: bool,
    restitution: f64,
    friction: f64,
    linear_damping: f64,
    angular_damping: f64,
    sleeping: bool,
    sleep_timer: f64,
}

impl BodyState {
    fn from_def(handle: BodyHandle, def: &RigidBodyDef) -> Self {
        let (inv_mass, inv_inertia) = if def.is_static || def.mass <= 0.0 {
            (0.0, [0.0; 3])
        } else {
            let diag = def.shape.inertia_diagonal(def.mass);
            (
                1.0 / def.mass,
                [
                    if diag[0] > 0.0 { 1.0 / diag[0] } else { 0.0 },
                    if diag[1] > 0.0 { 1.0 / diag[1] } else { 0.0 },
                    if diag[2] > 0.0 { 1.0 / diag[2] } else { 0.0 },
                ],
            )
        };

        BodyState {
            handle,
            shape: def.shape.clone(),
            mass: def.mass,
            inv_mass,
            inv_inertia,
            position: def.position,
            orientation: def.orientation,
            velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            force_accum: [0.0; 3],
            torque_accum: [0.0; 3],
            is_static: def.is_static,
            restitution: def.restitution,
            friction: def.friction,
            linear_damping: def.linear_damping,
            angular_damping: def.angular_damping,
            sleeping: false,
            sleep_timer: 0.0,
        }
    }

    fn kinetic_energy(&self) -> f64 {
        if self.is_static {
            return 0.0;
        }
        let v = self.velocity;
        let w = self.angular_velocity;
        let lin = 0.5 * self.mass * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
        // Approximate rotational KE using diagonal inertia
        let inertia = self.shape.inertia_diagonal(self.mass);
        let rot =
            0.5 * (inertia[0] * w[0] * w[0] + inertia[1] * w[1] * w[1] + inertia[2] * w[2] * w[2]);
        lin + rot
    }
}

// ── Vector math helpers ───────────────────────────────────────────────────────

#[inline]
fn vec3_add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn vec3_sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_scale(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn vec3_dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_len(a: [f64; 3]) -> f64 {
    vec3_dot(a, a).sqrt()
}

#[inline]
fn vec3_normalize(a: [f64; 3]) -> Option<[f64; 3]> {
    let len = vec3_len(a);
    if len < 1e-12 {
        None
    } else {
        Some([a[0] / len, a[1] / len, a[2] / len])
    }
}

#[inline]
#[allow(dead_code)]
fn vec3_cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Normalise a quaternion [x, y, z, w].
fn quat_normalize(q: [f64; 4]) -> [f64; 4] {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len < 1e-12 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [q[0] / len, q[1] / len, q[2] / len, q[3] / len]
}

/// Integrate angular velocity into quaternion orientation.
///
/// Uses the first-order approximation:
/// `q += 0.5 * dt * [ω_x, ω_y, ω_z, 0] ⊗ q`
fn quat_integrate(q: [f64; 4], omega: [f64; 3], dt: f64) -> [f64; 4] {
    // omega quaternion (pure imaginary)
    let wx = omega[0];
    let wy = omega[1];
    let wz = omega[2];
    // q_dot = 0.5 * omega_q * q
    let dqx = 0.5 * dt * (wy * q[2] - wz * q[1] + wx * q[3]);
    let dqy = 0.5 * dt * (wz * q[0] - wx * q[2] + wy * q[3]);
    let dqz = 0.5 * dt * (wx * q[1] - wy * q[0] + wz * q[3]);
    let dqw = 0.5 * dt * (-wx * q[0] - wy * q[1] - wz * q[2]);
    quat_normalize([q[0] + dqx, q[1] + dqy, q[2] + dqz, q[3] + dqw])
}

// ── Contact detection helpers ─────────────────────────────────────────────────

/// Sphere vs sphere narrow-phase.
fn sphere_sphere_contact(
    pa: [f64; 3],
    ra: f64,
    handle_a: BodyHandle,
    pb: [f64; 3],
    rb: f64,
    handle_b: BodyHandle,
) -> Option<ContactPair> {
    let d = vec3_sub(pa, pb);
    let dist = vec3_len(d);
    let sum_r = ra + rb;
    if dist < sum_r && dist > 1e-12 {
        let normal = vec3_normalize(d)?;
        let depth = sum_r - dist;
        let point = vec3_add(pb, vec3_scale(normal, rb));
        Some(ContactPair {
            body_a: handle_a,
            body_b: handle_b,
            point,
            normal,
            depth,
        })
    } else {
        None
    }
}

/// Approximate capsule bounding sphere radius (used for broad-phase).
fn effective_sphere_radius(shape: &ColliderShape) -> f64 {
    shape.bounding_radius()
}

// ── OxiRsWorld ────────────────────────────────────────────────────────────────

/// The main rigid body simulation world.
///
/// Add bodies with [`Self::add_body`], advance time with [`Self::step`], query transforms
/// with [`Self::get_transform`], and read contacts with [`Self::contact_pairs`].
pub struct OxiRsWorld {
    config: OxiRsConfig,
    bodies: Vec<BodyState>,
    handle_to_idx: HashMap<BodyHandle, usize>,
    next_id: u64,
    /// Contacts detected during the last `step()`.
    last_contacts: Vec<ContactPair>,
}

impl OxiRsWorld {
    /// Create a new empty world with the given configuration.
    pub fn new(config: OxiRsConfig) -> Self {
        OxiRsWorld {
            config,
            bodies: Vec::new(),
            handle_to_idx: HashMap::new(),
            next_id: 1,
            last_contacts: Vec::new(),
        }
    }

    /// Create a world with default configuration.
    pub fn default_config() -> Self {
        Self::new(OxiRsConfig::default())
    }

    // ── Body management ───────────────────────────────────────────────────────

    /// Add a body to the world and return its handle.
    pub fn add_body(&mut self, def: RigidBodyDef) -> BodyHandle {
        let handle = BodyHandle(self.next_id);
        self.next_id += 1;
        let idx = self.bodies.len();
        self.bodies.push(BodyState::from_def(handle, &def));
        self.handle_to_idx.insert(handle, idx);
        handle
    }

    /// Remove a body from the world.  The handle becomes invalid after this.
    pub fn remove_body(&mut self, handle: BodyHandle) {
        if let Some(&idx) = self.handle_to_idx.get(&handle) {
            self.bodies.swap_remove(idx);
            // Repair the index map after swap_remove
            self.handle_to_idx.remove(&handle);
            if idx < self.bodies.len() {
                let moved_handle = self.bodies[idx].handle;
                self.handle_to_idx.insert(moved_handle, idx);
            }
        }
    }

    // ── Configuration ─────────────────────────────────────────────────────────

    /// Override the gravity vector.
    pub fn set_gravity(&mut self, g: [f64; 3]) {
        self.config.gravity = g;
    }

    /// Return a reference to the current configuration.
    pub fn config(&self) -> &OxiRsConfig {
        &self.config
    }

    // ── Query ─────────────────────────────────────────────────────────────────

    /// Get the current world transform of a body: `(position, orientation)`.
    ///
    /// Returns `None` if the handle is not valid.
    pub fn get_transform(&self, handle: BodyHandle) -> Option<([f64; 3], [f64; 4])> {
        let idx = self.handle_to_idx.get(&handle)?;
        let b = &self.bodies[*idx];
        Some((b.position, b.orientation))
    }

    /// Get the current linear velocity of a body.
    pub fn get_velocity(&self, handle: BodyHandle) -> Option<[f64; 3]> {
        let idx = self.handle_to_idx.get(&handle)?;
        Some(self.bodies[*idx].velocity)
    }

    /// Get the current angular velocity of a body.
    pub fn get_angular_velocity(&self, handle: BodyHandle) -> Option<[f64; 3]> {
        let idx = self.handle_to_idx.get(&handle)?;
        Some(self.bodies[*idx].angular_velocity)
    }

    /// Returns the number of bodies currently in the world.
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    // ── Force / impulse application ───────────────────────────────────────────

    /// Accumulate a force (N) on a body for the next integration step.
    ///
    /// Force is cleared after each sub-step.
    pub fn apply_force(&mut self, handle: BodyHandle, force: [f64; 3]) {
        if let Some(&idx) = self.handle_to_idx.get(&handle) {
            let b = &mut self.bodies[idx];
            if !b.is_static {
                b.force_accum = vec3_add(b.force_accum, force);
            }
        }
    }

    /// Apply an instantaneous impulse (kg·m/s) to a body, changing velocity
    /// immediately.
    pub fn apply_impulse(&mut self, handle: BodyHandle, impulse: [f64; 3]) {
        if let Some(&idx) = self.handle_to_idx.get(&handle) {
            let b = &mut self.bodies[idx];
            if !b.is_static {
                b.velocity[0] += impulse[0] * b.inv_mass;
                b.velocity[1] += impulse[1] * b.inv_mass;
                b.velocity[2] += impulse[2] * b.inv_mass;
                b.sleeping = false;
                b.sleep_timer = 0.0;
            }
        }
    }

    /// Apply a torque impulse directly to angular velocity.
    pub fn apply_torque_impulse(&mut self, handle: BodyHandle, torque_impulse: [f64; 3]) {
        if let Some(&idx) = self.handle_to_idx.get(&handle) {
            let b = &mut self.bodies[idx];
            if !b.is_static {
                b.angular_velocity[0] += torque_impulse[0] * b.inv_inertia[0];
                b.angular_velocity[1] += torque_impulse[1] * b.inv_inertia[1];
                b.angular_velocity[2] += torque_impulse[2] * b.inv_inertia[2];
                b.sleeping = false;
                b.sleep_timer = 0.0;
            }
        }
    }

    // ── Simulation step ───────────────────────────────────────────────────────

    /// Advance the simulation by `dt` seconds (wall-clock time before scaling).
    ///
    /// Internally divides into `config.substeps` sub-steps, each integrating
    /// forces and resolving contacts.
    pub fn step(&mut self, dt: f64) {
        let scaled_dt = dt * self.config.time_scale;
        let sub_dt = if self.config.substeps > 0 {
            scaled_dt / self.config.substeps as f64
        } else {
            scaled_dt
        };

        let substeps = self.config.substeps.max(1);
        for _ in 0..substeps {
            self.sub_step(sub_dt);
        }
    }

    /// Single sub-step: integrate + solve contacts.
    fn sub_step(&mut self, dt: f64) {
        // 1. Apply gravity and integrate velocities (semi-implicit Euler)
        self.integrate_velocities(dt);

        // 2. Detect contacts
        let contacts = self.detect_contacts();

        // 3. Sequential impulse solver
        let iterations = self.config.solver_iterations;
        for _ in 0..iterations {
            self.resolve_contacts(&contacts);
        }

        // 4. Integrate positions
        self.integrate_positions(dt);

        // 5. Update sleep states
        if self.config.enable_sleeping {
            self.update_sleep(dt);
        }

        // 6. Clear force accumulators
        for b in &mut self.bodies {
            b.force_accum = [0.0; 3];
            b.torque_accum = [0.0; 3];
        }

        self.last_contacts = contacts;
    }

    fn integrate_velocities(&mut self, dt: f64) {
        let g = self.config.gravity;
        for b in &mut self.bodies {
            if b.is_static || b.sleeping {
                continue;
            }
            // Gravity
            b.force_accum[0] += b.mass * g[0];
            b.force_accum[1] += b.mass * g[1];
            b.force_accum[2] += b.mass * g[2];

            // Linear
            b.velocity[0] += b.force_accum[0] * b.inv_mass * dt;
            b.velocity[1] += b.force_accum[1] * b.inv_mass * dt;
            b.velocity[2] += b.force_accum[2] * b.inv_mass * dt;

            // Angular
            b.angular_velocity[0] += b.torque_accum[0] * b.inv_inertia[0] * dt;
            b.angular_velocity[1] += b.torque_accum[1] * b.inv_inertia[1] * dt;
            b.angular_velocity[2] += b.torque_accum[2] * b.inv_inertia[2] * dt;

            // Damping (multiplicative)
            let ld = (1.0 - b.linear_damping * dt).max(0.0);
            let ad = (1.0 - b.angular_damping * dt).max(0.0);
            b.velocity = vec3_scale(b.velocity, ld);
            b.angular_velocity = vec3_scale(b.angular_velocity, ad);
        }
    }

    fn integrate_positions(&mut self, dt: f64) {
        for b in &mut self.bodies {
            if b.is_static || b.sleeping {
                continue;
            }
            b.position[0] += b.velocity[0] * dt;
            b.position[1] += b.velocity[1] * dt;
            b.position[2] += b.velocity[2] * dt;

            b.orientation = quat_integrate(b.orientation, b.angular_velocity, dt);
        }
    }

    fn update_sleep(&mut self, dt: f64) {
        let threshold = self.config.sleep_threshold;
        for b in &mut self.bodies {
            if b.is_static {
                continue;
            }
            let ke = b.kinetic_energy();
            if ke < threshold {
                b.sleep_timer += dt;
                if b.sleep_timer > 0.5 {
                    b.sleeping = true;
                }
            } else {
                b.sleeping = false;
                b.sleep_timer = 0.0;
            }
        }
    }

    // ── Contact detection ─────────────────────────────────────────────────────

    fn detect_contacts(&self) -> Vec<ContactPair> {
        let mut contacts = Vec::new();
        let n = self.bodies.len();

        for i in 0..n {
            for j in (i + 1)..n {
                let a = &self.bodies[i];
                let b = &self.bodies[j];

                // Skip static-static pairs
                if a.is_static && b.is_static {
                    continue;
                }

                let ra = effective_sphere_radius(&a.shape);
                let rb = effective_sphere_radius(&b.shape);

                if let Some(cp) =
                    sphere_sphere_contact(a.position, ra, a.handle, b.position, rb, b.handle)
                {
                    contacts.push(cp);
                }
            }
        }
        contacts
    }

    // ── Sequential impulse contact resolver ───────────────────────────────────

    fn resolve_contacts(&mut self, contacts: &[ContactPair]) {
        for cp in contacts {
            let (idx_a, idx_b) = match (
                self.handle_to_idx.get(&cp.body_a).copied(),
                self.handle_to_idx.get(&cp.body_b).copied(),
            ) {
                (Some(a), Some(b)) => (a, b),
                _ => continue,
            };

            // Borrow split: need mutable access to two distinct elements
            let (a_idx, b_idx) = (idx_a, idx_b);
            if a_idx == b_idx {
                continue;
            }

            // Position correction (Baumgarte)
            let slop = 0.01;
            let beta = 0.2;
            let correction = (cp.depth - slop).max(0.0) * beta;
            {
                let total_inv = self.bodies[a_idx].inv_mass + self.bodies[b_idx].inv_mass;
                if total_inv > 1e-12 {
                    let ca = correction * self.bodies[a_idx].inv_mass / total_inv;
                    let cb = correction * self.bodies[b_idx].inv_mass / total_inv;
                    if !self.bodies[a_idx].is_static {
                        self.bodies[a_idx].position[0] += cp.normal[0] * ca;
                        self.bodies[a_idx].position[1] += cp.normal[1] * ca;
                        self.bodies[a_idx].position[2] += cp.normal[2] * ca;
                    }
                    if !self.bodies[b_idx].is_static {
                        self.bodies[b_idx].position[0] -= cp.normal[0] * cb;
                        self.bodies[b_idx].position[1] -= cp.normal[1] * cb;
                        self.bodies[b_idx].position[2] -= cp.normal[2] * cb;
                    }
                }
            }

            // Velocity impulse
            let rel_vel = {
                let va = self.bodies[a_idx].velocity;
                let vb = self.bodies[b_idx].velocity;
                vec3_dot(vec3_sub(va, vb), cp.normal)
            };

            // Only resolve if approaching
            if rel_vel >= 0.0 {
                continue;
            }

            let e = self.bodies[a_idx]
                .restitution
                .min(self.bodies[b_idx].restitution);
            let inv_a = self.bodies[a_idx].inv_mass;
            let inv_b = self.bodies[b_idx].inv_mass;
            let denom = inv_a + inv_b;
            if denom < 1e-12 {
                continue;
            }
            let j_mag = -(1.0 + e) * rel_vel / denom;

            if !self.bodies[a_idx].is_static {
                self.bodies[a_idx].velocity[0] += cp.normal[0] * j_mag * inv_a;
                self.bodies[a_idx].velocity[1] += cp.normal[1] * j_mag * inv_a;
                self.bodies[a_idx].velocity[2] += cp.normal[2] * j_mag * inv_a;
            }
            if !self.bodies[b_idx].is_static {
                self.bodies[b_idx].velocity[0] -= cp.normal[0] * j_mag * inv_b;
                self.bodies[b_idx].velocity[1] -= cp.normal[1] * j_mag * inv_b;
                self.bodies[b_idx].velocity[2] -= cp.normal[2] * j_mag * inv_b;
            }

            // Friction impulse (Coulomb)
            let friction = self.bodies[a_idx].friction.min(self.bodies[b_idx].friction);
            let va = self.bodies[a_idx].velocity;
            let vb = self.bodies[b_idx].velocity;
            let tangent_vel = {
                let rv = vec3_sub(va, vb);
                let n_proj = vec3_scale(cp.normal, vec3_dot(rv, cp.normal));
                vec3_sub(rv, n_proj)
            };
            if let Some(t_dir) = vec3_normalize(tangent_vel) {
                let jt_num = -vec3_dot(vec3_sub(va, vb), t_dir);
                let jt = jt_num / denom;
                let jt_clamped = jt.clamp(-friction * j_mag, friction * j_mag);

                if !self.bodies[a_idx].is_static {
                    self.bodies[a_idx].velocity[0] += t_dir[0] * jt_clamped * inv_a;
                    self.bodies[a_idx].velocity[1] += t_dir[1] * jt_clamped * inv_a;
                    self.bodies[a_idx].velocity[2] += t_dir[2] * jt_clamped * inv_a;
                }
                if !self.bodies[b_idx].is_static {
                    self.bodies[b_idx].velocity[0] -= t_dir[0] * jt_clamped * inv_b;
                    self.bodies[b_idx].velocity[1] -= t_dir[1] * jt_clamped * inv_b;
                    self.bodies[b_idx].velocity[2] -= t_dir[2] * jt_clamped * inv_b;
                }
            }
        }
    }

    // ── Ray cast ──────────────────────────────────────────────────────────────

    /// Cast a ray through the world and return the closest hit, if any.
    ///
    /// Uses approximate sphere tests against each body's bounding sphere.
    pub fn ray_cast(&self, origin: [f64; 3], dir: [f64; 3], max_dist: f64) -> Option<RayCastHit> {
        let dir_n = vec3_normalize(dir)?;
        let mut best: Option<RayCastHit> = None;

        for b in &self.bodies {
            let r = effective_sphere_radius(&b.shape);
            if let Some(t) = ray_sphere_intersect(origin, dir_n, b.position, r) {
                if t >= 0.0 && t <= max_dist {
                    let point = vec3_add(origin, vec3_scale(dir_n, t));
                    let normal =
                        vec3_normalize(vec3_sub(point, b.position)).unwrap_or([0.0, 1.0, 0.0]);
                    let candidate = RayCastHit {
                        handle: b.handle,
                        distance: t,
                        point,
                        normal,
                    };
                    match &best {
                        None => best = Some(candidate),
                        Some(prev) if t < prev.distance => best = Some(candidate),
                        _ => {}
                    }
                }
            }
        }
        best
    }

    // ── Contact query ─────────────────────────────────────────────────────────

    /// Return all contact pairs detected during the last simulation step.
    pub fn contact_pairs(&self) -> &[ContactPair] {
        &self.last_contacts
    }
}

/// Ray vs sphere intersection.  Returns parameter `t` (distance along ray) or
/// `None` if no intersection.
fn ray_sphere_intersect(
    origin: [f64; 3],
    dir: [f64; 3],
    center: [f64; 3],
    radius: f64,
) -> Option<f64> {
    let oc = vec3_sub(origin, center);
    let a = vec3_dot(dir, dir);
    let b = 2.0 * vec3_dot(oc, dir);
    let c = vec3_dot(oc, oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }
    let sqrt_d = discriminant.sqrt();
    let t0 = (-b - sqrt_d) / (2.0 * a);
    let t1 = (-b + sqrt_d) / (2.0 * a);
    if t0 >= 0.0 {
        Some(t0)
    } else if t1 >= 0.0 {
        Some(t1)
    } else {
        None
    }
}

// ── BodyRigMapper ─────────────────────────────────────────────────────────────

/// Maps OxiHuman body proxies to/from an [`OxiRsWorld`].
///
/// Provides factory and sync utilities for bridging the humanoid proxy
/// representation with the rigid body simulation.
pub struct BodyRigMapper {
    /// Map from proxy index (capsule/sphere) to body handle.
    pub proxy_to_handle: Vec<BodyHandle>,
    /// Map from body handle to proxy label.
    pub handle_to_label: HashMap<BodyHandle, String>,
}

impl BodyRigMapper {
    /// Build an [`OxiRsWorld`] from a set of body proxies.
    ///
    /// Capsule proxies become `ColliderShape::Capsule` bodies; sphere proxies
    /// become `ColliderShape::Sphere` bodies; box proxies become
    /// `ColliderShape::Box` bodies.  All bodies start at their proxy center
    /// with default identity orientation.
    pub fn build_from_proxies(proxies: &BodyProxies, config: &OxiRsConfig) -> (OxiRsWorld, Self) {
        let mut world = OxiRsWorld::new(config.clone());
        let mut proxy_to_handle = Vec::new();
        let mut handle_to_label = HashMap::new();

        // Capsules
        for cap in &proxies.capsules {
            let mid = midpoint(cap.center_a, cap.center_b);
            let half_h = half_height(cap.center_a, cap.center_b);
            let r = cap.radius as f64;
            let h = half_h as f64;
            let def = RigidBodyDef {
                shape: ColliderShape::Capsule(r, h),
                mass: capsule_mass(r, h),
                position: [mid[0] as f64, mid[1] as f64, mid[2] as f64],
                orientation: [0.0, 0.0, 0.0, 1.0],
                is_static: false,
                restitution: 0.1,
                friction: 0.6,
                linear_damping: 0.05,
                angular_damping: 0.05,
            };
            let handle = world.add_body(def);
            proxy_to_handle.push(handle);
            handle_to_label.insert(handle, cap.label.clone());
        }

        // Spheres
        for sph in &proxies.spheres {
            let r = sph.radius as f64;
            let def = RigidBodyDef {
                shape: ColliderShape::Sphere(r),
                mass: sphere_mass(r),
                position: [
                    sph.center[0] as f64,
                    sph.center[1] as f64,
                    sph.center[2] as f64,
                ],
                orientation: [0.0, 0.0, 0.0, 1.0],
                is_static: false,
                restitution: 0.1,
                friction: 0.5,
                linear_damping: 0.05,
                angular_damping: 0.05,
            };
            let handle = world.add_body(def);
            proxy_to_handle.push(handle);
            handle_to_label.insert(handle, sph.label.clone());
        }

        // Boxes
        for bx in &proxies.boxes {
            let he = bx.half_extents;
            let def = RigidBodyDef {
                shape: ColliderShape::Box(he[0] as f64, he[1] as f64, he[2] as f64),
                mass: box_mass(he),
                position: [
                    bx.center[0] as f64,
                    bx.center[1] as f64,
                    bx.center[2] as f64,
                ],
                orientation: [0.0, 0.0, 0.0, 1.0],
                is_static: false,
                restitution: 0.1,
                friction: 0.5,
                linear_damping: 0.05,
                angular_damping: 0.05,
            };
            let handle = world.add_body(def);
            proxy_to_handle.push(handle);
            handle_to_label.insert(handle, bx.label.clone());
        }

        let mapper = BodyRigMapper {
            proxy_to_handle,
            handle_to_label,
        };
        (world, mapper)
    }

    /// Synchronize world transforms back to the body proxies.
    ///
    /// For capsules the midpoint position is updated; for spheres and boxes
    /// the center is updated.  Orientation is not stored in the proxy structs,
    /// so only translation is synced.
    pub fn sync_transforms_to_proxies(&self, world: &OxiRsWorld, proxies: &mut BodyProxies) {
        let cap_len = proxies.capsules.len();
        let sph_len = proxies.spheres.len();

        for (local_idx, &handle) in self.proxy_to_handle.iter().enumerate() {
            if let Some((pos, _orient)) = world.get_transform(handle) {
                if local_idx < cap_len {
                    let c = &mut proxies.capsules[local_idx];
                    let half = half_height(c.center_a, c.center_b);
                    // Recenter capsule around new simulation position (axis stays vertical)
                    c.center_a = [pos[0] as f32, pos[1] as f32 - half, pos[2] as f32];
                    c.center_b = [pos[0] as f32, pos[1] as f32 + half, pos[2] as f32];
                } else if local_idx < cap_len + sph_len {
                    let s = &mut proxies.spheres[local_idx - cap_len];
                    s.center = [pos[0] as f32, pos[1] as f32, pos[2] as f32];
                } else {
                    let b_idx = local_idx - cap_len - sph_len;
                    if b_idx < proxies.boxes.len() {
                        let bx = &mut proxies.boxes[b_idx];
                        bx.center = [pos[0] as f32, pos[1] as f32, pos[2] as f32];
                    }
                }
            }
        }
    }
}

// ── Proxy mass helpers ────────────────────────────────────────────────────────

/// Approximate uniform-density sphere mass (density 1000 kg/m³).
fn sphere_mass(r: f64) -> f64 {
    let density = 1000.0;
    (4.0 / 3.0) * std::f64::consts::PI * r * r * r * density
}

/// Approximate capsule mass.
fn capsule_mass(r: f64, h: f64) -> f64 {
    let density = 1000.0;
    let cyl_vol = std::f64::consts::PI * r * r * (2.0 * h);
    let sph_vol = (4.0 / 3.0) * std::f64::consts::PI * r * r * r;
    (cyl_vol + sph_vol) * density
}

/// Approximate box mass.
fn box_mass(half_extents: [f32; 3]) -> f64 {
    let density = 1000.0;
    let vol = 8.0 * half_extents[0] as f64 * half_extents[1] as f64 * half_extents[2] as f64;
    vol * density
}

fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

fn half_height(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt() * 0.5
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BoxProxy, CapsuleProxy, SphereProxy};

    // ── Helper builders ───────────────────────────────────────────────────────

    fn unit_sphere_def(pos: [f64; 3]) -> RigidBodyDef {
        RigidBodyDef {
            shape: ColliderShape::Sphere(0.5),
            mass: 1.0,
            position: pos,
            ..Default::default()
        }
    }

    fn static_ground_def() -> RigidBodyDef {
        RigidBodyDef {
            shape: ColliderShape::Sphere(100.0),
            mass: 1.0,
            position: [0.0, -101.0, 0.0],
            is_static: true,
            ..Default::default()
        }
    }

    // ── World / handle tests ──────────────────────────────────────────────────

    #[test]
    fn test_add_body_increments_count() {
        let mut world = OxiRsWorld::default_config();
        assert_eq!(world.body_count(), 0);
        world.add_body(unit_sphere_def([0.0, 0.0, 0.0]));
        assert_eq!(world.body_count(), 1);
        world.add_body(unit_sphere_def([2.0, 0.0, 0.0]));
        assert_eq!(world.body_count(), 2);
    }

    #[test]
    fn test_remove_body_decrements_count() {
        let mut world = OxiRsWorld::default_config();
        let h = world.add_body(unit_sphere_def([0.0, 0.0, 0.0]));
        world.add_body(unit_sphere_def([2.0, 0.0, 0.0]));
        world.remove_body(h);
        assert_eq!(world.body_count(), 1);
    }

    #[test]
    fn test_get_transform_initial_position() {
        let mut world = OxiRsWorld::default_config();
        let h = world.add_body(unit_sphere_def([1.0, 2.0, 3.0]));
        let (pos, _orient) = world.get_transform(h).expect("transform should exist");
        assert!((pos[0] - 1.0).abs() < 1e-9);
        assert!((pos[1] - 2.0).abs() < 1e-9);
        assert!((pos[2] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_get_transform_invalid_handle_returns_none() {
        let world = OxiRsWorld::default_config();
        assert!(world.get_transform(BodyHandle(9999)).is_none());
    }

    #[test]
    fn test_remove_invalid_handle_noop() {
        let mut world = OxiRsWorld::default_config();
        world.remove_body(BodyHandle(9999)); // must not panic
        assert_eq!(world.body_count(), 0);
    }

    // ── Gravity / integration tests ───────────────────────────────────────────

    #[test]
    fn test_gravity_pulls_body_down() {
        let mut world = OxiRsWorld::new(OxiRsConfig {
            substeps: 1,
            solver_iterations: 0,
            ..Default::default()
        });
        let h = world.add_body(unit_sphere_def([0.0, 10.0, 0.0]));
        let dt = 0.1;
        world.step(dt);
        let (pos, _) = world.get_transform(h).expect("exists");
        assert!(pos[1] < 10.0, "body should have fallen, y={}", pos[1]);
    }

    #[test]
    fn test_static_body_does_not_move_under_gravity() {
        let mut world = OxiRsWorld::new(OxiRsConfig {
            substeps: 1,
            ..Default::default()
        });
        let h = world.add_body(static_ground_def());
        world.step(1.0);
        let (pos, _) = world.get_transform(h).expect("exists");
        assert!((pos[1] + 101.0).abs() < 1e-9, "static body must not move");
    }

    #[test]
    fn test_zero_gravity_body_stays_still() {
        let mut world = OxiRsWorld::new(OxiRsConfig {
            gravity: [0.0, 0.0, 0.0],
            substeps: 1,
            solver_iterations: 0,
            ..Default::default()
        });
        let h = world.add_body(unit_sphere_def([5.0, 5.0, 5.0]));
        world.step(1.0);
        let (pos, _) = world.get_transform(h).expect("exists");
        assert!((pos[0] - 5.0).abs() < 1e-6, "no x drift");
        assert!((pos[1] - 5.0).abs() < 1e-6, "no y drift");
        assert!((pos[2] - 5.0).abs() < 1e-6, "no z drift");
    }

    #[test]
    fn test_apply_impulse_changes_velocity() {
        let mut world = OxiRsWorld::new(OxiRsConfig {
            gravity: [0.0, 0.0, 0.0],
            substeps: 1,
            solver_iterations: 0,
            ..Default::default()
        });
        let h = world.add_body(unit_sphere_def([0.0, 0.0, 0.0]));
        world.apply_impulse(h, [10.0, 0.0, 0.0]);
        let v = world.get_velocity(h).expect("velocity");
        assert!(
            v[0] > 0.0,
            "impulse should set positive x velocity, got {}",
            v[0]
        );
    }

    #[test]
    fn test_apply_force_then_step_moves_body() {
        let mut world = OxiRsWorld::new(OxiRsConfig {
            gravity: [0.0, 0.0, 0.0],
            substeps: 1,
            solver_iterations: 0,
            ..Default::default()
        });
        let h = world.add_body(unit_sphere_def([0.0, 0.0, 0.0]));
        world.apply_force(h, [100.0, 0.0, 0.0]);
        world.step(0.1);
        let (pos, _) = world.get_transform(h).expect("pos");
        assert!(pos[0] > 0.0, "body moved in +x direction");
    }

    // ── Collision / contact tests ─────────────────────────────────────────────

    #[test]
    fn test_two_spheres_collide_and_separate() {
        let mut world = OxiRsWorld::new(OxiRsConfig {
            gravity: [0.0, 0.0, 0.0],
            substeps: 4,
            solver_iterations: 10,
            enable_sleeping: false,
            ..Default::default()
        });
        // Place two unit spheres close together (overlapping)
        let ha = world.add_body(unit_sphere_def([-0.4, 0.0, 0.0]));
        let hb = world.add_body(unit_sphere_def([0.4, 0.0, 0.0]));

        // Give them opposing impulses
        world.apply_impulse(ha, [-5.0, 0.0, 0.0]);
        world.apply_impulse(hb, [5.0, 0.0, 0.0]);

        world.step(0.05);

        let (pa, _) = world.get_transform(ha).expect("a");
        let (pb, _) = world.get_transform(hb).expect("b");
        let dist =
            ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
        // After collision + separation impulses they should be moving apart
        assert!(dist >= 0.8, "spheres should not overlap much, dist={dist}");
    }

    #[test]
    fn test_contact_pairs_populated_after_step() {
        let mut world = OxiRsWorld::new(OxiRsConfig {
            gravity: [0.0, 0.0, 0.0],
            substeps: 1,
            solver_iterations: 0,
            enable_sleeping: false,
            ..Default::default()
        });
        // Heavily overlapping spheres
        world.add_body(unit_sphere_def([0.0, 0.0, 0.0]));
        world.add_body(unit_sphere_def([0.5, 0.0, 0.0]));
        world.step(0.016);
        assert!(
            !world.contact_pairs().is_empty(),
            "overlapping spheres should produce contacts"
        );
    }

    // ── Ray cast tests ────────────────────────────────────────────────────────

    #[test]
    fn test_ray_hits_sphere() {
        let mut world = OxiRsWorld::default_config();
        world.add_body(unit_sphere_def([0.0, 0.0, 5.0]));
        let hit = world.ray_cast([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 100.0);
        assert!(hit.is_some(), "ray along +z should hit sphere at z=5");
    }

    #[test]
    fn test_ray_misses_sphere() {
        let mut world = OxiRsWorld::default_config();
        world.add_body(unit_sphere_def([0.0, 0.0, 5.0]));
        let hit = world.ray_cast([10.0, 0.0, 0.0], [0.0, 0.0, 1.0], 100.0);
        assert!(
            hit.is_none(),
            "ray offset by 10 units should miss sphere of r=0.5"
        );
    }

    #[test]
    fn test_ray_max_dist_respected() {
        let mut world = OxiRsWorld::default_config();
        world.add_body(unit_sphere_def([0.0, 0.0, 50.0]));
        let hit = world.ray_cast([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 10.0);
        assert!(hit.is_none(), "sphere at z=50 should be beyond max_dist=10");
    }

    #[test]
    fn test_ray_zero_dir_returns_none() {
        let world = OxiRsWorld::default_config();
        let hit = world.ray_cast([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 100.0);
        assert!(hit.is_none(), "zero direction should return None");
    }

    // ── Set gravity ───────────────────────────────────────────────────────────

    #[test]
    fn test_set_gravity_changes_config() {
        let mut world = OxiRsWorld::default_config();
        world.set_gravity([0.0, 1.0, 0.0]);
        assert!((world.config().gravity[1] - 1.0).abs() < 1e-9);
    }

    // ── Sleeping ──────────────────────────────────────────────────────────────

    #[test]
    fn test_body_wakes_on_impulse() {
        let mut world = OxiRsWorld::new(OxiRsConfig {
            gravity: [0.0, 0.0, 0.0],
            enable_sleeping: true,
            sleep_threshold: 1e9, // force sleep immediately
            substeps: 1,
            solver_iterations: 0,
            ..Default::default()
        });
        let h = world.add_body(unit_sphere_def([0.0, 0.0, 0.0]));
        // Step until sleeping
        for _ in 0..100 {
            world.step(0.016);
        }
        // Apply impulse — body should wake
        world.apply_impulse(h, [10.0, 0.0, 0.0]);
        // Velocity should now be nonzero
        let v = world.get_velocity(h).expect("velocity");
        assert!(
            v[0].abs() > 0.0,
            "body should have velocity after impulse wake"
        );
    }

    // ── BodyRigMapper tests ───────────────────────────────────────────────────

    #[test]
    fn test_build_from_proxies_correct_count() {
        let mut proxies = BodyProxies::new();
        proxies.capsules.push(CapsuleProxy::new(
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            0.2,
            "torso",
        ));
        proxies
            .spheres
            .push(SphereProxy::new([0.0, 1.2, 0.0], 0.12, "head"));

        let config = OxiRsConfig::default();
        let (world, mapper) = BodyRigMapper::build_from_proxies(&proxies, &config);
        assert_eq!(world.body_count(), 2, "one capsule + one sphere = 2 bodies");
        assert_eq!(mapper.proxy_to_handle.len(), 2);
    }

    #[test]
    fn test_sync_transforms_updates_proxy_positions() {
        let mut proxies = BodyProxies::new();
        proxies.capsules.push(CapsuleProxy::new(
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            0.2,
            "leg",
        ));

        let config = OxiRsConfig {
            gravity: [0.0, -9.81, 0.0],
            substeps: 4,
            ..Default::default()
        };
        let (mut world, mapper) = BodyRigMapper::build_from_proxies(&proxies, &config);
        world.step(0.1);
        let y_before = proxies.capsules[0].center_a[1];
        mapper.sync_transforms_to_proxies(&world, &mut proxies);
        let y_after = proxies.capsules[0].center_a[1];
        assert!(
            y_after < y_before,
            "capsule proxy should have fallen: before={y_before}, after={y_after}"
        );
    }

    #[test]
    fn test_collider_shape_bounding_radius() {
        let s = ColliderShape::Sphere(2.0);
        assert!((s.bounding_radius() - 2.0).abs() < 1e-9);
        let c = ColliderShape::Capsule(0.3, 0.8);
        assert!((c.bounding_radius() - 1.1).abs() < 1e-9);
        let b = ColliderShape::Box(1.0, 2.0, 2.0);
        assert!((b.bounding_radius() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_inertia_diagonal_sphere() {
        let s = ColliderShape::Sphere(1.0);
        let i = s.inertia_diagonal(1.0);
        // I = 0.4 * m * r^2 = 0.4
        assert!((i[0] - 0.4).abs() < 1e-9);
        assert!((i[1] - 0.4).abs() < 1e-9);
        assert!((i[2] - 0.4).abs() < 1e-9);
    }

    #[test]
    fn test_box_proxy_builds_correctly() {
        let mut proxies = BodyProxies::new();
        proxies.boxes.push(BoxProxy {
            center: [0.0, 0.5, 0.0],
            half_extents: [0.5, 0.5, 0.5],
            label: "block".to_string(),
        });
        let config = OxiRsConfig::default();
        let (world, _mapper) = BodyRigMapper::build_from_proxies(&proxies, &config);
        assert_eq!(world.body_count(), 1);
    }

    #[test]
    fn test_multiple_bodies_independent_integration() {
        let mut world = OxiRsWorld::new(OxiRsConfig {
            gravity: [0.0, -9.81, 0.0],
            substeps: 1,
            solver_iterations: 0,
            enable_sleeping: false,
            ..Default::default()
        });
        let h0 = world.add_body(unit_sphere_def([0.0, 100.0, 0.0]));
        let h1 = world.add_body(unit_sphere_def([10.0, 100.0, 0.0]));
        world.step(0.5);
        let (p0, _) = world.get_transform(h0).expect("p0");
        let (p1, _) = world.get_transform(h1).expect("p1");
        // Both should fall by same amount in y, differ only in x
        assert!((p0[0]).abs() < 1e-6, "first body stays at x=0");
        assert!((p1[0] - 10.0).abs() < 1e-6, "second body stays at x=10");
        assert!((p0[1] - p1[1]).abs() < 1e-3, "both fall equal distance");
    }

    #[test]
    fn test_time_scale_zero_no_movement() {
        let mut world = OxiRsWorld::new(OxiRsConfig {
            gravity: [0.0, -9.81, 0.0],
            time_scale: 0.0,
            substeps: 1,
            solver_iterations: 0,
            ..Default::default()
        });
        let h = world.add_body(unit_sphere_def([0.0, 5.0, 0.0]));
        world.step(1.0);
        let (pos, _) = world.get_transform(h).expect("pos");
        assert!(
            (pos[1] - 5.0).abs() < 1e-9,
            "time_scale=0 must not move body"
        );
    }
}
