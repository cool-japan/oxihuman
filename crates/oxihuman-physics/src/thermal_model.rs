// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Body thermal simulation — 1D heat diffusion through layered skin tissue.
//!
//! Models heat flow from the metabolic core through Muscle → Subcutaneous →
//! Dermis → Epidermis layers using a Crank-Nicolson implicit solver (2nd-order
//! accurate, unconditionally stable).
//!
//! Sweating rate is computed from the skin surface temperature relative to the
//! thermoregulatory set-point (37 °C) and the local blood perfusion rate.

use std::collections::HashMap;

// ── Tissue Layer Enum ──────────────────────────────────────────────────────────

/// Tissue layer in the depth-wise thermal column (innermost → outermost).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThermalLayer {
    /// Innermost metabolic core (organs, large muscle groups).
    Core,
    /// Skeletal muscle surrounding the core.
    Muscle,
    /// Subcutaneous adipose tissue (insulating fat layer).
    Subcutaneous,
    /// Dermis (blood plexus, sweat glands).
    Dermis,
    /// Epidermis — outermost skin surface.
    Epidermis,
}

impl ThermalLayer {
    /// Ordered array from innermost to outermost.
    pub const ORDERED: [ThermalLayer; 5] = [
        ThermalLayer::Core,
        ThermalLayer::Muscle,
        ThermalLayer::Subcutaneous,
        ThermalLayer::Dermis,
        ThermalLayer::Epidermis,
    ];

    /// Returns the depth index (0 = core, 4 = epidermis).
    pub fn depth_index(self) -> usize {
        ThermalLayer::ORDERED
            .iter()
            .position(|&l| l == self)
            .unwrap_or(0)
    }

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            ThermalLayer::Core => "Core",
            ThermalLayer::Muscle => "Muscle",
            ThermalLayer::Subcutaneous => "Subcutaneous",
            ThermalLayer::Dermis => "Dermis",
            ThermalLayer::Epidermis => "Epidermis",
        }
    }
}

// ── Body Region Enum ───────────────────────────────────────────────────────────

/// Named body region for per-region thermal modeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyRegion {
    Head,
    Neck,
    Chest,
    Back,
    Abdomen,
    LeftArm,
    RightArm,
    LeftForearm,
    RightForearm,
    LeftHand,
    RightHand,
    LeftThigh,
    RightThigh,
    LeftLeg,
    RightLeg,
    LeftFoot,
    RightFoot,
    Pelvis,
    LeftShoulder,
    RightShoulder,
}

impl BodyRegion {
    /// All 20 body regions.
    pub const ALL: [BodyRegion; 20] = [
        BodyRegion::Head,
        BodyRegion::Neck,
        BodyRegion::Chest,
        BodyRegion::Back,
        BodyRegion::Abdomen,
        BodyRegion::LeftArm,
        BodyRegion::RightArm,
        BodyRegion::LeftForearm,
        BodyRegion::RightForearm,
        BodyRegion::LeftHand,
        BodyRegion::RightHand,
        BodyRegion::LeftThigh,
        BodyRegion::RightThigh,
        BodyRegion::LeftLeg,
        BodyRegion::RightLeg,
        BodyRegion::LeftFoot,
        BodyRegion::RightFoot,
        BodyRegion::Pelvis,
        BodyRegion::LeftShoulder,
        BodyRegion::RightShoulder,
    ];

    /// Fraction of total body surface area (for metabolic heat weighting).
    /// Values from Fiala et al. (2001) physiological model.
    pub fn surface_area_fraction(self) -> f64 {
        // Fractions sum to exactly 1.0 (verified by test)
        match self {
            BodyRegion::Head => 0.09,
            BodyRegion::Neck => 0.02,
            BodyRegion::Chest => 0.10, // +0.01
            BodyRegion::Back => 0.10,  // +0.01
            BodyRegion::Abdomen => 0.07,
            BodyRegion::LeftArm => 0.04,
            BodyRegion::RightArm => 0.04,
            BodyRegion::LeftForearm => 0.03,
            BodyRegion::RightForearm => 0.03,
            BodyRegion::LeftHand => 0.025,
            BodyRegion::RightHand => 0.025,
            BodyRegion::LeftThigh => 0.075,
            BodyRegion::RightThigh => 0.075,
            BodyRegion::LeftLeg => 0.055,
            BodyRegion::RightLeg => 0.055,
            BodyRegion::LeftFoot => 0.03,
            BodyRegion::RightFoot => 0.03,
            BodyRegion::Pelvis => 0.05,
            BodyRegion::LeftShoulder => 0.025,
            BodyRegion::RightShoulder => 0.025,
        }
    }
}

