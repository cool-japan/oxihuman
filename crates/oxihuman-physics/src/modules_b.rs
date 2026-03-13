// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Auto-generated module re-exports — do not edit directly.

#[path = "capsule_pair.rs"]
pub mod capsule_pair;
pub use capsule_pair::{
    capsule_pair_normal, capsule_pair_overlap, capsule_pair_penetration, capsule_pair_sq_dist,
    closest_point_on_segment as cp_closest_on_segment, closest_points_segment_segment, CapsulePrim,
};

#[path = "collision_normal.rs"]
pub mod collision_normal;
pub use collision_normal::{
    flip_normal, is_valid_normal, normal_box_point, normal_dot, normal_sphere_sphere,
};

#[path = "cone_limit.rs"]
pub mod cone_limit;
pub use cone_limit::{angle_within_deg, cone_solid_angle, ConeLimit};

#[path = "contact_resolver.rs"]
pub mod contact_resolver;
pub use contact_resolver::{
    combined_restitution, positional_correction, relative_normal_velocity,
    resolve_contact as resolve_contact_impulse, ResolvableBody, ResolveContact,
};

#[path = "damped_body.rs"]
pub mod damped_body;
pub use damped_body::{
    critical_damping as db_critical_damping, damping_factor, overdamped_settling_time, DampedBody,
};

#[path = "deformable_spring.rs"]
pub mod deformable_spring;
pub use deformable_spring::{natural_frequency, spring_force_vector, DeformableSpring};

#[path = "distance_field_phys.rs"]
pub mod distance_field_phys;
pub use distance_field_phys::{bake_sphere_sdf, box_sdf, sphere_sdf, DistanceField, SdfContact};

#[path = "elastic_wave.rs"]
pub mod elastic_wave;
pub use elastic_wave::{cfl_stable_dt, phase_velocity, standing_wave_frequency, ElasticChain};

#[path = "force_limit.rs"]
pub mod force_limit;
pub use force_limit::{
    clamp_force, force_within_limit, limit_impulse, safe_force, BudgetedForceAccum,
    PerAxisForceLimit,
};

#[path = "friction_patch.rs"]
pub mod friction_patch;
pub use friction_patch::{
    compute_tangent_basis, is_sticking, kinetic_friction_magnitude, FrictionPatch, FrictionState,
};

#[path = "gravity_source.rs"]
pub mod gravity_source;
pub use gravity_source::{
    combined_gravity, gravitational_potential, orbital_period, GravitySource, UniformGravity, G,
};

#[path = "hinge_spring.rs"]
pub mod hinge_spring;
pub use hinge_spring::{torsional_critical_damping, torsional_natural_frequency, HingeSpring};

#[path = "joint_anchor.rs"]
pub mod joint_anchor;
pub use joint_anchor::{rotate_by_quaternion, AnchorSpace, JointAnchor, JointAnchorPair};

#[path = "angular_motor.rs"]
pub mod angular_motor;
pub use angular_motor::{
    angle_diff, critical_damping as motor_critical_damping, motor_at_target, motor_kinetic_energy,
    motor_set_target, motor_step, motor_torque, new_angular_motor, wrap_angle, AngularMotor,
};

#[path = "body_collision_group.rs"]
pub mod body_collision_group;
pub use body_collision_group::{
    cg_add_layer, cg_disable_filter, cg_enable_filter, cg_in_layer, cg_layer_count, cg_merge,
    cg_remove_layer, default_collision_group, ghost_collision_group, groups_collide,
    new_collision_group, CollisionGroup, CollisionMask,
};

#[path = "broad_phase_pair.rs"]
pub mod broad_phase_pair;
pub use broad_phase_pair::{
    aabb_center, aabb_expand as bpp_aabb_expand, aabb_merge, aabb_overlaps, aabb_volume,
    broad_phase_naive, new_broad_aabb, BroadAabb, OverlapPair,
};

