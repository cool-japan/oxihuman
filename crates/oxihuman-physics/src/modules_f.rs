// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Auto-generated module re-exports — do not edit directly.

#[path = "shell_kinematics.rs"]
pub mod shell_kinematics;
pub use shell_kinematics::{
    new_shell_kinematics, Curvature, MembraneStrain, ShellKinematics, ShellNode,
};

#[path = "plate_bending.rs"]
pub mod plate_bending;
pub use plate_bending::{new_plate_bending, PlateBending};

#[path = "beam_element.rs"]
pub mod beam_element;
pub use beam_element::{new_beam_element, BeamElem, BeamElement, BeamSection};

#[path = "contact_mechanics.rs"]
pub mod contact_mechanics;
pub use contact_mechanics::{
    contact_area, contact_stiffness, force_from_approach, sphere_on_flat, HertzContact,
};

#[path = "adhesion_model.rs"]
pub mod adhesion_model;
pub use adhesion_model::{
    adhesion_energy, dmt_pulloff_force, is_jkr_regime, jkr_pulloff_force, jkr_zero_force_radius,
    maugis_parameter, pulloff_force, AdhesionConfig, AdhesionModel,
};

#[path = "tribology_model.rs"]
pub mod tribology_model;
pub use tribology_model::{
    archard_wear_volume, friction_energy, friction_force, is_sliding_onset, stribeck_friction,
    FrictionRegime, TribologyParams,
};

#[path = "lubrication_model.rs"]
pub mod lubrication_model;
pub use lubrication_model::{
    barus_viscosity, classify_regime, entrainment_velocity, film_shear_stress,
    grubin_film_thickness, viscous_heat, LubricantConfig, LubricationRegime,
};

#[path = "thermal_expansion_deform.rs"]
pub mod thermal_expansion_deform;
pub use thermal_expansion_deform::{
    biaxial_thermal_stress, constrained_thermal_stress, isotropic_strain_tensor, new_length,
    temp_for_strain, thermal_elongation, thermal_strain, volumetric_strain,
    ThermalExpansionMaterial,
};

#[path = "thermoelastic_stress.rs"]
pub mod thermoelastic_stress;
pub use thermoelastic_stress::{
    hydrostatic_thermal_stress, inelastic_heat_fraction, plane_strain_stress_xx,
    temp_from_volumetric_strain, thermal_diffusivity, thermoelastic_damping, ThermoelasticMaterial,
};

#[path = "piezoelectric_model.rs"]
pub mod piezoelectric_model;
pub use piezoelectric_model::{
    blocking_force, charge_from_stress, coupling_coefficient, displacement_from_voltage,
    polarization_from_stress, resonant_frequency_stub,
    strain_from_field as piezo_strain_from_field, transverse_strain_from_field, PiezoConfig,
};

#[path = "electrostriction_model.rs"]
pub mod electrostriction_model;
pub use electrostriction_model::{
    dc_bias_strain, dielectric_energy, electrostriction_force, electrostrictive_strain,
    maxwell_stress, polarization_from_field as electrostrictive_pol_from_field,
    strain_from_e_field as electrostriction_strain_from_e,
    transverse_strain as electrostrictive_transverse_strain, ElectrostrictiveConfig,
};

#[path = "magnetostrictive_model.rs"]
pub mod magnetostrictive_model;
pub use magnetostrictive_model::{
    blocking_stress as magnetostrictive_blocking_stress,
    magnetization_from_field as magnetostrictive_mag_from_field, magnetostrictive_energy_density,
    magnetostrictive_force, magnetostrictive_strain,
    strain_from_field as magnetostrictive_strain_from_field, villari_delta_perm,
    MagnetostrictiveConfig,
};

#[path = "shape_memory_effect.rs"]
pub mod shape_memory_effect;
pub use shape_memory_effect::{
    austenite_fraction, current_phase, effective_modulus as sme_effective_modulus,
    is_fully_austenite, martensite_fraction, recoverable_strain, recovery_force, SmaConfig,
    SmaPhase as SmePhase,
};