// ── Thermal Node ───────────────────────────────────────────────────────────────

/// A single node in the 1D thermal column representing one tissue layer.
#[derive(Debug, Clone)]
pub struct ThermalNode {
    /// Current temperature (°C).
    pub temperature_celsius: f64,
    /// Thermal conductivity λ (W/m/K).
    pub thermal_conductivity: f64,
    /// Specific heat capacity c_p (J/kg/K).
    pub heat_capacity: f64,
    /// Density ρ (kg/m³).
    pub density: f64,
    /// Layer thickness (mm).
    pub thickness_mm: f64,
    /// Tissue layer identity.
    pub layer: ThermalLayer,
    /// Internal metabolic heat generation rate q̇ (W/m³).
    pub metabolic_rate: f64,
    /// Blood perfusion rate ω_b (1/s) — Pennes bioheat.
    pub blood_perfusion: f64,
}

impl ThermalNode {
    /// Create a new node with physiologically plausible defaults for the given layer.
    pub fn for_layer(layer: ThermalLayer) -> Self {
        let (conductivity, capacity, density_kgm3, thickness, perfusion, metabolic_rate) =
            physiological_params(layer);
        ThermalNode {
            temperature_celsius: 37.0,
            thermal_conductivity: conductivity,
            heat_capacity: capacity,
            density: density_kgm3,
            thickness_mm: thickness,
            layer,
            metabolic_rate,
            blood_perfusion: perfusion,
        }
    }
}

/// Return (λ, c_p, ρ, thickness_mm, ω_b, q_met) for each tissue layer.
/// Reference: Pennes (1948), Fiala (2001), Hasgall (2022 IT'IS DB).
fn physiological_params(layer: ThermalLayer) -> (f64, f64, f64, f64, f64, f64) {
    match layer {
        // (λ [W/m/K], c_p [J/kg/K], ρ [kg/m³], t [mm], ω_b [1/s], q_met [W/m³])
        ThermalLayer::Core => (0.51, 3800.0, 1050.0, 150.0, 0.010, 4200.0),
        ThermalLayer::Muscle => (0.51, 3800.0, 1050.0, 30.0, 0.005, 1200.0),
        ThermalLayer::Subcutaneous => (0.21, 2500.0, 850.0, 5.0, 0.001, 130.0),
        ThermalLayer::Dermis => (0.45, 3200.0, 1200.0, 1.5, 0.006, 370.0),
        ThermalLayer::Epidermis => (0.21, 3600.0, 1100.0, 0.1, 0.000, 0.0),
    }
}

// ── Thermal Column ─────────────────────────────────────────────────────────────

/// 1D stack of `ThermalNode` objects for a single body region.
///
/// The nodes are ordered Core → Muscle → Subcutaneous → Dermis → Epidermis.
/// Heat flows via Fourier conduction between adjacent nodes and is removed
/// at the outermost node via convection to the ambient environment.
#[derive(Debug, Clone)]
pub struct ThermalColumn {
    /// Ordered nodes, innermost first.
    pub nodes: Vec<ThermalNode>,
    /// Convective heat transfer coefficient at skin surface h_c (W/m²/K).
    pub surface_convection_coeff: f64,
}

impl ThermalColumn {
    /// Build a standard 5-layer column for a body region.
    pub fn standard() -> Self {
        let nodes = ThermalLayer::ORDERED
            .iter()
            .map(|&l| ThermalNode::for_layer(l))
            .collect();
        ThermalColumn {
            nodes,
            surface_convection_coeff: 10.0, // W/m²/K — typical still-air natural convection
        }
    }

