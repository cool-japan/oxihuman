// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

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

use oxihuman_mesh::measurements::{compute_aabb, compute_measurements, Aabb, BodyMeasurements};
use oxihuman_mesh::mesh::MeshBuffers;
use sampling::Capsule as SamplingCapsule;

/// A capsule collision primitive (line segment + radius).
#[derive(Debug, Clone, PartialEq)]
pub struct CapsuleProxy {
    /// Bottom center of the capsule.
    pub center_a: [f32; 3],
    /// Top center of the capsule.
    pub center_b: [f32; 3],
    /// Radius of the capsule.
    pub radius: f32,
    /// Label (e.g. "torso", "head", "arm_l").
    pub label: String,
}

impl CapsuleProxy {
    pub fn new(center_a: [f32; 3], center_b: [f32; 3], radius: f32, label: &str) -> Self {
        CapsuleProxy {
            center_a,
            center_b,
            radius,
            label: label.to_string(),
        }
    }
}

/// A sphere collision primitive.
#[derive(Debug, Clone, PartialEq)]
pub struct SphereProxy {
    pub center: [f32; 3],
    pub radius: f32,
    pub label: String,
}

impl SphereProxy {
    pub fn new(center: [f32; 3], radius: f32, label: &str) -> Self {
        SphereProxy {
            center,
            radius,
            label: label.to_string(),
        }
    }
}

/// A box (AABB) collision primitive.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxProxy {
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
    pub label: String,
}

/// Complete set of collision proxies for a humanoid body.
#[derive(Debug, Default, Clone)]
pub struct BodyProxies {
    pub capsules: Vec<CapsuleProxy>,
    pub spheres: Vec<SphereProxy>,
    pub boxes: Vec<BoxProxy>,
}

impl BodyProxies {
    pub fn new() -> Self {
        BodyProxies::default()
    }

    pub fn total_count(&self) -> usize {
        self.capsules.len() + self.spheres.len() + self.boxes.len()
    }
}

// ── JSON serialization ────────────────────────────────────────────────────────

/// Serialize a `[f32; 3]` to a compact JSON array string.
fn fmt_vec3(v: [f32; 3]) -> String {
    format!("[{},{},{}]", v[0], v[1], v[2])
}

/// Escape a string for safe embedding in JSON (handles `"` and `\`).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

/// Serialize [`BodyProxies`] to a JSON string.
///
/// Output format:
/// ```json
/// {
///   "capsules": [...],
///   "spheres": [...],
///   "boxes": [...]
/// }
/// ```
pub fn proxies_to_json(proxies: &BodyProxies) -> String {
    let mut out = String::with_capacity(512);
    out.push_str("{\n  \"capsules\": [\n");

    for (i, c) in proxies.capsules.iter().enumerate() {
        let comma = if i + 1 < proxies.capsules.len() {
            ","
        } else {
            ""
        };
        out.push_str(&format!(
            "    {{\"label\":\"{}\",\"center_a\":{},\"center_b\":{},\"radius\":{}}}{}\n",
            json_escape(&c.label),
            fmt_vec3(c.center_a),
            fmt_vec3(c.center_b),
            c.radius,
            comma
        ));
    }

    out.push_str("  ],\n  \"spheres\": [\n");

    for (i, s) in proxies.spheres.iter().enumerate() {
        let comma = if i + 1 < proxies.spheres.len() {
            ","
        } else {
            ""
        };
        out.push_str(&format!(
            "    {{\"label\":\"{}\",\"center\":{},\"radius\":{}}}{}\n",
            json_escape(&s.label),
            fmt_vec3(s.center),
            s.radius,
            comma
        ));
    }

    out.push_str("  ],\n  \"boxes\": [\n");

    for (i, b) in proxies.boxes.iter().enumerate() {
        let comma = if i + 1 < proxies.boxes.len() { "," } else { "" };
        out.push_str(&format!(
            "    {{\"label\":\"{}\",\"center\":{},\"half_extents\":{}}}{}\n",
            json_escape(&b.label),
            fmt_vec3(b.center),
            fmt_vec3(b.half_extents),
            comma
        ));
    }

    out.push_str("  ]\n}");
    out
}

/// Deserialize [`BodyProxies`] from a JSON string produced by [`proxies_to_json`].
///
/// Uses `serde_json` for reliable parsing.
pub fn proxies_from_json(s: &str) -> anyhow::Result<BodyProxies> {
    let v: serde_json::Value = serde_json::from_str(s)?;

    let parse_vec3 = |arr: &serde_json::Value| -> anyhow::Result<[f32; 3]> {
        let a = arr
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("expected array for vec3"))?;
        if a.len() != 3 {
            anyhow::bail!("vec3 must have 3 elements, got {}", a.len());
        }
        Ok([
            a[0].as_f64()
                .ok_or_else(|| anyhow::anyhow!("expected float"))? as f32,
            a[1].as_f64()
                .ok_or_else(|| anyhow::anyhow!("expected float"))? as f32,
            a[2].as_f64()
                .ok_or_else(|| anyhow::anyhow!("expected float"))? as f32,
        ])
    };

    let get_str = |obj: &serde_json::Value, key: &str| -> anyhow::Result<String> {
        obj[key]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("missing string field '{key}'"))
    };

    let get_f32 = |obj: &serde_json::Value, key: &str| -> anyhow::Result<f32> {
        obj[key]
            .as_f64()
            .map(|f| f as f32)
            .ok_or_else(|| anyhow::anyhow!("missing float field '{key}'"))
    };

    let mut proxies = BodyProxies::new();

    if let Some(caps) = v["capsules"].as_array() {
        for c in caps {
            proxies.capsules.push(CapsuleProxy {
                label: get_str(c, "label")?,
                center_a: parse_vec3(&c["center_a"])?,
                center_b: parse_vec3(&c["center_b"])?,
                radius: get_f32(c, "radius")?,
            });
        }
    }

    if let Some(spheres) = v["spheres"].as_array() {
        for s in spheres {
            proxies.spheres.push(SphereProxy {
                label: get_str(s, "label")?,
                center: parse_vec3(&s["center"])?,
                radius: get_f32(s, "radius")?,
            });
        }
    }

    if let Some(boxes) = v["boxes"].as_array() {
        for b in boxes {
            proxies.boxes.push(BoxProxy {
                label: get_str(b, "label")?,
                center: parse_vec3(&b["center"])?,
                half_extents: parse_vec3(&b["half_extents"])?,
            });
        }
    }

    Ok(proxies)
}

// ── Voxelization-based proxy generator ───────────────────────────────────────

/// Generate box proxies by voxelizing the mesh into a `grid_resolution^3` grid.
///
/// Each voxel that contains at least one mesh vertex is marked occupied.
/// Connected occupied voxels in the same body-part band are merged into one
/// [`BoxProxy`] whose label is determined by the band matching the voxel
/// center's Y-fraction within the mesh's total height.
pub fn voxelize_to_proxies(mesh: &MeshBuffers, grid_resolution: u32) -> BodyProxies {
    let mut proxies = BodyProxies::new();

    if mesh.positions.is_empty() || grid_resolution == 0 {
        return proxies;
    }

    let res = grid_resolution as usize;

    // ── Compute AABB of the mesh ──────────────────────────────────────────────
    let (mut xmin, mut xmax) = (f32::INFINITY, f32::NEG_INFINITY);
    let (mut ymin, mut ymax) = (f32::INFINITY, f32::NEG_INFINITY);
    let (mut zmin, mut zmax) = (f32::INFINITY, f32::NEG_INFINITY);
    for p in &mesh.positions {
        xmin = xmin.min(p[0]);
        xmax = xmax.max(p[0]);
        ymin = ymin.min(p[1]);
        ymax = ymax.max(p[1]);
        zmin = zmin.min(p[2]);
        zmax = zmax.max(p[2]);
    }

    let total_height = ymax - ymin;
    let cx = (xmin + xmax) * 0.5;

    // Guard against degenerate meshes.
    let extent_x = (xmax - xmin).max(1e-6);
    let extent_y = (ymax - ymin).max(1e-6);
    let extent_z = (zmax - zmin).max(1e-6);

    let cell_x = extent_x / res as f32;
    let cell_y = extent_y / res as f32;
    let cell_z = extent_z / res as f32;

    // ── Mark occupied voxels ──────────────────────────────────────────────────
    // Grid flattened as [xi * res * res + yi * res + zi].
    let total_cells = res * res * res;
    let mut occupied = vec![false; total_cells];

    for p in &mesh.positions {
        let xi = (((p[0] - xmin) / extent_x) * res as f32)
            .floor()
            .clamp(0.0, (res - 1) as f32) as usize;
        let yi = (((p[1] - ymin) / extent_y) * res as f32)
            .floor()
            .clamp(0.0, (res - 1) as f32) as usize;
        let zi = (((p[2] - zmin) / extent_z) * res as f32)
            .floor()
            .clamp(0.0, (res - 1) as f32) as usize;
        occupied[xi * res * res + yi * res + zi] = true;
    }

    // ── Assign labels to occupied voxels via BODY_PART_BANDS ─────────────────
    // For each voxel centre, pick the first matching band.
    let label_of = |xi: usize, yi: usize| -> &'static str {
        let y_frac = if total_height > 1e-6 {
            (ymin + (yi as f32 + 0.5) * cell_y - ymin) / total_height
        } else {
            0.5
        };
        let vox_x = xmin + (xi as f32 + 0.5) * cell_x;

        for band in BODY_PART_BANDS {
            if y_frac < band.y_lo || y_frac > band.y_hi {
                continue;
            }
            match band.x_sign {
                None => return band.name,
                Some(sign) => {
                    if sign < 0.0 && vox_x <= cx {
                        return band.name;
                    }
                    if sign > 0.0 && vox_x >= cx {
                        return band.name;
                    }
                }
            }
        }
        "body"
    };

    // ── Flood-fill: group connected occupied voxels with the same label ───────
    let mut region_id = vec![usize::MAX; total_cells];
    let mut regions: Vec<(String, Vec<usize>)> = Vec::new(); // (label, cell indices)

    let idx = |xi: usize, yi: usize, zi: usize| xi * res * res + yi * res + zi;

    for xi in 0..res {
        for yi in 0..res {
            for zi in 0..res {
                let cell = idx(xi, yi, zi);
                if !occupied[cell] || region_id[cell] != usize::MAX {
                    continue;
                }
                // BFS flood fill
                let label = label_of(xi, yi);
                let rid = regions.len();
                regions.push((label.to_string(), Vec::new()));

                let mut queue = std::collections::VecDeque::new();
                queue.push_back((xi, yi, zi));
                region_id[cell] = rid;

                while let Some((cx2, cy, cz)) = queue.pop_front() {
                    regions[rid].1.push(idx(cx2, cy, cz));

                    // 6-connected neighbours
                    let neighbours = [
                        (cx2.wrapping_sub(1), cy, cz),
                        (cx2 + 1, cy, cz),
                        (cx2, cy.wrapping_sub(1), cz),
                        (cx2, cy + 1, cz),
                        (cx2, cy, cz.wrapping_sub(1)),
                        (cx2, cy, cz + 1),
                    ];
                    for (nx, ny, nz) in neighbours {
                        if nx >= res || ny >= res || nz >= res {
                            continue;
                        }
                        let ncell = idx(nx, ny, nz);
                        if occupied[ncell]
                            && region_id[ncell] == usize::MAX
                            && label_of(nx, ny) == label
                        {
                            region_id[ncell] = rid;
                            queue.push_back((nx, ny, nz));
                        }
                    }
                }
            }
        }
    }

    // ── Convert each region to a BoxProxy ─────────────────────────────────────
    for (label, cells) in &regions {
        if cells.is_empty() {
            continue;
        }
        let mut bx_min = f32::INFINITY;
        let mut bx_max = f32::NEG_INFINITY;
        let mut by_min = f32::INFINITY;
        let mut by_max = f32::NEG_INFINITY;
        let mut bz_min = f32::INFINITY;
        let mut bz_max = f32::NEG_INFINITY;

        for &cell in cells {
            let zi = cell % res;
            let yi = (cell / res) % res;
            let xi = cell / (res * res);

            let vx0 = xmin + xi as f32 * cell_x;
            let vy0 = ymin + yi as f32 * cell_y;
            let vz0 = zmin + zi as f32 * cell_z;

            bx_min = bx_min.min(vx0);
            bx_max = bx_max.max(vx0 + cell_x);
            by_min = by_min.min(vy0);
            by_max = by_max.max(vy0 + cell_y);
            bz_min = bz_min.min(vz0);
            bz_max = bz_max.max(vz0 + cell_z);
        }

        let center = [
            (bx_min + bx_max) * 0.5,
            (by_min + by_max) * 0.5,
            (bz_min + bz_max) * 0.5,
        ];
        let half_extents = [
            ((bx_max - bx_min) * 0.5).max(1e-6),
            ((by_max - by_min) * 0.5).max(1e-6),
            ((bz_max - bz_min) * 0.5).max(1e-6),
        ];

        proxies.boxes.push(BoxProxy {
            center,
            half_extents,
            label: label.clone(),
        });
    }

    proxies
}

/// Generate body collision proxies from a `MeshBuffers`.
///
/// Returns `None` if the mesh is empty or measurements can't be computed.
pub fn generate_proxies(mesh: &MeshBuffers) -> Option<BodyProxies> {
    let aabb = compute_aabb(mesh)?;
    let meas = compute_measurements(mesh)?;
    Some(proxies_from_measurements(&meas, &aabb))
}

/// Generate proxies from pre-computed measurements and bounding box.
///
/// Produces 10 capsules + 1 sphere = 11 total primitives:
/// torso, hips, leg_l, leg_r, shin_l, shin_r, arm_l, arm_r, forearm_l, forearm_r,
/// and a head sphere.
pub fn proxies_from_measurements(meas: &BodyMeasurements, aabb: &Aabb) -> BodyProxies {
    let h = meas.total_height;
    let y0 = aabb.min[1];
    let cx = aabb.center()[0];
    let cz = aabb.center()[2];

    let mut proxies = BodyProxies::new();

    // ── Head (sphere) ─────────────────────────────────────────────────────────
    let head_y = y0 + h * 0.915;
    let head_r = h * 0.075;
    proxies
        .spheres
        .push(SphereProxy::new([cx, head_y, cz], head_r, "head"));

    // ── Torso (main capsule, extends to shoulder level covering neck) ─────────
    let torso_r = meas.shoulder_width.max(meas.waist_width) * 0.5 * 0.6;
    proxies.capsules.push(CapsuleProxy::new(
        [cx, y0 + h * 0.52, cz],
        [cx, y0 + h * 0.84, cz],
        torso_r.max(h * 0.06),
        "torso",
    ));

    // ── Hips (short wide capsule) ─────────────────────────────────────────────
    let hip_r = meas.hip_width * 0.5 * 0.65;
    proxies.capsules.push(CapsuleProxy::new(
        [cx, y0 + h * 0.44, cz],
        [cx, y0 + h * 0.54, cz],
        hip_r.max(h * 0.07),
        "hips",
    ));

    // ── Upper legs ────────────────────────────────────────────────────────────
    let leg_sep = meas.hip_width * 0.25;
    let leg_r = h * 0.040;
    for &(side, sign) in &[("leg_l", -1.0f32), ("leg_r", 1.0f32)] {
        proxies.capsules.push(CapsuleProxy::new(
            [cx + sign * leg_sep, y0 + h * 0.26, cz],
            [cx + sign * leg_sep, y0 + h * 0.44, cz],
            leg_r,
            side,
        ));
    }

    // ── Lower legs ────────────────────────────────────────────────────────────
    for &(side, sign) in &[("shin_l", -1.0f32), ("shin_r", 1.0f32)] {
        proxies.capsules.push(CapsuleProxy::new(
            [cx + sign * leg_sep, y0 + h * 0.05, cz],
            [cx + sign * leg_sep, y0 + h * 0.26, cz],
            leg_r * 0.85,
            side,
        ));
    }

    // ── Upper arms ───────────────────────────────────────────────────────────
    let shoulder_sep = meas.shoulder_width * 0.5;
    let arm_r = h * 0.028;
    for &(side, sign) in &[("arm_l", -1.0f32), ("arm_r", 1.0f32)] {
        proxies.capsules.push(CapsuleProxy::new(
            [cx + sign * (shoulder_sep * 0.7), y0 + h * 0.67, cz],
            [cx + sign * (shoulder_sep + h * 0.08), y0 + h * 0.73, cz],
            arm_r,
            side,
        ));
    }

    // ── Forearms ─────────────────────────────────────────────────────────────
    for &(side, sign) in &[("forearm_l", -1.0f32), ("forearm_r", 1.0f32)] {
        proxies.capsules.push(CapsuleProxy::new(
            [cx + sign * (shoulder_sep + h * 0.08), y0 + h * 0.73, cz],
            [cx + sign * (shoulder_sep + h * 0.19), y0 + h * 0.54, cz],
            arm_r * 0.8,
            side,
        ));
    }

    proxies
}

// ── Body-part vertex-group definitions ────────────────────────────────────────

/// Body part band: (name, y_fraction_low, y_fraction_high, optional x_sign for limbs).
/// x_sign = Some(-1.0) → left side (x < center_x), Some(1.0) → right side, None → full width.
struct BodyPartBand {
    name: &'static str,
    y_lo: f32,
    y_hi: f32,
    x_sign: Option<f32>,
}

