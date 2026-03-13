// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Auto-generated module re-exports — do not edit directly.

#[path = "rigid_tree.rs"]
pub mod rigid_tree;
pub use rigid_tree::{build_chain, RigidTree, RigidTreeBody};

#[path = "contact_lcp.rs"]
pub mod contact_lcp;
pub use contact_lcp::{
    apply_lcp_impulse, build_1d_contact_lcp, contact_count,
    friction_impulse as lcp_friction_impulse, lcp_gauss_seidel, relative_normal_vel, LcpContact,
    LcpSystem,
};

#[path = "warm_start_v2.rs"]
pub mod warm_start_v2;
pub use warm_start_v2::{apply_warm_start, blend_impulse, WarmImpulse, WarmStartCache};

#[path = "time_stepping.rs"]
pub mod time_stepping;
pub use time_stepping::{
    euler_stability_limit, verlet_stability_limit, TimeStepConfig, TimeStepper,
};

#[path = "solver_stats.rs"]
pub mod solver_stats;
pub use solver_stats::{compute_residual, jacobi_solve_stats, SolveStats, SolverStatistics};

#[path = "collision_detection.rs"]
pub mod collision_detection;
pub use collision_detection::{
    broad_phase_brute, broad_phase_sap_x, detect_sphere_contacts as cd_detect_sphere_contacts,
    narrow_sphere_plane, narrow_sphere_sphere, overlap_count, Aabb3, BroadPhaseObject,
    ContactPoint as CdContactPoint, OverlapPair as CdOverlapPair, Sphere as CdSphere,
};

#[path = "lattice_boltzmann_v2.rs"]
pub mod lattice_boltzmann_v2;
pub use lattice_boltzmann_v2::LatticeBoltzmannV2;

#[path = "sph_density.rs"]
pub mod sph_density;
pub use sph_density::{
    average_density, compute_density as sph_compute_density, cubic_spline_kernel,
    dist3 as sph_dist3, estimate_density_at, SphParticleDensity,
};

#[path = "sph_pressure_force.rs"]
pub mod sph_pressure_force;
pub use sph_pressure_force::{
    clear_forces as sph_clear_forces, compute_pressure_forces, integrate_pressure, kernel_gradient,
    pressure_eos, SphPressureParticle,
};

#[path = "sph_viscosity.rs"]
pub mod sph_viscosity;
pub use sph_viscosity::{
    apply_viscosity, clear_visc_forces, compute_viscosity_forces, visc_dissipation,
    viscosity_kernel_laplacian, SphViscParticle,
};

#[path = "eulerian_advection.rs"]
pub mod eulerian_advection;
pub use eulerian_advection::{field_max, fill_circle, AdvectionGrid};

#[path = "vorticity_confinement.rs"]
pub mod vorticity_confinement;
pub use vorticity_confinement::{max_vorticity, VorticityGrid};

#[path = "level_set_advect.rs"]
pub mod level_set_advect;
pub use level_set_advect::{gradient_magnitude, LevelSetGrid};

#[path = "marching_squares.rs"]
pub mod marching_squares;
pub use marching_squares::{
    count_segments, marching_squares as extract_contour, total_contour_length, Segment,
};

#[path = "rigid_contact_patch.rs"]
pub mod rigid_contact_patch;
pub use rigid_contact_patch::{prune_contacts, ContactPatch, ContactPoint as RigidContactPoint};

#[path = "friction_cone_v2.rs"]
pub mod friction_cone_v2;
pub use friction_cone_v2::{
    friction_clamp, sliding_direction, tangential_component, FrictionConeV2,
};

#[path = "bilateral_constraint.rs"]
pub mod bilateral_constraint;
pub use bilateral_constraint::{
    apply_impulse as bc_apply_impulse, position_violation, BilateralConstraint,
};

#[path = "unilateral_constraint.rs"]
pub mod unilateral_constraint;
pub use unilateral_constraint::{clamp_lambdas, total_normal_impulse, UnilateralConstraint};