#[path = "capsule_ray.rs"]
pub mod capsule_ray;
pub use capsule_ray::{
    capsule_surface_area, closest_t_on_segment as cr_closest_t, normalise as cr_normalise,
    ray_capsule_intersect, Ray, RayCapsule,
};

#[path = "cloth_constraint.rs"]
pub mod cloth_constraint;
pub use cloth_constraint::{
    bend_angle as cloth_bend_angle, dist_constraint_delta, dist_constraint_violation,
    dist_satisfied, flat_rest_angle, new_bend_constraint, new_dist_constraint,
    stiffness_from_compliance, ClothBendConstraint, ClothDistConstraint,
};

#[path = "cone_body.rs"]
pub mod cone_body;
pub use cone_body::{
    cone_apply_impulse, cone_centroid_height, cone_inertia_axial, cone_inertia_transverse_apex,
    cone_inertia_transverse_centroid, cone_integrate, cone_kinetic_energy, cone_surface_area,
    cone_volume, new_cone_body, ConeBody,
};

#[path = "contact_island.rs"]
pub mod contact_island;
pub use contact_island::{
    build_islands, island_count, island_labels, island_size, new_island_uf, same_island, uf_find,
    uf_union, IslandUnionFind,
};

#[path = "damping_ratio.rs"]
pub mod damping_ratio;
pub use damping_ratio::{
    classify_damping, critical_damping_coeff, damped_frequency, damped_period,
    damping_ratio as compute_damping_ratio, frequency_response,
    natural_frequency as dr_natural_frequency, peak_overshoot, settling_time_2pct, DampingCategory,
};

#[path = "deformable_mesh.rs"]
pub mod deformable_mesh;
pub use deformable_mesh::{
    dm_add_edge, dm_add_particle, dm_integrate, dm_kinetic_energy, dm_particle_count,
    dm_potential_energy, dm_spring_forces, new_deformable_mesh, DeformParticle, DeformableMesh,
};

#[path = "elastic_joint.rs"]
pub mod elastic_joint;
pub use elastic_joint::{
    elastic_critical_damping, elastic_damping_force, elastic_natural_frequency,
    elastic_potential_energy, elastic_set_rest, elastic_spring_force, elastic_within_limits,
    new_elastic_joint, ElasticJoint,
};

#[path = "force_field_uniform.rs"]
pub mod force_field_uniform;
pub use force_field_uniform::{
    field_acceleration_at, field_contains, field_force_at, field_magnitude, field_set_enabled,
    field_set_scale, field_work, new_bounded_field, new_uniform_field, FieldRegion,
    UniformForceField,
};

#[path = "friction_surface_model.rs"]
pub mod friction_surface_model;
pub use friction_surface_model::{
    angle_of_repose, combine_friction as fsm_combine_friction,
    combine_restitution as fsm_combine_restitution, friction_impulse, friction_power, ice_ice,
    is_sticking_contact, kinetic_friction_force, rolling_resistance, rubber_concrete,
    static_friction_on_slope, SurfaceMaterial,
};

#[path = "gravity_zone.rs"]
pub mod gravity_zone;
pub use gravity_zone::{
    gravity_at, new_box_zone, new_sphere_zone, zone_contains, zone_set_enabled, zone_set_gravity,
    zone_volume, GravityZone, ZoneShape,
};

#[path = "hinge_body.rs"]
pub mod hinge_body;
pub use hinge_body::{
    hinge_apply_torque, hinge_at_limit, hinge_full_turn, hinge_kinetic_energy,
    hinge_normalised_angle, hinge_range, hinge_reset, new_hinge_body, HingeBody,
};

#[path = "impulse_response.rs"]
pub mod impulse_response;
pub use impulse_response::{
    apply_impulse_a, apply_impulse_b, impulse_magnitude, relative_velocity,
};

#[path = "joint_damper.rs"]
pub mod joint_damper;
pub use joint_damper::{
    damper_angular_torque, damper_decay_factor, damper_energy_step, damper_linear_force,
    damper_linear_power, damper_set_angular, damper_set_enabled, damper_set_linear,
    new_joint_damper, JointDamper,
};

