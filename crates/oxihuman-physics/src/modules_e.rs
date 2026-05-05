// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Auto-generated module re-exports — do not edit directly.

#[path = "sensor_motion_capture.rs"]
pub mod sensor_motion_capture;
pub use sensor_motion_capture::{
    all_markers_valid, find_marker, marker_centroid, marker_distance, visible_marker_count,
    MocapConfig, MocapFrame, MocapMarker, MocapSensor,
};

#[path = "sensor_pressure_mat.rs"]
pub mod sensor_pressure_mat;
pub use sensor_pressure_mat::{
    active_cells, centre_of_pressure as pressure_mat_cop, contact_area_m2 as pressure_contact_area,
    peak_pressure, total_force_n, PressureFrame, PressureMatConfig, PressureMatSensor,
};

#[path = "sensor_flex.rs"]
pub mod sensor_flex;
pub use sensor_flex::{
    angle_to_resistance, max_bend_sample, mean_angle as flex_mean_angle, resistance_in_range,
    resistance_to_angle, FlexConfig, FlexSample, FlexSensor,
};

#[path = "sensor_strain_gauge.rs"]
pub mod sensor_strain_gauge;
pub use sensor_strain_gauge::{
    delta_resistance, mean_strain, peak_strain, resistance_to_strain, strain_overrange,
    strain_to_stress_pa, StrainGaugeConfig, StrainGaugeSensor, StrainSample,
};

#[path = "sensor_load_cell.rs"]
pub mod sensor_load_cell;
pub use sensor_load_cell::{
    compute_impulse, force_overrange as load_cell_overrange, force_to_voltage, mean_force,
    nonlinearity_error_n, peak_force, voltage_to_force, LoadCellConfig, LoadCellSample,
    LoadCellSensor,
};

#[path = "sensor_encoder.rs"]
pub mod sensor_encoder;
pub use sensor_encoder::{
    angular_velocity_rad_s, angular_velocity_rpm, count_to_degrees, count_to_radians,
    current_angle_deg as encoder_angle_deg, rpm_overrange, EncoderConfig, EncoderSample,
    EncoderSensor,
};

#[path = "sensor_potentiometer.rs"]
pub mod sensor_potentiometer;
pub use sensor_potentiometer::{
    angle_to_voltage, angular_velocity_deg_s, current_angle_deg as pot_angle_deg,
    max_linearity_error_deg, voltage_in_range, voltage_to_angle, PotSample, PotentiometerConfig,
    PotentiometerSensor,
};

#[path = "sensor_ultrasonic.rs"]
pub mod sensor_ultrasonic;
pub use sensor_ultrasonic::{
    beam_footprint_m, distance_in_range, distance_to_tof, median_distance, tof_to_distance,
    valid_fraction, UltrasonicConfig, UltrasonicSample, UltrasonicSensor,
};

#[path = "sensor_lidar.rs"]
pub mod sensor_lidar;
pub use sensor_lidar::{
    cloud_centroid, filter_by_range, high_intensity_count, max_point_range, point_range,
    LidarConfig, LidarFrame, LidarPoint, LidarSensor,
};

#[path = "sensor_camera.rs"]
pub mod sensor_camera;
pub use sensor_camera::{
    backproject_ray, fov_deg, pixel_area_at_depth, pixel_in_bounds, project_point,
    CameraIntrinsics, CameraPose, CameraStub,
};

#[path = "sensor_depth_camera.rs"]
pub mod sensor_depth_camera;
pub use sensor_depth_camera::{
    expected_pixel_count, mean_depth, unproject_frame, valid_pixel_count, DepthCameraConfig,
    DepthCameraSensor, DepthFrame, DepthPoint,
};

#[path = "sensor_tactile.rs"]
pub mod sensor_tactile;
pub use sensor_tactile::{
    active_taxels, any_taxel_overrange, centre_of_force, contact_area_m2 as tactile_contact_area,
    peak_taxel_force, total_contact_force, TactileConfig, TactileFrame, TactileSensor,
};