#[path = "penalty_force.rs"]
pub mod penalty_force;
pub use penalty_force::{apply_penalty_forces, total_penalty_energy, PenaltyElement};

#[path = "augmented_lagrangian.rs"]
pub mod augmented_lagrangian;
pub use augmented_lagrangian::{
    al_gradient_norm, AugmentedLagrangianConstraint, AugmentedLagrangianSolver,
};

#[path = "sequential_impulse.rs"]
pub mod sequential_impulse;
pub use sequential_impulse::{velocity_correction, ImpulseConstraint, SequentialImpulseSolver};

#[path = "warm_starting.rs"]
pub mod warm_starting;
pub use warm_starting::{
    warm_start_hit_rate, CachedImpulse, WarmStartCache as WarmStartingCache, WarmStartKey,
};

#[path = "soft_body_mass_spring.rs"]
pub mod soft_body_mass_spring;
pub use soft_body_mass_spring::{
    ms_add_particle, ms_add_spring, ms_kinetic_energy, ms_potential_energy, ms_step, new_ms_body,
    MsParticle, MsSoftBody, MsSpring,
};

#[path = "muscle_activation.rs"]
pub mod muscle_activation;
pub use muscle_activation::{
    force_length, force_velocity, muscle_activate, muscle_force, muscle_set_activation, new_muscle,
    passive_force, total_muscle_force, Muscle,
};

#[path = "tendon_model.rs"]
pub mod tendon_model;
pub use tendon_model::{
    new_tendon, tendon_elongation, tendon_force, tendon_is_taut, tendon_scale_stiffness,
    tendon_set_length, tendon_stored_energy, tendon_strain, tendon_tangent_stiffness, Tendon,
};

#[path = "joint_torque.rs"]
pub mod joint_torque;
pub use joint_torque::{
    jt_in_limits, jt_kinetic_energy, jt_moment_arm, jt_set_torque, jt_step, jt_stiffness_torque,
    jt_torque_from_force, new_joint_torque, JointTorque,
};

#[path = "skin_sliding.rs"]
pub mod skin_sliding;
pub use skin_sliding::{
    new_skin_layer, skin_adhesion_energy, skin_apply_slide, skin_offset_magnitude, skin_reset,
    skin_stiffness, skin_update_ref, SkinLayer,
};

#[path = "fascia_model.rs"]
pub mod fascia_model;
pub use fascia_model::{
    fascia_elastic_force, fascia_is_taut, fascia_set_hydration, fascia_set_length,
    fascia_stored_energy, fascia_strain, fascia_total_force, fascia_viscous_force, new_fascia,
    FasciaElement,
};

#[path = "adipose_sim.rs"]
pub mod adipose_sim;
pub use adipose_sim::{
    adipos_impulse, adipos_is_settled, adipos_kinetic_energy, adipos_offset_mag,
    adipos_potential_energy, adipos_reset, adipos_step, new_adipos_node, AdiposNode,
};

#[path = "cartilage_model.rs"]
pub mod cartilage_model;
pub use cartilage_model::{
    cartilage_contact_stress, cartilage_effective_modulus, cartilage_is_compressed,
    cartilage_repair, cartilage_reset, cartilage_step, cartilage_strain, new_cartilage, Cartilage,
};

#[path = "bone_deform.rs"]
pub mod bone_deform;
pub use bone_deform::{
    bone_axial_deformation, bone_axial_stiffness, bone_axial_stress, bone_bending_deflection,
    bone_bending_stress, bone_is_fractured, bone_set_axial, bone_set_bending, new_bone, Bone,
};

#[path = "blood_pressure_sim.rs"]
pub mod blood_pressure_sim;
pub use blood_pressure_sim::{
    bp_heartbeat, bp_is_normal, bp_mean_arterial_pressure, bp_period, bp_pressure_mmhg,
    bp_set_heart_rate, bp_simulate_cycle, bp_step, new_blood_pressure, BloodPressure,
};

