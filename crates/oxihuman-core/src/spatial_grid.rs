// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! General spatial indexing for core-level queries: 2D uniform grid + KD-tree stub.

#![allow(dead_code)]

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for the spatial index.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpatialIndexConfig {
    /// Number of grid cells along X axis.
    pub grid_cols: usize,
    /// Number of grid cells along Y axis.
    pub grid_rows: usize,
    /// World-space width of the grid.
    pub world_width: f32,
    /// World-space height of the grid.
    pub world_height: f32,
    /// If true, build a KD-tree on rebuild.
    pub use_kdtree: bool,
}

/// A 2-D point with an associated payload id.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialPoint {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

/// KD-tree node stub (for future full implementation).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KdNode {
    pub point: SpatialPoint,
    pub left: Option<Box<KdNode>>,
    pub right: Option<Box<KdNode>>,
    pub axis: u8, // 0 = X, 1 = Y
}

/// The spatial index structure.
#[allow(dead_code)]
pub struct SpatialIndex {
    pub config: SpatialIndexConfig,
    /// All stored points.
    pub points: Vec<SpatialPoint>,
    /// 2-D grid cells: each cell holds ids of points in it.
    pub grid: Vec<Vec<u32>>,
    /// KD-tree root (populated after rebuild_index if use_kdtree is true).
    pub kdtree: Option<Box<KdNode>>,
}

/// Axis-aligned bounding box for 2D queries.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Aabb2d {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

/// Index bounds type alias.
pub type IndexBounds = Aabb2d;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn cell_for_point(cfg: &SpatialIndexConfig, x: f32, y: f32) -> (usize, usize) {
    let col = ((x / cfg.world_width * cfg.grid_cols as f32) as isize)
        .clamp(0, cfg.grid_cols as isize - 1) as usize;
    let row = ((y / cfg.world_height * cfg.grid_rows as f32) as isize)
        .clamp(0, cfg.grid_rows as isize - 1) as usize;
    (col, row)
}

fn cell_index(cfg: &SpatialIndexConfig, col: usize, row: usize) -> usize {
    row * cfg.grid_cols + col
}

fn dist_sq(a: &SpatialPoint, b: &SpatialPoint) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn build_kdtree(mut pts: Vec<SpatialPoint>, depth: usize) -> Option<Box<KdNode>> {
    if pts.is_empty() {
        return None;
    }
    let axis = (depth % 2) as u8;
    if axis == 0 {
        pts.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        pts.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
    }
    let mid = pts.len() / 2;
    let right_pts = pts.split_off(mid + 1);
    let Some(point) = pts.pop() else { return None; };
    Some(Box::new(KdNode {
        point,
        left: build_kdtree(pts, depth + 1),
        right: build_kdtree(right_pts, depth + 1),
        axis,
    }))
}

fn kd_nearest<'a>(
    node: &'a KdNode,
    query: &SpatialPoint,
    best: &mut Option<&'a SpatialPoint>,
    best_dist: &mut f32,
) {
    let d = dist_sq(&node.point, query);
    if d < *best_dist {
        *best_dist = d;
        *best = Some(&node.point);
    }
    let diff = if node.axis == 0 {
        query.x - node.point.x
    } else {
        query.y - node.point.y
    };
    let (near, far) = if diff <= 0.0 {
        (node.left.as_deref(), node.right.as_deref())
    } else {
        (node.right.as_deref(), node.left.as_deref())
    };
    if let Some(n) = near {
        kd_nearest(n, query, best, best_dist);
    }
    if diff * diff < *best_dist {
        if let Some(f) = far {
            kd_nearest(f, query, best, best_dist);
        }
    }
}

fn kd_range<'a>(node: &'a KdNode, query: &SpatialPoint, radius_sq: f32, out: &mut Vec<&'a SpatialPoint>) {
    if dist_sq(&node.point, query) <= radius_sq {
        out.push(&node.point);
    }
    let diff = if node.axis == 0 {
        query.x - node.point.x
    } else {
        query.y - node.point.y
    };
    // Always search the near side; search the far side only when the
    // splitting plane may intersect the query circle.
    let (near, far) = if diff <= 0.0 {
        (node.left.as_deref(), node.right.as_deref())
    } else {
        (node.right.as_deref(), node.left.as_deref())
    };
    if let Some(n) = near {
        kd_range(n, query, radius_sq, out);
    }
    if diff * diff <= radius_sq {
        if let Some(f) = far {
            kd_range(f, query, radius_sq, out);
        }
    }
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Create a default config (100x100 grid, 1000x1000 world).
#[allow(dead_code)]
pub fn default_spatial_index_config() -> SpatialIndexConfig {
    SpatialIndexConfig {
        grid_cols: 100,
        grid_rows: 100,
        world_width: 1000.0,
        world_height: 1000.0,
        use_kdtree: true,
    }
}

