// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Auto-generated module re-exports — do not edit directly.

#[path = "constraint_group.rs"]
pub mod constraint_group;
#[path = "pbd_solver.rs"]
pub mod pbd_solver;
#[path = "sph_proxy.rs"]
pub mod sph_proxy;
pub use constraint_group::{
    build_constraint_graph, coloring_stats, constraint_particle_degree, greedy_graph_color,
    independent_set_coloring, optimal_substep_order, validate_coloring, ColorGroup, ColoringResult,
    ConstraintGraph,
};
pub use pbd_solver::{
    add_cloth_grid, add_rope, solve_distance_pbd, solve_ground_plane, PbdConfig, PbdConstraint,
    PbdConstraintKind, PbdParticle, PbdSimulation,
};
pub use sph_proxy::{
    compute_density, compute_pressure, kernel_poly6, kernel_spiky_grad, kernel_viscosity_lap,
    SphConfig, SphParticle, SphSystem,
};
#[path = "fluid_height.rs"]
pub mod fluid_height;
#[path = "joint_motor.rs"]
pub mod joint_motor;
#[path = "rigid_body.rs"]
pub mod rigid_body;
pub use fluid_height::{
    flat_fluid_grid, flow_divergence, height_gradient, FluidGrid as HeightFluidGrid,
    FluidGridConfig,
};
pub use joint_motor::{
    apply_motor_force, hinge_angle, joint_violation, standard_biped_joints, Joint, JointKind,
    JointMotor, JointSystem,
};
pub use rigid_body::{
    integrate_orientation, mat3_sym_inertia_box, mat3_sym_inertia_sphere, quat_from_axis_angle,
    quat_multiply, quat_normalize, RigidBody, RigidBodyState,
};

#[path = "fracture.rs"]
pub mod fracture;
pub use fracture::{
    cell_centroid, fracture_mesh, generate_voronoi_seeds, merge_small_cells, voronoi_fracture,
    FractureConfig, VoronoiCell,
};
#[path = "cloth_pattern.rs"]
pub mod cloth_pattern;
pub use cloth_pattern::{
    build_circular_panel, build_rectangle_panel, drape_panel_onto_sphere,
    pattern_to_cloth_particles, wrap_panel_to_cylinder, EdgeKind, GarmentPattern,
    GarmentPatternConfig, PatternEdge, PatternVertex,
};

#[path = "broadphase.rs"]
pub mod broadphase;
pub use broadphase::{
    aabb_expand, aabb_from_capsule, aabb_from_sphere, aabb_overlap, build_bvh,
    compute_all_pair_overlaps, query_aabb, query_ray, BvhAabb, BvhNode, BvhTree,
};

#[path = "cloth_tear.rs"]
pub mod cloth_tear;
pub use cloth_tear::{
    apply_force_at, compute_stretch_ratio, count_broken_constraints, count_intact_constraints,
    find_overloaded_constraints, is_fully_torn, new_tearable_grid, step_tear_simulation,
    tear_at_point, tearable_mesh_stats, TearConstraint, TearableMesh,
};

#[path = "particle_system.rs"]
pub mod particle_system;
pub use particle_system::{
    active_particle_count, count_expired_slots, emit_particle, lerp_particle_color,
    new_particle_system, particle_age_fraction, particle_system_bounds, reset_particle_system,
    set_emitter_position, step_particle_system, EmitterShape, Particle, ParticleEmitter,
    ParticleSystem,
};

#[path = "rope_sim.rs"]
pub mod rope_sim;
pub use rope_sim::{
    apply_impulse_to_rope, attach_rope_end, new_rope, pin_particle, rope_end_position, rope_energy,
    rope_length, rope_sag, rope_tension_at, rope_to_polyline, step_rope, unpin_particle, Rope,
    RopeConfig, RopeParticle, RopeSegment,
};

#[path = "joint_constraint.rs"]
pub mod joint_constraint;
pub use joint_constraint::{
    add_ball_joint, add_fixed_joint, add_hinge_joint, apply_joint_impulse, break_joint,
    compute_chain_positions, constraint_energy as joint_constraint_energy, count_active_joints,
    joint_violation as joint_constraint_violation, new_constraint_solver, solve_constraints,
    ConstraintSolver, JointConstraint, JointType,
};

