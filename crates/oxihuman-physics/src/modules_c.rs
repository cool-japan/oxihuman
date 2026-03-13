// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Auto-generated module re-exports — do not edit directly.

#[path = "elastic_mesh.rs"]
pub mod elastic_mesh;
pub use elastic_mesh::{em_step, new_elastic_mesh, ElasticMesh, ElasticParticle, ElasticSpring};

#[path = "fiber_body.rs"]
pub mod fiber_body;
pub use fiber_body::{fb_particle_count, fb_step, new_fiber_body, FiberBody, FiberParticle};

#[path = "fluid_body.rs"]
pub mod fluid_body;
pub use fluid_body::{
    fb2_add, fb2_step, new_fluid_body, poly6_kernel, spiky_kernel_grad, FluidBody, FluidParticle,
};

#[path = "foam_body.rs"]
pub mod foam_body;
pub use foam_body::{
    fob_add, fob_alive, fob_step, new_foam_body, FoamBody, FoamBubble, FoamConfig,
};

#[path = "foam_spring.rs"]
pub mod foam_spring;
pub use foam_spring::{
    fsn_add_node, fsn_add_spring, new_foam_spring, new_foam_spring_network, spring_period,
    FoamSpring, FoamSpringNetwork,
};

#[path = "gel_body.rs"]
pub mod gel_body;
pub use gel_body::{
    gb_add_particle, gb_add_spring, gb_step, new_gel_body, GelBody, GelMaterial, GelParticle,
    GelSpring,
};

#[path = "granular_body.rs"]
pub mod granular_body;
pub use granular_body::{
    grb_add, grb_step, new_granular_body, Grain as SimGrain, GranularBody as SimGranularBody,
};

#[path = "hydrofoil_body.rs"]
pub mod hydrofoil_body;
pub use hydrofoil_body::{
    hf_drag, hf_lift, hf_step, new_hydrofoil_body, HydrofoilBody, HydrofoilConfig,
};

#[path = "ice_body.rs"]
pub mod ice_body;
pub use ice_body::{ib_heat, ib_step, new_ice_body, IceBody, IceState};

#[path = "jet_body.rs"]
pub mod jet_body;
pub use jet_body::{
    jb_set_throttle, jb_step, jb_thrust, new_jet_body, tsiolkovsky_delta_v, JetBody, ThrusterConfig,
};

#[path = "laser_body.rs"]
pub mod laser_body;
pub use laser_body::{
    lb_beam_radius, lb_irradiance, lb_power_at, lb_reset_energy, lb_set_active, lb_set_power,
    lb_step, new_laser_body, LaserBody, LaserConfig,
};

#[path = "liquid_body.rs"]
pub mod liquid_body;
pub use liquid_body::{
    lb_avg_height, lb_cell_count, lb_get_height as liq_get_height, lb_set_height as liq_set_height,
    lb_step as liq_step, lb_total_volume, new_liquid_body, LiquidBody, LiquidCell,
};

#[path = "magnet_body.rs"]
pub mod magnet_body;
pub use magnet_body::{
    mb_distance, mb_field_at, mb_force_from_field, mb_moment_mag, mb_potential_energy,
    mb_step as mag_step, new_magnet_body, MagnetBody,
};

#[path = "membrane_body.rs"]
pub mod membrane_body;
pub use membrane_body::{
    mb2_avg_y, mb2_particle_count, mb2_pin, mb2_spring_count, mb2_step, new_membrane_body,
    MembraneBody, MembraneParticle, MembraneSpring,
};

#[path = "mesh_body.rs"]
pub mod mesh_body;
pub use mesh_body::{
    mbody_aabb_volume, mbody_recompute_normals, mbody_step, mbody_translate, mbody_triangle_count,
    mbody_vertex_count, new_mesh_body, MeshAabb, MeshBody, MeshTriangle, MeshVertex,
};

#[path = "muscle_body.rs"]
pub mod muscle_body;
pub use muscle_body::{
    mb_set_activation, mb_step_activation, mb_step_fiber, muscle_active_force, muscle_fl,
    muscle_fv, muscle_passive_force, new_muscle_body, MuscleBody,
};

