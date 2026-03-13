// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Auto-generated module re-exports — do not edit directly.

#[path = "diffusion_reaction.rs"]
pub mod diffusion_reaction;
pub use diffusion_reaction::{gs_get, gs_mean_u, gs_set, gs_step, new_gray_scott, GrayScottGrid};

#[path = "wave_equation_1d.rs"]
pub mod wave_equation_1d;
pub use wave_equation_1d::{
    new_wave_1d, wave_1d_energy, wave_1d_get, wave_1d_max, wave_1d_set, wave_1d_step, Wave1d,
};

#[path = "heat_equation_1d.rs"]
pub mod heat_equation_1d;
pub use heat_equation_1d::{
    heat_1d_get, heat_1d_max, heat_1d_mean, heat_1d_set, heat_1d_step, new_heat_1d, Heat1d,
};

#[path = "lattice_boltzmann_d2q9.rs"]
pub mod lattice_boltzmann_d2q9;
pub use lattice_boltzmann_d2q9::{
    lbm_density, lbm_equilibrium, lbm_step, lbm_velocity_x, new_lbm_d2q9, LbmD2q9,
};

pub use crate::sph_fluid::{
    new_sph_particle, sph_compute_density as sph_compute_density_v2, sph_kernel_dw, sph_kernel_w,
    sph_pressure_tait, SphParticle as SphCubicParticle,
};

#[path = "discrete_element_method.rs"]
pub mod discrete_element_method;
pub use discrete_element_method::{
    dem_contact_force, dem_kinetic_energy, dem_overlap, dem_step, new_dem_particle,
    DemParticle as DemParticle2d,
};

#[path = "crowd_simulation.rs"]
pub mod crowd_simulation;
pub use crowd_simulation::{
    agent_driving_force, agent_has_reached_goal, agent_repulsion, agent_step, new_crowd_agent,
    CrowdAgent,
};

#[path = "swarm_behavior.rs"]
pub mod swarm_behavior;
pub use swarm_behavior::{
    boid_alignment, boid_cohesion, boid_separation, boid_speed, boid_step, new_boid as new_boid_2d,
    Boid as Boid2d,
};

#[path = "sir_epidemic_model.rs"]
pub mod sir_epidemic_model;
pub use sir_epidemic_model::{
    new_sir_model, sir_herd_immunity_threshold, sir_is_epidemic, sir_r0, sir_step, sir_total,
    SirModel,
};

#[path = "lotka_volterra.rs"]
pub mod lotka_volterra;
pub use lotka_volterra::{
    lv_is_stable, lv_predator_loss, lv_prey_growth, lv_step, lv_total, new_lotka_volterra,
    LotkaVolterra,
};

#[path = "reaction_kinetics.rs"]
pub mod reaction_kinetics;
pub use reaction_kinetics::{
    new_reaction, reaction_activation_energy, reaction_half_life, reaction_rate,
    reaction_rate_constant, reaction_set_temperature, Reaction,
};

#[path = "enzyme_kinetics.rs"]
pub mod enzyme_kinetics;
pub use enzyme_kinetics::{
    enzyme_competitive_inhibition, enzyme_half_saturation, enzyme_is_saturated,
    enzyme_turnover_number, enzyme_velocity, new_enzyme_kinetics, EnzymeKinetics,
};

#[path = "vortex_model.rs"]
pub mod vortex_model;
pub use vortex_model::{
    new_vortex_ring, vortex_energy, vortex_impulse, vortex_is_stable, vortex_self_velocity,
    vortex_step, VortexRing,
};

#[path = "population_dynamics.rs"]
pub mod population_dynamics;
pub use population_dynamics::{
    new_population, population_carrying_fraction, population_doubling_time, population_equilibrium,
    population_is_growing, population_step, Population,
};

pub use crate::material_point_method::{
    mpm_particle_kinetic_energy, mpm_particle_momentum, mpm_particle_step, mpm_von_mises_stress,
    new_mpm_particle, MpmParticle,
};