const BODY_PART_BANDS: &[BodyPartBand] = &[
    BodyPartBand {
        name: "head",
        y_lo: 0.86,
        y_hi: 1.00,
        x_sign: None,
    },
    BodyPartBand {
        name: "torso",
        y_lo: 0.52,
        y_hi: 0.86,
        x_sign: None,
    },
    BodyPartBand {
        name: "hips",
        y_lo: 0.42,
        y_hi: 0.54,
        x_sign: None,
    },
    BodyPartBand {
        name: "leg_l",
        y_lo: 0.26,
        y_hi: 0.44,
        x_sign: Some(-1.0),
    },
    BodyPartBand {
        name: "leg_r",
        y_lo: 0.26,
        y_hi: 0.44,
        x_sign: Some(1.0),
    },
    BodyPartBand {
        name: "shin_l",
        y_lo: 0.05,
        y_hi: 0.26,
        x_sign: Some(-1.0),
    },
    BodyPartBand {
        name: "shin_r",
        y_lo: 0.05,
        y_hi: 0.26,
        x_sign: Some(1.0),
    },
    BodyPartBand {
        name: "arm_l",
        y_lo: 0.64,
        y_hi: 0.78,
        x_sign: Some(-1.0),
    },
    BodyPartBand {
        name: "arm_r",
        y_lo: 0.64,
        y_hi: 0.78,
        x_sign: Some(1.0),
    },
    BodyPartBand {
        name: "forearm_l",
        y_lo: 0.50,
        y_hi: 0.66,
        x_sign: Some(-1.0),
    },
    BodyPartBand {
        name: "forearm_r",
        y_lo: 0.50,
        y_hi: 0.66,
        x_sign: Some(1.0),
    },
];

/// Collect vertices that belong to a body-part band (by height fraction + optional X-side).
fn collect_band_vertices(
    mesh: &MeshBuffers,
    y_min: f32,
    total_height: f32,
    cx: f32,
    band: &BodyPartBand,
) -> Vec<[f32; 3]> {
    let y_lo = y_min + total_height * band.y_lo;
    let y_hi = y_min + total_height * band.y_hi;

    mesh.positions
        .iter()
        .filter(|p| {
            p[1] >= y_lo
                && p[1] <= y_hi
                && match band.x_sign {
                    None => true,
                    Some(sign) => {
                        if sign < 0.0 {
                            p[0] <= cx
                        } else {
                            p[0] >= cx
                        }
                    }
                }
        })
        .copied()
        .collect()
}

/// Generate surface-sampling based fitted capsules for each body part.
///
/// Uses PCA-based [`sampling::fit_capsule`] instead of AABB bounding boxes
/// for more accurate proxy fitting.
///
/// Returns a `Vec` of `(body_part_name, Capsule)` pairs — one entry per body part.
pub fn generate_fitted_proxies(
    mesh: &MeshBuffers,
    measurements: &BodyMeasurements,
) -> Vec<(String, SamplingCapsule)> {
    let total_height = measurements.total_height;
    if total_height < 1e-6 || mesh.positions.is_empty() {
        return Vec::new();
    }

    // Find y_min from the mesh itself
    let y_min = mesh
        .positions
        .iter()
        .map(|p| p[1])
        .fold(f32::INFINITY, f32::min);

    // Find center x from measurements/AABB
    let cx = mesh
        .positions
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), p| {
            (lo.min(p[0]), hi.max(p[0]))
        });
    let cx = (cx.0 + cx.1) * 0.5;

    let mut result = Vec::with_capacity(BODY_PART_BANDS.len());

    for band in BODY_PART_BANDS {
        let verts = collect_band_vertices(mesh, y_min, total_height, cx, band);

        let capsule = if verts.is_empty() {
            // Fallback: place a minimal capsule at the band's Y midpoint
            let y_mid = y_min + total_height * (band.y_lo + band.y_hi) * 0.5;
            SamplingCapsule {
                p0: [cx, y_mid, 0.0],
                p1: [cx, y_mid, 0.0],
                radius: 0.001,
            }
        } else {
            sampling::fit_capsule(&verts)
        };

        result.push((band.name.to_string(), capsule));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn unit_body_mesh() -> MeshBuffers {
        // Minimal mesh spanning [0..0.5, 0..1.8, 0..0.3]
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [0.5, 0.0, 0.0],
            [0.5, 0.0, 0.3],
            [0.0, 0.0, 0.3],
            [0.0, 1.8, 0.0],
            [0.5, 1.8, 0.0],
            [0.5, 1.8, 0.3],
            [0.0, 1.8, 0.3],
        ];
        MeshBuffers::from_morph(MB {
            positions,
            normals: vec![[0.0, 1.0, 0.0]; 8],
            uvs: vec![[0.0, 0.0]; 8],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    #[test]
    fn generate_proxies_produces_expected_count() {
        let mesh = unit_body_mesh();
        let proxies = generate_proxies(&mesh).unwrap();
        // Should have: head sphere + torso + hips + 2 legs + 2 shins + 2 arms + 2 forearms
        // = 1 sphere + 10 capsules = 11 total
        assert_eq!(proxies.total_count(), 11);
        assert_eq!(proxies.spheres.len(), 1);
        assert_eq!(proxies.capsules.len(), 10);
        assert_eq!(proxies.spheres[0].label, "head");
    }

    #[test]
    fn proxy_radii_positive() {
        let mesh = unit_body_mesh();
        let proxies = generate_proxies(&mesh).unwrap();
        for c in &proxies.capsules {
            assert!(
                c.radius > 0.0,
                "capsule {} has non-positive radius",
                c.label
            );
        }
        for s in &proxies.spheres {
            assert!(s.radius > 0.0, "sphere {} has non-positive radius", s.label);
        }
    }

    #[test]
    fn proxy_positions_within_mesh_height() {
        let mesh = unit_body_mesh();
        let aabb = compute_aabb(&mesh).unwrap();
        let proxies = generate_proxies(&mesh).unwrap();
        for c in &proxies.capsules {
            assert!(c.center_a[1] >= aabb.min[1], "{} below floor", c.label);
            assert!(
                c.center_b[1] <= aabb.max[1] + c.radius,
                "{} above ceiling",
                c.label
            );
        }
        for s in &proxies.spheres {
            assert!(s.center[1] >= aabb.min[1], "sphere {} below floor", s.label);
            assert!(
                s.center[1] <= aabb.max[1] + s.radius,
                "sphere {} above ceiling",
                s.label
            );
        }
    }

    #[test]
    fn real_base_mesh_proxies() {
        use oxihuman_core::parser::obj::parse_obj;
        let path = "/media/kitasan/Backup/resource/makehuman/makehuman/data/3dobjs/base.obj";
        if let Ok(src) = std::fs::read_to_string(path) {
            if let Ok(obj) = parse_obj(&src) {
                let morph_buf = oxihuman_morph::engine::MeshBuffers {
                    positions: obj.positions,
                    normals: obj.normals,
                    uvs: obj.uvs,
                    indices: obj.indices,
                    has_suit: false,
                };
                let mesh = MeshBuffers::from_morph(morph_buf);
                let proxies = generate_proxies(&mesh).unwrap();
                assert_eq!(proxies.total_count(), 11);
                // Head should be near top of mesh
                let head = &proxies.spheres[0];
                let aabb = compute_aabb(&mesh).unwrap();
                assert!(
                    head.center[1] > aabb.center()[1],
                    "head should be above center"
                );
            }
        }
    }

    // ── generate_fitted_proxies tests ─────────────────────────────────────────

    /// A denser body-shaped mesh for fitted proxy tests.
    fn body_mesh_dense() -> MeshBuffers {
        // Build a set of positions that cover the full human height range
        // so each band has some vertices.
        let mut positions = Vec::new();
        // Distribute 200 points evenly across y=0..1.8, with some x/z spread
        for i in 0..200usize {
            let t = i as f32 / 199.0;
            let y = t * 1.8;
            let angle = t * std::f32::consts::TAU * 10.0;
            positions.push([angle.cos() * 0.15, y, angle.sin() * 0.10]);
            positions.push([-angle.cos() * 0.15, y, angle.sin() * 0.10]);
        }
        MeshBuffers::from_morph(MB {
            normals: vec![[0.0, 1.0, 0.0]; positions.len()],
            uvs: vec![[0.0, 0.0]; positions.len()],
            indices: vec![0, 1, 2],
            has_suit: false,
            positions,
        })
    }

    #[test]
    fn generate_fitted_proxies_count() {
        let mesh = body_mesh_dense();
        let meas = compute_measurements(&mesh).unwrap();
        let proxies = generate_fitted_proxies(&mesh, &meas);
        // One entry per body part band
        assert_eq!(
            proxies.len(),
            BODY_PART_BANDS.len(),
            "expected {} fitted proxies, got {}",
            BODY_PART_BANDS.len(),
            proxies.len()
        );
    }

    #[test]
    fn generate_fitted_proxies_names() {
        let mesh = body_mesh_dense();
        let meas = compute_measurements(&mesh).unwrap();
        let proxies = generate_fitted_proxies(&mesh, &meas);
        let names: Vec<&str> = proxies.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"head"), "missing head");
        assert!(names.contains(&"torso"), "missing torso");
        assert!(names.contains(&"arm_l"), "missing arm_l");
        assert!(names.contains(&"forearm_r"), "missing forearm_r");
    }

    #[test]
    fn generate_fitted_proxies_radii_positive() {
        let mesh = body_mesh_dense();
        let meas = compute_measurements(&mesh).unwrap();
        let proxies = generate_fitted_proxies(&mesh, &meas);
        for (name, cap) in &proxies {
            assert!(
                cap.radius > 0.0,
                "fitted proxy '{name}' has non-positive radius {}",
                cap.radius
            );
        }
    }

    #[test]
    fn generate_fitted_proxies_count_real_mesh() {
        use oxihuman_core::parser::obj::parse_obj;
        let path = "/media/kitasan/Backup/resource/makehuman/makehuman/data/3dobjs/base.obj";
        if let Ok(src) = std::fs::read_to_string(path) {
            if let Ok(obj) = parse_obj(&src) {
                let morph_buf = MB {
                    positions: obj.positions,
                    normals: obj.normals,
                    uvs: obj.uvs,
                    indices: obj.indices,
                    has_suit: false,
                };
                let mesh = MeshBuffers::from_morph(morph_buf);
                let meas = compute_measurements(&mesh).unwrap();
                let proxies = generate_fitted_proxies(&mesh, &meas);
                assert_eq!(
                    proxies.len(),
                    BODY_PART_BANDS.len(),
                    "real mesh proxy count should match body part band count"
                );
            }
        }
    }

    // ── Task 1: JSON serialization tests ─────────────────────────────────────

    fn sample_proxies() -> BodyProxies {
        let mut p = BodyProxies::new();
        p.capsules.push(CapsuleProxy::new(
            [0.0, 0.52, 0.0],
            [0.0, 0.84, 0.0],
            0.12,
            "torso",
        ));
        p.spheres
            .push(SphereProxy::new([0.0, 1.65, 0.0], 0.11, "head"));
        p
    }

    #[test]
    fn proxies_to_json_is_valid_json() {
        let proxies = sample_proxies();
        let json = proxies_to_json(&proxies);
        // serde_json must be able to parse our hand-written output
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("proxies_to_json must produce valid JSON");
        assert!(parsed.is_object(), "top level must be an object");
    }

    #[test]
    fn proxies_to_json_contains_expected_keys() {
        let proxies = sample_proxies();
        let json = proxies_to_json(&proxies);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v["capsules"].is_array(), "must have capsules array");
        assert!(v["spheres"].is_array(), "must have spheres array");
        assert!(v["boxes"].is_array(), "must have boxes array");
    }

    #[test]
    fn proxies_to_json_capsule_label_present() {
        let proxies = sample_proxies();
        let json = proxies_to_json(&proxies);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let cap = &v["capsules"][0];
        assert_eq!(cap["label"].as_str(), Some("torso"));
    }

    #[test]
    fn proxies_to_json_sphere_label_present() {
        let proxies = sample_proxies();
        let json = proxies_to_json(&proxies);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let sph = &v["spheres"][0];
        assert_eq!(sph["label"].as_str(), Some("head"));
    }

    #[test]
    fn proxies_from_json_round_trip() {
        let original = sample_proxies();
        let json = proxies_to_json(&original);
        let restored = proxies_from_json(&json).expect("round-trip must succeed");

        assert_eq!(restored.capsules.len(), original.capsules.len());
        assert_eq!(restored.spheres.len(), original.spheres.len());
        assert_eq!(restored.boxes.len(), original.boxes.len());

        let orig_cap = &original.capsules[0];
        let rest_cap = &restored.capsules[0];
        assert_eq!(rest_cap.label, orig_cap.label);
        assert!((rest_cap.radius - orig_cap.radius).abs() < 1e-4);
        for i in 0..3 {
            assert!((rest_cap.center_a[i] - orig_cap.center_a[i]).abs() < 1e-4);
            assert!((rest_cap.center_b[i] - orig_cap.center_b[i]).abs() < 1e-4);
        }

        let orig_sph = &original.spheres[0];
        let rest_sph = &restored.spheres[0];
        assert_eq!(rest_sph.label, orig_sph.label);
        assert!((rest_sph.radius - orig_sph.radius).abs() < 1e-4);
    }

    #[test]
    fn proxies_from_json_invalid_input_errors() {
        assert!(
            proxies_from_json("not json at all").is_err(),
            "invalid JSON must return Err"
        );
    }

    #[test]
    fn proxies_to_json_empty_proxies() {
        let empty = BodyProxies::new();
        let json = proxies_to_json(&empty);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["capsules"].as_array().unwrap().len(), 0);
        assert_eq!(v["spheres"].as_array().unwrap().len(), 0);
        assert_eq!(v["boxes"].as_array().unwrap().len(), 0);
    }

    // ── Task 2: voxelization tests ────────────────────────────────────────────

    #[test]
    fn voxelize_to_proxies_nonempty_for_body_mesh() {
        let mesh = body_mesh_dense();
        let proxies = voxelize_to_proxies(&mesh, 8);
        assert!(
            !proxies.boxes.is_empty(),
            "voxelization must produce at least one box proxy"
        );
    }

    #[test]
    fn voxelize_to_proxies_labels_include_torso_or_head() {
        let mesh = body_mesh_dense();
        let proxies = voxelize_to_proxies(&mesh, 8);
        let labels: Vec<&str> = proxies.boxes.iter().map(|b| b.label.as_str()).collect();
        let has_torso_or_head = labels.contains(&"torso") || labels.contains(&"head");
        assert!(
            has_torso_or_head,
            "expected 'torso' or 'head' label, got: {:?}",
            labels
        );
    }

    #[test]
    fn voxelize_to_proxies_half_extents_positive() {
        let mesh = body_mesh_dense();
        let proxies = voxelize_to_proxies(&mesh, 6);
        for b in &proxies.boxes {
            assert!(
                b.half_extents[0] > 0.0 && b.half_extents[1] > 0.0 && b.half_extents[2] > 0.0,
                "box proxy '{}' has non-positive half_extents {:?}",
                b.label,
                b.half_extents
            );
        }
    }

    #[test]
    fn voxelize_to_proxies_empty_mesh_returns_empty() {
        let empty_mesh = MeshBuffers::from_morph(MB {
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            indices: vec![],
            has_suit: false,
        });
        let proxies = voxelize_to_proxies(&empty_mesh, 8);
        assert!(
            proxies.boxes.is_empty(),
            "empty mesh must produce no box proxies"
        );
    }

    #[test]
    fn voxelize_to_proxies_json_round_trip() {
        // Voxelize the body mesh, serialize to JSON, parse back, and verify counts match.
        let mesh = body_mesh_dense();
        let proxies = voxelize_to_proxies(&mesh, 6);
        let json = proxies_to_json(&proxies);
        let restored = proxies_from_json(&json).expect("voxel proxy JSON round-trip must succeed");
        assert_eq!(
            proxies.boxes.len(),
            restored.boxes.len(),
            "box count must survive JSON round-trip"
        );
    }
}

pub mod constraint_group;
pub mod pbd_solver;
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
pub mod fluid_height;
pub mod joint_motor;
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

pub mod fracture;
pub use fracture::{
    cell_centroid, fracture_mesh, generate_voronoi_seeds, merge_small_cells, voronoi_fracture,
    FractureConfig, VoronoiCell,
};
pub mod cloth_pattern;
pub use cloth_pattern::{
    build_circular_panel, build_rectangle_panel, drape_panel_onto_sphere,
    pattern_to_cloth_particles, wrap_panel_to_cylinder, EdgeKind, GarmentPattern,
    GarmentPatternConfig, PatternEdge, PatternVertex,
};

pub mod broadphase;
pub use broadphase::{
    aabb_expand, aabb_from_capsule, aabb_from_sphere, aabb_overlap, build_bvh,
    compute_all_pair_overlaps, query_aabb, query_ray, BvhAabb, BvhNode, BvhTree,
};

pub mod cloth_tear;
pub use cloth_tear::{
    apply_force_at, compute_stretch_ratio, count_broken_constraints, count_intact_constraints,
    find_overloaded_constraints, is_fully_torn, new_tearable_grid, step_tear_simulation,
    tear_at_point, tearable_mesh_stats, TearConstraint, TearableMesh,
};

pub mod particle_system;
pub use particle_system::{
    active_particle_count, count_expired_slots, emit_particle, lerp_particle_color,
    new_particle_system, particle_age_fraction, particle_system_bounds, reset_particle_system,
    set_emitter_position, step_particle_system, EmitterShape, Particle, ParticleEmitter,
    ParticleSystem,
};

pub mod rope_sim;
pub use rope_sim::{
    apply_impulse_to_rope, attach_rope_end, new_rope, pin_particle, rope_end_position, rope_energy,
    rope_length, rope_sag, rope_tension_at, rope_to_polyline, step_rope, unpin_particle, Rope,
    RopeConfig, RopeParticle, RopeSegment,
};

pub mod joint_constraint;
pub use joint_constraint::{
    add_ball_joint, add_fixed_joint, add_hinge_joint, apply_joint_impulse, break_joint,
    compute_chain_positions, constraint_energy as joint_constraint_energy, count_active_joints,
    joint_violation as joint_constraint_violation, new_constraint_solver, solve_constraints,
    ConstraintSolver, JointConstraint, JointType,
};