#[path = "sensor_temperature.rs"]
pub mod sensor_temperature;
pub use sensor_temperature::{
    celsius_to_fahrenheit, celsius_to_kelvin, fahrenheit_to_celsius, is_fever, mean_temperature,
    quantise as temperature_quantise, samples_at_site, temperature_in_range, TemperatureConfig,
    TemperatureSample, TemperatureSensor, TemperatureSensorType,
};

#[path = "actuator_dc_motor.rs"]
pub mod actuator_dc_motor;
pub use actuator_dc_motor::{
    clamp_voltage, compute_current as dc_motor_compute_current,
    compute_torque as dc_motor_compute_torque, no_load_speed, stall_torque,
    step_motor as dc_step_motor, DcMotor, DcMotorParams, DcMotorState,
};

#[path = "actuator_servo.rs"]
pub mod actuator_servo;
pub use actuator_servo::{
    angle_error, at_target, compute_servo_torque, new_servo_state, set_target, step_servo, RcServo,
    ServoConfig, ServoState,
};

#[path = "actuator_stepper.rs"]
pub mod actuator_stepper;
pub use actuator_stepper::{
    current_angle_rad, de_energize, effective_step_angle, energize, move_to_angle,
    new_stepper_state, step_motor as stepper_step_motor, steps_to_angle, StepperConfig,
    StepperMotor, StepperState,
};

#[path = "actuator_hydraulic.rs"]
pub mod actuator_hydraulic;
pub use actuator_hydraulic::{
    extension_force, extension_ratio as hydraulic_extension_ratio, hydraulic_power,
    retraction_force, set_valve_command, step_cylinder, HydraulicCylinder, HydraulicCylinderParams,
    HydraulicCylinderState,
};

#[path = "actuator_pneumatic.rs"]
pub mod actuator_pneumatic;
pub use actuator_pneumatic::{
    extension_ratio as pneumatic_extension_ratio, is_fully_extended, piston_force, pneumatic_power,
    set_valve, step_pneumatic, PneumaticCylinder, PneumaticCylinderParams, PneumaticCylinderState,
};

#[path = "actuator_linear_motor.rs"]
pub mod actuator_linear_motor;
pub use actuator_linear_motor::{
    coil_power_dissipation, compute_linear_current, compute_thrust, no_load_velocity, peak_thrust,
    step_linear_motor, LinearMotor, LinearMotorParams, LinearMotorState,
};

#[path = "actuator_cable_drive.rs"]
pub mod actuator_cable_drive;
pub use actuator_cable_drive::{
    cable_length_from_spool, compute_cable_tension, is_cable_slack, new_cable_state, spool_torque,
    step_cable_drive, wind_spool, CableDrive, CableDriveParams, CableDriveState,
};

#[path = "actuator_gear_train.rs"]
pub mod actuator_gear_train;
pub use actuator_gear_train::{
    add_stage, reflected_inertia as gear_reflected_inertia, single_stage_gear, stage_ratio,
    total_efficiency, total_ratio, update_gear_train, GearStageParams, GearTrain,
    GearTrainActuator, GearTrainState,
};

#[path = "actuator_harmonic_drive.rs"]
pub mod actuator_harmonic_drive;
pub use actuator_harmonic_drive::{
    back_drive_torque as harmonic_back_drive_torque, input_speed_valid, output_speed,
    output_torque as harmonic_output_torque, reflected_inertia as harmonic_reflected_inertia,
    update_harmonic_drive, HarmonicDriveActuator, HarmonicDriveParams, HarmonicDriveState,
};

#[path = "actuator_ball_screw.rs"]
pub mod actuator_ball_screw;
pub use actuator_ball_screw::{
    axial_force_to_torque, load_within_rating,
    mechanical_advantage as ball_screw_mechanical_advantage, omega_to_linear_velocity,
    step_ball_screw, torque_to_axial_force, BallScrewActuator, BallScrewParams, BallScrewState,
};