#[path = "net_body.rs"]
pub mod net_body;
pub use net_body::{
    net_add_link, net_avg_y, net_link_count, net_node_count, net_pin, net_step, new_net_body,
    NetBody, NetLink, NetNode,
};

#[path = "particle_filter.rs"]
pub mod particle_filter;
pub use particle_filter::{
    new_particle_filter, pf_count, pf_ess, pf_mean_state0, pf_normalize, pf_propagate, pf_resample,
    pf_update, FilterParticle, ParticleFilter,
};

#[path = "particle_system_v2.rs"]
pub mod particle_system_v2;
pub use particle_system_v2::{
    new_particle_system_v2, ps2_alive_count, ps2_avg_y, ps2_capacity, ps2_kill_all, ps2_set_origin,
    ps2_set_spawn_rate, ps2_step, Particle2, ParticleSystemV2,
};

#[path = "pendulum_chain_v2.rs"]
pub mod pendulum_chain_v2;
pub use pendulum_chain_v2::{
    new_pendulum_chain_v2, pc2_kinetic_energy, pc2_len, pc2_step, pc2_tip_pos, pc2_total_length,
    ChainLink, PendulumChainV2,
};

#[path = "plasma_body.rs"]
pub mod plasma_body;
pub use plasma_body::{
    coulomb_force, new_plasma_body, plasma_add_particle, plasma_center_of_mass, plasma_count,
    plasma_kinetic_energy, plasma_net_charge, plasma_step, PlasmaBody, PlasmaParticle,
};

#[path = "pneumatic_body.rs"]
pub mod pneumatic_body;
pub use pneumatic_body::{
    new_pneumatic_body, pnb_add_gas, pnb_compress, pnb_expand, pnb_force_on_surface, pnb_heat,
    pnb_is_burst, pnb_update_pressure, pnb_vent, PneumaticBody,
};

#[path = "powder_body.rs"]
pub mod powder_body;
pub use powder_body::{
    new_powder_body, pwd_add_grain, pwd_avg_y, pwd_grain_count, pwd_max_y, pwd_settled_count,
    pwd_step, PowderBody, PowderGrain,
};

#[path = "rack_body.rs"]
pub mod rack_body;
pub use rack_body::{
    new_rack_body, rack_angle_from_pos, rack_apply_force, rack_apply_torque, rack_gear_ratio,
    rack_kinetic_energy, rack_pos_from_angle, rack_reset, RackBody,
};

#[path = "rigid_compound.rs"]
pub mod rigid_compound;
pub use rigid_compound::{
    new_rigid_compound, rc_add_box, rc_add_sphere, rc_center_of_mass, rc_moment_of_inertia,
    rc_shape_count, rc_step, rc_total_mass, RigidCompound, SubShape,
};

#[path = "contact_cache.rs"]
pub mod contact_cache;
pub use contact_cache::{
    cc_clear, cc_count, cc_evict_old, cc_find, cc_has_contact, cc_insert, cc_warm_lambda,
    default_contact_cache_config, new_contact_cache, CachedContact, ContactCache,
    ContactCacheConfig,
};

#[path = "joint_motor_v2.rs"]
pub mod joint_motor_v2;
pub use joint_motor_v2::{
    jm_at_limit, jm_compute_force, jm_position_error, jm_reset, jm_set_off, jm_set_position_target,
    jm_set_velocity_target, jm_step, new_joint_motor_v2, JointMotorV2, MotorMode,
};

#[path = "collision_plane.rs"]
pub mod collision_plane;
pub use collision_plane::{
    ground_plane, new_infinite_plane, plane_point_above, plane_project_point,
    plane_signed_dist as infinite_plane_signed_dist, resolve_sphere_plane, sphere_plane_contact,
    InfinitePlane,
};

#[path = "rigid_stack.rs"]
pub mod rigid_stack;
pub use rigid_stack::{
    new_rigid_stack, rs_center_of_mass, rs_count as rigid_stack_count, rs_is_stable, rs_pop_box,
    rs_push_box, rs_sleep_all, rs_total_height, rs_total_mass as rigid_stack_total_mass,
    stack_top_y, RigidStack, StackBox,
};