pub mod spring_network;
pub use spring_network::{
    add_node, add_spring, apply_impulse, build_grid_network, clamp_velocities, count_pinned,
    network_bounding_box, network_energy, new_network, spring_extension, spring_force,
    step_network, Spring as SpringNetNode, SpringNetwork, SpringNetworkConfig, SpringNode,
};

pub mod buoyancy;
pub use buoyancy::{
    archimedes_force, compute_buoyancy_force, compute_wave_force, drag_force, equilibrium_depth,
    is_floating, multi_body_buoyancy, step_body, submerged_fraction,
    terminal_velocity as buoyancy_terminal_velocity, BuoyancyConfig, BuoyancyResult, SubmergedBody,
};

pub mod kinematic_body;
pub use kinematic_body::{
    aabb_of_body, add_kinematic_body, bodies_overlap, enabled_body_count, get_body,
    kinematic_body_count, move_body, new_kinematic_world, remove_body, set_body_rotation,
    set_layer_mask, sphere_sphere_contact, KinematicBody, KinematicContact, KinematicShape,
    KinematicWorld,
};

pub mod trigger_zone;
pub use trigger_zone::{
    add_aabb_trigger, add_capsule_trigger, add_sphere_trigger, enabled_trigger_count,
    get_trigger as get_trigger_zone, new_trigger_world, point_in_aabb, point_in_capsule,
    point_in_sphere, point_in_trigger, query_triggers, remove_trigger, trigger_zone_volume,
    TriggerEvent, TriggerShape, TriggerWorld, TriggerZone,
};

pub mod debris_system;
pub use debris_system::{
    apply_wind_to_debris, debris_bounding_box, default_debris_config, fragment_kinetic_energy,
    living_fragment_count, new_debris_system, remove_dead, set_floor, spawn_explosion,
    spawn_fragment, step_debris, total_kinetic_energy as debris_total_ke, DebrisConfig,
    DebrisFragment, DebrisSystem,
};

pub mod fluid_grid;
pub use fluid_grid::{
    add_density, add_velocity, advect_density, cell_index, default_fluid_config, diffuse_density,
    fluid_grid_stats, get_cell, get_cell_mut, max_velocity as fluid_max_velocity, new_fluid_grid,
    set_obstacle, step_fluid, total_density, FluidCell, FluidConfig, FluidGrid as EulerFluidGrid,
};

pub mod impulse_solver;
pub use impulse_solver::{
    add_impulse_body, apply_impulse_to_body, compute_impulse_magnitude, impulse_body_by_id,
    impulse_body_count, integrate_impulse_bodies, new_impulse_solver, relative_velocity_at_contact,
    remove_impulse_body, resolve_impulse_contact, separate_impulse_bodies,
    sphere_sphere_impulse_contact, total_impulse_kinetic_energy, ImpulseBody, ImpulseContact,
    ImpulseSolver,
};

pub mod contact_material;
pub use contact_material::{
    add_contact_pair, all_physics_materials, combine_friction, combine_restitution,
    contact_pair_count, default_contact_props, default_material_table, lookup_contact,
    material_density, material_friction, material_restitution, new_contact_table,
    physics_material_name, ContactMaterialTable, ContactProps, PhysicsMaterial,
};

pub mod granular_sim;
pub use granular_sim::{
    active_grain_count, add_grain, default_granular_config, grain_count, grain_pile_height,
    grains_overlap, new_granular_world, pour_grains, remove_grain, resolve_grain_floor,
    resolve_grain_grain, simulate_granular, total_granular_energy, Grain, GranularConfig,
    GranularWorld,
};

pub mod ragdoll;
pub use ragdoll::{
    activate_ragdoll, add_ragdoll_bone, add_ragdoll_joint, apply_impulse_to_bone,
    deactivate_ragdoll, default_humanoid_ragdoll, get_ragdoll_bone, new_ragdoll,
    ragdoll_bone_count, ragdoll_center_of_mass, ragdoll_joint_count, ragdoll_total_mass,
    simulate_ragdoll, Ragdoll, RagdollBone, RagdollJoint,
};

pub mod xpbd_solver;
pub use xpbd_solver::{
    add_bend_constraint as xpbd_add_bend, add_distance_constraint as xpbd_add_distance,
    add_xpbd_particle, new_xpbd_world, pin_particle as xpbd_pin_particle, predict_positions,
    reset_lambdas, solve_distance_constraint, unpin_particle as xpbd_unpin_particle,
    update_velocities, xpbd_constraint_count, xpbd_particle_count, xpbd_step, xpbd_total_energy,
    XpbdConstraint, XpbdConstraintType, XpbdParticle, XpbdWorld,
};

pub mod motor_controller;
pub use motor_controller::{
    add_motor_to_bank, default_pid_params, disable_motor, enable_motor, get_motor, motor_count,
    motor_update, new_motor, new_motor_bank, pid_update, proportional_controller, reset_pid,
    set_motor_target, update_all_motors, MotorBank, MotorController, PidParams, PidState,
};

pub mod character_controller;
pub use character_controller::{
    apply_gravity_cc, can_step_up, capsule_aabb, character_foot_position, character_head_position,
    character_speed, disable_controller, enable_controller, ground_check, jump_character,
    land_character, move_character, new_character_controller, push_character, CharacterCapsule,
    CharacterController, CharacterState,
};

pub mod ballistic;
pub use ballistic::{
    default_ballistic_config, drag_force as ballistic_drag_force, impact_point_on_plane,
    launch_velocity_for_target, max_range, new_projectile, projectile_kinetic_energy,
    simulate_projectile, simulate_trajectory, time_of_flight, trajectory_length,
    trajectory_max_height, BallisticConfig, Projectile, TrajectoryPoint,
};

pub mod soft_constraint;
pub use soft_constraint::{
    add_soft_constraint, add_soft_particle, disable_soft_constraint, enable_soft_constraint,
    new_soft_world, soft_constraint_count, soft_constraint_violation, soft_particle_count,
    soft_total_energy, solve_soft_distance, solve_soft_plane, solve_soft_point, step_soft_world,
    SoftConstraint, SoftConstraintType, SoftConstraintWorld,
};

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

pub mod ocean_waves;
pub use ocean_waves::{
    advance_ocean, default_ocean_config, gerstner_displacement, gerstner_normal,
    gerstner_phase_speed, new_ocean_surface, ocean_displacement, ocean_foam_mask, ocean_height_at,
    ocean_normal, sample_ocean_grid, wave_frequency, GerstnerWave, OceanConfig, OceanSurface,
};

pub mod particle_emitter;
pub use particle_emitter::{
    alive_emitter_count, clear_emitter_particles, default_emitter_config, disable_emitter,
    emit_burst, emitter_particle_count, emitter_point_position, emitter_total_spawned,
    enable_emitter, new_particle_emitter as new_pe_particle_emitter, particle_age_fraction_pe,
    spawn_particle as pe_spawn_particle, update_emitter, PeEmittedParticle as EmittedParticlePe,
    PeEmitterConfig as EmitterConfigPe, PeEmitterShape as EmitterShapePe,
    PeParticleEmitter as ParticleEmitterPe,
};

pub mod wind_field;
pub use wind_field::{
    add_wind_zone, apply_wind_to_particles, default_wind_config as wind_field_default_config,
    new_wind_zone, remove_wind_zone, turbulence_offset, update_wind_time, wind_at_point,
    wind_direction_degrees, wind_drag_force as wf_drag_force, wind_gust_factor,
    wind_lift_force as wf_lift_force, wind_speed as wf_wind_speed, wind_zone_count,
    WindFieldConfig, WindFieldSample, WindZone, WindZoneOp,
};

pub mod shape_matching;
pub use shape_matching::{
    apply_shape_matching, body_volume_estimate, compute_apq, compute_com,
    default_shape_matching_config, deformation_energy, new_shape_matching_body,
    polar_extract_rotation, reset_to_rest, set_particle_mass, shape_matching_particle_count,
    shape_matching_stiffness, Mat3, ShapeMatchingBody, ShapeMatchingConfig,
};

pub mod spring_damper;
pub use spring_damper::{
    add_spring as add_damper_spring, compute_spring_force, default_spring_damper_config,
    new_spring_damper, new_spring_damper_system, remove_spring, reset_spring_system,
    set_spring_damping, set_spring_stiffness, spring_count, spring_energy,
    spring_extension as damper_spring_extension, total_system_energy, update_spring_system,
    SpringDamper, SpringDamperConfig, SpringDamperSystem,
};

pub mod spatial_hash;
pub use spatial_hash::{
    cell_count, clear_grid, default_spatial_hash_config, entry_count, grid_stats, hash_position,
    insert_aabb, insert_point, new_spatial_hash, query_aabb as spatial_query_aabb, query_point,
    query_radius, rebuild_grid, remove_entry, GridStats, SpatialHashConfig, SpatialHashEntry,
    SpatialHashGrid,
};

pub mod constraint_solver;
pub use constraint_solver::{
    add_constraint, angle_constraint_project, constraint_compliance, constraint_count,
    default_solver_config, distance_constraint_project, new_solver_state,
    position_constraint_project, remove_constraint, reset_solver, set_solver_iterations,
    solve_iteration, solve_n_iterations, total_constraint_error, ConstraintSolverConfig,
    ConstraintSolverState, ConstraintType, GenericConstraint,
};

pub mod gyroscope;
pub use gyroscope::{
    angular_momentum, apply_gyro_torque, damped_gyro_update, default_gyro_config,
    gyro_stability_metric, gyroscopic_torque, new_gyro_body, nutation_angle, precession_rate,
    set_angular_velocity, set_inertia_tensor, spin_energy, update_gyro_state, GyroBody, GyroConfig,
    GyroState,
};

pub mod rope_cloth;
pub use rope_cloth::{
    add_cloth_quad, add_rope_segment, apply_gravity_rope_cloth, cloth_quad_count,
    default_rope_cloth_config, new_rope_cloth_body, pin_rope_cloth_particle, reset_rope_cloth,
    rope_cloth_energy, rope_cloth_particle_count, rope_segment_count, unpin_rope_cloth_particle,
    update_rope_cloth, AnnotatedConstraint, ClothQuadRecord, ConstraintKind as RopeConstraintKind,
    EnergyPair, RopeClothBody, RopeClothConfig, RopeClothConstraint, RopeClothParticle,
    RopeSegmentRecord,
};

pub mod pressure_force;
pub use pressure_force::{
    apply_pressure_forces, compute_enclosed_volume, default_pressure_config, deflate, inflate,
    new_pressure_body, pressure_body_vertex_count, pressure_body_volume, pressure_energy,
    pressure_force_on_triangle, sample_pressure_triangle, set_pressure, update_pressure_body,
    PressureBody, PressureConfig, PressureEnergyPair, PressureSample, PressureVertex,
};

pub mod ball_joint;
pub use ball_joint::BallJoint;

pub mod body_sleeping;
pub use body_sleeping::BodySleeping;

pub mod capsule_shape;
pub use capsule_shape::CapsuleShape;

pub mod collision_response_model;
pub use collision_response_model::CollisionResponseModel;

pub mod constraint_bound;
pub use constraint_bound::ConstraintBound;

pub mod contact_friction_model;
pub use contact_friction_model::ContactFrictionModel;

pub mod damped_spring;
pub use damped_spring::DampedSpring;

pub mod elastic_surface;
pub use elastic_surface::ElasticSurface;

pub mod force_clamp;
pub use force_clamp::ForceClamp;

pub mod gravity_model;
pub use gravity_model::GravityModel;

pub mod impulse_cache_v2;
pub use impulse_cache_v2::{CacheEntry as ImpulseCacheEntry, ImpulseCacheV2};

pub mod joint_motor_drive;
pub use joint_motor_drive::{DriveMode, JointMotorDrive};

pub mod kinematic_target;
pub use kinematic_target::KinematicTarget;

pub mod mass_distribution;
pub use mass_distribution::MassDistribution;

pub mod plane_collider;
pub use plane_collider::PlaneCollider;

pub mod pulley_joint;
pub use pulley_joint::PulleyJoint;

pub mod angular_spring;
pub use angular_spring::AngularSpring;

pub mod body_aabb;
pub use body_aabb::BodyAabb;

pub mod capsule_contact;
pub use capsule_contact::{CapsuleContactResult, PhysCapsule};

pub mod collision_layer_matrix;
pub use collision_layer_matrix::CollisionLayerMatrix;

pub mod cone_twist;
pub use cone_twist::ConeTwist;

pub mod contact_pair;
pub use contact_pair::ContactPairSet;

pub mod continuous_collision;
pub use continuous_collision::{CcdResult, MovingSphere};

pub mod damper_element;
pub use damper_element::{DamperElement, DamperElement3d};

pub mod dynamic_body;
pub use dynamic_body::DynamicBody;

pub mod elastic_body;
pub use elastic_body::ElasticBody;

pub mod force_field_radial;
pub use force_field_radial::{FalloffType, ForceFieldRadial};

pub mod friction_joint;
pub use friction_joint::FrictionJoint;

pub mod gravity_well;
pub use gravity_well::GravityWell;

pub mod hinge_limit;
pub use hinge_limit::HingeLimit;

pub mod impulse_pair;
pub use impulse_pair::{ImpulsePair, ImpulsePairBuffer};

pub mod inertia_body;
pub use inertia_body::InertiaBody;

pub mod angular_velocity_body;
pub use angular_velocity_body::{
    axis_angle_to_angular_velocity, clamp_angular_velocity,
    integrate_orientation as avb_integrate_orientation,
    normalize_quaternion as avb_normalize_quaternion, rotation_period, AngularVelocityBody,
};

pub mod body_friction;
pub use body_friction::{
    anisotropic_friction, coulomb_friction, kinetic_friction, max_static_friction,
    viscous_friction, BodyFrictionParams, FrictionModelType,
};

pub mod capsule_pair;
pub use capsule_pair::{
    capsule_pair_normal, capsule_pair_overlap, capsule_pair_penetration, capsule_pair_sq_dist,
    closest_point_on_segment as cp_closest_on_segment, closest_points_segment_segment, CapsulePrim,
};

pub mod collision_normal;
pub use collision_normal::{
    flip_normal, is_valid_normal, normal_box_point, normal_dot, normal_sphere_sphere,
};

pub mod cone_limit;
pub use cone_limit::{angle_within_deg, cone_solid_angle, ConeLimit};

pub mod contact_resolver;
pub use contact_resolver::{
    combined_restitution, positional_correction, relative_normal_velocity,
    resolve_contact as resolve_contact_impulse, ResolvableBody, ResolveContact,
};

pub mod damped_body;
pub use damped_body::{
    critical_damping as db_critical_damping, damping_factor, overdamped_settling_time, DampedBody,
};

pub mod deformable_spring;
pub use deformable_spring::{natural_frequency, spring_force_vector, DeformableSpring};

pub mod distance_field_phys;
pub use distance_field_phys::{bake_sphere_sdf, box_sdf, sphere_sdf, DistanceField, SdfContact};

pub mod elastic_wave;
pub use elastic_wave::{cfl_stable_dt, phase_velocity, standing_wave_frequency, ElasticChain};

pub mod force_limit;
pub use force_limit::{
    clamp_force, force_within_limit, limit_impulse, safe_force, BudgetedForceAccum,
    PerAxisForceLimit,
};

pub mod friction_patch;
pub use friction_patch::{
    compute_tangent_basis, is_sticking, kinetic_friction_magnitude, FrictionPatch, FrictionState,
};

pub mod gravity_source;
pub use gravity_source::{
    combined_gravity, gravitational_potential, orbital_period, GravitySource, UniformGravity, G,
};

pub mod hinge_spring;
pub use hinge_spring::{torsional_critical_damping, torsional_natural_frequency, HingeSpring};

pub mod joint_anchor;
pub use joint_anchor::{rotate_by_quaternion, AnchorSpace, JointAnchor, JointAnchorPair};

pub mod angular_motor;
pub use angular_motor::{
    angle_diff, critical_damping as motor_critical_damping, motor_at_target, motor_kinetic_energy,
    motor_set_target, motor_step, motor_torque, new_angular_motor, wrap_angle, AngularMotor,
};

pub mod body_collision_group;
pub use body_collision_group::{
    cg_add_layer, cg_disable_filter, cg_enable_filter, cg_in_layer, cg_layer_count, cg_merge,
    cg_remove_layer, default_collision_group, ghost_collision_group, groups_collide,
    new_collision_group, CollisionGroup, CollisionMask,
};

pub mod broad_phase_pair;
pub use broad_phase_pair::{
    aabb_center, aabb_expand as bpp_aabb_expand, aabb_merge, aabb_overlaps, aabb_volume,
    broad_phase_naive, new_broad_aabb, BroadAabb, OverlapPair,
};

pub mod capsule_ray;
pub use capsule_ray::{
    capsule_surface_area, closest_t_on_segment as cr_closest_t, normalise as cr_normalise,
    ray_capsule_intersect, Ray, RayCapsule,
};

pub mod cloth_constraint;
pub use cloth_constraint::{
    bend_angle as cloth_bend_angle, dist_constraint_delta, dist_constraint_violation,
    dist_satisfied, flat_rest_angle, new_bend_constraint, new_dist_constraint,
    stiffness_from_compliance, ClothBendConstraint, ClothDistConstraint,
};

pub mod cone_body;
pub use cone_body::{
    cone_apply_impulse, cone_centroid_height, cone_inertia_axial, cone_inertia_transverse_apex,
    cone_inertia_transverse_centroid, cone_integrate, cone_kinetic_energy, cone_surface_area,
    cone_volume, new_cone_body, ConeBody,
};

pub mod contact_island;
pub use contact_island::{
    build_islands, island_count, island_labels, island_size, new_island_uf, same_island, uf_find,
    uf_union, IslandUnionFind,
};