/// Create a new empty spatial index.
#[allow(dead_code)]
pub fn new_spatial_index(config: SpatialIndexConfig) -> SpatialIndex {
    let cells = config.grid_cols * config.grid_rows;
    SpatialIndex {
        grid: vec![Vec::new(); cells],
        config,
        points: Vec::new(),
        kdtree: None,
    }
}

/// Insert a point. Returns the point's id.
#[allow(dead_code)]
pub fn insert_point(idx: &mut SpatialIndex, id: u32, x: f32, y: f32) {
    let pt = SpatialPoint { id, x, y };
    let (col, row) = cell_for_point(&idx.config, x, y);
    let ci = cell_index(&idx.config, col, row);
    idx.grid[ci].push(id);
    idx.points.push(pt);
}

/// Remove a point by id. Returns true if found.
#[allow(dead_code)]
pub fn remove_point(idx: &mut SpatialIndex, id: u32) -> bool {
    if let Some(pos) = idx.points.iter().position(|p| p.id == id) {
        let pt = idx.points.swap_remove(pos);
        let (col, row) = cell_for_point(&idx.config, pt.x, pt.y);
        let ci = cell_index(&idx.config, col, row);
        idx.grid[ci].retain(|&pid| pid != id);
        true
    } else {
        false
    }
}