#[path = "sand_body.rs"]
pub mod sand_body;
pub use sand_body::{
    default_sand_config, new_sand_body, sand_avalanche_step, sand_critical_delta, sand_deposit,
    sand_get, sand_is_stable, sand_max_slope, sand_set, sand_total_volume, SandBody, SandConfig,
};

#[path = "balloon_body.rs"]
pub mod balloon_body;
pub use balloon_body::{
    balloon_deflate, balloon_elastic_force, balloon_inflate, balloon_internal_pressure,
    balloon_is_burst, balloon_move, balloon_net_pressure, balloon_radial_force, balloon_step,
    balloon_surface_area, balloon_volume, new_balloon_body, BalloonBody,
};

#[path = "capsule_body.rs"]
pub mod capsule_body;
pub use capsule_body::{
    capsule_closest_point, capsule_inertia_longitudinal, capsule_inertia_transverse, capsule_step,
    capsule_total_length, capsule_volume as capsule_body_volume, new_capsule_body, CapsuleBody,
};

#[path = "cylinder_body.rs"]
pub mod cylinder_body;
pub use cylinder_body::{
    cylinder_apply_rolling_constraint, cylinder_apply_torque, cylinder_bottom_center,
    cylinder_density, cylinder_inertia_axial, cylinder_inertia_transverse, cylinder_step,
    cylinder_top_center, cylinder_volume, new_cylinder_body, CylinderBody,
};

#[path = "torus_body.rs"]
pub mod torus_body;
pub use torus_body::{
    new_torus_body, torus_apply_spin, torus_contains_point_2d, torus_density, torus_inertia_axial,
    torus_inertia_transverse, torus_inner_radius, torus_outer_radius, torus_step,
    torus_surface_area, torus_volume, TorusBody,
};

#[path = "lattice_body.rs"]
pub mod lattice_body;
pub use lattice_body::{
    lattice_kinetic_energy, lattice_node_count, lattice_pin, lattice_spring_count,
    lattice_step as lattice_body_step, lattice_unpin, new_lattice_body, LatticeBody, LatticeNode,
    LatticeSpring as LatticeBodySpring,
};

#[path = "fiber_body_v2.rs"]
pub mod fiber_body_v2;
pub use fiber_body_v2::{
    fiber_bending_angle, fiber_kinetic_energy, fiber_rest_length, fiber_segment_count,
    fiber_segment_length, fiber_step as fiber_body_step, fiber_stretch_force, fiber_tip,
    new_fiber_body_v2, FiberBodyV2, FiberSegment,
};

#[path = "water_wheel.rs"]
pub mod water_wheel;
pub use water_wheel::{
    new_water_wheel, water_wheel_brake, water_wheel_energy, water_wheel_power, water_wheel_reset,
    water_wheel_rpm, water_wheel_step, water_wheel_torque, WaterWheel,
};

#[path = "windmill_body.rs"]
pub mod windmill_body;
pub use windmill_body::{
    new_windmill, windmill_apply_load, windmill_energy, windmill_power, windmill_reset,
    windmill_rpm, windmill_step, windmill_torque, windmill_tsr, WindmillBody,
};

#[path = "flywheel.rs"]
pub mod flywheel;
pub use flywheel::{
    flywheel_angular_momentum, flywheel_at_max, flywheel_brake, flywheel_energy, flywheel_power,
    flywheel_reset, flywheel_rpm, flywheel_step, new_flywheel, Flywheel,
};

#[path = "double_spring.rs"]
pub mod double_spring;
pub use double_spring::{
    double_spring_kinetic_energy, double_spring_omega1, double_spring_omega2,
    double_spring_potential_energy, double_spring_reset, double_spring_set_state,
    double_spring_step, double_spring_total_energy, new_double_spring, DoubleSpring,
};

#[path = "damper_body.rs"]
pub mod damper_body;
pub use damper_body::{
    damper_at_limit, damper_compression_ratio, damper_force, damper_power, damper_reset,
    damper_set_c, damper_step, damper_total_impulse, new_damper_body, DamperBody,
};

#[path = "actuator_body.rs"]
pub mod actuator_body;
pub use actuator_body::{
    actuator_force, actuator_is_extended, actuator_is_retracted, actuator_power, actuator_reset,
    actuator_set_position, actuator_set_pressure, actuator_step, actuator_stroke_ratio,
    new_actuator, ActuatorBody,
};

