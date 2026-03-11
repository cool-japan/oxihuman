//! Terrain heightmap physics for character grounding.
//!
//! Provides a grid-based heightmap with bilinear interpolation for height
//! and normal queries, sphere-terrain contact detection, slope traversal
//! checks, and helpers to convert the terrain into a triangle mesh.

/// Configuration for a terrain physics grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TerrainConfig {
    /// Width of a single cell in world units.
    pub cell_size: f32,
    /// Maximum traversable slope angle (radians).
    pub max_slope: f32,
    /// Gravity acceleration (world units / s²).
    pub gravity: f32,
}

/// A heightmap grid storing height samples.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TerrainGrid {
    /// Width in grid samples (columns).
    pub width: usize,
    /// Height in grid samples (rows).
    pub height: usize,
    /// Flat row-major array of heights.  Length == width * height.
    pub heights: Vec<f32>,
    /// World-space cell size.
    pub cell_size: f32,
}

/// Result of a terrain–sphere contact query.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TerrainContact {
    /// Whether the sphere is penetrating the terrain.
    pub is_contacting: bool,
    /// World-space contact point on the terrain surface.
    pub contact_point: [f32; 3],
    /// Surface normal at the contact point.
    pub contact_normal: [f32; 3],
    /// Penetration depth (positive = penetrating).
    pub penetration_depth: f32,
}

/// Returns a default `TerrainConfig`.
#[allow(dead_code)]
pub fn default_terrain_config() -> TerrainConfig {
    TerrainConfig {
        cell_size: 1.0,
        max_slope: std::f32::consts::PI / 4.0, // 45 degrees
        gravity: 9.81,
    }
}

/// Creates a new `TerrainGrid` filled with zero heights.
#[allow(dead_code)]
pub fn new_terrain_grid(width: usize, height: usize, cell_size: f32) -> TerrainGrid {
    TerrainGrid {
        width,
        height,
        heights: vec![0.0; width * height],
        cell_size,
    }
}

/// Sets the height at grid sample `(col, row)`.
#[allow(dead_code)]
pub fn set_height(grid: &mut TerrainGrid, col: usize, row: usize, h: f32) {
    if col < grid.width && row < grid.height {
        grid.heights[row * grid.width + col] = h;
    }
}

/// Gets the height at grid sample `(col, row)`.
#[allow(dead_code)]
pub fn get_height(grid: &TerrainGrid, col: usize, row: usize) -> f32 {
    if col < grid.width && row < grid.height {
        grid.heights[row * grid.width + col]
    } else {
        0.0
    }
}

/// Computes the terrain normal at world position `(wx, wz)` using
/// bilinear interpolation of the four surrounding cell normals.
#[allow(dead_code)]
pub fn terrain_normal_at(grid: &TerrainGrid, wx: f32, wz: f32) -> [f32; 3] {
    let cs = grid.cell_size;
    let fx = wx / cs;
    let fz = wz / cs;
    let col = fx.floor() as isize;
    let row = fz.floor() as isize;

    let h_get = |c: isize, r: isize| -> f32 {
        let c = c.clamp(0, grid.width as isize - 1) as usize;
        let r = r.clamp(0, grid.height as isize - 1) as usize;
        get_height(grid, c, r)
    };

    // Central differences for the normal
    let hx0 = h_get(col - 1, row);
    let hx1 = h_get(col + 1, row);
    let hz0 = h_get(col, row - 1);
    let hz1 = h_get(col, row + 1);

    let nx = -(hx1 - hx0) / (2.0 * cs);
    let ny = 1.0_f32;
    let nz = -(hz1 - hz0) / (2.0 * cs);
    normalize3([nx, ny, nz])
}

/// Bilinearly interpolates terrain height at world position `(wx, wz)`.
#[allow(dead_code)]
pub fn terrain_height_at(grid: &TerrainGrid, wx: f32, wz: f32) -> f32 {
    let cs = grid.cell_size;
    let fx = (wx / cs).clamp(0.0, (grid.width as f32) - 1.0);
    let fz = (wz / cs).clamp(0.0, (grid.height as f32) - 1.0);
    let col = fx.floor() as usize;
    let row = fz.floor() as usize;
    let tx = fx - col as f32;
    let tz = fz - row as f32;

    let c0 = col.min(grid.width - 1);
    let c1 = (col + 1).min(grid.width - 1);
    let r0 = row.min(grid.height - 1);
    let r1 = (row + 1).min(grid.height - 1);

    let h00 = get_height(grid, c0, r0);
    let h10 = get_height(grid, c1, r0);
    let h01 = get_height(grid, c0, r1);
    let h11 = get_height(grid, c1, r1);

    let hx0 = h00 * (1.0 - tx) + h10 * tx;
    let hx1 = h01 * (1.0 - tx) + h11 * tx;
    hx0 * (1.0 - tz) + hx1 * tz
}