pub mod damping_ratio;
pub use damping_ratio::{
    classify_damping, critical_damping_coeff, damped_frequency, damped_period,
    damping_ratio as compute_damping_ratio, frequency_response,
    natural_frequency as dr_natural_frequency, peak_overshoot, settling_time_2pct, DampingCategory,
};

pub mod deformable_mesh;
pub use deformable_mesh::{
    dm_add_edge, dm_add_particle, dm_integrate, dm_kinetic_energy, dm_particle_count,
    dm_potential_energy, dm_spring_forces, new_deformable_mesh, DeformParticle, DeformableMesh,
};

pub mod elastic_joint;
pub use elastic_joint::{
    elastic_critical_damping, elastic_damping_force, elastic_natural_frequency,
    elastic_potential_energy, elastic_set_rest, elastic_spring_force, elastic_within_limits,
    new_elastic_joint, ElasticJoint,
};

pub mod force_field_uniform;
pub use force_field_uniform::{
    field_acceleration_at, field_contains, field_force_at, field_magnitude, field_set_enabled,
    field_set_scale, field_work, new_bounded_field, new_uniform_field, FieldRegion,
    UniformForceField,
};

pub mod friction_surface_model;
pub use friction_surface_model::{
    angle_of_repose, combine_friction as fsm_combine_friction,
    combine_restitution as fsm_combine_restitution, friction_impulse, friction_power, ice_ice,
    is_sticking_contact, kinetic_friction_force, rolling_resistance, rubber_concrete,
    static_friction_on_slope, SurfaceMaterial,
};

pub mod gravity_zone;
pub use gravity_zone::{
    gravity_at, new_box_zone, new_sphere_zone, zone_contains, zone_set_enabled, zone_set_gravity,
    zone_volume, GravityZone, ZoneShape,
};

pub mod hinge_body;
pub use hinge_body::{
    hinge_apply_torque, hinge_at_limit, hinge_full_turn, hinge_kinetic_energy,
    hinge_normalised_angle, hinge_range, hinge_reset, new_hinge_body, HingeBody,
};

pub mod impulse_response;
pub use impulse_response::{
    apply_impulse_a, apply_impulse_b, impulse_magnitude, relative_velocity,
};

pub mod joint_damper;
pub use joint_damper::{
    damper_angular_torque, damper_decay_factor, damper_energy_step, damper_linear_force,
    damper_linear_power, damper_set_angular, damper_set_enabled, damper_set_linear,
    new_joint_damper, JointDamper,
};

pub mod kinematic_controller;
pub use kinematic_controller::{
    kc_heading_deg, kc_integrate, kc_jump, kc_kinetic_energy, kc_move, kc_on_ground, kc_speed,
    kc_stop, new_kinematic_controller, KinematicController,
};

pub mod lattice_spring;
pub use lattice_spring::{
    ls_aabb, ls_add_particle, ls_add_spring, ls_center_of_mass, ls_kinetic_energy,
    ls_particle_count, ls_spring_count, ls_step, new_lattice_sim, LatticeParticle, LatticeSim,
    LatticeSpring,
};

pub mod linear_actuator;
pub use linear_actuator::{
    la_at_target, la_kinetic_energy, la_normalized_pos, la_potential_energy, la_range, la_reset,
    la_set_target, la_step, new_linear_actuator, LinearActuator,
};

pub mod mass_spring_chain;
pub use mass_spring_chain::{
    msc_kinetic_energy, msc_max_velocity, msc_node_count, msc_potential_energy,
    msc_reset_velocities, msc_span, msc_step, new_mass_spring_chain, ChainNode, MassSpringChain,
};

pub mod mesh_collider;
pub use mesh_collider::{
    make_unit_tri, mc_nearest_tri, mc_point_in_aabb, mc_sphere_overlaps, mc_total_area,
    mc_tri_count, new_mesh_collider, MeshCollider, Triangle,
};

pub mod motor_joint_v2;
pub use motor_joint_v2::{
    angle_diff as mjv2_angle_diff, mjv2_at_target, mjv2_kinetic_energy, mjv2_normalized_angle,
    mjv2_range, mjv2_reset, mjv2_set_target, mjv2_step, new_motor_joint_v2, MotorJointV2,
};

pub mod particle_chain;
pub use particle_chain::{
    new_particle_chain, pc_avg_seg_len, pc_chain_length, pc_end_pos, pc_particle_count, pc_step,
    pc_tip_distance, ChainParticle, ParticleChain,
};

pub mod pendulum_body;
pub use pendulum_body::{
    new_pendulum_body, pb_bob_pos, pb_frequency, pb_is_at_rest, pb_kinetic_energy, pb_period,
    pb_potential_energy, pb_step, pb_total_energy, PendulumBody,
};

pub mod phase_space;
pub use phase_space::{
    new_phase_space, ps_apply_force, ps_apply_harmonic, ps_kinetic_energy, ps_max_p, ps_max_q,
    ps_period, ps_reset, ps_traj_len, ps_velocity, PhasePoint, PhaseSpace,
};

pub mod pivot_body;
pub use pivot_body::{
    new_pivot_body, pivb_angle_deg, pivb_apply_torque, pivb_inertia, pivb_kinetic_energy,
    pivb_reset, pivb_tip_pos, pivb_tip_velocity, PivotBody,
};

pub mod plane_body;
pub use plane_body::{
    new_plane_body, plane_closest_point, plane_is_above, plane_project_vel, plane_reflect_vel,
    plane_signed_dist, plane_sphere_penetration, PlaneBody,
};

pub mod position_integrator;
pub use position_integrator::{
    new_position_integrator, pi_displacement, pi_kinetic_energy, pi_reset, pi_speed, pi_step,
    pi_steps, IntegratorKind, PositionIntegrator,
};

pub mod potential_energy;
pub use potential_energy::{
    bending_pe, centrifugal_pe, coulomb_pe, free_fall_height, gravitational_pe, lennard_jones_pe,
    newtonian_grav_pe, spring_oscillator_period, spring_pe, torsional_pe, total_spring_pe,
};

pub mod pressure_body;
pub use pressure_body::{
    new_pressure_body as new_pressure_body_inflate, prb_compress, prb_density, prb_expand,
    prb_is_over_pressured, prb_normalized_volume, prb_outward_force, prb_pressure,
    prb_restoring_force, prb_volume_error, PressureBody as InflatablePressureBody,
};

pub mod prismatic_joint;
pub use prismatic_joint::{
    new_prismatic_joint, pj_apply_force, pj_at_limit, pj_kinetic_energy, pj_lock,
    pj_normalized_pos, pj_range, pj_reset, pj_unlock, pj_world_offset, PrismaticJoint,
};

pub mod projectile_body;
pub use projectile_body::{
    new_projectile_body, proj_horizontal_range, proj_is_active, proj_kinetic_energy, proj_launch,
    proj_max_range_vacuum, proj_speed, proj_step, proj_time_of_flight, ProjectileBody,
};

pub mod rack_pinion;
pub use rack_pinion::{new_rack_pinion, RackPinion};

pub mod restitution_body;
pub use restitution_body::{collision_impulse, new_restitution_body, RestitutionBody};

pub mod revolute_joint;
pub use revolute_joint::{new_revolute_joint, RevoluteJoint};

pub mod rigid_body_group;
pub use rigid_body_group::{new_rigid_body_group, GroupMember, RigidBodyGroup};

pub mod rope_segment;
pub use rope_segment::{
    new_rope_segment, RopeParticle as RopeChainParticle, RopeSegment as RopeSegmentChain,
};

pub mod rotational_body;
pub use rotational_body::{new_rotational_body, RotationalBody};

pub mod screw_joint;
pub use screw_joint::{new_screw_joint, ScrewJoint};

pub mod shape_intersection;
pub use shape_intersection::{
    capsule_capsule_closest, segment_segment_dist, sphere_plane_intersect, sphere_sphere_intersect,
};

pub mod shock_absorber;
pub use shock_absorber::{new_shock_absorber, ShockAbsorber};

pub mod sliding_body;
pub use sliding_body::{new_sliding_body, SlidingBody};

pub mod soft_body_volume;
pub use soft_body_volume::{new_soft_body_volume, tet_signed_volume, SoftBodyVolume, SoftParticle};

pub mod sphere_body;
pub use sphere_body::{new_sphere_body, SphereBody};

pub mod spring_chain;
pub use spring_chain::{new_spring_chain, ChainMass, ChainSpring, SpringChain};

pub mod static_body;
pub use static_body::{
    new_static_box, new_static_plane, new_static_sphere, StaticBody, StaticShape,
};

pub mod torsion_body;
pub use torsion_body::{new_torsion_body, TorsionBody};

pub mod universal_joint;
pub use universal_joint::{new_universal_joint, UniversalJoint};

pub mod velocity_verlet;
pub use velocity_verlet::{new_velocity_verlet, VelocityVerlet, VvParticle};

pub mod vibration_body;
pub use vibration_body::{new_vibration_body, VibrationBody};

pub mod viscous_body;
pub use viscous_body::{new_viscous_body, ViscousBody};

pub mod vortex_field;
pub use vortex_field::{circulation, new_vortex_field, Vortex, VortexField};

pub mod wave_body;
pub use wave_body::{new_wave_body, WaveBody};

pub mod wheel_body;
pub use wheel_body::{new_wheel_body, WheelBody};

pub mod wind_body;
pub use wind_body::{new_wind_body, WindBody};

pub mod xpbd_cloth;
pub use xpbd_cloth::{
    new_xpbd_cloth, ClothConstraint, ClothParticle as XpbdClothParticle, XpbdCloth,
};

pub mod xpbd_particle;
pub use xpbd_particle::{
    new_xpbd_particle_system, XpbdParticle as XpbdSimParticle, XpbdParticleSystem,
};

pub mod xpbd_shape;
pub use xpbd_shape::{new_xpbd_shape, ShapeParticle, XpbdShape};

pub mod xpbd_volume;
pub use xpbd_volume::{
    new_xpbd_volume, tet_signed_volume as xpbd_tet_signed_volume,
    VolumeConstraint as XpbdVolumeConstraint, XpbdVolume,
};

pub mod yoke_joint;
pub use yoke_joint::{new_yoke_joint, YokeJoint};

pub mod zero_gravity_body;
pub use zero_gravity_body::{new_zero_gravity_body, ZeroGravityBody};

pub mod buoyant_body;
pub use buoyant_body::{new_buoyant_body, BuoyantBody};

pub mod centrifugal_body;
pub use centrifugal_body::{new_centrifugal_body, CentrifugalBody};

pub mod coriolis_body;
pub use coriolis_body::{new_coriolis_body, CoriolisBody};

pub mod aerial_body;
pub use aerial_body::{
    ab_drag, ab_dynamic_pressure, ab_lift, ab_step, angle_of_attack, new_aerial_body, AerialBody,
    AerialBodyConfig,
};

pub mod atmosphere_body;
pub use atmosphere_body::{
    density_at, mach_number, new_atmosphere_body, pressure_at, speed_of_sound, temperature_at,
    AtmoLayer, AtmosphereBody,
};

pub mod cable_body;
pub use cable_body::{cb_particle_count, cb_step, new_cable_body, CableBody, CableParticle};

pub mod collision_event_log;
pub use collision_event_log::{
    cel_event_count, cel_record, new_collision_event_log, CollisionEvent, CollisionEventLog,
};

pub mod constraint_graph;
pub use constraint_graph::{
    cg_add, cg_edge_count, cg_remove, new_constraint_graph, ConstraintEdge,
    ConstraintGraph as CgConstraintGraph, ConstraintType as CgConstraintType,
};

pub mod damper_network;
pub use damper_network::{
    dn_add_link, dn_add_node, dn_step, new_damper_network, DamperLink, DamperNetwork, DamperNode,
};

pub mod elastic_mesh;
pub use elastic_mesh::{em_step, new_elastic_mesh, ElasticMesh, ElasticParticle, ElasticSpring};

pub mod fiber_body;
pub use fiber_body::{fb_particle_count, fb_step, new_fiber_body, FiberBody, FiberParticle};

pub mod fluid_body;
pub use fluid_body::{
    fb2_add, fb2_step, new_fluid_body, poly6_kernel, spiky_kernel_grad, FluidBody, FluidParticle,
};

pub mod foam_body;
pub use foam_body::{
    fob_add, fob_alive, fob_step, new_foam_body, FoamBody, FoamBubble, FoamConfig,
};

pub mod foam_spring;
pub use foam_spring::{
    fsn_add_node, fsn_add_spring, new_foam_spring, new_foam_spring_network, spring_period,
    FoamSpring, FoamSpringNetwork,
};

pub mod gel_body;
pub use gel_body::{
    gb_add_particle, gb_add_spring, gb_step, new_gel_body, GelBody, GelMaterial, GelParticle,
    GelSpring,
};

pub mod granular_body;
pub use granular_body::{
    grb_add, grb_step, new_granular_body, Grain as SimGrain, GranularBody as SimGranularBody,
};

pub mod hydrofoil_body;
pub use hydrofoil_body::{
    hf_drag, hf_lift, hf_step, new_hydrofoil_body, HydrofoilBody, HydrofoilConfig,
};

pub mod ice_body;
pub use ice_body::{ib_heat, ib_step, new_ice_body, IceBody, IceState};

pub mod jet_body;
pub use jet_body::{
    jb_set_throttle, jb_step, jb_thrust, new_jet_body, tsiolkovsky_delta_v, JetBody, ThrusterConfig,
};

pub mod laser_body;
pub use laser_body::{
    lb_beam_radius, lb_irradiance, lb_power_at, lb_reset_energy, lb_set_active, lb_set_power,
    lb_step, new_laser_body, LaserBody, LaserConfig,
};

pub mod liquid_body;
pub use liquid_body::{
    lb_avg_height, lb_cell_count, lb_get_height as liq_get_height, lb_set_height as liq_set_height,
    lb_step as liq_step, lb_total_volume, new_liquid_body, LiquidBody, LiquidCell,
};

pub mod magnet_body;
pub use magnet_body::{
    mb_distance, mb_field_at, mb_force_from_field, mb_moment_mag, mb_potential_energy,
    mb_step as mag_step, new_magnet_body, MagnetBody,
};

pub mod membrane_body;
pub use membrane_body::{
    mb2_avg_y, mb2_particle_count, mb2_pin, mb2_spring_count, mb2_step, new_membrane_body,
    MembraneBody, MembraneParticle, MembraneSpring,
};

pub mod mesh_body;
pub use mesh_body::{
    mbody_aabb_volume, mbody_recompute_normals, mbody_step, mbody_translate, mbody_triangle_count,
    mbody_vertex_count, new_mesh_body, MeshAabb, MeshBody, MeshTriangle, MeshVertex,
};

pub mod muscle_body;
pub use muscle_body::{
    mb_set_activation, mb_step_activation, mb_step_fiber, muscle_active_force, muscle_fl,
    muscle_fv, muscle_passive_force, new_muscle_body, MuscleBody,
};

pub mod net_body;
pub use net_body::{
    net_add_link, net_avg_y, net_link_count, net_node_count, net_pin, net_step, new_net_body,
    NetBody, NetLink, NetNode,
};

pub mod particle_filter;
pub use particle_filter::{
    new_particle_filter, pf_count, pf_ess, pf_mean_state0, pf_normalize, pf_propagate, pf_resample,
    pf_update, FilterParticle, ParticleFilter,
};

pub mod particle_system_v2;
pub use particle_system_v2::{
    new_particle_system_v2, ps2_alive_count, ps2_avg_y, ps2_capacity, ps2_kill_all, ps2_set_origin,
    ps2_set_spawn_rate, ps2_step, Particle2, ParticleSystemV2,
};

pub mod pendulum_chain_v2;
pub use pendulum_chain_v2::{
    new_pendulum_chain_v2, pc2_kinetic_energy, pc2_len, pc2_step, pc2_tip_pos, pc2_total_length,
    ChainLink, PendulumChainV2,
};

pub mod plasma_body;
pub use plasma_body::{
    coulomb_force, new_plasma_body, plasma_add_particle, plasma_center_of_mass, plasma_count,
    plasma_kinetic_energy, plasma_net_charge, plasma_step, PlasmaBody, PlasmaParticle,
};

pub mod pneumatic_body;
pub use pneumatic_body::{
    new_pneumatic_body, pnb_add_gas, pnb_compress, pnb_expand, pnb_force_on_surface, pnb_heat,
    pnb_is_burst, pnb_update_pressure, pnb_vent, PneumaticBody,
};

pub mod powder_body;
pub use powder_body::{
    new_powder_body, pwd_add_grain, pwd_avg_y, pwd_grain_count, pwd_max_y, pwd_settled_count,
    pwd_step, PowderBody, PowderGrain,
};

pub mod rack_body;
pub use rack_body::{
    new_rack_body, rack_angle_from_pos, rack_apply_force, rack_apply_torque, rack_gear_ratio,
    rack_kinetic_energy, rack_pos_from_angle, rack_reset, RackBody,
};

pub mod rigid_compound;
pub use rigid_compound::{
    new_rigid_compound, rc_add_box, rc_add_sphere, rc_center_of_mass, rc_moment_of_inertia,
    rc_shape_count, rc_step, rc_total_mass, RigidCompound, SubShape,
};

pub mod contact_cache;
pub use contact_cache::{
    cc_clear, cc_count, cc_evict_old, cc_find, cc_has_contact, cc_insert, cc_warm_lambda,
    default_contact_cache_config, new_contact_cache, CachedContact, ContactCache,
    ContactCacheConfig,
};

pub mod joint_motor_v2;
pub use joint_motor_v2::{
    jm_at_limit, jm_compute_force, jm_position_error, jm_reset, jm_set_off, jm_set_position_target,
    jm_set_velocity_target, jm_step, new_joint_motor_v2, JointMotorV2, MotorMode,
};

pub mod collision_plane;
pub use collision_plane::{
    ground_plane, new_infinite_plane, plane_point_above, plane_project_point,
    plane_signed_dist as infinite_plane_signed_dist, resolve_sphere_plane, sphere_plane_contact,
    InfinitePlane,
};

