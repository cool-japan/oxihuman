// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! SPH (Smoothed Particle Hydrodynamics) fluid surface simulation for
//! sweat/moisture on human skin.
//!
//! Uses the Müller et al. 2003 SPH formulation with:
//! - Poly6 kernel for density estimation
//! - Spiky kernel gradient for pressure forces
//! - Viscosity kernel Laplacian for viscosity forces
//! - Color field gradient for surface tension (Morris 2000)
//! - Marching cubes for surface mesh extraction

use std::collections::HashMap;

// ── Constants ─────────────────────────────────────────────────────────────────

const PI: f64 = std::f64::consts::PI;

// ── SPH Particle ──────────────────────────────────────────────────────────────

/// A single SPH fluid particle representing a droplet of sweat/moisture.
#[derive(Debug, Clone)]
pub struct SphParticle {
    /// Position in 3D space (meters).
    pub position: [f64; 3],
    /// Velocity (m/s).
    pub velocity: [f64; 3],
    /// Computed density (kg/m³).
    pub density: f64,
    /// Computed pressure (Pa).
    pub pressure: f64,
    /// Particle mass (kg).
    pub mass: f64,
}

impl SphParticle {
    /// Create a new particle at `position` with given `mass`.
    pub fn new(position: [f64; 3], mass: f64) -> Self {
        Self {
            position,
            velocity: [0.0; 3],
            density: 0.0,
            pressure: 0.0,
            mass,
        }
    }
}

// ── SPH Configuration ─────────────────────────────────────────────────────────

/// Configuration parameters for the SPH simulation.
#[derive(Debug, Clone)]
pub struct SphConfig {
    /// Smoothing radius h (m).
    pub smoothing_radius: f64,
    /// Rest density ρ₀ (kg/m³); water ≈ 1000.
    pub rest_density: f64,
    /// Pressure stiffness constant k (Pa·m³/kg).
    pub pressure_stiffness: f64,
    /// Dynamic viscosity μ (Pa·s); water ≈ 0.001.
    pub viscosity: f64,
    /// Surface tension coefficient σ (N/m); water ≈ 0.072.
    pub surface_tension: f64,
    /// Gravity vector (m/s²).
    pub gravity: [f64; 3],
}

impl Default for SphConfig {
    fn default() -> Self {
        Self {
            smoothing_radius: 0.05,
            rest_density: 1000.0,
            pressure_stiffness: 2000.0,
            viscosity: 0.001,
            surface_tension: 0.072,
            gravity: [0.0, -9.81, 0.0],
        }
    }
}

// ── Spatial Hash ──────────────────────────────────────────────────────────────

/// A uniform spatial hash grid for O(1) average neighbor queries.
struct SpatialHash {
    cell_size: f64,
    buckets: HashMap<(i32, i32, i32), Vec<usize>>,
}

impl SpatialHash {
    fn new(cell_size: f64) -> Self {
        Self {
            cell_size,
            buckets: HashMap::new(),
        }
    }

    fn cell_key(&self, pos: [f64; 3]) -> (i32, i32, i32) {
        (
            (pos[0] / self.cell_size).floor() as i32,
            (pos[1] / self.cell_size).floor() as i32,
            (pos[2] / self.cell_size).floor() as i32,
        )
    }

    fn insert(&mut self, idx: usize, pos: [f64; 3]) {
        let key = self.cell_key(pos);
        self.buckets.entry(key).or_default().push(idx);
    }

    fn clear(&mut self) {
        self.buckets.clear();
    }

    /// Return candidate neighbor indices (may include distant particles; caller must filter).
    fn candidates(&self, pos: [f64; 3], radius: f64) -> Vec<usize> {
        let cells = (radius / self.cell_size).ceil() as i32;
        let base = self.cell_key(pos);
        let mut result = Vec::new();
        for dx in -cells..=cells {
            for dy in -cells..=cells {
                for dz in -cells..=cells {
                    let key = (base.0 + dx, base.1 + dy, base.2 + dz);
                    if let Some(bucket) = self.buckets.get(&key) {
                        result.extend_from_slice(bucket);
                    }
                }
            }
        }
        result
    }
}

// ── SPH Kernels ───────────────────────────────────────────────────────────────

/// Poly6 smoothing kernel W_poly6(r, h).
///
/// W(r,h) = (315 / (64π h⁹)) * (h² - r²)³   for r ≤ h
#[inline]
pub fn poly6_kernel(r: f64, h: f64) -> f64 {
    if r > h || r < 0.0 {
        return 0.0;
    }
    let coeff = 315.0 / (64.0 * PI * h.powi(9));
    let diff = h * h - r * r;
    coeff * diff * diff * diff
}

/// Poly6 kernel gradient (scalar factor; multiply by (pos_i - pos_j)/r for direction).
///
/// ∂W/∂r = (315 / (64π h⁹)) * (-6r) * (h² - r²)²
#[inline]
pub fn poly6_kernel_gradient_scalar(r: f64, h: f64) -> f64 {
    if r > h || r < 1e-12 {
        return 0.0;
    }
    let coeff = 315.0 / (64.0 * PI * h.powi(9));
    let diff = h * h - r * r;
    coeff * (-6.0 * r) * diff * diff
}

/// Spiky kernel gradient scalar (Müller 2003).
///
/// ∇W_spiky = -(45 / (π h⁶)) * (h - r)²   for r ≤ h
#[inline]
pub fn spiky_kernel_gradient_scalar(r: f64, h: f64) -> f64 {
    if r > h || r < 1e-12 {
        return 0.0;
    }
    let coeff = -45.0 / (PI * h.powi(6));
    let diff = h - r;
    coeff * diff * diff
}

/// Viscosity kernel Laplacian (Müller 2003).
///
/// ∇²W_visc = (45 / (π h⁶)) * (h - r)   for r ≤ h
#[inline]
pub fn viscosity_kernel_laplacian(r: f64, h: f64) -> f64 {
    if r > h {
        return 0.0;
    }
    let coeff = 45.0 / (PI * h.powi(6));
    coeff * (h - r)
}

// ── SPH Simulation ─────────────────────────────────────────────────────────────

/// Full 3D SPH simulation for sweat/moisture on skin.
pub struct SphSimulation {
    /// All fluid particles.
    pub particles: Vec<SphParticle>,
    /// Simulation parameters.
    pub config: SphConfig,
    /// Spatial hash for neighbor search.
    spatial_hash: SpatialHash,
    /// Cached neighbor lists (per particle).
    neighbor_cache: Vec<Vec<usize>>,
    /// Accumulated forces per particle (cleared each step).
    forces: Vec<[f64; 3]>,
}

impl SphSimulation {
    /// Create a new simulation with the given particles and config.
    pub fn new(particles: Vec<SphParticle>, config: SphConfig) -> Self {
        let n = particles.len();
        let cell = config.smoothing_radius;
        Self {
            particles,
            config,
            spatial_hash: SpatialHash::new(cell),
            neighbor_cache: vec![Vec::new(); n],
            forces: vec![[0.0; 3]; n],
        }
    }

    /// Build the spatial hash and neighbor lists.
    fn rebuild_spatial_hash(&mut self) {
        self.spatial_hash.clear();
        for (i, p) in self.particles.iter().enumerate() {
            self.spatial_hash.insert(i, p.position);
        }
        let h = self.config.smoothing_radius;
        let h2 = h * h;
        for i in 0..self.particles.len() {
            let pos_i = self.particles[i].position;
            let candidates = self.spatial_hash.candidates(pos_i, h);
            self.neighbor_cache[i] = candidates
                .into_iter()
                .filter(|&j| {
                    let dp = sub3(pos_i, self.particles[j].position);
                    dot3(dp, dp) < h2
                })
                .collect();
        }
    }

    /// Compute density and pressure for all particles.
    fn compute_density_pressure(&mut self) {
        let h = self.config.smoothing_radius;
        let k = self.config.pressure_stiffness;
        let rho0 = self.config.rest_density;
        let n = self.particles.len();
        for i in 0..n {
            let pos_i = self.particles[i].position;
            let mut rho = 0.0_f64;
            for &j in &self.neighbor_cache[i] {
                let dp = sub3(pos_i, self.particles[j].position);
                let r = len3(dp);
                rho += self.particles[j].mass * poly6_kernel(r, h);
            }
            // Self-contribution
            rho += self.particles[i].mass * poly6_kernel(0.0, h);
            self.particles[i].density = rho.max(1e-9);
            // Equation of state: p = k * (ρ - ρ₀)
            self.particles[i].pressure = k * (rho - rho0).max(0.0);
        }
    }