#[path = "spring_network.rs"]
pub mod spring_network;
pub use spring_network::{
    add_node, add_spring, apply_impulse, build_grid_network, clamp_velocities, count_pinned,
    network_bounding_box, network_energy, new_network, spring_extension, spring_force,
    step_network, Spring as SpringNetNode, SpringNetwork, SpringNetworkConfig, SpringNode,
};

#[path = "buoyancy.rs"]
pub mod buoyancy;
pub use buoyancy::{
    archimedes_force, compute_buoyancy_force, compute_wave_force, drag_force, equilibrium_depth,
    is_floating, multi_body_buoyancy, step_body, submerged_fraction,
    terminal_velocity as buoyancy_terminal_velocity, BuoyancyConfig, BuoyancyResult, SubmergedBody,
};

#[path = "kinematic_body.rs"]
pub mod kinematic_body;
pub use kinematic_body::{
    aabb_of_body, add_kinematic_body, bodies_overlap, enabled_body_count, get_body,
    kinematic_body_count, move_body, new_kinematic_world, remove_body, set_body_rotation,
    set_layer_mask, sphere_sphere_contact, KinematicBody, KinematicContact, KinematicShape,
    KinematicWorld,
};

#[path = "trigger_zone.rs"]
pub mod trigger_zone;
pub use trigger_zone::{
    add_aabb_trigger, add_capsule_trigger, add_sphere_trigger, enabled_trigger_count,
    get_trigger as get_trigger_zone, new_trigger_world, point_in_aabb, point_in_capsule,
    point_in_sphere, point_in_trigger, query_triggers, remove_trigger, trigger_zone_volume,
    TriggerEvent, TriggerShape, TriggerWorld, TriggerZone,
};

#[path = "debris_system.rs"]
pub mod debris_system;
pub use debris_system::{
    apply_wind_to_debris, debris_bounding_box, default_debris_config, fragment_kinetic_energy,
    living_fragment_count, new_debris_system, remove_dead, set_floor, spawn_explosion,
    spawn_fragment, step_debris, total_kinetic_energy as debris_total_ke, DebrisConfig,
    DebrisFragment, DebrisSystem,
};

#[path = "fluid_grid.rs"]
pub mod fluid_grid;
pub use fluid_grid::{
    add_density, add_velocity, advect_density, cell_index, default_fluid_config, diffuse_density,
    fluid_grid_stats, get_cell, get_cell_mut, max_velocity as fluid_max_velocity, new_fluid_grid,
    set_obstacle, step_fluid, total_density, FluidCell, FluidConfig, FluidGrid as EulerFluidGrid,
};

#[path = "impulse_solver.rs"]
pub mod impulse_solver;
pub use impulse_solver::{
    add_impulse_body, apply_impulse_to_body, compute_impulse_magnitude, impulse_body_by_id,
    impulse_body_count, integrate_impulse_bodies, new_impulse_solver, relative_velocity_at_contact,
    remove_impulse_body, resolve_impulse_contact, separate_impulse_bodies,
    sphere_sphere_impulse_contact, total_impulse_kinetic_energy, ImpulseBody, ImpulseContact,
    ImpulseSolver,
};

#[path = "contact_material.rs"]
pub mod contact_material;
pub use contact_material::{
    add_contact_pair, all_physics_materials, combine_friction, combine_restitution,
    contact_pair_count, default_contact_props, default_material_table, lookup_contact,
    material_density, material_friction, material_restitution, new_contact_table,
    physics_material_name, ContactMaterialTable, ContactProps, PhysicsMaterial,
};

#[path = "granular_sim.rs"]
pub mod granular_sim;
pub use granular_sim::{
    active_grain_count, add_grain, default_granular_config, grain_count, grain_pile_height,
    grains_overlap, new_granular_world, pour_grains, remove_grain, resolve_grain_floor,
    resolve_grain_grain, simulate_granular, total_granular_energy, Grain, GranularConfig,
    GranularWorld,
};

#[path = "ragdoll.rs"]
pub mod ragdoll;
pub use ragdoll::{
    activate_ragdoll, add_ragdoll_bone, add_ragdoll_joint, apply_impulse_to_bone,
    deactivate_ragdoll, default_humanoid_ragdoll, get_ragdoll_bone, new_ragdoll,
    ragdoll_bone_count, ragdoll_center_of_mass, ragdoll_joint_count, ragdoll_total_mass,
    simulate_ragdoll, Ragdoll, RagdollBone, RagdollJoint,
};