#[path = "actuator_rack_pinion.rs"]
pub mod actuator_rack_pinion;
pub use actuator_rack_pinion::{
    mechanical_advantage as rack_mechanical_advantage, omega_to_rack_velocity,
    rack_force_to_torque, rack_travel_ratio, step_rack_pinion, torque_to_rack_force,
    RackPinionActuator, RackPinionParams, RackPinionState,
};

#[path = "actuator_worm_gear.rs"]
pub mod actuator_worm_gear;
pub use actuator_worm_gear::{
    back_drive_torque as worm_back_drive_torque, forward_efficiency,
    gear_ratio as actuator_worm_gear_ratio, input_speed_from_output, is_self_locking,
    update_worm_gear, WormGearActuator, WormGearParams, WormGearState,
};

#[path = "actuator_differential.rs"]
pub mod actuator_differential;
pub use actuator_differential::{
    average_output_omega, speed_difference, total_output_torque, update_differential, yaw_rate,
    DifferentialDrive, DifferentialParams, DifferentialState,
};

#[path = "actuator_parallel_robot.rs"]
pub mod actuator_parallel_robot;
pub use actuator_parallel_robot::{
    forward_kinematics, home_z, joints_in_limits, set_joint_angles, workspace_radius,
    DeltaJointAngles, DeltaRobot, DeltaRobotParams, DeltaRobotState, Vec3 as DeltaVec3,
};

#[path = "actuator_tendon_drive.rs"]
pub mod actuator_tendon_drive;
pub use actuator_tendon_drive::{
    is_grasping, joint_torques, new_tendon_state, step_tendon_drive, tendon_displacement,
    total_closure, TendonDriveFinger, TendonDriveParams, TendonDriveState,
};

#[path = "actuator_soft_robot.rs"]
pub mod actuator_soft_robot;
pub use actuator_soft_robot::{
    any_at_max_pressure, chamber_force, new_soft_robot_state, set_chamber_pressure,
    step_soft_robot, total_elongation, SoftRobotActuator, SoftRobotParams, SoftRobotState,
};

#[path = "rigid_body_2d.rs"]
pub mod rigid_body_2d;
pub use rigid_body_2d::{
    angular_momentum_2d, apply_gravity_2d, apply_impulse_2d, body_2d_momentum, RigidBody2d,
};

#[path = "collision_2d.rs"]
pub mod collision_2d;
pub use collision_2d::{
    aabb_aabb_2d, aabb_circle_2d, aabb_overlap_area, circle_circle_2d,
    point_in_aabb as point_in_aabb_2d, point_in_circle, Aabb2d, Circle2d,
};

#[path = "joint_2d.rs"]
pub mod joint_2d;
pub use joint_2d::{
    clamp_angle, joint_damping_force, joint_error, joint_spring_force, Joint2d, JointKind2d,
};

#[path = "chain_pendulum_2d.rs"]
pub mod chain_pendulum_2d;
pub use chain_pendulum_2d::{
    chain_end_pos, link_energy, small_angle_period, ChainPendulum2d, PendulumLink2d,
};

#[path = "car_physics_2d.rs"]
pub mod car_physics_2d;
pub use car_physics_2d::{car_distance_from_origin, car_heading_deg, car_kinetic_energy, Car2d};

#[path = "projectile_2d.rs"]
pub mod projectile_2d;
pub use projectile_2d::{
    max_height, optimal_launch_angle, range as projectile_range,
    simulate_trajectory as simulate_trajectory_2d, time_of_flight as time_of_flight_2d,
    Projectile2d,
};

#[path = "fluid_2d.rs"]
pub mod fluid_2d;
pub use fluid_2d::{divergence_at, grid_cell_count, FluidGrid2d};

#[path = "cloth_2d.rs"]
pub mod cloth_2d;
pub use cloth_2d::{
    apply_spring_2d, cloth_total_kinetic_energy, Cloth2d, ClothParticle2d, ClothSpring2d,
};

#[path = "soft_body_2d.rs"]
pub mod soft_body_2d;
pub use soft_body_2d::{make_soft_square_2d, SoftBody2d, SoftEdge2d, SoftNode2d};