/// Checks sphere-terrain contact.
///
/// `sphere_center` is `[x, y, z]` in world space; `radius` is the sphere radius.
#[allow(dead_code)]
pub fn terrain_contact(
    grid: &TerrainGrid,
    sphere_center: [f32; 3],
    radius: f32,
) -> TerrainContact {
    let terrain_y = terrain_height_at(grid, sphere_center[0], sphere_center[2]);
    let bottom_y = sphere_center[1] - radius;
    let penetration = terrain_y - bottom_y;
    let is_contacting = penetration > 0.0;
    let contact_point = [sphere_center[0], terrain_y, sphere_center[2]];
    let contact_normal = terrain_normal_at(grid, sphere_center[0], sphere_center[2]);
    TerrainContact {
        is_contacting,
        contact_point,
        contact_normal,
        penetration_depth: if is_contacting { penetration } else { 0.0 },
    }
}

/// Returns the number of grid columns.
#[allow(dead_code)]
pub fn terrain_grid_width(grid: &TerrainGrid) -> usize {
    grid.width
}

/// Returns the number of grid rows.
#[allow(dead_code)]
pub fn terrain_grid_height(grid: &TerrainGrid) -> usize {
    grid.height
}

/// Computes the slope (radians) at world position `(wx, wz)`.
///
/// Slope is the angle between the surface normal and the up vector `[0,1,0]`.
#[allow(dead_code)]
pub fn terrain_slope_at(grid: &TerrainGrid, wx: f32, wz: f32) -> f32 {
    let n = terrain_normal_at(grid, wx, wz);
    n[1].clamp(-1.0, 1.0).acos()
}

/// Returns `true` if the slope at `(wx, wz)` is below `config.max_slope`.
#[allow(dead_code)]
pub fn is_traversable(grid: &TerrainGrid, config: &TerrainConfig, wx: f32, wz: f32) -> bool {
    terrain_slope_at(grid, wx, wz) <= config.max_slope
}

/// Returns the axis-aligned bounding box of the terrain as `[min_x, min_y, min_z, max_x, max_y, max_z]`.
#[allow(dead_code)]
pub fn terrain_bounds(grid: &TerrainGrid) -> [f32; 6] {
    let min_h = grid.heights.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_h = grid.heights.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let max_x = grid.width as f32 * grid.cell_size;
    let max_z = grid.height as f32 * grid.cell_size;
    [0.0, min_h, 0.0, max_x, max_h, max_z]
}

/// Applies gravity to a character, moving it toward the terrain.
///
/// `character_pos` is `[x, y, z]`; `velocity_y` is the current vertical
/// velocity.  Returns the new `[x, y, z]` and updated `velocity_y`.
#[allow(dead_code)]
pub fn apply_gravity_to_character(
    grid: &TerrainGrid,
    config: &TerrainConfig,
    character_pos: [f32; 3],
    velocity_y: f32,
    dt: f32,
) -> ([f32; 3], f32) {
    let new_vy = velocity_y - config.gravity * dt;
    let new_y = character_pos[1] + new_vy * dt;
    let terrain_y = terrain_height_at(grid, character_pos[0], character_pos[2]);
    if new_y <= terrain_y {
        ([character_pos[0], terrain_y, character_pos[2]], 0.0)
    } else {
        ([character_pos[0], new_y, character_pos[2]], new_vy)
    }
}