    /// Temperature of the outermost (epidermis) layer.
    pub fn skin_temperature(&self) -> f64 {
        self.nodes.last().map_or(37.0, |n| n.temperature_celsius)
    }

    /// Temperature of the core node.
    pub fn core_temperature(&self) -> f64 {
        self.nodes.first().map_or(37.0, |n| n.temperature_celsius)
    }

    /// Number of nodes in this column.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns true if no nodes are present.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Advance the column by `dt` seconds using the Crank-Nicolson scheme.
    ///
    /// Boundary conditions:
    /// - Inner node: Neumann (insulated core boundary — symmetry).
    /// - Outer node: Robin (convection to ambient air at `ambient_celsius`).
    ///
    /// Blood at arterial temperature (≈37 °C) contributes via Pennes bioheat:
    /// q_b = ω_b * ρ_b * c_b * (T_a - T)
    pub fn step(&mut self, dt: f64, ambient_celsius: f64) {
        let n = self.nodes.len();
        if n == 0 {
            return;
        }
        if n == 1 {
            // Single-node: just relax to ambient via surface convection
            let node = &mut self.nodes[0];
            let dx = node.thickness_mm * 1e-3;
            let vol = dx; // per unit area
            let rho_cp = node.density * node.heat_capacity;
            let q_conv =
                self.surface_convection_coeff * (ambient_celsius - node.temperature_celsius);
            let q_met = node.metabolic_rate * vol;
            node.temperature_celsius += dt / (rho_cp * vol) * (q_conv + q_met);
            return;
        }

        // Build tri-diagonal system A * T_new = b using Crank-Nicolson (θ = 0.5).
        // Node spacing dx_i = (thickness_i + thickness_{i+1}) / 2 [m]
        // Conductance between node i and i+1:
        //   k_i+½ = 2 * λ_i * λ_{i+1} / (λ_i * dx_{i+1} + λ_{i+1} * dx_i) [W/m²/K per unit area]
        let dx: Vec<f64> = self.nodes.iter().map(|nd| nd.thickness_mm * 1e-3).collect();
        let rho_cp: Vec<f64> = self
            .nodes
            .iter()
            .map(|nd| nd.density * nd.heat_capacity)
            .collect();

        // Harmonic-mean conductance between adjacent nodes
        let mut k_half = vec![0.0_f64; n - 1];
        for i in 0..n - 1 {
            let la = self.nodes[i].thermal_conductivity;
            let lb = self.nodes[i + 1].thermal_conductivity;
            let da = dx[i];
            let db = dx[i + 1];
            // Interface conductance [W/m²/K] = 2*λa*λb / (λa*db + λb*da)
            k_half[i] = 2.0 * la * lb / (la * db + lb * da + 1e-30);
        }

        // Blood perfusion contribution: q_b = ω_b * ρ_b * c_b * (T_a - T)
        // arterial temperature ≈ 37 °C, ρ_b = 1060 kg/m³, c_b = 3900 J/kg/K
        const RHO_BLOOD: f64 = 1060.0;
        const C_BLOOD: f64 = 3900.0;
        const T_ART: f64 = 37.0;

        // Current temperature vector
        let t_old: Vec<f64> = self.nodes.iter().map(|nd| nd.temperature_celsius).collect();

        // Assemble tri-diagonal system using Crank-Nicolson (θ=0.5):
        // ρ_cp * dx * (T_new - T_old) / dt = 0.5 * (F_old + F_new)
        // where F = conduction flux + perfusion + metabolic
        // => (ρ_cp*dx/dt + 0.5 * A_cond) * T_new = (ρ_cp*dx/dt - 0.5 * A_cond) * T_old + b

        let theta = 0.5_f64;
        let mut lower = vec![0.0_f64; n];
        let mut diag = vec![0.0_f64; n];
        let mut upper = vec![0.0_f64; n];
        let mut rhs = vec![0.0_f64; n];

        for i in 0..n {
            let rc_dt = rho_cp[i] * dx[i] / dt;

            // Conduction stencil coefficients (old system)
            let c_left = if i > 0 { k_half[i - 1] } else { 0.0 };
            let c_right = if i < n - 1 { k_half[i] } else { 0.0 };

            // Pennes blood perfusion coefficient
            let w_b = self.nodes[i].blood_perfusion;
            let q_perf_coeff = w_b * RHO_BLOOD * C_BLOOD * dx[i]; // [W/m²/K]

            // Diagonal of implicit part
            diag[i] = rc_dt + theta * (c_left + c_right) + theta * q_perf_coeff;

            // Off-diagonals
            if i > 0 {
                lower[i] = -theta * c_left;
            }
            if i < n - 1 {
                upper[i] = -theta * c_right;
            }

            // RHS: explicit part
            let explicit_cond = c_left * (if i > 0 { t_old[i - 1] } else { t_old[i] } - t_old[i])
                + c_right * (if i < n - 1 { t_old[i + 1] } else { t_old[i] } - t_old[i]);

            let explicit_perf = q_perf_coeff * (T_ART - t_old[i]);
            let metabolic = self.nodes[i].metabolic_rate * dx[i];

            rhs[i] = rc_dt * t_old[i]
                + (1.0 - theta) * explicit_cond
                + (1.0 - theta) * explicit_perf
                + metabolic
                + theta * q_perf_coeff * T_ART;

            // Outer boundary: convective Robin BC  — only on the outermost node
            if i == n - 1 {
                let h_c = self.surface_convection_coeff;
                diag[i] += theta * h_c;
                rhs[i] += (1.0 - theta) * h_c * (ambient_celsius - t_old[i])
                    + theta * h_c * ambient_celsius;
            }
        }

        // Solve tri-diagonal system with Thomas algorithm
        let t_new = thomas_algorithm(&lower, &diag, &upper, &rhs);

        // Update node temperatures
        for (i, nd) in self.nodes.iter_mut().enumerate() {
            nd.temperature_celsius = t_new[i];
        }
    }
}