pub mod rigid_stack;
pub use rigid_stack::{
    new_rigid_stack, rs_center_of_mass, rs_count as rigid_stack_count, rs_is_stable, rs_pop_box,
    rs_push_box, rs_sleep_all, rs_total_height, rs_total_mass as rigid_stack_total_mass,
    stack_top_y, RigidStack, StackBox,
};

pub mod sand_body;
pub use sand_body::{
    default_sand_config, new_sand_body, sand_avalanche_step, sand_critical_delta, sand_deposit,
    sand_get, sand_is_stable, sand_max_slope, sand_set, sand_total_volume, SandBody, SandConfig,
};

pub mod balloon_body;
pub use balloon_body::{
    balloon_deflate, balloon_elastic_force, balloon_inflate, balloon_internal_pressure,
    balloon_is_burst, balloon_move, balloon_net_pressure, balloon_radial_force, balloon_step,
    balloon_surface_area, balloon_volume, new_balloon_body, BalloonBody,
};

pub mod capsule_body;
pub use capsule_body::{
    capsule_closest_point, capsule_inertia_longitudinal, capsule_inertia_transverse, capsule_step,
    capsule_total_length, capsule_volume as capsule_body_volume, new_capsule_body, CapsuleBody,
};

pub mod cylinder_body;
pub use cylinder_body::{
    cylinder_apply_rolling_constraint, cylinder_apply_torque, cylinder_bottom_center,
    cylinder_density, cylinder_inertia_axial, cylinder_inertia_transverse, cylinder_step,
    cylinder_top_center, cylinder_volume, new_cylinder_body, CylinderBody,
};

pub mod torus_body;
pub use torus_body::{
    new_torus_body, torus_apply_spin, torus_contains_point_2d, torus_density, torus_inertia_axial,
    torus_inertia_transverse, torus_inner_radius, torus_outer_radius, torus_step,
    torus_surface_area, torus_volume, TorusBody,
};

pub mod lattice_body;
pub use lattice_body::{
    lattice_kinetic_energy, lattice_node_count, lattice_pin, lattice_spring_count,
    lattice_step as lattice_body_step, lattice_unpin, new_lattice_body, LatticeBody, LatticeNode,
    LatticeSpring as LatticeBodySpring,
};

pub mod fiber_body_v2;
pub use fiber_body_v2::{
    fiber_bending_angle, fiber_kinetic_energy, fiber_rest_length, fiber_segment_count,
    fiber_segment_length, fiber_step as fiber_body_step, fiber_stretch_force, fiber_tip,
    new_fiber_body_v2, FiberBodyV2, FiberSegment,
};

pub mod water_wheel;
pub use water_wheel::{
    new_water_wheel, water_wheel_brake, water_wheel_energy, water_wheel_power, water_wheel_reset,
    water_wheel_rpm, water_wheel_step, water_wheel_torque, WaterWheel,
};

pub mod windmill_body;
pub use windmill_body::{
    new_windmill, windmill_apply_load, windmill_energy, windmill_power, windmill_reset,
    windmill_rpm, windmill_step, windmill_torque, windmill_tsr, WindmillBody,
};

pub mod flywheel;
pub use flywheel::{
    flywheel_angular_momentum, flywheel_at_max, flywheel_brake, flywheel_energy, flywheel_power,
    flywheel_reset, flywheel_rpm, flywheel_step, new_flywheel, Flywheel,
};

pub mod double_spring;
pub use double_spring::{
    double_spring_kinetic_energy, double_spring_omega1, double_spring_omega2,
    double_spring_potential_energy, double_spring_reset, double_spring_set_state,
    double_spring_step, double_spring_total_energy, new_double_spring, DoubleSpring,
};

pub mod damper_body;
pub use damper_body::{
    damper_at_limit, damper_compression_ratio, damper_force, damper_power, damper_reset,
    damper_set_c, damper_step, damper_total_impulse, new_damper_body, DamperBody,
};

pub mod actuator_body;
pub use actuator_body::{
    actuator_force, actuator_is_extended, actuator_is_retracted, actuator_power, actuator_reset,
    actuator_set_position, actuator_set_pressure, actuator_step, actuator_stroke_ratio,
    new_actuator, ActuatorBody,
};

pub mod brake_body;
pub use brake_body::{
    brake_cool, brake_is_locked, brake_power, brake_reset, brake_set_clamp, brake_set_engaged,
    brake_step, brake_torque, new_brake, BrakeBody,
};

pub mod cam_follower;
pub use cam_follower::{
    cam_contact_force, cam_follower_lift, cam_follower_step, cam_follower_velocity_from_profile,
    cam_max_lift, cam_radius_at, cam_reset, cam_rpm, cam_set_omega, new_cam_follower, CamFollower,
};

pub mod worm_gear;
pub use worm_gear::{
    new_worm_gear, worm_backdrive_efficiency, worm_gear_ratio, worm_input_power,
    worm_is_self_locking, worm_output_power, worm_power_loss, worm_reduction_ratio, worm_reset,
    worm_set_input, WormGear,
};

pub mod bevel_gear;
pub use bevel_gear::{
    bevel_gear_ratio, bevel_input_power, bevel_is_miter, bevel_output_power,
    bevel_pitch_cone_angle, bevel_power_loss, bevel_reset, bevel_set_input, bevel_shaft_angle_rad,
    new_bevel_gear, BevelGear,
};

pub mod chain_drive;
pub use chain_drive::{
    chain_center_distance, chain_driven_radius, chain_driver_radius, chain_gear_ratio,
    chain_has_slack, chain_length, chain_reset, chain_set_input, chain_slack_tension,
    chain_tight_tension, new_chain_drive, ChainDrive,
};

pub mod belt_drive;
pub use belt_drive::{
    belt_center_distance, belt_gear_ratio, belt_max_force_ratio, belt_max_power, belt_power_loss,
    belt_reset, belt_set_input, belt_set_slip, belt_slack_tension, belt_tight_tension,
    new_belt_drive, BeltDrive,
};

pub mod ratchet_body;
pub use ratchet_body::{
    new_ratchet, ratchet_energy, ratchet_full_rotations, ratchet_is_blocked, ratchet_reset,
    ratchet_rpm, ratchet_step, ratchet_tooth_angle, RatchetBody, RatchetDir,
};

pub mod clutch_body;
pub use clutch_body::{
    clutch_cool, clutch_is_locked, clutch_is_slipping, clutch_max_torque, clutch_power_transfer,
    clutch_reset, clutch_set_engagement, clutch_slip_speed, clutch_step, new_clutch, ClutchBody,
    ClutchState,
};

pub mod spring_pendulum;
pub use spring_pendulum::SpringPendulum;

pub mod double_pendulum;
pub use double_pendulum::DoublePendulum;

pub mod van_der_pol;
pub use van_der_pol::{vdp_trajectory, VanDerPol};

pub mod duffing_body;
pub use duffing_body::{duffing_trajectory, DuffingBody};

pub mod lorenz_attractor;
pub use lorenz_attractor::{lorenz_is_chaotic, lorenz_trajectory, LorenzAttractor};

pub mod runge_kutta;
pub use runge_kutta::{euler_scalar, integrate_scalar, rk4_scalar, rk4_vec2, rk4_vec3, rk4_vecn};

pub mod symplectic_euler;
pub use symplectic_euler::{symp_oscillator_trajectory, SympOscillator1D, SympParticle};

pub mod leapfrog_integrator;
pub use leapfrog_integrator::{leapfrog_trajectory, LeapfrogOscillator, LeapfrogParticle};

pub mod nbody_gravity;
pub use nbody_gravity::{GravBody, NBodyGravity, G as GRAV_CONST};

pub mod sph_fluid;
pub use sph_fluid::{
    poly6_2d, spiky_grad_2d, viscosity_lap_2d, SphConfig as SphFluidSimConfig, SphFluidV2,
    SphParticleV2,
};

pub mod lattice_boltzmann;
pub use lattice_boltzmann::LatticeBoltzmann;

pub mod shallow_water;
pub use shallow_water::ShallowWater;

pub mod coupled_oscillator;
pub use coupled_oscillator::CoupledOscillators;

pub mod particle_grid;
pub use particle_grid::{
    default_particle_grid_config, new_particle_grid, pg_cell_for_pos, pg_clear, pg_insert,
    pg_neighbors, pg_stats, pg_total_entries, GridStats as ParticleGridStats, ParticleGrid,
    ParticleGridConfig,
};

pub mod contact_manifold_v2;
pub use contact_manifold_v2::{ContactManifoldV2, ContactPoint as ContactPointV2, ManifoldCache};

pub mod position_based_v2;
pub use position_based_v2::{
    pbd_v2_integrate, pbd_v2_kinetic_energy, pbd_v2_update_vel, project_dist_v2, project_volume_v2,
    BendConstraintV2, DistConstraintV2, PbdV2Particle, VolumeConstraintV2,
};

pub mod xpbd_v2;
pub use xpbd_v2::{
    xpbd_v2_dist_count, xpbd_v2_kinetic_energy, xpbd_v2_predict, xpbd_v2_project_dist,
    xpbd_v2_reset_lambdas, xpbd_v2_update_vel, XpbdDihedralV2, XpbdDistV2, XpbdV2Particle,
};

pub mod projective_dynamics;
pub use projective_dynamics::{
    pd_constraint_count, pd_global_step, pd_kinetic_energy, pd_local_step, pd_predict,
    pd_update_vel, PdParticle, PdSpringConstraint,
};

pub mod fem_linear;
pub use fem_linear::{
    tet_shape_gradients, tet_signed_volume as fem_tet_signed_volume, tet_stiffness_scalar,
    FemMaterial, FemMesh, FemNode, FemTet,
};

pub mod fem_corotational;
pub use fem_corotational::{
    compute_deformation_gradient, deformation_measure, green_strain, polar_decompose,
    strain_energy_density, Mat3x3,
};

pub mod vbd_solver;
pub use vbd_solver::{
    vbd_kinetic_energy, vbd_local_solve, vbd_predict, vbd_spring_energy, vbd_step, vbd_update_vel,
    VbdParticle, VbdSpring,
};

pub mod incremental_potential;
pub use incremental_potential::{
    ipc_active_pair_count, ipc_barrier_energy, ipc_barrier_gradient, ipc_dist_sq,
    ipc_gradient_step, ipc_kinetic_energy, ipc_total_contact_energy, IpcConfig, IpcParticle,
};

pub mod material_point;
pub use material_point::{MpmCell, MpmConfig, MpmGrid, MpmPoint};

pub mod smooth_particle;
pub use smooth_particle::{
    sph2_compute_densities, sph2_compute_pressures, sph2_integrate, sph2_kinetic_energy,
    sph2_poly6, sph2_spiky_grad_mag, sph2_viscosity_lap, sph2_xsph_correction, SphV2Particle,
    WcsphConfig,
};

pub mod rigid_tree;
pub use rigid_tree::{build_chain, RigidTree, RigidTreeBody};

pub mod contact_lcp;
pub use contact_lcp::{
    apply_lcp_impulse, build_1d_contact_lcp, contact_count,
    friction_impulse as lcp_friction_impulse, lcp_gauss_seidel, relative_normal_vel, LcpContact,
    LcpSystem,
};

pub mod warm_start_v2;
pub use warm_start_v2::{apply_warm_start, blend_impulse, WarmImpulse, WarmStartCache};

pub mod time_stepping;
pub use time_stepping::{
    euler_stability_limit, verlet_stability_limit, TimeStepConfig, TimeStepper,
};

pub mod solver_stats;
pub use solver_stats::{compute_residual, jacobi_solve_stats, SolveStats, SolverStatistics};

pub mod collision_detection;
pub use collision_detection::{
    broad_phase_brute, broad_phase_sap_x, detect_sphere_contacts as cd_detect_sphere_contacts,
    narrow_sphere_plane, narrow_sphere_sphere, overlap_count, Aabb3, BroadPhaseObject,
    ContactPoint as CdContactPoint, OverlapPair as CdOverlapPair, Sphere as CdSphere,
};

pub mod lattice_boltzmann_v2;
pub use lattice_boltzmann_v2::LatticeBoltzmannV2;

pub mod sph_density;
pub use sph_density::{
    average_density, compute_density as sph_compute_density, cubic_spline_kernel,
    dist3 as sph_dist3, estimate_density_at, SphParticleDensity,
};

pub mod sph_pressure_force;
pub use sph_pressure_force::{
    clear_forces as sph_clear_forces, compute_pressure_forces, integrate_pressure, kernel_gradient,
    pressure_eos, SphPressureParticle,
};

pub mod sph_viscosity;
pub use sph_viscosity::{
    apply_viscosity, clear_visc_forces, compute_viscosity_forces, visc_dissipation,
    viscosity_kernel_laplacian, SphViscParticle,
};

pub mod eulerian_advection;
pub use eulerian_advection::{field_max, fill_circle, AdvectionGrid};

pub mod vorticity_confinement;
pub use vorticity_confinement::{max_vorticity, VorticityGrid};

pub mod level_set_advect;
pub use level_set_advect::{gradient_magnitude, LevelSetGrid};

pub mod marching_squares;
pub use marching_squares::{
    count_segments, marching_squares as extract_contour, total_contour_length, Segment,
};

pub mod rigid_contact_patch;
pub use rigid_contact_patch::{prune_contacts, ContactPatch, ContactPoint as RigidContactPoint};

pub mod friction_cone_v2;
pub use friction_cone_v2::{
    friction_clamp, sliding_direction, tangential_component, FrictionConeV2,
};

pub mod bilateral_constraint;
pub use bilateral_constraint::{
    apply_impulse as bc_apply_impulse, position_violation, BilateralConstraint,
};

pub mod unilateral_constraint;
pub use unilateral_constraint::{clamp_lambdas, total_normal_impulse, UnilateralConstraint};

pub mod penalty_force;
pub use penalty_force::{apply_penalty_forces, total_penalty_energy, PenaltyElement};

pub mod augmented_lagrangian;
pub use augmented_lagrangian::{
    al_gradient_norm, AugmentedLagrangianConstraint, AugmentedLagrangianSolver,
};

pub mod sequential_impulse;
pub use sequential_impulse::{velocity_correction, ImpulseConstraint, SequentialImpulseSolver};

pub mod warm_starting;
pub use warm_starting::{
    warm_start_hit_rate, CachedImpulse, WarmStartCache as WarmStartingCache, WarmStartKey,
};

pub mod soft_body_mass_spring;
pub use soft_body_mass_spring::{
    ms_add_particle, ms_add_spring, ms_kinetic_energy, ms_potential_energy, ms_step, new_ms_body,
    MsParticle, MsSoftBody, MsSpring,
};

pub mod muscle_activation;
pub use muscle_activation::{
    force_length, force_velocity, muscle_activate, muscle_force, muscle_set_activation, new_muscle,
    passive_force, total_muscle_force, Muscle,
};

pub mod tendon_model;
pub use tendon_model::{
    new_tendon, tendon_elongation, tendon_force, tendon_is_taut, tendon_scale_stiffness,
    tendon_set_length, tendon_stored_energy, tendon_strain, tendon_tangent_stiffness, Tendon,
};

pub mod joint_torque;
pub use joint_torque::{
    jt_in_limits, jt_kinetic_energy, jt_moment_arm, jt_set_torque, jt_step, jt_stiffness_torque,
    jt_torque_from_force, new_joint_torque, JointTorque,
};

pub mod skin_sliding;
pub use skin_sliding::{
    new_skin_layer, skin_adhesion_energy, skin_apply_slide, skin_offset_magnitude, skin_reset,
    skin_stiffness, skin_update_ref, SkinLayer,
};

pub mod fascia_model;
pub use fascia_model::{
    fascia_elastic_force, fascia_is_taut, fascia_set_hydration, fascia_set_length,
    fascia_stored_energy, fascia_strain, fascia_total_force, fascia_viscous_force, new_fascia,
    FasciaElement,
};

pub mod adipose_sim;
pub use adipose_sim::{
    adipos_impulse, adipos_is_settled, adipos_kinetic_energy, adipos_offset_mag,
    adipos_potential_energy, adipos_reset, adipos_step, new_adipos_node, AdiposNode,
};

pub mod cartilage_model;
pub use cartilage_model::{
    cartilage_contact_stress, cartilage_effective_modulus, cartilage_is_compressed,
    cartilage_repair, cartilage_reset, cartilage_step, cartilage_strain, new_cartilage, Cartilage,
};

pub mod bone_deform;
pub use bone_deform::{
    bone_axial_deformation, bone_axial_stiffness, bone_axial_stress, bone_bending_deflection,
    bone_bending_stress, bone_is_fractured, bone_set_axial, bone_set_bending, new_bone, Bone,
};

pub mod blood_pressure_sim;
pub use blood_pressure_sim::{
    bp_heartbeat, bp_is_normal, bp_mean_arterial_pressure, bp_period, bp_pressure_mmhg,
    bp_set_heart_rate, bp_simulate_cycle, bp_step, new_blood_pressure, BloodPressure,
};

pub mod lung_mechanics;
pub use lung_mechanics::{
    lung_above_frc, lung_breathe, lung_compliance, lung_elastic_pressure, lung_set_volume,
    lung_step, lung_tidal_volume, lung_ventilation, new_lung, Lung,
};

pub mod eye_pressure;
pub use eye_pressure::{
    iop_apply_medication, iop_is_elevated, iop_is_normal, iop_mmhg, iop_set_production,
    iop_steady_state, iop_step, new_eye_pressure, EyePressure,
};

pub mod fluid_muscle;
pub use fluid_muscle::{
    fluid_muscle_force, fm_blocked_force, fm_contraction_ratio, fm_is_contracting, fm_set_length,
    fm_set_pressure, fm_update_angle, new_fluid_muscle, FluidMuscle,
};

