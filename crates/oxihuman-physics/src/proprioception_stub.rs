// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Neuromuscular proprioception models.
//!
//! Provides physiologically-grounded models for:
//! - **Muscle spindle Ia afferents**: dynamic sensitivity to stretch velocity with
//!   gamma-dynamic fusimotor drive.
//! - **Muscle spindle II afferents**: static position sensitivity with gamma-static
//!   fusimotor drive.
//! - **Golgi tendon organs (GTO)**: force-dependent firing with adaptation.
//! - **Joint receptors**: angle-dependent firing with range sensitivity.
//! - **Stretch reflex arc**: Ia -> alpha motor neuron -> muscle contraction with latency.
//! - **Adaptation**: firing rate decay over sustained stimulus.
//! - **Alpha-gamma coactivation**: fusimotor set linked to alpha drive.
//! - **Multi-muscle proprioceptive integration**.
//!
//! # Mathematical models
//!
//! - Ia firing rate: `f_Ia = K_d * gamma_d * dL/dt + K_s * gamma_s * (L - L0) + baseline`
//! - II firing rate: `f_II = K_s2 * gamma_s * (L - L0) + baseline`
//! - GTO firing rate: `f_GTO = K_f * F^0.5 * (1 - exp(-t/tau_adapt))`
//! - Reflex torque: `tau = G_reflex * f_Ia * (1 - exp(-t/tau_delay))`

// ---------------------------------------------------------------------------
// Legacy API (preserved from stub, now f64)
// ---------------------------------------------------------------------------

/// Basic muscle sensor state (legacy API, preserved for backward compatibility).
pub struct MuscleSensor {
    pub muscle_length: f64,
    pub velocity: f64,
    pub force: f64,
    pub spindle_gain: f64,
    pub gto_gain: f64,
}

/// Create a new `MuscleSensor` with default values.
pub fn new_muscle_sensor() -> MuscleSensor {
    MuscleSensor {
        muscle_length: 1.0,
        velocity: 0.0,
        force: 0.0,
        spindle_gain: 1.0,
        gto_gain: 0.5,
    }
}

/// Ia afferent firing rate (legacy API).
#[allow(non_snake_case)]
pub fn spindle_Ia_firing(s: &MuscleSensor) -> f64 {
    s.spindle_gain * (s.velocity + 0.1 * s.muscle_length)
}

/// II afferent firing rate (legacy API).
#[allow(non_snake_case)]
pub fn spindle_II_firing(s: &MuscleSensor) -> f64 {
    s.spindle_gain * s.muscle_length
}

/// GTO firing rate (legacy API).
pub fn gto_firing(s: &MuscleSensor) -> f64 {
    s.gto_gain * s.force
}

/// Update sensor state (legacy API).
pub fn sensor_update(s: &mut MuscleSensor, length: f64, velocity: f64, force: f64) {
    s.muscle_length = length;
    s.velocity = velocity;
    s.force = force;
}

/// Check if force exceeds threshold (legacy API).
pub fn sensor_is_overloaded(s: &MuscleSensor, threshold: f64) -> bool {
    gto_firing(s) > threshold
}

// ---------------------------------------------------------------------------
// Configurable receptor parameters
// ---------------------------------------------------------------------------

/// Parameters for muscle spindle Ia afferent model.
#[derive(Debug, Clone)]
pub struct SpindleIaParams {
    /// Dynamic sensitivity gain (impulses/s per unit velocity).
    pub k_dynamic: f64,
    /// Static sensitivity gain (impulses/s per unit length deviation).
    pub k_static: f64,
    /// Resting (reference) muscle length.
    pub rest_length: f64,
    /// Baseline firing rate (impulses/s).
    pub baseline: f64,
    /// Adaptation time constant (seconds).
    pub tau_adapt: f64,
}

impl Default for SpindleIaParams {
    fn default() -> Self {
        Self {
            k_dynamic: 4.3,
            k_static: 0.5,
            rest_length: 1.0,
            baseline: 1.0,
            tau_adapt: 0.2,
        }
    }
}

/// Parameters for muscle spindle II afferent model.
#[derive(Debug, Clone)]
pub struct SpindleIiParams {
    /// Static sensitivity gain (impulses/s per unit length deviation).
    pub k_static: f64,
    /// Resting (reference) muscle length.
    pub rest_length: f64,
    /// Baseline firing rate (impulses/s).
    pub baseline: f64,
    /// Adaptation time constant (seconds).
    pub tau_adapt: f64,
}

impl Default for SpindleIiParams {
    fn default() -> Self {
        Self {
            k_static: 1.8,
            rest_length: 1.0,
            baseline: 1.0,
            tau_adapt: 0.5,
        }
    }
}

/// Parameters for Golgi tendon organ model.
#[derive(Debug, Clone)]
pub struct GtoParams {
    /// Force sensitivity gain.
    pub k_force: f64,
    /// Adaptation time constant (seconds).
    pub tau_adapt: f64,
    /// Baseline firing rate (impulses/s).
    pub baseline: f64,
    /// Force exponent (typically 0.5 for square-root nonlinearity).
    pub force_exponent: f64,
}

impl Default for GtoParams {
    fn default() -> Self {
        Self {
            k_force: 6.0,
            tau_adapt: 0.15,
            baseline: 0.0,
            force_exponent: 0.5,
        }
    }
}

/// Parameters for joint receptor model.
#[derive(Debug, Clone)]
pub struct JointReceptorParams {
    /// Preferred angle (radians) — center of sensitivity range.
    pub preferred_angle: f64,
    /// Range half-width (radians).
    pub range_half_width: f64,
    /// Peak firing gain (impulses/s).
    pub peak_gain: f64,
    /// Baseline firing rate (impulses/s).
    pub baseline: f64,
    /// Adaptation time constant (seconds).
    pub tau_adapt: f64,
}