#[path = "kinematic_controller.rs"]
pub mod kinematic_controller;
pub use kinematic_controller::{
    kc_heading_deg, kc_integrate, kc_jump, kc_kinetic_energy, kc_move, kc_on_ground, kc_speed,
    kc_stop, new_kinematic_controller, KinematicController,
};

#[path = "lattice_spring.rs"]
pub mod lattice_spring;
pub use lattice_spring::{
    ls_aabb, ls_add_particle, ls_add_spring, ls_center_of_mass, ls_kinetic_energy,
    ls_particle_count, ls_spring_count, ls_step, new_lattice_sim, LatticeParticle, LatticeSim,
    LatticeSpring,
};

#[path = "linear_actuator.rs"]
pub mod linear_actuator;
pub use linear_actuator::{
    la_at_target, la_kinetic_energy, la_normalized_pos, la_potential_energy, la_range, la_reset,
    la_set_target, la_step, new_linear_actuator, LinearActuator,
};

#[path = "mass_spring_chain.rs"]
pub mod mass_spring_chain;
pub use mass_spring_chain::{
    msc_kinetic_energy, msc_max_velocity, msc_node_count, msc_potential_energy,
    msc_reset_velocities, msc_span, msc_step, new_mass_spring_chain, ChainNode, MassSpringChain,
};

#[path = "mesh_collider.rs"]
pub mod mesh_collider;
pub use mesh_collider::{
    make_unit_tri, mc_nearest_tri, mc_point_in_aabb, mc_sphere_overlaps, mc_total_area,
    mc_tri_count, new_mesh_collider, MeshCollider, Triangle,
};

#[path = "motor_joint_v2.rs"]
pub mod motor_joint_v2;
pub use motor_joint_v2::{
    angle_diff as mjv2_angle_diff, mjv2_at_target, mjv2_kinetic_energy, mjv2_normalized_angle,
    mjv2_range, mjv2_reset, mjv2_set_target, mjv2_step, new_motor_joint_v2, MotorJointV2,
};

#[path = "particle_chain.rs"]
pub mod particle_chain;
pub use particle_chain::{
    new_particle_chain, pc_avg_seg_len, pc_chain_length, pc_end_pos, pc_particle_count, pc_step,
    pc_tip_distance, ChainParticle, ParticleChain,
};

#[path = "pendulum_body.rs"]
pub mod pendulum_body;
pub use pendulum_body::{
    new_pendulum_body, pb_bob_pos, pb_frequency, pb_is_at_rest, pb_kinetic_energy, pb_period,
    pb_potential_energy, pb_step, pb_total_energy, PendulumBody,
};

#[path = "phase_space.rs"]
pub mod phase_space;
pub use phase_space::{
    new_phase_space, ps_apply_force, ps_apply_harmonic, ps_kinetic_energy, ps_max_p, ps_max_q,
    ps_period, ps_reset, ps_traj_len, ps_velocity, PhasePoint, PhaseSpace,
};

#[path = "pivot_body.rs"]
pub mod pivot_body;
pub use pivot_body::{
    new_pivot_body, pivb_angle_deg, pivb_apply_torque, pivb_inertia, pivb_kinetic_energy,
    pivb_reset, pivb_tip_pos, pivb_tip_velocity, PivotBody,
};

#[path = "plane_body.rs"]
pub mod plane_body;
pub use plane_body::{
    new_plane_body, plane_closest_point, plane_is_above, plane_project_vel, plane_reflect_vel,
    plane_signed_dist, plane_sphere_penetration, PlaneBody,
};

#[path = "position_integrator.rs"]
pub mod position_integrator;
pub use position_integrator::{
    new_position_integrator, pi_displacement, pi_kinetic_energy, pi_reset, pi_speed, pi_step,
    pi_steps, IntegratorKind, PositionIntegrator,
};