pub mod piezo_actuator;
pub use piezo_actuator::{
    new_piezo, piezo_coupling_k2, piezo_force, piezo_free_stroke, piezo_is_full_stroke,
    piezo_power, piezo_set_voltage, piezo_update_extension, PiezoActuator,
};

pub mod shape_memory_alloy;
pub use shape_memory_alloy::{
    new_sma_spring, sma_effective_modulus, sma_is_actuated, sma_martensite_fraction, sma_phase,
    sma_recovery_force, sma_set_temperature, sma_update_strain, SmaPhase, SmaSpring,
};

pub mod magnetorheological_fluid;
pub use magnetorheological_fluid::{
    mr_coil_power, mr_damping_force, mr_effective_viscosity, mr_is_off, mr_is_on, mr_set_field,
    mr_set_shear_rate, mr_shear_stress, mr_yield_stress, new_mr_fluid, MrFluid,
};

pub mod tissue_deform;
pub use tissue_deform::{
    apply_tissue_forces, integrate_tissue, rest_length as tissue_rest_length, TissueMesh,
    TissueNode, TissueParams,
};

pub mod fracture_mechanics;
pub use fracture_mechanics::{
    analyze_fracture, critical_crack_size, critical_stress as fracture_critical_stress,
    energy_release_rate_mode1, stress_intensity_mode1, stress_intensity_mode2, will_propagate,
    FractureMaterial, FractureMode, FractureResult,
};

pub mod crack_propagation;
pub use crack_propagation::{
    advance_crack, current_ki, cycles_to_failure as crack_cycles_to_failure, is_fractured,
    paris_law_da_dn, Crack, CrackPoint, ParisLawParams,
};

pub mod delamination_model;
pub use delamination_model::{
    bilinear_traction, damage_variable, dissipated_energy, failed_element_count, grow_delamination,
    is_delaminated, update_interface, CohesiveParams, InterfaceElement,
};

pub mod creep_model;
pub use creep_model::{
    integrate_creep, is_creep_damaged, larson_miller_param, monkman_grant_check,
    multiaxial_creep_rate, norton_creep_rate, rupture_time_h, CreepParams, CreepState,
};

pub mod fatigue_model;
pub use fatigue_model::{
    accumulate_damage, cycles_to_failure_sn, goodman_correction, remaining_cycles,
    stress_ratio as fatigue_stress_ratio, FatigueDamage, SnCurveParams, StressCycle,
};

pub mod plasticity_model;
pub use plasticity_model::{
    current_yield_stress, is_yielding, plastic_strain_increment, radial_return,
    shear_modulus as plasticity_shear_modulus, von_mises_stress, yield_function, PlasticState,
    PlasticityParams,
};

pub mod hyperelastic_model;
pub use hyperelastic_model::{
    cauchy_stress_isotropic, hydrostatic_pressure as hyper_hydrostatic_pressure,
    is_stable as hyper_is_stable, isochoric_energy,
    strain_energy_density as neo_hookean_strain_energy, volumetric_energy, DeformGrad,
    NeoHookeanParams,
};

pub mod viscoelastic_model;
pub use viscoelastic_model::{
    loss_tangent, maxwell_loss_modulus, maxwell_storage_modulus, KelvinVoigtModel, MaxwellModel,
    SlsModel,
};

pub mod anisotropic_material;
pub use anisotropic_material::{rotate_stiffness_z, StiffnessTensor};

pub mod composite_material;
pub use composite_material::{CompositeMaterial, Layer as CompositeLayer};

pub mod porous_media;
pub use porous_media::{
    advance_pressure_1d, darcy_flux, darcy_flux_3d, is_darcy_regime, kozeny_carman_permeability,
    porous_reynolds, seepage_velocity, storage_coefficient, PorousCell, PorousParams,
};

pub mod granular_material;
pub use granular_material::{
    angle_of_repose_deg, avalanche_threshold_height, drucker_prager_yield,
    mohr_coulomb_shear_strength, overburden_pressure as granular_overburden, settle_pile,
    will_slope_fail, GranularParams, GranularPile,
};

pub mod foam_material;
pub use foam_material::{
    energy_absorption, foam_stress, gibson_ashby_plateau, is_elastic as foam_is_elastic,
    open_cell_poisson, specific_energy_absorption, update_foam_state, FoamParams, FoamState,
};

pub mod auxetic_material;
pub use auxetic_material::{
    anisotropy_ratio, compute_auxetic_properties, effective_modulus_x, effective_poisson_ratio,
    impact_absorption_factor, is_auxetic, lateral_strain,
    relative_density as auxetic_relative_density, AuxeticParams, AuxeticProperties,
};

pub mod metamaterial_stub;
pub use metamaterial_stub::{
    bragg_frequency_hz, effective_mass_density, is_in_bandgap, locally_resonant_bandgap,
    resonator_frequency_hz, transmission_loss_db, wave_speed as metamaterial_wave_speed,
    BandgapInfo, MetamaterialKind, MetamaterialParams,
};

pub mod rigid_body_tree;
pub use rigid_body_tree::{
    count_leaves, find_root, forward_dynamics_step, total_kinetic_energy, ArtBody, ArtBodyTree,
};

pub mod floating_base;
pub use floating_base::{
    apply_wrench as apply_floating_wrench, is_at_rest, linear_kinetic_energy, normalize_quat,
    reset_velocity, FloatingBaseState,
};

pub mod zero_moment_point;
pub use zero_moment_point::{
    compute_zmp, evaluate_zmp_stability, zmp_margin, zmp_stability_label, SupportPolygon, ZmpResult,
};

pub mod capture_point;
pub use capture_point::{
    capture_point_distance_to_centre, capture_point_stable, capture_point_velocity,
    compute_capture_point, natural_frequency as lip_natural_frequency, CapturePoint,
};

pub mod com_trajectory;
pub use com_trajectory::{
    average_com_height, linear_com_trajectory, sample_com_trajectory, ComTrajectory, ComWaypoint,
};

pub mod foot_placement;
pub use foot_placement::{
    footstep_alternates, footstep_distance, footstep_feasible, footstep_path_length,
    plan_footsteps, FootPlacementConfig, Footstep,
};

pub mod gait_scheduler;
pub use gait_scheduler::{
    gait_phase, is_double_support, phase_to_leg_phase, scheduler_phases, time_to_next_transition,
    GaitScheduler, GaitType, LegPhase,
};

pub mod step_pattern;
pub use step_pattern::{
    active_step, generate_walk_pattern, pattern_alternates, pattern_total_duration, StepEvent,
    StepPattern,
};

pub mod balance_controller;
pub use balance_controller::{
    balance_error, balance_error_within_tolerance, clamp_torques, compute_balance_torques,
    BalanceController, BalanceGains,
};

pub mod push_recovery;
pub use push_recovery::{
    classify_push, compute_recovery, emergency_step_required, recommended_step_reach,
    PushRecoveryConfig, PushSeverity, RecoveryResponse,
};

pub mod fall_detection;
pub use fall_detection::{
    classify_fall_state, fall_detected, fall_state_label, recovery_possible, update_fall_detector,
    FallDetector, FallDetectorConfig, FallState,
};

pub mod self_righting;
pub use self_righting::{
    advance_planner, is_righted, righting_torque, RightingPhase, SelfRightingConfig,
    SelfRightingPlanner,
};

pub mod locomotion_fsm;
pub use locomotion_fsm::{
    is_moving, mode_label, mode_step_frequency, update_locomotion_fsm, LocoMode, LocomotionFsm,
};

pub mod terrain_estimator;
pub use terrain_estimator::{
    flat_terrain, terrain_gradient, terrain_normal, terrain_too_steep, update_terrain_from_contact,
    TerrainEstimate, TerrainEstimator,
};

pub mod contact_estimator;
pub use contact_estimator::{
    any_hard_contact, contact_count as foot_contact_count, estimate_contact_state, total_grf,
    update_foot_contact, ContactEstimatorConfig, ContactState, FootContact,
};

pub mod wrench_estimator;
pub use wrench_estimator::{
    add_wrenches, external_wrench_detected, force_direction, update_wrench_estimate, Wrench,
    WrenchEstimator, WrenchEstimatorConfig,
};

pub mod sensor_imu;
pub use sensor_imu::{
    accel_to_roll_pitch, integrate_gyro, is_static as imu_is_static,
    vec3_magnitude as imu_vec3_magnitude, ImuConfig, ImuSample, ImuSensor,
};

pub mod sensor_force_plate;
pub use sensor_force_plate::{
    centre_of_pressure as force_plate_cop, force_overrange as force_plate_overrange, is_contact,
    mean_fz, resultant_force, ForcePlateConfig, ForcePlateSample, ForcePlateSensor,
};

pub mod sensor_emg;
pub use sensor_emg::{
    dominant_channel, moving_average_envelope, normalise as emg_normalise, rms, EmgConfig,
    EmgFrame, EmgSensor,
};

pub mod sensor_motion_capture;
pub use sensor_motion_capture::{
    all_markers_valid, find_marker, marker_centroid, marker_distance, visible_marker_count,
    MocapConfig, MocapFrame, MocapMarker, MocapSensor,
};

pub mod sensor_pressure_mat;
pub use sensor_pressure_mat::{
    active_cells, centre_of_pressure as pressure_mat_cop, contact_area_m2 as pressure_contact_area,
    peak_pressure, total_force_n, PressureFrame, PressureMatConfig, PressureMatSensor,
};

pub mod sensor_flex;
pub use sensor_flex::{
    angle_to_resistance, max_bend_sample, mean_angle as flex_mean_angle, resistance_in_range,
    resistance_to_angle, FlexConfig, FlexSample, FlexSensor,
};

pub mod sensor_strain_gauge;
pub use sensor_strain_gauge::{
    delta_resistance, mean_strain, peak_strain, resistance_to_strain, strain_overrange,
    strain_to_stress_pa, StrainGaugeConfig, StrainGaugeSensor, StrainSample,
};

pub mod sensor_load_cell;
pub use sensor_load_cell::{
    compute_impulse, force_overrange as load_cell_overrange, force_to_voltage, mean_force,
    nonlinearity_error_n, peak_force, voltage_to_force, LoadCellConfig, LoadCellSample,
    LoadCellSensor,
};

pub mod sensor_encoder;
pub use sensor_encoder::{
    angular_velocity_rad_s, angular_velocity_rpm, count_to_degrees, count_to_radians,
    current_angle_deg as encoder_angle_deg, rpm_overrange, EncoderConfig, EncoderSample,
    EncoderSensor,
};

pub mod sensor_potentiometer;
pub use sensor_potentiometer::{
    angle_to_voltage, angular_velocity_deg_s, current_angle_deg as pot_angle_deg,
    max_linearity_error_deg, voltage_in_range, voltage_to_angle, PotSample, PotentiometerConfig,
    PotentiometerSensor,
};

pub mod sensor_ultrasonic;
pub use sensor_ultrasonic::{
    beam_footprint_m, distance_in_range, distance_to_tof, median_distance, tof_to_distance,
    valid_fraction, UltrasonicConfig, UltrasonicSample, UltrasonicSensor,
};

pub mod sensor_lidar_stub;
pub use sensor_lidar_stub::{
    cloud_centroid, filter_by_range, high_intensity_count, max_point_range, point_range,
    LidarConfig, LidarFrame, LidarPoint, LidarSensor,
};

pub mod sensor_camera_stub;
pub use sensor_camera_stub::{
    backproject_ray, fov_deg, pixel_area_at_depth, pixel_in_bounds, project_point,
    CameraIntrinsics, CameraPose, CameraStub,
};

pub mod sensor_depth_camera;
pub use sensor_depth_camera::{
    expected_pixel_count, mean_depth, unproject_frame, valid_pixel_count, DepthCameraConfig,
    DepthCameraSensor, DepthFrame, DepthPoint,
};

pub mod sensor_tactile;
pub use sensor_tactile::{
    active_taxels, any_taxel_overrange, centre_of_force, contact_area_m2 as tactile_contact_area,
    peak_taxel_force, total_contact_force, TactileConfig, TactileFrame, TactileSensor,
};

pub mod sensor_temperature;
pub use sensor_temperature::{
    celsius_to_fahrenheit, celsius_to_kelvin, fahrenheit_to_celsius, is_fever, mean_temperature,
    quantise as temperature_quantise, samples_at_site, temperature_in_range, TemperatureConfig,
    TemperatureSample, TemperatureSensor, TemperatureSensorType,
};

pub mod actuator_dc_motor;
pub use actuator_dc_motor::{
    clamp_voltage, compute_current as dc_motor_compute_current,
    compute_torque as dc_motor_compute_torque, no_load_speed, stall_torque,
    step_motor as dc_step_motor, DcMotor, DcMotorParams, DcMotorState,
};

pub mod actuator_servo;
pub use actuator_servo::{
    angle_error, at_target, compute_servo_torque, new_servo_state, set_target, step_servo, RcServo,
    ServoConfig, ServoState,
};

pub mod actuator_stepper;
pub use actuator_stepper::{
    current_angle_rad, de_energize, effective_step_angle, energize, move_to_angle,
    new_stepper_state, step_motor as stepper_step_motor, steps_to_angle, StepperConfig,
    StepperMotor, StepperState,
};

pub mod actuator_hydraulic;
pub use actuator_hydraulic::{
    extension_force, extension_ratio as hydraulic_extension_ratio, hydraulic_power,
    retraction_force, set_valve_command, step_cylinder, HydraulicCylinder, HydraulicCylinderParams,
    HydraulicCylinderState,
};

pub mod actuator_pneumatic;
pub use actuator_pneumatic::{
    extension_ratio as pneumatic_extension_ratio, is_fully_extended, piston_force, pneumatic_power,
    set_valve, step_pneumatic, PneumaticCylinder, PneumaticCylinderParams, PneumaticCylinderState,
};

pub mod actuator_linear_motor;
pub use actuator_linear_motor::{
    coil_power_dissipation, compute_linear_current, compute_thrust, no_load_velocity, peak_thrust,
    step_linear_motor, LinearMotor, LinearMotorParams, LinearMotorState,
};

pub mod actuator_cable_drive;
pub use actuator_cable_drive::{
    cable_length_from_spool, compute_cable_tension, is_cable_slack, new_cable_state, spool_torque,
    step_cable_drive, wind_spool, CableDrive, CableDriveParams, CableDriveState,
};

pub mod actuator_gear_train;
pub use actuator_gear_train::{
    add_stage, reflected_inertia as gear_reflected_inertia, single_stage_gear, stage_ratio,
    total_efficiency, total_ratio, update_gear_train, GearStageParams, GearTrain,
    GearTrainActuator, GearTrainState,
};

pub mod actuator_harmonic_drive;
pub use actuator_harmonic_drive::{
    back_drive_torque as harmonic_back_drive_torque, input_speed_valid, output_speed,
    output_torque as harmonic_output_torque, reflected_inertia as harmonic_reflected_inertia,
    update_harmonic_drive, HarmonicDriveActuator, HarmonicDriveParams, HarmonicDriveState,
};

pub mod actuator_ball_screw;
pub use actuator_ball_screw::{
    axial_force_to_torque, load_within_rating,
    mechanical_advantage as ball_screw_mechanical_advantage, omega_to_linear_velocity,
    step_ball_screw, torque_to_axial_force, BallScrewActuator, BallScrewParams, BallScrewState,
};

pub mod actuator_rack_pinion;
pub use actuator_rack_pinion::{
    mechanical_advantage as rack_mechanical_advantage, omega_to_rack_velocity,
    rack_force_to_torque, rack_travel_ratio, step_rack_pinion, torque_to_rack_force,
    RackPinionActuator, RackPinionParams, RackPinionState,
};

pub mod actuator_worm_gear;
pub use actuator_worm_gear::{
    back_drive_torque as worm_back_drive_torque, forward_efficiency,
    gear_ratio as actuator_worm_gear_ratio, input_speed_from_output, is_self_locking,
    update_worm_gear, WormGearActuator, WormGearParams, WormGearState,
};

pub mod actuator_differential;
pub use actuator_differential::{
    average_output_omega, speed_difference, total_output_torque, update_differential, yaw_rate,
    DifferentialDrive, DifferentialParams, DifferentialState,
};

pub mod actuator_parallel_robot;
pub use actuator_parallel_robot::{
    forward_kinematics, home_z, joints_in_limits, set_joint_angles, workspace_radius,
    DeltaJointAngles, DeltaRobot, DeltaRobotParams, DeltaRobotState, Vec3 as DeltaVec3,
};

pub mod actuator_tendon_drive;
pub use actuator_tendon_drive::{
    is_grasping, joint_torques, new_tendon_state, step_tendon_drive, tendon_displacement,
    total_closure, TendonDriveFinger, TendonDriveParams, TendonDriveState,
};

pub mod actuator_soft_robot;
pub use actuator_soft_robot::{
    any_at_max_pressure, chamber_force, new_soft_robot_state, set_chamber_pressure,
    step_soft_robot, total_elongation, SoftRobotActuator, SoftRobotParams, SoftRobotState,
};

pub mod rigid_body_2d;
pub use rigid_body_2d::{
    angular_momentum_2d, apply_gravity_2d, apply_impulse_2d, body_2d_momentum, RigidBody2d,
};

pub mod collision_2d;
pub use collision_2d::{
    aabb_aabb_2d, aabb_circle_2d, aabb_overlap_area, circle_circle_2d,
    point_in_aabb as point_in_aabb_2d, point_in_circle, Aabb2d, Circle2d,
};

pub mod joint_2d;
pub use joint_2d::{
    clamp_angle, joint_damping_force, joint_error, joint_spring_force, Joint2d, JointKind2d,
};

pub mod chain_pendulum_2d;
pub use chain_pendulum_2d::{
    chain_end_pos, link_energy, small_angle_period, ChainPendulum2d, PendulumLink2d,
};

pub mod car_physics_2d;
pub use car_physics_2d::{car_distance_from_origin, car_heading_deg, car_kinetic_energy, Car2d};