#[path = "electrowetting.rs"]
pub mod electrowetting;
pub use electrowetting::{
    capillary_pressure, contact_angle_ewod, contact_line_force, ewod_number, is_saturated,
    restore_contact_angle, spreading_velocity_stub, EwodConfig,
};

#[path = "dielectrophoresis_model.rs"]
pub mod dielectrophoresis_model;
pub use dielectrophoresis_model::{
    clausius_mossotti_real, crossover_frequency, dep_force, dep_velocity, is_positive_dep,
    DepConfig,
};

#[path = "ferrofluid_model.rs"]
pub mod ferrofluid_model;
pub use ferrofluid_model::{
    effective_viscosity as ferrofluid_effective_viscosity, kelvin_force_density, magnetic_pressure,
    magnetization as ferrofluid_magnetization, rosensweig_threshold, spike_height,
    wetting_pressure, FerrofluidConfig,
};

#[path = "ferroelectric_model.rs"]
pub mod ferroelectric_model;
pub use ferroelectric_model::{
    displacement_field, hysteresis_energy_density, hysteresis_polarization, is_above_coercive,
    polarization_vs_temp, remnant_polarization, small_signal_permittivity, FerroelectricConfig,
};

#[path = "multiferroic_model.rs"]
pub mod multiferroic_model;
pub use multiferroic_model::{
    combined_order_parameter, electric_polarization_from_h, is_ferroelectrically_ordered,
    is_magnetically_ordered, magnetization_from_e, me_current_coefficient, me_figure_of_merit,
    me_voltage_coefficient, stress_modulated_coupling, MultiferroicConfig,
};

#[path = "topological_insulator.rs"]
pub mod topological_insulator;
pub use topological_insulator::{
    anomalous_hall_conductance_stub, chern_number, fermi_wavevector, is_in_bulk_gap,
    mean_free_path, spin_angle, surface_dispersion, surface_dos, z2_invariant, TopoInsulatorConfig,
};

#[path = "capillary_action.rs"]
pub mod capillary_action;
pub use capillary_action::{
    capillary_pressure as jurin_capillary_pressure, capillary_rise_height, capillary_set_angle,
    capillary_volume, new_capillary_tube, CapillaryTube,
};

#[path = "osmotic_pressure.rs"]
pub mod osmotic_pressure;
pub use osmotic_pressure::{
    new_osmotic_solution, osmotic_equilibrium_concentration, osmotic_flow_direction,
    osmotic_is_hypertonic, osmotic_pressure_pa, OsmoticSolution,
};

#[path = "lymph_flow.rs"]
pub mod lymph_flow;
pub use lymph_flow::{
    lymph_flow_rate, lymph_is_absorbing, lymph_net_filtration_pressure, lymph_surface_area,
    new_lymph_capillary, LymphCapillary,
};

#[path = "bone_remodeling.rs"]
pub mod bone_remodeling;
pub use bone_remodeling::{
    bone_equilibrium_density, bone_is_dense, bone_is_osteoporotic, bone_mineral_content, bone_step,
    new_bone_element, BoneElement,
};

#[path = "cartilage_stress.rs"]
pub mod cartilage_stress;
pub use cartilage_stress::{
    cartilage_apply_load, cartilage_fluid_pressure, cartilage_recovery, cartilage_total_stress,
    new_cartilage_layer, CartilageLayer,
};

#[path = "tendon_viscoelastic.rs"]
pub mod tendon_viscoelastic;
pub use tendon_viscoelastic::{
    new_tendon as new_viscoelastic_tendon, tendon_elongate, tendon_energy,
    tendon_force as viscoelastic_tendon_force, tendon_relax,
    tendon_strain as viscoelastic_tendon_strain, Tendon as ViscoelasticTendon,
};