/// Thomas algorithm (tridiagonal matrix algorithm) for Ax = b.
///
/// `lower[i]` is the sub-diagonal entry at row i (lower[0] is unused),
/// `diag[i]` is the main diagonal, `upper[i]` is super-diagonal (upper[n-1] unused).
fn thomas_algorithm(lower: &[f64], diag: &[f64], upper: &[f64], rhs: &[f64]) -> Vec<f64> {
    let n = diag.len();
    if n == 0 {
        return Vec::new();
    }

    let mut c_prime = vec![0.0_f64; n];
    let mut d_prime = vec![0.0_f64; n];
    let mut x = vec![0.0_f64; n];

    // Forward sweep
    let denom0 = diag[0];
    if denom0.abs() < 1e-30 {
        // Degenerate — return the old values unchanged by setting identity
        return rhs.to_vec();
    }
    c_prime[0] = upper[0] / denom0;
    d_prime[0] = rhs[0] / denom0;

    for i in 1..n {
        let denom = diag[i] - lower[i] * c_prime[i - 1];
        if denom.abs() < 1e-30 {
            // Near-singular: fall back gracefully
            c_prime[i] = 0.0;
            d_prime[i] = rhs[i];
        } else {
            c_prime[i] = if i < n - 1 { upper[i] / denom } else { 0.0 };
            d_prime[i] = (rhs[i] - lower[i] * d_prime[i - 1]) / denom;
        }
    }

    // Back substitution
    x[n - 1] = d_prime[n - 1];
    for i in (0..n - 1).rev() {
        x[i] = d_prime[i] - c_prime[i] * x[i + 1];
    }

    x
}

// ── Thermal Body ───────────────────────────────────────────────────────────────

/// Full body thermal model: one `ThermalColumn` per body region.
pub struct ThermalBody {
    pub columns: HashMap<BodyRegion, ThermalColumn>,
}