impl Default for JointReceptorParams {
    fn default() -> Self {
        Self {
            preferred_angle: 0.0,
            range_half_width: core::f64::consts::FRAC_PI_4,
            peak_gain: 50.0,
            baseline: 2.0,
            tau_adapt: 0.3,
        }
    }
}

/// Parameters for the stretch reflex arc.
#[derive(Debug, Clone)]
pub struct ReflexArcParams {
    /// Reflex gain (torque per impulse/s of Ia firing).
    pub gain: f64,
    /// Monosynaptic delay time constant (seconds).
    pub tau_delay: f64,
    /// Ia firing threshold below which no reflex is triggered.
    pub threshold: f64,
}

impl Default for ReflexArcParams {
    fn default() -> Self {
        Self {
            gain: 0.1,
            tau_delay: 0.03,
            threshold: 5.0,
        }
    }
}

/// Alpha-gamma coactivation parameters.
#[derive(Debug, Clone)]
pub struct CoactivationParams {
    /// Coupling coefficient: gamma_d = coupling_d * alpha_drive.
    pub coupling_dynamic: f64,
    /// Coupling coefficient: gamma_s = coupling_s * alpha_drive.
    pub coupling_static: f64,
    /// Alpha drive clamp range [0, max_alpha].
    pub max_alpha: f64,
}