#[path = "brake_body.rs"]
pub mod brake_body;
pub use brake_body::{
    brake_cool, brake_is_locked, brake_power, brake_reset, brake_set_clamp, brake_set_engaged,
    brake_step, brake_torque, new_brake, BrakeBody,
};

#[path = "cam_follower.rs"]
pub mod cam_follower;
pub use cam_follower::{
    cam_contact_force, cam_follower_lift, cam_follower_step, cam_follower_velocity_from_profile,
    cam_max_lift, cam_radius_at, cam_reset, cam_rpm, cam_set_omega, new_cam_follower, CamFollower,
};

#[path = "worm_gear.rs"]
pub mod worm_gear;
pub use worm_gear::{
    new_worm_gear, worm_backdrive_efficiency, worm_gear_ratio, worm_input_power,
    worm_is_self_locking, worm_output_power, worm_power_loss, worm_reduction_ratio, worm_reset,
    worm_set_input, WormGear,
};

#[path = "bevel_gear.rs"]
pub mod bevel_gear;
pub use bevel_gear::{
    bevel_gear_ratio, bevel_input_power, bevel_is_miter, bevel_output_power,
    bevel_pitch_cone_angle, bevel_power_loss, bevel_reset, bevel_set_input, bevel_shaft_angle_rad,
    new_bevel_gear, BevelGear,
};

#[path = "chain_drive.rs"]
pub mod chain_drive;
pub use chain_drive::{
    chain_center_distance, chain_driven_radius, chain_driver_radius, chain_gear_ratio,
    chain_has_slack, chain_length, chain_reset, chain_set_input, chain_slack_tension,
    chain_tight_tension, new_chain_drive, ChainDrive,
};

#[path = "belt_drive.rs"]
pub mod belt_drive;
pub use belt_drive::{
    belt_center_distance, belt_gear_ratio, belt_max_force_ratio, belt_max_power, belt_power_loss,
    belt_reset, belt_set_input, belt_set_slip, belt_slack_tension, belt_tight_tension,
    new_belt_drive, BeltDrive,
};

#[path = "ratchet_body.rs"]
pub mod ratchet_body;
pub use ratchet_body::{
    new_ratchet, ratchet_energy, ratchet_full_rotations, ratchet_is_blocked, ratchet_reset,
    ratchet_rpm, ratchet_step, ratchet_tooth_angle, RatchetBody, RatchetDir,
};

#[path = "clutch_body.rs"]
pub mod clutch_body;
pub use clutch_body::{
    clutch_cool, clutch_is_locked, clutch_is_slipping, clutch_max_torque, clutch_power_transfer,
    clutch_reset, clutch_set_engagement, clutch_slip_speed, clutch_step, new_clutch, ClutchBody,
    ClutchState,
};

#[path = "spring_pendulum.rs"]
pub mod spring_pendulum;
pub use spring_pendulum::SpringPendulum;

#[path = "double_pendulum.rs"]
pub mod double_pendulum;
pub use double_pendulum::DoublePendulum;

#[path = "van_der_pol.rs"]
pub mod van_der_pol;
pub use van_der_pol::{vdp_trajectory, VanDerPol};

#[path = "duffing_body.rs"]
pub mod duffing_body;
pub use duffing_body::{duffing_trajectory, DuffingBody};

#[path = "lorenz_attractor.rs"]
pub mod lorenz_attractor;
pub use lorenz_attractor::{lorenz_is_chaotic, lorenz_trajectory, LorenzAttractor};

#[path = "runge_kutta.rs"]
pub mod runge_kutta;
pub use runge_kutta::{euler_scalar, integrate_scalar, rk4_scalar, rk4_vec2, rk4_vec3, rk4_vecn};

#[path = "symplectic_euler.rs"]
pub mod symplectic_euler;
pub use symplectic_euler::{symp_oscillator_trajectory, SympOscillator1D, SympParticle};

#[path = "leapfrog_integrator.rs"]
pub mod leapfrog_integrator;
pub use leapfrog_integrator::{leapfrog_trajectory, LeapfrogOscillator, LeapfrogParticle};