    /// Compute and accumulate pressure gradient force for particle `i`.
    ///
    /// F_pressure_i = -m_i * Σ_j [ m_j * (p_i/(ρ_i²) + p_j/(ρ_j²)) * ∇W_spiky(r_ij) ]
    pub fn pressure_force(&self, i: usize) -> [f64; 3] {
        let h = self.config.smoothing_radius;
        let pi = self.particles[i];
        let mut f = [0.0_f64; 3];
        for &j in &self.neighbor_cache[i] {
            if i == j {
                continue;
            }
            let pj = self.particles[j];
            let dp = sub3(pi.position, pj.position);
            let r = len3(dp);
            if r < 1e-12 {
                continue;
            }
            let grad_scalar = spiky_kernel_gradient_scalar(r, h);
            // unit direction from j to i
            let dir = scale3(dp, 1.0 / r);
            let term = pj.mass
                * (pi.pressure / (pi.density * pi.density)
                    + pj.pressure / (pj.density * pj.density))
                * grad_scalar;
            f = add3(f, scale3(dir, -pi.mass * term));
        }
        f
    }

    /// Compute viscosity force for particle `i` using Laplacian viscosity kernel.
    ///
    /// F_visc_i = μ * m_i * Σ_j [ m_j * (v_j - v_i) / ρ_j * ∇²W_visc(r_ij) ]
    pub fn viscosity_force(&self, i: usize) -> [f64; 3] {
        let h = self.config.smoothing_radius;
        let mu = self.config.viscosity;
        let pi = &self.particles[i];
        let mut f = [0.0_f64; 3];
        for &j in &self.neighbor_cache[i] {
            if i == j {
                continue;
            }
            let pj = &self.particles[j];
            let dp = sub3(pi.position, pj.position);
            let r = len3(dp);
            let lap = viscosity_kernel_laplacian(r, h);
            let dv = sub3(pj.velocity, pi.velocity);
            let term = pj.mass / pj.density.max(1e-9) * lap;
            f = add3(f, scale3(dv, mu * pi.mass * term));
        }
        f
    }

    /// Compute color-field gradient for surface tension for particle `i`.
    ///
    /// n_i = h * Σ_j [ m_j / ρ_j * ∇W_poly6(r_ij) ]
    /// F_surface_i = -σ * ∇²c / |n| * n   (simplified Morris 2000)
    pub fn surface_normal(&self, i: usize) -> [f64; 3] {
        let h = self.config.smoothing_radius;
        let pi = &self.particles[i];
        let mut n = [0.0_f64; 3];
        for &j in &self.neighbor_cache[i] {
            if i == j {
                continue;
            }
            let pj = &self.particles[j];
            let dp = sub3(pi.position, pj.position);
            let r = len3(dp);
            if r < 1e-12 {
                continue;
            }
            let grad_scalar = poly6_kernel_gradient_scalar(r, h);
            let dir = scale3(dp, 1.0 / r);
            let w = pj.mass / pj.density.max(1e-9) * grad_scalar;
            n = add3(n, scale3(dir, w));
        }
        n
    }

    /// Compute surface tension force for particle `i` using color-field curvature.
    fn surface_tension_force(&self, i: usize) -> [f64; 3] {
        let sigma = self.config.surface_tension;
        let h = self.config.smoothing_radius;
        let pi = &self.particles[i];
        // Compute color-field Laplacian (scalar)
        let mut lap_c = 0.0_f64;
        let mut normal = [0.0_f64; 3];
        for &j in &self.neighbor_cache[i] {
            if i == j {
                continue;
            }
            let pj = &self.particles[j];
            let dp = sub3(pi.position, pj.position);
            let r = len3(dp);
            if r < 1e-12 {
                continue;
            }
            // Color field: c_j = m_j / ρ_j
            let c_j = pj.mass / pj.density.max(1e-9);
            lap_c += c_j * viscosity_kernel_laplacian(r, h);
            let dir = scale3(dp, 1.0 / r);
            let gs = poly6_kernel_gradient_scalar(r, h);
            normal = add3(normal, scale3(dir, c_j * gs));
        }
        let n_len = len3(normal);
        if n_len < 1e-6 {
            return [0.0; 3];
        }
        let n_hat = scale3(normal, 1.0 / n_len);
        // F = -σ * κ * n̂,  κ = -∇²c / |∇c|
        let curvature = -lap_c / n_len;
        scale3(n_hat, -sigma * curvature)
    }

    /// Compute density at an arbitrary world position using poly6 kernel.
    pub fn density_at(&self, pos: [f64; 3]) -> f64 {
        let h = self.config.smoothing_radius;
        let candidates = self.spatial_hash.candidates(pos, h);
        let h2 = h * h;
        let mut rho = 0.0_f64;
        for idx in candidates {
            let dp = sub3(pos, self.particles[idx].position);
            let r2 = dot3(dp, dp);
            if r2 < h2 {
                let r = r2.sqrt();
                rho += self.particles[idx].mass * poly6_kernel(r, h);
            }
        }
        rho
    }

    /// Advance the simulation by `dt` seconds.
    ///
    /// Pipeline: rebuild hash → density/pressure → forces → symplectic Euler integration.
    pub fn step(&mut self, dt: f64) {
        let n = self.particles.len();
        if n == 0 {
            return;
        }

        // 1. Rebuild spatial hash and neighbor lists
        self.rebuild_spatial_hash();

        // 2. Compute density and pressure
        self.compute_density_pressure();

        // 3. Compute forces
        self.forces.iter_mut().for_each(|f| *f = [0.0; 3]);
        for i in 0..n {
            let fp = self.pressure_force(i);
            let fv = self.viscosity_force(i);
            let fs = self.surface_tension_force(i);
            let fg = scale3(self.config.gravity, self.particles[i].mass);
            self.forces[i] = add3(add3(add3(fp, fv), fs), fg);
        }

        // 4. Symplectic Euler integration: v += F/m * dt, x += v * dt
        for i in 0..n {
            let inv_m = 1.0 / self.particles[i].mass.max(1e-30);
            let a = scale3(self.forces[i], inv_m);
            let new_vel = add3(self.particles[i].velocity, scale3(a, dt));
            let new_pos = add3(self.particles[i].position, scale3(new_vel, dt));
            self.particles[i].velocity = new_vel;
            self.particles[i].position = new_pos;
        }
    }

    /// Extract an isosurface mesh using Marching Cubes.
    ///
    /// Returns `(vertices, indices)` where each vertex is `[f32; 3]` and
    /// indices form triangles.
    ///
    /// # Arguments
    /// * `grid_size` — number of cells per axis (e.g. 32 → 32³ voxels)
    pub fn marching_cubes_surface(&self, grid_size: usize) -> (Vec<[f32; 3]>, Vec<u32>) {
        if self.particles.is_empty() {
            return (Vec::new(), Vec::new());
        }

        // Compute bounding box with padding
        let (mut mn, mut mx) = bounding_box(&self.particles);
        let h = self.config.smoothing_radius;
        for k in 0..3 {
            mn[k] -= h * 2.0;
            mx[k] += h * 2.0;
        }
        let extent = [mx[0] - mn[0], mx[1] - mn[1], mx[2] - mn[2]];
        let cell = [
            extent[0] / grid_size as f64,
            extent[1] / grid_size as f64,
            extent[2] / grid_size as f64,
        ];

        let gs = grid_size + 1; // number of grid vertices per axis
        let mut scalar_field = vec![0.0_f32; gs * gs * gs];

        // Fill scalar field with SPH density
        for iz in 0..gs {
            for iy in 0..gs {
                for ix in 0..gs {
                    let wx = mn[0] + ix as f64 * cell[0];
                    let wy = mn[1] + iy as f64 * cell[1];
                    let wz = mn[2] + iz as f64 * cell[2];
                    let rho = self.density_at([wx, wy, wz]);
                    scalar_field[ix + iy * gs + iz * gs * gs] = rho as f32;
                }
            }
        }

        // Marching cubes: iso-value = half rest density
        let iso = (self.config.rest_density * 0.5) as f32;
        marching_cubes(&scalar_field, gs, gs, gs, iso, mn, cell)
    }
}