#[path = "lung_mechanics.rs"]
pub mod lung_mechanics;
pub use lung_mechanics::{
    lung_above_frc, lung_breathe, lung_compliance, lung_elastic_pressure, lung_set_volume,
    lung_step, lung_tidal_volume, lung_ventilation, new_lung, Lung,
};

#[path = "eye_pressure.rs"]
pub mod eye_pressure;
pub use eye_pressure::{
    iop_apply_medication, iop_is_elevated, iop_is_normal, iop_mmhg, iop_set_production,
    iop_steady_state, iop_step, new_eye_pressure, EyePressure,
};

#[path = "fluid_muscle.rs"]
pub mod fluid_muscle;
pub use fluid_muscle::{
    fluid_muscle_force, fm_blocked_force, fm_contraction_ratio, fm_is_contracting, fm_set_length,
    fm_set_pressure, fm_update_angle, new_fluid_muscle, FluidMuscle,
};

#[path = "piezo_actuator.rs"]
pub mod piezo_actuator;
pub use piezo_actuator::{
    new_piezo, piezo_coupling_k2, piezo_force, piezo_free_stroke, piezo_is_full_stroke,
    piezo_power, piezo_set_voltage, piezo_update_extension, PiezoActuator,
};

#[path = "shape_memory_alloy.rs"]
pub mod shape_memory_alloy;
pub use shape_memory_alloy::{
    new_sma_spring, sma_effective_modulus, sma_is_actuated, sma_martensite_fraction, sma_phase,
    sma_recovery_force, sma_set_temperature, sma_update_strain, SmaPhase, SmaSpring,
};

#[path = "magnetorheological_fluid.rs"]
pub mod magnetorheological_fluid;
pub use magnetorheological_fluid::{
    mr_coil_power, mr_damping_force, mr_effective_viscosity, mr_is_off, mr_is_on, mr_set_field,
    mr_set_shear_rate, mr_shear_stress, mr_yield_stress, new_mr_fluid, MrFluid,
};

#[path = "tissue_deform.rs"]
pub mod tissue_deform;
pub use tissue_deform::{
    apply_tissue_forces, integrate_tissue, rest_length as tissue_rest_length, TissueMesh,
    TissueNode, TissueParams,
};

#[path = "fracture_mechanics.rs"]
pub mod fracture_mechanics;
pub use fracture_mechanics::{
    analyze_fracture, critical_crack_size, critical_stress as fracture_critical_stress,
    energy_release_rate_mode1, stress_intensity_mode1, stress_intensity_mode2, will_propagate,
    FractureMaterial, FractureMode, FractureResult,
};

#[path = "crack_propagation.rs"]
pub mod crack_propagation;
pub use crack_propagation::{
    advance_crack, current_ki, cycles_to_failure as crack_cycles_to_failure, is_fractured,
    paris_law_da_dn, Crack, CrackPoint, ParisLawParams,
};

#[path = "delamination_model.rs"]
pub mod delamination_model;
pub use delamination_model::{
    bilinear_traction, damage_variable, dissipated_energy, failed_element_count, grow_delamination,
    is_delaminated, update_interface, CohesiveParams, InterfaceElement,
};

#[path = "creep_model.rs"]
pub mod creep_model;
pub use creep_model::{
    integrate_creep, is_creep_damaged, larson_miller_param, monkman_grant_check,
    multiaxial_creep_rate, norton_creep_rate, rupture_time_h, CreepParams, CreepState,
};

#[path = "fatigue_model.rs"]
pub mod fatigue_model;
pub use fatigue_model::{
    accumulate_damage, cycles_to_failure_sn, goodman_correction, remaining_cycles,
    stress_ratio as fatigue_stress_ratio, FatigueDamage, SnCurveParams, StressCycle,
};