#[path = "nbody_gravity.rs"]
pub mod nbody_gravity;
pub use nbody_gravity::{GravBody, NBodyGravity, G as GRAV_CONST};

#[path = "sph_fluid.rs"]
pub mod sph_fluid;
pub use sph_fluid::{
    poly6_2d, spiky_grad_2d, viscosity_lap_2d, SphConfig as SphFluidSimConfig, SphFluidV2,
    SphParticleV2,
};

#[path = "lattice_boltzmann.rs"]
pub mod lattice_boltzmann;
pub use lattice_boltzmann::LatticeBoltzmann;

#[path = "shallow_water.rs"]
pub mod shallow_water;
pub use shallow_water::ShallowWater;

#[path = "coupled_oscillator.rs"]
pub mod coupled_oscillator;
pub use coupled_oscillator::CoupledOscillators;

#[path = "particle_grid.rs"]
pub mod particle_grid;
pub use particle_grid::{
    default_particle_grid_config, new_particle_grid, pg_cell_for_pos, pg_clear, pg_insert,
    pg_neighbors, pg_stats, pg_total_entries, GridStats as ParticleGridStats, ParticleGrid,
    ParticleGridConfig,
};

#[path = "contact_manifold_v2.rs"]
pub mod contact_manifold_v2;
pub use contact_manifold_v2::{ContactManifoldV2, ContactPoint as ContactPointV2, ManifoldCache};

#[path = "position_based_v2.rs"]
pub mod position_based_v2;
pub use position_based_v2::{
    pbd_v2_integrate, pbd_v2_kinetic_energy, pbd_v2_update_vel, project_dist_v2, project_volume_v2,
    BendConstraintV2, DistConstraintV2, PbdV2Particle, VolumeConstraintV2,
};

#[path = "xpbd_v2.rs"]
pub mod xpbd_v2;
pub use xpbd_v2::{
    xpbd_v2_dist_count, xpbd_v2_kinetic_energy, xpbd_v2_predict, xpbd_v2_project_dist,
    xpbd_v2_reset_lambdas, xpbd_v2_update_vel, XpbdDihedralV2, XpbdDistV2, XpbdV2Particle,
};

#[path = "projective_dynamics.rs"]
pub mod projective_dynamics;
pub use projective_dynamics::{
    pd_constraint_count, pd_global_step, pd_kinetic_energy, pd_local_step, pd_predict,
    pd_update_vel, PdParticle, PdSpringConstraint,
};

#[path = "fem_linear.rs"]
pub mod fem_linear;
pub use fem_linear::{
    tet_shape_gradients, tet_signed_volume as fem_tet_signed_volume, tet_stiffness_scalar,
    FemMaterial, FemMesh, FemNode, FemTet,
};

#[path = "fem_corotational.rs"]
pub mod fem_corotational;
pub use fem_corotational::{
    compute_deformation_gradient, deformation_measure, green_strain, polar_decompose,
    strain_energy_density, Mat3x3,
};

#[path = "vbd_solver.rs"]
pub mod vbd_solver;
pub use vbd_solver::{
    vbd_kinetic_energy, vbd_local_solve, vbd_predict, vbd_spring_energy, vbd_step, vbd_update_vel,
    VbdParticle, VbdSpring,
};

#[path = "incremental_potential.rs"]
pub mod incremental_potential;
pub use incremental_potential::{
    ipc_active_pair_count, ipc_barrier_energy, ipc_barrier_gradient, ipc_dist_sq,
    ipc_gradient_step, ipc_kinetic_energy, ipc_total_contact_energy, IpcConfig, IpcParticle,
};

#[path = "material_point.rs"]
pub mod material_point;
pub use material_point::{MpmCell, MpmConfig, MpmGrid, MpmPoint};

#[path = "smooth_particle.rs"]
pub mod smooth_particle;
pub use smooth_particle::{
    sph2_compute_densities, sph2_compute_pressures, sph2_integrate, sph2_kinetic_energy,
    sph2_poly6, sph2_spiky_grad_mag, sph2_viscosity_lap, sph2_xsph_correction, SphV2Particle,
    WcsphConfig,
};