#[path = "ligament_spring.rs"]
pub mod ligament_spring;
pub use ligament_spring::{
    ligament_elongate, ligament_force, ligament_is_taut, ligament_stiffness, ligament_strain,
    new_ligament, Ligament,
};

#[path = "synovial_fluid.rs"]
pub mod synovial_fluid;
pub use synovial_fluid::{
    new_synovial_fluid, synovial_is_shear_thinning, synovial_lubrication_number,
    synovial_shear_stress, synovial_viscosity, SynovialFluid,
};

#[path = "interstitial_fluid.rs"]
pub mod interstitial_fluid;
pub use interstitial_fluid::{
    interstitial_is_edematous, interstitial_net_flow, interstitial_step,
    interstitial_volume_change, new_interstitial_compartment, InterstitialCompartment,
};

#[path = "blood_viscosity.rs"]
pub mod blood_viscosity;
pub use blood_viscosity::{
    blood_apparent_viscosity, blood_is_flowing, blood_viscosity_casson, blood_yield_stress,
    new_blood, Blood,
};

#[path = "cardiac_output.rs"]
pub mod cardiac_output;
pub use cardiac_output::{
    heart_cardiac_output_l_per_min, heart_ejection_fraction, heart_frank_starling_adjust,
    heart_mean_arterial_pressure, new_heart, Heart,
};

#[path = "pulmonary_flow.rs"]
pub mod pulmonary_flow;
pub use pulmonary_flow::{
    new_pulmonary_circuit, pulmonary_flow_rate, pulmonary_resistance, pulmonary_transit_time,
    pulmonary_update_radius, PulmonaryCircuit,
};

#[path = "renal_filtration.rs"]
pub mod renal_filtration;
pub use renal_filtration::{
    gfr_filtration_rate, gfr_is_filtering, gfr_net_filtration_pressure, gfr_update_pressure,
    new_glomerulus, Glomerulus,
};

#[path = "digestive_peristalsis.rs"]
pub mod digestive_peristalsis;
pub use digestive_peristalsis::{
    new_peristaltic_segment, peristalsis_bolus_velocity, peristalsis_is_contracted,
    peristalsis_radius, peristalsis_step, PeristalticSegment,
};

#[path = "sweat_gland_model.rs"]
pub mod sweat_gland_model;
pub use sweat_gland_model::{
    new_sweat_gland, sweat_cooling_power_w, sweat_heat_loss, sweat_is_active, sweat_rate,
    sweat_set_core_temp, SweatGland,
};

#[path = "melanin_distribution.rs"]
pub mod melanin_distribution;
pub use melanin_distribution::{
    melanin_ratio, melanin_set_eumelanin, melanin_set_pheomelanin, melanin_skin_tone_index,
    melanin_total, melanin_uv_protection_factor, new_melanin_layer, MelaninLayer,
};

#[path = "wound_healing_model.rs"]
pub mod wound_healing_model;
pub use wound_healing_model::{
    new_wound, wound_apply_treatment, wound_healing_time_hours, wound_is_healed,
    wound_percent_closed, wound_step, Wound,
};

#[path = "cell_migration_model.rs"]
pub mod cell_migration_model;
pub use cell_migration_model::{
    cell_chemotaxis_step, cell_distance_from_origin, cell_position, cell_step, new_cell,
    Cell as MigratingCell,
};

#[path = "tumor_growth.rs"]
pub mod tumor_growth;
pub use tumor_growth::{
    new_tumor, tumor_apply_treatment, tumor_doubling_time_days, tumor_is_detectable, tumor_step,
    tumor_viable_volume, Tumor,
};

#[path = "muscle_fatigue_model.rs"]
pub mod muscle_fatigue_model;
pub use muscle_fatigue_model::{
    fatigue_can_exert, fatigue_percent, fatigue_recovery_step, fatigue_reset, fatigue_step,
    new_muscle_fatigue, MuscleFatigue,
};