#[path = "plasticity_model.rs"]
pub mod plasticity_model;
pub use plasticity_model::{
    current_yield_stress, is_yielding, plastic_strain_increment, radial_return,
    shear_modulus as plasticity_shear_modulus, von_mises_stress, yield_function, PlasticState,
    PlasticityParams,
};

#[path = "hyperelastic_model.rs"]
pub mod hyperelastic_model;
pub use hyperelastic_model::{
    cauchy_stress_isotropic, hydrostatic_pressure as hyper_hydrostatic_pressure,
    is_stable as hyper_is_stable, isochoric_energy,
    strain_energy_density as neo_hookean_strain_energy, volumetric_energy, DeformGrad,
    NeoHookeanParams,
};

#[path = "viscoelastic_model.rs"]
pub mod viscoelastic_model;
pub use viscoelastic_model::{
    loss_tangent, maxwell_loss_modulus, maxwell_storage_modulus, KelvinVoigtModel, MaxwellModel,
    SlsModel,
};

#[path = "anisotropic_material.rs"]
pub mod anisotropic_material;
pub use anisotropic_material::{rotate_stiffness_z, StiffnessTensor};

#[path = "composite_material.rs"]
pub mod composite_material;
pub use composite_material::{CompositeMaterial, Layer as CompositeLayer};

#[path = "porous_media.rs"]
pub mod porous_media;
pub use porous_media::{
    advance_pressure_1d, darcy_flux, darcy_flux_3d, is_darcy_regime, kozeny_carman_permeability,
    porous_reynolds, seepage_velocity, storage_coefficient, PorousCell, PorousParams,
};

#[path = "granular_material.rs"]
pub mod granular_material;
pub use granular_material::{
    angle_of_repose_deg, avalanche_threshold_height, drucker_prager_yield,
    mohr_coulomb_shear_strength, overburden_pressure as granular_overburden, settle_pile,
    will_slope_fail, GranularParams, GranularPile,
};

#[path = "foam_material.rs"]
pub mod foam_material;
pub use foam_material::{
    energy_absorption, foam_stress, gibson_ashby_plateau, is_elastic as foam_is_elastic,
    open_cell_poisson, specific_energy_absorption, update_foam_state, FoamParams, FoamState,
};

#[path = "auxetic_material.rs"]
pub mod auxetic_material;
pub use auxetic_material::{
    anisotropy_ratio, compute_auxetic_properties, effective_modulus_x, effective_poisson_ratio,
    impact_absorption_factor, is_auxetic, lateral_strain,
    relative_density as auxetic_relative_density, AuxeticParams, AuxeticProperties,
};

#[path = "metamaterial_stub.rs"]
pub mod metamaterial_stub;
pub use metamaterial_stub::{
    bragg_frequency_hz, effective_mass_density, is_in_bandgap, locally_resonant_bandgap,
    resonator_frequency_hz, transmission_loss_db, wave_speed as metamaterial_wave_speed,
    BandgapInfo, MetamaterialKind, MetamaterialParams,
};

#[path = "rigid_body_tree.rs"]
pub mod rigid_body_tree;
pub use rigid_body_tree::{
    count_leaves, find_root, forward_dynamics_step, total_kinetic_energy, ArtBody, ArtBodyTree,
};

#[path = "floating_base.rs"]
pub mod floating_base;
pub use floating_base::{
    apply_wrench as apply_floating_wrench, is_at_rest, linear_kinetic_energy, normalize_quat,
    reset_velocity, FloatingBaseState,
};

#[path = "zero_moment_point.rs"]
pub mod zero_moment_point;
pub use zero_moment_point::{
    compute_zmp, evaluate_zmp_stability, zmp_margin, zmp_stability_label, SupportPolygon, ZmpResult,
};

#[path = "capture_point.rs"]
pub mod capture_point;
pub use capture_point::{
    capture_point_distance_to_centre, capture_point_stable, capture_point_velocity,
    compute_capture_point, natural_frequency as lip_natural_frequency, CapturePoint,
};