#[path = "cellular_automata_phys.rs"]
pub mod cellular_automata_phys;
pub use cellular_automata_phys::{
    ca_count_material, ca_get, ca_is_settled, ca_set, ca_step, new_ca_sand_grid, CaSandGrid,
    CELL_EMPTY, CELL_SAND, CELL_WALL, CELL_WATER,
};

// ── Wave 151A: Physics Engineering Modules ──────────────────────────────────

#[path = "contact_stiffness.rs"]
pub mod contact_stiffness;
pub use contact_stiffness::{
    contact_energy_loss, contact_friction_force, contact_impulse, contact_is_stiff,
    contact_normal_force, new_contact_stiffness, ContactStiffnessModel,
};

#[path = "friction_coefficient.rs"]
pub mod friction_coefficient;
pub use friction_coefficient::{
    friction_coefficient_for, friction_force as fc_friction_force, is_sliding, new_friction_model,
    FrictionModel, FrictionType,
};

#[path = "thermal_convection.rs"]
pub mod thermal_convection;
pub use thermal_convection::{
    convection_equilibrium_temp, convective_heat_rate, lumped_time_constant,
    new_thermal_convection, nusselt_approx, thermal_resistance, ThermalConvectionModel,
};

#[path = "radiation_heat.rs"]
pub mod radiation_heat;
pub use radiation_heat::{
    blackbody_emission, effective_temperature as radiation_effective_temp, net_radiation,
    new_radiation_heat, radiated_power, radiative_flux, RadiationHeatModel, STEFAN_BOLTZMANN,
};

#[path = "acoustic_wave.rs"]
pub mod acoustic_wave;
pub use acoustic_wave::{
    acoustic_impedance, decibel_spl, new_acoustic_wave, sound_intensity, wave_period,
    wave_pressure, wavelength, AcousticWaveModel,
};

#[path = "vibration_analysis.rs"]
pub mod vibration_analysis;
pub use vibration_analysis::{
    damped_natural_frequency, damping_ratio as vibration_damping_ratio, is_overdamped,
    logarithmic_decrement, natural_frequency as vibration_natural_frequency, new_vibration_model,
    resonance_amplitude, static_deflection, VibrationModel,
};

#[path = "buckling_analysis.rs"]
pub mod buckling_analysis;
pub use buckling_analysis::{
    buckling_safety_factor, critical_stress, effective_length, euler_critical_load, is_slender,
    new_buckling_model, slenderness_ratio, BucklingModel,
};

#[path = "fatigue_life.rs"]
pub mod fatigue_life;
pub use fatigue_life::{
    cycles_to_failure, effective_endurance_limit, fatigue_strength_at_cycles, goodman_ratio,
    is_safe as fatigue_is_safe, miners_damage, new_fatigue_model, FatigueModel,
};

pub use crate::fracture_mechanics::{
    critical_crack_length, new_fracture_model, stress_intensity, will_fracture, FractureModel,
};

pub use crate::creep_model::{
    creep_is_significant, new_creep_model, steady_state_creep_rate, strain_at_time,
    SimpleCreepModel as CreepModel,
};

pub use crate::composite_material::{
    composite_strength, is_fiber_dominated, longitudinal_modulus, new_composite,
    transverse_modulus, SimpleComposite,
};

#[path = "fracture_mechanics_props.rs"]
pub mod fracture_mechanics_props;
pub use fracture_mechanics_props::{
    critical_crack_length as fracture_props_critical_crack, fracture_energy, is_fracture_critical,
    j_integral, new_fracture_props, paris_law, stress_intensity_mode_i, FractureProps,
};

#[path = "creep_deform.rs"]
pub mod creep_deform;
pub use creep_deform::{
    creep_compliance, creep_remaining_life, creep_strain_increment, creep_temperature_factor,
    creep_total_strain, larson_miller_parameter, monkman_grant_rupture_time,
    norton_creep_rate as creep_deform_norton_rate,
};