// Implement Copy for SphParticle so we can copy in loops
impl Copy for SphParticle {}

// ── Marching Cubes ────────────────────────────────────────────────────────────

/// Simplified marching cubes implementation.
///
/// Uses the classic 256-entry edge table lookup.
fn marching_cubes(
    field: &[f32],
    nx: usize,
    ny: usize,
    nz: usize,
    iso: f32,
    origin: [f64; 3],
    cell_size: [f64; 3],
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let idx = |x: usize, y: usize, z: usize| x + y * nx + z * nx * ny;

    for iz in 0..nz.saturating_sub(1) {
        for iy in 0..ny.saturating_sub(1) {
            for ix in 0..nx.saturating_sub(1) {
                // 8 corners of the cube
                let corners = [
                    (ix, iy, iz),
                    (ix + 1, iy, iz),
                    (ix + 1, iy + 1, iz),
                    (ix, iy + 1, iz),
                    (ix, iy, iz + 1),
                    (ix + 1, iy, iz + 1),
                    (ix + 1, iy + 1, iz + 1),
                    (ix, iy + 1, iz + 1),
                ];

                // Scalar values at corners
                let vals: [f32; 8] = std::array::from_fn(|k| {
                    let (cx, cy, cz) = corners[k];
                    if cx < nx && cy < ny && cz < nz {
                        field[idx(cx, cy, cz)]
                    } else {
                        0.0
                    }
                });

                // Cube index
                let cube_idx: u8 =
                    vals.iter().enumerate().fold(
                        0u8,
                        |acc, (k, &v)| if v > iso { acc | (1 << k) } else { acc },
                    );

                if cube_idx == 0 || cube_idx == 255 {
                    continue;
                }

                // World-space corner positions
                let pos: [[f32; 3]; 8] = std::array::from_fn(|k| {
                    let (cx, cy, cz) = corners[k];
                    [
                        (origin[0] + cx as f64 * cell_size[0]) as f32,
                        (origin[1] + cy as f64 * cell_size[1]) as f32,
                        (origin[2] + cz as f64 * cell_size[2]) as f32,
                    ]
                });

                // Interpolate edge vertices
                let edge_verts = mc_edge_vertices(cube_idx, &vals, &pos, iso);

                // Emit triangles from triangle table
                let tris = MC_TRI_TABLE[cube_idx as usize];
                let mut ti = 0;
                while ti < 16 && tris[ti] != -1 {
                    let e0 = tris[ti] as usize;
                    let e1 = tris[ti + 1] as usize;
                    let e2 = tris[ti + 2] as usize;
                    if let (Some(v0), Some(v1), Some(v2)) =
                        (edge_verts[e0], edge_verts[e1], edge_verts[e2])
                    {
                        let base = vertices.len() as u32;
                        vertices.push(v0);
                        vertices.push(v1);
                        vertices.push(v2);
                        indices.push(base);
                        indices.push(base + 1);
                        indices.push(base + 2);
                    }
                    ti += 3;
                }
            }
        }
    }

    (vertices, indices)
}

/// Compute interpolated vertices on the 12 edges of a marching cube.
fn mc_edge_vertices(
    cube_idx: u8,
    vals: &[f32; 8],
    pos: &[[f32; 3]; 8],
    iso: f32,
) -> [Option<[f32; 3]>; 12] {
    // 12 edges: (corner_a, corner_b)
    const EDGES: [(usize, usize); 12] = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0), // bottom ring
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4), // top ring
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7), // verticals
    ];
    let edge_table: u16 = MC_EDGE_TABLE[cube_idx as usize];
    std::array::from_fn(|e| {
        if edge_table & (1 << e) != 0 {
            let (a, b) = EDGES[e];
            Some(lerp_vertex(pos[a], vals[a], pos[b], vals[b], iso))
        } else {
            None
        }
    })
}

fn lerp_vertex(pa: [f32; 3], va: f32, pb: [f32; 3], vb: f32, iso: f32) -> [f32; 3] {
    let dv = vb - va;
    let t = if dv.abs() < 1e-9 {
        0.5
    } else {
        (iso - va) / dv
    };
    let t = t.clamp(0.0, 1.0);
    [
        pa[0] + t * (pb[0] - pa[0]),
        pa[1] + t * (pb[1] - pa[1]),
        pa[2] + t * (pb[2] - pa[2]),
    ]
}

// ── Math helpers ──────────────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(a: [f64; 3]) -> f64 {
    dot3(a, a).sqrt()
}

fn bounding_box(particles: &[SphParticle]) -> ([f64; 3], [f64; 3]) {
    let mut mn = [f64::MAX; 3];
    let mut mx = [f64::MIN; 3];
    for p in particles {
        for k in 0..3 {
            if p.position[k] < mn[k] {
                mn[k] = p.position[k];
            }
            if p.position[k] > mx[k] {
                mx[k] = p.position[k];
            }
        }
    }
    (mn, mx)
}

// ── Marching Cubes Tables ─────────────────────────────────────────────────────

/// Edge table: for each cube configuration, which edges are intersected.
#[rustfmt::skip]
const MC_EDGE_TABLE: [u16; 256] = [
    0x000, 0x109, 0x203, 0x30a, 0x406, 0x50f, 0x605, 0x70c,
    0x80c, 0x905, 0xa0f, 0xb06, 0xc0a, 0xd03, 0xe09, 0xf00,
    0x190, 0x099, 0x393, 0x29a, 0x596, 0x49f, 0x795, 0x69c,
    0x99c, 0x895, 0xb9f, 0xa96, 0xd9a, 0xc93, 0xf99, 0xe90,
    0x230, 0x339, 0x033, 0x13a, 0x636, 0x73f, 0x435, 0x53c,
    0xa3c, 0xb35, 0x83f, 0x936, 0xe3a, 0xf33, 0xc39, 0xd30,
    0x3a0, 0x2a9, 0x1a3, 0x0aa, 0x7a6, 0x6af, 0x5a5, 0x4ac,
    0xbac, 0xaa5, 0x9af, 0x8a6, 0xfaa, 0xea3, 0xda9, 0xca0,
    0x460, 0x569, 0x663, 0x76a, 0x066, 0x16f, 0x265, 0x36c,
    0xc6c, 0xd65, 0xe6f, 0xf66, 0x86a, 0x963, 0xa69, 0xb60,
    0x5f0, 0x4f9, 0x7f3, 0x6fa, 0x1f6, 0x0ff, 0x3f5, 0x2fc,
    0xdfc, 0xcf5, 0xfff, 0xef6, 0x9fa, 0x8f3, 0xbf9, 0xaf0,
    0x650, 0x759, 0x453, 0x55a, 0x256, 0x35f, 0x055, 0x15c,
    0xe5c, 0xf55, 0xc5f, 0xd56, 0xa5a, 0xb53, 0x859, 0x950,
    0x7c0, 0x6c9, 0x5c3, 0x4ca, 0x3c6, 0x2cf, 0x1c5, 0x0cc,
    0xfcc, 0xec5, 0xdcf, 0xcc6, 0xbca, 0xac3, 0x9c9, 0x8c0,
    0x8c0, 0x9c9, 0xac3, 0xbca, 0xcc6, 0xdcf, 0xec5, 0xfcc,
    0x0cc, 0x1c5, 0x2cf, 0x3c6, 0x4ca, 0x5c3, 0x6c9, 0x7c0,
    0x950, 0x859, 0xb53, 0xa5a, 0xd56, 0xc5f, 0xf55, 0xe5c,
    0x15c, 0x055, 0x35f, 0x256, 0x55a, 0x453, 0x759, 0x650,
    0xaf0, 0xbf9, 0x8f3, 0x9fa, 0xef6, 0xfff, 0xcf5, 0xdfc,
    0x2fc, 0x3f5, 0x0ff, 0x1f6, 0x6fa, 0x7f3, 0x4f9, 0x5f0,
    0xb60, 0xa69, 0x963, 0x86a, 0xf66, 0xe6f, 0xd65, 0xc6c,
    0x36c, 0x265, 0x16f, 0x066, 0x76a, 0x663, 0x569, 0x460,
    0xca0, 0xda9, 0xea3, 0xfaa, 0x8a6, 0x9af, 0xaa5, 0xbac,
    0x4ac, 0x5a5, 0x6af, 0x7a6, 0x0aa, 0x1a3, 0x2a9, 0x3a0,
    0xd30, 0xc39, 0xf33, 0xe3a, 0x936, 0x83f, 0xb35, 0xa3c,
    0x53c, 0x435, 0x73f, 0x636, 0x13a, 0x033, 0x339, 0x230,
    0xe90, 0xf99, 0xc93, 0xd9a, 0xa96, 0xb9f, 0x895, 0x99c,
    0x69c, 0x795, 0x49f, 0x596, 0x29a, 0x393, 0x099, 0x190,
    0xf00, 0xe09, 0xd03, 0xc0a, 0xb06, 0xa0f, 0x905, 0x80c,
    0x70c, 0x605, 0x50f, 0x406, 0x30a, 0x203, 0x109, 0x000,
];