#[path = "potential_energy.rs"]
pub mod potential_energy;
pub use potential_energy::{
    bending_pe, centrifugal_pe, coulomb_pe, free_fall_height, gravitational_pe, lennard_jones_pe,
    newtonian_grav_pe, spring_oscillator_period, spring_pe, torsional_pe, total_spring_pe,
};

#[path = "pressure_body.rs"]
pub mod pressure_body;
pub use pressure_body::{
    new_pressure_body as new_pressure_body_inflate, prb_compress, prb_density, prb_expand,
    prb_is_over_pressured, prb_normalized_volume, prb_outward_force, prb_pressure,
    prb_restoring_force, prb_volume_error, PressureBody as InflatablePressureBody,
};

#[path = "prismatic_joint.rs"]
pub mod prismatic_joint;
pub use prismatic_joint::{
    new_prismatic_joint, pj_apply_force, pj_at_limit, pj_kinetic_energy, pj_lock,
    pj_normalized_pos, pj_range, pj_reset, pj_unlock, pj_world_offset, PrismaticJoint,
};

#[path = "projectile_body.rs"]
pub mod projectile_body;
pub use projectile_body::{
    new_projectile_body, proj_horizontal_range, proj_is_active, proj_kinetic_energy, proj_launch,
    proj_max_range_vacuum, proj_speed, proj_step, proj_time_of_flight, ProjectileBody,
};

#[path = "rack_pinion.rs"]
pub mod rack_pinion;
pub use rack_pinion::{new_rack_pinion, RackPinion};

#[path = "restitution_body.rs"]
pub mod restitution_body;
pub use restitution_body::{collision_impulse, new_restitution_body, RestitutionBody};

#[path = "revolute_joint.rs"]
pub mod revolute_joint;
pub use revolute_joint::{new_revolute_joint, RevoluteJoint};

#[path = "rigid_body_group.rs"]
pub mod rigid_body_group;
pub use rigid_body_group::{new_rigid_body_group, GroupMember, RigidBodyGroup};

#[path = "rope_segment.rs"]
pub mod rope_segment;
pub use rope_segment::{
    new_rope_segment, RopeParticle as RopeChainParticle, RopeSegment as RopeSegmentChain,
};

#[path = "rotational_body.rs"]
pub mod rotational_body;
pub use rotational_body::{new_rotational_body, RotationalBody};

#[path = "screw_joint.rs"]
pub mod screw_joint;
pub use screw_joint::{new_screw_joint, ScrewJoint};

#[path = "shape_intersection.rs"]
pub mod shape_intersection;
pub use shape_intersection::{
    capsule_capsule_closest, segment_segment_dist, sphere_plane_intersect, sphere_sphere_intersect,
};

#[path = "shock_absorber.rs"]
pub mod shock_absorber;
pub use shock_absorber::{new_shock_absorber, ShockAbsorber};

#[path = "sliding_body.rs"]
pub mod sliding_body;
pub use sliding_body::{new_sliding_body, SlidingBody};

#[path = "soft_body_volume.rs"]
pub mod soft_body_volume;
pub use soft_body_volume::{new_soft_body_volume, tet_signed_volume, SoftBodyVolume, SoftParticle};

#[path = "sphere_body.rs"]
pub mod sphere_body;
pub use sphere_body::{new_sphere_body, SphereBody};

#[path = "spring_chain.rs"]
pub mod spring_chain;
pub use spring_chain::{new_spring_chain, ChainMass, ChainSpring, SpringChain};

#[path = "static_body.rs"]
pub mod static_body;
pub use static_body::{
    new_static_box, new_static_plane, new_static_sphere, StaticBody, StaticShape,
};

#[path = "torsion_body.rs"]
pub mod torsion_body;
pub use torsion_body::{new_torsion_body, TorsionBody};

#[path = "universal_joint.rs"]
pub mod universal_joint;
pub use universal_joint::{new_universal_joint, UniversalJoint};

#[path = "velocity_verlet.rs"]
pub mod velocity_verlet;
pub use velocity_verlet::{new_velocity_verlet, VelocityVerlet, VvParticle};