/// Find the nearest point to (qx, qy). Returns None if empty.
#[allow(dead_code)]
pub fn nearest_neighbor(idx: &SpatialIndex, qx: f32, qy: f32) -> Option<&SpatialPoint> {
    if idx.points.is_empty() {
        return None;
    }
    if let Some(root) = idx.kdtree.as_deref() {
        let query = SpatialPoint { id: 0, x: qx, y: qy };
        let mut best: Option<&SpatialPoint> = None;
        let mut best_dist = f32::MAX;
        kd_nearest(root, &query, &mut best, &mut best_dist);
        return best;
    }
    // Fallback: linear scan.
    let query = SpatialPoint { id: 0, x: qx, y: qy };
    idx.points.iter().min_by(|a, b| {
        dist_sq(a, &query)
            .partial_cmp(&dist_sq(b, &query))
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// Find the k nearest points to (qx, qy).
#[allow(dead_code)]
pub fn k_nearest(idx: &SpatialIndex, qx: f32, qy: f32, k: usize) -> Vec<&SpatialPoint> {
    if idx.points.is_empty() || k == 0 {
        return Vec::new();
    }
    let query = SpatialPoint { id: 0, x: qx, y: qy };
    let mut sorted: Vec<&SpatialPoint> = idx.points.iter().collect();
    sorted.sort_by(|a, b| {
        dist_sq(a, &query)
            .partial_cmp(&dist_sq(b, &query))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted.truncate(k);
    sorted
}

/// Return all points within `radius` of (cx, cy).
#[allow(dead_code)]
pub fn range_query(idx: &SpatialIndex, cx: f32, cy: f32, radius: f32) -> Vec<&SpatialPoint> {
    let r2 = radius * radius;
    let query = SpatialPoint { id: 0, x: cx, y: cy };
    if let Some(root) = idx.kdtree.as_deref() {
        let mut out = Vec::new();
        kd_range(root, &query, r2, &mut out);
        return out;
    }
    idx.points.iter().filter(|p| dist_sq(p, &query) <= r2).collect()
}

/// Return all points within an AABB.
#[allow(dead_code)]
pub fn aabb_query_2d(idx: &SpatialIndex, aabb: Aabb2d) -> Vec<&SpatialPoint> {
    idx.points
        .iter()
        .filter(|p| p.x >= aabb.min_x && p.x <= aabb.max_x && p.y >= aabb.min_y && p.y <= aabb.max_y)
        .collect()
}

/// Total number of points in the index.
#[allow(dead_code)]
pub fn point_count(idx: &SpatialIndex) -> usize {
    idx.points.len()
}

/// Rebuild grid and optionally the KD-tree from current points.
#[allow(dead_code)]
pub fn rebuild_index(idx: &mut SpatialIndex) {
    let cells = idx.config.grid_cols * idx.config.grid_rows;
    idx.grid = vec![Vec::new(); cells];
    for pt in &idx.points {
        let (col, row) = cell_for_point(&idx.config, pt.x, pt.y);
        let ci = cell_index(&idx.config, col, row);
        idx.grid[ci].push(pt.id);
    }
    if idx.config.use_kdtree {
        idx.kdtree = build_kdtree(idx.points.clone(), 0);
    } else {
        idx.kdtree = None;
    }
}

/// Remove all points and reset the grid.
#[allow(dead_code)]
pub fn clear_spatial_index(idx: &mut SpatialIndex) {
    idx.points.clear();
    idx.kdtree = None;
    for cell in &mut idx.grid {
        cell.clear();
    }
}

/// Return the AABB that encloses all indexed points. Returns None if empty.
#[allow(dead_code)]
pub fn index_bounds(idx: &SpatialIndex) -> Option<IndexBounds> {
    if idx.points.is_empty() {
        return None;
    }
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for p in &idx.points {
        if p.x < min_x { min_x = p.x; }
        if p.y < min_y { min_y = p.y; }
        if p.x > max_x { max_x = p.x; }
        if p.y > max_y { max_y = p.y; }
    }
    Some(Aabb2d { min_x, min_y, max_x, max_y })
}

/// Serialize index metadata to a simple JSON string.
#[allow(dead_code)]
pub fn spatial_index_to_json(idx: &SpatialIndex) -> String {
    let bounds_str = if let Some(b) = index_bounds(idx) {
        format!(
            r#"{{"min_x":{:.2},"min_y":{:.2},"max_x":{:.2},"max_y":{:.2}}}"#,
            b.min_x, b.min_y, b.max_x, b.max_y
        )
    } else {
        "null".to_string()
    };
    format!(
        r#"{{"point_count":{},"grid_cols":{},"grid_rows":{},"kdtree":{},"bounds":{}}}"#,
        point_count(idx),
        idx.config.grid_cols,
        idx.config.grid_rows,
        idx.kdtree.is_some(),
        bounds_str
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_idx() -> SpatialIndex {
        new_spatial_index(default_spatial_index_config())
    }

    fn make_small_idx() -> SpatialIndex {
        new_spatial_index(SpatialIndexConfig {
            grid_cols: 10,
            grid_rows: 10,
            world_width: 100.0,
            world_height: 100.0,
            use_kdtree: true,
        })
    }

    #[test]
    fn test_new_index_empty() {
        let idx = make_idx();
        assert_eq!(point_count(&idx), 0);
    }

    #[test]
    fn test_insert_point_increments_count() {
        let mut idx = make_small_idx();
        insert_point(&mut idx, 1, 10.0, 20.0);
        insert_point(&mut idx, 2, 30.0, 40.0);
        assert_eq!(point_count(&idx), 2);
    }

    #[test]
    fn test_remove_existing_point() {
        let mut idx = make_small_idx();
        insert_point(&mut idx, 1, 10.0, 10.0);
        assert!(remove_point(&mut idx, 1));
        assert_eq!(point_count(&idx), 0);
    }

    #[test]
    fn test_remove_nonexistent_point() {
        let mut idx = make_small_idx();
        assert!(!remove_point(&mut idx, 999));
    }

    #[test]
    fn test_nearest_neighbor_empty() {
        let idx = make_small_idx();
        assert!(nearest_neighbor(&idx, 0.0, 0.0).is_none());
    }

    #[test]
    fn test_nearest_neighbor_single() {
        let mut idx = make_small_idx();
        insert_point(&mut idx, 42, 50.0, 50.0);
        rebuild_index(&mut idx);
        let nn = nearest_neighbor(&idx, 55.0, 55.0).unwrap();
        assert_eq!(nn.id, 42);
    }

    #[test]
    fn test_nearest_neighbor_closest() {
        let mut idx = make_small_idx();
        insert_point(&mut idx, 1, 10.0, 10.0);
        insert_point(&mut idx, 2, 50.0, 50.0);
        insert_point(&mut idx, 3, 90.0, 90.0);
        rebuild_index(&mut idx);
        let nn = nearest_neighbor(&idx, 48.0, 48.0).unwrap();
        assert_eq!(nn.id, 2);
    }

    #[test]
    fn test_k_nearest_returns_k() {
        let mut idx = make_small_idx();
        for i in 0..10_u32 {
            insert_point(&mut idx, i, (i as f32) * 5.0, 0.0);
        }
        rebuild_index(&mut idx);
        let result = k_nearest(&idx, 25.0, 0.0, 3);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_k_nearest_empty() {
        let idx = make_small_idx();
        assert!(k_nearest(&idx, 0.0, 0.0, 3).is_empty());
    }

    #[test]
    fn test_range_query_circle() {
        let mut idx = make_small_idx();
        insert_point(&mut idx, 1, 10.0, 10.0);
        insert_point(&mut idx, 2, 50.0, 50.0);
        insert_point(&mut idx, 3, 11.0, 11.0);
        rebuild_index(&mut idx);
        let result = range_query(&idx, 10.0, 10.0, 5.0);
        let ids: Vec<u32> = result.iter().map(|p| p.id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&3));
        assert!(!ids.contains(&2));
    }

    #[test]
    fn test_aabb_query_2d() {
        let mut idx = make_small_idx();
        insert_point(&mut idx, 1, 10.0, 10.0);
        insert_point(&mut idx, 2, 50.0, 50.0);
        insert_point(&mut idx, 3, 80.0, 80.0);
        let aabb = Aabb2d { min_x: 0.0, min_y: 0.0, max_x: 60.0, max_y: 60.0 };
        let result = aabb_query_2d(&idx, aabb);
        let ids: Vec<u32> = result.iter().map(|p| p.id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(!ids.contains(&3));
    }

    #[test]
    fn test_point_count() {
        let mut idx = make_small_idx();
        assert_eq!(point_count(&idx), 0);
        insert_point(&mut idx, 1, 5.0, 5.0);
        assert_eq!(point_count(&idx), 1);
    }

    #[test]
    fn test_rebuild_index() {
        let mut idx = make_small_idx();
        insert_point(&mut idx, 1, 10.0, 10.0);
        insert_point(&mut idx, 2, 90.0, 90.0);
        rebuild_index(&mut idx);
        assert!(idx.kdtree.is_some());
        assert_eq!(point_count(&idx), 2);
    }

    #[test]
    fn test_clear_spatial_index() {
        let mut idx = make_small_idx();
        insert_point(&mut idx, 1, 10.0, 10.0);
        insert_point(&mut idx, 2, 20.0, 20.0);
        clear_spatial_index(&mut idx);
        assert_eq!(point_count(&idx), 0);
        assert!(idx.kdtree.is_none());
    }

    #[test]
    fn test_index_bounds_empty() {
        let idx = make_small_idx();
        assert!(index_bounds(&idx).is_none());
    }

    #[test]
    fn test_index_bounds_nonempty() {
        let mut idx = make_small_idx();
        insert_point(&mut idx, 1, 10.0, 20.0);
        insert_point(&mut idx, 2, 80.0, 90.0);
        let b = index_bounds(&idx).unwrap();
        assert!((b.min_x - 10.0).abs() < 1e-5);
        assert!((b.max_y - 90.0).abs() < 1e-5);
    }

    #[test]
    fn test_spatial_index_to_json() {
        let mut idx = make_small_idx();
        insert_point(&mut idx, 1, 10.0, 10.0);
        let json = spatial_index_to_json(&idx);
        assert!(json.contains("point_count"));
        assert!(json.contains('1'));
    }

    #[test]
    fn test_default_config() {
        let cfg = default_spatial_index_config();
        assert!(cfg.grid_cols > 0);
        assert!(cfg.grid_rows > 0);
        assert!(cfg.world_width > 0.0);
        assert!(cfg.world_height > 0.0);
    }

    #[test]
    fn test_no_kdtree_fallback() {
        let cfg = SpatialIndexConfig {
            grid_cols: 5,
            grid_rows: 5,
            world_width: 50.0,
            world_height: 50.0,
            use_kdtree: false,
        };
        let mut idx = new_spatial_index(cfg);
        insert_point(&mut idx, 1, 10.0, 10.0);
        insert_point(&mut idx, 2, 40.0, 40.0);
        rebuild_index(&mut idx);
        assert!(idx.kdtree.is_none());
        let nn = nearest_neighbor(&idx, 12.0, 12.0).unwrap();
        assert_eq!(nn.id, 1);
    }
}