pub mod projectile_2d;
pub use projectile_2d::{
    max_height, optimal_launch_angle, range as projectile_range,
    simulate_trajectory as simulate_trajectory_2d, time_of_flight as time_of_flight_2d,
    Projectile2d,
};

pub mod fluid_2d;
pub use fluid_2d::{divergence_at, grid_cell_count, FluidGrid2d};

pub mod cloth_2d;
pub use cloth_2d::{
    apply_spring_2d, cloth_total_kinetic_energy, Cloth2d, ClothParticle2d, ClothSpring2d,
};

pub mod soft_body_2d;
pub use soft_body_2d::{make_soft_square_2d, SoftBody2d, SoftEdge2d, SoftNode2d};

pub mod rope_2d;
pub use rope_2d::{rope_segment_length, Rope2d, RopeNode2d};

pub mod particle_system_2d;
pub use particle_system_2d::{
    particle_count_alive, particles_above_ground, Particle2d, ParticleEmitter2d,
};

pub mod explosion_2d;
pub use explosion_2d::{
    apply_explosion_to_bodies, explosion_energy, explosion_force, shockwave_radius, Explosion2d,
};

pub mod buoyancy_2d;
pub use buoyancy_2d::{
    buoyancy_force_2d, equilibrium_depth as equilibrium_depth_2d, net_force_2d, step_buoyant_body,
    BuoyancyFluid2d, BuoyantBody2d,
};

pub mod magnetic_2d;
pub use magnetic_2d::{
    apply_magnetic_force, field_magnitude_2d, lorentz_force_2d, magnetic_field_2d, total_field_2d,
    MagneticSource2d,
};

pub mod gravity_well_2d;
pub use gravity_well_2d::{
    apply_gravity_wells, gravity_force_2d, potential_energy_2d, GravityWell2d,
};

pub mod portal_physics_2d;
pub use portal_physics_2d::{
    check_portal_crossing, point_near_portal, portal_pair, teleport_through, Portal2d,
};

pub mod chaos_pendulum;
pub use chaos_pendulum::{
    new_double_pendulum, pendulum_bob2_pos, pendulum_kinetic_energy, pendulum_step,
    DoublePendulum as ChaosPendulum,
};

pub mod lorenz_system;
pub use lorenz_system::{
    lorenz_divergence, lorenz_position, lorenz_step, new_lorenz_system, LorenzParams, LorenzSystem,
};

pub mod duffing_oscillator;
pub use duffing_oscillator::{
    duffing_energy, duffing_position, duffing_step, duffing_velocity, new_duffing_oscillator,
    DuffingOscillator,
};

pub mod van_der_pol_osc;
pub use van_der_pol_osc::{
    new_van_der_pol_osc, vdp_energy, vdp_position, vdp_step, vdp_velocity, VanDerPolOsc,
};

pub mod rossler_attractor;
pub use rossler_attractor::{
    new_rossler, rossler_divergence, rossler_position, rossler_step, RosslerAttractor,
};

pub mod logistic_map;
pub use logistic_map::{
    lm_is_bounded, lm_iterate, lm_orbit, lm_step, lm_value, new_logistic_map, LogisticMap,
};

pub mod henon_map;
pub use henon_map::{
    henon_iterate, henon_position, henon_step, henon_trajectory, new_henon_map, HenonMap,
};

pub mod mandelbrot_orbit;
pub use mandelbrot_orbit::{
    mandelbrot_compute, mandelbrot_escape_iter, mandelbrot_escape_velocity, mandelbrot_in_set,
    MandelbrotOrbit,
};

pub mod julia_orbit;
pub use julia_orbit::{
    julia_compute, julia_escape_iter, julia_escape_velocity, julia_in_set, JuliaOrbit,
};

pub mod fractal_dimension;
pub use fractal_dimension::{
    fd_box_count, fd_estimate, fd_point_count, new_fractal_dimension, FractalDimension,
};

pub mod lyapunov_exponent;
pub use lyapunov_exponent::{
    lyapunov_estimate, lyapunov_is_chaotic, new_lyapunov_estimator, LyapunovEstimator,
};

pub mod bifurcation_map;
pub use bifurcation_map::{
    bif_attractor_count, bif_point_count, bifurcation_compute, compute_bifurcation,
    BifurcationPoint,
};

pub mod cellular_automaton_1d;
pub use cellular_automaton_1d::{
    ca1d_density, ca1d_iterate, ca1d_live_count, ca1d_step, new_ca1d, CellularAutomaton1D,
};

pub mod reaction_diffusion;
pub use reaction_diffusion::{
    new_reaction_diffusion, rd_cell_count, rd_mean_u, rd_mean_v, rd_step, GrayScottParams,
    ReactionDiffusion,
};

pub mod turing_pattern;
pub use turing_pattern::{
    new_turing_pattern, tp_mean_a, tp_mean_h, tp_step, tp_variance_a, TuringParams, TuringPattern,
};

pub mod boids_simulation;
pub use boids_simulation::{
    boids_avg_speed, boids_center, boids_count, boids_step, new_boids_simulation, Boid,
    BoidsParams, BoidsSimulation,
};

pub mod finite_element_3d;
pub use finite_element_3d::{
    new_finite_element_3d, FiniteElement3D, Node3D, Tetrahedron as FemTetrahedron,
};

pub mod isogeometric_analysis;
pub use isogeometric_analysis::{bspline_basis, new_iga_patch, IgaPatch1D};

pub mod boundary_element;
pub use boundary_element::{
    bem_add_element, bem_add_node, bem_centroid, bem_element_count, bem_total_length,
    new_boundary_mesh, BoundaryElement, BoundaryMesh,
};

pub mod meshfree_sph;
pub use meshfree_sph::{
    cubic_kernel, new_meshfree_sph, MeshfreeSph, SphParticle as MeshfreeSphParticle,
};

pub mod material_point_method;
pub use material_point_method::{new_mpm, GridNode, MaterialPoint, MaterialPointMethod};

pub mod peridynamics;
pub use peridynamics::{new_peridynamics, Bond, PdPoint, Peridynamics};

pub mod phase_field_fracture;
pub use phase_field_fracture::{new_phase_field_fracture, PhaseFieldFracture};

pub mod discrete_element;
pub use discrete_element::{new_discrete_element, DemParticle, DiscreteElement};

pub mod lattice_spring_model;
pub use lattice_spring_model::{
    new_lattice_spring_model, LatticeBond, LatticeNode as LatticeSpringNode, LatticeSpringModel,
};

pub mod fiber_network;
pub use fiber_network::{new_fiber_network, Fiber, FiberNetwork};

pub mod polymer_chain;
pub use polymer_chain::{new_polymer_chain, ChainSegment, PolymerChain};

pub mod worm_like_chain;
pub use worm_like_chain::{new_worm_like_chain, WormLikeChain};

pub mod membrane_tension;
pub use membrane_tension::{new_membrane_tension, MemNode, MemSpring, MembraneTension};

pub mod shell_kinematics;
pub use shell_kinematics::{
    new_shell_kinematics, Curvature, MembraneStrain, ShellKinematics, ShellNode,
};

pub mod plate_bending;
pub use plate_bending::{new_plate_bending, PlateBending};

pub mod beam_element;
pub use beam_element::{new_beam_element, BeamElem, BeamElement, BeamSection};

pub mod contact_mechanics;
pub use contact_mechanics::{
    contact_area, contact_stiffness, force_from_approach, sphere_on_flat, HertzContact,
};

pub mod adhesion_model;
pub use adhesion_model::{
    adhesion_energy, dmt_pulloff_force, is_jkr_regime, jkr_pulloff_force, jkr_zero_force_radius,
    maugis_parameter, pulloff_force, AdhesionConfig, AdhesionModel,
};

pub mod tribology_model;
pub use tribology_model::{
    archard_wear_volume, friction_energy, friction_force, is_sliding_onset, stribeck_friction,
    FrictionRegime, TribologyParams,
};

pub mod lubrication_model;
pub use lubrication_model::{
    barus_viscosity, classify_regime, entrainment_velocity, film_shear_stress,
    grubin_film_thickness, viscous_heat, LubricantConfig, LubricationRegime,
};

pub mod thermal_expansion_deform;
pub use thermal_expansion_deform::{
    biaxial_thermal_stress, constrained_thermal_stress, isotropic_strain_tensor, new_length,
    temp_for_strain, thermal_elongation, thermal_strain, volumetric_strain,
    ThermalExpansionMaterial,
};

pub mod thermoelastic_stress;
pub use thermoelastic_stress::{
    hydrostatic_thermal_stress, inelastic_heat_fraction, plane_strain_stress_xx,
    temp_from_volumetric_strain, thermal_diffusivity, thermoelastic_damping, ThermoelasticMaterial,
};

pub mod piezoelectric_model;
pub use piezoelectric_model::{
    blocking_force, charge_from_stress, coupling_coefficient, displacement_from_voltage,
    polarization_from_stress, resonant_frequency_stub,
    strain_from_field as piezo_strain_from_field, transverse_strain_from_field, PiezoConfig,
};

pub mod electrostriction_model;
pub use electrostriction_model::{
    dc_bias_strain, dielectric_energy, electrostriction_force, electrostrictive_strain,
    maxwell_stress, polarization_from_field as electrostrictive_pol_from_field,
    strain_from_e_field as electrostriction_strain_from_e,
    transverse_strain as electrostrictive_transverse_strain, ElectrostrictiveConfig,
};

pub mod magnetostrictive_model;
pub use magnetostrictive_model::{
    blocking_stress as magnetostrictive_blocking_stress,
    magnetization_from_field as magnetostrictive_mag_from_field, magnetostrictive_energy_density,
    magnetostrictive_force, magnetostrictive_strain,
    strain_from_field as magnetostrictive_strain_from_field, villari_delta_perm,
    MagnetostrictiveConfig,
};

pub mod shape_memory_effect;
pub use shape_memory_effect::{
    austenite_fraction, current_phase, effective_modulus as sme_effective_modulus,
    is_fully_austenite, martensite_fraction, recoverable_strain, recovery_force, SmaConfig,
    SmaPhase as SmePhase,
};

pub mod electrowetting;
pub use electrowetting::{
    capillary_pressure, contact_angle_ewod, contact_line_force, ewod_number, is_saturated,
    restore_contact_angle, spreading_velocity_stub, EwodConfig,
};

pub mod dielectrophoresis_model;
pub use dielectrophoresis_model::{
    clausius_mossotti_real, crossover_frequency, dep_force, dep_velocity, is_positive_dep,
    DepConfig,
};

pub mod ferrofluid_model;
pub use ferrofluid_model::{
    effective_viscosity as ferrofluid_effective_viscosity, kelvin_force_density, magnetic_pressure,
    magnetization as ferrofluid_magnetization, rosensweig_threshold, spike_height,
    wetting_pressure, FerrofluidConfig,
};

pub mod ferroelectric_model;
pub use ferroelectric_model::{
    displacement_field, hysteresis_energy_density, hysteresis_polarization, is_above_coercive,
    polarization_vs_temp, remnant_polarization, small_signal_permittivity, FerroelectricConfig,
};

pub mod multiferroic_model;
pub use multiferroic_model::{
    combined_order_parameter, electric_polarization_from_h, is_ferroelectrically_ordered,
    is_magnetically_ordered, magnetization_from_e, me_current_coefficient, me_figure_of_merit,
    me_voltage_coefficient, stress_modulated_coupling, MultiferroicConfig,
};

pub mod topological_insulator_stub;
pub use topological_insulator_stub::{
    anomalous_hall_conductance_stub, chern_number, fermi_wavevector, is_in_bulk_gap,
    mean_free_path, spin_angle, surface_dispersion, surface_dos, z2_invariant, TopoInsulatorConfig,
};

pub mod capillary_action;
pub use capillary_action::{
    capillary_pressure as jurin_capillary_pressure, capillary_rise_height, capillary_set_angle,
    capillary_volume, new_capillary_tube, CapillaryTube,
};

pub mod osmotic_pressure;
pub use osmotic_pressure::{
    new_osmotic_solution, osmotic_equilibrium_concentration, osmotic_flow_direction,
    osmotic_is_hypertonic, osmotic_pressure_pa, OsmoticSolution,
};

pub mod lymph_flow;
pub use lymph_flow::{
    lymph_flow_rate, lymph_is_absorbing, lymph_net_filtration_pressure, lymph_surface_area,
    new_lymph_capillary, LymphCapillary,
};

pub mod bone_remodeling;
pub use bone_remodeling::{
    bone_equilibrium_density, bone_is_dense, bone_is_osteoporotic, bone_mineral_content, bone_step,
    new_bone_element, BoneElement,
};

pub mod cartilage_stress;
pub use cartilage_stress::{
    cartilage_apply_load, cartilage_fluid_pressure, cartilage_recovery, cartilage_total_stress,
    new_cartilage_layer, CartilageLayer,
};

pub mod tendon_viscoelastic;
pub use tendon_viscoelastic::{
    new_tendon as new_viscoelastic_tendon, tendon_elongate, tendon_energy,
    tendon_force as viscoelastic_tendon_force, tendon_relax,
    tendon_strain as viscoelastic_tendon_strain, Tendon as ViscoelasticTendon,
};

pub mod ligament_spring;
pub use ligament_spring::{
    ligament_elongate, ligament_force, ligament_is_taut, ligament_stiffness, ligament_strain,
    new_ligament, Ligament,
};

pub mod synovial_fluid;
pub use synovial_fluid::{
    new_synovial_fluid, synovial_is_shear_thinning, synovial_lubrication_number,
    synovial_shear_stress, synovial_viscosity, SynovialFluid,
};

pub mod interstitial_fluid;
pub use interstitial_fluid::{
    interstitial_is_edematous, interstitial_net_flow, interstitial_step,
    interstitial_volume_change, new_interstitial_compartment, InterstitialCompartment,
};

pub mod blood_viscosity;
pub use blood_viscosity::{
    blood_apparent_viscosity, blood_is_flowing, blood_viscosity_casson, blood_yield_stress,
    new_blood, Blood,
};

pub mod cardiac_output;
pub use cardiac_output::{
    heart_cardiac_output_l_per_min, heart_ejection_fraction, heart_frank_starling_adjust,
    heart_mean_arterial_pressure, new_heart, Heart,
};

pub mod pulmonary_flow;
pub use pulmonary_flow::{
    new_pulmonary_circuit, pulmonary_flow_rate, pulmonary_resistance, pulmonary_transit_time,
    pulmonary_update_radius, PulmonaryCircuit,
};

pub mod renal_filtration;
pub use renal_filtration::{
    gfr_filtration_rate, gfr_is_filtering, gfr_net_filtration_pressure, gfr_update_pressure,
    new_glomerulus, Glomerulus,
};

pub mod digestive_peristalsis;
pub use digestive_peristalsis::{
    new_peristaltic_segment, peristalsis_bolus_velocity, peristalsis_is_contracted,
    peristalsis_radius, peristalsis_step, PeristalticSegment,
};

pub mod sweat_gland_model;
pub use sweat_gland_model::{
    new_sweat_gland, sweat_cooling_power_w, sweat_heat_loss, sweat_is_active, sweat_rate,
    sweat_set_core_temp, SweatGland,
};

pub mod melanin_distribution;
pub use melanin_distribution::{
    melanin_ratio, melanin_set_eumelanin, melanin_set_pheomelanin, melanin_skin_tone_index,
    melanin_total, melanin_uv_protection_factor, new_melanin_layer, MelaninLayer,
};

pub mod wound_healing_model;
pub use wound_healing_model::{
    new_wound, wound_apply_treatment, wound_healing_time_hours, wound_is_healed,
    wound_percent_closed, wound_step, Wound,
};

pub mod cell_migration_model;
pub use cell_migration_model::{
    cell_chemotaxis_step, cell_distance_from_origin, cell_position, cell_step, new_cell,
    Cell as MigratingCell,
};

pub mod tumor_growth_stub;
pub use tumor_growth_stub::{
    new_tumor, tumor_apply_treatment, tumor_doubling_time_days, tumor_is_detectable, tumor_step,
    tumor_viable_volume, Tumor,
};

pub mod muscle_fatigue_model;
pub use muscle_fatigue_model::{
    fatigue_can_exert, fatigue_percent, fatigue_recovery_step, fatigue_reset, fatigue_step,
    new_muscle_fatigue, MuscleFatigue,
};

pub mod vestibular_model;
pub use vestibular_model::{
    canal_is_stimulated, canal_perceived_rotation, canal_reset, canal_step, new_semicircular_canal,
    SemicircularCanal,
};

pub mod thermoregulation_core;
pub use thermoregulation_core::{
    new_thermocore, thermo_heat_loss_w, thermo_is_hyperthermia, thermo_is_hypothermia,
    thermo_is_normal, thermo_step, ThermoCore,
};

pub mod circadian_rhythm;
pub use circadian_rhythm::{
    circadian_alertness, circadian_is_nighttime, circadian_phase_shift, circadian_step,
    circadian_time_to_peak, new_circadian_oscillator, CircadianOscillator,
};

pub mod balance_control;
pub use balance_control::{
    balance_angle_deg, balance_apply_perturbation, balance_control_torque, balance_is_stable,
    balance_step, new_balance_pendulum, BalancePendulum,
};

pub mod posture_sway_model;
pub use posture_sway_model::{
    new_posture_sway, sway_cop_position, sway_mean_displacement, sway_path_length, sway_rms,
    sway_step, PostureSway,
};

pub mod grip_force_model;
pub use grip_force_model::{
    grip_is_slipping, grip_overshoot_factor, grip_required_force, grip_set_target, grip_step,
    new_grip_force, GripForce,
};

pub mod reflex_arc;
pub use reflex_arc::{
    new_reflex_arc, reflex_apply_fatigue, reflex_is_active, reflex_peak_force,
    reflex_reset_fatigue, reflex_response, ReflexArc,
};