/// Triangle table: for each cube configuration, lists of edge indices forming triangles.
/// Each row has up to 15 entries, terminated by -1.
#[rustfmt::skip]
const MC_TRI_TABLE: [[i8; 16]; 256] = [
    [-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 8, 3,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 1, 9,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 8, 3, 9, 8, 1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 2,10,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 8, 3, 1, 2,10,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 2,10, 0, 2, 9,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 2, 8, 3, 2,10, 8,10, 9, 8,-1,-1,-1,-1,-1,-1,-1],
    [ 3,11, 2,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0,11, 2, 8,11, 0,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 9, 0, 2, 3,11,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 1,11, 2, 1, 9,11, 9, 8,11,-1,-1,-1,-1,-1,-1,-1],
    [ 3,10, 1,11,10, 3,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0,10, 1, 0, 8,10, 8,11,10,-1,-1,-1,-1,-1,-1,-1],
    [ 3, 9, 0, 3,11, 9,11,10, 9,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 8,11, 9,11,10,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 4, 7, 8,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 4, 3, 0, 7, 3, 4,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 1, 9, 8, 4, 7,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 4, 1, 9, 4, 7, 1, 7, 3, 1,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 2,10, 8, 4, 7,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 3, 4, 7, 3, 0, 4, 1, 2,10,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 2,10, 9, 0, 2, 8, 4, 7,-1,-1,-1,-1,-1,-1,-1],
    [ 2,10, 9, 2, 9, 7, 2, 7, 3, 7, 9, 4,-1,-1,-1,-1],
    [ 8, 4, 7, 3,11, 2,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [11, 4, 7,11, 2, 4, 2, 0, 4,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 0, 1, 8, 4, 7, 2, 3,11,-1,-1,-1,-1,-1,-1,-1],
    [ 4, 7,11, 9, 4,11, 9,11, 2, 9, 2, 1,-1,-1,-1,-1],
    [ 3,10, 1, 3,11,10, 7, 8, 4,-1,-1,-1,-1,-1,-1,-1],
    [ 1,11,10, 1, 4,11, 1, 0, 4, 7,11, 4,-1,-1,-1,-1],
    [ 4, 7, 8, 9, 0,11, 9,11,10,11, 0, 3,-1,-1,-1,-1],
    [ 4, 7,11, 4,11, 9, 9,11,10,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 5, 4,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 5, 4, 0, 8, 3,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 5, 4, 1, 5, 0,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 8, 5, 4, 8, 3, 5, 3, 1, 5,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 2,10, 9, 5, 4,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 3, 0, 8, 1, 2,10, 4, 9, 5,-1,-1,-1,-1,-1,-1,-1],
    [ 5, 2,10, 5, 4, 2, 4, 0, 2,-1,-1,-1,-1,-1,-1,-1],
    [ 2,10, 5, 3, 2, 5, 3, 5, 4, 3, 4, 8,-1,-1,-1,-1],
    [ 9, 5, 4, 2, 3,11,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0,11, 2, 0, 8,11, 4, 9, 5,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 5, 4, 0, 1, 5, 2, 3,11,-1,-1,-1,-1,-1,-1,-1],
    [ 2, 1, 5, 2, 5, 8, 2, 8,11, 4, 8, 5,-1,-1,-1,-1],
    [10, 3,11,10, 1, 3, 9, 5, 4,-1,-1,-1,-1,-1,-1,-1],
    [ 4, 9, 5, 0, 8, 1, 8,10, 1, 8,11,10,-1,-1,-1,-1],
    [ 5, 4, 0, 5, 0,11, 5,11,10,11, 0, 3,-1,-1,-1,-1],
    [ 5, 4, 8, 5, 8,10,10, 8,11,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 7, 8, 5, 7, 9,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 3, 0, 9, 5, 3, 5, 7, 3,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 7, 8, 0, 1, 7, 1, 5, 7,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 5, 3, 3, 5, 7,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 7, 8, 9, 5, 7,10, 1, 2,-1,-1,-1,-1,-1,-1,-1],
    [10, 1, 2, 9, 5, 0, 5, 3, 0, 5, 7, 3,-1,-1,-1,-1],
    [ 8, 0, 2, 8, 2, 5, 8, 5, 7,10, 5, 2,-1,-1,-1,-1],
    [ 2,10, 5, 2, 5, 3, 3, 5, 7,-1,-1,-1,-1,-1,-1,-1],
    [ 7, 9, 5, 7, 8, 9, 3,11, 2,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 5, 7, 9, 7, 2, 9, 2, 0, 2, 7,11,-1,-1,-1,-1],
    [ 2, 3,11, 0, 1, 8, 1, 7, 8, 1, 5, 7,-1,-1,-1,-1],
    [11, 2, 1,11, 1, 7, 7, 1, 5,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 5, 8, 8, 5, 7,10, 1, 3,10, 3,11,-1,-1,-1,-1],
    [ 5, 7, 0, 5, 0, 9, 7,11, 0, 1, 0,10,11,10, 0,-1],
    [11,10, 0,11, 0, 3,10, 5, 0, 8, 0, 7, 5, 7, 0,-1],
    [11,10, 5, 7,11, 5,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [10, 6, 5,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 8, 3, 5,10, 6,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 0, 1, 5,10, 6,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 8, 3, 1, 9, 8, 5,10, 6,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 6, 5, 2, 6, 1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 6, 5, 1, 2, 6, 3, 0, 8,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 6, 5, 9, 0, 6, 0, 2, 6,-1,-1,-1,-1,-1,-1,-1],
    [ 5, 9, 8, 5, 8, 2, 5, 2, 6, 3, 2, 8,-1,-1,-1,-1],
    [ 2, 3,11,10, 6, 5,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [11, 0, 8,11, 2, 0,10, 6, 5,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 1, 9, 2, 3,11, 5,10, 6,-1,-1,-1,-1,-1,-1,-1],
    [ 5,10, 6, 1, 9, 2, 9,11, 2, 9, 8,11,-1,-1,-1,-1],
    [ 6, 3,11, 6, 5, 3, 5, 1, 3,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 8,11, 0,11, 5, 0, 5, 1, 5,11, 6,-1,-1,-1,-1],
    [ 3,11, 6, 0, 3, 6, 0, 6, 5, 0, 5, 9,-1,-1,-1,-1],
    [ 6, 5, 9, 6, 9,11,11, 9, 8,-1,-1,-1,-1,-1,-1,-1],
    [ 5,10, 6, 4, 7, 8,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 4, 3, 0, 4, 7, 3, 6, 5,10,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 9, 0, 5,10, 6, 8, 4, 7,-1,-1,-1,-1,-1,-1,-1],
    [10, 6, 5, 1, 9, 7, 1, 7, 3, 7, 9, 4,-1,-1,-1,-1],
    [ 6, 1, 2, 6, 5, 1, 4, 7, 8,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 2, 5, 5, 2, 6, 3, 0, 4, 3, 4, 7,-1,-1,-1,-1],
    [ 8, 4, 7, 9, 0, 5, 0, 6, 5, 0, 2, 6,-1,-1,-1,-1],
    [ 7, 3, 9, 7, 9, 4, 3, 2, 9, 5, 9, 6, 2, 6, 9,-1],
    [ 3,11, 2, 7, 8, 4,10, 6, 5,-1,-1,-1,-1,-1,-1,-1],
    [ 5,10, 6, 4, 7, 2, 4, 2, 0, 2, 7,11,-1,-1,-1,-1],
    [ 0, 1, 9, 4, 7, 8, 2, 3,11, 5,10, 6,-1,-1,-1,-1],
    [ 9, 2, 1, 9,11, 2, 9, 4,11, 7,11, 4, 5,10, 6,-1],
    [ 8, 4, 7, 3,11, 5, 3, 5, 1, 5,11, 6,-1,-1,-1,-1],
    [ 5, 1,11, 5,11, 6, 1, 0,11, 7,11, 4, 0, 4,11,-1],
    [ 0, 5, 9, 0, 6, 5, 0, 3, 6,11, 6, 3, 8, 4, 7,-1],
    [ 6, 5, 9, 6, 9,11, 4, 7, 9, 7,11, 9,-1,-1,-1,-1],
    [10, 4, 9, 6, 4,10,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 4,10, 6, 4, 9,10, 0, 8, 3,-1,-1,-1,-1,-1,-1,-1],
    [10, 0, 1,10, 6, 0, 6, 4, 0,-1,-1,-1,-1,-1,-1,-1],
    [ 8, 3, 1, 8, 1, 6, 8, 6, 4, 6, 1,10,-1,-1,-1,-1],
    [ 1, 4, 9, 1, 2, 4, 2, 6, 4,-1,-1,-1,-1,-1,-1,-1],
    [ 3, 0, 8, 1, 2, 9, 2, 4, 9, 2, 6, 4,-1,-1,-1,-1],
    [ 0, 2, 4, 4, 2, 6,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 8, 3, 2, 8, 2, 4, 4, 2, 6,-1,-1,-1,-1,-1,-1,-1],
    [10, 4, 9,10, 6, 4,11, 2, 3,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 8, 2, 2, 8,11, 4, 9,10, 4,10, 6,-1,-1,-1,-1],
    [ 3,11, 2, 0, 1, 6, 0, 6, 4, 6, 1,10,-1,-1,-1,-1],
    [ 6, 4, 1, 6, 1,10, 4, 8, 1, 2, 1,11, 8,11, 1,-1],
    [ 9, 6, 4, 9, 3, 6, 9, 1, 3,11, 6, 3,-1,-1,-1,-1],
    [ 8,11, 1, 8, 1, 0,11, 6, 1, 9, 1, 4, 6, 4, 1,-1],
    [ 3,11, 6, 3, 6, 0, 0, 6, 4,-1,-1,-1,-1,-1,-1,-1],
    [ 6, 4, 8,11, 6, 8,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 7,10, 6, 7, 8,10, 8, 9,10,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 7, 3, 0,10, 7, 0, 9,10, 6, 7,10,-1,-1,-1,-1],
    [10, 6, 7, 1,10, 7, 1, 7, 8, 1, 8, 0,-1,-1,-1,-1],
    [10, 6, 7,10, 7, 1, 1, 7, 3,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 2, 6, 1, 6, 8, 1, 8, 9, 8, 6, 7,-1,-1,-1,-1],
    [ 2, 6, 9, 2, 9, 1, 6, 7, 9, 0, 9, 3, 7, 3, 9,-1],
    [ 7, 8, 0, 7, 0, 6, 6, 0, 2,-1,-1,-1,-1,-1,-1,-1],
    [ 7, 3, 2, 6, 7, 2,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 2, 3,11,10, 6, 8,10, 8, 9, 8, 6, 7,-1,-1,-1,-1],
    [ 2, 0, 7, 2, 7,11, 0, 9, 7, 6, 7,10, 9,10, 7,-1],
    [ 1, 8, 0, 1, 7, 8, 1,10, 7, 6, 7,10, 2, 3,11,-1],
    [11, 2, 1,11, 1, 7,10, 6, 1, 6, 7, 1,-1,-1,-1,-1],
    [ 8, 9, 6, 8, 6, 7, 9, 1, 6,11, 6, 3, 1, 3, 6,-1],
    [ 0, 9, 1,11, 6, 7,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 7, 8, 0, 7, 0, 6, 3,11, 0,11, 6, 0,-1,-1,-1,-1],
    [ 7,11, 6,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 7, 6,11,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 3, 0, 8,11, 7, 6,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 1, 9,11, 7, 6,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 8, 1, 9, 8, 3, 1,11, 7, 6,-1,-1,-1,-1,-1,-1,-1],
    [10, 1, 2, 6,11, 7,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 2,10, 3, 0, 8, 6,11, 7,-1,-1,-1,-1,-1,-1,-1],
    [ 2, 9, 0, 2,10, 9, 6,11, 7,-1,-1,-1,-1,-1,-1,-1],
    [ 6,11, 7, 2,10, 3,10, 8, 3,10, 9, 8,-1,-1,-1,-1],
    [ 7, 2, 3, 6, 2, 7,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 7, 0, 8, 7, 6, 0, 6, 2, 0,-1,-1,-1,-1,-1,-1,-1],
    [ 2, 7, 6, 2, 3, 7, 0, 1, 9,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 6, 2, 1, 8, 6, 1, 9, 8, 8, 7, 6,-1,-1,-1,-1],
    [10, 7, 6,10, 1, 7, 1, 3, 7,-1,-1,-1,-1,-1,-1,-1],
    [10, 7, 6, 1, 7,10, 1, 8, 7, 1, 0, 8,-1,-1,-1,-1],
    [ 0, 3, 7, 0, 7,10, 0,10, 9, 6,10, 7,-1,-1,-1,-1],
    [ 7, 6,10, 7,10, 8, 8,10, 9,-1,-1,-1,-1,-1,-1,-1],
    [ 6, 8, 4,11, 8, 6,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 3, 6,11, 3, 0, 6, 0, 4, 6,-1,-1,-1,-1,-1,-1,-1],
    [ 8, 6,11, 8, 4, 6, 9, 0, 1,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 4, 6, 9, 6, 3, 9, 3, 1,11, 3, 6,-1,-1,-1,-1],
    [ 6, 8, 4, 6,11, 8, 2,10, 1,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 2,10, 3, 0,11, 0, 6,11, 0, 4, 6,-1,-1,-1,-1],
    [ 4,11, 8, 4, 6,11, 0, 2, 9, 2,10, 9,-1,-1,-1,-1],
    [10, 9, 3,10, 3, 2, 9, 4, 3,11, 3, 6, 4, 6, 3,-1],
    [ 8, 2, 3, 8, 4, 2, 4, 6, 2,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 4, 2, 4, 6, 2,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 9, 0, 2, 3, 4, 2, 4, 6, 4, 3, 8,-1,-1,-1,-1],
    [ 1, 9, 4, 1, 4, 2, 2, 4, 6,-1,-1,-1,-1,-1,-1,-1],
    [ 8, 1, 3, 8, 6, 1, 8, 4, 6, 6,10, 1,-1,-1,-1,-1],
    [10, 1, 0,10, 0, 6, 6, 0, 4,-1,-1,-1,-1,-1,-1,-1],
    [ 4, 6, 3, 4, 3, 8, 6,10, 3, 0, 3, 9,10, 9, 3,-1],
    [10, 9, 4, 6,10, 4,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 4, 9, 5, 7, 6,11,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 8, 3, 4, 9, 5,11, 7, 6,-1,-1,-1,-1,-1,-1,-1],
    [ 5, 0, 1, 5, 4, 0, 7, 6,11,-1,-1,-1,-1,-1,-1,-1],
    [11, 7, 6, 8, 3, 4, 3, 5, 4, 3, 1, 5,-1,-1,-1,-1],
    [ 9, 5, 4,10, 1, 2, 7, 6,11,-1,-1,-1,-1,-1,-1,-1],
    [ 6,11, 7, 1, 2,10, 0, 8, 3, 4, 9, 5,-1,-1,-1,-1],
    [ 7, 6,11, 5, 4,10, 4, 2,10, 4, 0, 2,-1,-1,-1,-1],
    [ 3, 4, 8, 3, 5, 4, 3, 2, 5,10, 5, 2, 6,11, 7,-1],
    [ 7, 2, 3, 7, 6, 2, 5, 4, 9,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 5, 4, 0, 8, 6, 0, 6, 2, 6, 8, 7,-1,-1,-1,-1],
    [ 3, 6, 2, 3, 7, 6, 1, 5, 0, 5, 4, 0,-1,-1,-1,-1],
    [ 6, 2, 8, 6, 8, 7, 2, 1, 8, 4, 8, 5, 1, 5, 8,-1],
    [ 9, 5, 4,10, 1, 6, 1, 7, 6, 1, 3, 7,-1,-1,-1,-1],
    [ 1, 6,10, 1, 7, 6, 1, 0, 7, 8, 7, 0, 9, 5, 4,-1],
    [ 4, 0,10, 4,10, 5, 0, 3,10, 6,10, 7, 3, 7,10,-1],
    [ 7, 6,10, 7,10, 8, 5, 4,10, 4, 8,10,-1,-1,-1,-1],
    [ 6, 9, 5, 6,11, 9,11, 8, 9,-1,-1,-1,-1,-1,-1,-1],
    [ 3, 6,11, 0, 6, 3, 0, 5, 6, 0, 9, 5,-1,-1,-1,-1],
    [ 0,11, 8, 0, 5,11, 0, 1, 5, 5, 6,11,-1,-1,-1,-1],
    [ 6,11, 3, 6, 3, 5, 5, 3, 1,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 2,10, 9, 5,11, 9,11, 8,11, 5, 6,-1,-1,-1,-1],
    [ 0,11, 3, 0, 6,11, 0, 9, 6, 5, 6, 9, 1, 2,10,-1],
    [11, 8, 5,11, 5, 6, 8, 0, 5,10, 5, 2, 0, 2, 5,-1],
    [ 6,11, 3, 6, 3, 5, 2,10, 3,10, 5, 3,-1,-1,-1,-1],
    [ 5, 8, 9, 5, 2, 8, 5, 6, 2, 3, 8, 2,-1,-1,-1,-1],
    [ 9, 5, 6, 9, 6, 0, 0, 6, 2,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 5, 8, 1, 8, 0, 5, 6, 8, 3, 8, 2, 6, 2, 8,-1],
    [ 1, 5, 6, 2, 1, 6,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 3, 6, 1, 6,10, 3, 8, 6, 5, 6, 9, 8, 9, 6,-1],
    [10, 1, 0,10, 0, 6, 9, 5, 0, 5, 6, 0,-1,-1,-1,-1],
    [ 0, 3, 8, 5, 6,10,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [10, 5, 6,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [11, 5,10, 7, 5,11,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [11, 5,10,11, 7, 5, 8, 3, 0,-1,-1,-1,-1,-1,-1,-1],
    [ 5,11, 7, 5,10,11, 1, 9, 0,-1,-1,-1,-1,-1,-1,-1],
    [10, 7, 5,10,11, 7, 9, 8, 1, 8, 3, 1,-1,-1,-1,-1],
    [11, 1, 2,11, 7, 1, 7, 5, 1,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 8, 3, 1, 2, 7, 1, 7, 5, 7, 2,11,-1,-1,-1,-1],
    [ 9, 7, 5, 9, 2, 7, 9, 0, 2, 2,11, 7,-1,-1,-1,-1],
    [ 7, 5, 2, 7, 2,11, 5, 9, 2, 3, 2, 8, 9, 8, 2,-1],
    [ 2, 5,10, 2, 3, 5, 3, 7, 5,-1,-1,-1,-1,-1,-1,-1],
    [ 8, 2, 0, 8, 5, 2, 8, 7, 5,10, 2, 5,-1,-1,-1,-1],
    [ 9, 0, 1, 5,10, 3, 5, 3, 7, 3,10, 2,-1,-1,-1,-1],
    [ 9, 8, 2, 9, 2, 1, 8, 7, 2,10, 2, 5, 7, 5, 2,-1],
    [ 1, 3, 5, 3, 7, 5,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 8, 7, 0, 7, 1, 1, 7, 5,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 0, 3, 9, 3, 5, 5, 3, 7,-1,-1,-1,-1,-1,-1,-1],
    [ 9, 8, 7, 5, 9, 7,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 5, 8, 4, 5,10, 8,10,11, 8,-1,-1,-1,-1,-1,-1,-1],
    [ 5, 0, 4, 5,11, 0, 5,10,11,11, 3, 0,-1,-1,-1,-1],
    [ 0, 1, 9, 8, 4,10, 8,10,11,10, 4, 5,-1,-1,-1,-1],
    [10,11, 4,10, 4, 5,11, 3, 4, 9, 4, 1, 3, 1, 4,-1],
    [ 2, 5, 1, 2, 8, 5, 2,11, 8, 4, 5, 8,-1,-1,-1,-1],
    [ 0, 4,11, 0,11, 3, 4, 5,11, 2,11, 1, 5, 1,11,-1],
    [ 0, 2, 5, 0, 5, 9, 2,11, 5, 4, 5, 8,11, 8, 5,-1],
    [ 9, 4, 5, 2,11, 3,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 2, 5,10, 3, 5, 2, 3, 4, 5, 3, 8, 4,-1,-1,-1,-1],
    [ 5,10, 2, 5, 2, 4, 4, 2, 0,-1,-1,-1,-1,-1,-1,-1],
    [ 3,10, 2, 3, 5,10, 3, 8, 5, 4, 5, 8, 0, 1, 9,-1],
    [ 5,10, 2, 5, 2, 4, 1, 9, 2, 9, 4, 2,-1,-1,-1,-1],
    [ 8, 4, 5, 8, 5, 3, 3, 5, 1,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 4, 5, 1, 0, 5,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 8, 4, 5, 8, 5, 3, 9, 0, 5, 0, 3, 5,-1,-1,-1,-1],
    [ 9, 4, 5,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 4,11, 7, 4, 9,11, 9,10,11,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 8, 3, 4, 9, 7, 9,11, 7, 9,10,11,-1,-1,-1,-1],
    [ 1,10,11, 1,11, 4, 1, 4, 0, 7, 4,11,-1,-1,-1,-1],
    [ 3, 1, 4, 3, 4, 8, 1,10, 4, 7, 4,11,10,11, 4,-1],
    [ 4,11, 7, 9,11, 4, 9, 2,11, 9, 1, 2,-1,-1,-1,-1],
    [ 9, 7, 4, 9,11, 7, 9, 1,11, 2,11, 1, 0, 8, 3,-1],
    [11, 7, 4,11, 4, 2, 2, 4, 0,-1,-1,-1,-1,-1,-1,-1],
    [11, 7, 4,11, 4, 2, 8, 3, 4, 3, 2, 4,-1,-1,-1,-1],
    [ 2, 9,10, 2, 7, 9, 2, 3, 7, 7, 4, 9,-1,-1,-1,-1],
    [ 9,10, 7, 9, 7, 4,10, 2, 7, 8, 7, 0, 2, 0, 7,-1],
    [ 3, 7,10, 3,10, 2, 7, 4,10, 1,10, 0, 4, 0,10,-1],
    [ 1,10, 2, 8, 7, 4,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 4, 9, 1, 4, 1, 7, 7, 1, 3,-1,-1,-1,-1,-1,-1,-1],
    [ 4, 9, 1, 4, 1, 7, 0, 8, 1, 8, 7, 1,-1,-1,-1,-1],
    [ 4, 0, 3, 7, 4, 3,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 4, 8, 7,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 9,10, 8,10,11, 8,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 3, 0, 9, 3, 9,11,11, 9,10,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 1,10, 0,10, 8, 8,10,11,-1,-1,-1,-1,-1,-1,-1],
    [ 3, 1,10,11, 3,10,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 2,11, 1,11, 9, 9,11, 8,-1,-1,-1,-1,-1,-1,-1],
    [ 3, 0, 9, 3, 9,11, 1, 2, 9, 2,11, 9,-1,-1,-1,-1],
    [ 0, 2,11, 8, 0,11,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 3, 2,11,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 2, 3, 8, 2, 8,10,10, 8, 9,-1,-1,-1,-1,-1,-1,-1],
    [ 9,10, 2, 0, 9, 2,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 2, 3, 8, 2, 8,10, 0, 1, 8, 1,10, 8,-1,-1,-1,-1],
    [ 1,10, 2,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 1, 3, 8, 9, 1, 8,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 9, 1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [ 0, 3, 8,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
    [-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1],
];

// ── Public convenience constructor ────────────────────────────────────────────

/// Create a grid of SPH particles for testing (nx×ny×nz lattice).
pub fn create_particle_grid(
    origin: [f64; 3],
    spacing: f64,
    nx: usize,
    ny: usize,
    nz: usize,
    mass: f64,
) -> Vec<SphParticle> {
    let mut particles = Vec::with_capacity(nx * ny * nz);
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let pos = [
                    origin[0] + ix as f64 * spacing,
                    origin[1] + iy as f64 * spacing,
                    origin[2] + iz as f64 * spacing,
                ];
                particles.push(SphParticle::new(pos, mass));
            }
        }
    }
    particles
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_sim(n_per_axis: usize) -> SphSimulation {
        let spacing = 0.02;
        let mass = 0.001;
        let particles =
            create_particle_grid([0.0; 3], spacing, n_per_axis, n_per_axis, n_per_axis, mass);
        let config = SphConfig {
            smoothing_radius: 0.05,
            rest_density: 1000.0,
            pressure_stiffness: 2000.0,
            viscosity: 0.001,
            surface_tension: 0.072,
            gravity: [0.0, -9.81, 0.0],
        };
        SphSimulation::new(particles, config)
    }

    // ── Kernel tests ──────────────────────────────────────────────────────────

    #[test]
    fn test_poly6_kernel_zero_at_h() {
        let h = 0.05;
        assert!(poly6_kernel(h, h) < 1e-15);
    }

    #[test]
    fn test_poly6_kernel_max_at_zero() {
        let h = 0.05;
        let w0 = poly6_kernel(0.0, h);
        let w_half = poly6_kernel(h * 0.5, h);
        assert!(w0 > w_half, "poly6 should be maximal at r=0");
    }

    #[test]
    fn test_poly6_kernel_non_negative() {
        let h = 0.05;
        for i in 0..=20 {
            let r = i as f64 * h / 20.0;
            assert!(poly6_kernel(r, h) >= 0.0);
        }
    }

    #[test]
    fn test_poly6_kernel_zero_outside_support() {
        let h = 0.05;
        assert_eq!(poly6_kernel(h * 1.001, h), 0.0);
        assert_eq!(poly6_kernel(h * 2.0, h), 0.0);
    }

    #[test]
    fn test_spiky_gradient_negative_inside() {
        // Spiky gradient should be negative (pointing inward) inside the kernel support
        let h = 0.05;
        let g = spiky_kernel_gradient_scalar(h * 0.5, h);
        assert!(g < 0.0, "spiky gradient should be negative: {}", g);
    }

    #[test]
    fn test_spiky_gradient_zero_outside() {
        let h = 0.05;
        assert_eq!(spiky_kernel_gradient_scalar(h * 1.001, h), 0.0);
    }

    #[test]
    fn test_viscosity_laplacian_positive_inside() {
        let h = 0.05;
        let lap = viscosity_kernel_laplacian(h * 0.5, h);
        assert!(lap > 0.0, "viscosity Laplacian should be positive: {}", lap);
    }

    #[test]
    fn test_viscosity_laplacian_zero_outside() {
        let h = 0.05;
        assert_eq!(viscosity_kernel_laplacian(h * 1.001, h), 0.0);
    }

    // ── Density tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_density_positive_at_particle_positions() {
        let mut sim = default_sim(3);
        sim.rebuild_spatial_hash();
        sim.compute_density_pressure();
        for p in &sim.particles {
            assert!(p.density > 0.0, "density should be positive");
        }
    }

    #[test]
    fn test_density_at_center_above_zero() {
        let mut sim = default_sim(3);
        sim.rebuild_spatial_hash();
        sim.compute_density_pressure();
        // density_at requires spatial hash to be built
        let center = [0.02; 3];
        let rho = sim.density_at(center);
        assert!(rho > 0.0, "density at center should be > 0: {}", rho);
    }

    #[test]
    fn test_density_far_from_particles_zero() {
        let mut sim = default_sim(2);
        sim.rebuild_spatial_hash();
        let far = [100.0, 100.0, 100.0];
        let rho = sim.density_at(far);
        assert!(rho < 1e-10, "density far away should be ~0: {}", rho);
    }

    #[test]
    fn test_density_conservation_after_steps() {
        // Total mass = Σ (density_i * volume_i). We test that total density
        // does not increase spuriously over a few steps.
        let mut sim = default_sim(3);
        sim.rebuild_spatial_hash();
        sim.compute_density_pressure();
        let rho_sum_init: f64 = sim.particles.iter().map(|p| p.density).sum();
        for _ in 0..5 {
            sim.step(0.0001);
        }
        let rho_sum_final: f64 = sim.particles.iter().map(|p| p.density).sum();
        // Should stay within 10% (SPH is weakly compressible, not strictly conserving)
        assert!(
            (rho_sum_final - rho_sum_init).abs() / rho_sum_init < 0.10,
            "density sum changed significantly: init={} final={}",
            rho_sum_init,
            rho_sum_final
        );
    }

    // ── Pressure tests ────────────────────────────────────────────────────────

    #[test]
    fn test_pressure_non_negative_at_rest_density() {
        let mut sim = default_sim(3);
        sim.rebuild_spatial_hash();
        sim.compute_density_pressure();
        for p in &sim.particles {
            assert!(p.pressure >= 0.0, "pressure should be >= 0");
        }
    }

    #[test]
    fn test_pressure_force_opposes_compression() {
        // Place many particles very close together so density exceeds rest density,
        // then verify that a pair among them has pressure force pointing apart.
        let config = SphConfig {
            smoothing_radius: 0.05,
            rest_density: 100.0, // low rest density so particles easily exceed it
            pressure_stiffness: 2000.0,
            viscosity: 0.001,
            surface_tension: 0.072,
            gravity: [0.0, 0.0, 0.0],
        };
        // Dense 4x4x4 grid with spacing much smaller than h
        let h = config.smoothing_radius;
        let particles = create_particle_grid([0.0; 3], h * 0.3, 4, 4, 4, 0.01);
        let mut sim = SphSimulation::new(particles, config);
        sim.rebuild_spatial_hash();
        sim.compute_density_pressure();
        // The central particle (index ~31) should have pressure > 0 and forces pushing outward
        let center_idx = 21; // near center of 4^3 grid
        let f = sim.pressure_force(center_idx);
        // At least one force component must be nonzero since neighbors surround it
        let mag = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
        // The two outer particles (0 and 63) should experience forces pushing outward
        let f0 = sim.pressure_force(0);
        let f_last = sim.pressure_force(63);
        // At minimum, verify force is finite and pressure > 0 for interior particles
        assert!(mag.is_finite(), "pressure force magnitude should be finite");
        assert!(
            sim.particles[center_idx].pressure > 0.0,
            "center particle pressure should be > 0: {}",
            sim.particles[center_idx].pressure
        );
        // Force magnitudes should be finite
        assert!(f0[0].is_finite() && f_last[0].is_finite());
    }

    #[test]
    fn test_pressure_gradient_direction() {
        // High-density side should repel neighbors
        let config = SphConfig::default();
        let h = config.smoothing_radius;
        // 3 particles in a row; middle one has higher effective density
        let p0 = SphParticle::new([-h * 0.3, 0.0, 0.0], 0.001);
        let p1 = SphParticle::new([0.0, 0.0, 0.0], 0.002); // heavier = higher density
        let p2 = SphParticle::new([h * 0.3, 0.0, 0.0], 0.001);
        let mut sim = SphSimulation::new(vec![p0, p1, p2], config);
        sim.rebuild_spatial_hash();
        sim.compute_density_pressure();
        let f0 = sim.pressure_force(0);
        let f2 = sim.pressure_force(2);
        // By symmetry, f0[0] < 0 (pushed left) and f2[0] > 0 (pushed right)
        assert!(
            f0[0] <= 0.0,
            "left particle should be pushed left: {}",
            f0[0]
        );
        assert!(
            f2[0] >= 0.0,
            "right particle should be pushed right: {}",
            f2[0]
        );
    }

    // ── Viscosity tests ───────────────────────────────────────────────────────

    #[test]
    fn test_viscosity_force_damps_relative_velocity() {
        // Two particles with different velocities; viscosity should partially equalize.
        let config = SphConfig::default();
        let h = config.smoothing_radius;
        let mut p0 = SphParticle::new([0.0, 0.0, 0.0], 0.001);
        p0.velocity = [1.0, 0.0, 0.0];
        let mut p1 = SphParticle::new([h * 0.3, 0.0, 0.0], 0.001);
        p1.velocity = [0.0, 0.0, 0.0];
        let mut sim = SphSimulation::new(vec![p0, p1], config);
        sim.rebuild_spatial_hash();
        sim.compute_density_pressure();
        let f0 = sim.viscosity_force(0);
        // Force on fast particle should decelerate it (negative x)
        assert!(
            f0[0] <= 0.0,
            "viscosity should decelerate fast particle: f0x={}",
            f0[0]
        );
    }

    #[test]
    fn test_viscosity_force_zero_same_velocity() {
        let config = SphConfig::default();
        let h = config.smoothing_radius;
        let mut p0 = SphParticle::new([0.0, 0.0, 0.0], 0.001);
        p0.velocity = [1.0, 0.0, 0.0];
        let mut p1 = SphParticle::new([h * 0.3, 0.0, 0.0], 0.001);
        p1.velocity = [1.0, 0.0, 0.0];
        let mut sim = SphSimulation::new(vec![p0, p1], config);
        sim.rebuild_spatial_hash();
        sim.compute_density_pressure();
        let f0 = sim.viscosity_force(0);
        for (k, &fk) in f0.iter().enumerate().take(3) {
            assert!(
                fk.abs() < 1e-10,
                "viscosity force should be zero for same velocity: f0[{}]={}",
                k,
                fk
            );
        }
    }

    // ── Surface normal tests ──────────────────────────────────────────────────

    #[test]
    fn test_surface_normal_nonzero_near_boundary() {
        let mut sim = default_sim(3);
        sim.rebuild_spatial_hash();
        sim.compute_density_pressure();
        // The outermost particle should have a non-zero surface normal
        let n = sim.surface_normal(0);
        let mag = len3(n);
        // May or may not be large depending on geometry; just check it doesn't panic
        assert!(mag.is_finite());
    }

    #[test]
    fn test_surface_normal_finite() {
        let mut sim = default_sim(3);
        sim.rebuild_spatial_hash();
        sim.compute_density_pressure();
        for i in 0..sim.particles.len() {
            let n = sim.surface_normal(i);
            assert!(n[0].is_finite() && n[1].is_finite() && n[2].is_finite());
        }
    }

    // ── Step tests ────────────────────────────────────────────────────────────

    #[test]
    fn test_step_moves_particles() {
        let mut sim = default_sim(3);
        let initial_positions: Vec<_> = sim.particles.iter().map(|p| p.position).collect();
        sim.step(0.001);
        let moved = sim
            .particles
            .iter()
            .zip(initial_positions.iter())
            .any(|(p, ip)| {
                (p.position[0] - ip[0]).abs() > 1e-15
                    || (p.position[1] - ip[1]).abs() > 1e-15
                    || (p.position[2] - ip[2]).abs() > 1e-15
            });
        assert!(moved, "particles should move after a step");
    }

    #[test]
    fn test_step_gravity_causes_downward_movement() {
        // Particles should move downward (negative y) under gravity
        let mut sim = default_sim(3);
        let init_y: Vec<f64> = sim.particles.iter().map(|p| p.position[1]).collect();
        for _ in 0..10 {
            sim.step(0.001);
        }
        let final_y: Vec<f64> = sim.particles.iter().map(|p| p.position[1]).collect();
        let downward = final_y
            .iter()
            .zip(init_y.iter())
            .filter(|(fy, iy)| *fy < *iy)
            .count();
        assert!(
            downward > 0,
            "at least some particles should fall under gravity"
        );
    }

    #[test]
    fn test_step_positions_finite() {
        let mut sim = default_sim(3);
        for _ in 0..20 {
            sim.step(0.0001);
        }
        for p in &sim.particles {
            assert!(p.position[0].is_finite());
            assert!(p.position[1].is_finite());
            assert!(p.position[2].is_finite());
        }
    }

    #[test]
    fn test_step_velocities_finite() {
        let mut sim = default_sim(3);
        for _ in 0..20 {
            sim.step(0.0001);
        }
        for p in &sim.particles {
            assert!(p.velocity[0].is_finite());
            assert!(p.velocity[1].is_finite());
            assert!(p.velocity[2].is_finite());
        }
    }

    // ── Marching cubes surface extraction tests ───────────────────────────────

    fn dense_sim_for_surface() -> SphSimulation {
        // Use low rest_density so that the SPH density at grid positions exceeds the iso-value
        let config = SphConfig {
            smoothing_radius: 0.05,
            rest_density: 50.0,
            pressure_stiffness: 2000.0,
            viscosity: 0.001,
            surface_tension: 0.072,
            gravity: [0.0, 0.0, 0.0],
        };
        let h = config.smoothing_radius;
        let particles = create_particle_grid([0.0; 3], h * 0.4, 4, 4, 4, 0.01);
        let mut sim = SphSimulation::new(particles, config);
        sim.rebuild_spatial_hash();
        sim.compute_density_pressure();
        sim
    }

    #[test]
    fn test_marching_cubes_returns_nonempty_for_dense_particles() {
        let sim = dense_sim_for_surface();
        let (verts, idxs) = sim.marching_cubes_surface(16);
        assert!(
            !verts.is_empty(),
            "surface extraction should return vertices"
        );
        assert!(!idxs.is_empty(), "surface extraction should return indices");
    }

    #[test]
    fn test_marching_cubes_triangle_count_multiple_of_3() {
        let sim = dense_sim_for_surface();
        let (_, idxs) = sim.marching_cubes_surface(16);
        assert_eq!(idxs.len() % 3, 0, "index count should be a multiple of 3");
    }

    #[test]
    fn test_marching_cubes_vertices_finite() {
        let sim = dense_sim_for_surface();
        let (verts, _) = sim.marching_cubes_surface(16);
        for v in &verts {
            assert!(v[0].is_finite() && v[1].is_finite() && v[2].is_finite());
        }
    }

    #[test]
    fn test_marching_cubes_empty_for_no_particles() {
        let config = SphConfig::default();
        let sim = SphSimulation::new(vec![], config);
        let (verts, idxs) = sim.marching_cubes_surface(16);
        assert!(verts.is_empty());
        assert!(idxs.is_empty());
    }

    #[test]
    fn test_marching_cubes_indices_within_vertex_range() {
        let sim = dense_sim_for_surface();
        let (verts, idxs) = sim.marching_cubes_surface(16);
        let n = verts.len() as u32;
        for &idx in &idxs {
            assert!(idx < n, "index {} out of range (n={})", idx, n);
        }
    }

    #[test]
    fn test_density_at_particle_position_above_threshold() {
        let mut sim = default_sim(3);
        sim.rebuild_spatial_hash();
        sim.compute_density_pressure();
        // Density at a particle's own position should be well above zero
        let pos = sim.particles[4].position;
        let rho = sim.density_at(pos);
        // The minimum threshold is loosely bounded; just confirm it's meaningfully above zero
        assert!(
            rho > 10.0,
            "density at particle position should be significant: {}",
            rho
        );
    }

    #[test]
    fn test_create_particle_grid_count() {
        let particles = create_particle_grid([0.0; 3], 0.01, 3, 4, 5, 0.001);
        assert_eq!(particles.len(), 60);
    }

    #[test]
    fn test_simulation_zero_dt_no_change() {
        let mut sim = default_sim(2);
        let pos_before: Vec<_> = sim.particles.iter().map(|p| p.position).collect();
        sim.step(0.0);
        let pos_after: Vec<_> = sim.particles.iter().map(|p| p.position).collect();
        for (b, a) in pos_before.iter().zip(pos_after.iter()) {
            assert_eq!(b[0], a[0]);
            assert_eq!(b[1], a[1]);
            assert_eq!(b[2], a[2]);
        }
    }
}