#[path = "vibration_body.rs"]
pub mod vibration_body;
pub use vibration_body::{new_vibration_body, VibrationBody};

#[path = "viscous_body.rs"]
pub mod viscous_body;
pub use viscous_body::{new_viscous_body, ViscousBody};

#[path = "vortex_field.rs"]
pub mod vortex_field;
pub use vortex_field::{circulation, new_vortex_field, Vortex, VortexField};

#[path = "wave_body.rs"]
pub mod wave_body;
pub use wave_body::{new_wave_body, WaveBody};

#[path = "wheel_body.rs"]
pub mod wheel_body;
pub use wheel_body::{new_wheel_body, WheelBody};

#[path = "wind_body.rs"]
pub mod wind_body;
pub use wind_body::{new_wind_body, WindBody};

#[path = "xpbd_cloth.rs"]
pub mod xpbd_cloth;
pub use xpbd_cloth::{
    new_xpbd_cloth, ClothConstraint, ClothParticle as XpbdClothParticle, XpbdCloth,
};

#[path = "xpbd_particle.rs"]
pub mod xpbd_particle;
pub use xpbd_particle::{
    new_xpbd_particle_system, XpbdParticle as XpbdSimParticle, XpbdParticleSystem,
};

#[path = "xpbd_shape.rs"]
pub mod xpbd_shape;
pub use xpbd_shape::{new_xpbd_shape, ShapeParticle, XpbdShape};

#[path = "xpbd_volume.rs"]
pub mod xpbd_volume;
pub use xpbd_volume::{
    new_xpbd_volume, tet_signed_volume as xpbd_tet_signed_volume,
    VolumeConstraint as XpbdVolumeConstraint, XpbdVolume,
};

#[path = "yoke_joint.rs"]
pub mod yoke_joint;
pub use yoke_joint::{new_yoke_joint, YokeJoint};

#[path = "zero_gravity_body.rs"]
pub mod zero_gravity_body;
pub use zero_gravity_body::{new_zero_gravity_body, ZeroGravityBody};

#[path = "buoyant_body.rs"]
pub mod buoyant_body;
pub use buoyant_body::{new_buoyant_body, BuoyantBody};

#[path = "centrifugal_body.rs"]
pub mod centrifugal_body;
pub use centrifugal_body::{new_centrifugal_body, CentrifugalBody};

#[path = "coriolis_body.rs"]
pub mod coriolis_body;
pub use coriolis_body::{new_coriolis_body, CoriolisBody};

#[path = "aerial_body.rs"]
pub mod aerial_body;
pub use aerial_body::{
    ab_drag, ab_dynamic_pressure, ab_lift, ab_step, angle_of_attack, new_aerial_body, AerialBody,
    AerialBodyConfig,
};

#[path = "atmosphere_body.rs"]
pub mod atmosphere_body;
pub use atmosphere_body::{
    density_at, mach_number, new_atmosphere_body, pressure_at, speed_of_sound, temperature_at,
    AtmoLayer, AtmosphereBody,
};

#[path = "cable_body.rs"]
pub mod cable_body;
pub use cable_body::{cb_particle_count, cb_step, new_cable_body, CableBody, CableParticle};

#[path = "collision_event_log.rs"]
pub mod collision_event_log;
pub use collision_event_log::{
    cel_event_count, cel_record, new_collision_event_log, CollisionEvent, CollisionEventLog,
};

#[path = "constraint_graph.rs"]
pub mod constraint_graph;
pub use constraint_graph::{
    cg_add, cg_edge_count, cg_remove, new_constraint_graph, ConstraintEdge,
    ConstraintGraph as CgConstraintGraph, ConstraintType as CgConstraintType,
};

#[path = "damper_network.rs"]
pub mod damper_network;
pub use damper_network::{
    dn_add_link, dn_add_node, dn_step, new_damper_network, DamperLink, DamperNetwork, DamperNode,
};