pub mod proprioception_stub;
pub use proprioception_stub::{
    gto_firing, new_muscle_sensor, sensor_is_overloaded, sensor_update, spindle_II_firing,
    spindle_Ia_firing, MuscleSensor,
};

pub mod pain_threshold_model;
pub use pain_threshold_model::{
    new_nociceptor, noci_adapt, noci_is_active, noci_reset, noci_sensitize, noci_threshold,
    NociceptorState,
};

pub mod neural_signal_model;
pub use neural_signal_model::{
    fhn_is_spiking, fhn_membrane_potential, fhn_recovery, fhn_set_current, fhn_step,
    new_fitzhugh_nagumo, FitzHughNagumo,
};

pub mod sleep_wake_cycle;
pub use sleep_wake_cycle::{
    new_sleep_wake_model, sleep_alertness, sleep_hours_since_wake, sleep_set_asleep,
    sleep_should_sleep, sleep_step, SleepWakeModel,
};

pub mod fall_risk_model;
pub use fall_risk_model::{
    fall_dominant_factor, fall_is_high_risk, fall_risk_category, fall_risk_score, fall_set_balance,
    new_fall_risk_factors, FallRiskFactors,
};

pub mod joint_contact_model;
pub use joint_contact_model::{
    joint_contact_energy, joint_contact_force, joint_contact_pressure, joint_is_in_contact,
    new_joint_contact, JointContact,
};

pub mod intervertebral_disc;
pub use intervertebral_disc::{
    disc_apply_load, disc_axial_stiffness, disc_height_loss, disc_is_herniated, disc_recovery,
    new_iv_disc, IvDisc,
};

pub mod spinal_column_model;
pub use spinal_column_model::{
    new_spinal_segment, spinal_column_range_of_motion, spinal_segment_is_overloaded,
    spinal_segment_step, spinal_total_stiffness, SpinalSegment,
};

pub mod rib_cage_model;
pub use rib_cage_model::{
    new_rib_cage, rib_elastic_recoil_pressure, rib_expansion, rib_is_expanded, rib_step, RibCage,
};

pub mod lung_mechanics_v2;
pub use lung_mechanics_v2::{
    lung_v2_driving_pressure, lung_v2_flow_step, lung_v2_is_hyperinflated, lung_v2_tidal_volume,
    new_lung_mechanics_v2, LungMechanicsV2,
};

pub mod diaphragm_model;
pub use diaphragm_model::{
    diaphragm_activate, diaphragm_fatigue_step, diaphragm_force, diaphragm_is_paralyzed,
    diaphragm_pressure_contribution, new_diaphragm, Diaphragm,
};

pub mod heart_valve_model;
pub use heart_valve_model::{
    new_heart_valve, valve_flow_rate_ml_per_s, valve_is_stenotic, valve_regurgitation_fraction,
    valve_update, HeartValve,
};

pub mod coronary_flow;
pub use coronary_flow::{
    coronary_ffr, coronary_flow_ml_per_min, coronary_is_critical, coronary_resistance,
    new_coronary_artery, CoronaryArtery,
};

pub mod venous_return;
pub use venous_return::{
    new_venous_system, venous_cardiac_output_balance, venous_pressure_mmhg, venous_return_flow,
    venous_shift_volume, VenousSystem,
};

pub mod lymph_node_model;
pub use lymph_node_model::{
    lymph_node_filter, lymph_node_is_activated, lymph_node_output_load, lymph_node_swelling,
    new_lymph_node, LymphNode,
};

pub mod liver_clearance;
pub use liver_clearance::{
    liver_bioavailability, liver_clearance_rate, liver_half_life, liver_intrinsic_clearance,
    new_liver_clearance, LiverClearance,
};

pub mod gastric_acid_model;
pub use gastric_acid_model::{
    gastric_apply_antacid, gastric_is_acidic, gastric_secretion_inhibit, gastric_step,
    new_gastric_acid, GastricAcid,
};

pub mod intestinal_absorption;
pub use intestinal_absorption::{
    gut_bioavailability, gut_fraction_absorbed, gut_peak_absorption_time, gut_step,
    new_gut_compartment, GutCompartment,
};

pub mod bladder_model;
pub use bladder_model::{
    bladder_fullness, bladder_pressure_cmh2o, bladder_step, bladder_urge_threshold, bladder_void,
    new_bladder, Bladder,
};

pub mod spleen_model;
pub use spleen_model::{
    new_spleen_model, spleen_cells_filtered_per_min, spleen_is_splenomegaly,
    spleen_platelet_pool_fraction, spleen_total_volume, SpleenModel,
};

pub mod renal_tubular_reabsorption;
pub use renal_tubular_reabsorption::{
    new_tubular_reabsorption, tubular_excretion_rate, tubular_filtered_load,
    tubular_reabsorption_rate, tubular_threshold_concentration, TubularReabsorption,
};

pub mod diffusion_reaction;
pub use diffusion_reaction::{gs_get, gs_mean_u, gs_set, gs_step, new_gray_scott, GrayScottGrid};

pub mod wave_equation_1d;
pub use wave_equation_1d::{
    new_wave_1d, wave_1d_energy, wave_1d_get, wave_1d_max, wave_1d_set, wave_1d_step, Wave1d,
};

pub mod heat_equation_1d;
pub use heat_equation_1d::{
    heat_1d_get, heat_1d_max, heat_1d_mean, heat_1d_set, heat_1d_step, new_heat_1d, Heat1d,
};

pub mod lattice_boltzmann_d2q9;
pub use lattice_boltzmann_d2q9::{
    lbm_density, lbm_equilibrium, lbm_step, lbm_velocity_x, new_lbm_d2q9, LbmD2q9,
};

pub use sph_fluid::{
    new_sph_particle, sph_compute_density as sph_compute_density_v2, sph_kernel_dw, sph_kernel_w,
    sph_pressure_tait, SphParticle as SphCubicParticle,
};

pub mod discrete_element_method;
pub use discrete_element_method::{
    dem_contact_force, dem_kinetic_energy, dem_overlap, dem_step, new_dem_particle,
    DemParticle as DemParticle2d,
};

pub mod crowd_simulation;
pub use crowd_simulation::{
    agent_driving_force, agent_has_reached_goal, agent_repulsion, agent_step, new_crowd_agent,
    CrowdAgent,
};

pub mod swarm_behavior;
pub use swarm_behavior::{
    boid_alignment, boid_cohesion, boid_separation, boid_speed, boid_step, new_boid as new_boid_2d,
    Boid as Boid2d,
};

pub mod sir_epidemic_model;
pub use sir_epidemic_model::{
    new_sir_model, sir_herd_immunity_threshold, sir_is_epidemic, sir_r0, sir_step, sir_total,
    SirModel,
};

pub mod lotka_volterra;
pub use lotka_volterra::{
    lv_is_stable, lv_predator_loss, lv_prey_growth, lv_step, lv_total, new_lotka_volterra,
    LotkaVolterra,
};

pub mod reaction_kinetics;
pub use reaction_kinetics::{
    new_reaction, reaction_activation_energy, reaction_half_life, reaction_rate,
    reaction_rate_constant, reaction_set_temperature, Reaction,
};

pub mod enzyme_kinetics;
pub use enzyme_kinetics::{
    enzyme_competitive_inhibition, enzyme_half_saturation, enzyme_is_saturated,
    enzyme_turnover_number, enzyme_velocity, new_enzyme_kinetics, EnzymeKinetics,
};

pub mod vortex_model;
pub use vortex_model::{
    new_vortex_ring, vortex_energy, vortex_impulse, vortex_is_stable, vortex_self_velocity,
    vortex_step, VortexRing,
};

pub mod population_dynamics;
pub use population_dynamics::{
    new_population, population_carrying_fraction, population_doubling_time, population_equilibrium,
    population_is_growing, population_step, Population,
};

pub use material_point_method::{
    mpm_particle_kinetic_energy, mpm_particle_momentum, mpm_particle_step, mpm_von_mises_stress,
    new_mpm_particle, MpmParticle,
};

pub mod cellular_automata_phys;
pub use cellular_automata_phys::{
    ca_count_material, ca_get, ca_is_settled, ca_set, ca_step, new_ca_sand_grid, CaSandGrid,
    CELL_EMPTY, CELL_SAND, CELL_WALL, CELL_WATER,
};

// ── Wave 151A: Physics Engineering Modules ──────────────────────────────────

pub mod contact_stiffness;
pub use contact_stiffness::{
    contact_energy_loss, contact_friction_force, contact_impulse, contact_is_stiff,
    contact_normal_force, new_contact_stiffness, ContactStiffnessModel,
};

pub mod friction_coefficient;
pub use friction_coefficient::{
    friction_coefficient_for, friction_force as fc_friction_force, is_sliding, new_friction_model,
    FrictionModel, FrictionType,
};

pub mod thermal_convection;
pub use thermal_convection::{
    convection_equilibrium_temp, convective_heat_rate, lumped_time_constant,
    new_thermal_convection, nusselt_approx, thermal_resistance, ThermalConvectionModel,
};

pub mod radiation_heat;
pub use radiation_heat::{
    blackbody_emission, effective_temperature as radiation_effective_temp, net_radiation,
    new_radiation_heat, radiated_power, radiative_flux, RadiationHeatModel, STEFAN_BOLTZMANN,
};

pub mod acoustic_wave;
pub use acoustic_wave::{
    acoustic_impedance, decibel_spl, new_acoustic_wave, sound_intensity, wave_period,
    wave_pressure, wavelength, AcousticWaveModel,
};

pub mod vibration_analysis;
pub use vibration_analysis::{
    damped_natural_frequency, damping_ratio as vibration_damping_ratio, is_overdamped,
    logarithmic_decrement, natural_frequency as vibration_natural_frequency, new_vibration_model,
    resonance_amplitude, static_deflection, VibrationModel,
};

pub mod buckling_analysis;
pub use buckling_analysis::{
    buckling_safety_factor, critical_stress, effective_length, euler_critical_load, is_slender,
    new_buckling_model, slenderness_ratio, BucklingModel,
};

pub mod fatigue_life;
pub use fatigue_life::{
    cycles_to_failure, effective_endurance_limit, fatigue_strength_at_cycles, goodman_ratio,
    is_safe as fatigue_is_safe, miners_damage, new_fatigue_model, FatigueModel,
};

pub use fracture_mechanics::{
    critical_crack_length, new_fracture_model, stress_intensity, will_fracture, FractureModel,
};

pub use creep_model::{
    creep_is_significant, new_creep_model, steady_state_creep_rate, strain_at_time,
    SimpleCreepModel as CreepModel,
};

pub use composite_material::{
    composite_strength, is_fiber_dominated, longitudinal_modulus, new_composite,
    transverse_modulus, SimpleComposite,
};

pub mod fracture_mechanics_props;
pub use fracture_mechanics_props::{
    critical_crack_length as fracture_props_critical_crack, fracture_energy, is_fracture_critical,
    j_integral, new_fracture_props, paris_law, stress_intensity_mode_i, FractureProps,
};

pub mod creep_deform;
pub use creep_deform::{
    creep_compliance, creep_remaining_life, creep_strain_increment, creep_temperature_factor,
    creep_total_strain, larson_miller_parameter, monkman_grant_rupture_time,
    norton_creep_rate as creep_deform_norton_rate,
};

pub mod fiber_composite;
pub use fiber_composite::{
    composite_density as fiber_composite_density, composite_e1, composite_e2, composite_g12,
    composite_longitudinal_strength, composite_nu12, new_fiber_composite, vf_from_weight_fraction,
    FiberComposite,
};

pub mod biomechanical_loading;
pub use biomechanical_loading::{
    daily_load_cycles, is_high_load, joint_cartilage_stress, joint_reaction_force, load_index,
    new_biomechanical_load, peak_load_estimate, BiomechanicalLoad,
};

pub mod ergonomics_model;
pub use ergonomics_model::{
    duty_cycle, force_demand_ratio, is_high_risk, musculoskeletal_risk, new_ergonomics_model,
    reach_strain, rula_score, ErgonomicsModel,
};

pub mod motion_capture_model;
pub use motion_capture_model::{
    mocap_add_marker, mocap_centroid, mocap_find_marker, mocap_frame_duration, mocap_marker_count,
    mocap_marker_velocity, new_mocap_frame, MarkerPos, MotionCaptureFrame,
};

pub mod inverse_dynamics;
pub use inverse_dynamics::{
    compute_torque as compute_joint_torque, joint_torques_slice, max_joint_torque,
    new_inverse_dynamics, set_joint_angle, set_link_mass, total_joint_work, InverseDynamicsModel,
};

pub mod forward_kinematics;
pub use forward_kinematics::{
    fk_chain_length, fk_end_effector_2d, fk_end_effector_dist, fk_joint_count,
    fk_joint_position_2d, fk_set_angle, fk_workspace_radius, new_fk_chain, ForwardKinematicsChain,
};

pub mod deformable_body;
pub use deformable_body::{
    build_chain_body, new_deformable_body, DeformableBody, DeformableParticle, DeformableSpringEdge,
};

pub mod granular_flow;
pub use granular_flow::{
    default_dem_config, new_granular_flow, DemConfig, GranularFlow, GranularParticle,
};

pub mod surface_tension;
pub use surface_tension::{
    bond_number, capillary_length, laplace_pressure, new_surface_tension_model, weber_number,
    SurfaceParticle, SurfaceTensionConfig, SurfaceTensionModel,
};

pub mod magnetic_particle;
pub use magnetic_particle::{
    dipole_field, dipole_force, new_magnetic_dipole, new_magnetic_system, MagneticDipole,
    MagneticParticleSystem,
};

pub mod electrostatic_force;
pub use electrostatic_force::{
    coulomb_force as electrostatic_coulomb_force, coulomb_potential, electric_field,
    new_charged_particle, new_electrostatic_system, ChargedParticle, ElectrostaticSystem,
};

pub mod lubrication_force;
pub use lubrication_force::{
    couette_shear_stress, ehd_min_film_thickness, new_lubricant_fluid, new_lubrication_film,
    slider_bearing_load, sommerfeld_number, squeeze_film_force, LubricantFluid, LubricationFilm,
};

pub mod acoustic_impedance;
pub use acoustic_impedance::{
    acoustic_impedance as acoustic_impedance_value, air_medium, analyze_interface,
    intensity_reflection_coeff, intensity_transmission_coeff, new_acoustic_medium,
    pressure_reflection_coeff, pressure_transmission_coeff, standing_wave_ratio, steel_medium,
    transmission_loss_db as impedance_transmission_loss_db, water_medium, AcousticMedium,
    InterfaceResult,
};

pub mod thermal_expansion;
pub use thermal_expansion::{
    aluminum_thermal_material, hydrostatic_thermal_stress as te_hydrostatic_thermal_stress,
    new_thermal_bar, new_thermal_material, steel_thermal_material, ThermalBar, ThermalMaterial,
};

pub mod foam_model;
pub use foam_model::{
    new_foam_compression, new_foam_params as new_gibson_foam_params, polystyrene_foam,
    polyurethane_foam, FoamCellType, FoamCompression,
};

pub mod gel_model;
pub use gel_model::{
    chi_from_solubility, degree_of_swelling, hydrogel_params, new_gel_params, new_gel_state,
    pnipam_gel_params, shear_modulus_from_crosslink_density, GelParams, GelState,
};

pub mod heat_equation;
pub use heat_equation::{new_heat_pulse, new_heat_sine, steady_state_linear, HeatEquation1D};

pub mod diffusion_model;
pub use diffusion_model::{
    ficks_first_law, gaussian_solution, new_diffusion_1d, new_diffusion_2d, Diffusion1D,
    Diffusion2D,
};

pub mod advection_model;
pub use advection_model::{cfl_number, lax_wendroff_step, new_advection_1d, Advection1D};

pub mod poisson_solver;
pub use poisson_solver::{new_poisson_solver, poisson_1d, PoissonSolver2D};

pub mod navier_stokes_2d;
pub use navier_stokes_2d::{new_navier_stokes_2d, NavierStokes2D};

pub mod smoothed_particle_2d;
pub use smoothed_particle_2d::{new_sph_2d, wendland_grad_w, wendland_w, Sph2D, SphParticle2D};

pub mod discrete_fracture;
pub use discrete_fracture::{cubic_law_flow, new_dfn, DiscreteFractureNetwork, Fracture};

pub mod porous_flow;
pub use porous_flow::{
    darcy_velocity, forchheimer_velocity, new_darcy_1d, new_darcy_2d, DarcyFlow1D, DarcyFlow2D,
};

pub mod electrokinetic;
pub use electrokinetic::{
    henrys_function_smoluchowski, huckel_mobility, new_electrokinetic_system,
    smoluchowski_mobility, ElectrokineticConfig, ElectrokineticParticle, ElectrokineticSystem,
};

pub mod brownian_motion;
pub use brownian_motion::{
    einstein_diffusion, expected_msd_3d, new_langevin, stokes_einstein_diffusion, LangevinParticle,
    LangevinSimulation,
};

pub mod dna_model;
pub use dna_model::{bdna_model, fjc_force, new_dna_model, odijk_deflection_length, WlcDnaModel};

pub mod membrane_model;
pub use membrane_model::{
    new_helfrich_membrane, new_membrane_patch, HelfrichMembrane, MembranePatch,
};

pub mod fluid_surface;
pub use fluid_surface::{
    create_particle_grid, poly6_kernel as fluid_surface_poly6_kernel, poly6_kernel_gradient_scalar,
    spiky_kernel_gradient_scalar, viscosity_kernel_laplacian as fluid_surface_viscosity_lap,
    SphConfig as FluidSurfaceSphConfig, SphParticle as FluidSurfaceSphParticle, SphSimulation,
};

pub mod thermal_model;
pub use thermal_model::{
    BodyRegion, ThermalBody, ThermalColumn, ThermalLayer, ThermalNode, ThermalSimulation,
};
