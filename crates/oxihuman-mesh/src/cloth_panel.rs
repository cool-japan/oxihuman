// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;
use std::f32::consts::PI;

// ─── PanelParams ─────────────────────────────────────────────────────────────

/// Parameters controlling resolution and UV scale for cloth panels.
pub struct PanelParams {
    /// Horizontal grid divisions (default 8).
    pub resolution_u: usize,
    /// Vertical grid divisions (default 8).
    pub resolution_v: usize,
    /// UV space scale factor (default 1.0).
    pub uv_scale: f32,
    /// Extra margin added around the panel for sewing seams (default 0.0).
    pub add_seam_margin: f32,
}

impl Default for PanelParams {
    fn default() -> Self {
        Self {
            resolution_u: 8,
            resolution_v: 8,
            uv_scale: 1.0,
            add_seam_margin: 0.0,
        }
    }
}

// ─── ClothPanel ──────────────────────────────────────────────────────────────

/// A flat cloth panel mesh with seam edge information for garment construction.
pub struct ClothPanel {
    /// Human-readable name for this panel.
    pub name: String,
    /// The mesh geometry lying flat in the XY plane (Z = 0).
    pub mesh: MeshBuffers,
    /// Vertex index pairs that form sewing seam edges.
    pub seam_edges: Vec<(u32, u32)>,
    /// UV bounding box: (min_uv, max_uv).
    pub uv_bounds: ([f32; 2], [f32; 2]),
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn make_flat_mesh(positions: Vec<[f32; 3]>, uvs: Vec<[f32; 2]>, indices: Vec<u32>) -> MeshBuffers {
    let n = positions.len();
    let mut m = MeshBuffers {
        positions,
        normals: vec![[0.0, 0.0, 1.0]; n],
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
        uvs,
        indices,
        colors: None,
        has_suit: false,
    };
    compute_normals(&mut m);
    m
}

fn uv_bounds_of(mesh: &MeshBuffers) -> ([f32; 2], [f32; 2]) {
    if mesh.uvs.is_empty() {
        return ([0.0, 0.0], [0.0, 0.0]);
    }
    let mut min_u = f32::MAX;
    let mut min_v = f32::MAX;
    let mut max_u = f32::MIN;
    let mut max_v = f32::MIN;
    for uv in &mesh.uvs {
        if uv[0] < min_u {
            min_u = uv[0];
        }
        if uv[1] < min_v {
            min_v = uv[1];
        }
        if uv[0] > max_u {
            max_u = uv[0];
        }
        if uv[1] > max_v {
            max_v = uv[1];
        }
    }
    ([min_u, min_v], [max_u, max_v])
}

// ─── rectangular_panel ───────────────────────────────────────────────────────

/// Generate a rectangular cloth panel as a grid mesh in the XY plane.
///
/// UV coordinates are `(x / width, y / height) * uv_scale`.
/// Seam edges cover all four border sides.
pub fn rectangular_panel(name: &str, width: f32, height: f32, params: &PanelParams) -> ClothPanel {
    let ru = params.resolution_u.max(1);
    let rv = params.resolution_v.max(1);
    let margin = params.add_seam_margin;
    let w = width + 2.0 * margin;
    let h = height + 2.0 * margin;

    let cols = ru + 1;
    let rows = rv + 1;

    let mut positions = Vec::with_capacity(cols * rows);
    let mut uvs = Vec::with_capacity(cols * rows);

    for row in 0..rows {
        let t = row as f32 / rv as f32;
        let y = t * h - margin;
        for col in 0..cols {
            let s = col as f32 / ru as f32;
            let x = s * w - margin;
            positions.push([x, y, 0.0]);
            uvs.push([s * params.uv_scale, t * params.uv_scale]);
        }
    }

    let mut indices = Vec::with_capacity(ru * rv * 6);
    for row in 0..rv {
        for col in 0..ru {
            let a = (row * cols + col) as u32;
            let b = a + 1;
            let c = a + cols as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }

    // Seam edges: bottom (row 0), top (row rv), left (col 0), right (col ru)
    let mut seam_edges = Vec::new();
    // bottom edge
    for col in 0..ru {
        seam_edges.push((col as u32, (col + 1) as u32));
    }
    // top edge
    let top_off = (rv * cols) as u32;
    for col in 0..ru {
        seam_edges.push((top_off + col as u32, top_off + col as u32 + 1));
    }
    // left edge
    for row in 0..rv {
        seam_edges.push(((row * cols) as u32, ((row + 1) * cols) as u32));
    }
    // right edge
    for row in 0..rv {
        seam_edges.push(((row * cols + ru) as u32, ((row + 1) * cols + ru) as u32));
    }

    let mesh = make_flat_mesh(positions, uvs, indices);
    let uv_bounds = uv_bounds_of(&mesh);

    ClothPanel {
        name: name.to_string(),
        mesh,
        seam_edges,
        uv_bounds,
    }
}

// ─── trapezoid_panel ─────────────────────────────────────────────────────────

/// Generate a trapezoidal (skirt gore) panel.
///
/// X-coordinates are linearly interpolated between `top_width` and
/// `bottom_width` per row.  UV = (horizontal_fraction, row_fraction).
pub fn trapezoid_panel(
    name: &str,
    top_width: f32,
    bottom_width: f32,
    height: f32,
    params: &PanelParams,
) -> ClothPanel {
    let ru = params.resolution_u.max(1);
    let rv = params.resolution_v.max(1);
    let cols = ru + 1;
    let rows = rv + 1;

    let mut positions = Vec::with_capacity(cols * rows);
    let mut uvs = Vec::with_capacity(cols * rows);

    for row in 0..rows {
        let t = row as f32 / rv as f32;
        let y = t * height;
        let row_width = top_width + (bottom_width - top_width) * t;
        for col in 0..cols {
            let s = col as f32 / ru as f32;
            let x = (s - 0.5) * row_width;
            positions.push([x, y, 0.0]);
            uvs.push([s * params.uv_scale, t * params.uv_scale]);
        }
    }

    let mut indices = Vec::with_capacity(ru * rv * 6);
    for row in 0..rv {
        for col in 0..ru {
            let a = (row * cols + col) as u32;
            let b = a + 1;
            let c = a + cols as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }

    let mut seam_edges = Vec::new();
    for col in 0..ru {
        seam_edges.push((col as u32, (col + 1) as u32));
    }
    let top_off = (rv * cols) as u32;
    for col in 0..ru {
        seam_edges.push((top_off + col as u32, top_off + col as u32 + 1));
    }
    for row in 0..rv {
        seam_edges.push(((row * cols) as u32, ((row + 1) * cols) as u32));
        seam_edges.push(((row * cols + ru) as u32, ((row + 1) * cols + ru) as u32));
    }

    let mesh = make_flat_mesh(positions, uvs, indices);
    let uv_bounds = uv_bounds_of(&mesh);

    ClothPanel {
        name: name.to_string(),
        mesh,
        seam_edges,
        uv_bounds,
    }
}

// ─── triangle_panel ──────────────────────────────────────────────────────────

/// Generate a triangular panel subdivided into rows.
///
/// Row 0 (bottom) has `resolution_u + 1` vertices; each subsequent row
/// has one fewer vertex, converging to a single apex vertex at the top.
pub fn triangle_panel(name: &str, base: f32, height: f32, params: &PanelParams) -> ClothPanel {
    let rv = params.resolution_v.max(1);
    // number of columns at the bottom row = rv+1 (simplification: use rv as base segments)
    let base_segs = rv;

    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    // row_start[r] = starting index for row r
    let mut row_start: Vec<usize> = Vec::with_capacity(rv + 2);

    for row in 0..=rv {
        let t = row as f32 / rv as f32;
        let y = t * height;
        // vertices in this row: base_segs+1 - row (shrink toward apex)
        let n_in_row = (base_segs + 1).saturating_sub(row).max(1);
        row_start.push(positions.len());
        for col in 0..n_in_row {
            let s = if n_in_row > 1 {
                col as f32 / (n_in_row - 1) as f32
            } else {
                0.5
            };
            let x = (s - 0.5) * base * (1.0 - t);
            positions.push([x, y, 0.0]);
            uvs.push([s * params.uv_scale, t * params.uv_scale]);
        }
    }

    let mut indices = Vec::new();
    for row in 0..rv {
        let n_bot = (base_segs + 1).saturating_sub(row).max(1);
        let n_top = (base_segs + 1).saturating_sub(row + 1).max(1);
        let bot = row_start[row];
        let top = row_start[row + 1];

        let mut bi = 0usize;
        let mut ti = 0usize;
        while bi < n_bot - 1 || ti < n_top - 1 {
            if ti >= n_top - 1
                || (bi < n_bot - 1
                    && bi * (n_top.saturating_sub(1)) <= ti * (n_bot.saturating_sub(1)))
            {
                // advance bottom
                indices.push((bot + bi) as u32);
                indices.push((bot + bi + 1) as u32);
                indices.push((top + ti.min(n_top - 1)) as u32);
                bi += 1;
            } else {
                // advance top
                indices.push((bot + bi.min(n_bot - 1)) as u32);
                indices.push((top + ti + 1) as u32);
                indices.push((top + ti) as u32);
                ti += 1;
            }
        }
    }

    let mut seam_edges = Vec::new();
    // bottom base edge
    let n_base = (base_segs + 1).max(1);
    for col in 0..n_base.saturating_sub(1) {
        seam_edges.push((col as u32, (col + 1) as u32));
    }
    // left slant
    for row in 0..rv {
        seam_edges.push((row_start[row] as u32, row_start[row + 1] as u32));
    }
    // right slant
    for row in 0..rv {
        let n_bot = (base_segs + 1).saturating_sub(row).max(1);
        let n_top = (base_segs + 1).saturating_sub(row + 1).max(1);
        let right_bot = (row_start[row] + n_bot - 1) as u32;
        let right_top = (row_start[row + 1] + n_top - 1) as u32;
        if right_bot != right_top {
            seam_edges.push((right_bot, right_top));
        }
    }

    let mesh = make_flat_mesh(positions, uvs, indices);
    let uv_bounds = uv_bounds_of(&mesh);

    ClothPanel {
        name: name.to_string(),
        mesh,
        seam_edges,
        uv_bounds,
    }
}

// ─── circular_panel ──────────────────────────────────────────────────────────

/// Generate a full circular panel (like a circular skirt) using a polar grid.
///
/// UV = (angle / (2π), r / radius) scaled by `uv_scale`.
pub fn circular_panel(name: &str, radius: f32, params: &PanelParams) -> ClothPanel {
    let rings = params.resolution_v.max(1);
    let slices = params.resolution_u.max(3);

    let mut positions = Vec::new();
    let mut uvs = Vec::new();

    // Center vertex
    positions.push([0.0, 0.0, 0.0]);
    uvs.push([0.5 * params.uv_scale, 0.5 * params.uv_scale]);

    for ring in 1..=rings {
        let r = radius * ring as f32 / rings as f32;
        for slice in 0..slices {
            let theta = 2.0 * PI * slice as f32 / slices as f32;
            let x = r * theta.cos();
            let y = r * theta.sin();
            positions.push([x, y, 0.0]);
            let u = (theta / (2.0 * PI)) * params.uv_scale;
            let v = (r / radius) * params.uv_scale;
            uvs.push([u, v]);
        }
    }

    let mut indices = Vec::new();
    // Center fan (ring 1)
    for slice in 0..slices {
        let a = 0u32;
        let b = 1 + slice as u32;
        let c = 1 + (slice + 1) as u32 % slices as u32;
        indices.extend_from_slice(&[a, b, c]);
    }
    // Outer rings
    for ring in 1..rings {
        let inner_off = 1 + (ring - 1) * slices;
        let outer_off = 1 + ring * slices;
        for slice in 0..slices {
            let next = (slice + 1) % slices;
            let a = (inner_off + slice) as u32;
            let b = (inner_off + next) as u32;
            let c = (outer_off + slice) as u32;
            let d = (outer_off + next) as u32;
            indices.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }

    let mut seam_edges = Vec::new();
    // Outer boundary ring
    let outer_off = 1 + (rings - 1) * slices;
    for slice in 0..slices {
        let a = (outer_off + slice) as u32;
        let b = (outer_off + (slice + 1) % slices) as u32;
        seam_edges.push((a, b));
    }

    let mesh = make_flat_mesh(positions, uvs, indices);
    let uv_bounds = uv_bounds_of(&mesh);

    ClothPanel {
        name: name.to_string(),
        mesh,
        seam_edges,
        uv_bounds,
    }
}

// ─── sleeve_panel ────────────────────────────────────────────────────────────

/// Generate a sleeve panel (cylinder with varying radius).
///
/// The cylinder is "unrolled" flat: columns represent the angular position
/// around the sleeve; rows represent the length from wrist (`top_radius`) to
/// shoulder (`bottom_radius`).  Z = 0 for all vertices.
pub fn sleeve_panel(
    name: &str,
    length: f32,
    top_radius: f32,
    bottom_radius: f32,
    params: &PanelParams,
) -> ClothPanel {
    let slices = params.resolution_u.max(3);
    let stacks = params.resolution_v.max(1);
    let cols = slices + 1;
    let rows = stacks + 1;

    let mut positions = Vec::with_capacity(cols * rows);
    let mut uvs = Vec::with_capacity(cols * rows);

    for stack in 0..rows {
        let t = stack as f32 / stacks as f32;
        let y = t * length;
        let r = top_radius + (bottom_radius - top_radius) * t;
        for col in 0..cols {
            let theta = 2.0 * PI * col as f32 / slices as f32;
            // Unrolled: x = arc position, y = length position
            let x = r * theta;
            positions.push([x, y, 0.0]);
            uvs.push([
                (col as f32 / slices as f32) * params.uv_scale,
                t * params.uv_scale,
            ]);
        }
    }

    let mut indices = Vec::with_capacity(slices * stacks * 6);
    for stack in 0..stacks {
        for col in 0..slices {
            let a = (stack * cols + col) as u32;
            let b = a + 1;
            let c = a + cols as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }

    let mut seam_edges = Vec::new();
    // Bottom edge (stack 0)
    for col in 0..slices {
        seam_edges.push((col as u32, (col + 1) as u32));
    }
    // Top edge (stack = stacks)
    let top_off = (stacks * cols) as u32;
    for col in 0..slices {
        seam_edges.push((top_off + col as u32, top_off + col as u32 + 1));
    }
    // Left seam
    for stack in 0..stacks {
        seam_edges.push(((stack * cols) as u32, ((stack + 1) * cols) as u32));
    }
    // Right seam
    for stack in 0..stacks {
        seam_edges.push((
            (stack * cols + slices) as u32,
            ((stack + 1) * cols + slices) as u32,
        ));
    }

    let mesh = make_flat_mesh(positions, uvs, indices);
    let uv_bounds = uv_bounds_of(&mesh);

    ClothPanel {
        name: name.to_string(),
        mesh,
        seam_edges,
        uv_bounds,
    }
}

// ─── layout_panels_flat ──────────────────────────────────────────────────────

/// Lay out multiple panels side-by-side in 2D (X axis) for UV atlas packing.
///
/// Each panel is translated so panels don't overlap.  UV bounds are updated
/// to reflect the new positions.
pub fn layout_panels_flat(panels: &[ClothPanel], spacing: f32) -> Vec<ClothPanel> {
    let mut result = Vec::with_capacity(panels.len());
    let mut cursor_x = 0.0f32;

    for panel in panels {
        // Find x extent of this panel
        let x_min = panel
            .mesh
            .positions
            .iter()
            .map(|p| p[0])
            .fold(f32::MAX, f32::min);
        let x_max = panel
            .mesh
            .positions
            .iter()
            .map(|p| p[0])
            .fold(f32::MIN, f32::max);
        let offset_x = cursor_x - x_min;

        let new_positions: Vec<[f32; 3]> = panel
            .mesh
            .positions
            .iter()
            .map(|p| [p[0] + offset_x, p[1], p[2]])
            .collect();

        let new_uvs: Vec<[f32; 2]> = panel
            .mesh
            .uvs
            .iter()
            .map(|uv| [uv[0] + cursor_x, uv[1]])
            .collect();

        let n = new_positions.len();
        let mut new_mesh = MeshBuffers {
            positions: new_positions,
            normals: panel.mesh.normals.clone(),
            tangents: panel.mesh.tangents.clone(),
            uvs: new_uvs,
            indices: panel.mesh.indices.clone(),
            colors: panel.mesh.colors.clone(),
            has_suit: panel.mesh.has_suit,
        };
        compute_normals(&mut new_mesh);
        let _ = n; // suppress unused warning

        let uv_bounds = uv_bounds_of(&new_mesh);
        cursor_x += (x_max - x_min) + spacing;

        result.push(ClothPanel {
            name: panel.name.clone(),
            mesh: new_mesh,
            seam_edges: panel.seam_edges.clone(),
            uv_bounds,
        });
    }

    result
}

// ─── join_panels ─────────────────────────────────────────────────────────────

/// Join two panels along matching seam edges by welding vertices within `seam_epsilon`.
///
/// Returns a single merged `MeshBuffers` with welded boundary vertices.
pub fn join_panels(a: &ClothPanel, b: &ClothPanel, seam_epsilon: f32) -> MeshBuffers {
    let na = a.mesh.positions.len();

    // Concatenate all vertices from both panels
    let mut positions: Vec<[f32; 3]> = a.mesh.positions.clone();
    positions.extend_from_slice(&b.mesh.positions);

    let mut normals: Vec<[f32; 3]> = a.mesh.normals.clone();
    normals.extend_from_slice(&b.mesh.normals);

    let mut tangents: Vec<[f32; 4]> = a.mesh.tangents.clone();
    tangents.extend_from_slice(&b.mesh.tangents);

    let mut uvs: Vec<[f32; 2]> = a.mesh.uvs.clone();
    uvs.extend_from_slice(&b.mesh.uvs);

    // Indices from b are offset by na
    let mut indices: Vec<u32> = a.mesh.indices.clone();
    for &idx in &b.mesh.indices {
        indices.push(idx + na as u32);
    }

    // Build remapping: for each vertex in b, if it is within epsilon of a vertex
    // in a, map it to the a vertex index.
    let n_total = positions.len();
    let mut remap: Vec<u32> = (0..n_total as u32).collect();

    // positions[na..] come from mesh b; find closest vertex in a's range
    let a_positions: Vec<[f32; 3]> = positions[..na].to_vec();
    for bi in na..n_total {
        let pb = positions[bi];
        for (ai, pa) in a_positions.iter().enumerate() {
            let dx = pa[0] - pb[0];
            let dy = pa[1] - pb[1];
            let dz = pa[2] - pb[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            if dist < seam_epsilon {
                remap[bi] = ai as u32;
                break;
            }
        }
    }

    // Apply remapping to indices
    let remapped_indices: Vec<u32> = indices.iter().map(|&i| remap[i as usize]).collect();

    let n_out = positions.len();
    let mut mesh = MeshBuffers {
        positions,
        normals,
        tangents,
        uvs,
        indices: remapped_indices,
        colors: None,
        has_suit: false,
    };
    let _ = n_out;
    compute_normals(&mut mesh);
    mesh
}

// ─── total_panel_area ────────────────────────────────────────────────────────

/// Compute the total cloth surface area from a collection of panels.
pub fn total_panel_area(panels: &[ClothPanel]) -> f32 {
    panels.iter().map(|p| panel_surface_area(&p.mesh)).sum()
}

fn panel_surface_area(mesh: &MeshBuffers) -> f32 {
    let mut area = 0.0f32;
    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= mesh.positions.len() || i1 >= mesh.positions.len() || i2 >= mesh.positions.len() {
            continue;
        }
        let p0 = mesh.positions[i0];
        let p1 = mesh.positions[i1];
        let p2 = mesh.positions[i2];
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        area += 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
    }
    area
}

// ─── tshirt_panels ───────────────────────────────────────────────────────────

/// Generate a simple T-shirt panel set: front body, back body, left sleeve, right sleeve.
pub fn tshirt_panels(params: &PanelParams) -> Vec<ClothPanel> {
    vec![
        rectangular_panel("tshirt_front", 0.5, 0.7, params),
        rectangular_panel("tshirt_back", 0.5, 0.7, params),
        trapezoid_panel("tshirt_sleeve_left", 0.12, 0.20, 0.6, params),
        trapezoid_panel("tshirt_sleeve_right", 0.12, 0.20, 0.6, params),
    ]
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> PanelParams {
        PanelParams::default()
    }

    #[test]
    fn test_panel_params_default() {
        let p = PanelParams::default();
        assert_eq!(p.resolution_u, 8);
        assert_eq!(p.resolution_v, 8);
        assert!((p.uv_scale - 1.0).abs() < 1e-6);
        assert!((p.add_seam_margin - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_rectangular_panel_vertices() {
        let p = default_params();
        let panel = rectangular_panel("rect", 1.0, 1.0, &p);
        // (ru+1) * (rv+1) = 9 * 9 = 81
        assert_eq!(panel.mesh.positions.len(), 81);
        // All Z should be 0
        for pos in &panel.mesh.positions {
            assert!((pos[2]).abs() < 1e-6, "Z should be 0, got {}", pos[2]);
        }
    }

    #[test]
    fn test_rectangular_panel_faces() {
        let p = default_params();
        let panel = rectangular_panel("rect", 1.0, 1.0, &p);
        // ru * rv * 2 triangles = 8*8*2 = 128 triangles => 384 indices
        assert_eq!(panel.mesh.indices.len(), 384);
        assert_eq!(panel.mesh.face_count(), 128);
    }

    #[test]
    fn test_rectangular_panel_uvs() {
        let p = default_params();
        let panel = rectangular_panel("rect", 1.0, 1.0, &p);
        assert_eq!(panel.mesh.uvs.len(), panel.mesh.positions.len());
        // UV should be in [0, 1] for default scale
        for uv in &panel.mesh.uvs {
            assert!(
                uv[0] >= -1e-6 && uv[0] <= 1.0 + 1e-6,
                "U out of range: {}",
                uv[0]
            );
            assert!(
                uv[1] >= -1e-6 && uv[1] <= 1.0 + 1e-6,
                "V out of range: {}",
                uv[1]
            );
        }
    }

    #[test]
    fn test_trapezoid_panel() {
        let p = default_params();
        let panel = trapezoid_panel("trap", 0.3, 0.6, 1.0, &p);
        assert_eq!(panel.mesh.positions.len(), 81); // (8+1)^2
        assert_eq!(panel.mesh.face_count(), 128);
        // Bottom row should have narrower width than top
        let bottom_x_max = panel
            .mesh
            .positions
            .iter()
            .filter(|pos| pos[1].abs() < 1e-4)
            .map(|pos| pos[0].abs())
            .fold(0.0f32, f32::max);
        let top_x_max = panel
            .mesh
            .positions
            .iter()
            .filter(|pos| (pos[1] - 1.0).abs() < 1e-4)
            .map(|pos| pos[0].abs())
            .fold(0.0f32, f32::max);
        assert!(top_x_max > bottom_x_max, "Top should be wider than bottom");
    }

    #[test]
    fn test_triangle_panel() {
        let p = default_params();
        let panel = triangle_panel("tri", 1.0, 1.0, &p);
        // Should have vertices, indices, seam edges
        assert!(!panel.mesh.positions.is_empty());
        assert!(!panel.mesh.indices.is_empty());
        assert!(panel.mesh.indices.len().is_multiple_of(3));
        // All Z = 0
        for pos in &panel.mesh.positions {
            assert!((pos[2]).abs() < 1e-6);
        }
        // Apex vertex should be at the top
        let max_y = panel
            .mesh
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f32::MIN, f32::max);
        assert!(
            (max_y - 1.0).abs() < 1e-5,
            "Apex y should be 1.0, got {}",
            max_y
        );
    }

    #[test]
    fn test_circular_panel() {
        let p = default_params();
        let panel = circular_panel("circle", 1.0, &p);
        assert!(!panel.mesh.positions.is_empty());
        assert!(!panel.mesh.indices.is_empty());
        // Center vertex at origin
        let center = panel.mesh.positions[0];
        assert!(center[0].abs() < 1e-6);
        assert!(center[1].abs() < 1e-6);
        assert!(center[2].abs() < 1e-6);
        // All Z = 0
        for pos in &panel.mesh.positions {
            assert!((pos[2]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_sleeve_panel() {
        let p = default_params();
        let panel = sleeve_panel("sleeve", 0.6, 0.05, 0.1, &p);
        let cols = p.resolution_u + 1;
        let rows = p.resolution_v + 1;
        assert_eq!(panel.mesh.positions.len(), cols * rows);
        assert_eq!(panel.mesh.face_count(), p.resolution_u * p.resolution_v * 2);
        // All Z = 0
        for pos in &panel.mesh.positions {
            assert!((pos[2]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_layout_panels_flat() {
        let p = default_params();
        let panels = vec![
            rectangular_panel("a", 1.0, 1.0, &p),
            rectangular_panel("b", 1.0, 1.0, &p),
        ];
        let laid = layout_panels_flat(&panels, 0.1);
        assert_eq!(laid.len(), 2);
        // Second panel should be to the right of the first
        let a_x_max = laid[0]
            .mesh
            .positions
            .iter()
            .map(|p| p[0])
            .fold(f32::MIN, f32::max);
        let b_x_min = laid[1]
            .mesh
            .positions
            .iter()
            .map(|p| p[0])
            .fold(f32::MAX, f32::min);
        assert!(
            b_x_min >= a_x_max - 1e-5,
            "Second panel should not overlap first"
        );
    }

    #[test]
    fn test_total_panel_area() {
        let p = default_params();
        let panels = vec![
            rectangular_panel("rect1", 1.0, 1.0, &p),
            rectangular_panel("rect2", 2.0, 1.0, &p),
        ];
        let area = total_panel_area(&panels);
        // rect1 = 1.0, rect2 = 2.0, total ~ 3.0
        assert!((area - 3.0).abs() < 0.01, "Expected ~3.0, got {}", area);
    }

    #[test]
    fn test_tshirt_panels_count() {
        let p = default_params();
        let panels = tshirt_panels(&p);
        assert_eq!(panels.len(), 4);
        assert_eq!(panels[0].name, "tshirt_front");
        assert_eq!(panels[1].name, "tshirt_back");
        assert_eq!(panels[2].name, "tshirt_sleeve_left");
        assert_eq!(panels[3].name, "tshirt_sleeve_right");
    }

    #[test]
    fn test_panel_seam_edges() {
        let p = default_params();
        let panel = rectangular_panel("rect", 1.0, 1.0, &p);
        // Should have 4 * 8 = 32 seam edges (4 sides * 8 segments each)
        assert_eq!(panel.seam_edges.len(), 32);
        // All edge indices must be valid
        let n = panel.mesh.positions.len() as u32;
        for (a, b) in &panel.seam_edges {
            assert!(*a < n, "Seam edge vertex {} out of range", a);
            assert!(*b < n, "Seam edge vertex {} out of range", b);
        }
    }

    #[test]
    fn test_panel_uv_bounds() {
        let p = default_params();
        let panel = rectangular_panel("rect", 1.0, 1.0, &p);
        let (min_uv, max_uv) = panel.uv_bounds;
        assert!(min_uv[0] >= -1e-6);
        assert!(min_uv[1] >= -1e-6);
        assert!(max_uv[0] <= 1.0 + 1e-6);
        assert!(max_uv[1] <= 1.0 + 1e-6);
        // min should be < max
        assert!(min_uv[0] < max_uv[0]);
        assert!(min_uv[1] < max_uv[1]);
    }
}