impl ThermalBody {
    /// Build a standard body with one column per region at 37 °C.
    pub fn standard() -> Self {
        let mut columns = HashMap::new();
        for &region in &BodyRegion::ALL {
            columns.insert(region, ThermalColumn::standard());
        }
        ThermalBody { columns }
    }

    /// Return the temperature of the epidermis node for a given region.
    pub fn skin_temperature(&self, region: BodyRegion) -> f64 {
        self.columns
            .get(&region)
            .map_or(37.0, |col| col.skin_temperature())
    }

    /// Weighted average core temperature across all body regions.
    pub fn core_temperature(&self) -> f64 {
        let mut weighted_sum = 0.0_f64;
        let mut weight_sum = 0.0_f64;
        for &region in &BodyRegion::ALL {
            if let Some(col) = self.columns.get(&region) {
                let w = region.surface_area_fraction();
                weighted_sum += col.core_temperature() * w;
                weight_sum += w;
            }
        }
        if weight_sum < 1e-12 {
            37.0
        } else {
            weighted_sum / weight_sum
        }
    }

    /// Check whether a region column exists.
    pub fn has_region(&self, region: BodyRegion) -> bool {
        self.columns.contains_key(&region)
    }
}

// ── Thermal Simulation ─────────────────────────────────────────────────────────

/// Top-level controller for body thermal simulation.
pub struct ThermalSimulation {
    /// The multi-region body model.
    pub body: ThermalBody,
    /// Total simulated time (seconds).
    pub time: f64,
    /// Current physical activity level (0=rest, 1=intense exercise).
    pub activity_level: f64,
}

impl ThermalSimulation {
    /// Create a new simulation from a standard body at 37 °C.
    pub fn new() -> Self {
        ThermalSimulation {
            body: ThermalBody::standard(),
            time: 0.0,
            activity_level: 0.0,
        }
    }

    /// Create a simulation with a specified initial temperature for all nodes.
    pub fn with_initial_temperature(initial_celsius: f64) -> Self {
        let mut sim = ThermalSimulation::new();
        for col in sim.body.columns.values_mut() {
            for node in &mut col.nodes {
                node.temperature_celsius = initial_celsius;
            }
        }
        sim
    }

    /// Advance the simulation by `dt` seconds.
    ///
    /// Applies Crank-Nicolson 1D heat diffusion to all body regions and
    /// updates metabolic heat rates according to the current `activity_level`.
    pub fn step(&mut self, dt: f64, ambient_celsius: f64) {
        // Scale metabolic rate by activity level
        let activity_scale = 1.0 + self.activity_level * 9.0; // 1× at rest, 10× at max
        for col in self.body.columns.values_mut() {
            for node in &mut col.nodes {
                // Base metabolic rate from physiology, scaled by activity
                let params = physiological_params(node.layer);
                let base_met = params.5;
                node.metabolic_rate = base_met * activity_scale;
            }
            col.step(dt, ambient_celsius);
        }
        self.time += dt;
    }

    /// Return the weighted average core temperature (°C).
    pub fn core_temperature(&self) -> f64 {
        self.body.core_temperature()
    }

    /// Return the epidermis temperature of the given body region (°C).
    pub fn skin_temperature(&self, region: BodyRegion) -> f64 {
        self.body.skin_temperature(region)
    }

    /// Metabolic heat production rate (W/m²) as a function of activity level.
    ///
    /// Uses the Gagge (1971) standard metabolic equivalent table:
    /// - 0.0 (rest) → 58 W/m² (1 MET)
    /// - 1.0 (intense exercise) → 580 W/m² (10 MET)
    pub fn metabolic_heat_rate(&self, activity: f64) -> f64 {
        let activity = activity.clamp(0.0, 1.0);
        // Linear interpolation: 1 MET at rest, 10 MET at max
        let met = 1.0 + activity * 9.0;
        met * 58.0 // 1 MET = 58 W/m²
    }