#[path = "xpbd_solver.rs"]
pub mod xpbd_solver;
pub use xpbd_solver::{
    add_bend_constraint as xpbd_add_bend, add_distance_constraint as xpbd_add_distance,
    add_xpbd_particle, new_xpbd_world, pin_particle as xpbd_pin_particle, predict_positions,
    reset_lambdas, solve_distance_constraint, unpin_particle as xpbd_unpin_particle,
    update_velocities, xpbd_constraint_count, xpbd_particle_count, xpbd_step, xpbd_total_energy,
    XpbdConstraint, XpbdConstraintType, XpbdParticle, XpbdWorld,
};

#[path = "motor_controller.rs"]
pub mod motor_controller;
pub use motor_controller::{
    add_motor_to_bank, default_pid_params, disable_motor, enable_motor, get_motor, motor_count,
    motor_update, new_motor, new_motor_bank, pid_update, proportional_controller, reset_pid,
    set_motor_target, update_all_motors, MotorBank, MotorController, PidParams, PidState,
};

#[path = "character_controller.rs"]
pub mod character_controller;
pub use character_controller::{
    apply_gravity_cc, can_step_up, capsule_aabb, character_foot_position, character_head_position,
    character_speed, disable_controller, enable_controller, ground_check, jump_character,
    land_character, move_character, new_character_controller, push_character, CharacterCapsule,
    CharacterController, CharacterState,
};

#[path = "ballistic.rs"]
pub mod ballistic;
pub use ballistic::{
    default_ballistic_config, drag_force as ballistic_drag_force, impact_point_on_plane,
    launch_velocity_for_target, max_range, new_projectile, projectile_kinetic_energy,
    simulate_projectile, simulate_trajectory, time_of_flight, trajectory_length,
    trajectory_max_height, BallisticConfig, Projectile, TrajectoryPoint,
};

#[path = "soft_constraint.rs"]
pub mod soft_constraint;
pub use soft_constraint::{
    add_soft_constraint, add_soft_particle, disable_soft_constraint, enable_soft_constraint,
    new_soft_world, soft_constraint_count, soft_constraint_violation, soft_particle_count,
    soft_total_energy, solve_soft_distance, solve_soft_plane, solve_soft_point, step_soft_world,
    SoftConstraint, SoftConstraintType, SoftConstraintWorld,
};

#[path = "fluid_particle.rs"]
pub mod fluid_particle;
pub use fluid_particle::{
    add_sph_particle as fluid_add_sph_particle, compute_densities as fluid_compute_densities,
    compute_pressures as fluid_compute_pressures, compute_sph_forces,
    default_sph_config as fluid_default_sph_config, enforce_sph_bounds, integrate_sph,
    new_sph_world as fluid_new_sph_world, sph_kernel_poly6, sph_kernel_spiky_grad,
    sph_kernel_viscosity_lap, sph_particle_count as fluid_sph_particle_count,
    sph_step as fluid_sph_step, FluidSphConfig as SphFluidConfig,
    FluidSphParticle as SphFluidParticle, FluidSphWorld as SphFluidWorld,
};

#[path = "ocean_waves.rs"]
pub mod ocean_waves;
pub use ocean_waves::{
    advance_ocean, default_ocean_config, gerstner_displacement, gerstner_normal,
    gerstner_phase_speed, new_ocean_surface, ocean_displacement, ocean_foam_mask, ocean_height_at,
    ocean_normal, sample_ocean_grid, wave_frequency, GerstnerWave, OceanConfig, OceanSurface,
};

#[path = "particle_emitter.rs"]
pub mod particle_emitter;
pub use particle_emitter::{
    alive_emitter_count, clear_emitter_particles, default_emitter_config, disable_emitter,
    emit_burst, emitter_particle_count, emitter_point_position, emitter_total_spawned,
    enable_emitter, new_particle_emitter as new_pe_particle_emitter, particle_age_fraction_pe,
    spawn_particle as pe_spawn_particle, update_emitter, PeEmittedParticle as EmittedParticlePe,
    PeEmitterConfig as EmitterConfigPe, PeEmitterShape as EmitterShapePe,
    PeParticleEmitter as ParticleEmitterPe,
};