#[path = "vestibular_model.rs"]
pub mod vestibular_model;
pub use vestibular_model::{
    canal_is_stimulated, canal_perceived_rotation, canal_reset, canal_step, new_semicircular_canal,
    SemicircularCanal,
};

#[path = "thermoregulation_core.rs"]
pub mod thermoregulation_core;
pub use thermoregulation_core::{
    new_thermocore, thermo_heat_loss_w, thermo_is_hyperthermia, thermo_is_hypothermia,
    thermo_is_normal, thermo_step, ThermoCore,
};

#[path = "circadian_rhythm.rs"]
pub mod circadian_rhythm;
pub use circadian_rhythm::{
    circadian_alertness, circadian_is_nighttime, circadian_phase_shift, circadian_step,
    circadian_time_to_peak, new_circadian_oscillator, CircadianOscillator,
};

#[path = "balance_control.rs"]
pub mod balance_control;
pub use balance_control::{
    balance_angle_deg, balance_apply_perturbation, balance_control_torque, balance_is_stable,
    balance_step, new_balance_pendulum, BalancePendulum,
};

#[path = "posture_sway_model.rs"]
pub mod posture_sway_model;
pub use posture_sway_model::{
    new_posture_sway, sway_cop_position, sway_mean_displacement, sway_path_length, sway_rms,
    sway_step, PostureSway,
};

#[path = "grip_force_model.rs"]
pub mod grip_force_model;
pub use grip_force_model::{
    grip_is_slipping, grip_overshoot_factor, grip_required_force, grip_set_target, grip_step,
    new_grip_force, GripForce,
};

#[path = "reflex_arc.rs"]
pub mod reflex_arc;
pub use reflex_arc::{
    new_reflex_arc, reflex_apply_fatigue, reflex_is_active, reflex_peak_force,
    reflex_reset_fatigue, reflex_response, ReflexArc,
};

#[path = "proprioception.rs"]
pub mod proprioception;
pub use proprioception::{
    gto_firing, new_muscle_sensor, sensor_is_overloaded, sensor_update, spindle_II_firing,
    spindle_Ia_firing, MuscleSensor,
};

#[path = "pain_threshold_model.rs"]
pub mod pain_threshold_model;
pub use pain_threshold_model::{
    new_nociceptor, noci_adapt, noci_is_active, noci_reset, noci_sensitize, noci_threshold,
    NociceptorState,
};

#[path = "neural_signal_model.rs"]
pub mod neural_signal_model;
pub use neural_signal_model::{
    fhn_is_spiking, fhn_membrane_potential, fhn_recovery, fhn_set_current, fhn_step,
    new_fitzhugh_nagumo, FitzHughNagumo,
};

#[path = "sleep_wake_cycle.rs"]
pub mod sleep_wake_cycle;
pub use sleep_wake_cycle::{
    new_sleep_wake_model, sleep_alertness, sleep_hours_since_wake, sleep_set_asleep,
    sleep_should_sleep, sleep_step, SleepWakeModel,
};

#[path = "fall_risk_model.rs"]
pub mod fall_risk_model;
pub use fall_risk_model::{
    fall_dominant_factor, fall_is_high_risk, fall_risk_category, fall_risk_score, fall_set_balance,
    new_fall_risk_factors, FallRiskFactors,
};

#[path = "joint_contact_model.rs"]
pub mod joint_contact_model;
pub use joint_contact_model::{
    joint_contact_energy, joint_contact_force, joint_contact_pressure, joint_is_in_contact,
    new_joint_contact, JointContact,
};

#[path = "intervertebral_disc.rs"]
pub mod intervertebral_disc;
pub use intervertebral_disc::{
    disc_apply_load, disc_axial_stiffness, disc_height_loss, disc_is_herniated, disc_recovery,
    new_iv_disc, IvDisc,
};

