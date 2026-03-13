// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Physics simulation and collision proxy generation for OxiHuman meshes.
//!
//! This crate provides two distinct capabilities:
//!
//! **Collision proxies** — a set of [`CapsuleProxy`], [`SphereProxy`], and
//! [`BoxProxy`] primitives assembled into a [`BodyProxies`] set that
//! approximates the humanoid body for use in external physics engines (Rapier,
//! PhysX, Bullet, etc.). The [`sampling`] sub-module fits capsules via PCA on
//! surface-sampled point clouds for higher accuracy.
//!
//! **Simulation subsystems** — cloth ([`ClothSim`]), soft-body tetrahedra
//! ([`SoftBody`]), hair strands ([`HairSystem`]), aerodynamics ([`AeroConfig`]),
//! garment fitting ([`GarmentFitConfig`]), wind forces ([`WindConfig`]),
//! distance/bend/volume constraints ([`DistanceConstraint`] etc.), and a
//! contact solver ([`ContactSolverConfig`]).
//!
//! # Example: generate body proxies
//!
//! ```rust,no_run
//! use oxihuman_physics::BodyProxies;
//! // proxies are normally produced by oxihuman_physics::rig::build_rig
//! let proxies = BodyProxies::new();
//! println!("total proxy count: {}", proxies.total_count());
//! ```

pub mod oxirs_adapter;
pub use oxirs_adapter::{
    BodyHandle, BodyRigMapper, ColliderShape, ContactPair as OxiRsContactPair, OxiRsConfig,
    OxiRsWorld, RayCastHit, RigidBodyDef,
};

pub mod cloth;
pub mod joint_limits;
pub mod sampling;
pub mod self_intersection;
pub mod soft_body_v2;
pub use cloth::{ClothParticle, ClothSim, Spring, SpringKind};
pub mod material;
pub use material::{ClothMaterial, ClothStack};
pub mod hair;
pub use hair::{HairConfig, HairStrand, HairSystem};
pub mod collision;
pub use collision::{
    aabb_aabb, aabb_plane, capsule_capsule, capsule_plane, capsule_sphere, closest_point_on_aabb,
    closest_point_on_segment, resolve_contact, sphere_aabb, sphere_plane, sphere_sphere,
    sq_dist_point_segment, Capsule, CollisionAabb, CollisionPlane, Contact, Sphere,
};
pub mod rig;
pub use rig::{build_rig, CapsuleChain, PhysicsRig, RigJoint};

pub mod constraint;
pub use constraint::{
    apply_bend_constraint, apply_distance_constraint, apply_volume_constraint, constraint_energy,
    tet_volume, BendConstraint, ConstraintKind, DistanceConstraint, VolumeConstraint,
};
pub mod contact_solver;
pub use contact_solver::{
    contact_energy, detect_sphere_contacts, resolve_contacts, Contact as SolverContact,
    ContactSolverConfig,
};
pub mod soft_body;
pub use soft_body::{build_tet_edges, make_cube_soft_body, SoftBody, Tetrahedron};
pub mod aerodynamics;
pub mod garment_fit;
pub mod garment_fit_v2;
pub mod sdf_gen;
pub mod self_collision;
pub mod wind_force;
pub use aerodynamics::{
    apply_aero_to_particles, compute_aero_force, drag_force_magnitude, reynolds_number,
    stokes_drag, terminal_velocity, AeroConfig, AeroForce,
};
pub use garment_fit::{
    fit_garment_to_proxies, garment_clearance_stats, point_to_capsule_sdf, point_to_sphere_sdf,
    push_out_of_capsule, spring_pull, GarmentFitConfig, GarmentFitResult, GarmentVertex,
};
pub use wind_force::{
    apply_wind_to_cloth, dynamic_pressure, value_noise_3d, wind_force_on_face, wind_speed_beaufort,
    WindConfig, WindField, WindSample,
};

pub mod proxy_types;
pub use proxy_types::{
    proxies_from_json, proxies_to_json, BodyProxies, BoxProxy, CapsuleProxy, SphereProxy,
};

pub mod proxy_gen;
pub use proxy_gen::{
    generate_fitted_proxies, generate_proxies, proxies_from_measurements, voxelize_to_proxies,
    BODY_PART_BANDS,
};

pub mod proxy_tests;

pub mod modules_a;
pub use modules_a::*;

pub mod modules_b;
pub use modules_b::*;

pub mod modules_c;
pub use modules_c::*;

pub mod modules_d;
pub use modules_d::*;

pub mod modules_e;
pub use modules_e::*;

pub mod modules_f;
pub use modules_f::*;

pub mod modules_g;
pub use modules_g::*;