#[path = "fiber_composite.rs"]
pub mod fiber_composite;
pub use fiber_composite::{
    composite_density as fiber_composite_density, composite_e1, composite_e2, composite_g12,
    composite_longitudinal_strength, composite_nu12, new_fiber_composite, vf_from_weight_fraction,
    FiberComposite,
};

#[path = "biomechanical_loading.rs"]
pub mod biomechanical_loading;
pub use biomechanical_loading::{
    daily_load_cycles, is_high_load, joint_cartilage_stress, joint_reaction_force, load_index,
    new_biomechanical_load, peak_load_estimate, BiomechanicalLoad,
};

#[path = "ergonomics_model.rs"]
pub mod ergonomics_model;
pub use ergonomics_model::{
    duty_cycle, force_demand_ratio, is_high_risk, musculoskeletal_risk, new_ergonomics_model,
    reach_strain, rula_score, ErgonomicsModel,
};

#[path = "motion_capture_model.rs"]
pub mod motion_capture_model;
pub use motion_capture_model::{
    mocap_add_marker, mocap_centroid, mocap_find_marker, mocap_frame_duration, mocap_marker_count,
    mocap_marker_velocity, new_mocap_frame, MarkerPos, MotionCaptureFrame,
};

#[path = "inverse_dynamics.rs"]
pub mod inverse_dynamics;
pub use inverse_dynamics::{
    compute_torque as compute_joint_torque, joint_torques_slice, max_joint_torque,
    new_inverse_dynamics, set_joint_angle, set_link_mass, total_joint_work, InverseDynamicsModel,
};

#[path = "forward_kinematics.rs"]
pub mod forward_kinematics;
pub use forward_kinematics::{
    fk_chain_length, fk_end_effector_2d, fk_end_effector_dist, fk_joint_count,
    fk_joint_position_2d, fk_set_angle, fk_workspace_radius, new_fk_chain, ForwardKinematicsChain,
};

#[path = "deformable_body.rs"]
pub mod deformable_body;
pub use deformable_body::{
    build_chain_body, new_deformable_body, DeformableBody, DeformableParticle, DeformableSpringEdge,
};

#[path = "granular_flow.rs"]
pub mod granular_flow;
pub use granular_flow::{
    default_dem_config, new_granular_flow, DemConfig, GranularFlow, GranularParticle,
};

#[path = "surface_tension.rs"]
pub mod surface_tension;
pub use surface_tension::{
    bond_number, capillary_length, laplace_pressure, new_surface_tension_model, weber_number,
    SurfaceParticle, SurfaceTensionConfig, SurfaceTensionModel,
};

#[path = "magnetic_particle.rs"]
pub mod magnetic_particle;
pub use magnetic_particle::{
    dipole_field, dipole_force, new_magnetic_dipole, new_magnetic_system, MagneticDipole,
    MagneticParticleSystem,
};

#[path = "electrostatic_force.rs"]
pub mod electrostatic_force;
pub use electrostatic_force::{
    coulomb_force as electrostatic_coulomb_force, coulomb_potential, electric_field,
    new_charged_particle, new_electrostatic_system, ChargedParticle, ElectrostaticSystem,
};

#[path = "lubrication_force.rs"]
pub mod lubrication_force;
pub use lubrication_force::{
    couette_shear_stress, ehd_min_film_thickness, new_lubricant_fluid, new_lubrication_film,
    slider_bearing_load, sommerfeld_number, squeeze_film_force, LubricantFluid, LubricationFilm,
};

#[path = "acoustic_impedance.rs"]
pub mod acoustic_impedance;
pub use acoustic_impedance::{
    acoustic_impedance as acoustic_impedance_value, air_medium, analyze_interface,
    intensity_reflection_coeff, intensity_transmission_coeff, new_acoustic_medium,
    pressure_reflection_coeff, pressure_transmission_coeff, standing_wave_ratio, steel_medium,
    transmission_loss_db as impedance_transmission_loss_db, water_medium, AcousticMedium,
    InterfaceResult,
};