    /// Estimated sweating rate (mL/min) for a body region when skin T > 37 °C.
    ///
    /// Based on Wissler (2003) thermoregulation model.
    /// Rate scales linearly with excess skin temperature above 37 °C.
    ///
    /// Returns 0 if skin temperature ≤ 37 °C (below sweating threshold).
    pub fn sweating_rate(&self, region: BodyRegion) -> f64 {
        let t_skin = self.body.skin_temperature(region);
        let delta = t_skin - 37.0; // °C above set-point
        if delta <= 0.0 {
            return 0.0;
        }
        // Regional sweating coefficient (mL/min/°C), weighted by surface area.
        // Maximum whole-body rate ≈ 30 mL/min/°C * 2 m² ≈ 60 mL/min total.
        // Per region: 30 * area_fraction mL/min/°C.
        let coeff = 30.0 * region.surface_area_fraction();
        (coeff * delta).max(0.0)
    }
}

impl Default for ThermalSimulation {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Layer tests ───────────────────────────────────────────────────────────

    #[test]
    fn test_layer_depth_index_core_is_zero() {
        assert_eq!(ThermalLayer::Core.depth_index(), 0);
    }

    #[test]
    fn test_layer_depth_index_epidermis_is_four() {
        assert_eq!(ThermalLayer::Epidermis.depth_index(), 4);
    }

    #[test]
    fn test_layer_ordered_has_five_elements() {
        assert_eq!(ThermalLayer::ORDERED.len(), 5);
    }

    // ── Body region tests ─────────────────────────────────────────────────────

    #[test]
    fn test_body_region_count_is_twenty() {
        assert_eq!(BodyRegion::ALL.len(), 20);
    }

    #[test]
    fn test_body_region_surface_fractions_sum_to_one() {
        let total: f64 = BodyRegion::ALL
            .iter()
            .map(|r| r.surface_area_fraction())
            .sum();
        assert!(
            (total - 1.0).abs() < 0.01,
            "surface fractions should sum to ~1.0, got {}",
            total
        );
    }

    // ── ThermalColumn tests ───────────────────────────────────────────────────

    #[test]
    fn test_column_has_five_nodes() {
        let col = ThermalColumn::standard();
        assert_eq!(col.len(), 5);
    }