#[path = "spinal_column_model.rs"]
pub mod spinal_column_model;
pub use spinal_column_model::{
    new_spinal_segment, spinal_column_range_of_motion, spinal_segment_is_overloaded,
    spinal_segment_step, spinal_total_stiffness, SpinalSegment,
};

#[path = "rib_cage_model.rs"]
pub mod rib_cage_model;
pub use rib_cage_model::{
    new_rib_cage, rib_elastic_recoil_pressure, rib_expansion, rib_is_expanded, rib_step, RibCage,
};

#[path = "lung_mechanics_v2.rs"]
pub mod lung_mechanics_v2;
pub use lung_mechanics_v2::{
    lung_v2_driving_pressure, lung_v2_flow_step, lung_v2_is_hyperinflated, lung_v2_tidal_volume,
    new_lung_mechanics_v2, LungMechanicsV2,
};

#[path = "diaphragm_model.rs"]
pub mod diaphragm_model;
pub use diaphragm_model::{
    diaphragm_activate, diaphragm_fatigue_step, diaphragm_force, diaphragm_is_paralyzed,
    diaphragm_pressure_contribution, new_diaphragm, Diaphragm,
};

#[path = "heart_valve_model.rs"]
pub mod heart_valve_model;
pub use heart_valve_model::{
    new_heart_valve, valve_flow_rate_ml_per_s, valve_is_stenotic, valve_regurgitation_fraction,
    valve_update, HeartValve,
};

#[path = "coronary_flow.rs"]
pub mod coronary_flow;
pub use coronary_flow::{
    coronary_ffr, coronary_flow_ml_per_min, coronary_is_critical, coronary_resistance,
    new_coronary_artery, CoronaryArtery,
};

#[path = "venous_return.rs"]
pub mod venous_return;
pub use venous_return::{
    new_venous_system, venous_cardiac_output_balance, venous_pressure_mmhg, venous_return_flow,
    venous_shift_volume, VenousSystem,
};

#[path = "lymph_node_model.rs"]
pub mod lymph_node_model;
pub use lymph_node_model::{
    lymph_node_filter, lymph_node_is_activated, lymph_node_output_load, lymph_node_swelling,
    new_lymph_node, LymphNode,
};

#[path = "liver_clearance.rs"]
pub mod liver_clearance;
pub use liver_clearance::{
    liver_bioavailability, liver_clearance_rate, liver_half_life, liver_intrinsic_clearance,
    new_liver_clearance, LiverClearance,
};

#[path = "gastric_acid_model.rs"]
pub mod gastric_acid_model;
pub use gastric_acid_model::{
    gastric_apply_antacid, gastric_is_acidic, gastric_secretion_inhibit, gastric_step,
    new_gastric_acid, GastricAcid,
};

#[path = "intestinal_absorption.rs"]
pub mod intestinal_absorption;
pub use intestinal_absorption::{
    gut_bioavailability, gut_fraction_absorbed, gut_peak_absorption_time, gut_step,
    new_gut_compartment, GutCompartment,
};

#[path = "bladder_model.rs"]
pub mod bladder_model;
pub use bladder_model::{
    bladder_fullness, bladder_pressure_cmh2o, bladder_step, bladder_urge_threshold, bladder_void,
    new_bladder, Bladder,
};

#[path = "spleen_model.rs"]
pub mod spleen_model;
pub use spleen_model::{
    new_spleen_model, spleen_cells_filtered_per_min, spleen_is_splenomegaly,
    spleen_platelet_pool_fraction, spleen_total_volume, SpleenModel,
};

#[path = "renal_tubular_reabsorption.rs"]
pub mod renal_tubular_reabsorption;
pub use renal_tubular_reabsorption::{
    new_tubular_reabsorption, tubular_excretion_rate, tubular_filtered_load,
    tubular_reabsorption_rate, tubular_threshold_concentration, TubularReabsorption,
};
