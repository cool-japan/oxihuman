// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Proxy generation functions: measurement-based, PCA-fitted, and voxelized.

use crate::sampling::Capsule as SamplingCapsule;
use crate::{BodyProxies, BoxProxy, CapsuleProxy, SphereProxy};
use oxihuman_mesh::measurements::{compute_aabb, compute_measurements, Aabb, BodyMeasurements};
use oxihuman_mesh::mesh::MeshBuffers;

// ── Body-part vertex-group definitions ────────────────────────────────────────

/// Body part band: (name, y_fraction_low, y_fraction_high, optional x_sign for limbs).
/// x_sign = Some(-1.0) → left side (x < center_x), Some(1.0) → right side, None → full width.
pub struct BodyPartBand {
    pub name: &'static str,
    pub y_lo: f32,
    pub y_hi: f32,
    pub x_sign: Option<f32>,
}

pub const BODY_PART_BANDS: &[BodyPartBand] = &[
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

/// Generate surface-sampling based fitted capsules for each body part.
///
/// Uses PCA-based [`crate::sampling::fit_capsule`] instead of AABB bounding boxes
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
            crate::sampling::fit_capsule(&verts)
        };

        result.push((band.name.to_string(), capsule));
    }

    result
}

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

                while let Some((cx2, cy, cz2)) = queue.pop_front() {
                    regions[rid].1.push(idx(cx2, cy, cz2));

                    // 6-connected neighbours
                    let neighbours = [
                        (cx2.wrapping_sub(1), cy, cz2),
                        (cx2 + 1, cy, cz2),
                        (cx2, cy.wrapping_sub(1), cz2),
                        (cx2, cy + 1, cz2),
                        (cx2, cy, cz2.wrapping_sub(1)),
                        (cx2, cy, cz2 + 1),
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