#[path = "thermal_expansion.rs"]
pub mod thermal_expansion;
pub use thermal_expansion::{
    aluminum_thermal_material, hydrostatic_thermal_stress as te_hydrostatic_thermal_stress,
    new_thermal_bar, new_thermal_material, steel_thermal_material, ThermalBar, ThermalMaterial,
};

#[path = "foam_model.rs"]
pub mod foam_model;
pub use foam_model::{
    new_foam_compression, new_foam_params as new_gibson_foam_params, polystyrene_foam,
    polyurethane_foam, FoamCellType, FoamCompression,
};

#[path = "gel_model.rs"]
pub mod gel_model;
pub use gel_model::{
    chi_from_solubility, degree_of_swelling, hydrogel_params, new_gel_params, new_gel_state,
    pnipam_gel_params, shear_modulus_from_crosslink_density, GelParams, GelState,
};

#[path = "heat_equation.rs"]
pub mod heat_equation;
pub use heat_equation::{new_heat_pulse, new_heat_sine, steady_state_linear, HeatEquation1D};

#[path = "diffusion_model.rs"]
pub mod diffusion_model;
pub use diffusion_model::{
    ficks_first_law, gaussian_solution, new_diffusion_1d, new_diffusion_2d, Diffusion1D,
    Diffusion2D,
};

#[path = "advection_model.rs"]
pub mod advection_model;
pub use advection_model::{cfl_number, lax_wendroff_step, new_advection_1d, Advection1D};

#[path = "poisson_solver.rs"]
pub mod poisson_solver;
pub use poisson_solver::{new_poisson_solver, poisson_1d, PoissonSolver2D};

#[path = "navier_stokes_2d.rs"]
pub mod navier_stokes_2d;
pub use navier_stokes_2d::{new_navier_stokes_2d, NavierStokes2D};

#[path = "smoothed_particle_2d.rs"]
pub mod smoothed_particle_2d;
pub use smoothed_particle_2d::{new_sph_2d, wendland_grad_w, wendland_w, Sph2D, SphParticle2D};

#[path = "discrete_fracture.rs"]
pub mod discrete_fracture;
pub use discrete_fracture::{cubic_law_flow, new_dfn, DiscreteFractureNetwork, Fracture};

#[path = "porous_flow.rs"]
pub mod porous_flow;
pub use porous_flow::{
    darcy_velocity, forchheimer_velocity, new_darcy_1d, new_darcy_2d, DarcyFlow1D, DarcyFlow2D,
};

#[path = "electrokinetic.rs"]
pub mod electrokinetic;
pub use electrokinetic::{
    henrys_function_smoluchowski, huckel_mobility, new_electrokinetic_system,
    smoluchowski_mobility, ElectrokineticConfig, ElectrokineticParticle, ElectrokineticSystem,
};

#[path = "brownian_motion.rs"]
pub mod brownian_motion;
pub use brownian_motion::{
    einstein_diffusion, expected_msd_3d, new_langevin, stokes_einstein_diffusion, LangevinParticle,
    LangevinSimulation,
};

#[path = "dna_model.rs"]
pub mod dna_model;
pub use dna_model::{bdna_model, fjc_force, new_dna_model, odijk_deflection_length, WlcDnaModel};

#[path = "membrane_model.rs"]
pub mod membrane_model;
pub use membrane_model::{
    new_helfrich_membrane, new_membrane_patch, HelfrichMembrane, MembranePatch,
};

#[path = "fluid_surface.rs"]
pub mod fluid_surface;
pub use fluid_surface::{
    create_particle_grid, poly6_kernel as fluid_surface_poly6_kernel, poly6_kernel_gradient_scalar,
    spiky_kernel_gradient_scalar, viscosity_kernel_laplacian as fluid_surface_viscosity_lap,
    SphConfig as FluidSurfaceSphConfig, SphParticle as FluidSurfaceSphParticle, SphSimulation,
};

#[path = "thermal_model.rs"]
pub mod thermal_model;
pub use thermal_model::{
    BodyRegion, ThermalBody, ThermalColumn, ThermalLayer, ThermalNode, ThermalSimulation,
};