impl Default for CoactivationParams {
    fn default() -> Self {
        Self {
            coupling_dynamic: 0.7,
            coupling_static: 0.5,
            max_alpha: 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Fusimotor drive state
// ---------------------------------------------------------------------------

/// Gamma fusimotor drive levels (set by the CNS or via coactivation).
#[derive(Debug, Clone)]
pub struct FusimotorDrive {
    /// Dynamic fusimotor drive (0..1 typically).
    pub gamma_dynamic: f64,
    /// Static fusimotor drive (0..1 typically).
    pub gamma_static: f64,
}

impl Default for FusimotorDrive {
    fn default() -> Self {
        Self {
            gamma_dynamic: 1.0,
            gamma_static: 1.0,
        }
    }
}

impl FusimotorDrive {
    /// Compute fusimotor drive from alpha motor neuron activation level
    /// using alpha-gamma coactivation.
    pub fn from_coactivation(alpha_drive: f64, params: &CoactivationParams) -> Self {
        let alpha_clamped = alpha_drive.clamp(0.0, params.max_alpha);
        Self {
            gamma_dynamic: (params.coupling_dynamic * alpha_clamped).clamp(0.0, 1.0),
            gamma_static: (params.coupling_static * alpha_clamped).clamp(0.0, 1.0),
        }
    }
}

// ---------------------------------------------------------------------------
// Adaptation state
// ---------------------------------------------------------------------------

/// Tracks firing rate adaptation (exponential decay of response over time).
#[derive(Debug, Clone)]
pub struct AdaptationState {
    /// Accumulated adaptation factor in [0, 1]. 1 = fully adapted (silent).
    adapted_fraction: f64,
    /// Previous raw (unadapted) firing rate, for detecting stimulus change.
    prev_raw_rate: f64,
}

impl Default for AdaptationState {
    fn default() -> Self {
        Self {
            adapted_fraction: 0.0,
            prev_raw_rate: 0.0,
        }
    }
}

impl AdaptationState {
    /// Create a fresh (unadapted) state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update adaptation and return the adapted firing rate.
    ///
    /// When the raw rate changes substantially, adaptation resets.
    /// Over time with constant stimulus, the output decays toward baseline.
    ///
    /// `raw_rate`: unadapted firing rate (impulses/s).
    /// `baseline`: minimum firing rate even at full adaptation.
    /// `tau`: adaptation time constant (seconds).
    /// `dt`: simulation time step (seconds).
    pub fn apply(&mut self, raw_rate: f64, baseline: f64, tau: f64, dt: f64) -> f64 {
        // Detect stimulus change: reset adaptation if raw rate changed
        // by more than 10% or 2 impulses/s.
        let delta = (raw_rate - self.prev_raw_rate).abs();
        let relative_change = if self.prev_raw_rate.abs() > 1e-12 {
            delta / self.prev_raw_rate.abs()
        } else {
            delta
        };
        if relative_change > 0.1 || delta > 2.0 {
            self.adapted_fraction = 0.0;
        }
        self.prev_raw_rate = raw_rate;

        // Exponential growth of adaptation toward 1.
        let safe_tau = tau.max(1e-6);
        let decay = (-dt / safe_tau).exp();
        self.adapted_fraction = 1.0 - (1.0 - self.adapted_fraction) * decay;
        // But clamp to leave some residual fraction of signal above baseline.
        self.adapted_fraction = self.adapted_fraction.clamp(0.0, 0.95);

        let above_baseline = (raw_rate - baseline).max(0.0);
        baseline + above_baseline * (1.0 - self.adapted_fraction)
    }

    /// Reset adaptation state completely.
    pub fn reset(&mut self) {
        self.adapted_fraction = 0.0;
        self.prev_raw_rate = 0.0;
    }
}

// ---------------------------------------------------------------------------
// Ia afferent model
// ---------------------------------------------------------------------------

/// Muscle spindle Ia afferent: dynamic + static sensitivity with fusimotor modulation.
#[derive(Debug, Clone)]
pub struct SpindleIaAfferent {
    params: SpindleIaParams,
    adaptation: AdaptationState,
}

impl SpindleIaAfferent {
    /// Create with given parameters.
    pub fn new(params: SpindleIaParams) -> Self {
        Self {
            params,
            adaptation: AdaptationState::new(),
        }
    }

    /// Create with default parameters.
    pub fn with_defaults() -> Self {
        Self::new(SpindleIaParams::default())
    }

    /// Compute raw (unadapted) Ia firing rate.
    ///
    /// `length`: current muscle length.
    /// `velocity`: stretch velocity (dL/dt, positive = lengthening).
    /// `fusimotor`: gamma fusimotor drive.
    pub fn raw_firing_rate(&self, length: f64, velocity: f64, fusimotor: &FusimotorDrive) -> f64 {
        let stretch = length - self.params.rest_length;
        let dynamic_component = self.params.k_dynamic * fusimotor.gamma_dynamic * velocity;
        let static_component = self.params.k_static * fusimotor.gamma_static * stretch;
        let raw = dynamic_component + static_component + self.params.baseline;
        raw.max(0.0)
    }

    /// Compute adapted Ia firing rate (call each time step).
    pub fn firing_rate(
        &mut self,
        length: f64,
        velocity: f64,
        fusimotor: &FusimotorDrive,
        dt: f64,
    ) -> f64 {
        let raw = self.raw_firing_rate(length, velocity, fusimotor);
        self.adaptation
            .apply(raw, self.params.baseline, self.params.tau_adapt, dt)
    }

    /// Access parameters.
    pub fn params(&self) -> &SpindleIaParams {
        &self.params
    }

    /// Mutable access to parameters.
    pub fn params_mut(&mut self) -> &mut SpindleIaParams {
        &mut self.params
    }

    /// Reset adaptation.
    pub fn reset_adaptation(&mut self) {
        self.adaptation.reset();
    }
}

// ---------------------------------------------------------------------------
// II afferent model
// ---------------------------------------------------------------------------

/// Muscle spindle group II afferent: static position sensitivity.
#[derive(Debug, Clone)]
pub struct SpindleIiAfferent {
    params: SpindleIiParams,
    adaptation: AdaptationState,
}

impl SpindleIiAfferent {
    /// Create with given parameters.
    pub fn new(params: SpindleIiParams) -> Self {
        Self {
            params,
            adaptation: AdaptationState::new(),
        }
    }

    /// Create with default parameters.
    pub fn with_defaults() -> Self {
        Self::new(SpindleIiParams::default())
    }

    /// Compute raw (unadapted) II firing rate.
    pub fn raw_firing_rate(&self, length: f64, fusimotor: &FusimotorDrive) -> f64 {
        let stretch = length - self.params.rest_length;
        let rate = self.params.k_static * fusimotor.gamma_static * stretch + self.params.baseline;
        rate.max(0.0)
    }

    /// Compute adapted II firing rate.
    pub fn firing_rate(&mut self, length: f64, fusimotor: &FusimotorDrive, dt: f64) -> f64 {
        let raw = self.raw_firing_rate(length, fusimotor);
        self.adaptation
            .apply(raw, self.params.baseline, self.params.tau_adapt, dt)
    }

    /// Access parameters.
    pub fn params(&self) -> &SpindleIiParams {
        &self.params
    }

    /// Mutable access to parameters.
    pub fn params_mut(&mut self) -> &mut SpindleIiParams {
        &mut self.params
    }

    /// Reset adaptation.
    pub fn reset_adaptation(&mut self) {
        self.adaptation.reset();
    }
}

// ---------------------------------------------------------------------------
// Golgi tendon organ model
// ---------------------------------------------------------------------------

/// Golgi tendon organ: force-dependent firing with adaptation.
#[derive(Debug, Clone)]
pub struct GolgiTendonOrgan {
    params: GtoParams,
    /// Elapsed time since last force onset (for adaptation ramp).
    elapsed: f64,
    /// Whether force was present in previous step.
    force_was_active: bool,
}

impl GolgiTendonOrgan {
    /// Create with given parameters.
    pub fn new(params: GtoParams) -> Self {
        Self {
            params,
            elapsed: 0.0,
            force_was_active: false,
        }
    }

    /// Create with default parameters.
    pub fn with_defaults() -> Self {
        Self::new(GtoParams::default())
    }

    /// Compute GTO firing rate.
    ///
    /// `force`: tendon force (N). Negative forces are clamped to 0.
    /// `dt`: simulation time step (seconds).
    ///
    /// Model: `f_GTO = K_f * F^exponent * (1 - exp(-t/tau_adapt)) + baseline`
    pub fn firing_rate(&mut self, force: f64, dt: f64) -> f64 {
        let f = force.max(0.0);
        let force_active = f > 1e-9;

        if force_active {
            if !self.force_was_active {
                // Force onset: reset elapsed time.
                self.elapsed = 0.0;
            }
            self.elapsed += dt;
        } else {
            self.elapsed = 0.0;
        }
        self.force_was_active = force_active;

        if !force_active {
            return self.params.baseline;
        }

        let safe_tau = self.params.tau_adapt.max(1e-9);
        let adaptation_factor = 1.0 - (-self.elapsed / safe_tau).exp();
        let force_term = f.powf(self.params.force_exponent);
        let rate = self.params.k_force * force_term * adaptation_factor + self.params.baseline;
        rate.max(0.0)
    }

    /// Access parameters.
    pub fn params(&self) -> &GtoParams {
        &self.params
    }

    /// Mutable access to parameters.
    pub fn params_mut(&mut self) -> &mut GtoParams {
        &mut self.params
    }

    /// Reset internal timing state.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.force_was_active = false;
    }
}

// ---------------------------------------------------------------------------
// Joint receptor model
// ---------------------------------------------------------------------------

/// Joint receptor: angle-dependent firing with range sensitivity.
///
/// The receptor fires maximally when the joint angle is near its preferred
/// angle and decays in a Gaussian-like fashion with distance from it.
#[derive(Debug, Clone)]
pub struct JointReceptor {
    params: JointReceptorParams,
    adaptation: AdaptationState,
}

impl JointReceptor {
    /// Create with given parameters.
    pub fn new(params: JointReceptorParams) -> Self {
        Self {
            params,
            adaptation: AdaptationState::new(),
        }
    }

    /// Create with default parameters.
    pub fn with_defaults() -> Self {
        Self::new(JointReceptorParams::default())
    }

    /// Compute raw firing rate.
    ///
    /// `angle`: current joint angle (radians).
    pub fn raw_firing_rate(&self, angle: f64) -> f64 {
        let delta = angle - self.params.preferred_angle;
        let half_w = self.params.range_half_width.max(1e-9);
        // Gaussian-like tuning curve.
        let normalized = delta / half_w;
        let tuning = (-0.5 * normalized * normalized).exp();
        let rate = self.params.peak_gain * tuning + self.params.baseline;
        rate.max(0.0)
    }

    /// Compute adapted firing rate.
    pub fn firing_rate(&mut self, angle: f64, dt: f64) -> f64 {
        let raw = self.raw_firing_rate(angle);
        self.adaptation
            .apply(raw, self.params.baseline, self.params.tau_adapt, dt)
    }

    /// Access parameters.
    pub fn params(&self) -> &JointReceptorParams {
        &self.params
    }

    /// Mutable access to parameters.
    pub fn params_mut(&mut self) -> &mut JointReceptorParams {
        &mut self.params
    }

    /// Reset adaptation.
    pub fn reset_adaptation(&mut self) {
        self.adaptation.reset();
    }
}

// ---------------------------------------------------------------------------
// Stretch reflex arc
// ---------------------------------------------------------------------------

/// Stretch reflex arc: Ia afferent -> alpha motor neuron -> muscle contraction.
///
/// Models the monosynaptic stretch reflex with configurable latency.
#[derive(Debug, Clone)]
pub struct StretchReflexArc {
    params: ReflexArcParams,
    /// Elapsed time since Ia firing exceeded threshold.
    elapsed_since_trigger: f64,
    /// Whether the reflex is currently active.
    active: bool,
    /// The Ia firing rate that triggered the reflex.
    trigger_rate: f64,
}

impl StretchReflexArc {
    /// Create with given parameters.
    pub fn new(params: ReflexArcParams) -> Self {
        Self {
            params,
            elapsed_since_trigger: 0.0,
            active: false,
            trigger_rate: 0.0,
        }
    }

    /// Create with default parameters.
    pub fn with_defaults() -> Self {
        Self::new(ReflexArcParams::default())
    }

    /// Compute reflex torque output.
    ///
    /// `ia_firing_rate`: current Ia afferent firing rate (impulses/s).
    /// `dt`: time step (seconds).
    ///
    /// Returns reflex-induced torque (N*m).
    ///
    /// Model: `tau = G_reflex * f_Ia * (1 - exp(-t/tau_delay))`
    /// where t is time since Ia exceeded threshold.
    pub fn compute_torque(&mut self, ia_firing_rate: f64, dt: f64) -> f64 {
        if ia_firing_rate > self.params.threshold {
            if !self.active {
                self.active = true;
                self.elapsed_since_trigger = 0.0;
                self.trigger_rate = ia_firing_rate;
            } else {
                self.trigger_rate = ia_firing_rate;
            }
            self.elapsed_since_trigger += dt;
        } else {
            if self.active {
                self.active = false;
            }
            self.elapsed_since_trigger = (self.elapsed_since_trigger - dt).max(0.0);
            if self.elapsed_since_trigger < 1e-9 {
                self.trigger_rate = 0.0;
                return 0.0;
            }
        }

        if self.trigger_rate < 1e-9 {
            return 0.0;
        }

        let safe_tau = self.params.tau_delay.max(1e-9);
        let delay_factor = 1.0 - (-self.elapsed_since_trigger / safe_tau).exp();
        let excess = (self.trigger_rate - self.params.threshold).max(0.0);
        self.params.gain * excess * delay_factor
    }

    /// Access parameters.
    pub fn params(&self) -> &ReflexArcParams {
        &self.params
    }

    /// Mutable access to parameters.
    pub fn params_mut(&mut self) -> &mut ReflexArcParams {
        &mut self.params
    }

    /// Reset reflex state.
    pub fn reset(&mut self) {
        self.elapsed_since_trigger = 0.0;
        self.active = false;
        self.trigger_rate = 0.0;
    }
}

// ---------------------------------------------------------------------------
// Integrated muscle proprioceptor
// ---------------------------------------------------------------------------

/// Complete proprioceptive sensor for a single muscle-tendon unit.
///
/// Combines Ia, II afferents, GTO, and stretch reflex in one struct.
#[derive(Debug, Clone)]
pub struct MuscleProprioceptor {
    /// Ia (dynamic) afferent.
    pub ia: SpindleIaAfferent,
    /// II (static) afferent.
    pub ii: SpindleIiAfferent,
    /// Golgi tendon organ.
    pub gto: GolgiTendonOrgan,
    /// Stretch reflex arc.
    pub reflex: StretchReflexArc,
    /// Fusimotor drive state.
    pub fusimotor: FusimotorDrive,
    /// Coactivation parameters.
    pub coactivation: CoactivationParams,
}

/// Output of a single-step proprioceptive evaluation.
#[derive(Debug, Clone, Default)]
pub struct ProprioceptiveOutput {
    /// Ia afferent firing rate (impulses/s).
    pub ia_rate: f64,
    /// II afferent firing rate (impulses/s).
    pub ii_rate: f64,
    /// GTO firing rate (impulses/s).
    pub gto_rate: f64,
    /// Reflex torque (N*m).
    pub reflex_torque: f64,
}

impl MuscleProprioceptor {
    /// Create with all default parameters.
    pub fn with_defaults() -> Self {
        Self {
            ia: SpindleIaAfferent::with_defaults(),
            ii: SpindleIiAfferent::with_defaults(),
            gto: GolgiTendonOrgan::with_defaults(),
            reflex: StretchReflexArc::with_defaults(),
            fusimotor: FusimotorDrive::default(),
            coactivation: CoactivationParams::default(),
        }
    }

    /// Create with specified rest length (shared by Ia and II).
    pub fn with_rest_length(rest_length: f64) -> Self {
        let mut p = Self::with_defaults();
        p.ia.params_mut().rest_length = rest_length;
        p.ii.params_mut().rest_length = rest_length;
        p
    }

    /// Set alpha drive and update fusimotor via coactivation.
    pub fn set_alpha_drive(&mut self, alpha_drive: f64) {
        self.fusimotor = FusimotorDrive::from_coactivation(alpha_drive, &self.coactivation);
    }

    /// Run one simulation step and return all proprioceptive outputs.
    ///
    /// `length`: current muscle length.
    /// `velocity`: stretch velocity (dL/dt).
    /// `force`: tendon force (N).
    /// `dt`: time step (seconds).
    pub fn step(
        &mut self,
        length: f64,
        velocity: f64,
        force: f64,
        dt: f64,
    ) -> ProprioceptiveOutput {
        let ia_rate = self.ia.firing_rate(length, velocity, &self.fusimotor, dt);
        let ii_rate = self.ii.firing_rate(length, &self.fusimotor, dt);
        let gto_rate = self.gto.firing_rate(force, dt);
        let reflex_torque = self.reflex.compute_torque(ia_rate, dt);

        ProprioceptiveOutput {
            ia_rate,
            ii_rate,
            gto_rate,
            reflex_torque,
        }
    }

    /// Reset all internal states (adaptation, reflex, GTO timing).
    pub fn reset(&mut self) {
        self.ia.reset_adaptation();
        self.ii.reset_adaptation();
        self.gto.reset();
        self.reflex.reset();
    }
}

// ---------------------------------------------------------------------------
// Multi-muscle proprioceptive integration
// ---------------------------------------------------------------------------

/// Integrates proprioceptive signals from multiple muscles.
#[derive(Debug, Clone)]
pub struct MultiMuscleProprioception {
    /// Named muscle proprioceptors.
    muscles: Vec<(String, MuscleProprioceptor)>,
    /// Optional joint receptors associated with this group.
    joint_receptors: Vec<(String, JointReceptor)>,
}

/// Aggregated output from multi-muscle integration.
#[derive(Debug, Clone, Default)]
pub struct IntegratedProprioception {
    /// Per-muscle outputs, keyed by name.
    pub muscle_outputs: Vec<(String, ProprioceptiveOutput)>,
    /// Per-joint receptor firing rates, keyed by name.
    pub joint_rates: Vec<(String, f64)>,
    /// Weighted mean Ia rate across all muscles.
    pub mean_ia_rate: f64,
    /// Weighted mean II rate across all muscles.
    pub mean_ii_rate: f64,
    /// Maximum GTO rate across all muscles (for overload detection).
    pub max_gto_rate: f64,
    /// Total reflex torque (sum across muscles).
    pub total_reflex_torque: f64,
}

impl MultiMuscleProprioception {
    /// Create an empty multi-muscle proprioception integrator.
    pub fn new() -> Self {
        Self {
            muscles: Vec::new(),
            joint_receptors: Vec::new(),
        }
    }

    /// Add a muscle proprioceptor with a given name.
    pub fn add_muscle(&mut self, name: &str, proprioceptor: MuscleProprioceptor) {
        self.muscles.push((name.to_string(), proprioceptor));
    }

    /// Add a joint receptor with a given name.
    pub fn add_joint_receptor(&mut self, name: &str, receptor: JointReceptor) {
        self.joint_receptors.push((name.to_string(), receptor));
    }

    /// Get mutable reference to a named muscle.
    pub fn muscle_mut(&mut self, name: &str) -> Option<&mut MuscleProprioceptor> {
        self.muscles
            .iter_mut()
            .find(|(n, _)| n == name)
            .map(|(_, m)| m)
    }

    /// Get reference to a named muscle.
    pub fn muscle(&self, name: &str) -> Option<&MuscleProprioceptor> {
        self.muscles.iter().find(|(n, _)| n == name).map(|(_, m)| m)
    }

    /// Number of muscles.
    pub fn muscle_count(&self) -> usize {
        self.muscles.len()
    }

    /// Number of joint receptors.
    pub fn joint_receptor_count(&self) -> usize {
        self.joint_receptors.len()
    }

    /// Run one integration step.
    ///
    /// `muscle_inputs`: slice of `(name, length, velocity, force)`.
    /// `joint_angles`: slice of `(name, angle)`.
    /// `dt`: time step (seconds).
    pub fn step(
        &mut self,
        muscle_inputs: &[(&str, f64, f64, f64)],
        joint_angles: &[(&str, f64)],
        dt: f64,
    ) -> IntegratedProprioception {
        let mut muscle_outputs = Vec::with_capacity(self.muscles.len());
        let mut sum_ia = 0.0;
        let mut sum_ii = 0.0;
        let mut max_gto = 0.0_f64;
        let mut total_reflex = 0.0;
        let mut count = 0u32;

        for (mname, proprioceptor) in &mut self.muscles {
            if let Some(&(_, length, velocity, force)) = muscle_inputs
                .iter()
                .find(|(n, _, _, _)| *n == mname.as_str())
            {
                let out = proprioceptor.step(length, velocity, force, dt);
                sum_ia += out.ia_rate;
                sum_ii += out.ii_rate;
                max_gto = max_gto.max(out.gto_rate);
                total_reflex += out.reflex_torque;
                count += 1;
                muscle_outputs.push((mname.clone(), out));
            }
        }

        let mut joint_rates = Vec::with_capacity(self.joint_receptors.len());
        for (jname, receptor) in &mut self.joint_receptors {
            if let Some(&(_, angle)) = joint_angles.iter().find(|(n, _)| *n == jname.as_str()) {
                let rate = receptor.firing_rate(angle, dt);
                joint_rates.push((jname.clone(), rate));
            }
        }

        let n = if count > 0 { count as f64 } else { 1.0 };
        IntegratedProprioception {
            muscle_outputs,
            joint_rates,
            mean_ia_rate: sum_ia / n,
            mean_ii_rate: sum_ii / n,
            max_gto_rate: max_gto,
            total_reflex_torque: total_reflex,
        }
    }

    /// Set alpha drive (and update fusimotor via coactivation) for all muscles.
    pub fn set_alpha_drive_all(&mut self, alpha_drive: f64) {
        for (_, m) in &mut self.muscles {
            m.set_alpha_drive(alpha_drive);
        }
    }

    /// Reset all internal states.
    pub fn reset_all(&mut self) {
        for (_, m) in &mut self.muscles {
            m.reset();
        }
        for (_, j) in &mut self.joint_receptors {
            j.reset_adaptation();
        }
    }
}

impl Default for MultiMuscleProprioception {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Legacy API tests (preserved from stub) ---

    #[test]
    fn test_new_muscle_sensor() {
        /* new sensor starts at default state */
        let s = new_muscle_sensor();
        assert!((s.muscle_length - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_spindle_ia_firing() {
        /* Ia fires based on velocity and length */
        let s = new_muscle_sensor();
        let f = spindle_Ia_firing(&s);
        assert!(f >= 0.0);
    }

    #[test]
    fn test_spindle_ii_firing() {
        /* II fires based on length */
        let s = new_muscle_sensor();
        let f = spindle_II_firing(&s);
        assert!((f - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_gto_firing() {
        /* GTO fires based on force */
        let mut s = new_muscle_sensor();
        sensor_update(&mut s, 1.0, 0.0, 10.0);
        let f = gto_firing(&s);
        assert!((f - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_sensor_update() {
        /* sensor_update changes all fields */
        let mut s = new_muscle_sensor();
        sensor_update(&mut s, 2.0, 0.5, 5.0);
        assert!((s.muscle_length - 2.0).abs() < 1e-5);
        assert!((s.velocity - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_sensor_is_overloaded_false() {
        /* not overloaded at low force */
        let s = new_muscle_sensor();
        assert!(!sensor_is_overloaded(&s, 10.0));
    }

    #[test]
    fn test_sensor_is_overloaded_true() {
        /* overloaded at high force relative to threshold */
        let mut s = new_muscle_sensor();
        sensor_update(&mut s, 1.0, 0.0, 100.0);
        assert!(sensor_is_overloaded(&s, 10.0));
    }

    // --- Ia afferent tests ---

    #[test]
    fn test_ia_baseline_at_rest() {
        let ia = SpindleIaAfferent::with_defaults();
        let fusi = FusimotorDrive::default();
        let rate = ia.raw_firing_rate(1.0, 0.0, &fusi);
        assert!(
            (rate - ia.params().baseline).abs() < 1e-9,
            "Ia at rest should be baseline, got {rate}"
        );
    }

    #[test]
    fn test_ia_velocity_sensitivity() {
        let ia = SpindleIaAfferent::with_defaults();
        let fusi = FusimotorDrive::default();
        let rate_slow = ia.raw_firing_rate(1.0, 1.0, &fusi);
        let rate_fast = ia.raw_firing_rate(1.0, 5.0, &fusi);
        assert!(
            rate_fast > rate_slow,
            "Ia should increase with velocity: slow={rate_slow}, fast={rate_fast}"
        );
    }

    #[test]
    fn test_ia_stretch_sensitivity() {
        let ia = SpindleIaAfferent::with_defaults();
        let fusi = FusimotorDrive::default();
        let rate_short = ia.raw_firing_rate(1.0, 0.0, &fusi);
        let rate_stretched = ia.raw_firing_rate(1.5, 0.0, &fusi);
        assert!(
            rate_stretched > rate_short,
            "Ia should increase with stretch"
        );
    }

    #[test]
    fn test_ia_gamma_dynamic_modulation() {
        let ia = SpindleIaAfferent::with_defaults();
        let fusi_low = FusimotorDrive {
            gamma_dynamic: 0.2,
            gamma_static: 1.0,
        };
        let fusi_high = FusimotorDrive {
            gamma_dynamic: 1.0,
            gamma_static: 1.0,
        };
        let rate_low = ia.raw_firing_rate(1.0, 3.0, &fusi_low);
        let rate_high = ia.raw_firing_rate(1.0, 3.0, &fusi_high);
        assert!(
            rate_high > rate_low,
            "Higher gamma_dynamic should increase velocity response"
        );
    }

    #[test]
    fn test_ia_adaptation() {
        let mut ia = SpindleIaAfferent::with_defaults();
        let fusi = FusimotorDrive::default();
        let dt = 0.01;
        let mut rates = Vec::new();
        for _ in 0..100 {
            let r = ia.firing_rate(1.5, 0.0, &fusi, dt);
            rates.push(r);
        }
        let first = rates[0];
        let last = rates[rates.len() - 1];
        assert!(
            last < first,
            "Ia should adapt (decrease) over time: first={first}, last={last}"
        );
        assert!(
            last >= ia.params().baseline,
            "Adapted rate should stay >= baseline"
        );
    }

    // --- II afferent tests ---

    #[test]
    fn test_ii_position_sensitivity() {
        let ii = SpindleIiAfferent::with_defaults();
        let fusi = FusimotorDrive::default();
        let rate_rest = ii.raw_firing_rate(1.0, &fusi);
        let rate_stretched = ii.raw_firing_rate(1.5, &fusi);
        assert!(
            rate_stretched > rate_rest,
            "II should increase with stretch"
        );
    }

    #[test]
    fn test_ii_gamma_static_modulation() {
        let ii = SpindleIiAfferent::with_defaults();
        let fusi_low = FusimotorDrive {
            gamma_dynamic: 1.0,
            gamma_static: 0.3,
        };
        let fusi_high = FusimotorDrive {
            gamma_dynamic: 1.0,
            gamma_static: 1.0,
        };
        let rate_low = ii.raw_firing_rate(1.3, &fusi_low);
        let rate_high = ii.raw_firing_rate(1.3, &fusi_high);
        assert!(
            rate_high > rate_low,
            "Higher gamma_static should increase II response"
        );
    }

    // --- GTO tests ---

    #[test]
    fn test_gto_force_response() {
        let mut gto = GolgiTendonOrgan::with_defaults();
        let dt = 0.01;
        let mut rate = 0.0;
        for _ in 0..50 {
            rate = gto.firing_rate(20.0, dt);
        }
        assert!(rate > 0.0, "GTO should fire under force");
    }

    #[test]
    fn test_gto_zero_force() {
        let mut gto = GolgiTendonOrgan::with_defaults();
        let rate = gto.firing_rate(0.0, 0.01);
        assert!(
            (rate - gto.params().baseline).abs() < 1e-9,
            "GTO with zero force should return baseline"
        );
    }

    #[test]
    fn test_gto_adaptation_ramp() {
        let mut gto = GolgiTendonOrgan::with_defaults();
        let dt = 0.01;
        let r1 = gto.firing_rate(10.0, dt);
        let r2 = gto.firing_rate(10.0, dt);
        let r3 = gto.firing_rate(10.0, dt);
        assert!(
            r2 >= r1 && r3 >= r2,
            "GTO should ramp up: r1={r1}, r2={r2}, r3={r3}"
        );
    }

    #[test]
    fn test_gto_sqrt_nonlinearity() {
        let mut gto1 = GolgiTendonOrgan::with_defaults();
        let mut gto2 = GolgiTendonOrgan::with_defaults();
        let dt = 0.001;
        let mut r_low = 0.0;
        let mut r_high = 0.0;
        for _ in 0..1000 {
            r_low = gto1.firing_rate(1.0, dt);
            r_high = gto2.firing_rate(100.0, dt);
        }
        let ratio = r_high / r_low.max(1e-12);
        assert!(
            ratio < 50.0,
            "GTO sqrt nonlinearity: ratio should be << 100, got {ratio}"
        );
    }

    // --- Joint receptor tests ---

    #[test]
    fn test_joint_receptor_peak_at_preferred() {
        let jr = JointReceptor::with_defaults();
        let at_preferred = jr.raw_firing_rate(0.0);
        let away = jr.raw_firing_rate(1.0);
        assert!(
            at_preferred > away,
            "Joint receptor should fire most at preferred angle"
        );
    }

    #[test]
    fn test_joint_receptor_symmetric() {
        let jr = JointReceptor::with_defaults();
        let pos = jr.raw_firing_rate(0.3);
        let neg = jr.raw_firing_rate(-0.3);
        assert!(
            (pos - neg).abs() < 1e-9,
            "Joint receptor should be symmetric around preferred angle"
        );
    }

    // --- Stretch reflex arc tests ---

    #[test]
    fn test_reflex_below_threshold() {
        let mut reflex = StretchReflexArc::with_defaults();
        let torque = reflex.compute_torque(2.0, 0.01);
        assert!(torque.abs() < 1e-12, "No reflex below threshold");
    }

    #[test]
    fn test_reflex_above_threshold() {
        let mut reflex = StretchReflexArc::with_defaults();
        let dt = 0.01;
        let mut torque = 0.0;
        for _ in 0..20 {
            torque = reflex.compute_torque(20.0, dt);
        }
        assert!(
            torque > 0.0,
            "Reflex should produce torque when Ia > threshold"
        );
    }

    #[test]
    fn test_reflex_latency() {
        let mut reflex = StretchReflexArc::with_defaults();
        let dt = 0.001;
        let t1 = reflex.compute_torque(30.0, dt);
        let mut t_late = 0.0;
        for _ in 0..100 {
            t_late = reflex.compute_torque(30.0, dt);
        }
        assert!(
            t_late > t1,
            "Reflex torque should ramp up (latency): t1={t1}, t_late={t_late}"
        );
    }

    // --- Coactivation tests ---

    #[test]
    fn test_coactivation() {
        let params = CoactivationParams::default();
        let fusi = FusimotorDrive::from_coactivation(0.5, &params);
        assert!(fusi.gamma_dynamic > 0.0 && fusi.gamma_dynamic < 1.0);
        assert!(fusi.gamma_static > 0.0 && fusi.gamma_static < 1.0);
    }

    #[test]
    fn test_coactivation_zero_alpha() {
        let params = CoactivationParams::default();
        let fusi = FusimotorDrive::from_coactivation(0.0, &params);
        assert!(
            fusi.gamma_dynamic.abs() < 1e-12,
            "Zero alpha should give zero gamma_dynamic"
        );
        assert!(
            fusi.gamma_static.abs() < 1e-12,
            "Zero alpha should give zero gamma_static"
        );
    }

    // --- Integrated muscle proprioceptor tests ---

    #[test]
    fn test_muscle_proprioceptor_step() {
        let mut mp = MuscleProprioceptor::with_defaults();
        let out = mp.step(1.0, 0.0, 0.0, 0.01);
        assert!(out.ia_rate >= 0.0);
        assert!(out.ii_rate >= 0.0);
        assert!(out.gto_rate >= 0.0);
    }

    #[test]
    fn test_muscle_proprioceptor_coactivation() {
        let mut mp = MuscleProprioceptor::with_defaults();
        mp.set_alpha_drive(0.8);
        assert!(mp.fusimotor.gamma_dynamic > 0.0);
        assert!(mp.fusimotor.gamma_static > 0.0);
    }

    #[test]
    fn test_muscle_proprioceptor_stretch_increases_ia() {
        let mut mp = MuscleProprioceptor::with_defaults();
        let dt = 0.01;
        let out_rest = mp.step(1.0, 0.0, 0.0, dt);
        mp.reset();
        let out_stretch = mp.step(1.5, 2.0, 0.0, dt);
        assert!(
            out_stretch.ia_rate > out_rest.ia_rate,
            "Stretch should increase Ia rate"
        );
    }

    // --- Multi-muscle integration tests ---

    #[test]
    fn test_multi_muscle_basic() {
        let mut multi = MultiMuscleProprioception::new();
        multi.add_muscle("biceps", MuscleProprioceptor::with_defaults());
        multi.add_muscle("triceps", MuscleProprioceptor::with_defaults());
        multi.add_joint_receptor("elbow", JointReceptor::with_defaults());

        assert_eq!(multi.muscle_count(), 2);
        assert_eq!(multi.joint_receptor_count(), 1);
    }

    #[test]
    fn test_multi_muscle_step() {
        let mut multi = MultiMuscleProprioception::new();
        multi.add_muscle("biceps", MuscleProprioceptor::with_defaults());
        multi.add_muscle("triceps", MuscleProprioceptor::with_defaults());
        multi.add_joint_receptor("elbow", JointReceptor::with_defaults());

        let inputs = [("biceps", 1.2, 0.5, 5.0), ("triceps", 0.9, -0.3, 3.0)];
        let joints = [("elbow", 0.1)];
        let out = multi.step(&inputs, &joints, 0.01);

        assert_eq!(out.muscle_outputs.len(), 2);
        assert_eq!(out.joint_rates.len(), 1);
        assert!(out.mean_ia_rate > 0.0);
    }

    #[test]
    fn test_multi_muscle_set_alpha_all() {
        let mut multi = MultiMuscleProprioception::new();
        multi.add_muscle("a", MuscleProprioceptor::with_defaults());
        multi.add_muscle("b", MuscleProprioceptor::with_defaults());
        multi.set_alpha_drive_all(0.6);
        let m_a = multi.muscle("a").expect("muscle a should exist");
        assert!(m_a.fusimotor.gamma_dynamic > 0.0);
        let m_b = multi.muscle("b").expect("muscle b should exist");
        assert!(m_b.fusimotor.gamma_dynamic > 0.0);
    }

    #[test]
    fn test_adaptation_state_reset() {
        let mut adapt = AdaptationState::new();
        let _ = adapt.apply(50.0, 1.0, 0.2, 0.01);
        let _ = adapt.apply(50.0, 1.0, 0.2, 0.01);
        adapt.reset();
        assert!(adapt.adapted_fraction.abs() < 1e-12);
    }

    #[test]
    fn test_gto_reset() {
        let mut gto = GolgiTendonOrgan::with_defaults();
        let _ = gto.firing_rate(10.0, 0.01);
        gto.reset();
        assert!(gto.elapsed.abs() < 1e-12);
    }

    #[test]
    fn test_ia_formula_correctness() {
        let params = SpindleIaParams {
            k_dynamic: 2.0,
            k_static: 3.0,
            rest_length: 1.0,
            baseline: 5.0,
            tau_adapt: 0.2,
        };
        let ia = SpindleIaAfferent::new(params);
        let fusi = FusimotorDrive {
            gamma_dynamic: 0.8,
            gamma_static: 0.6,
        };
        let rate = ia.raw_firing_rate(1.5, 2.0, &fusi);
        // Expected: 2.0*0.8*2.0 + 3.0*0.6*0.5 + 5.0 = 3.2 + 0.9 + 5.0 = 9.1
        assert!(
            (rate - 9.1).abs() < 1e-9,
            "Ia formula check: expected 9.1, got {rate}"
        );
    }

    #[test]
    fn test_ii_formula_correctness() {
        let params = SpindleIiParams {
            k_static: 4.0,
            rest_length: 1.0,
            baseline: 2.0,
            tau_adapt: 0.5,
        };
        let ii = SpindleIiAfferent::new(params);
        let fusi = FusimotorDrive {
            gamma_dynamic: 1.0,
            gamma_static: 0.7,
        };
        let rate = ii.raw_firing_rate(1.3, &fusi);
        // Expected: 4.0*0.7*0.3 + 2.0 = 0.84 + 2.0 = 2.84
        assert!(
            (rate - 2.84).abs() < 1e-9,
            "II formula check: expected 2.84, got {rate}"
        );
    }

    #[test]
    fn test_reflex_torque_formula() {
        let params = ReflexArcParams {
            gain: 0.5,
            tau_delay: 0.01,
            threshold: 10.0,
        };
        let mut reflex = StretchReflexArc::new(params);
        let dt = 0.001;
        let ia_rate = 30.0;
        let mut torque = 0.0;
        for _ in 0..1000 {
            torque = reflex.compute_torque(ia_rate, dt);
        }
        // Steady state: delay_factor -> 1, excess = 20
        // torque -> 0.5 * 20 = 10.0
        assert!(
            (torque - 10.0).abs() < 0.5,
            "Reflex steady state: expected ~10.0, got {torque}"
        );
    }

    #[test]
    fn test_negative_force_clamped() {
        let mut gto = GolgiTendonOrgan::with_defaults();
        let rate = gto.firing_rate(-5.0, 0.01);
        assert!(
            (rate - gto.params().baseline).abs() < 1e-9,
            "Negative force should be clamped to zero"
        );
    }

    #[test]
    fn test_ia_negative_velocity_floor() {
        let ia = SpindleIaAfferent::with_defaults();
        let fusi = FusimotorDrive::default();
        let rate = ia.raw_firing_rate(1.0, -100.0, &fusi);
        assert!(rate >= 0.0, "Ia rate should not go negative");
    }

    #[test]
    fn test_muscle_proprioceptor_rest_length() {
        let mp = MuscleProprioceptor::with_rest_length(0.8);
        assert!(
            (mp.ia.params().rest_length - 0.8).abs() < 1e-9,
            "Ia rest length should be set"
        );
        assert!(
            (mp.ii.params().rest_length - 0.8).abs() < 1e-9,
            "II rest length should be set"
        );
    }
}