#[path = "wind_field.rs"]
pub mod wind_field;
pub use wind_field::{
    add_wind_zone, apply_wind_to_particles, default_wind_config as wind_field_default_config,
    new_wind_zone, remove_wind_zone, turbulence_offset, update_wind_time, wind_at_point,
    wind_direction_degrees, wind_drag_force as wf_drag_force, wind_gust_factor,
    wind_lift_force as wf_lift_force, wind_speed as wf_wind_speed, wind_zone_count,
    WindFieldConfig, WindFieldSample, WindZone, WindZoneOp,
};

#[path = "shape_matching.rs"]
pub mod shape_matching;
pub use shape_matching::{
    apply_shape_matching, body_volume_estimate, compute_apq, compute_com,
    default_shape_matching_config, deformation_energy, new_shape_matching_body,
    polar_extract_rotation, reset_to_rest, set_particle_mass, shape_matching_particle_count,
    shape_matching_stiffness, Mat3, ShapeMatchingBody, ShapeMatchingConfig,
};

#[path = "spring_damper.rs"]
pub mod spring_damper;
pub use spring_damper::{
    add_spring as add_damper_spring, compute_spring_force, default_spring_damper_config,
    new_spring_damper, new_spring_damper_system, remove_spring, reset_spring_system,
    set_spring_damping, set_spring_stiffness, spring_count, spring_energy,
    spring_extension as damper_spring_extension, total_system_energy, update_spring_system,
    SpringDamper, SpringDamperConfig, SpringDamperSystem,
};

#[path = "spatial_hash.rs"]
pub mod spatial_hash;
pub use spatial_hash::{
    cell_count, clear_grid, default_spatial_hash_config, entry_count, grid_stats, hash_position,
    insert_aabb, insert_point, new_spatial_hash, query_aabb as spatial_query_aabb, query_point,
    query_radius, rebuild_grid, remove_entry, GridStats, SpatialHashConfig, SpatialHashEntry,
    SpatialHashGrid,
};

#[path = "constraint_solver.rs"]
pub mod constraint_solver;
pub use constraint_solver::{
    add_constraint, angle_constraint_project, constraint_compliance, constraint_count,
    default_solver_config, distance_constraint_project, new_solver_state,
    position_constraint_project, remove_constraint, reset_solver, set_solver_iterations,
    solve_iteration, solve_n_iterations, total_constraint_error, ConstraintSolverConfig,
    ConstraintSolverState, ConstraintType, GenericConstraint,
};

#[path = "gyroscope.rs"]
pub mod gyroscope;
pub use gyroscope::{
    angular_momentum, apply_gyro_torque, damped_gyro_update, default_gyro_config,
    gyro_stability_metric, gyroscopic_torque, new_gyro_body, nutation_angle, precession_rate,
    set_angular_velocity, set_inertia_tensor, spin_energy, update_gyro_state, GyroBody, GyroConfig,
    GyroState,
};

#[path = "rope_cloth.rs"]
pub mod rope_cloth;
pub use rope_cloth::{
    add_cloth_quad, add_rope_segment, apply_gravity_rope_cloth, cloth_quad_count,
    default_rope_cloth_config, new_rope_cloth_body, pin_rope_cloth_particle, reset_rope_cloth,
    rope_cloth_energy, rope_cloth_particle_count, rope_segment_count, unpin_rope_cloth_particle,
    update_rope_cloth, AnnotatedConstraint, ClothQuadRecord, ConstraintKind as RopeConstraintKind,
    EnergyPair, RopeClothBody, RopeClothConfig, RopeClothConstraint, RopeClothParticle,
    RopeSegmentRecord,
};

#[path = "pressure_force.rs"]
pub mod pressure_force;
pub use pressure_force::{
    apply_pressure_forces, compute_enclosed_volume, default_pressure_config, deflate, inflate,
    new_pressure_body, pressure_body_vertex_count, pressure_body_volume, pressure_energy,
    pressure_force_on_triangle, sample_pressure_triangle, set_pressure, update_pressure_body,
    PressureBody, PressureConfig, PressureEnergyPair, PressureSample, PressureVertex,
};

#[path = "ball_joint.rs"]
pub mod ball_joint;
pub use ball_joint::BallJoint;

#[path = "body_sleeping.rs"]
pub mod body_sleeping;
pub use body_sleeping::BodySleeping;