#[path = "rope_2d.rs"]
pub mod rope_2d;
pub use rope_2d::{rope_segment_length, Rope2d, RopeNode2d};

#[path = "particle_system_2d.rs"]
pub mod particle_system_2d;
pub use particle_system_2d::{
    particle_count_alive, particles_above_ground, Particle2d, ParticleEmitter2d,
};

#[path = "explosion_2d.rs"]
pub mod explosion_2d;
pub use explosion_2d::{
    apply_explosion_to_bodies, explosion_energy, explosion_force, shockwave_radius, Explosion2d,
};

#[path = "buoyancy_2d.rs"]
pub mod buoyancy_2d;
pub use buoyancy_2d::{
    buoyancy_force_2d, equilibrium_depth as equilibrium_depth_2d, net_force_2d, step_buoyant_body,
    BuoyancyFluid2d, BuoyantBody2d,
};

#[path = "magnetic_2d.rs"]
pub mod magnetic_2d;
pub use magnetic_2d::{
    apply_magnetic_force, field_magnitude_2d, lorentz_force_2d, magnetic_field_2d, total_field_2d,
    MagneticSource2d,
};

#[path = "gravity_well_2d.rs"]
pub mod gravity_well_2d;
pub use gravity_well_2d::{
    apply_gravity_wells, gravity_force_2d, potential_energy_2d, GravityWell2d,
};

#[path = "portal_physics_2d.rs"]
pub mod portal_physics_2d;
pub use portal_physics_2d::{
    check_portal_crossing, point_near_portal, portal_pair, teleport_through, Portal2d,
};

#[path = "chaos_pendulum.rs"]
pub mod chaos_pendulum;
pub use chaos_pendulum::{
    new_double_pendulum, pendulum_bob2_pos, pendulum_kinetic_energy, pendulum_step,
    DoublePendulum as ChaosPendulum,
};

#[path = "lorenz_system.rs"]
pub mod lorenz_system;
pub use lorenz_system::{
    lorenz_divergence, lorenz_position, lorenz_step, new_lorenz_system, LorenzParams, LorenzSystem,
};

#[path = "duffing_oscillator.rs"]
pub mod duffing_oscillator;
pub use duffing_oscillator::{
    duffing_energy, duffing_position, duffing_step, duffing_velocity, new_duffing_oscillator,
    DuffingOscillator,
};

#[path = "van_der_pol_osc.rs"]
pub mod van_der_pol_osc;
pub use van_der_pol_osc::{
    new_van_der_pol_osc, vdp_energy, vdp_position, vdp_step, vdp_velocity, VanDerPolOsc,
};

#[path = "rossler_attractor.rs"]
pub mod rossler_attractor;
pub use rossler_attractor::{
    new_rossler, rossler_divergence, rossler_position, rossler_step, RosslerAttractor,
};

#[path = "logistic_map.rs"]
pub mod logistic_map;
pub use logistic_map::{
    lm_is_bounded, lm_iterate, lm_orbit, lm_step, lm_value, new_logistic_map, LogisticMap,
};

#[path = "henon_map.rs"]
pub mod henon_map;
pub use henon_map::{
    henon_iterate, henon_position, henon_step, henon_trajectory, new_henon_map, HenonMap,
};

#[path = "mandelbrot_orbit.rs"]
pub mod mandelbrot_orbit;
pub use mandelbrot_orbit::{
    mandelbrot_compute, mandelbrot_escape_iter, mandelbrot_escape_velocity, mandelbrot_in_set,
    MandelbrotOrbit,
};

#[path = "julia_orbit.rs"]
pub mod julia_orbit;
pub use julia_orbit::{
    julia_compute, julia_escape_iter, julia_escape_velocity, julia_in_set, JuliaOrbit,
};

#[path = "fractal_dimension.rs"]
pub mod fractal_dimension;
pub use fractal_dimension::{
    fd_box_count, fd_estimate, fd_point_count, new_fractal_dimension, FractalDimension,
};