#[path = "com_trajectory.rs"]
pub mod com_trajectory;
pub use com_trajectory::{
    average_com_height, linear_com_trajectory, sample_com_trajectory, ComTrajectory, ComWaypoint,
};

#[path = "foot_placement.rs"]
pub mod foot_placement;
pub use foot_placement::{
    footstep_alternates, footstep_distance, footstep_feasible, footstep_path_length,
    plan_footsteps, FootPlacementConfig, Footstep,
};

#[path = "gait_scheduler.rs"]
pub mod gait_scheduler;
pub use gait_scheduler::{
    gait_phase, is_double_support, phase_to_leg_phase, scheduler_phases, time_to_next_transition,
    GaitScheduler, GaitType, LegPhase,
};

#[path = "step_pattern.rs"]
pub mod step_pattern;
pub use step_pattern::{
    active_step, generate_walk_pattern, pattern_alternates, pattern_total_duration, StepEvent,
    StepPattern,
};

#[path = "balance_controller.rs"]
pub mod balance_controller;
pub use balance_controller::{
    balance_error, balance_error_within_tolerance, clamp_torques, compute_balance_torques,
    BalanceController, BalanceGains,
};

#[path = "push_recovery.rs"]
pub mod push_recovery;
pub use push_recovery::{
    classify_push, compute_recovery, emergency_step_required, recommended_step_reach,
    PushRecoveryConfig, PushSeverity, RecoveryResponse,
};

#[path = "fall_detection.rs"]
pub mod fall_detection;
pub use fall_detection::{
    classify_fall_state, fall_detected, fall_state_label, recovery_possible, update_fall_detector,
    FallDetector, FallDetectorConfig, FallState,
};

#[path = "self_righting.rs"]
pub mod self_righting;
pub use self_righting::{
    advance_planner, is_righted, righting_torque, RightingPhase, SelfRightingConfig,
    SelfRightingPlanner,
};

#[path = "locomotion_fsm.rs"]
pub mod locomotion_fsm;
pub use locomotion_fsm::{
    is_moving, mode_label, mode_step_frequency, update_locomotion_fsm, LocoMode, LocomotionFsm,
};

#[path = "terrain_estimator.rs"]
pub mod terrain_estimator;
pub use terrain_estimator::{
    flat_terrain, terrain_gradient, terrain_normal, terrain_too_steep, update_terrain_from_contact,
    TerrainEstimate, TerrainEstimator,
};

#[path = "contact_estimator.rs"]
pub mod contact_estimator;
pub use contact_estimator::{
    any_hard_contact, contact_count as foot_contact_count, estimate_contact_state, total_grf,
    update_foot_contact, ContactEstimatorConfig, ContactState, FootContact,
};

#[path = "wrench_estimator.rs"]
pub mod wrench_estimator;
pub use wrench_estimator::{
    add_wrenches, external_wrench_detected, force_direction, update_wrench_estimate, Wrench,
    WrenchEstimator, WrenchEstimatorConfig,
};

#[path = "sensor_imu.rs"]
pub mod sensor_imu;
pub use sensor_imu::{
    accel_to_roll_pitch, integrate_gyro, is_static as imu_is_static,
    vec3_magnitude as imu_vec3_magnitude, ImuConfig, ImuSample, ImuSensor,
};

#[path = "sensor_force_plate.rs"]
pub mod sensor_force_plate;
pub use sensor_force_plate::{
    centre_of_pressure as force_plate_cop, force_overrange as force_plate_overrange, is_contact,
    mean_fz, resultant_force, ForcePlateConfig, ForcePlateSample, ForcePlateSensor,
};

#[path = "sensor_emg.rs"]
pub mod sensor_emg;
pub use sensor_emg::{
    dominant_channel, moving_average_envelope, normalise as emg_normalise, rms, EmgConfig,
    EmgFrame, EmgSensor,
};