#[path = "capsule_shape.rs"]
pub mod capsule_shape;
pub use capsule_shape::CapsuleShape;

#[path = "collision_response_model.rs"]
pub mod collision_response_model;
pub use collision_response_model::CollisionResponseModel;

#[path = "constraint_bound.rs"]
pub mod constraint_bound;
pub use constraint_bound::ConstraintBound;

#[path = "contact_friction_model.rs"]
pub mod contact_friction_model;
pub use contact_friction_model::ContactFrictionModel;

#[path = "damped_spring.rs"]
pub mod damped_spring;
pub use damped_spring::DampedSpring;

#[path = "elastic_surface.rs"]
pub mod elastic_surface;
pub use elastic_surface::ElasticSurface;

#[path = "force_clamp.rs"]
pub mod force_clamp;
pub use force_clamp::ForceClamp;

#[path = "gravity_model.rs"]
pub mod gravity_model;
pub use gravity_model::GravityModel;

#[path = "impulse_cache_v2.rs"]
pub mod impulse_cache_v2;
pub use impulse_cache_v2::{CacheEntry as ImpulseCacheEntry, ImpulseCacheV2};

#[path = "joint_motor_drive.rs"]
pub mod joint_motor_drive;
pub use joint_motor_drive::{DriveMode, JointMotorDrive};

#[path = "kinematic_target.rs"]
pub mod kinematic_target;
pub use kinematic_target::KinematicTarget;

#[path = "mass_distribution.rs"]
pub mod mass_distribution;
pub use mass_distribution::MassDistribution;

#[path = "plane_collider.rs"]
pub mod plane_collider;
pub use plane_collider::PlaneCollider;

#[path = "pulley_joint.rs"]
pub mod pulley_joint;
pub use pulley_joint::PulleyJoint;

#[path = "angular_spring.rs"]
pub mod angular_spring;
pub use angular_spring::AngularSpring;

#[path = "body_aabb.rs"]
pub mod body_aabb;
pub use body_aabb::BodyAabb;

#[path = "capsule_contact.rs"]
pub mod capsule_contact;
pub use capsule_contact::{CapsuleContactResult, PhysCapsule};

#[path = "collision_layer_matrix.rs"]
pub mod collision_layer_matrix;
pub use collision_layer_matrix::CollisionLayerMatrix;

#[path = "cone_twist.rs"]
pub mod cone_twist;
pub use cone_twist::ConeTwist;

#[path = "contact_pair.rs"]
pub mod contact_pair;
pub use contact_pair::ContactPairSet;

#[path = "continuous_collision.rs"]
pub mod continuous_collision;
pub use continuous_collision::{CcdResult, MovingSphere};

#[path = "damper_element.rs"]
pub mod damper_element;
pub use damper_element::{DamperElement, DamperElement3d};

#[path = "dynamic_body.rs"]
pub mod dynamic_body;
pub use dynamic_body::DynamicBody;

#[path = "elastic_body.rs"]
pub mod elastic_body;
pub use elastic_body::ElasticBody;

#[path = "force_field_radial.rs"]
pub mod force_field_radial;
pub use force_field_radial::{FalloffType, ForceFieldRadial};

#[path = "friction_joint.rs"]
pub mod friction_joint;
pub use friction_joint::FrictionJoint;

#[path = "gravity_well.rs"]
pub mod gravity_well;
pub use gravity_well::GravityWell;

#[path = "hinge_limit.rs"]
pub mod hinge_limit;
pub use hinge_limit::HingeLimit;

#[path = "impulse_pair.rs"]
pub mod impulse_pair;
pub use impulse_pair::{ImpulsePair, ImpulsePairBuffer};

#[path = "inertia_body.rs"]
pub mod inertia_body;
pub use inertia_body::InertiaBody;

#[path = "angular_velocity_body.rs"]
pub mod angular_velocity_body;
pub use angular_velocity_body::{
    axis_angle_to_angular_velocity, clamp_angular_velocity,
    integrate_orientation as avb_integrate_orientation,
    normalize_quaternion as avb_normalize_quaternion, rotation_period, AngularVelocityBody,
};

#[path = "body_friction.rs"]
pub mod body_friction;
pub use body_friction::{
    anisotropic_friction, coulomb_friction, kinetic_friction, max_static_friction,
    viscous_friction, BodyFrictionParams, FrictionModelType,
};