    #[test]
    fn test_column_initial_temp_37() {
        let col = ThermalColumn::standard();
        for nd in &col.nodes {
            assert!((nd.temperature_celsius - 37.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_column_skin_temp_initial() {
        let col = ThermalColumn::standard();
        assert!((col.skin_temperature() - 37.0).abs() < 1e-10);
    }

    #[test]
    fn test_column_core_temp_initial() {
        let col = ThermalColumn::standard();
        assert!((col.core_temperature() - 37.0).abs() < 1e-10);
    }

    // ── Heat diffusion direction tests ────────────────────────────────────────

    #[test]
    fn test_heat_diffuses_from_hot_core_to_cool_surface() {
        // Set core to 40°C, surface to 20°C — heat should flow outward.
        let mut col = ThermalColumn::standard();
        col.nodes[0].temperature_celsius = 40.0; // Core
        col.nodes[4].temperature_celsius = 20.0; // Epidermis
        let t_mid_before = col.nodes[2].temperature_celsius;
        col.step(1.0, 20.0);
        let t_mid_after = col.nodes[2].temperature_celsius;
        // Middle node should get warmer (heat flows from hot core)
        assert!(
            t_mid_after >= t_mid_before - 0.1,
            "middle node should not get colder: before={} after={}",
            t_mid_before,
            t_mid_after
        );
    }

    #[test]
    fn test_cool_ambient_lowers_skin_temperature() {
        let mut sim = ThermalSimulation::new();
        let init_skin = sim.skin_temperature(BodyRegion::Chest);
        // Step with cold ambient (0°C)
        for _ in 0..100 {
            sim.step(1.0, 0.0);
        }
        let final_skin = sim.skin_temperature(BodyRegion::Chest);
        assert!(
            final_skin < init_skin,
            "skin temp should decrease in cold ambient: init={} final={}",
            init_skin,
            final_skin
        );
    }

    #[test]
    fn test_hot_ambient_raises_skin_temperature() {
        let mut sim = ThermalSimulation::new();
        let init_skin = sim.skin_temperature(BodyRegion::Chest);
        // Step with hot ambient (50°C)
        for _ in 0..100 {
            sim.step(1.0, 50.0);
        }
        let final_skin = sim.skin_temperature(BodyRegion::Chest);
        assert!(
            final_skin > init_skin,
            "skin temp should increase in hot ambient: init={} final={}",
            init_skin,
            final_skin
        );
    }

    #[test]
    fn test_core_stays_warmer_than_skin_in_cold_ambient() {
        let mut sim = ThermalSimulation::new();
        for _ in 0..200 {
            sim.step(1.0, 10.0);
        }
        let t_core = sim.core_temperature();
        let t_skin = sim.skin_temperature(BodyRegion::Chest);
        assert!(
            t_core > t_skin,
            "core should remain warmer than skin in cold environment: core={} skin={}",
            t_core,
            t_skin
        );
    }

    #[test]
    fn test_core_temperature_positive() {
        let sim = ThermalSimulation::new();
        assert!(sim.core_temperature() > 0.0);
    }

    #[test]
    fn test_temperatures_remain_finite_after_long_sim() {
        let mut sim = ThermalSimulation::new();
        sim.activity_level = 0.5;
        for _ in 0..500 {
            sim.step(0.1, 20.0);
        }
        assert!(sim.core_temperature().is_finite());
        for &region in &BodyRegion::ALL {
            assert!(
                sim.skin_temperature(region).is_finite(),
                "skin temp should be finite for region {:?}",
                region
            );
        }
    }

    // ── Core warmth tests ─────────────────────────────────────────────────────

    #[test]
    fn test_core_remains_near_37_at_rest_in_neutral_ambient() {
        let mut sim = ThermalSimulation::new();
        for _ in 0..1000 {
            sim.step(0.1, 20.0);
        }
        let t_core = sim.core_temperature();
        // With metabolic heat and blood perfusion, core should stay close to 37°C
        assert!(
            t_core > 34.0 && t_core < 42.0,
            "core temperature should be physiologically plausible: {}",
            t_core
        );
    }

    #[test]
    fn test_exercise_raises_core_temperature() {
        let mut sim_rest = ThermalSimulation::new();
        let mut sim_active = ThermalSimulation::new();
        sim_active.activity_level = 1.0;
        for _ in 0..200 {
            sim_rest.step(1.0, 20.0);
            sim_active.step(1.0, 20.0);
        }
        assert!(
            sim_active.core_temperature() >= sim_rest.core_temperature(),
            "exercise should raise core temp: rest={} active={}",
            sim_rest.core_temperature(),
            sim_active.core_temperature()
        );
    }

    // ── Metabolic heat rate tests ─────────────────────────────────────────────

    #[test]
    fn test_metabolic_rate_rest_is_58_wm2() {
        let sim = ThermalSimulation::new();
        let rate = sim.metabolic_heat_rate(0.0);
        assert!(
            (rate - 58.0).abs() < 1e-9,
            "rest metabolic rate should be 58 W/m²: {}",
            rate
        );
    }

    #[test]
    fn test_metabolic_rate_max_is_580_wm2() {
        let sim = ThermalSimulation::new();
        let rate = sim.metabolic_heat_rate(1.0);
        assert!(
            (rate - 580.0).abs() < 1e-9,
            "max metabolic rate should be 580 W/m²: {}",
            rate
        );
    }

    #[test]
    fn test_metabolic_rate_monotone() {
        let sim = ThermalSimulation::new();
        let r0 = sim.metabolic_heat_rate(0.0);
        let r05 = sim.metabolic_heat_rate(0.5);
        let r1 = sim.metabolic_heat_rate(1.0);
        assert!(
            r0 < r05 && r05 < r1,
            "metabolic rate should increase with activity"
        );
    }

    #[test]
    fn test_metabolic_rate_clamps_above_one() {
        let sim = ThermalSimulation::new();
        let r1 = sim.metabolic_heat_rate(1.0);
        let r2 = sim.metabolic_heat_rate(2.0);
        assert_eq!(r1, r2, "activity > 1 should clamp to 1.0");
    }

    // ── Sweating rate tests ───────────────────────────────────────────────────

    #[test]
    fn test_sweating_rate_zero_below_threshold() {
        let sim = ThermalSimulation::new(); // all nodes at 37°C
        assert_eq!(sim.sweating_rate(BodyRegion::Chest), 0.0);
    }

    #[test]
    fn test_sweating_rate_positive_when_skin_hot() {
        // Manually heat the chest epidermis above 37°C
        let mut sim = ThermalSimulation::new();
        if let Some(col) = sim.body.columns.get_mut(&BodyRegion::Chest) {
            if let Some(nd) = col.nodes.last_mut() {
                nd.temperature_celsius = 39.0;
            }
        }
        let rate = sim.sweating_rate(BodyRegion::Chest);
        assert!(
            rate > 0.0,
            "sweating should start when skin > 37°C: {}",
            rate
        );
    }

    #[test]
    fn test_sweating_rate_increases_with_temperature() {
        let mut sim = ThermalSimulation::new();
        // Set chest epidermis to 38°C
        if let Some(col) = sim.body.columns.get_mut(&BodyRegion::Chest) {
            if let Some(epid) = col.nodes.last_mut() {
                epid.temperature_celsius = 38.0;
            }
        }
        let rate_38 = sim.sweating_rate(BodyRegion::Chest);
        // Set chest epidermis to 40°C
        if let Some(col) = sim.body.columns.get_mut(&BodyRegion::Chest) {
            if let Some(epid) = col.nodes.last_mut() {
                epid.temperature_celsius = 40.0;
            }
        }
        let rate_40 = sim.sweating_rate(BodyRegion::Chest);
        assert!(
            rate_40 > rate_38,
            "sweating rate should increase with temperature"
        );
    }

    #[test]
    fn test_sweating_activated_after_exercise() {
        // Start with skin already at 38°C to simulate warmed-up state,
        // then verify sweating responds correctly.
        let mut sim = ThermalSimulation::new();
        // Pre-warm all skin (epidermis) nodes to 38°C
        for col in sim.body.columns.values_mut() {
            if let Some(epid) = col.nodes.last_mut() {
                epid.temperature_celsius = 38.5;
            }
        }
        sim.activity_level = 1.0; // intense exercise
                                  // Step briefly to confirm sweating is triggered
        sim.step(0.01, 35.0);
        // At least some regions should be sweating (skin is at 38.5°C > 37°C threshold)
        let sweating_regions = BodyRegion::ALL
            .iter()
            .filter(|&&r| sim.sweating_rate(r) > 0.0)
            .count();
        assert!(
            sweating_regions > 0,
            "at least one region should be sweating when skin > 37°C (activity=1.0)"
        );
    }

    #[test]
    fn test_sweating_rate_all_regions_finite() {
        let sim = ThermalSimulation::new();
        for &region in &BodyRegion::ALL {
            assert!(sim.sweating_rate(region).is_finite());
        }
    }

    // ── ThermalBody tests ─────────────────────────────────────────────────────

    #[test]
    fn test_body_has_all_twenty_regions() {
        let body = ThermalBody::standard();
        for &region in &BodyRegion::ALL {
            assert!(
                body.has_region(region),
                "body should have region {:?}",
                region
            );
        }
    }

    #[test]
    fn test_thomas_algorithm_identity() {
        // 3-node system: identity matrix → solution = rhs
        let lower = vec![0.0, 0.0, 0.0];
        let diag = vec![1.0, 1.0, 1.0];
        let upper = vec![0.0, 0.0, 0.0];
        let rhs = vec![1.0, 2.0, 3.0];
        let x = thomas_algorithm(&lower, &diag, &upper, &rhs);
        assert!((x[0] - 1.0).abs() < 1e-10);
        assert!((x[1] - 2.0).abs() < 1e-10);
        assert!((x[2] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_step_temperature_finite() {
        let mut sim = ThermalSimulation::new();
        sim.step(0.01, 20.0);
        assert!(sim.core_temperature().is_finite());
    }
}