/// Converts the terrain grid into a flat triangle mesh.
///
/// Returns `(positions, indices)` suitable for rendering.
#[allow(dead_code)]
pub fn terrain_to_mesh(grid: &TerrainGrid) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut positions = Vec::with_capacity(grid.width * grid.height);
    for row in 0..grid.height {
        for col in 0..grid.width {
            let x = col as f32 * grid.cell_size;
            let y = get_height(grid, col, row);
            let z = row as f32 * grid.cell_size;
            positions.push([x, y, z]);
        }
    }
    let mut indices = Vec::new();
    for row in 0..grid.height.saturating_sub(1) {
        for col in 0..grid.width.saturating_sub(1) {
            let tl = (row * grid.width + col) as u32;
            let tr = tl + 1;
            let bl = tl + grid.width as u32;
            let br = bl + 1;
            // Two triangles per quad
            indices.extend_from_slice(&[tl, bl, tr]);
            indices.extend_from_slice(&[tr, bl, br]);
        }
    }
    (positions, indices)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len_sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if len_sq < 1e-12 {
        return [0.0, 1.0, 0.0];
    }
    let inv = 1.0 / len_sq.sqrt();
    [v[0] * inv, v[1] * inv, v[2] * inv]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_grid(w: usize, h: usize) -> TerrainGrid {
        new_terrain_grid(w, h, 1.0)
    }

    fn slope_grid() -> TerrainGrid {
        let mut g = new_terrain_grid(4, 4, 1.0);
        for row in 0..4 {
            for col in 0..4 {
                set_height(&mut g, col, row, col as f32 * 0.5);
            }
        }
        g
    }

    #[test]
    fn test_default_terrain_config() {
        let cfg = default_terrain_config();
        assert!(cfg.cell_size > 0.0);
        assert!(cfg.gravity > 0.0);
    }

    #[test]
    fn test_new_terrain_grid_dimensions() {
        let g = new_terrain_grid(8, 8, 1.0);
        assert_eq!(terrain_grid_width(&g), 8);
        assert_eq!(terrain_grid_height(&g), 8);
    }

    #[test]
    fn test_set_and_get_height() {
        let mut g = flat_grid(4, 4);
        set_height(&mut g, 2, 1, 5.0);
        assert!((get_height(&g, 2, 1) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_get_height_out_of_bounds_returns_zero() {
        let g = flat_grid(4, 4);
        assert_eq!(get_height(&g, 100, 100), 0.0);
    }

    #[test]
    fn test_terrain_height_at_flat() {
        let g = flat_grid(4, 4);
        let h = terrain_height_at(&g, 1.5, 1.5);
        assert!(h.abs() < 1e-6);
    }

    #[test]
    fn test_terrain_height_at_bilinear() {
        let mut g = new_terrain_grid(3, 3, 1.0);
        // Set heights so we can predict the bilinear result
        set_height(&mut g, 0, 0, 0.0);
        set_height(&mut g, 1, 0, 1.0);
        set_height(&mut g, 0, 1, 0.0);
        set_height(&mut g, 1, 1, 1.0);
        // At (0.5, 0) halfway between col 0 and col 1, row 0 → height 0.5
        let h = terrain_height_at(&g, 0.5, 0.0);
        assert!((h - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_terrain_normal_at_flat_is_up() {
        let g = flat_grid(8, 8);
        let n = terrain_normal_at(&g, 3.5, 3.5);
        assert!((n[1] - 1.0).abs() < 1e-5, "y should be ~1 on flat terrain, got {:?}", n);
    }

    #[test]
    fn test_terrain_normal_unit_length() {
        let g = slope_grid();
        let n = terrain_normal_at(&g, 1.5, 1.5);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_terrain_contact_below_surface() {
        let mut g = flat_grid(10, 10);
        set_height(&mut g, 5, 5, 1.0);
        // Sphere just below the surface
        let center = [5.0, 0.5, 5.0];
        let contact = terrain_contact(&g, center, 0.4);
        assert!(contact.is_contacting);
        assert!(contact.penetration_depth > 0.0);
    }

    #[test]
    fn test_terrain_contact_above_surface() {
        let g = flat_grid(10, 10);
        let center = [5.0, 5.0, 5.0];
        let contact = terrain_contact(&g, center, 0.5);
        assert!(!contact.is_contacting);
    }

    #[test]
    fn test_terrain_slope_at_flat() {
        let g = flat_grid(8, 8);
        let slope = terrain_slope_at(&g, 3.5, 3.5);
        assert!(slope < 1e-4, "Flat terrain slope should be ~0, got {}", slope);
    }

    #[test]
    fn test_is_traversable_flat() {
        let g = flat_grid(8, 8);
        let cfg = default_terrain_config();
        assert!(is_traversable(&g, &cfg, 3.5, 3.5));
    }

    #[test]
    fn test_terrain_bounds() {
        let g = flat_grid(4, 4);
        let bounds = terrain_bounds(&g);
        assert_eq!(bounds[0], 0.0); // min_x
        assert!((bounds[3] - 4.0).abs() < 1e-5); // max_x = width * cell_size
    }

    #[test]
    fn test_apply_gravity_lands_on_terrain() {
        let g = flat_grid(10, 10);
        let cfg = default_terrain_config();
        let pos = [5.0, 10.0, 5.0];
        let mut p = pos;
        let mut vy = 0.0_f32;
        for _ in 0..200 {
            let (np, nvy) = apply_gravity_to_character(&g, &cfg, p, vy, 0.05);
            p = np;
            vy = nvy;
        }
        // Character should have landed at terrain height (0.0 for flat grid)
        assert!(p[1].abs() < 1e-4, "Should land on terrain, got y={}", p[1]);
    }

    #[test]
    fn test_terrain_to_mesh_vertex_count() {
        let g = flat_grid(4, 4);
        let (pos, _) = terrain_to_mesh(&g);
        assert_eq!(pos.len(), 16);
    }

    #[test]
    fn test_terrain_to_mesh_index_count() {
        let g = flat_grid(4, 4);
        let (_, idx) = terrain_to_mesh(&g);
        // (4-1)*(4-1)*2*3 = 9*6 = 54
        assert_eq!(idx.len(), 54);
    }
}