#[path = "lyapunov_exponent.rs"]
pub mod lyapunov_exponent;
pub use lyapunov_exponent::{
    lyapunov_estimate, lyapunov_is_chaotic, new_lyapunov_estimator, LyapunovEstimator,
};

#[path = "bifurcation_map.rs"]
pub mod bifurcation_map;
pub use bifurcation_map::{
    bif_attractor_count, bif_point_count, bifurcation_compute, compute_bifurcation,
    BifurcationPoint,
};

#[path = "cellular_automaton_1d.rs"]
pub mod cellular_automaton_1d;
pub use cellular_automaton_1d::{
    ca1d_density, ca1d_iterate, ca1d_live_count, ca1d_step, new_ca1d, CellularAutomaton1D,
};

#[path = "reaction_diffusion.rs"]
pub mod reaction_diffusion;
pub use reaction_diffusion::{
    new_reaction_diffusion, rd_cell_count, rd_mean_u, rd_mean_v, rd_step, GrayScottParams,
    ReactionDiffusion,
};

#[path = "turing_pattern.rs"]
pub mod turing_pattern;
pub use turing_pattern::{
    new_turing_pattern, tp_mean_a, tp_mean_h, tp_step, tp_variance_a, TuringParams, TuringPattern,
};

#[path = "boids_simulation.rs"]
pub mod boids_simulation;
pub use boids_simulation::{
    boids_avg_speed, boids_center, boids_count, boids_step, new_boids_simulation, Boid,
    BoidsParams, BoidsSimulation,
};

#[path = "finite_element_3d.rs"]
pub mod finite_element_3d;
pub use finite_element_3d::{
    new_finite_element_3d, FiniteElement3D, Node3D, Tetrahedron as FemTetrahedron,
};

#[path = "isogeometric_analysis.rs"]
pub mod isogeometric_analysis;
pub use isogeometric_analysis::{bspline_basis, new_iga_patch, IgaPatch1D};

#[path = "boundary_element.rs"]
pub mod boundary_element;
pub use boundary_element::{
    bem_add_element, bem_add_node, bem_centroid, bem_element_count, bem_total_length,
    new_boundary_mesh, BoundaryElement, BoundaryMesh,
};

#[path = "meshfree_sph.rs"]
pub mod meshfree_sph;
pub use meshfree_sph::{
    cubic_kernel, new_meshfree_sph, MeshfreeSph, SphParticle as MeshfreeSphParticle,
};

#[path = "material_point_method.rs"]
pub mod material_point_method;
pub use material_point_method::{new_mpm, GridNode, MaterialPoint, MaterialPointMethod};

#[path = "peridynamics.rs"]
pub mod peridynamics;
pub use peridynamics::{new_peridynamics, Bond, PdPoint, Peridynamics};

#[path = "phase_field_fracture.rs"]
pub mod phase_field_fracture;
pub use phase_field_fracture::{new_phase_field_fracture, PhaseFieldFracture};

#[path = "discrete_element.rs"]
pub mod discrete_element;
pub use discrete_element::{new_discrete_element, DemParticle, DiscreteElement};

#[path = "lattice_spring_model.rs"]
pub mod lattice_spring_model;
pub use lattice_spring_model::{
    new_lattice_spring_model, LatticeBond, LatticeNode as LatticeSpringNode, LatticeSpringModel,
};

#[path = "fiber_network.rs"]
pub mod fiber_network;
pub use fiber_network::{new_fiber_network, Fiber, FiberNetwork};

#[path = "polymer_chain.rs"]
pub mod polymer_chain;
pub use polymer_chain::{new_polymer_chain, ChainSegment, PolymerChain};

#[path = "worm_like_chain.rs"]
pub mod worm_like_chain;
pub use worm_like_chain::{new_worm_like_chain, WormLikeChain};

#[path = "membrane_tension.rs"]
pub mod membrane_tension;
pub use membrane_tension::{new_membrane_tension, MemNode, MemSpring, MembraneTension};
